//! An Pengi skill for the AI apps people already pay for: Claude Code, Codex, and Claude on
//! claude.ai or Claude Desktop (as a .zip to upload). A skill is a folder with SKILL.md, which
//! the AI reads when a question is about its subject, and reference files it reads as needed.
//!
//! The skill is made from the app's own files: skill/SKILL.md (how Pengi is built and what to
//! check), the guides (help/*.md) and the app's description that the local assistant gets too
//! (src/assistant/knowledge.md). Installed copies are rewritten when the app starts, so they
//! follow the app's updates.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager};

/// In SKILL.md of every skill Pengi wrote; a folder without it is someone else's.
const MARK: &str = "<!-- Made by Pengi";
/// The same, from when Pengi was called EasyWSL.
const OLD_MARK: &str = "<!-- Made by EasyWSL";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Target {
    Claude,
    Codex,
}

impl Target {
    const ALL: [Target; 2] = [Target::Claude, Target::Codex];

    fn name(self) -> &'static str {
        match self {
            Target::Claude => "Claude Code",
            Target::Codex => "Codex",
        }
    }

    /// Where the app reads personal skills: Claude Code ~/.claude/skills, Codex ~/.agents/skills.
    fn dir(self) -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(match self {
            Target::Claude => home.join(".claude").join("skills").join("pengi"),
            Target::Codex => home.join(".agents").join("skills").join("pengi"),
        })
    }

    /// Where the skill went while Pengi was called EasyWSL.
    fn old_dir(self) -> Option<PathBuf> {
        self.dir().map(|d| d.with_file_name("easywsl"))
    }

    /// Whether the app is on this PC: its settings folder, or its command on the PATH.
    fn detected(self) -> bool {
        let (folder, command) = match self {
            Target::Claude => (".claude", "claude"),
            Target::Codex => (".codex", "codex"),
        };
        dirs::home_dir().is_some_and(|h| h.join(folder).is_dir()) || on_path(command)
    }
}

fn on_path(command: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else { return false };
    std::env::split_paths(&path).any(|dir| ["exe", "cmd", "bat", "ps1"].iter().any(|ext| dir.join(format!("{command}.{ext}")).is_file()))
}

fn ours(dir: &Path) -> bool {
    std::fs::read_to_string(dir.join("SKILL.md")).is_ok_and(|t| t.contains(MARK) || t.contains(OLD_MARK))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetStatus {
    pub id: Target,
    pub name: &'static str,
    pub detected: bool,
    pub installed: bool,
    pub path: String,
}

pub fn status() -> Vec<TargetStatus> {
    Target::ALL
        .iter()
        .filter_map(|&t| {
            let dir = t.dir()?;
            Some(TargetStatus { id: t, name: t.name(), detected: t.detected(), installed: ours(&dir), path: dir.to_string_lossy().into_owned() })
        })
        .collect()
}

/// "---\ntitle: X\nsummary: Y\norder: 3\n---" -> (title, summary, order).
fn front_matter(text: &str) -> (String, String, i32) {
    let text = text.replace("\r\n", "\n");
    let head = text.strip_prefix("---\n").and_then(|t| t.split_once("\n---\n")).map(|(h, _)| h).unwrap_or("");
    let field = |name: &str| head.lines().find_map(|l| l.strip_prefix(name).map(|v| v.trim().to_string()));
    (field("title:").unwrap_or_default(), field("summary:").unwrap_or_default(), field("order:").and_then(|o| o.parse().ok()).unwrap_or(50))
}

/// Where the skill is made from: the template, the guides' folder, the app's description.
struct Sources {
    template: PathBuf,
    guides: PathBuf,
    knowledge: PathBuf,
}

/// The copies that come with the app (tauri.conf.json, resources).
fn sources(app: &AppHandle) -> anyhow::Result<Sources> {
    let r = app.path().resource_dir()?;
    Ok(Sources { template: r.join("skill").join("SKILL.md"), guides: r.join("help"), knowledge: r.join("assistant").join("knowledge.md") })
}

/// Writes the skill's files into `out` (which must not exist yet).
fn build(app: &AppHandle, out: &Path) -> anyhow::Result<()> {
    build_from(&sources(app)?, out)
}

fn build_from(sources: &Sources, out: &Path) -> anyhow::Result<()> {
    let template = std::fs::read_to_string(&sources.template)?;
    let knowledge = std::fs::read_to_string(&sources.knowledge)?;
    let guides_dir = out.join("references").join("guides");
    std::fs::create_dir_all(&guides_dir)?;

    let mut guides = Vec::new();
    for entry in std::fs::read_dir(&sources.guides)?.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "md") {
            let text = std::fs::read_to_string(&path)?;
            let (title, summary, order) = front_matter(&text);
            let file = entry.file_name().to_string_lossy().into_owned();
            std::fs::write(guides_dir.join(&file), &text)?;
            guides.push((order, title, file, summary));
        }
    }
    guides.sort();
    let index: Vec<String> = guides.iter().map(|(_, title, file, summary)| format!("- `references/guides/{file}`: {title}. {summary}")).collect();

    // The description of the app, without the local assistant's own instructions.
    let app_md = knowledge.split("\n# How to answer").next().unwrap_or(&knowledge).trim().to_string();
    std::fs::write(out.join("references").join("app.md"), format!("# Pengi: the app\n\n{app_md}\n"))?;
    let skill = template.replace("{{VERSION}}", env!("CARGO_PKG_VERSION")).replace("{{GUIDES}}", &index.join("\n"));
    std::fs::write(out.join("SKILL.md"), skill)?;
    Ok(())
}

/// Puts the skill into its folder, replacing an older one Pengi wrote.
fn write(app: &AppHandle, dir: &Path) -> Result<(), String> {
    if dir.exists() && !ours(dir) {
        return Err(format!("{} already has a skill named pengi that Pengi did not make; it was left as it is.", dir.display()));
    }
    let fresh = dir.with_extension("new");
    let _ = std::fs::remove_dir_all(&fresh);
    build(app, &fresh).map_err(|e| {
        let _ = std::fs::remove_dir_all(&fresh);
        format!("The skill could not be made: {e}")
    })?;
    if dir.exists() {
        std::fs::remove_dir_all(dir).map_err(|e| format!("The old skill could not be replaced: {e}"))?;
    }
    std::fs::rename(&fresh, dir).map_err(|e| format!("The skill could not be put in place: {e}"))
}

pub fn install(app: &AppHandle, target: Target) -> Result<(), String> {
    let dir = target.dir().ok_or("Your user folder was not found.")?;
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    write(app, &dir)
}

pub fn remove(target: Target) -> Result<(), String> {
    let dir = target.dir().ok_or("Your user folder was not found.")?;
    if !ours(&dir) {
        return Ok(());
    }
    std::fs::remove_dir_all(&dir).map_err(|e| format!("The skill could not be removed: {e}"))
}

/// Rewrites the installed skills from this version of the app (run when it starts).
pub fn refresh(app: &AppHandle) {
    for t in Target::ALL {
        // A skill installed as easywsl moves to its new name.
        if let (Some(old), Some(new)) = (t.old_dir().filter(|d| ours(d)), t.dir()) {
            if !new.exists() && write(app, &new).is_ok() {
                let _ = std::fs::remove_dir_all(&old);
            }
        }
        if let Some(dir) = t.dir().filter(|d| ours(d)) {
            let _ = write(app, &dir);
        }
    }
}

/// The skill as a .zip in Downloads, for claude.ai or Claude Desktop (Settings, Capabilities,
/// Skills, Upload). Returns its path.
pub fn export_zip(app: &AppHandle) -> Result<String, String> {
    let staging = std::env::temp_dir().join(format!("pengi-skill-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    build(app, &staging.join("pengi")).map_err(|e| format!("The skill could not be made: {e}"))?;
    let downloads = dirs::download_dir().unwrap_or_else(std::env::temp_dir);
    let zip = downloads.join("pengi-skill.zip");
    let _ = std::fs::remove_file(&zip);
    use std::os::windows::process::CommandExt;
    let out = std::process::Command::new(crate::containers::windows_tar())
        .arg("-a")
        .arg("-cf")
        .arg(&zip)
        .arg("-C")
        .arg(&staging)
        .arg("pengi")
        .creation_flags(crate::wsl::CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string());
    let _ = std::fs::remove_dir_all(&staging);
    match out {
        Ok(o) if o.status.success() => Ok(zip.to_string_lossy().into_owned()),
        Ok(o) => Err(format!("The .zip could not be made: {}", String::from_utf8_lossy(&o.stderr).trim())),
        Err(e) => Err(format!("The .zip could not be made: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Makes the skill from the repository's files into the temp folder (kept for a look).
    #[test]
    fn makes_the_skill() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
        let sources = Sources {
            template: repo.join("skill").join("SKILL.md"),
            guides: repo.join("..").join("help"),
            knowledge: repo.join("..").join("src").join("assistant").join("knowledge.md"),
        };
        let out = std::env::temp_dir().join("pengi-skill-test").join("pengi");
        let _ = std::fs::remove_dir_all(&out);
        build_from(&sources, &out).unwrap();
        let skill = std::fs::read_to_string(out.join("SKILL.md")).unwrap();
        assert!(skill.starts_with("---\nname: pengi\ndescription: "));
        assert!(skill.contains(MARK) && !skill.contains("{{"));
        assert!(skill.contains("- `references/guides/welcome.md`: "));
        let app = std::fs::read_to_string(out.join("references").join("app.md")).unwrap();
        assert!(app.contains("# Common tasks") && !app.contains("# How to answer"));
        assert!(out.join("references").join("guides").join("docker.md").is_file());
    }

    #[test]
    fn reads_front_matter() {
        let (title, summary, order) = front_matter("---\r\ntitle: Files\r\nsummary: Moving files.\r\norder: 12\r\n---\r\n# Files\r\n");
        assert_eq!((title.as_str(), summary.as_str(), order), ("Files", "Moving files.", 12));
    }
}
