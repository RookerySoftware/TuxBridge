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

/// Copies a bundled folder to `target` once (a folder there is complete or absent).
fn unpack(source: &Path, target: &Path) -> anyhow::Result<String> {
    if !target.exists() {
        let partial = target.with_extension("partial");
        let _ = std::fs::remove_dir_all(&partial);
        copy_dir(source, &partial)?;
        std::fs::rename(&partial, target)?;
    }
    paths::to_wsl(target)
}

/// The engine's folder as seen from inside WSL (/mnt/c/Users/.../engine/<version>).
pub fn prepare(app: &AppHandle) -> anyhow::Result<String> {
    let source = bundled(app)?;
    unpack(&source, &paths::data_dir().join("engine").join(version(&source)))
}

/// EasyWSL's own Linux scripts (src-tauri/linux: the distros' own desktops), as seen from inside
/// WSL. Named after a hash of their contents, so a changed script is never served from an old copy.
pub fn prepare_linux(app: &AppHandle) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let source = app.path().resource_dir()?.join("linux");
    anyhow::ensure!(source.join("desktop-start.sh").exists(), "the app's desktop files are missing; reinstall EasyWSL");
    let mut names: Vec<_> = std::fs::read_dir(&source)?.flatten().map(|e| e.path()).collect();
    names.sort();
    let mut hash = Sha256::new();
    for path in names {
        hash.update(std::fs::read(path).unwrap_or_default());
    }
    let tag: String = hex::encode(hash.finalize()).chars().take(12).collect();
    unpack(&source, &paths::data_dir().join("linux").join(tag))
}
