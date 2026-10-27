---
title: Linux Mint
summary: A calm, Windows-like Linux with the Cinnamon desktop.
section: Your system
order: 41
---

# Linux Mint

Linux Mint is built on Ubuntu and known for feeling familiar to people coming from Windows. It
uses Ubuntu's software, so Ubuntu guides work here too.

## Everyday commands

```
sudo apt update && sudo apt upgrade   # update everything
sudo apt install <name>               # install a program
```

## The desktop: Cinnamon

Cinnamon looks and works much like Windows:

- The **menu** at the bottom left (or the Windows key) lists every program, with a search.
- The **panel** at the bottom shows open windows, and the clock and status at the right.
- Right-click the desktop for **Change desktop background**; **System Settings** in the menu
  has the rest (themes, fonts, the panel).
- **Files** (Nemo) is the file manager. Your Windows drives are under `/mnt`.

Things that need a real PC (Wi-Fi, Bluetooth, power, screens) do nothing here: WSL and Windows
handle them.

## Good to know

- Mint's graphical Update Manager is not part of this set-up; update with the command above.
- Mint's own tools (`mintinstall`, the Software Manager) can be installed with
  `sudo apt install mintinstall` if you prefer clicking.
