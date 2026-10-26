//! The Linux systems EasyWSL offers, described for people who have never used Linux.

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Source {
    /// Microsoft's WSL catalogue (`wsl --install <name>`).
    Wsl { name: &'static str },
    /// A Docker Hub image, for distros without a WSL image.
    Docker { image: &'static str },
    /// Arch Linux turned into Omarchy by hyprland-wsl's Omarchy installer.
    Omarchy,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Hyprland {
    /// Not offered (not tested with hyprland-wsl yet).
    No,
    /// Offered as a checkbox at install, and later.
    Optional,
    /// Part of the system (Omarchy is a Hyprland desktop).
    Included,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: &'static str,
    pub title: &'static str,
    /// One line: who it is for.
    pub tagline: &'static str,
    pub description: &'static str,
    pub source: Source,
    pub hyprland: Hyprland,
    /// Shown next to the Hyprland checkbox.
    pub hyprland_note: Option<&'static str>,
    /// Needs third-party package sources for Hyprland (asked before installing).
    pub third_party: Option<&'static str>,
    pub minutes: &'static str,
    pub size_gb: f32,
    pub recommended: bool,
    pub accent: &'static str,
    pub website: &'static str,
}

pub fn all() -> Vec<Entry> {
    vec![
        Entry {
            id: "ubuntu",
            title: "Ubuntu 26.04 LTS",
            tagline: "The most popular Linux. A safe first choice.",
            description: "Ubuntu is the Linux most guides and answers online are written for, so help is easy to find. \
                          LTS means it gets updates for years.",
            source: Source::Wsl { name: "Ubuntu-26.04" },
            hyprland: Hyprland::Optional,
            hyprland_note: Some("Ubuntu's Hyprland is older and a little less smooth than on Arch."),
            third_party: None,
            minutes: "3 to 5",
            size_gb: 2.0,
            recommended: false,
            accent: "#E95420",
            website: "https://ubuntu.com",
        },
        Entry {
            id: "arch",
            title: "Arch Linux",
            tagline: "Always the newest software. Best with the Hyprland desktop.",
            description: "Arch updates continuously, so it gets new versions first. It is the smoothest system for the Hyprland desktop, \
                          and its wiki is one of the best Linux references there is.",
            source: Source::Wsl { name: "archlinux" },
            hyprland: Hyprland::Optional,
            hyprland_note: None,
            third_party: None,
            minutes: "3 to 5",
            size_gb: 2.5,
            recommended: true,
            accent: "#1793D1",
            website: "https://archlinux.org",
        },
        Entry {
            id: "omarchy",
            title: "Omarchy",
            tagline: "A beautiful, ready-made Hyprland desktop by DHH. Beta.",
            description: "Omarchy is Arch Linux with a polished Hyprland desktop, themes and a large set of apps already set up. \
                          The first start shows Omarchy's own setup, where you choose your account. It is the largest download here.",
            source: Source::Omarchy,
            hyprland: Hyprland::Included,
            hyprland_note: Some("Omarchy is a Hyprland desktop; it is always included."),
            third_party: None,
            minutes: "10 to 20",
            size_gb: 11.0,
            recommended: false,
            accent: "#8FBF8F",
            website: "https://omarchy.org",
        },
        Entry {
            id: "mint",
            title: "Linux Mint 22.3",
            tagline: "Familiar and calm. Built on Ubuntu.",
            description: "Linux Mint is known for feeling comfortable to people coming from Windows. \
                          It is built on Ubuntu 24.04, so Ubuntu's guides work for it too.",
            source: Source::Docker { image: "linuxmintd/mint22.3-amd64" },
            hyprland: Hyprland::Optional,
            hyprland_note: Some("Mint has no Hyprland of its own; it comes from a community package source."),
            third_party: Some(
                "Hyprland and a newer graphics driver come from two community package sources (PPAs) that are not part of Linux Mint.",
            ),
            minutes: "4 to 8",
            size_gb: 2.5,
            recommended: false,
            accent: "#87CF3E",
            website: "https://linuxmint.com",
        },
        Entry {
            id: "cachyos",
            title: "CachyOS",
            tagline: "Arch Linux, tuned for speed.",
            description: "CachyOS is based on Arch Linux and builds its packages for modern processors. \
                          Everything about Arch applies to it.",
            source: Source::Docker { image: "cachyos/cachyos" },
            hyprland: Hyprland::Optional,
            hyprland_note: None,
            third_party: None,
            minutes: "3 to 6",
            size_gb: 2.5,
            recommended: false,
            accent: "#00CCB4",
            website: "https://cachyos.org",
        },
        Entry {
            id: "debian",
            title: "Debian",
            tagline: "Rock-solid and conservative. The base of Ubuntu.",
            description: "Debian changes slowly and carefully, which makes it very stable. Ubuntu, Mint and Kali are all built on it.",
            source: Source::Wsl { name: "Debian" },
            hyprland: Hyprland::No,
            hyprland_note: None,
            third_party: None,
            minutes: "2 to 4",
            size_gb: 1.5,
            recommended: false,
            accent: "#A81D33",
            website: "https://www.debian.org",
        },
        Entry {
            id: "kali",
            title: "Kali Linux",
            tagline: "For security testing and learning about it.",
            description: "Kali comes with the tools security professionals use to test systems. It is made for that job, \
                          not as an everyday system.",
            source: Source::Wsl { name: "kali-linux" },
            hyprland: Hyprland::Optional,
            hyprland_note: None,
            third_party: None,
            minutes: "3 to 5",
            size_gb: 2.0,
            recommended: false,
            accent: "#367BF0",
            website: "https://www.kali.org",
        },
    ]
}

pub fn get(id: &str) -> Option<Entry> {
    all().into_iter().find(|e| e.id == id)
}
