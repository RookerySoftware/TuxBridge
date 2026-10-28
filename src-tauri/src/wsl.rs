//! Talking to WSL: which distros exist (from the registry, so nothing depends on the language
//! wsl.exe prints in), which are running, and running wsl.exe without a console window.

use std::collections::HashSet;
use std::path::PathBuf;

use serde::Serialize;
use tokio::process::Command;
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;
pub const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
const LXSS: &str = r"Software\Microsoft\Windows\CurrentVersion\Lxss";

/// The current WSL lives in Program Files; Windows' own wsl.exe hands over to it, but the old
/// WSL built into Windows 10 does not exist there.
pub fn exe() -> PathBuf {
    let program_files = std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
    let current = program_files.join("WSL").join("wsl.exe");
    if current.exists() {
        return current;
    }
    let windows = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    windows.join("System32").join("wsl.exe")
}

/// wsl.exe with UTF-8 output and no console window.
pub fn command<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::new(exe());
    // Linux commands start in the Windows folder they were run from; the app's own folder may
    // not be reachable from inside, the user's home is.
    if let Some(home) = dirs::home_dir() {
        cmd.current_dir(home);
    }
    cmd.args(args)
        .env("WSL_UTF8", "1")
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(std::process::Stdio::null());
    cmd
}

/// wsl.exe's text. The old WSL ignores WSL_UTF8 and writes UTF-16; dropping the zero bytes
/// turns its (ASCII) messages into text as well.
pub fn decode(bytes: &[u8]) -> String {
    let bytes: Vec<u8> = bytes.iter().copied().filter(|&b| b != 0).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

pub struct Output {
    pub code: i32,
    pub text: String,
}

/// WSL can stop answering altogether (its virtual machine hangs); then every wsl.exe call
/// waits forever. Quick calls give up after this long, and the window says WSL is stuck.
pub const QUICK: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
#[error("WSL is not responding")]
pub struct NotResponding;

/// Runs wsl.exe and returns its exit code with stdout and stderr together. Gives up (and ends
/// the call) after QUICK.
pub async fn output<I, S>(args: I) -> anyhow::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    output_within(args, QUICK).await
}

pub async fn output_within<I, S>(args: I, limit: std::time::Duration) -> anyhow::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = command(args);
    cmd.kill_on_drop(true);
    let out = match tokio::time::timeout(limit, cmd.output()).await {
        Ok(out) => out?,
        Err(_) => return Err(NotResponding.into()),
    };
    let mut text = decode(&out.stdout);
    text.push_str(&decode(&out.stderr));
    Ok(Output { code: out.status.code().unwrap_or(-1), text })
}

/// Whether a failure's text says WSL's virtual machine did not answer.
pub fn vm_stuck(text: &str) -> bool {
    ["HCS_E_CONNECTION_TIMEOUT", "response was not received from the virtual machine", "WSL is not responding"]
        .iter()
        .any(|s| text.contains(s))
}

/// Stops all of WSL; the next use starts it fresh. For when it stopped answering.
pub async fn shutdown() -> anyhow::Result<()> {
    let out = output_within(["--shutdown"], std::time::Duration::from_secs(90)).await?;
    anyhow::ensure!(out.code == 0, "{}", out.text.trim());
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Registered {
    pub id: String,
    pub name: String,
    pub base_path: String,
    pub version: u32,
    pub is_default: bool,
}

/// The distros of this Windows user, as WSL records them.
pub fn registered() -> Vec<Registered> {
    let Ok(lxss) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(LXSS, KEY_READ) else {
        return Vec::new();
    };
    let default: String = lxss.get_value("DefaultDistribution").unwrap_or_default();
    let mut list = Vec::new();
    for id in lxss.enum_keys().flatten() {
        let Ok(key) = lxss.open_subkey_with_flags(&id, KEY_READ) else { continue };
        let Ok(name) = key.get_value::<String, _>("DistributionName") else { continue };
        let base_path: String = key.get_value("BasePath").unwrap_or_default();
        let base_path = base_path.strip_prefix(r"\\?\").unwrap_or(&base_path).to_string();
        let version: u32 = key.get_value("Version").unwrap_or(2);
        list.push(Registered { is_default: id == default, id, name, base_path, version });
    }
    list.sort_by_key(|d| d.name.to_lowercase());
    list
}

/// Skips the distro's own first-run account creation (its "OOBE"), for a distro whose account
/// Pengi has made.
pub fn skip_oobe(name: &str) {
    let Ok(lxss) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(LXSS, KEY_READ) else { return };
    for id in lxss.enum_keys().flatten() {
        let Ok(key) = lxss.open_subkey_with_flags(&id, KEY_READ | KEY_SET_VALUE) else { continue };
        if key.get_value::<String, _>("DistributionName").ok().as_deref() == Some(name) {
            let _ = key.set_value("RunOOBE", &0u32);
        }
    }
}

/// Names of the running distros (wsl.exe exits non-zero when none runs), or None when WSL does
/// not answer.
pub async fn running() -> Option<HashSet<String>> {
    match output(["--list", "--running", "--quiet"]).await {
        Ok(out) if vm_stuck(&out.text) => None,
        Ok(out) => Some(out.text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()),
        Err(_) => None,
    }
}

pub fn exists(name: &str) -> bool {
    registered().iter().any(|d| d.name.eq_ignore_ascii_case(name))
}

pub async fn terminate(name: &str) {
    let _ = output(["--terminate", name]).await;
}
