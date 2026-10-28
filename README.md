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

### Feedback

The Feedback page sends reports to Rookery Software's feedback endpoint,
`https://feedback.rookerysoftware.app/v1/report`: a small Cloudflare Worker (its own repository,
`RookerySoftware/feedback`) that checks each report, limits how often one address may send, and
forwards it to the team's Discord. The Discord webhook is a secret of the Worker, never in the app
or this repository. To test against another Worker, build with `TUXBRIDGE_FEEDBACK_URL` set.
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

## Releasing

1. Set the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`,
   and add it to [CHANGELOG.md](CHANGELOG.md).
2. `pwsh .\scripts\release-build.ps1` refreshes `THIRD-PARTY-NOTICES.md` and builds
   `src-tauri/target/release/bundle/nsis/TuxBridge_<version>_x64-setup.exe` (Pip on its pages,
   `src-tauri/installer/`; the hook that upgrades earlier test builds, `src-tauri/installer-hooks.nsh`)
   and its **signature**, `…-setup.exe.sig`, made with the updater's private key
   (`%USERPROFILE%\.tauri\tuxbridge-updater.key`; its public half is in `tauri.conf.json`). Keep
   that key backed up: without it, installed copies can't update any more, only be reinstalled.
3. Commit, tag `v<version>`, and publish a GitHub release with the installer **and its .sig**, the
   changelog entry as its notes (`--prerelease` for a pre-release).
4. Publishing the release puts it on tuxbridge.com by itself: the workflow
   `.github/workflows/publish-to-site.yml` copies the installer into the website repository,
   updates its download section, the `/download/TuxBridge-setup.exe` redirect and
   `download/latest.json` (what Settings → **Update TuxBridge** reads: version, notes, address and
   signature), and pushes; Cloudflare Pages deploys it within a minute or two. (It needs the
   `WEBSITE_DEPLOY_KEY` secret: a deploy key with write access to
   `RookerySoftware/tuxbridge-website`. Without it, run `.github/scripts/publish_to_site.py`
   against a local copy of the website and push.)

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

## Licence

TuxBridge is licensed under the [Apache License 2.0](LICENSE). Copyright 2026 Rookery Software;
see [NOTICE](NOTICE). The libraries it uses are listed in
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

"TuxBridge", "Rookery Software" and Pip (the penguin's name and artwork) are trademarks of
Rookery Software and are not covered by the licence: forks need a name and logo of their own.
