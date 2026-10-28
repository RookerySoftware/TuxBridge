//! What TuxBridge remembers between runs: the distros it set up, and the settings. WSL itself
//! knows nothing about catalogue entries, accounts or Hyprland, and asking a stopped distro
//! would start it, so this is recorded when TuxBridge makes a change.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DistroInfo {
    /// The catalogue entry it was installed from.
    pub catalog_id: Option<String>,
    pub user: Option<String>,
    /// The distro's own desktop, if installed: "cinnamon", "kde" or "xfce".
    pub desktop: Option<String>,
    pub hyprland: bool,
    /// The Hyprland main key, "SUPER" (the Windows key) or "ALT".
    pub main_key: Option<String>,
    /// Installed by TuxBridge (removing one that was not gets an extra warning).
    pub created_by_easywsl: bool,
    /// A setup that has to run in a terminal before the first desktop (Omarchy's).
    pub first_start_pending: bool,
    pub installed_at: Option<u64>,
    /// A system TuxBridge did not make was asked once what it is (see lib.rs, probe).
    pub probed: bool,
    /// Its `docker` uses TuxBridge's Docker engine (linux/docker-client.sh).
    pub docker: bool,
}

impl DistroInfo {
    /// Whether TuxBridge knows anything about this system yet.
    pub fn known(&self) -> bool {
        self.created_by_easywsl || self.probed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Folder new distros go into; None means paths::default_distros_dir().
    pub install_dir: Option<String>,
    /// Open the desktop full screen (the Windows key reaches Linux only then).
    pub full_screen: bool,
    /// The desktop's size: None fits the screen (full screen) or the window, "screen" is the
    /// screen's size even in a window, "WxH" a fixed size. The viewer scales it to the window.
    pub resolution: Option<String>,
    /// The most frames a second the desktop sends (and its refresh rate where it has one).
    pub frame_rate: u32,
    /// Start Docker's engine when the app opens, if it is set up and not running.
    pub start_docker: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { install_dir: None, full_screen: true, resolution: None, frame_rate: 60, start_docker: true }
    }
}

/// The assistant, when installed (ai.rs).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiInfo {
    /// Its folder: Ollama (ollama\) and the model (models\).
    pub dir: String,
    /// The Ollama model, like "qwen3.5:4b-q4_K_M".
    pub model: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Data {
    distros: BTreeMap<String, DistroInfo>,
    settings: Settings,
    ai: Option<AiInfo>,
}

pub struct Store {
    path: PathBuf,
    data: Mutex<Data>,
}

impl Store {
    pub fn load() -> Self {
        let path = paths::data_dir().join("state.json");
        let data = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Store { path, data: Mutex::new(data) }
    }

    fn save(&self, data: &Data) {
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        // Written beside and renamed, so a crash never leaves half a file.
        let tmp = self.path.with_extension("json.tmp");
        if let Ok(text) = serde_json::to_string_pretty(data) {
            if std::fs::write(&tmp, text).is_ok() {
                let _ = std::fs::rename(&tmp, &self.path);
            }
        }
    }

    pub fn distro(&self, name: &str) -> DistroInfo {
        self.data.lock().unwrap().distros.get(name).cloned().unwrap_or_default()
    }

    pub fn update_distro(&self, name: &str, change: impl FnOnce(&mut DistroInfo)) {
        let mut data = self.data.lock().unwrap();
        change(data.distros.entry(name.to_string()).or_default());
        self.save(&data);
    }

    pub fn forget_distro(&self, name: &str) {
        let mut data = self.data.lock().unwrap();
        data.distros.remove(name);
        self.save(&data);
    }

    pub fn settings(&self) -> Settings {
        self.data.lock().unwrap().settings.clone()
    }

    pub fn set_settings(&self, settings: Settings) {
        let mut data = self.data.lock().unwrap();
        data.settings = settings;
        self.save(&data);
    }

    pub fn ai(&self) -> Option<AiInfo> {
        self.data.lock().unwrap().ai.clone()
    }

    pub fn set_ai(&self, ai: Option<AiInfo>) {
        let mut data = self.data.lock().unwrap();
        data.ai = ai;
        self.save(&data);
    }

    pub fn install_dir(&self) -> PathBuf {
        self.settings().install_dir.map(PathBuf::from).unwrap_or_else(paths::default_distros_dir)
    }
}
