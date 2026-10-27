---
title: Your first ten minutes
summary: Open the terminal, update the system, find your files, open the desktop.
section: Start here
order: 2
---

# Your first ten minutes

You installed a system in **Get Linux**. Here is what to do first. Everything starts from its
card in **My systems**.

## 1. Open the terminal

Click **Open**, then **Terminal** (or choose it from the arrow next to Open). A window opens
with a line ending in `$`: Linux waits for a command. Type this and press Enter:

```
whoami
```

It answers with your Linux user name. Commands are case-sensitive: `whoami` works, `WhoAmI`
does not.

## 2. Update the system

New systems are often a few weeks behind. Updating is one command; it asks for the password
you chose when installing (nothing appears while you type it, that is normal).

| System | Command |
| --- | --- |
| Ubuntu, Linux Mint, Debian, Kali | `sudo apt update && sudo apt upgrade` |
| Arch Linux, CachyOS, Omarchy | `sudo pacman -Syu` |

Answer `Y` when it asks. Do this every week or two.

## 3. Find your files

- **From Windows:** the system's card, then **Open files**. File Explorer opens your Linux home
  folder; it is also under **Linux** in Explorer's sidebar.
- **From Linux:** your Windows drives are under `/mnt`: `C:\Users\you\Documents` is
  `/mnt/c/Users/you/Documents`.

More in [Files between Windows and Linux](files.md).

## 4. Open the desktop

If the system has a desktop, **Open** starts it in a window of its own. It takes a few
seconds the first time. Close the window to end the desktop session: save your work first,
like shutting down a PC.

**Ctrl+Alt+Enter** switches between full screen and a window. See
[Using a desktop window](desktop-window.md).

## 5. Install something

Try a small program, for example `htop`, a live view of what runs:

```
sudo apt install htop      # Ubuntu, Mint, Debian, Kali
sudo pacman -S htop        # Arch, CachyOS, Omarchy
htop
```

Press `q` to leave it. More in [Installing software](software.md).
