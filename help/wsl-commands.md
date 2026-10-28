---
title: The wsl command
summary: Everything EasyWSL's buttons do, from PowerShell: list, start, stop, export, import, move, update.
section: WSL and Windows
order: 50
---

# The wsl command

Everything EasyWSL does goes through `wsl.exe`, which you can also use yourself in PowerShell or
Windows Terminal. Handy for scripts, or when you want to know what's happening.

## Seeing what is there

```powershell
wsl --list --verbose        # every system, whether it runs, and WSL 1 or 2 (short: wsl -l -v)
wsl --status                # the default system and WSL's settings
wsl --version               # WSL's version, and its Linux kernel
```

## Opening a system

```powershell
wsl                         # a shell in the default system
wsl -d Ubuntu               # a shell in a named system
wsl -d Ubuntu -u root       # as root (no password asked)
wsl ~                       # starting in your Linux home folder
wsl ls -la                  # run one Linux command and come back to PowerShell
wsl -d Ubuntu -e htop       # run a program directly, without a shell in between
```

`wsl -- command` runs through your login shell, so `$` and quotes are read by Linux; `-e` does
not. In Windows Terminal, each system also appears in the **new tab** menu.

## Stopping

```powershell
wsl --terminate Ubuntu      # stop one system (like Stop in EasyWSL; short: -t)
wsl --shutdown              # stop every system and WSL itself (like Restart WSL)
```

`--shutdown` is the fix for most strange WSL problems. Save your work in Linux first.

## The default system

```powershell
wsl --set-default Ubuntu    # what plain `wsl` opens (short: -s; Make default in EasyWSL)
```

## Backing up and moving systems

```powershell
wsl --export Ubuntu D:\backups\ubuntu.tar          # the whole system into one file
wsl --import Ubuntu2 D:\WSL\Ubuntu2 D:\backups\ubuntu.tar   # a new system from that file
wsl --import-in-place Name D:\WSL\disk.vhdx        # register an existing virtual disk
wsl --manage Ubuntu --move D:\WSL\Ubuntu           # move a system's disk (recent WSL versions)
```

A system brought in with `--import` signs in as root. To make your user the default again,
put this in its `/etc/wsl.conf` (see [Configuring WSL](wsl-config.md)) and restart it:

```ini
[user]
default=yourname
```

`--export --format vhd` saves the virtual disk itself instead of a `.tar`, which is faster for
big systems.

## Removing

```powershell
wsl --unregister Ubuntu     # deletes the system and EVERYTHING in it, at once, no Recycle Bin
```

This is what **Remove** in EasyWSL does. Copy what you want to keep first.

## Updating WSL

```powershell
wsl --update                # the newest WSL and Linux kernel from Microsoft
wsl --update --pre-release  # the preview version
wsl --version               # check afterwards
```

## WSL 1 and WSL 2

```powershell
wsl --set-version Ubuntu 2      # convert a system to WSL 2 (takes a while)
wsl --set-default-version 2     # new systems use WSL 2
```

EasyWSL uses WSL 2, which runs a real Linux kernel and is needed for Docker, desktops and
systemd. WSL 1 only makes sense for special cases.

## Disks and USB

```powershell
wsl --mount \\.\PHYSICALDRIVE2 --partition 1    # attach a physical disk to Linux (admin)
wsl --unmount \\.\PHYSICALDRIVE2
```

For USB devices, see [USB devices](usb.md).

## Reaching Linux files from PowerShell

```powershell
explorer.exe \\wsl.localhost\Ubuntu\home\you
Copy-Item \\wsl.localhost\Ubuntu\home\you\report.pdf $HOME\Desktop
```

## When a command fails

| Message | Means |
| --- | --- |
| `There is no distribution with the supplied name` | The name is wrong: check `wsl -l -v` (names are as shown there). |
| `WSL_E_DEFAULT_DISTRO_NOT_FOUND` | No default: `wsl --set-default <name>`. |
| `HCS_E_CONNECTION_TIMEOUT` or a hang | WSL is stuck: `wsl --shutdown`, or restart Windows. |
| `0x80370102` | Virtualization is off: turn it on in the PC's BIOS/UEFI (EasyWSL's check explains how). |
| `0x8000000d` on `--import` | The `.tar` file is damaged or incomplete. |
