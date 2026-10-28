---
title: Linux Mint
summary: A calm, Windows-like Linux with the Cinnamon desktop, its tools and common problems.
section: Your system
order: 41
---

# Linux Mint

Linux Mint is built on Ubuntu and known for feeling familiar to people coming from Windows. It
uses Ubuntu's software, so Ubuntu guides work here too (Mint 22 is built on Ubuntu 24.04).

**Good for:** people coming from Windows, everyday desktop use, learning Linux at a calm pace.

## First things to do

1. **Update it:** **Update** in the system's **⋯** menu on My systems, or:
   ```bash
   sudo apt update && sudo apt upgrade
   ```
2. **Get the Software Manager** if you prefer clicking to typing:
   ```bash
   sudo apt install mintinstall
   ```
   It is then in the menu as **Software Manager**.

## Everyday commands

```bash
sudo apt update && sudo apt upgrade   # update everything
apt search <word>                     # find a program
sudo apt install <name>               # install a program
sudo apt remove <name>                # remove it
```

## The desktop: Cinnamon

Cinnamon looks and works much like Windows:

- The **menu** at the bottom left (or the Windows key) lists every program, with a search.
- The **panel** at the bottom shows open windows, and the clock and status at the right.
- Right-click the desktop for **Change desktop background**; **System Settings** in the menu
  has the rest (themes, fonts, the panel, keyboard shortcuts).
- **Files** (Nemo) is the file manager. Your Windows drives are under `/mnt`.
- **Applets** (right-click the panel → **Applets**) add things to the panel, like a weather
  display or a system monitor.

Things that need a real PC (Wi-Fi, Bluetooth, power, screens) do nothing here: WSL and Windows
handle them.

## Mint's own tools

| Tool | Install | Does |
| --- | --- | --- |
| Software Manager | `sudo apt install mintinstall` | Browse and install apps |
| Update Manager | `sudo apt install mintupdate` | Updates with a window (the app's **Update** also works) |
| Timeshift | `sudo apt install timeshift` | Snapshots; in WSL, the app's backups are simpler (see [Backups](backups.md)) |

## Common problems

| Problem | Fix |
| --- | --- |
| A program from an Ubuntu guide is missing | Mint uses Ubuntu's packages; the same `apt install` works. |
| Snap commands fail | Mint turns snaps off on purpose; use `apt` or Flatpak. |
| The desktop is slow to start the first time | Cinnamon prepares its settings once; the next start is quicker. |

## Learn more

- Linux Mint's user guide: https://linuxmint-user-guide.readthedocs.io
- The Mint forums: https://forums.linuxmint.com
