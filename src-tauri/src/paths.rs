//! Where Pengi keeps its files, and how a Windows path looks from inside WSL.

use std::path::{Path, PathBuf};

/// %LOCALAPPDATA%\Pengi: settings, logs, downloads, the unpacked engine. A PC that ran Pengi
/// under its earlier name, EasyWSL, keeps %LOCALAPPDATA%\EasyWSL: its systems' disks, Docker's
/// certificates and the docker command's place on the PATH all point there, so it isn't moved.
pub fn data_dir() -> PathBuf {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let local = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(r"C:\Users\Public"));
        let (new, old) = (local.join("Pengi"), local.join("EasyWSL"));
        if !new.join("state.json").exists() && old.join("state.json").exists() { old } else { new }
    })
    .clone()
}

pub fn logs_dir() -> PathBuf {
    data_dir().join("logs")
}

pub fn downloads_dir() -> PathBuf {
    data_dir().join("downloads")
}

/// The default folder for a new distro's disk, when the settings name none.
/// HYPRLAND_WSL_DISTROS (hyprland-wsl's setting) is honoured, so both tools agree.
pub fn default_distros_dir() -> PathBuf {
    match std::env::var_os("HYPRLAND_WSL_DISTROS") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => data_dir().join("Distros"),
    }
}

/// C:\Users\me\x -> /mnt/c/Users/me/x, the path WSL's default automount gives a Windows folder.
pub fn to_wsl(path: &Path) -> anyhow::Result<String> {
    let s = path.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    let mut chars = s.chars();
    match (chars.next(), chars.next()) {
        (Some(drive), Some(':')) if drive.is_ascii_alphabetic() => {
            let rest = &s[2..];
            Ok(format!("/mnt/{}{}", drive.to_ascii_lowercase(), rest.replace('\\', "/")))
        }
        _ => anyhow::bail!("{s} is not on a drive WSL can reach"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_paths_map_to_mnt() {
        assert_eq!(to_wsl(Path::new(r"C:\Users\me\x")).unwrap(), "/mnt/c/Users/me/x");
        assert_eq!(to_wsl(Path::new(r"\\?\P:\a b\c")).unwrap(), "/mnt/p/a b/c");
        assert!(to_wsl(Path::new(r"\\server\share")).is_err());
    }
}
