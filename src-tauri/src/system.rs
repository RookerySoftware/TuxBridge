//! Is this PC ready for WSL 2, and if not, what is the one next thing to do. The checks and
//! their order come from hyprland-wsl's bootstrap.ps1, tested on fresh Windows 10 and 11.

use serde::Serialize;
use winreg::RegKey;
use winreg::enums::HKEY_LOCAL_MACHINE;

use crate::{paths, win, wsl};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum Readiness {
    /// Everything works.
    Ready { wsl_version: String },
    /// Windows older than 10 21H2: WSL 2 needs an update of Windows itself.
    WindowsTooOld { build: u32 },
    /// Windows 10 without its November 2022 updates: Windows' wsl cannot hand over to the
    /// current WSL.
    NeedsWindowsUpdate { build: u32, ubr: u32 },
    /// Virtualization is off in the firmware, or the PC is a virtual machine without nested
    /// virtualization.
    VirtualizationOff { virtual_machine: Option<String> },
    /// No WSL, or only the old one built into Windows 10: install the current WSL.
    WslMissing,
    /// WSL is there, but the Windows features it needs are not switched on.
    FeaturesOff,
    /// Something was switched on that needs a restart of Windows.
    RestartNeeded,
}

fn windows_build() -> (u32, u32) {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
    let Ok(key) = key else { return (0, 0) };
    let build: String = key.get_value("CurrentBuildNumber").unwrap_or_default();
    let ubr: u32 = key.get_value("UBR").unwrap_or(0);
    (build.parse().unwrap_or(0), ubr)
}

/// The machine's maker and model, when it looks like a virtual machine.
fn virtual_machine() -> Option<String> {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"HARDWARE\DESCRIPTION\System\BIOS").ok()?;
    let maker: String = key.get_value("SystemManufacturer").unwrap_or_default();
    let model: String = key.get_value("SystemProductName").unwrap_or_default();
    let both = format!("{maker} {model}");
    ["VirtualBox", "VMware", "Virtual Machine", "QEMU", "KVM", "Parallels"]
        .iter()
        .any(|v| both.contains(v))
        .then(|| model.trim().to_string())
}

/// When a restart is pending, the boot time recorded with it differs from the current one only
/// after Windows restarted.
fn boot_marker() -> u64 {
    let uptime = unsafe { windows::Win32::System::SystemInformation::GetTickCount64() } / 1000;
    // Rounded to a minute, so the same boot gives the same value.
    (crate::jobs::now().saturating_sub(uptime)) / 60
}

fn restart_file() -> std::path::PathBuf {
    paths::data_dir().join("restart-pending")
}

pub fn mark_restart_needed() {
    let _ = std::fs::create_dir_all(paths::data_dir());
    let _ = std::fs::write(restart_file(), boot_marker().to_string());
}

fn restart_pending() -> bool {
    match std::fs::read_to_string(restart_file()) {
        Ok(marker) => {
            let same_boot = marker.trim().parse::<u64>().map(|m| m.abs_diff(boot_marker()) <= 1).unwrap_or(false);
            if !same_boot {
                let _ = std::fs::remove_file(restart_file());
            }
            same_boot
        }
        Err(_) => false,
    }
}

pub async fn check() -> Readiness {
    let (build, ubr) = windows_build();
    if build != 0 && build < 19044 {
        return Readiness::WindowsTooOld { build };
    }
    if build != 0 && build < 22000 && ubr != 0 && ubr < 2311 {
        return Readiness::NeedsWindowsUpdate { build, ubr };
    }
    if restart_pending() {
        return Readiness::RestartNeeded;
    }
    if !win::hypervisor_present() && !win::virtualization_firmware_enabled() {
        return Readiness::VirtualizationOff { virtual_machine: virtual_machine() };
    }
    // The current WSL knows --version; the old one built into Windows 10 does not.
    let version = match wsl::output(["--version"]).await {
        Ok(out) if out.code == 0 => out.text,
        _ => return Readiness::WslMissing,
    };
    // A WSL without distros also mentions "wsl --install", so only these count as missing features.
    let status = wsl::output(["--status"]).await.map(|o| (o.code, o.text)).unwrap_or((1, String::new()));
    let fine = ["Default Version", "Default Distribution", "no installed distributions"].iter().any(|s| status.1.contains(s));
    let missing = ["Virtual Machine Platform", "Optional Component", "not enabled", "not supported with your current machine", "0x80370102", "HYPERV_NOT_INSTALLED"]
        .iter()
        .any(|s| status.1.contains(s));
    if missing || (status.0 != 0 && !fine) {
        return Readiness::FeaturesOff;
    }
    if status.1.contains("Default Version: 1") {
        let _ = wsl::output(["--set-default-version", "2"]).await;
    }
    let wsl_version = version
        .lines()
        .next()
        .and_then(|l| l.split(':').nth(1))
        .map(|v| v.trim().to_string())
        .unwrap_or_default();
    Readiness::Ready { wsl_version }
}

#[derive(serde::Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(serde::Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Installs the current WSL from Microsoft's release on GitHub (it takes over from the old one
/// built into Windows 10).
async fn install_wsl_package() -> anyhow::Result<()> {
    let arch = if cfg!(target_arch = "aarch64") { "arm64" } else { "x64" };
    let client = reqwest::Client::builder().user_agent("EasyWSL").build()?;
    let release: Release = client
        .get("https://api.github.com/repos/microsoft/WSL/releases/latest")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.starts_with("wsl.") && a.name.ends_with(&format!(".{arch}.msi")))
        .ok_or_else(|| anyhow::anyhow!("WSL {} has no installer for this PC", release.tag_name))?;
    let dir = paths::downloads_dir();
    tokio::fs::create_dir_all(&dir).await?;
    let msi = dir.join(&asset.name);
    let bytes = client.get(&asset.browser_download_url).send().await?.error_for_status()?.bytes().await?;
    tokio::fs::write(&msi, &bytes).await?;
    anyhow::ensure!(win::signature_valid(&msi), "the downloaded WSL installer has no valid signature");
    let msiexec = std::path::PathBuf::from(std::env::var("SystemRoot").unwrap_or(r"C:\Windows".into())).join(r"System32\msiexec.exe");
    let args = format!("/i \"{}\" /quiet /norestart", msi.display());
    let code = tokio::task::spawn_blocking(move || win::run_elevated(&msiexec, &args)).await??;
    let _ = tokio::fs::remove_file(&msi).await;
    anyhow::ensure!(code == 0 || code == 3010, "the WSL installer stopped (code {code})");
    Ok(())
}

/// Does the one next thing the readiness check asked for. Returns the new state.
pub async fn fix(state: &str) -> Result<Readiness, String> {
    match state {
        "wslMissing" => {
            install_wsl_package().await.map_err(|e| format!("Installing WSL did not work: {e}. You can also install \"Windows Subsystem for Linux\" from the Microsoft Store."))?;
        }
        "featuresOff" => {
            let exe = wsl::exe();
            let code = tokio::task::spawn_blocking(move || win::run_elevated(&exe, "--install --no-distribution"))
                .await
                .map_err(|e| e.to_string())?
                .map_err(|e| format!("Switching on WSL did not work: {e}."))?;
            if code != 0 {
                return Err(format!("Switching on WSL did not finish (code {code}). Restart the PC and try again."));
            }
            mark_restart_needed();
        }
        _ => {}
    }
    Ok(check().await)
}
