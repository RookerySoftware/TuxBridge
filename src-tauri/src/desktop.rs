//! The Hyprland desktop of a distro, shown in a TigerVNC viewer on Windows: the steps of
//! hyprland-wsl's hyprland-wsl.ps1, plus a free port per session (all WSL distros share one
//! network, so two sessions cannot both use 5900).

use std::collections::BTreeMap;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncReadExt;

use crate::store::Store;
use crate::{paths, win, wsl};

const VIEWER_VERSION: &str = "1.16.2";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub distro: String,
    pub port: u16,
    /// "starting", "running", "ended", "failed"
    pub state: String,
    pub message: Option<String>,
}

#[derive(Default)]
pub struct Sessions(Mutex<BTreeMap<String, Session>>);

impl Sessions {
    pub fn all(&self) -> Vec<Session> {
        self.0.lock().unwrap().values().cloned().collect()
    }
}

fn set(app: &AppHandle, distro: &str, port: u16, state: &str, message: Option<String>) {
    let session = Session { distro: distro.into(), port, state: state.into(), message };
    let sessions = app.state::<Sessions>();
    let mut map = sessions.0.lock().unwrap();
    if state == "ended" {
        map.remove(distro);
    } else {
        map.insert(distro.into(), session.clone());
    }
    drop(map);
    let _ = app.emit("session", session);
}

/// A port nobody listens on. Binding on Windows' side also fails for a port a WSL distro
/// listens on (WSL forwards those to Windows' localhost).
fn free_port(taken: &[u16]) -> anyhow::Result<u16> {
    (5900..5999)
        .find(|p| !taken.contains(p) && TcpListener::bind(("127.0.0.1", *p)).is_ok())
        .ok_or_else(|| anyhow::anyhow!("no free port between 5900 and 5999"))
}

fn vnc_answers(port: u16) -> bool {
    use std::io::Read;
    let Ok(mut stream) = TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), Duration::from_millis(500)) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut banner = [0u8; 12];
    matches!(stream.read(&mut banner), Ok(n) if n >= 3 && &banner[..3] == b"RFB")
}

/// An installed TigerVNC, a copy fetched earlier, or a fresh download of its standalone viewer
/// (kept only if it carries a valid signature).
async fn viewer() -> anyhow::Result<PathBuf> {
    let program_files = std::env::var_os("ProgramFiles").map(PathBuf::from).unwrap_or(r"C:\Program Files".into());
    let installed = program_files.join(r"TigerVNC\vncviewer.exe");
    if installed.exists() {
        return Ok(installed);
    }
    let own = paths::data_dir().join("viewer").join(format!("vncviewer64-{VIEWER_VERSION}.exe"));
    if own.exists() {
        return Ok(own);
    }
    tokio::fs::create_dir_all(own.parent().unwrap()).await?;
    let url = format!("https://sourceforge.net/projects/tigervnc/files/stable/{VIEWER_VERSION}/vncviewer64-{VIEWER_VERSION}.exe/download");
    // SourceForge sends browsers to a mirror page; a plain client gets the file.
    let client = reqwest::Client::builder().user_agent("curl/8.0 (EasyWSL)").build()?;
    let bytes = client.get(url).send().await?.error_for_status()?.bytes().await?;
    let tmp = own.with_extension("download");
    tokio::fs::write(&tmp, &bytes).await?;
    if !win::signature_valid(&tmp) {
        let _ = tokio::fs::remove_file(&tmp).await;
        anyhow::bail!("the downloaded viewer does not carry a valid signature");
    }
    tokio::fs::rename(&tmp, &own).await?;
    Ok(own)
}

/// Runs a distro's one-time setup (Omarchy's first-boot setup) in a terminal window and waits
/// for it. Exit 2 is an older launcher without that command: nothing to set up.
async fn first_start(distro: &str) -> anyhow::Result<()> {
    let status = tokio::process::Command::new(wsl::exe())
        .args(["-d", distro, "-u", "root", "-e", "hyprland-wsl", "first-start"])
        .env("WSL_UTF8", "1")
        .creation_flags(wsl::CREATE_NEW_CONSOLE)
        .status()
        .await?;
    match status.code() {
        Some(0) | Some(2) => Ok(()),
        _ => anyhow::bail!("the setup did not finish; open the desktop again to go on with it"),
    }
}

/// The size the desktop gets: the monitor the app is on, in Windows' scaled pixels (Windows
/// enlarges the viewer's picture), or a window of 1600x900.
fn desktop_size(app: &AppHandle, full_screen: bool) -> String {
    if !full_screen {
        return "1600x900".into();
    }
    let monitor = app.get_webview_window("main").and_then(|w| w.current_monitor().ok().flatten());
    match monitor {
        Some(m) => {
            let size = m.size();
            let scale = m.scale_factor().max(1.0);
            format!("{}x{}", (size.width as f64 / scale) as u32, (size.height as f64 / scale) as u32)
        }
        None => "1920x1080".into(),
    }
}

/// Which desktop to open: Hyprland, or the distro's own (Cinnamon, KDE Plasma, Xfce).
#[derive(Debug, Clone, Copy, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Hyprland,
    Own,
}

pub async fn open(app: AppHandle, distro: String, kind: Kind) -> Result<(), String> {
    if app.state::<Sessions>().all().iter().any(|s| s.distro == distro) {
        return Err(format!("The desktop of {distro} is open already."));
    }
    let taken: Vec<u16> = app.state::<Sessions>().all().iter().map(|s| s.port).collect();
    let port = free_port(&taken).map_err(|e| e.to_string())?;
    set(&app, &distro, port, "starting", Some("Preparing".into()));

    let result = session(&app, &distro, port, kind).await;
    match &result {
        Ok(()) => set(&app, &distro, port, "ended", None),
        Err(e) => set(&app, &distro, port, "failed", Some(e.clone())),
    }
    result
}

async fn session(app: &AppHandle, distro: &str, port: u16, kind: Kind) -> Result<(), String> {
    let store = app.state::<Store>();
    let info = store.distro(distro);
    let full_screen = store.settings().full_screen;

    let viewer = viewer().await.map_err(|e| format!("The desktop viewer could not be downloaded: {e}. Check the internet connection."))?;

    if kind == Kind::Hyprland && info.first_start_pending {
        set(app, distro, port, "starting", Some("Finish the setup in the window that opened".into()));
        first_start(distro).await.map_err(|e| e.to_string())?;
        // The setup chose the account; the launcher's config names it.
        let out = wsl::output(["-d", distro, "-u", "root", "-e", "sh", "-c", "sed -n 's/^HYPR_USER=//p' /etc/hyprland-wsl.conf"]).await;
        let user = out.ok().map(|o| o.text.trim().to_string()).filter(|u| !u.is_empty());
        store.update_distro(distro, |d| {
            d.first_start_pending = false;
            if user.is_some() {
                d.user = user.clone();
            }
        });
    }

    set(app, distro, port, "starting", Some("Starting the desktop".into()));
    let size = desktop_size(app, full_screen);
    let log_path = paths::logs_dir().join(format!("desktop-{distro}.log"));
    let _ = std::fs::create_dir_all(paths::logs_dir());
    let port_text = port.to_string();
    let mut launcher = match kind {
        Kind::Hyprland => wsl::command(["-d", distro, "-u", "root", "-e", "hyprland-wsl", &size, "--scale", "1", "--port", &port_text, "--fps", "60", "--no-viewer"]),
        Kind::Own => {
            let (Some(desktop), Some(user)) = (info.desktop.clone(), info.user.clone()) else {
                return Err(format!("{distro} has no desktop of its own installed."));
            };
            let linux = crate::engine::prepare_linux(app).map_err(|e| e.to_string())?;
            wsl::command(["-d", distro, "-u", "root", "-e", "bash", &format!("{linux}/desktop-start.sh"), "run", &user, &desktop, &size, &port_text])
        }
    };
    launcher.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    let mut child = launcher.spawn().map_err(|e| format!("WSL did not start: {e}"))?;

    // The launcher's messages go to a log, read back when it fails.
    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    let log_task = tokio::spawn({
        let log_path = log_path.clone();
        async move {
            let mut a = Vec::new();
            let mut b = Vec::new();
            let _ = tokio::join!(out.read_to_end(&mut a), err.read_to_end(&mut b));
            let mut text = wsl::decode(&a);
            text.push_str(&wsl::decode(&b));
            let _ = std::fs::write(log_path, &text);
            text
        }
    });

    let mut up = false;
    for _ in 0..90 {
        if vnc_answers(port) {
            up = true;
            break;
        }
        if let Ok(Some(_)) = child.try_wait() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    if !up {
        let _ = child.start_kill();
        let text = log_task.await.unwrap_or_default();
        let reason = text.lines().rev().find(|l| l.contains("error") || l.contains("did not")).unwrap_or("").trim().to_string();
        return Err(if reason.is_empty() {
            format!("The desktop did not start. Details are in {}.", log_path.display())
        } else {
            format!("The desktop did not start: {reason}")
        });
    }

    set(app, distro, port, "running", None);
    let mut args = vec!["-RemoteResize=0", "-FullscreenSystemKeys=1", "-ReconnectOnError=0", "-AlwaysCursor=1", "-CursorType=Dot"];
    if full_screen {
        args.push("-FullScreen=1");
    }
    let target = format!("127.0.0.1::{port}");
    args.push(&target);
    // The viewer leaves display scaling to Windows, which enlarges its picture to the screen.
    let viewer_status = tokio::process::Command::new(&viewer).args(&args).env("__COMPAT_LAYER", "DpiUnaware").status().await;

    // Closing the viewer leaves the session running, so it is ended here. When the session
    // ended from inside (Power Off stops the whole instance), its wsl.exe ends at about the same
    // time, and nothing more is done: a wsl.exe call naming a stopping instance would start it again.
    let ended = tokio::time::timeout(Duration::from_secs(4), child.wait()).await.is_ok();
    if !ended {
        match kind {
            Kind::Hyprland => {
                let _ = wsl::output(["-d", distro, "-u", "root", "-e", "pkill", "-x", "Hyprland"]).await;
            }
            Kind::Own => {
                if let Ok(linux) = crate::engine::prepare_linux(app) {
                    let _ = wsl::output(["-d", distro, "-u", "root", "-e", "bash", &format!("{linux}/desktop-start.sh"), "stop"]).await;
                }
            }
        }
        let _ = tokio::time::timeout(Duration::from_secs(10), child.wait()).await;
    }
    let _ = log_task.await;
    viewer_status.map_err(|e| format!("The viewer did not start: {e}"))?;
    Ok(())
}
