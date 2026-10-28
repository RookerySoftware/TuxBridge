---
title: Disk space and moving systems
summary: See what takes space, clean up, shrink a system's virtual disk, and move systems to another drive.
section: WSL and Windows
order: 56
---

# Disk space and moving systems

Each system keeps everything in one file on Windows, a **virtual disk** (`ext4.vhdx`), in the
folder chosen when installing. It grows as you add things and, unless set to be *sparse*, does
not shrink by itself when you delete them.

## What takes space inside Linux?

```bash
df -h /                          # how full the system's disk is
du -sh ~/* ~/.[!.]* 2> /dev/null | sort -h      # the biggest things in your home folder
sudo du -xh / --max-depth=2 2> /dev/null | sort -h | tail -20   # the biggest folders overall
```

**ncdu** is a friendly, interactive version: `sudo apt install ncdu` (or `pacman -S ncdu`), then
`ncdu ~` or `sudo ncdu -x /`. Arrow keys move, Enter opens a folder, `d` deletes.

## Cleaning up

| What | Command |
| --- | --- |
| Downloaded packages (apt) | `sudo apt clean && sudo apt autoremove` |
| Downloaded packages (pacman) | `sudo pacman -Sc` (`paccache -rk1` keeps one old version; from `pacman-contrib`) |
| Old logs | `sudo journalctl --vacuum-size=100M` |
| Programming caches | `~/.cache` (safe to empty), `npm cache clean --force`, `pip cache purge`, `cargo cache`… |
| Docker | **Disk use and clean-up** on the Docker page, or `docker system prune` |
| Old projects' build folders | `node_modules`, `target`, `build`, `.venv` can always be rebuilt |

## Giving the space back to Windows

Deleting files frees space *inside* Linux, but the `.vhdx` file stays large. To shrink it:

**Sparse disks (easiest).** Newer WSL can make a disk shrink automatically. With the system
stopped, in PowerShell:

```powershell
wsl --manage <system> --set-sparse true
```

**Compacting by hand.** With the system and WSL stopped (`wsl --shutdown`), in an
**administrator** PowerShell (needs the Hyper-V tools; on Windows Home use `diskpart` below):

```powershell
Optimize-VHD -Path "D:\WSL\ubuntu\ext4.vhdx" -Mode Full
```

or with `diskpart`:

```
diskpart
select vdisk file="D:\WSL\ubuntu\ext4.vhdx"
attach vdisk readonly
compact vdisk
detach vdisk
exit
```

Run `sudo fstrim -av` inside Linux first, so it knows which blocks are free. The disk is in the
folder chosen when installing; to list every system's folder, in PowerShell:

```powershell
Get-ChildItem HKCU:\Software\Microsoft\Windows\CurrentVersion\Lxss | Get-ItemProperty | Select-Object DistributionName, BasePath
```

## Moving a system to another drive

**Easiest (recent WSL):**

```powershell
wsl --shutdown
wsl --manage <system> --move D:\WSL\<system>
```

**Any WSL version:** export, remove, import:

```powershell
wsl --export <system> D:\backup.tar
wsl --unregister <system>
wsl --import <system> D:\WSL\<system> D:\backup.tar
```

After an import, set your user as the default again (see [The wsl command](wsl-commands.md)).

**For new systems:** Pengi's **Settings → Folder for new systems**, or **More options** in the
install form.

## The disk is full

A system's disk has a maximum size (1 TB by default on current WSL, so usually it's the Windows
drive that fills up). If Linux reports *No space left on device*:

1. Check Windows' drive in File Explorer: free space there first.
2. Clean up inside Linux (above).
3. Move the system to a bigger drive.

A disk's maximum can be raised with `wsl --manage <system> --resize <size>` on recent WSL.
