# EasyWSL

Linux on Windows, made easy: a Windows app that installs and runs Linux systems (through WSL)
for people who have never used Linux or a command line. Each system can get a full Linux
desktop, Hyprland, with one checkbox. *EasyWSL is a working title.*

Status: **pre-release, Phase 1** (see [docs/roadmap.md](docs/roadmap.md)). Next: Docker.

## What it does

- Checks the PC (Windows version, virtualization, WSL) and fixes what it can, with one
  "Yes" prompt from Windows
- **Get Linux**: Ubuntu, Arch Linux, Omarchy, Linux Mint, CachyOS, Debian, Kali, described
  in plain words; install with a name, an account and a "Use the Hyprland desktop" checkbox
- **My systems**: open a system's terminal, its files, or its Hyprland desktop; stop it, make
  it the default, add Hyprland later, remove it; recognises systems installed without EasyWSL
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
| `src-tauri/src/desktop.rs` | Hyprland desktop sessions and the viewer |
| `src-tauri/src/docker.rs` | Downloading Docker Hub images (for systems without a WSL image) |
| `src-tauri/src/jobs.rs` | Long tasks with steps and a log |
| `src/` | The window (React, Tailwind) |
