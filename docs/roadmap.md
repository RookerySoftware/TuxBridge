# Roadmap

TuxBridge: Linux on Windows with as little friction and foreknowledge as possible, for people
who have never used WSL, Linux or a command line. Called TuxBridge since 2026-10-28 (formerly EasyWSL, briefly Pengi). Free, to be open-sourced once finalized.

## Phase 1: the app and WSL (done)

- App shell (Tauri 2: Rust backend, React UI)
- Setup check: Windows version, virtualization, WSL missing / old / features off / restart
  needed, and installing WSL with one "Yes" prompt
- My systems: installed distros with their state; open a terminal, open the files, open the
  Hyprland desktop, stop, remove, add Hyprland later
- Get Linux: a gallery of distros, described for newcomers; install with a name, an account
  and a "Use the Hyprland desktop" checkbox; progress in plain steps, a log for bug reports
- Desktops (Hyprland, GNOME, Cinnamon, Xfce) in TuxBridge's own viewer: no network, no VNC
  (docs/desktops.md)
- Settings: where new systems go, full screen, resolution and frame rate

## Phase 2: Docker (done, see docs/docker.md)

- "Turn on Docker": a small, hidden distro with the free Docker Engine; `docker` works from
  PowerShell and from every distro, all talking to that engine
- Containers page: start, stop, restart, logs, remove, "Open in browser" for web ports, disk
  use and clean-up
- One-click app gallery: ready-made apps (media server, cloud storage, home automation, ad
  blocking, databases, AI chat), asking only what they need, showing their address when ready
- Docker Desktop: detected when installed, its containers shown and managed too
- Next: bind mounts from a Linux system's own folders (an API proxy rewriting the paths, as
  Docker Desktop does); Docker Desktop's side is untested (not installed on the test PC)

## Phase 3: help directory (started)

- A beginner guide per distro and general topics (what Linux is, files between Windows and
  Linux, installing software, the Hyprland desktop), written in Markdown and shared with the
  website (Cloudflare Pages)
- Done: the Help page (sections, search, links between guides, "Guide to ..." in each system's
  menu) and the guides in `help/` (front matter: title, summary, section, order), a full guide
  for every distro (including Parrot), plus backups
- Done (2026-10-28): about 30 more guides — Programming (Git/SSH, VS Code, Python, Node,
  other languages), WSL and Windows (the wsl command, .wslconfig/wsl.conf, interop, networking,
  GPU, USB, disk space), GNOME/Cinnamon/Xfce, editors, finding, pipes, archives, shell setup,
  scripts, scheduling, a command reference, shortcuts, glossary, FAQ, error messages; the
  Ubuntu, Mint, Debian, Arch, CachyOS and Omarchy guides rewritten in depth. Help's search
  (`src/search.ts`) works per heading, with operators, stemming, synonyms, typo correction and
  highlighting; the assistant uses the same index
- Next: pictures in the guides, the website sharing `help/`

## Phase 4: learn (started, see docs/learn.md)

- Done: the **Learn** page — guided lessons (step-by-step walk-throughs) and HackTheBox-style
  quests that build practice files in a chosen system (bundled `quest-*.sh`) and check flags in
  the browser against their SHA-256; a defensive Security Basics quest for Kali and Parrot; a
  Kids mode with a Treasure Hunt
- Done: **Pip**, a penguin companion (original artwork, `src/companion/`): page tips, button
  explanations on hover, suggestions, fun facts; a skippable first-launch tour with a spotlight;
  "Show me around" tours of a newly installed system; in Kids mode, short lessons with quizzes
  and stars. See docs/companion.md
- Next: more lessons, a lesson that drives the app's own buttons, progress shared with the
  assistant
- Later: a friendly assistant window over the desktop viewer that follows Hyprland's event
  socket and advances when a step is done

## Phase 5: release

- Signed installer, auto-update, winget
- Final name: done, **TuxBridge** (trademarks for TuxBridge and Pip still to register)

## Phase 6: local AI (started early, see docs/assistant.md)

- Done: the Assistant page, an optional local model (Qwen 3.5 on TuxBridge's own Ollama) that
  answers from the guides, a description of the app and the PC's live state; "Ask the
  assistant" on failed tasks; a TuxBridge skill for Claude Code, Codex and claude.ai
- Next: let the assistant act (with the person's OK: update a system, start a container),
  embeddings instead of keyword search when the guides grow, AMD/Intel graphics detection
