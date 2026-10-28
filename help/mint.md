---
title: Linux Mint
summary: A calm, Windows-like Linux: first steps, apt, Mint's tools, Cinnamon, Flatpaks, upgrades and common problems.
section: Your system
order: 41
---

# Linux Mint

Linux Mint is built on Ubuntu's LTS releases and known for feeling familiar to people coming
from Windows. It uses Ubuntu's software, so Ubuntu guides work here too (Mint 22 is built on
Ubuntu 24.04, for example). Mint adds its own tools, turns snaps off, and prefers Flatpak for
desktop apps.

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
3. **Install a few basics:**
   ```bash
   sudo apt install build-essential git curl htop
   ```
4. **Check your version and its Ubuntu base:**
   ```bash
   cat /etc/os-release        # Mint's version
   cat /etc/upstream-release/lsb-release    # the Ubuntu it is built on
   ```

## apt, the package manager

Mint uses the same `apt` as Ubuntu:

```bash
sudo apt update && sudo apt upgrade   # update everything
apt search <word>                     # find a program
apt show <name>                       # about it
sudo apt install <name>               # install it
sudo apt install ./file.deb           # install a downloaded .deb
sudo apt remove <name>                # remove it
sudo apt autoremove                   # tidy up
```

Everything in [Ubuntu's apt section](ubuntu.md#apt-the-package-manager) applies: `-dev` packages,
`.deb` files, PPAs (`sudo add-apt-repository ppa:…`).

When a guide names an Ubuntu version (for example a vendor repository for "noble"), use the
**Ubuntu base** of your Mint (above), not Mint's own codename.

## Flatpak

Mint comes ready for Flatpak (Flathub), which has the newest desktop apps:

```bash
flatpak search gimp
flatpak install flathub org.gimp.GIMP
flatpak run org.gimp.GIMP
flatpak update
```

If `flatpak` is missing: `sudo apt install flatpak`. The Software Manager shows Flatpaks too.

## Why there are no snaps

Mint blocks snaps on purpose (they install from one company's store and update themselves).
In WSL that is good news: snaps rarely work there. Use apt or Flatpak. Firefox in Mint is a
normal apt package.

## The desktop: Cinnamon

Cinnamon looks and works much like Windows: a menu at the bottom left, a panel, a tray and a
clock. Full guide: [The Cinnamon desktop](cinnamon.md).

## Mint's own tools

| Tool | Install | Does |
| --- | --- | --- |
| Software Manager | `sudo apt install mintinstall` | Browse and install apps, including Flatpaks |
| Update Manager | `sudo apt install mintupdate` | Updates with a window (the app's **Update** also works) |
| Driver Manager | — | Not needed in WSL: Windows has the drivers |
| Timeshift | `sudo apt install timeshift` | Snapshots; in WSL, exports are simpler (see [Backups](backups.md)) |
| Warpinator | `sudo apt install warpinator` | Send files to other PCs on the network |
| Hypnotix | `sudo apt install hypnotix` | TV and radio streams |

## Upgrading to a new Mint

Minor versions (22 → 22.1 → 22.2) come through the Update Manager: *Edit → Upgrade to…*, or
with `sudo apt install mintupgrade` and the instructions in Mint's release notes. Major versions
(21 → 22) follow a new Ubuntu base and use the **mintupgrade** tool; read Mint's upgrade guide
first, or install the new Mint beside the old one with **Get Linux** and move your files over.

## Common problems

| Problem | Fix |
| --- | --- |
| A program from an Ubuntu guide is missing | Mint uses Ubuntu's packages; the same `apt install` works. Check the Ubuntu base for version-specific steps. |
| Snap commands fail | Mint turns snaps off on purpose; use `apt` or Flatpak. |
| A Flatpak app can't see your files | Flatpaks are sandboxed: grant access with **Flatseal** (`flatpak install flathub com.github.tchx84.Flatseal`). |
| The desktop is slow to start the first time | Cinnamon prepares its settings once; the next start is quicker. |
| Cinnamon misbehaves | **Ctrl+Alt+Esc** restarts it without closing your apps. |
| `Could not get lock` | Another update is running; wait a few minutes. |

## Use cases

- **A friendly Linux desktop** to get used to Linux in a window, with familiar menus.
- **Office and media:** LibreOffice, GIMP, VLC and friends with `apt` or Flatpak.
- **Learning:** the Learn page's lessons all work on Mint, and so do Ubuntu tutorials.

## Learn more

- Linux Mint's user guide: https://linuxmint-user-guide.readthedocs.io
- The Mint forums: https://forums.linuxmint.com
- Ubuntu's documentation applies to most things: see [Ubuntu](ubuntu.md).
