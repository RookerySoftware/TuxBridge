# Changelog

## 0.1.3 (pre-release), 2026-10-28

- **Fixed:** updating Omarchy opened a broken second terminal tab ("cannot find the file
  specified") instead of running the update.
- **Fixed:** in a desktop window, the Windows key could stop reaching Linux (opening the Start
  menu instead) after a busy moment, until the desktop was reopened.

## 0.1.2 (pre-release), 2026-10-28

- **Fixed:** on a PC that never had an earlier test build, TuxBridge closed as soon as you started
  installing any system. Installing works again, and 0.1.1's leftovers from it are tidied away.

## 0.1.1 (pre-release), 2026-10-28

- **Update TuxBridge** in Settings: checks for a newer version, shows what's new, and installs it
  with one click; TuxBridge restarts by itself. Updates are signed, so nothing altered can be installed.
- **Feedback:** report a bug or share an idea from the app, straight to the team (through Rookery's
  feedback service to Discord), with an optional diagnostic report (previewable, Windows user name
  hidden), or save it as a file. Failed tasks have **Report this**, which fills the report in.
- **Licence:** TuxBridge is open source under the Apache License 2.0; the installer now includes
  LICENSE and NOTICE alongside the third-party notices.

## 0.1.0 (pre-release), 2026-10-28

The first release of TuxBridge, by Rookery Software, for testers.

### Linux systems
- **Get Linux:** Ubuntu, Linux Mint, Debian, Arch Linux, CachyOS, Omarchy, Kali Linux and Parrot
  Security, each described in plain words, installed with a name, an account and a desktop.
- **Desktops** (GNOME, Cinnamon, Xfce, KDE Plasma, Hyprland) in a window of their own or full
  screen, through TuxBridge's own display (no VNC, no network).
- **My systems:** terminal, files, desktops, updates, stop, default, add a desktop, remove; also
  systems installed without TuxBridge.
- **Show me around:** Pip walks you through a new system once it's ready.

### Docker
- One button sets up the free Docker Engine in a hidden system of its own; an app gallery
  (media server, cloud storage, AI chat, databases…); `docker` from PowerShell and Linux.

### Learning and help
- **Help:** about 50 guides, with a search that understands operators (`"exact"`, `-word`, `OR`,
  `word*`, `title:`, `in:`, `section:`, `code:`), corrects typos and opens the guide at the right
  heading.
- **Learn:** guided lessons, hands-on challenges (Linux Basics, Security Basics, a Treasure Hunt),
  and Kids mode.
- **Ask Pip:** an optional AI that runs on your own PC; or a TuxBridge skill for Claude Code and
  Codex.

### Pip
- A penguin companion with page tips, explanations of whatever you point at, suggestions, fun
  facts, a welcome tour, and lessons with quizzes in Kids mode. Off by default (Settings), always
  on in Kids mode.

### Windows
- Closing the window keeps TuxBridge in the tray; right-click Pip there for recent systems,
  Settings and Quit.
- The installer includes Microsoft's WebView2 setup, and nothing needs the Visual C++ runtime or
  PowerShell.
- Upgrades a PC that had TuxBridge's earlier test builds (EasyWSL, Pengi), keeping its systems
  and settings.

### Known gaps
- The installer is not signed yet, so Windows SmartScreen warns about it: **More info → Run
  anyway**.
- Copy and paste between Windows and a desktop window isn't there yet.
