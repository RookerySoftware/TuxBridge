---
name: easywsl
description: Help with EasyWSL, the Windows app that installs and runs Linux systems (WSL), their desktops and Docker, and with Linux, WSL, Windows and Docker in general on a PC that has it. Use when the person mentions EasyWSL, their WSL systems or distros, a Linux desktop window, EasyWSL's Docker engine or app gallery, or a task in EasyWSL that failed.
---

<!-- Made by EasyWSL {{VERSION}}. EasyWSL rewrites this folder when it updates; changes here are lost. -->

# EasyWSL

EasyWSL is a Windows app for people new to Linux. It installs Linux systems in WSL (the Windows
Subsystem for Linux), opens their desktops in a window, runs Docker, and explains things. The
person may not know Linux terms: answer in plain words, and prefer EasyWSL's own buttons when the
app can do the thing (`references/app.md` describes every page and button).

## Look before answering

These read-only commands show the state of the PC (PowerShell; `wsl.exe` prints UTF-16, so
pipe through `Out-String` or use `wsl.exe -l -q` where text matters):

| What | Command |
| --- | --- |
| Linux systems, running or stopped, WSL version | `wsl -l -v` |
| What EasyWSL knows about them (catalogue entry, user, desktops), its settings | `Get-Content $env:LOCALAPPDATA\EasyWSL\state.json` |
| The newest task logs (installing, updates, desktops, Docker) | `Get-ChildItem $env:LOCALAPPDATA\EasyWSL\logs \| Sort-Object LastWriteTime -Descending \| Select-Object -First 10` |
| Inside a system | `wsl -d <name> -e sh -c '<command>'` (as root: add `-u root`) |
| Docker's engine and containers | `docker version`, `docker ps -a` (context `easywsl`) |

A failed task's log ends with the command that failed and its output; read it before guessing.

## How EasyWSL is built

- **Systems** are ordinary WSL distributions. Their disks are in the folder set in Settings
  (`installDir` in state.json; default `%LOCALAPPDATA%\EasyWSL\Distros`). Anything `wsl.exe` can
  do works on them; EasyWSL notices changes made outside it.
- **Desktops** (GNOME, Cinnamon, Xfce, Hyprland) open from the app only: it starts the
  session inside the system and shows it in its own window (Ctrl+Alt+Enter: full screen). You
  cannot open one from a terminal; ask the person to use **Open desktop** on the system's card.
- **Docker**: EasyWSL's engine runs in a hidden Alpine system, `easywsl-docker`. Never install
  packages in it or change it by hand; use `docker`. The engine listens on `127.0.0.1:2376` with
  TLS; Windows' `docker.exe` (in `%LOCALAPPDATA%\EasyWSL\bin`) uses the context `easywsl`, whose
  certificates are in `%LOCALAPPDATA%\EasyWSL\docker\tls`. Start the engine from the app's
  Docker page, or `Start-Process -WindowStyle Hidden wsl.exe '-d','easywsl-docker','-u','root','-e','easywsl-docker','run'`;
  stop it with `wsl -d easywsl-docker -u root -e easywsl-docker stop`. Apps from the gallery are
  containers labelled `easywsl.app=<id>` with volumes named `easywsl-<id>-<name>`.
- **Folder sharing with Docker:** the engine sees Windows drives as `/mnt/c/...`. From
  PowerShell, `-v C:\...` and Compose's relative paths (`./data`) do not work with this engine
  yet; use `-v /mnt/c/Users/<name>/<folder>:/data` or a named volume.
- **The assistant** (EasyWSL's own local AI) runs Ollama on `127.0.0.1:11435` while the app is
  open. It is separate from you.

## Doing things for the person

- Updating a system: `wsl -d <name> -u root -e sh -c 'apt-get update && DEBIAN_FRONTEND=noninteractive apt-get -y dist-upgrade'`
  (Ubuntu, Mint, Debian, Kali, Parrot), `wsl -d <name> -u root -e pacman -Syu --noconfirm` (Arch,
  CachyOS). Omarchy updates with `omarchy-update`, run by the person in a terminal (it asks for
  their password). The system's **Update** in EasyWSL does the same.
- Installing a program: the system's package manager (`apt`, `pacman`); see
  `references/guides/software.md`.
- Containers: plain `docker` / `docker compose` commands work against EasyWSL's engine.

## Ask first

These lose data or interrupt the person's work. Say what will happen and wait for a clear yes:
`wsl --unregister <name>` (deletes a system and everything in it), `wsl --shutdown` and
`wsl -t <name>` (close open desktops and terminals), `docker rm`, `docker volume rm`,
`docker system prune`, `rm -rf` in a home folder, and anything touching `easywsl-docker` itself.

## The guides

EasyWSL's own guides, written for people new to Linux, are in `references/guides/`. Read the one
that matches before explaining, and point the person to it (it is also on the app's Help page).
`references/app.md` describes the app's pages, buttons and facts.

{{GUIDES}}
