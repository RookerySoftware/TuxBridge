//! The engine: hyprland-wsl's Linux side, bundled with the app (src-tauri/engine, filled by
//! scripts/sync-engine.ps1). It is copied once per version into <data>\unpacked\engine\<version>,
//! a folder every distro reaches under /mnt, and run from there.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::paths;

fn bundled(app: &AppHandle) -> anyhow::Result<PathBuf> {
    let dir = app.path().resource_dir()?.join("engine");
    anyhow::ensure!(dir.join("install.sh").exists(), "the app's engine files are missing; reinstall TuxBridge");
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
        // Never copy a folder into itself: with `to` inside `from`, the copy would find itself
        // and recurse until the stack ran out, closing the app with nothing logged.
        if to.starts_with(entry.path()) {
            continue;
        }
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Where a bundled folder is copied to: <data>\unpacked\<name>\<version>. Not <data>\<name>\...:
/// on a fresh PC the data folder is the install folder (%LOCALAPPDATA%\TuxBridge), so that would
/// be inside the bundled folder itself.
fn unpacked(name: &str, version: &str) -> PathBuf {
    paths::data_dir().join("unpacked").join(name).join(version)
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

/// A folder name an old copy gave inside a bundled folder: a 12-character version or hash, or
/// "unknown", perhaps unfinished (".partial").
fn runaway_copy(name: &str) -> bool {
    let stem = name.strip_suffix(".partial").unwrap_or(name);
    stem.len() == 12 && stem.chars().all(|c| c.is_ascii_hexdigit()) || stem == "unknown"
}

/// Removes what 0.1.1 left behind on a fresh PC: copies it made inside the bundled engine and
/// linux folders themselves (named after a version or hash, or unfinished `.partial`), nested
/// deeper and deeper until the app crashed. The bundled files themselves are never touched.
pub fn clean_up_runaway_copies(app: &AppHandle) {
    let Ok(resources) = app.path().resource_dir() else { return };
    for name in ["engine", "linux"] {
        let Ok(entries) = std::fs::read_dir(resources.join(name)) else { continue };
        for entry in entries.flatten() {
            if runaway_copy(&entry.file_name().to_string_lossy()) && entry.file_type().is_ok_and(|t| t.is_dir()) {
                // The nesting goes past Windows' usual path length; the \\?\ form doesn't mind.
                let path = std::fs::canonicalize(entry.path()).unwrap_or_else(|_| entry.path());
                let _ = std::fs::remove_dir_all(path);
            }
        }
    }
}

/// The engine's folder as seen from inside WSL (/mnt/c/Users/.../engine/<version>).
pub fn prepare(app: &AppHandle) -> anyhow::Result<String> {
    let source = bundled(app)?;
    unpack(&source, &unpacked("engine", &version(&source)))
}

/// TuxBridge's own Linux scripts (src-tauri/linux: the distros' own desktops), as seen from inside
/// WSL. Named after a hash of their contents, so a changed script is never served from an old copy.
pub fn prepare_linux(app: &AppHandle) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let source = app.path().resource_dir()?.join("linux");
    anyhow::ensure!(source.join("desktop-start.sh").exists(), "the app's desktop files are missing; reinstall TuxBridge");
    let mut names: Vec<_> = std::fs::read_dir(&source)?.flatten().map(|e| e.path()).collect();
    names.sort();
    let mut hash = Sha256::new();
    for path in names {
        hash.update(std::fs::read(path).unwrap_or_default());
    }
    let tag: String = hex::encode(hash.finalize()).chars().take(12).collect();
    unpack(&source, &unpacked("linux", &tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh PC: the copy's target is inside the folder being copied. It must not recurse.
    #[test]
    fn a_folder_copied_into_itself_does_not_recurse() {
        let root = std::env::temp_dir().join(format!("tuxbridge-unpack-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let source = root.join("engine");
        std::fs::create_dir_all(source.join("tools")).unwrap();
        std::fs::write(source.join("install.sh"), "echo hi").unwrap();
        std::fs::write(source.join("tools").join("a.sh"), "echo a").unwrap();
        let target = source.join("abc123");
        unpack(&source, &target).unwrap();
        assert!(target.join("install.sh").exists());
        assert!(target.join("tools").join("a.sh").exists());
        assert!(!target.join("abc123").exists(), "copied into itself");
        assert!(!target.join("abc123.partial").exists(), "copied into itself");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Version-named copies and .partial leftovers count as ours; the bundled folders don't.
    #[test]
    fn only_runaway_copies_look_like_ours() {
        let ours = runaway_copy;
        assert!(ours("3f2a9c01be77") && ours("3f2a9c01be77.partial") && ours("unknown.partial"));
        assert!(!ours("tools") && !ours("patches") && !ours("src") && !ours("install.sh"));
    }
}
