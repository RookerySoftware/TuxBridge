//! A distro's desktop, shown in EasyWSL's own viewer (display/viewer). The viewer starts the
//! session through wsl.exe and draws it; easywsl-agent, next to the session in Linux, hands it
//! over on wsl.exe's stdin and stdout. No network and no ports are involved, and the session
//! ends when the viewer closes.
//!
//!   Hyprland                 hyprland-wsl --agent (wlr-screencopy, virtual input)
//!   GNOME                    gnome-start.sh (Mutter's remote desktop interfaces)
//!   Cinnamon, Xfce, ...      desktop-start.sh on a virtual X server (Xvfb)

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncReadExt;

use crate::store::Store;
use crate::wsl;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub distro: String,
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

fn set(app: &AppHandle, distro: &str, state: &str, message: Option<String>) {
    let session = Session { distro: distro.into(), state: state.into(), message };
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

/// Whose desktop it is, for a system EasyWSL knows no account of: the one the desktop was set
/// up for, or else the first regular account.
async fn desktop_user(distro: &str) -> Option<String> {
    let script = r#"cat /etc/easywsl/desktop-user 2> /dev/null ||
        awk -F: '$3 >= 1000 && $3 < 60000 && $6 ~ /^\/home\// { print $1; exit }' /etc/passwd"#;
    let out = wsl::output(["-d", distro, "-u", "root", "-e", "sh", "-c", script]).await.ok()?;
    Some(out.text.trim().to_string()).filter(|u| !u.is_empty())
}

/// The monitor the app is on: its size in real pixels and Windows' scaling.
fn monitor(app: &AppHandle) -> ((u32, u32), f64) {
    let monitor = app.get_webview_window("main").and_then(|w| w.current_monitor().ok().flatten());
    match monitor {
        Some(m) => ((m.size().width, m.size().height), m.scale_factor().max(1.0)),
        None => ((1920, 1080), 1.0),
    }
}

/// Which desktop to open: Hyprland, or the distro's own (GNOME, Cinnamon, Xfce).
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
    set(&app, &distro, "starting", Some("Preparing".into()));

    let result = session(&app, &distro, kind).await;
    match &result {
        Ok(()) => set(&app, &distro, "ended", None),
        Err(e) => set(&app, &distro, "failed", Some(e.clone())),
    }
    result
}

async fn session(app: &AppHandle, distro: &str, kind: Kind) -> Result<(), String> {
    let store = app.state::<Store>();
    let info = store.distro(distro);
    let full_screen = store.settings().full_screen;

    if kind == Kind::Hyprland && info.first_start_pending {
        set(app, distro, "starting", Some("Finish the setup in the window that opened".into()));
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
    let info = store.distro(distro);

    let viewer = app.path().resource_dir().map_err(|e| e.to_string())?.join("bin").join("easywsl-viewer.exe");
    if !viewer.exists() {
        return Err("The desktop viewer is missing; reinstall EasyWSL.".into());
    }
    let linux = crate::engine::prepare_linux(app).map_err(|e| e.to_string())?;
    let agent = format!("{linux}/easywsl-agent");

    // Real pixels, one desktop pixel per screen pixel; the desktop scales its text (Hyprland
    // everything) as Windows does.
    let ((mw, mh), scale) = monitor(app);
    let (w, h) = if full_screen { (mw, mh) } else { ((1600.0 * scale) as u32, (900.0 * scale) as u32) };
    let size = format!("{w}x{h}");
    let scale_text = format!("{scale:.2}");

    let mut user = info.user.clone();
    if user.is_none() {
        user = desktop_user(distro).await;
        store.update_distro(distro, |d| d.user = user.clone());
    }
    // The command the viewer runs, and the log in the distro that says why a session failed.
    let (command, log): (Vec<String>, Option<String>) = match kind {
        Kind::Hyprland => {
            // The launcher shipped with this app, which knows --agent; the one installed in the
            // distro may be older.
            let engine = crate::engine::prepare(app).map_err(|e| e.to_string())?;
            (vec!["bash".into(), format!("{engine}/hyprland-wsl"), size.clone(), "--scale".into(), scale_text, "--agent".into(), agent], None)
        }
        Kind::Own => {
            let (Some(desktop), Some(user)) = (info.desktop.clone(), user.clone()) else {
                return Err(format!("{distro} has no desktop of its own installed."));
            };
            if desktop == "gnome" {
                let log = format!("/run/easywsl/gnome-{user}.log");
                (vec!["bash".into(), format!("{linux}/gnome-start.sh"), user, size.clone(), agent, scale_text], Some(log))
            } else {
                let log = format!("/run/easywsl/desktop-{user}.log");
                (vec!["bash".into(), format!("{linux}/desktop-start.sh"), "run".into(), user, desktop, size.clone(), agent, scale_text], Some(log))
            }
        }
    };

    let mut args: Vec<String> = Vec::new();
    if full_screen {
        args.push("--fullscreen".into());
    }
    args.extend(["--size".into(), size, "--title".into(), format!("{distro} - EasyWSL"), "--".into()]);
    args.push(wsl::exe().to_string_lossy().into_owned());
    args.extend(["-d", distro, "-u", "root", "-e"].map(String::from));
    args.extend(command);

    let mut child = tokio::process::Command::new(&viewer)
        .args(&args)
        .env("WSL_UTF8", "1")
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("The desktop viewer did not start: {e}"))?;
    set(app, distro, "running", None);
    // What the viewer and the launcher say, for when the session fails.
    let mut stderr = child.stderr.take().unwrap();
    let said = tokio::spawn(async move {
        let mut bytes = Vec::new();
        let _ = stderr.read_to_end(&mut bytes).await;
        wsl::decode(&bytes)
    });
    let status = child.wait().await.map_err(|e| e.to_string())?;
    let said = said.await.unwrap_or_default();
    let _ = std::fs::create_dir_all(crate::paths::logs_dir());
    let _ = std::fs::write(crate::paths::logs_dir().join(format!("desktop-{distro}.log")), &said);
    if status.success() {
        return Ok(());
    }

    let mut text = said;
    if let Some(log) = log {
        if let Ok(out) = wsl::output(["-d", distro, "-u", "root", "-e", "tail", "-n", "5", &log]).await {
            text.push_str(&out.text);
        }
    }
    let reason = text
        .lines()
        .rev()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|l| l.trim_start_matches("hyprland-wsl: ").to_string());
    Err(match reason {
        Some(r) => format!("The desktop did not start: {r}"),
        None => "The desktop did not start.".into(),
    })
}
