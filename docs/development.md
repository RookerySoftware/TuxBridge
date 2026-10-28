# Developing TuxBridge

## Requirements

- Windows 10 (21H2 or newer) or Windows 11, with WSL 2
- [Rust](https://rustup.rs) (stable), [Node.js](https://nodejs.org) 20+ with [pnpm](https://pnpm.io)
- Visual Studio's C++ build tools (everything [Tauri](https://tauri.app) needs on Windows)

## Running it

```powershell
pnpm install
pnpm tauri dev          # the app, with hot reload
pnpm tauri build        # the installer, in src-tauri/target/release/bundle/nsis
```

`pnpm dev` alone serves the window in a browser with a mock backend (made-up systems, no WSL
needed), for working on the UI. `?setup=featuresOff` and similar show the setup screens, `?stuck`
the "WSL is not responding" banner, `?update` an available update.

## Tests

```powershell
pnpm exec tsc --noEmit          # the window's types
cd src-tauri; cargo test --lib   # unit tests
cargo test -- --ignored         # tests against the real PC: WSL, the registry, Docker Hub
```

## The display

`display/` holds TuxBridge's own display, used for every desktop (GNOME, Xfce, Cinnamon, KDE
Plasma, Hyprland): a Rust **agent** inside Linux and a Rust **viewer** window on Windows, talking
over `wsl.exe` (no network, no VNC). See [desktops.md](desktops.md). Build both before running
the app:

```powershell
pwsh .\scripts\build-display.ps1 -BuildDistro <a distro with Rust, libpipewire-0.3-dev and clang> -BuildUser <its user>
```

Build the agent on the oldest distribution it should run on (Ubuntu 24.04 covers Linux Mint 22).

## The engine

The Hyprland desktop comes from [hyprland-wsl](https://github.com/ItsOverk1ll/hyprland-wsl), whose
Linux side is bundled as the "engine" in `src-tauri/engine`:

```powershell
pwsh .\scripts\sync-engine.ps1 -Repo <hyprland-wsl clone> -Ref <branch or tag>
```

copies that ref into `src-tauri/engine` and records the commit in `src-tauri/engine/VERSION`.

## Feedback

The Feedback page sends reports to Rookery Software's feedback endpoint,
`https://feedback.rookerysoftware.app/v1/report`: a small Cloudflare Worker that checks each
report, limits how often one address may send, and forwards it to the team's Discord. No webhook
or key is built into the app. To test against another endpoint, build with
`TUXBRIDGE_FEEDBACK_URL` set.

## Releasing

1. Set the version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`, and
   add it to [CHANGELOG.md](../CHANGELOG.md).
2. `pwsh .\scripts\release-build.ps1` refreshes `THIRD-PARTY-NOTICES.md` and builds
   `TuxBridge_<version>_x64-setup.exe` with its updater **signature** (`.exe.sig`), made with
   Rookery Software's private key (its public half is in `tauri.conf.json`; the updater refuses
   anything it didn't sign).
3. Commit, tag `v<version>`, and publish a GitHub release with the installer and its `.sig`, the
   changelog entry as its notes.
4. `.github/workflows/publish-to-site.yml` then puts the release on tuxbridge.com: the installer,
   the download section, the `/download/TuxBridge-setup.exe` redirect, and `download/latest.json`,
   which Settings → **Update TuxBridge** reads.

## Layout

| Path | What |
| --- | --- |
| `src/` | The window: React, TypeScript, Tailwind |
| `src/pages/` | One file per page: My systems, Get Linux, Docker, Learn, Ask Pip, Help, Feedback, Settings |
| `src/companion/` | Pip: the artwork, what Pip says, tours, Kids lessons |
| `src/search.ts` | The guides' search, with operators |
| `src/assistant/` | Ask Pip: what the AI is told, and the chat |
| `src/learn/` | Lessons and challenges |
| `src/tray/` | The tray menu |
| `src-tauri/src/wsl.rs` | `wsl.exe` and the registry: distros, running ones, time limits |
| `src-tauri/src/system.rs` | Is the PC ready, and fixing it |
| `src-tauri/src/catalog.rs` | The systems on offer, described for newcomers |
| `src-tauri/src/install.rs` | Installing one, step by step |
| `src-tauri/src/desktop.rs` | Desktop sessions in TuxBridge's viewer |
| `src-tauri/src/docker.rs` | Downloading Docker Hub images (for systems without a WSL image) |
| `src-tauri/src/containers.rs` | Docker: the engine, the `docker` command, containers, apps |
| `src-tauri/src/apps.rs` | The app gallery |
| `src-tauri/src/ai.rs` | Ask Pip's local AI (Ollama) |
| `src-tauri/src/feedback.rs` | The Feedback page's report and sending |
| `src-tauri/src/updates.rs` | Updating TuxBridge itself |
| `src-tauri/src/jobs.rs` | Long tasks with steps and a log |
| `src-tauri/linux/` | TuxBridge's scripts inside Linux: desktops, Docker, updates, Learn's challenges |
| `src-tauri/engine/` | The bundled hyprland-wsl engine |
| `display/` | The desktop display: protocol, Linux agent, Windows viewer |
| `help/` | The guides, in Markdown |
| `docs/` | How the parts work |
