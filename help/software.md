---
title: Installing software
summary: Package managers, finding programs, updating, removing, and the commands per system.
section: Using Linux
order: 12
---

# Installing software

On Linux you rarely download installers from websites. Each system has a **package manager**:
one command that finds a program in the system's trusted collection (its **repositories**),
installs it with everything it needs, and keeps it updated with the rest of the system. Think of
it as an app store for the terminal.

## The commands

| To | Ubuntu, Mint, Debian, Kali (`apt`) | Arch, CachyOS, Omarchy (`pacman`) |
| --- | --- | --- |
| Update everything | `sudo apt update && sudo apt upgrade` | `sudo pacman -Syu` |
| Find a program | `apt search firefox` | `pacman -Ss firefox` |
| Install it | `sudo apt install firefox` | `sudo pacman -S firefox` |
| Remove it | `sudo apt remove firefox` | `sudo pacman -Rs firefox` |
| See what is installed | `apt list --installed` | `pacman -Q` |

`apt update` refreshes the list of what is available; `apt upgrade` installs the new versions.
On Arch, always update everything (`-Syu`) before installing: installing one new program on an
old system can break it.

## Programs with windows

Programs with a window (Firefox, GIMP, VS Code's Linux version...) open as their own windows on
your Windows desktop, through WSL's built-in support, even without a Linux desktop. Inside a
Linux desktop opened from EasyWSL, they open on that desktop instead.

## When a program is not in the repositories

- **Arch:** the Arch User Repository (AUR) has almost everything, built from recipes written by
  users. A helper like `yay` installs from it; see [Arch Linux](arch.md).
- **Flatpak** works on every system and has many desktop apps (`flathub.org`).
- **The program's own website** may offer a `.deb` (Ubuntu, Mint, Debian, Kali:
  `sudo apt install ./file.deb`) or install instructions for Linux.

Only install from sources you trust, as on Windows: a command run with `sudo` can change
anything in that system.

## Programming languages

Python is usually there already (`python3`). For others, the package manager has them
(`sudo apt install nodejs`, `sudo pacman -S go`), or use the language's own version manager
(`rustup`, `nvm`, `uv`), which gets newer versions.
