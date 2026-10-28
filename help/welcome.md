---
title: Linux and WSL in five minutes
summary: What Linux is, what WSL does, and what Pengi adds.
section: Start here
order: 1
---

# Linux and WSL in five minutes

**Linux** is a family of operating systems, like Windows is one. It runs most of the internet's
servers, Android phones, the Steam Deck and many developers' PCs. It is free, and anyone may
build their own version of it: those versions are called **distributions** (Ubuntu, Linux Mint,
Arch Linux and so on). They share the same core and most programs, and differ in how they look,
how often they update, and how they install software.

**WSL** (Windows Subsystem for Linux) is part of Windows. It runs real Linux systems next to
Windows, in a small, fast virtual machine that starts in a second or two. Each system is a
complete Linux with its own files and programs, kept apart from Windows, and you can have
several.

**Pengi** takes care of the parts that usually need a guide:

- checking that your PC is ready, and turning WSL on
- installing a system with an account, and optionally a desktop
- opening the system's terminal, its files, or its desktop in a window
- Docker and ready-made apps, if you want them

## What you can do with it

- **Learn Linux** without changing your PC: nothing is installed over Windows, and removing a
  system removes everything it had.
- **Develop software** with the same tools the servers it runs on use.
- **Run apps** that only exist for Linux, including whole desktops.
- **Try things safely.** A system you break can be removed and installed again in minutes.

## Words you will meet

| Word | Means |
| --- | --- |
| Distribution, distro | One version of Linux: Ubuntu, Mint, Arch... |
| Terminal, shell | The window where you type commands |
| Package | A program packed for installing, like an installer on Windows |
| Package manager | The program that installs packages (`apt`, `pacman`) |
| `sudo` | "Do this as administrator", like Windows' "Yes" prompt |
| Root | Linux's administrator account |
| Home folder | Your own folder, `/home/yourname`, like `C:\Users\you` |
| Desktop environment | The windows, menus and taskbar: GNOME, Cinnamon, Xfce, Hyprland |

Next: [Your first ten minutes](first-steps.md).
