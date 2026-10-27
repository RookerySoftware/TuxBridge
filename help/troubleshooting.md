---
title: When something goes wrong
summary: WSL not responding, a desktop that does not open, a forgotten password, a full disk.
section: When something goes wrong
order: 90
---

# When something goes wrong

## "WSL is not responding"

WSL sometimes hangs, especially after the PC slept. My systems then shows a banner with
**Restart WSL**: it stops every running system (save your work in them first) and starts WSL
fresh. If that does not help, restart Windows; that always clears it.

From a terminal, `wsl --shutdown` in PowerShell does the same as the button.

## A desktop does not open

- Try once more: the first start of a desktop can take longer than EasyWSL waits.
- The message on the system's card says what went wrong when it is known.
- Details are in `%LOCALAPPDATA%\EasyWSL\logs\desktop-<system>.log`; paste that into File
  Explorer's address bar. Please include it when reporting a problem.

## I forgot my Linux password

In PowerShell:

```
wsl -d <system name> -u root passwd <your linux name>
```

See [Your account, sudo and passwords](accounts.md).

## Something broke after an update, or I broke the system

A system is easy to replace: copy the files you want to keep to Windows (**Open files**), then
**Remove** it and install it again in **Get Linux**. Ten minutes, and it is new.

## The disk is filling up

A system's disk grows as you install things, and does not shrink by itself when you delete
them.

- Clear package caches: `sudo apt clean` (Ubuntu, Mint, Debian, Kali) or
  `sudo pacman -Sc` (Arch, CachyOS, Omarchy).
- Docker: **Disk use and clean-up** on the Docker page.
- Remove systems you no longer use.

Settings → **Folder for new systems** puts new systems on another drive.

## No internet inside Linux

- A VPN on Windows can block WSL's network. Try with the VPN off.
- Restart WSL (above): it sets the network up again.

## Reporting a problem

Every install and change has a log; **Show the log file** on its card opens it. EasyWSL's logs
are in `%LOCALAPPDATA%\EasyWSL\logs`.
