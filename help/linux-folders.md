---
title: How Linux is organised
summary: The folders of a Linux system, what lives where, and how they compare to Windows.
section: Using Linux
order: 14
---

# How Linux is organised

Linux has no drive letters. Everything, every disk included, hangs from one top folder: `/`
(called "root"). These folders are the same on every distribution.

| Folder | Holds | Windows' nearest |
| --- | --- | --- |
| `/home/you` (`~`) | Your files and personal settings | `C:\Users\you` |
| `/etc` | System-wide settings, as text files | The registry, roughly |
| `/usr/bin` | Programs (`ls`, `python3`, `firefox`) | `C:\Program Files` |
| `/usr/share` | Programs' data: icons, manuals, themes | |
| `/var/log` | Logs | Event Viewer |
| `/tmp` | Temporary files, emptied on restart | `%TEMP%` |
| `/root` | The administrator's home | |
| `/opt` | Big programs that bring their own folder | `C:\Program Files` |
| `/mnt` | Other disks; in WSL your Windows drives (`/mnt/c`) | `C:\`, `D:\` |
| `/dev`, `/proc`, `/sys` | Devices and the running system, shown as files | Device Manager |

## Hidden files

A file or folder whose name starts with a dot is hidden: `.bashrc`, `.config`. `ls -a` shows
them. Most programs keep their settings in your home this way, in `~/.config/<program>`.

## Everything is a file

Linux shows a lot as files: settings are text files in `/etc`, and even information about the
processor is a file you can read:

```bash
cat /etc/os-release     # which distribution and version this is
cat /proc/cpuinfo       # the processor
df -h                   # how full the disks are
du -sh ~/*              # how big each thing in your home is
```

## Paths

- **Absolute** paths start at the top: `/home/you/notes.txt`
- **Relative** paths start where you are: `notes.txt`, `../other-folder`
- `~` is your home, `.` is the current folder, `..` the one above

Names are case-sensitive: `Notes.txt` and `notes.txt` are two different files. Spaces in names
need quotes (`"My Files"`) or a backslash (`My\ Files`).
