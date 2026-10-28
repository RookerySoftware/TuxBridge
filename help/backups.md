---
title: Backing up a system
summary: Copy your Linux files to Windows, or export a whole system to a single file.
section: Using Linux
order: 17
---

# Backing up a system

**Removing a system deletes everything in it.** Before you remove or rebuild one, and now and
then anyway, keep a copy of what matters.

## The easy way: copy your files

Most of what you care about is in your home folder. On the system's card in **My systems**,
choose **Open files**, then copy the folders you want (for example `projects`) into a Windows
folder like Documents. That is enough for your own work.

## A whole-system backup: export

WSL can save an entire system to one file, which you can import again later or on another PC.
In PowerShell:

```powershell
wsl --export "<system name>" D:\backups\mysystem.tar
```

To bring it back:

```powershell
wsl --import "<system name>" C:\path\for\the\system D:\backups\mysystem.tar
```

The file is as big as the system (several GB); keep it on a drive with room. A system that is
running exports fine, but close your work in it first.

## What not to rely on

- Linux backup tools that use snapshots (Timeshift and the like) are built for a normal PC and
  do not fit WSL well. Export is the WSL way.
- Files under `/mnt/c` are already on Windows; back those up with your Windows files.
