//! The app gallery: ready-made apps that run as Docker containers on TuxBridge's engine, each
//! asking only what it needs. Installed by containers::install_app, with the container named
//! after the app and labelled easywsl.app=<id>, its data in named volumes (easywsl-<id>-<name>)
//! that removing the container keeps.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Password,
    /// A Windows folder the app gets (mounted at `mount`).
    Folder,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub id: &'static str,
    pub label: &'static str,
    pub hint: Option<&'static str>,
    pub kind: Kind,
    pub required: bool,
    /// Environment variables the answer goes into (Password).
    pub env: &'static [&'static str],
    /// Where a folder appears inside the app.
    pub mount: Option<&'static str>,
    pub read_only: bool,
    /// Prefilled: "%USERPROFILE%\Videos" and the like are expanded by the window.
    pub default: Option<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Port {
    /// On the PC (the next free one is taken when this one is in use).
    pub host: u16,
    pub container: u16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub name: &'static str,
    pub path: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub id: &'static str,
    pub name: &'static str,
    /// "Media", "Files", "Home", "AI", "Tools", "Databases"
    pub category: &'static str,
    pub tagline: &'static str,
    pub description: &'static str,
    pub image: &'static str,
    /// The first is the app's own page or service.
    pub ports: &'static [Port],
    /// Whether the first port is a web page to open in the browser.
    pub web: bool,
    pub volumes: &'static [Volume],
    pub env: &'static [(&'static str, &'static str)],
    pub questions: &'static [Question],
    /// More `docker run` options.
    pub args: &'static [&'static str],
    /// Shown before installing: what to know or do first.
    pub note: Option<&'static str>,
    /// Shown once it runs: how to start using it.
    pub after: &'static str,
    /// Download estimate.
    pub download: &'static str,
    pub accent: &'static str,
    pub website: &'static str,
}

const fn password(env: &'static [&'static str], hint: &'static str) -> Question {
    Question { id: "password", label: "Password", hint: Some(hint), kind: Kind::Password, required: true, env, mount: None, read_only: false, default: None }
}

static APPS: &[App] = &[
    App {
        id: "jellyfin",
        name: "Jellyfin",
        category: "Media",
        tagline: "Your own Netflix for the films, shows and music on your PC.",
        description: "Jellyfin organises a folder of videos and music with covers and descriptions, and plays them in the browser, on phones and on TVs. Free and without accounts in the cloud.",
        image: "jellyfin/jellyfin:latest",
        ports: &[Port { host: 8096, container: 8096 }],
        web: true,
        volumes: &[Volume { name: "config", path: "/config" }, Volume { name: "cache", path: "/cache" }],
        env: &[],
        questions: &[Question {
            id: "media",
            label: "Your videos and music",
            hint: Some("A Windows folder. Jellyfin reads it and never changes it."),
            kind: Kind::Folder,
            required: true,
            env: &[],
            mount: Some("/media"),
            read_only: true,
            default: Some(r"%USERPROFILE%\Videos"),
        }],
        args: &[],
        note: None,
        after: "Open it and follow the set-up: make an account, then add a library from the folder /media.",
        download: "about a minute",
        accent: "#AA5CC3",
        website: "https://jellyfin.org",
    },
    App {
        id: "nextcloud",
        name: "Nextcloud",
        category: "Files",
        tagline: "Your own cloud for files, calendars and contacts.",
        description: "Nextcloud keeps files, photos, calendars and contacts on your PC instead of someone else's cloud, with apps for phones and computers.",
        image: "nextcloud:latest",
        ports: &[Port { host: 8080, container: 80 }],
        web: true,
        volumes: &[Volume { name: "data", path: "/var/www/html" }],
        env: &[],
        questions: &[],
        args: &[],
        note: Some("It is reachable from this PC only. Reaching it from your phone takes more set-up."),
        after: "Open it and make the administrator account; the built-in database (SQLite) is fine for one person.",
        download: "1 to 2 minutes",
        accent: "#0082C9",
        website: "https://nextcloud.com",
    },
    App {
        id: "homeassistant",
        name: "Home Assistant",
        category: "Home",
        tagline: "Control lights, plugs and sensors from one place.",
        description: "Home Assistant brings smart home devices from many brands together, with automations like turning lights on at sunset.",
        image: "ghcr.io/home-assistant/home-assistant:stable",
        ports: &[Port { host: 8123, container: 8123 }],
        web: true,
        volumes: &[Volume { name: "config", path: "/config" }],
        env: &[],
        questions: &[],
        args: &[],
        note: Some("Running inside WSL, it cannot discover devices on your network by itself; add them by their address. It runs while your PC is on."),
        after: "Open it and make your account; it takes a minute to get ready the first time.",
        download: "2 to 3 minutes",
        accent: "#18BCF2",
        website: "https://www.home-assistant.io",
    },
    App {
        id: "open-webui",
        name: "Open WebUI",
        category: "AI",
        tagline: "A ChatGPT-like page for AI models that run on your PC.",
        description: "Open WebUI is a chat page for local AI models (through Ollama) or online ones with your own key. Chats stay on your PC.",
        image: "ghcr.io/open-webui/open-webui:main",
        ports: &[Port { host: 3000, container: 8080 }],
        web: true,
        volumes: &[Volume { name: "data", path: "/app/backend/data" }],
        env: &[("OLLAMA_BASE_URL", "http://host.docker.internal:11434")],
        questions: &[],
        args: &["--add-host=host.docker.internal:host-gateway"],
        note: Some("For models on your PC, install Ollama from this gallery too."),
        after: "Open it and make the first account, which becomes the administrator.",
        download: "3 to 5 minutes",
        accent: "#111111",
        website: "https://openwebui.com",
    },
    App {
        id: "ollama",
        name: "Ollama",
        category: "AI",
        tagline: "Runs AI models like Llama and Gemma on your PC.",
        description: "Ollama downloads and runs open AI models. Use it through Open WebUI, or from your own programs at localhost:11434.",
        image: "ollama/ollama:latest",
        ports: &[Port { host: 11434, container: 11434 }],
        web: false,
        volumes: &[Volume { name: "models", path: "/root/.ollama" }],
        env: &[],
        questions: &[],
        args: &[],
        note: Some("It runs on the processor here, which is fine for small models (a few GB) and slow for big ones."),
        after: "Get a model in Open WebUI (Settings, Models), or in a terminal: docker exec -it ollama ollama pull gemma3",
        download: "1 to 2 minutes",
        accent: "#333333",
        website: "https://ollama.com",
    },
    App {
        id: "uptime-kuma",
        name: "Uptime Kuma",
        category: "Tools",
        tagline: "Tells you when a website or service goes down.",
        description: "Uptime Kuma checks websites and services every minute and shows their history, with alerts by e-mail, Discord and more.",
        image: "louislam/uptime-kuma:latest",
        ports: &[Port { host: 3001, container: 3001 }],
        web: true,
        volumes: &[Volume { name: "data", path: "/app/data" }],
        env: &[],
        questions: &[],
        args: &[],
        note: None,
        after: "Open it, make your account, and add a first monitor.",
        download: "under a minute",
        accent: "#5CDD8B",
        website: "https://uptime.kuma.pet",
    },
    App {
        id: "stirling-pdf",
        name: "Stirling PDF",
        category: "Tools",
        tagline: "Merge, split, convert and sign PDFs, privately.",
        description: "Stirling PDF does dozens of PDF jobs in the browser (merge, split, compress, convert, OCR, sign) without uploading anything to the internet.",
        image: "stirlingtools/stirling-pdf:latest",
        ports: &[Port { host: 8082, container: 8080 }],
        web: true,
        volumes: &[Volume { name: "configs", path: "/configs" }],
        env: &[],
        questions: &[],
        args: &[],
        note: None,
        after: "Open it and pick a tool.",
        download: "1 to 2 minutes",
        accent: "#D33A2C",
        website: "https://www.stirlingpdf.com",
    },
    App {
        id: "vaultwarden",
        name: "Vaultwarden",
        category: "Tools",
        tagline: "A password manager you host yourself (Bitwarden compatible).",
        description: "Vaultwarden keeps your passwords in a vault on your PC and works with Bitwarden's apps and browser extensions.",
        image: "vaultwarden/server:latest",
        ports: &[Port { host: 8081, container: 80 }],
        web: true,
        volumes: &[Volume { name: "data", path: "/data" }],
        env: &[],
        questions: &[],
        args: &[],
        note: Some("It is reachable from this PC only, so phones cannot sync with it yet. Remember the master password: nobody can recover it."),
        after: "Open it and make your account.",
        download: "under a minute",
        accent: "#175DDC",
        website: "https://github.com/dani-garcia/vaultwarden",
    },
    App {
        id: "code-server",
        name: "VS Code in the browser",
        category: "Tools",
        tagline: "Visual Studio Code, running in Linux, opened in your browser.",
        description: "code-server is VS Code running inside Linux, so projects build and run with Linux tools while you edit in the browser.",
        image: "lscr.io/linuxserver/code-server:latest",
        ports: &[Port { host: 8443, container: 8443 }],
        web: true,
        volumes: &[Volume { name: "config", path: "/config" }],
        env: &[("PUID", "1000"), ("PGID", "1000"), ("TZ", "Etc/UTC")],
        questions: &[password(&["PASSWORD"], "Asked when you open it.")],
        args: &[],
        note: None,
        after: "Open it and sign in with the password you chose.",
        download: "1 to 2 minutes",
        accent: "#007ACC",
        website: "https://github.com/coder/code-server",
    },
    App {
        id: "postgres",
        name: "PostgreSQL",
        category: "Databases",
        tagline: "The popular open-source database, ready for your projects.",
        description: "PostgreSQL for development: connect from your programs or a database tool at localhost:5432 as the user postgres.",
        image: "postgres:17",
        ports: &[Port { host: 5432, container: 5432 }],
        web: false,
        volumes: &[Volume { name: "data", path: "/var/lib/postgresql/data" }],
        env: &[],
        questions: &[password(&["POSTGRES_PASSWORD"], "For the user postgres.")],
        args: &[],
        note: None,
        after: "Connect to localhost on the port shown, user postgres, with your password.",
        download: "under a minute",
        accent: "#336791",
        website: "https://www.postgresql.org",
    },
    App {
        id: "mariadb",
        name: "MariaDB",
        category: "Databases",
        tagline: "MySQL-compatible database for your projects.",
        description: "MariaDB, the community version of MySQL: connect at localhost:3306 as root.",
        image: "mariadb:11",
        ports: &[Port { host: 3306, container: 3306 }],
        web: false,
        volumes: &[Volume { name: "data", path: "/var/lib/mysql" }],
        env: &[],
        questions: &[password(&["MARIADB_ROOT_PASSWORD"], "For the user root.")],
        args: &[],
        note: None,
        after: "Connect to localhost on the port shown, user root, with your password.",
        download: "under a minute",
        accent: "#C0765A",
        website: "https://mariadb.org",
    },
    App {
        id: "redis",
        name: "Redis",
        category: "Databases",
        tagline: "A fast in-memory store for caches and queues.",
        description: "Redis for development, at localhost:6379. Its data is saved to disk now and then.",
        image: "redis:7",
        ports: &[Port { host: 6379, container: 6379 }],
        web: false,
        volumes: &[Volume { name: "data", path: "/data" }],
        env: &[],
        questions: &[],
        args: &[],
        note: None,
        after: "Connect to localhost on the port shown.",
        download: "under a minute",
        accent: "#DC382D",
        website: "https://redis.io",
    },
];

pub fn all() -> &'static [App] {
    APPS
}

pub fn get(id: &str) -> Option<&'static App> {
    APPS.iter().find(|a| a.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_container_names() {
        let mut ids: Vec<&str> = APPS.iter().map(|a| a.id).collect();
        assert!(ids.iter().all(|id| id.chars().all(|c| c.is_ascii_lowercase() || c == '-')));
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), APPS.len());
    }
}
