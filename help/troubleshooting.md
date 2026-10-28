---
title: When something goes wrong
summary: WSL not responding, installs that fail, desktops that don't open, forgotten passwords, full disks, no internet, slowness, and how to report a problem.
section: When something goes wrong
order: 90
---

# When something goes wrong

Start with the two fixes that solve most problems:

1. **Restart WSL:** the **Restart WSL** button on My systems, or `wsl --shutdown` in PowerShell.
   It stops every Linux system (save your work in them first).
2. **Restart Windows.** It clears anything WSL itself got stuck on.

For a specific message, see [Common error messages](errors.md), or paste part of it into the
search.

## "WSL is not responding"

WSL sometimes hangs, especially after the PC slept or hibernated. My systems then shows a banner
with **Restart WSL**. If that doesn't help, restart Windows; that always clears it.

If it happens often:

- Update WSL: `wsl --update` in PowerShell.
- Update Windows (Settings → Windows Update).
- VPN, antivirus or network tools that hook into Windows' network sometimes interfere; try
  without them.

## Installing a system fails

- **Show the log file** on the failed job says what went wrong; the last lines matter most.
- **Download failed / timed out:** check the internet connection and try again; a VPN or proxy
  can block downloads.
- **Not enough space:** each system needs several GB; pick another drive in **More options**
  or **Settings → Folder for new systems**.
- **A name already in use:** each system needs a different name.
- **Stuck at one step for a long time:** desktops take 5–15 minutes; a step that shows no change
  for much longer than its estimate is worth reporting; Restart WSL, remove the half-made system and try again.

## A desktop does not open

- Try once more: the first start of a desktop can take longer than Pengi waits.
- The message on the system's card says what went wrong when it is known.
- **Stop** the system, then open the desktop again.
- Details are in `%LOCALAPPDATA%\Pengi\logs\desktop-<system>.log`; paste that into File
  Explorer's address bar. Please include it when reporting a problem.
- A desktop that was working and now shows black: close the window, Restart WSL, open again.

## The desktop is slow or choppy

- Lower **Settings → Desktops → Resolution** (1920 x 1080 is much lighter than 4K).
- Close heavy programs on both sides.
- Animations: turn them off in the desktop's settings (Cinnamon: *Effects*; GNOME: *Accessibility
  → Reduce animation* or *Tweaks*).
- Give WSL more memory or processors (see [Configuring WSL](wsl-config.md)).

## I forgot my Linux password

In PowerShell:

```powershell
wsl -d <system name> -u root passwd <your linux name>
```

See [Your account, sudo and passwords](accounts.md).

## A terminal opens and closes at once

The system can't start its shell. In PowerShell, run `wsl -d <system>` to see the message. A
broken `~/.bashrc` is a common cause: open it as root and fix or rename it:

```powershell
wsl -d <system> -u root -e mv /home/<you>/.bashrc /home/<you>/.bashrc.broken
```

## Something broke after an update, or I broke the system

A system is easy to replace: copy the files you want to keep to Windows (**Files**), then
**Remove** it and install it again in **Get Linux**. Ten minutes, and it is new. Next time,
export a backup first (see [Backing up a system](backups.md)).

Before that, the package manager can usually repair itself:

```bash
sudo dpkg --configure -a && sudo apt install -f     # Ubuntu, Mint, Debian
sudo pacman -Syu                                    # Arch, CachyOS (read archlinux.org/news)
```

## The disk is filling up

A system's disk grows as you install things, and doesn't shrink by itself when you delete them.

- Clear package caches: `sudo apt clean` or `sudo pacman -Sc`.
- Docker: **Disk use and clean-up** on the Docker page.
- Remove systems you no longer use.
- Shrink or move a system: [Disk space and moving systems](disk-space.md).

## No internet inside Linux

- A VPN on Windows can block WSL's network. Try with the VPN off, or see the VPN part of
  [Networking](networking.md).
- Restart WSL: it sets the network up again.
- `ping -c 3 1.1.1.1` works but names don't? That's DNS; see [Networking](networking.md).

## A website I run in Linux doesn't open in Windows

Make it listen on `0.0.0.0` (not `127.0.0.1`), check the port with `ss -tlnp`, and Restart WSL
after sleep. See [Networking](networking.md).

## Linux is slow

- **Working on files under `/mnt/c`?** Linux reaches Windows' drives much more slowly than its
  own disk. Move projects into Linux (`~/projects`). See [Files](files.md).
- **Memory:** `free -h`. If Linux is short, raise `memory=` in `.wslconfig`; if *Windows* is
  short, lower it.
- **Something busy in the background:** `htop` shows it.
- **Antivirus scanning WSL's files:** Windows Security can be told to exclude the folder with
  your systems' disks (only if you trust what's in them).

## Windows feels slow while Linux runs

WSL may hold memory it no longer needs. **Stop** systems you're not using, turn on
`autoMemoryReclaim` (see [Configuring WSL](wsl-config.md)), or set a `memory=` limit.

## Docker doesn't start

See the Docker page's messages, and [Docker](docker.md). Restart WSL if the engine won't start
after the PC slept. Docker Desktop and Pengi's engine can both be installed, but run one at a
time.

## Ask Pip doesn't answer

Pip's AI runs on your PC: the first answer after starting takes longer while the model
loads. On PCs with little graphics memory, choose the *Small* size in Settings. See
[Ask Pip, and AI apps](assistant.md).

## Reporting a problem

Every install and change has a log; **Show the log file** on its card opens it. Pengi's logs are in
`%LOCALAPPDATA%\Pengi\logs`

- what you did, what you expected, what happened;
- the log, and the message on screen;
- `wsl --version` (in PowerShell) and your Windows version.
