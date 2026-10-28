//! The Feedback page: a message (and, if the person agrees, a diagnostic report) sent to Rookery
//! Software, or saved as a file to send another way.
//!
//! Reports go to Rookery's feedback endpoint (a Cloudflare Worker, RookerySoftware/feedback),
//! which checks them, limits how often one address may send, and forwards them to the team's
//! Discord. The Discord webhook is a secret of that Worker, never in the app.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use winreg::RegKey;
use winreg::enums::HKEY_LOCAL_MACHINE;

/// Where reports go. A build can point elsewhere (a test Worker) with TUXBRIDGE_FEEDBACK_URL.
const ENDPOINT: &str = match option_env!("TUXBRIDGE_FEEDBACK_URL") {
    Some(url) => url,
    None => "https://feedback.rookerysoftware.app/v1/report",
};

/// Shortest gap between two sends from this app; the endpoint has its own limit too.
const SEND_GAP: Duration = Duration::from_secs(60);
static LAST_SENT: Mutex<Option<Instant>> = Mutex::new(None);

/// Whether this build can send (otherwise the page offers saving a file only).
pub fn can_send() -> bool {
    ENDPOINT.starts_with("https://")
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Bug,
    Idea,
    Other,
}

impl Kind {
    fn title(self) -> &'static str {
        match self {
            Kind::Bug => "🐞 Bug report",
            Kind::Idea => "💡 Idea",
            Kind::Other => "💬 Feedback",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    kind: Kind,
    message: String,
    name: String,
    include_report: bool,
}

// ---------------------------------------------------------------- the report

pub fn windows_version() -> String {
    let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion") else {
        return "Windows (unknown version)".into();
    };
    let build: String = key.get_value("CurrentBuildNumber").unwrap_or_default();
    let ubr: u32 = key.get_value("UBR").unwrap_or(0);
    let edition: String = key.get_value("EditionID").unwrap_or_default();
    let display: String = key.get_value("DisplayVersion").unwrap_or_default();
    // ProductName still says "Windows 10" on Windows 11; the build tells them apart.
    let name = if build.parse::<u32>().unwrap_or(0) >= 22000 { "Windows 11" } else { "Windows 10" };
    format!("{name} {edition} {display} (build {build}.{ubr})").split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The Windows user name in paths becomes `<you>`.
fn hide_user(text: &str) -> String {
    let user = std::env::var("USERNAME").unwrap_or_default();
    if user.len() < 2 {
        return text.to_string();
    }
    text.replace(&format!("\\{user}\\"), "\\<you>\\")
        .replace(&format!("\\{user}"), "\\<you>")
        .replace(&format!("/{user}/"), "/<you>/")
}

/// The last `n` lines of a log file.
fn tail(path: &PathBuf, n: usize) -> String {
    let bytes = std::fs::read(path).unwrap_or_default();
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

/// Everything that helps track a problem down, as plain text: versions, the systems (without
/// account names), Docker, and the newest task logs. The Windows user name is hidden.
pub async fn build_report(app: &AppHandle) -> String {
    let mut r = String::new();
    let mut line = |s: String| {
        r.push_str(&s);
        r.push('\n');
    };
    line(format!("TuxBridge {}", app.package_info().version));
    line(windows_version());
    line(format!("report made {} (seconds since 1970, UTC)", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)));
    let wsl = crate::wsl::output_within(["--version"], Duration::from_secs(10)).await;
    line(String::new());
    line("---- WSL ----".into());
    line(match wsl {
        Ok(o) => o.text.trim().to_string(),
        Err(e) => format!("(wsl --version failed: {e})"),
    });

    line(String::new());
    line("---- systems ----".into());
    match crate::list_distros(app.clone(), app.state(), app.state()).await {
        Ok(listing) => {
            let v = serde_json::to_value(&listing).unwrap_or_default();
            if v["responding"] == serde_json::Value::Bool(false) {
                line("WSL is not responding".into());
            }
            let distros = v["distros"].as_array().cloned().unwrap_or_default();
            if distros.is_empty() {
                line("(none)".into());
            }
            for d in distros {
                let info = &d["info"];
                let flag = |b: &serde_json::Value, s: &str| if b.as_bool() == Some(true) { format!(", {s}") } else { String::new() };
                line(format!(
                    "{}: {}, WSL {}{}{}{}{}{}{}",
                    d["name"].as_str().unwrap_or("?"),
                    info["catalogId"].as_str().unwrap_or("not made by TuxBridge"),
                    d["wslVersion"],
                    info["desktop"].as_str().map(|s| format!(", desktop {s}")).unwrap_or_default(),
                    flag(&info["hyprland"], "Hyprland"),
                    flag(&d["running"], "running"),
                    flag(&d["isDefault"], "default"),
                    flag(&d["busy"], "busy"),
                    flag(&info["firstStartPending"], "first start pending"),
                ));
            }
        }
        Err(e) => line(format!("(couldn't list them: {e})")),
    }

    line(String::new());
    line("---- Docker ----".into());
    let docker = crate::containers::status().await;
    line(format!(
        "TuxBridge's engine: {}{}; Docker Desktop: {}",
        if docker.installed { if docker.running { "running" } else { "stopped" } } else { "not set up" },
        docker.version.map(|v| format!(" ({v})")).unwrap_or_default(),
        if docker.desktop_installed { if docker.desktop_running { "running" } else { "installed" } } else { "not installed" },
    ));

    let mut logs: Vec<(SystemTime, PathBuf)> = std::fs::read_dir(crate::paths::logs_dir())
        .map(|d| d.flatten().filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path()))).collect())
        .unwrap_or_default();
    logs.sort_by(|a, b| b.0.cmp(&a.0));
    line(String::new());
    line(format!("---- newest logs ({} of {}) ----", logs.len().min(6), logs.len()));
    for (_, path) in logs.iter().take(6) {
        line(format!("== {} ==", path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()));
        line(tail(path, 80));
    }
    hide_user(&r)
}

// ---------------------------------------------------------------- sending and saving

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max { s.to_string() } else { s.chars().take(max - 1).collect::<String>() + "…" }
}

pub async fn send(app: &AppHandle, req: Request) -> Result<(), String> {
    if !can_send() {
        return Err("This build of TuxBridge can't send feedback; save it as a file instead.".into());
    }
    if let Some(at) = *LAST_SENT.lock().unwrap() {
        let wait = SEND_GAP.saturating_sub(at.elapsed());
        if !wait.is_zero() {
            return Err(format!("Please wait {} seconds before sending again.", wait.as_secs() + 1));
        }
    }
    let report = if req.include_report { Some(fit_report(build_report(app).await)) } else { None };
    let body = serde_json::json!({
        "app": "tuxbridge",
        "version": app.package_info().version.to_string(),
        "windows": truncate(&windows_version(), 200),
        "kind": req.kind,
        "message": truncate(req.message.trim(), 4000),
        "name": truncate(req.name.trim(), 100),
        "report": report,
    });
    let client = reqwest::Client::builder().user_agent("TuxBridge").timeout(Duration::from_secs(30)).build().map_err(|e| e.to_string())?;
    let response = client.post(ENDPOINT).json(&body).send().await.map_err(|_| "Couldn't reach the feedback service. Are you online?".to_string())?;
    let status = response.status();
    if status.is_success() {
        *LAST_SENT.lock().unwrap() = Some(Instant::now());
        return Ok(());
    }
    // The endpoint explains a refusal in words meant for people.
    let reason = response.json::<serde_json::Value>().await.ok().and_then(|v| v["error"].as_str().map(str::to_string));
    Err(reason.unwrap_or_else(|| format!("The feedback service answered with error {}.", status.as_u16())))
}

/// The endpoint takes reports up to 256 KB: a longer one keeps its start (the versions and
/// systems) and loses the oldest log lines.
fn fit_report(report: String) -> String {
    const MAX: usize = 250 * 1024;
    if report.len() <= MAX {
        return report;
    }
    let mut cut = MAX;
    while !report.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}\n…(cut to fit)", &report[..cut])
}

/// Saves the message and the report as a text file on the Desktop, and shows it in Explorer.
pub async fn save(app: &AppHandle, req: Request) -> Result<String, String> {
    let report = build_report(app).await;
    let stamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let dir = dirs::desktop_dir().unwrap_or_else(std::env::temp_dir);
    let path = dir.join(format!("tuxbridge-feedback-{stamp}.txt"));
    let text = format!(
        "{}\nfrom: {}\n\n{}\n\n==================== diagnostic report ====================\n{report}",
        req.kind.title(),
        if req.name.trim().is_empty() { "anonymous" } else { req.name.trim() },
        req.message.trim(),
    );
    std::fs::write(&path, text).map_err(|e| format!("Couldn't save the report: {e}"))?;
    let _ = std::process::Command::new("explorer.exe").arg(format!("/select,{}", path.display())).spawn();
    Ok(path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_names_are_hidden() {
        let user = std::env::var("USERNAME").unwrap_or_default();
        if user.len() > 1 {
            let r = hide_user(&format!(r"log C:\Users\{user}\AppData\x"));
            assert!(!r.contains(&format!(r"\{user}\")), "{r}");
            assert!(r.contains(r"\<you>\"), "{r}");
        }
    }

    #[test]
    fn long_messages_are_cut_for_discord() {
        assert_eq!(truncate(&"é".repeat(5000), 3900).chars().count(), 3900);
        assert_eq!(truncate("short", 3900), "short");
    }
}
