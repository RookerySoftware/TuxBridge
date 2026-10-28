# TuxBridge

Linux on Windows, made easy: a Windows app that installs and runs Linux systems (through WSL)
for people who have never used Linux or a command line. Each system can get a full Linux
desktop, Hyprland, with one checkbox. Its mascot is Pip, the little penguin who lives in your
Linux system and bridges you to the Linux side.

By [Rookery Software](https://rookerysoftware.app) · [tuxbridge.com](https://tuxbridge.com) (its
source is in the separate `tuxbridge-website` repository).

Status: **pre-release** (see [docs/roadmap.md](docs/roadmap.md)).

## What it does

- Checks the PC (Windows version, virtualization, WSL) and fixes what it can, with one
  "Yes" prompt from Windows
- **Get Linux**: Ubuntu, Arch Linux, Omarchy, Linux Mint, CachyOS, Debian, Kali, Parrot Security, described
  in plain words; install with a name, an account and a "Use the Hyprland desktop" checkbox
- **My systems**: open a system's terminal, its files, or its Hyprland desktop; stop it, make
  it the default, add Hyprland later, remove it; recognises systems installed without TuxBridge
- **Docker**: one button sets up the free Docker Engine in a hidden system of its own; manage
  containers, install ready-made apps (media server, cloud storage, AI chat, databases...),
  and use `docker` from PowerShell and from the Linux systems. See [docs/docker.md](docs/docker.md)
- **Ask Pip** (optional): Pip, with an AI on the PC, answers questions about TuxBridge, Linux, WSL,
  Windows and Docker from the guides and the PC's state; or a TuxBridge skill for Claude Code,
  Codex and claude.ai instead. See [docs/assistant.md](docs/assistant.md)
- **Learn**: guided lessons (the terminal, software, the file bridge, tours of Ubuntu, Kali and
  Parrot) and HackTheBox-style quests that build harmless practice files in a chosen system and
  have you hunt for flags; a defensive Security Basics quest for Kali and Parrot, and a Kids
  mode with a gentle Treasure Hunt. See [docs/learn.md](docs/learn.md)
- **Help**: about 50 guides, from "what is Linux" to programming, WSL settings, networking, the
  GPU, each desktop and each system in depth (`help/`, in Markdown, to be shared with the
  website), with a search that understands operators (`"exact"`, `-word`, `OR`, `word*`,
  `title:`, `in:ubuntu`, `section:`, `code:`), corrects typos and opens the guide at the right
  heading
- **Pip**: an optional penguin companion with tips, explanations of what you point at and fun
  facts; a skippable first-launch tour; "Show me around" tours of a new system; and little
  lessons with quizzes in Kids mode. See [docs/companion.md](docs/companion.md)
- Says so plainly when WSL itself stops responding, and offers to restart it

The Hyprland desktop comes from [hyprland-wsl](https://github.com/ItsOverk1ll/hyprland-wsl),
whose Linux side is bundled here as the "engine" (`src-tauri/engine`).

## Building

Needs Rust, Node.js with pnpm, and Visual Studio's C++ build tools (all Tauri needs on Windows).

```powershell
pnpm install
pnpm tauri dev          # run it
pnpm tauri build        # installer in src-tauri/target/release/bundle/nsis
```

`pnpm dev` alone serves the window in a browser with a mock backend (made-up systems), for
working on the UI; `?setup=featuresOff` and the like show the setup screens, `?stuck` the
"WSL is not responding" banner.

Tests against the real PC (WSL, the registry, Docker Hub): `cargo test -- --ignored` in
`src-tauri`.

## The display

`display/` holds TuxBridge's own display, used for every desktop (GNOME, Xfce, Cinnamon, Hyprland):
a Rust agent inside Linux and a Rust viewer window on Windows, talking over wsl.exe (no network,
no VNC). See [docs/desktops.md](docs/desktops.md).
Build both before running the app:

```powershell
pwsh .\scripts\build-display.ps1 -BuildDistro <a distro with Rust, libpipewire-0.3-dev and clang> -BuildUser <its user>
```

## Updating the engine

```powershell
pwsh .\scripts\sync-engine.ps1 -Repo <hyprland-wsl clone> -Ref <branch or tag>
```

copies hyprland-wsl's Linux side from that ref into `src-tauri/engine` and records the commit
in `src-tauri/engine/VERSION`. Commit the result.

## Layout

| Path | What |
| --- | --- |
| `src-tauri/src/wsl.rs` | wsl.exe and the registry: distros, running ones, time limits |
| `src-tauri/src/system.rs` | Is the PC ready, and fixing it |
| `src-tauri/src/catalog.rs` | The systems on offer, described for newcomers |
| `src-tauri/src/install.rs` | Installing one, step by step |
| `src-tauri/src/desktop.rs` | Desktop sessions in TuxBridge's viewer |
| `src-tauri/src/docker.rs` | Downloading Docker Hub images (for systems without a WSL image) |
| `src-tauri/src/containers.rs` | Docker: the engine, the docker command, containers, installing apps |
| `src-tauri/src/apps.rs` | The app gallery |
| `src-tauri/linux/` | TuxBridge's scripts inside Linux: desktops, the Docker engine and client |
| `src-tauri/src/jobs.rs` | Long tasks with steps and a log |
| `src/` | The window (React, Tailwind) |
