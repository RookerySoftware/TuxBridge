//! The assistant: a language model that runs on this PC, for questions about TuxBridge, Linux,
//! WSL, Windows and Docker.
//!
//! It runs on Ollama, TuxBridge's own copy (the official Windows build from Ollama's releases)
//! in the assistant's folder next to the systems, apart from any Ollama the person installed:
//! its own port (127.0.0.1:11435, this PC only), its own models folder. The server starts
//! when the assistant is first asked something and ends with the app (a Windows job object
//! ends it even when the app does not close cleanly). Nothing leaves the PC once the model
//! is downloaded.
//!
//! What makes it know TuxBridge is the conversation the window sends (src/assistant): the guides
//! that match the question, a description of the app, and what is on this PC right now.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::containers::{fetch, text, windows_tar};
use crate::jobs::Job;
use crate::wsl;

const ADDRESS: &str = "127.0.0.1:11435";
const RELEASES: &str = "https://github.com/ollama/ollama/releases/latest/download";
/// How much of the conversation the model sees: the instructions, the guides found for the
/// question, and the chat so far.
const CONTEXT: u32 = 16384;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: &'static str,
    /// Ollama's name for it.
    pub tag: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub size_gb: f32,
}

/// Qwen 3.5 (Alibaba, Apache 2.0) in three sizes: it knows Linux, Windows and Docker well for
/// its size and follows instructions closely.
pub const MODELS: [Model; 3] = [
    Model { id: "small", tag: "qwen3.5:2b-q4_K_M", name: "Small", description: "For PCs with little memory. Quick, but makes more mistakes.", size_gb: 1.9 },
    Model { id: "standard", tag: "qwen3.5:4b-q4_K_M", name: "Standard", description: "Good answers on most PCs.", size_gb: 3.3 },
    Model { id: "large", tag: "qwen3.5:9b-q4_K_M", name: "Large", description: "The best answers. Needs a graphics card with 12 GB or more.", size_gb: 6.6 },
];

/// Ollama itself, for NVIDIA, AMD and Intel graphics and the processor alone.
const RUNTIME_GB: f32 = 1.6;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub installed: bool,
    pub running: bool,
    /// The installed model's id ("standard").
    pub model: Option<String>,
    pub models: Vec<Model>,
    /// The size that suits this PC.
    pub recommended: String,
    pub runtime_gb: f32,
    /// What was found: "NVIDIA GeForce RTX 4060, 8 GB" and the memory.
    pub hardware: String,
}

fn exe(dir: &Path) -> PathBuf {
    dir.join("ollama").join("ollama.exe")
}

pub fn model_by_tag(tag: &str) -> Option<&'static Model> {
    MODELS.iter().find(|m| m.tag == tag)
}

/// The installed RAM in GB.
fn memory_gb() -> f32 {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
    unsafe { GlobalMemoryStatusEx(&mut status) }.map(|_| status.ullTotalPhys as f32 / 1_073_741_824.0).unwrap_or(0.0)
}

/// An NVIDIA card's name and memory in GB (nvidia-smi comes with its driver).
async fn nvidia() -> Option<(String, f32)> {
    let mut cmd = tokio::process::Command::new("nvidia-smi");
    cmd.args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"]).creation_flags(wsl::CREATE_NO_WINDOW);
    let out = tokio::time::timeout(Duration::from_secs(10), cmd.output()).await.ok()?.ok()?;
    let line = String::from_utf8_lossy(&out.stdout).lines().next()?.to_string();
    let (name, mb) = line.rsplit_once(',')?;
    Some((name.trim().to_string(), mb.trim().parse::<f32>().ok()? / 1024.0))
}

/// The size for this PC. The large one needs a 12 GB card: on an 8 GB one (RTX 4060) a few of
/// its layers run on the processor and it writes 2 words a second, where the standard one fits
/// whole and writes about 50. Without a graphics card the processor runs the standard one well
/// enough with 12 GB of memory or more.
async fn recommend() -> (String, String) {
    let ram = memory_gb();
    let gpu = nvidia().await;
    let id = match (&gpu, ram) {
        (Some((_, vram)), _) if *vram >= 11.5 => "large",
        (_, ram) if ram >= 12.0 => "standard",
        _ => "small",
    };
    let hardware = match gpu {
        Some((name, vram)) => format!("{name} with {vram:.0} GB, {ram:.0} GB of memory"),
        None => format!("{ram:.0} GB of memory, no NVIDIA graphics card found"),
    };
    (id.to_string(), hardware)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().user_agent("TuxBridge").build().unwrap_or_default()
}

async fn answering() -> bool {
    client().get(format!("http://{ADDRESS}/api/version")).timeout(Duration::from_secs(2)).send().await.is_ok_and(|r| r.status().is_success())
}

pub async fn status(info: Option<crate::store::AiInfo>) -> Status {
    let (recommended, hardware) = recommend().await;
    let installed = info.as_ref().is_some_and(|i| exe(Path::new(&i.dir)).exists());
    Status {
        installed,
        running: installed && answering().await,
        model: info.filter(|_| installed).and_then(|i| model_by_tag(&i.model)).map(|m| m.id.to_string()),
        models: MODELS.to_vec(),
        recommended,
        runtime_gb: RUNTIME_GB,
        hardware,
    }
}

// ------------------------------------------------------------------------------------------
// The server

/// Ollama's server, started by the app.
static SERVER: Mutex<Option<std::process::Child>> = Mutex::new(None);

/// Starts Ollama's server (if it does not answer already) and waits until it does.
pub async fn start(dir: &Path) -> Result<(), String> {
    if answering().await {
        return Ok(());
    }
    {
        let mut server = SERVER.lock().unwrap();
        if let Some(mut old) = server.take() {
            let _ = old.kill();
        }
        use std::os::windows::process::CommandExt;
        let child = std::process::Command::new(exe(dir))
            .arg("serve")
            .env("OLLAMA_HOST", ADDRESS)
            .env("OLLAMA_MODELS", dir.join("models"))
            // The model leaves the graphics card's memory 10 minutes after the last question.
            .env("OLLAMA_KEEP_ALIVE", "10m")
            .env("OLLAMA_CONTEXT_LENGTH", CONTEXT.to_string())
            .env("OLLAMA_NO_CLOUD", "1")
            .env_remove("OLLAMA_ORIGINS")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::fs::File::create(dir.join("server.log")).map(std::process::Stdio::from).unwrap_or_else(|_| std::process::Stdio::null()))
            .creation_flags(wsl::CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("The assistant did not start: {e}"))?;
        crate::win::end_with_app(&child);
        *server = Some(child);
    }
    for _ in 0..60 {
        if answering().await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Err("The assistant did not start. Its log is server.log in its folder.".into())
}

/// Loads the model into memory before the first question, so that one is answered at once:
/// the first load after starting takes up to a minute (Windows checks the files on first use).
pub async fn warm_up(dir: &Path, model: &str) {
    if start(dir).await.is_err() {
        return;
    }
    let body = serde_json::json!({
        "model": model,
        "messages": [{ "role": "user", "content": "Hi" }],
        "stream": false,
        "think": false,
        "options": { "num_ctx": CONTEXT, "num_predict": 1 },
    });
    let _ = client().post(format!("http://{ADDRESS}/api/chat")).json(&body).timeout(Duration::from_secs(300)).send().await;
}

pub fn stop() {
    if let Some(mut child) = SERVER.lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

// ------------------------------------------------------------------------------------------
// Installing and removing

/// Ollama's Windows build and its checksum, from Ollama's newest release.
async fn runtime(job: &Job, dir: &Path) -> Result<(), String> {
    const ZIP: &str = "ollama-windows-amd64.zip";
    let sums = text(&format!("{RELEASES}/sha256sum.txt")).await.map_err(|e| format!("Ollama's download list did not load: {e}"))?;
    let expected = sums
        .lines()
        .find_map(|l| l.strip_suffix(&format!("./{ZIP}")).map(|h| h.trim().to_string()))
        .ok_or("Ollama's download list has no Windows build.")?;
    let zip = dir.join(ZIP);
    let hash = fetch(job, &format!("{RELEASES}/{ZIP}"), &zip).await.map_err(|e| {
        let _ = std::fs::remove_file(&zip);
        if e.to_string().contains("os error 112") {
            "The drive is full. The assistant needs about 6 GB free.".to_string()
        } else {
            format!("Downloading Ollama did not work ({e}). Check the internet connection and try again.")
        }
    })?;
    if hash != expected {
        let _ = std::fs::remove_file(&zip);
        return Err("The download was damaged (its checksum is wrong). Try again.".into());
    }
    job.step("Unpacking it");
    let target = dir.join("ollama");
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    let mut tar = tokio::process::Command::new(windows_tar());
    tar.arg("-xf").arg(&zip).arg("-C").arg(&target).creation_flags(wsl::CREATE_NO_WINDOW);
    let result = job.run(tar, None).await;
    let _ = std::fs::remove_file(&zip);
    result.map_err(|_| "Unpacking Ollama did not work. Try again.".to_string())?;
    if !exe(dir).exists() {
        return Err("Ollama's download did not contain ollama.exe.".into());
    }
    Ok(())
}

/// Ollama's own command, pointed at the assistant's server.
fn ollama(dir: &Path) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(exe(dir));
    cmd.env("OLLAMA_HOST", ADDRESS).creation_flags(wsl::CREATE_NO_WINDOW).kill_on_drop(true);
    cmd
}

/// Downloads a model through the server, showing its progress.
async fn pull(job: &Job, tag: &str) -> Result<(), String> {
    let failed = |e: String| {
        job.log_line(&e);
        if e.contains("space") {
            "The drive is full. Free some space and try again.".to_string()
        } else {
            "Downloading the model did not work. Check the internet connection and try again.".to_string()
        }
    };
    let response = client()
        .post(format!("http://{ADDRESS}/api/pull"))
        .json(&serde_json::json!({ "model": tag, "stream": true }))
        .send()
        .await
        .map_err(|e| failed(e.to_string()))?;
    let mut stream = response.bytes_stream();
    let (mut pending, mut shown) = (Vec::new(), String::new());
    while let Some(bytes) = stream.next().await {
        pending.extend_from_slice(&bytes.map_err(|e| failed(e.to_string()))?);
        while let Some(end) = pending.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = pending.drain(..=end).collect();
            let Ok(v) = serde_json::from_slice::<serde_json::Value>(&line) else { continue };
            if let Some(e) = v["error"].as_str() {
                return Err(failed(e.to_string()));
            }
            let status = v["status"].as_str().unwrap_or("");
            // The big part's progress in whole percent; other steps by name.
            let text = match (v["completed"].as_u64(), v["total"].as_u64()) {
                (Some(done), Some(total)) if total > 100_000_000 => {
                    format!("{}% of {:.1} GB", done * 100 / total, total as f64 / 1e9)
                }
                (_, Some(_)) => continue,
                _ => status.to_string(),
            };
            if text != shown {
                job.say(&text);
                shown = text;
            }
        }
    }
    if shown != "success" {
        return Err(failed(format!("the download ended at: {shown}")));
    }
    Ok(())
}

/// Installs the assistant with this model into `dir`, or changes its model (Ollama is kept).
pub async fn install(job: Arc<Job>, dir: PathBuf, model: &'static Model, old: Option<String>) -> Result<(), String> {
    let downloading = format!("Downloading the {} model ({} GB)", model.name.to_lowercase(), model.size_gb);
    let have_runtime = exe(&dir).exists();
    let mut plan: Vec<(&str, Option<&str>)> = Vec::new();
    if !have_runtime {
        plan.push(("Downloading Ollama (1.6 GB)", Some("a few minutes")));
        plan.push(("Unpacking it", None));
    }
    plan.push((&downloading, Some("a few minutes")));
    job.plan(&plan);
    std::fs::create_dir_all(&dir).map_err(|e| format!("The folder {} could not be made: {e}", dir.display()))?;

    if !have_runtime {
        job.step("Downloading Ollama (1.6 GB)");
        runtime(&job, &dir).await?;
    }
    job.step(&downloading);
    start(&dir).await?;
    pull(&job, model.tag).await?;
    if let Some(old) = old.filter(|o| o != model.tag) {
        let mut cmd = ollama(&dir);
        cmd.args(["rm", &old]);
        let _ = job.run(cmd, None).await;
    }
    Ok(())
}

/// Removes the assistant: Ollama and the model.
pub fn remove(dir: &Path) -> Result<(), String> {
    stop();
    // Ollama's runners can take a moment to let go of their files.
    for _ in 0..10 {
        match std::fs::remove_dir_all(dir) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => std::thread::sleep(Duration::from_millis(500)),
        }
    }
    std::fs::remove_dir_all(dir).map_err(|e| format!("The assistant's folder {} could not be removed: {e}", dir.display()))
}

// ------------------------------------------------------------------------------------------
// Chatting

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// "system", "user" or "assistant".
    pub role: String,
    pub content: String,
}

/// A piece of an answer, sent to the window as it is written.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Chunk {
    id: u64,
    text: String,
    done: bool,
    error: Option<String>,
}

static STOPS: Mutex<Option<HashMap<u64, Arc<AtomicBool>>>> = Mutex::new(None);

pub fn cancel(id: u64) {
    if let Some(flag) = STOPS.lock().unwrap().as_ref().and_then(|m| m.get(&id)) {
        flag.store(true, Ordering::SeqCst);
    }
}

/// Answers the conversation, sending the answer as "ai-chunk" events with this id.
pub async fn chat(app: AppHandle, dir: PathBuf, model: String, id: u64, messages: Vec<Message>) {
    let stop = Arc::new(AtomicBool::new(false));
    STOPS.lock().unwrap().get_or_insert_with(HashMap::new).insert(id, stop.clone());
    let send = |text: String, done: bool, error: Option<String>| {
        let _ = app.emit("ai-chunk", Chunk { id, text, done, error });
    };
    let result = answer(&dir, &model, &messages, &stop, |t| send(t, false, None)).await;
    match result {
        Ok(()) => send(String::new(), true, None),
        Err(e) => send(String::new(), true, Some(e)),
    }
    if let Some(m) = STOPS.lock().unwrap().as_mut() {
        m.remove(&id);
    }
}

async fn answer(dir: &Path, model: &str, messages: &[Message], stop: &AtomicBool, mut on_text: impl FnMut(String)) -> Result<(), String> {
    start(dir).await?;
    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        // Thinking first makes answers slower to start than a person waits for here.
        "think": false,
        "options": { "num_ctx": CONTEXT, "temperature": 0.3 },
    });
    let response = client()
        .post(format!("http://{ADDRESS}/api/chat"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("The assistant did not answer: {e}"))?;
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("The assistant could not answer: {}", text.trim()));
    }
    let mut stream = response.bytes_stream();
    let mut pending = Vec::new();
    while let Some(bytes) = stream.next().await {
        if stop.load(Ordering::SeqCst) {
            return Ok(());
        }
        pending.extend_from_slice(&bytes.map_err(|e| e.to_string())?);
        while let Some(end) = pending.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = pending.drain(..=end).collect();
            let Ok(v) = serde_json::from_slice::<serde_json::Value>(&line) else { continue };
            if let Some(e) = v["error"].as_str() {
                return Err(e.to_string());
            }
            if let Some(t) = v["message"]["content"].as_str().filter(|t| !t.is_empty()) {
                on_text(t.to_string());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) async fn answer_for_test(dir: &Path, model: &str, messages: &[Message]) -> Result<String, String> {
    let mut text = String::new();
    answer(dir, model, messages, &AtomicBool::new(false), |t| text.push_str(&t)).await?;
    Ok(text)
}
