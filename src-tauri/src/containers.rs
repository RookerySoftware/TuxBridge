//! Docker: EasyWSL's own engine, the `docker` command for Windows, and the containers.
//!
//! The engine is the free Docker Engine in a small Alpine Linux system of its own
//! (easywsl-docker, set up by linux/docker-engine.sh). It listens on 127.0.0.1:2376, which WSL
//! forwards to Windows, and takes only clients with a certificate from an authority made for
//! this PC when Docker is turned on (the authority's key is thrown away at once). The client's
//! certificate stays in the user's profile, so only this Windows account reaches the engine.
//!
//! Docker Desktop, when installed and running, is shown next to it: its containers can be
//! managed here too.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::jobs::Job;
use crate::{paths, wsl};

/// The engine's own system, hidden from My systems.
pub const DISTRO: &str = "easywsl-docker";
const ADDRESS: &str = "tcp://127.0.0.1:2376";
const DESKTOP_PIPE: &str = r"\\.\pipe\dockerDesktopLinuxEngine";

/// %LOCALAPPDATA%\EasyWSL\docker: the client's TLS files (tls\) and the docker command's own
/// settings for the app's calls (config\), apart from the user's.
fn home() -> PathBuf {
    paths::data_dir().join("docker")
}

fn tls() -> PathBuf {
    home().join("tls")
}

/// The folder EasyWSL's docker.exe is in; added to the user's PATH when no other docker is.
pub fn bin_dir() -> PathBuf {
    paths::data_dir().join("bin")
}

fn cli() -> PathBuf {
    bin_dir().join("docker.exe")
}

/// Which engine a container runs on.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Engine {
    Easywsl,
    Desktop,
}

/// docker, talking to an engine, without a console window.
fn docker(engine: Engine) -> Command {
    let mut cmd = Command::new(cli());
    cmd.env("DOCKER_CONFIG", home().join("config"))
        .env("DOCKER_CLI_HINTS", "false")
        .creation_flags(wsl::CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .kill_on_drop(true);
    match engine {
        Engine::Easywsl => {
            let t = tls();
            cmd.args(["-H", ADDRESS, "--tlsverify"])
                .arg("--tlscacert")
                .arg(t.join("ca.pem"))
                .arg("--tlscert")
                .arg(t.join("cert.pem"))
                .arg("--tlskey")
                .arg(t.join("key.pem"));
        }
        Engine::Desktop => {
            cmd.args(["-H", "npipe:////./pipe/dockerDesktopLinuxEngine"]);
        }
    }
    cmd
}

/// Runs docker and returns what it printed; fails with its error message.
async fn run(engine: Engine, args: &[&str], limit: Duration) -> Result<String, String> {
    let mut cmd = docker(engine);
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    let out = tokio::time::timeout(limit, cmd.output())
        .await
        .map_err(|_| "Docker did not answer in time.".to_string())?
        .map_err(|e| format!("docker did not run: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(err.strip_prefix("Error response from daemon: ").map(String::from).unwrap_or(err))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// EasyWSL's engine is set up (its system, the docker command and the certificates).
    pub installed: bool,
    pub running: bool,
    pub version: Option<String>,
    pub desktop_installed: bool,
    pub desktop_running: bool,
}

fn desktop_installed() -> bool {
    let program_files = std::env::var_os("ProgramFiles").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
    program_files.join(r"Docker\Docker\Docker Desktop.exe").exists() || wsl::exists("docker-desktop")
}

pub fn installed() -> bool {
    wsl::exists(DISTRO) && cli().exists() && tls().join("key.pem").exists()
}

pub async fn status() -> Status {
    let installed = installed();
    let version = if installed {
        run(Engine::Easywsl, &["version", "--format", "{{.Server.Version}}"], Duration::from_secs(8))
            .await
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    } else {
        None
    };
    let desktop_running = Path::new(DESKTOP_PIPE).exists() && cli().exists();
    Status { installed, running: version.is_some(), version, desktop_installed: desktop_installed(), desktop_running }
}

/// Starts the engine: wsl.exe running it, detached from the app so Docker keeps running when
/// the app closes (the running wsl.exe also keeps the system from being stopped as idle).
pub fn start() -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
    let spawn = |flags: u32| {
        std::process::Command::new(wsl::exe())
            .args(["-d", DISTRO, "-u", "root", "-e", "easywsl-docker", "run"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(flags)
            .spawn()
    };
    // Out of the app's job object where Windows allows it, or a terminal closing the app would
    // take Docker along.
    spawn(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_BREAKAWAY_FROM_JOB)
        .or_else(|_| spawn(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP))
        .map(|_| ())
        .map_err(|e| format!("Docker did not start: {e}"))
}

/// Waits until the engine answers.
pub async fn wait_ready(limit: Duration) -> bool {
    let deadline = tokio::time::Instant::now() + limit;
    while tokio::time::Instant::now() < deadline {
        if run(Engine::Easywsl, &["version", "--format", "{{.Server.Version}}"], Duration::from_secs(5)).await.is_ok() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(700)).await;
    }
    false
}

/// Stops the engine (its containers first) and the system it runs in.
pub async fn stop() -> Result<(), String> {
    let out = wsl::output_within(["-d", DISTRO, "-u", "root", "-e", "easywsl-docker", "stop"], Duration::from_secs(60)).await;
    wsl::terminate(DISTRO).await;
    match out {
        Ok(o) if o.code != 0 => Err(format!("Docker did not stop cleanly: {}", o.text.trim())),
        Err(e) => Err(format!("Docker did not stop cleanly: {e}")),
        _ => Ok(()),
    }
}

// ------------------------------------------------------------------------------------------
// Turning Docker on

/// Downloads `url` to `path`, logging progress, and returns the file's SHA-256 (hex).
async fn fetch(job: &Job, url: &str, path: &Path) -> anyhow::Result<String> {
    let client = reqwest::Client::builder().user_agent("EasyWSL").build()?;
    let response = client.get(url).send().await?.error_for_status()?;
    let total = response.content_length().unwrap_or(0);
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    let mut file = tokio::fs::File::create(path).await?;
    let mut hasher = Sha256::new();
    let mut stream = response.bytes_stream();
    let (mut done, mut last) = (0u64, u64::MAX);
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
        done += chunk.len() as u64;
        let percent = if total > 0 { done * 100 / total } else { 0 };
        if percent / 10 != last {
            last = percent / 10;
            job.log_line(&format!("downloaded {percent}% of {} MB", total / 1_000_000));
        }
    }
    file.flush().await?;
    let hash = hex::encode(hasher.finalize());
    job.log_line(&format!("{url}: sha256 {hash}"));
    Ok(hash)
}

async fn text(url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder().user_agent("EasyWSL").build()?;
    Ok(client.get(url).send().await?.error_for_status()?.text().await?)
}

/// Alpine Linux's newest minimal root filesystem: its address and SHA-256, from Alpine's own
/// list of releases.
async fn alpine() -> anyhow::Result<(String, String)> {
    const BASE: &str = "https://dl-cdn.alpinelinux.org/alpine/latest-stable/releases/x86_64";
    let list = text(&format!("{BASE}/latest-releases.yaml")).await?;
    let entry = list
        .split("\n-")
        .find(|e| e.contains("flavor: alpine-minirootfs"))
        .ok_or_else(|| anyhow::anyhow!("Alpine's release list has no minimal root filesystem"))?;
    let field = |name: &str| entry.lines().find_map(|l| l.trim().strip_prefix(name).map(|v| v.trim().to_string()));
    let file = field("file:").ok_or_else(|| anyhow::anyhow!("no file in Alpine's release list"))?;
    let sha = field("sha256:").ok_or_else(|| anyhow::anyhow!("no checksum in Alpine's release list"))?;
    Ok((format!("{BASE}/{file}"), sha))
}

/// The newest docker command for Windows, from Docker's own downloads.
async fn cli_zip() -> anyhow::Result<String> {
    const BASE: &str = "https://download.docker.com/win/static/stable/x86_64/";
    let listing = text(BASE).await?;
    let version = |name: &str| -> Option<Vec<u32>> {
        name.strip_prefix("docker-")?.strip_suffix(".zip")?.split('.').map(|p| p.parse().ok()).collect()
    };
    let newest = listing
        .split('"')
        .filter(|s| s.starts_with("docker-") && s.ends_with(".zip"))
        .filter_map(|s| version(s).map(|v| (v, s.to_string())))
        .max()
        .ok_or_else(|| anyhow::anyhow!("no docker download found"))?;
    Ok(format!("{BASE}{}", newest.1))
}

/// Windows' own tar (it reads zip files too).
fn windows_tar() -> PathBuf {
    std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows")).join(r"System32\tar.exe")
}

/// A certificate authority for this PC, the engine's certificate (for 127.0.0.1 and localhost)
/// and the app's; the authority's key is not kept, so no other certificate can be made.
fn make_certificates(server_dir: &Path, client_dir: &Path) -> anyhow::Result<()> {
    use rcgen::{BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer, KeyPair, KeyUsagePurpose, SanType};
    let ca_key = KeyPair::generate()?;
    let mut ca = CertificateParams::new(Vec::<String>::new())?;
    ca.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign, KeyUsagePurpose::DigitalSignature];
    ca.distinguished_name.push(DnType::CommonName, "EasyWSL Docker engine");
    let ca_cert = ca.self_signed(&ca_key)?;
    let issuer = Issuer::new(ca, ca_key);

    let server_key = KeyPair::generate()?;
    let mut server = CertificateParams::new(vec!["localhost".to_string()])?;
    server.subject_alt_names.push(SanType::IpAddress("127.0.0.1".parse()?));
    server.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    server.distinguished_name.push(DnType::CommonName, "localhost");
    let server_cert = server.signed_by(&server_key, &issuer)?;

    let client_key = KeyPair::generate()?;
    let mut client = CertificateParams::new(Vec::<String>::new())?;
    client.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
    client.distinguished_name.push(DnType::CommonName, "EasyWSL");
    let client_cert = client.signed_by(&client_key, &issuer)?;

    std::fs::create_dir_all(server_dir)?;
    std::fs::write(server_dir.join("ca.pem"), ca_cert.pem())?;
    std::fs::write(server_dir.join("server-cert.pem"), server_cert.pem())?;
    std::fs::write(server_dir.join("server-key.pem"), server_key.serialize_pem())?;
    std::fs::create_dir_all(client_dir)?;
    std::fs::write(client_dir.join("ca.pem"), ca_cert.pem())?;
    std::fs::write(client_dir.join("cert.pem"), client_cert.pem())?;
    std::fs::write(client_dir.join("key.pem"), client_key.serialize_pem())?;
    Ok(())
}

/// Adds a folder to the user's PATH (for new terminals) unless it is there already.
fn add_to_path(dir: &Path) -> anyhow::Result<()> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, RegType};
    let env = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags("Environment", KEY_READ | KEY_SET_VALUE)?;
    let current: String = env.get_value("Path").unwrap_or_default();
    let dir = dir.to_string_lossy();
    if current.split(';').any(|p| p.trim_end_matches('\\').eq_ignore_ascii_case(dir.trim_end_matches('\\'))) {
        return Ok(());
    }
    let new = if current.is_empty() || current.ends_with(';') { format!("{current}{dir}") } else { format!("{current};{dir}") };
    // REG_EXPAND_SZ, as Windows keeps it (entries may use %USERPROFILE% and the like).
    let value = winreg::RegValue { bytes: new.encode_utf16().chain([0]).flat_map(|u| u.to_le_bytes()).collect(), vtype: RegType::REG_EXPAND_SZ };
    env.set_raw_value("Path", &value)?;
    // Tells Explorer, so terminals opened from now on see it.
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE};
        let name: Vec<u16> = "Environment\0".encode_utf16().collect();
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            windows::Win32::Foundation::WPARAM(0),
            windows::Win32::Foundation::LPARAM(name.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            2000,
            None,
        );
    }
    Ok(())
}

/// The user's own docker settings: a context named "easywsl" for this engine, and the one in
/// use unless Docker Desktop is installed (whose context the user may rely on).
async fn set_up_context(job: &Job) {
    let t = tls();
    let spec = format!(
        "host={ADDRESS},ca={},cert={},key={}",
        t.join("ca.pem").display(),
        t.join("cert.pem").display(),
        t.join("key.pem").display()
    );
    let user_docker = |args: &[&str]| {
        let mut cmd = Command::new(cli());
        cmd.args(args).env_remove("DOCKER_CONFIG").creation_flags(wsl::CREATE_NO_WINDOW);
        cmd
    };
    let exists = user_docker(&["context", "inspect", "easywsl"]).stdout(Stdio::null()).stderr(Stdio::null()).status().await.is_ok_and(|s| s.success());
    let verb = if exists { "update" } else { "create" };
    let _ = job.run(user_docker(&["context", verb, "easywsl", "--description", "EasyWSL's Docker engine", "--docker", &spec]), None).await;
    if !desktop_installed() {
        let _ = job.run(user_docker(&["context", "use", "easywsl"]), None).await;
    }
}

const NET: &str = "Check the internet connection and try again.";

/// Turns Docker on: the engine's system, Docker Engine in it, the docker command, and a first
/// start. `location` is the folder for the system's disk.
pub async fn enable(job: Arc<Job>, location: PathBuf, linux: String) -> Result<(), String> {
    job.plan(&[
        ("Downloading Alpine Linux", Some("under a minute")),
        ("Installing Docker Engine", Some("1 to 2 minutes")),
        ("Getting the docker command", Some("about a minute")),
        ("Starting Docker", None),
    ]);
    let downloads = paths::downloads_dir().join("docker");
    let _ = std::fs::remove_dir_all(&downloads);

    job.step("Downloading Alpine Linux");
    if !wsl::exists(DISTRO) {
        let (url, sha) = alpine().await.map_err(|e| {
            job.log_line(&format!("{e:#}"));
            format!("Finding Alpine Linux did not work. {NET}")
        })?;
        let rootfs = downloads.join("alpine.tar.gz");
        let got = fetch(&job, &url, &rootfs).await.map_err(|e| {
            job.log_line(&format!("{e:#}"));
            format!("Downloading Alpine Linux did not work. {NET}")
        })?;
        if got != sha {
            return Err("The download arrived damaged. Try again.".into());
        }
        std::fs::create_dir_all(&location).map_err(|e| format!("The folder {} cannot be created: {e}", location.display()))?;
        job.run(wsl::command(["--import".as_ref(), DISTRO.as_ref(), location.as_os_str(), rootfs.as_os_str()]), None)
            .await
            .map_err(|_| "Creating Docker's system did not work. Check that the drive has a few GB free and try again.".to_string())?;
    }

    job.step("Installing Docker Engine");
    let server = home().join("server");
    let result: Result<(), String> = async {
        make_certificates(&server, &tls()).map_err(|e| format!("Making the engine's certificates did not work: {e}"))?;
        let server_wsl = paths::to_wsl(&server).map_err(|e| e.to_string())?;
        job.run(wsl::command(["-d", DISTRO, "-u", "root", "-e", "sh", &format!("{linux}/docker-engine.sh"), "setup", &server_wsl]), None)
            .await
            .map_err(|_| format!("Installing Docker Engine did not work. {NET}"))
    }
    .await;
    // The engine's key lives in its system only.
    let _ = std::fs::remove_dir_all(&server);
    result?;
    // The system restarts with its new settings (wsl.conf) at the first start.
    wsl::terminate(DISTRO).await;

    job.step("Getting the docker command");
    if !cli().exists() {
        let url = cli_zip().await.map_err(|e| {
            job.log_line(&format!("{e:#}"));
            format!("Finding the docker command did not work. {NET}")
        })?;
        let zip = downloads.join("docker.zip");
        fetch(&job, &url, &zip).await.map_err(|e| {
            job.log_line(&format!("{e:#}"));
            format!("Downloading the docker command did not work. {NET}")
        })?;
        let unpacked = downloads.join("cli");
        std::fs::create_dir_all(&unpacked).map_err(|e| e.to_string())?;
        let mut cmd = Command::new(windows_tar());
        cmd.creation_flags(wsl::CREATE_NO_WINDOW);
        cmd.arg("-xf").arg(&zip).arg("-C").arg(&unpacked).arg("docker/docker.exe");
        job.run(cmd, None).await.map_err(|_| "Unpacking the docker command did not work. Try again.".to_string())?;
        std::fs::create_dir_all(bin_dir()).map_err(|e| e.to_string())?;
        std::fs::rename(unpacked.join("docker").join("docker.exe"), cli()).map_err(|e| format!("Placing the docker command did not work: {e}"))?;
    }
    // Docker Compose, which most guides use, unless the user has it already.
    if let Some(plugins) = dirs::home_dir().map(|h| h.join(r".docker\cli-plugins")) {
        let compose = plugins.join("docker-compose.exe");
        if !compose.exists() {
            const URL: &str = "https://github.com/docker/compose/releases/latest/download/docker-compose-windows-x86_64.exe";
            let partial = downloads.join("docker-compose.exe");
            let fetched = async {
                let expected = text(&format!("{URL}.sha256")).await?;
                let got = fetch(&job, URL, &partial).await?;
                anyhow::ensure!(expected.split_whitespace().next() == Some(got.as_str()), "docker compose arrived damaged");
                std::fs::create_dir_all(&plugins)?;
                std::fs::rename(&partial, &compose)?;
                anyhow::Ok(())
            }
            .await;
            // Docker works without it; the log says why it is missing.
            if let Err(e) = fetched {
                job.log_line(&format!("docker compose was not installed: {e:#}"));
            }
        }
    }
    if !desktop_installed() {
        if let Err(e) = add_to_path(&bin_dir()) {
            job.log_line(&format!("adding docker to the PATH did not work: {e:#}"));
        }
    }

    job.step("Starting Docker");
    start()?;
    if !wait_ready(Duration::from_secs(60)).await {
        let log = wsl::output(["-d", DISTRO, "-u", "root", "-e", "tail", "-n", "20", "/var/log/dockerd.log"]).await;
        if let Ok(log) = log {
            job.log_line(&log.text);
        }
        return Err("Docker did not start. Try again; if it fails again, please report it with the log file.".into());
    }
    set_up_context(&job).await;
    let _ = std::fs::remove_dir_all(&downloads);
    Ok(())
}

/// Removes the engine with everything in it (containers, images, volumes). The docker command
/// stays, for Docker Desktop or a later engine.
pub async fn remove() -> Result<(), String> {
    let _ = stop().await;
    if wsl::exists(DISTRO) {
        let base = wsl::registered().into_iter().find(|d| d.name == DISTRO).map(|d| d.base_path);
        let out = wsl::output_within(["--unregister", DISTRO], Duration::from_secs(120)).await.map_err(|e| e.to_string())?;
        if out.code != 0 {
            return Err(format!("Removing Docker did not work: {}", out.text.trim()));
        }
        if let Some(base) = base {
            let _ = std::fs::remove_dir(base);
        }
    }
    let _ = std::fs::remove_dir_all(tls());
    if cli().exists() {
        let mut cmd = Command::new(cli());
        cmd.args(["context", "rm", "-f", "easywsl"]).creation_flags(wsl::CREATE_NO_WINDOW).stdout(Stdio::null()).stderr(Stdio::null());
        let _ = cmd.status().await;
    }
    Ok(())
}

// ------------------------------------------------------------------------------------------
// docker in the user's distros (linux/docker-client.sh)

/// Static Linux builds of docker and docker compose, downloaded once for every distro.
async fn linux_client(job: &Job) -> Result<PathBuf, String> {
    let dir = home().join("linux");
    if dir.join("docker").exists() {
        return Ok(dir);
    }
    let downloads = paths::downloads_dir().join("docker-linux");
    let _ = std::fs::remove_dir_all(&downloads);
    let result: anyhow::Result<()> = async {
        // The same version as the Windows command, from Docker's own downloads.
        let windows = cli_zip().await?;
        let version = windows.rsplit('/').next().unwrap_or_default().trim_end_matches(".zip").to_string();
        let tgz = downloads.join("docker.tgz");
        fetch(job, &format!("https://download.docker.com/linux/static/stable/x86_64/{version}.tgz"), &tgz).await?;
        let mut cmd = Command::new(windows_tar());
        cmd.creation_flags(wsl::CREATE_NO_WINDOW).arg("-xzf").arg(&tgz).arg("-C").arg(&downloads).arg("docker/docker");
        job.run(cmd, None).await?;
        const COMPOSE: &str = "https://github.com/docker/compose/releases/latest/download/docker-compose-linux-x86_64";
        let expected = text(&format!("{COMPOSE}.sha256")).await?;
        let compose = downloads.join("docker-compose");
        let got = fetch(job, COMPOSE, &compose).await?;
        anyhow::ensure!(expected.split_whitespace().next() == Some(got.as_str()), "docker compose arrived damaged");
        let partial = dir.with_extension("partial");
        let _ = std::fs::remove_dir_all(&partial);
        std::fs::create_dir_all(&partial)?;
        std::fs::rename(downloads.join("docker").join("docker"), partial.join("docker"))?;
        std::fs::rename(&compose, partial.join("docker-compose"))?;
        std::fs::rename(&partial, &dir)?;
        Ok(())
    }
    .await;
    let _ = std::fs::remove_dir_all(&downloads);
    result.map_err(|e| {
        job.log_line(&format!("{e:#}"));
        format!("Downloading docker for Linux did not work. {NET}")
    })?;
    Ok(dir)
}

/// Makes `docker` in a distro use this engine.
pub async fn use_in(job: Arc<Job>, distro: &str, user: &str, linux: &str) -> Result<(), String> {
    let get = "Getting docker for Linux";
    let set = format!("Setting it up in {distro}");
    job.plan(&[(get, Some("under a minute")), (&set, None)]);
    job.step(get);
    let client = linux_client(&job).await?;

    job.step(&set);
    // The files the script takes, in one folder; the key's copy goes again at once.
    let staging = home().join(format!("client-{distro}"));
    let _ = std::fs::remove_dir_all(&staging);
    let staged: std::io::Result<()> = (|| {
        std::fs::create_dir_all(&staging)?;
        for (from, name) in [
            (client.join("docker"), "docker"),
            (client.join("docker-compose"), "docker-compose"),
            (tls().join("ca.pem"), "ca.pem"),
            (tls().join("cert.pem"), "cert.pem"),
            (tls().join("key.pem"), "key.pem"),
        ] {
            std::fs::copy(from, staging.join(name))?;
        }
        Ok(())
    })();
    let result = match staged {
        Err(e) => Err(format!("Preparing the files did not work: {e}")),
        Ok(()) => {
            let files = paths::to_wsl(&staging).map_err(|e| e.to_string())?;
            job.run(wsl::command(["-d", distro, "-u", "root", "-e", "sh", &format!("{linux}/docker-client.sh"), "install", &files, user]), None)
                .await
                .map_err(|e| {
                    if e.to_string().contains("exit code 3") {
                        format!("{distro} has a docker of its own, which was left as it is.")
                    } else {
                        format!("Setting up docker in {distro} did not work. Try again; if it fails again, please report it with the log file.")
                    }
                })
        }
    };
    let _ = std::fs::remove_dir_all(&staging);
    result
}

// ------------------------------------------------------------------------------------------
// Containers

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub engine: Engine,
    pub id: String,
    pub name: String,
    pub image: String,
    /// "running", "exited", "created", "restarting", "paused", "dead"
    pub state: String,
    /// Docker's own words: "Up 2 hours", "Exited (0) 3 days ago".
    pub status: String,
    /// Ports published on the PC (TCP), lowest first.
    pub ports: Vec<u16>,
    pub created: String,
    /// The app from the gallery it was installed as.
    pub app: Option<String>,
}

/// "0.0.0.0:8089->80/tcp, [::]:8089->80/tcp, 53/udp" -> [8089]
fn published_ports(text: &str) -> Vec<u16> {
    let mut ports: Vec<u16> = text
        .split(", ")
        .filter_map(|p| {
            let (host, target) = p.split_once("->")?;
            target.ends_with("/tcp").then_some(())?;
            host.rsplit(':').next()?.split('-').next()?.parse().ok()
        })
        .collect();
    ports.sort_unstable();
    ports.dedup();
    ports
}

fn label(labels: &str, key: &str) -> Option<String> {
    labels.split(',').find_map(|kv| kv.strip_prefix(key)?.strip_prefix('=').map(String::from))
}

async fn containers_of(engine: Engine) -> Result<Vec<Container>, String> {
    let out = run(engine, &["ps", "-a", "--no-trunc", "--format", "{{json .}}"], Duration::from_secs(15)).await?;
    Ok(out
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .map(|v| {
            let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
            Container {
                engine,
                id: s("ID"),
                name: s("Names"),
                image: s("Image"),
                state: s("State"),
                status: s("Status"),
                ports: published_ports(&s("Ports")),
                created: s("CreatedAt"),
                app: label(&s("Labels"), "easywsl.app"),
            }
        })
        .collect())
}

/// The containers of every running engine: EasyWSL's, and Docker Desktop's.
pub async fn containers() -> Vec<Container> {
    let mut all = Vec::new();
    if installed() {
        all.extend(containers_of(Engine::Easywsl).await.unwrap_or_default());
    }
    if Path::new(DESKTOP_PIPE).exists() && cli().exists() {
        all.extend(containers_of(Engine::Desktop).await.unwrap_or_default());
    }
    all.sort_by(|a, b| (a.state != "running", a.name.to_lowercase()).cmp(&(b.state != "running", b.name.to_lowercase())));
    all
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && id.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c))
}

pub async fn act(engine: Engine, id: &str, action: &str) -> Result<(), String> {
    if !valid_id(id) {
        return Err("That is not a container.".into());
    }
    let args: &[&str] = match action {
        "start" => &["start", id],
        "stop" => &["stop", id],
        "restart" => &["restart", id],
        // Its named volumes (an app's data) stay; Clean up does not remove them either.
        "remove" => &["rm", "-f", id],
        _ => return Err(format!("unknown action {action}")),
    };
    run(engine, args, Duration::from_secs(90)).await.map(|_| ())
}

pub async fn logs(engine: Engine, id: &str) -> Result<String, String> {
    if !valid_id(id) {
        return Err("That is not a container.".into());
    }
    let mut cmd = docker(engine);
    cmd.args(["logs", "--tail", "400", id]).stdout(Stdio::piped()).stderr(Stdio::piped());
    let out = tokio::time::timeout(Duration::from_secs(15), cmd.output())
        .await
        .map_err(|_| "Docker did not answer in time.".to_string())?
        .map_err(|e| e.to_string())?;
    // Containers write to both; the order between the two is lost, which is fine for reading.
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok(text)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskUse {
    /// "Images", "Containers", "Local Volumes", "Build Cache"
    pub kind: String,
    pub count: String,
    pub size: String,
    pub reclaimable: String,
}

pub async fn disk_use() -> Result<Vec<DiskUse>, String> {
    let out = run(Engine::Easywsl, &["system", "df", "--format", "{{json .}}"], Duration::from_secs(30)).await?;
    Ok(out
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .map(|v| {
            let s = |k: &str| v.get(k).map(|x| x.as_str().map(String::from).unwrap_or_else(|| x.to_string())).unwrap_or_default();
            DiskUse { kind: s("Type"), count: s("TotalCount"), size: s("Size"), reclaimable: s("Reclaimable") }
        })
        .collect())
}

/// Removes stopped containers, unused networks, build cache and images no container uses
/// (all unused ones with `images`). Volumes, which hold apps' data, are kept.
pub async fn clean_up(images: bool) -> Result<String, String> {
    let mut args = vec!["system", "prune", "-f"];
    if images {
        args.push("-a");
    }
    let out = run(Engine::Easywsl, &args, Duration::from_secs(300)).await?;
    Ok(out.lines().rev().find(|l| l.starts_with("Total reclaimed space")).unwrap_or("Nothing to clean up.").to_string())
}

// ------------------------------------------------------------------------------------------
// Apps from the gallery (apps.rs)

/// Whether a port on the PC is free: nothing listens on it, Windows' or WSL's forwarding.
fn port_free(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// The first free port from `wanted` on.
fn free_port(wanted: u16, taken: &[u16]) -> u16 {
    (wanted..wanted.saturating_add(100)).find(|p| !taken.contains(p) && port_free(*p)).unwrap_or(wanted)
}

pub async fn install_app(job: Arc<Job>, app: &'static crate::apps::App, answers: std::collections::HashMap<String, String>) -> Result<(), String> {
    use crate::apps::Kind;
    let pull = format!("Downloading {}", app.name);
    job.plan(&[(&pull, Some(app.download)), ("Starting it", None)]);

    job.step(&pull);
    let mut cmd = docker(Engine::Easywsl);
    cmd.args(["pull", app.image]);
    job.run(cmd, None).await.map_err(|_| {
        let tail = job.log_tail();
        if tail.contains("no space left") {
            "The drive is full. Free a few GB and try again.".to_string()
        } else {
            format!("Downloading {} did not work. {NET}", app.name)
        }
    })?;

    job.step("Starting it");
    let taken: Vec<u16> = containers().await.iter().flat_map(|c| c.ports.clone()).collect();
    let mut args: Vec<String> = vec!["run".into(), "-d".into(), "--name".into(), app.id.into(), "--restart".into(), "unless-stopped".into()];
    args.extend(["--label".into(), format!("easywsl.app={}", app.id)]);
    let mut chosen = Vec::new();
    for p in app.ports {
        let host = free_port(p.host, &taken);
        chosen.push(host);
        args.extend(["-p".into(), format!("{host}:{}", p.container)]);
    }
    if let Some(&main) = chosen.first() {
        args.extend(["--label".into(), format!("easywsl.port={main}")]);
    }
    for v in app.volumes {
        args.extend(["-v".into(), format!("easywsl-{}-{}:{}", app.id, v.name, v.path)]);
    }
    for (k, v) in app.env {
        args.extend(["-e".into(), format!("{k}={v}")]);
    }
    for q in app.questions {
        let answer = answers.get(q.id).map(|a| a.trim()).filter(|a| !a.is_empty());
        let Some(answer) = answer else {
            if q.required {
                return Err(format!("{} is needed.", q.label));
            }
            continue;
        };
        match q.kind {
            Kind::Password => {
                for var in q.env {
                    args.extend(["-e".into(), format!("{var}={answer}")]);
                }
            }
            Kind::Folder => {
                let folder = PathBuf::from(answer);
                if !folder.is_dir() {
                    return Err(format!("The folder {answer} does not exist."));
                }
                let inside = paths::to_wsl(&folder).map_err(|e| e.to_string())?;
                let target = q.mount.unwrap_or("/data");
                args.extend(["-v".into(), format!("{inside}:{target}{}", if q.read_only { ":ro" } else { "" })]);
            }
        }
    }
    args.extend(app.args.iter().map(|a| a.to_string()));
    args.push(app.image.into());
    // Logged without the answers' values (they may be passwords).
    job.log_line(&format!("starting {} with ports {chosen:?}", app.id));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run(Engine::Easywsl, &refs, Duration::from_secs(120)).await.map_err(|e| {
        job.log_line(&e);
        if e.contains("is already in use by container") {
            format!("{} is installed already. Remove it under Containers first to install it again.", app.name)
        } else {
            format!("Starting {} did not work: {e}", app.name)
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_from_docker_text() {
        assert_eq!(published_ports("0.0.0.0:8089->80/tcp, [::]:8089->80/tcp"), vec![8089]);
        assert_eq!(published_ports("0.0.0.0:53->53/udp, 0.0.0.0:3000->3000/tcp, 80/tcp"), vec![3000]);
        assert_eq!(published_ports(""), Vec::<u16>::new());
    }

    #[test]
    fn labels() {
        assert_eq!(label("a=b,easywsl.app=jellyfin,c=d", "easywsl.app").as_deref(), Some("jellyfin"));
        assert_eq!(label("maintainer=x", "easywsl.app"), None);
    }

    #[test]
    fn certificates_are_made() {
        let dir = std::env::temp_dir().join(format!("easywsl-cert-test-{}", std::process::id()));
        make_certificates(&dir.join("server"), &dir.join("client")).unwrap();
        for file in ["server/ca.pem", "server/server-cert.pem", "client/cert.pem"] {
            assert!(std::fs::read_to_string(dir.join(file)).unwrap().starts_with("-----BEGIN CERTIFICATE-----"));
        }
        assert!(std::fs::read_to_string(dir.join("client/key.pem")).unwrap().contains("PRIVATE KEY"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
