//! The engine: hyprland-wsl's Linux side, bundled with the app (src-tauri/engine, filled by
//! scripts/sync-engine.ps1). It is copied once per version into %LOCALAPPDATA%\EasyWSL\engine,
//! a folder every distro reaches under /mnt, and run from there.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::paths;

fn bundled(app: &AppHandle) -> anyhow::Result<PathBuf> {
    let dir = app.path().resource_dir()?.join("engine");
    anyhow::ensure!(dir.join("install.sh").exists(), "the app's engine files are missing; reinstall EasyWSL");
    Ok(dir)
}

/// "commit=..." from the bundled VERSION file, to tell copies apart.
fn version(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("VERSION"))
        .ok()
        .and_then(|v| v.lines().find_map(|l| l.strip_prefix("commit=").map(|c| c.chars().take(12).collect())))
        .unwrap_or_else(|| "unknown".into())
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// The engine's folder as seen from inside WSL (/mnt/c/Users/.../engine/<version>).
pub fn prepare(app: &AppHandle) -> anyhow::Result<String> {
    let source = bundled(app)?;
    let target = paths::data_dir().join("engine").join(version(&source));
    if !target.join("install.sh").exists() {
        let partial = target.with_extension("partial");
        let _ = std::fs::remove_dir_all(&partial);
        copy_dir(&source, &partial)?;
        let _ = std::fs::remove_dir_all(&target);
        std::fs::rename(&partial, &target)?;
    }
    paths::to_wsl(&target)
}
