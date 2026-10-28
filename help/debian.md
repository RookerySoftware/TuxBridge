---
title: Debian
summary: The steady base of Ubuntu and Mint, with the light Xfce desktop, releases and backports.
section: Your system
order: 42
---

# Debian

Debian is one of the oldest Linux distributions and the base of Ubuntu, Mint, Kali and Parrot.
Its **stable** release changes slowly and carefully: programs are a little older, and things
rarely break. A new stable release comes about every two years. Debian is made by volunteers,
with no company behind it.

**Good for:** servers, things that must keep working, light systems, learning what the others
are built on.

## First things to do

```bash
sudo apt update && sudo apt upgrade
sudo apt install curl git htop unzip
```

## Everyday commands

```bash
sudo apt update && sudo apt upgrade   # update everything
apt search <word>                     # find a program
sudo apt install <name>               # install it
sudo apt remove <name>                # remove it
```

## The desktop: Xfce

Xfce is light and quick, with a classic layout:

- the **Applications** menu at the top left lists every program
- open windows show in the panel; the clock and status are at the right
- settings are under **Applications → Settings**: appearance, panel, keyboard
- **Thunar** is the file manager; **Mousepad** edits text

## Releases and backports

Debian names releases after Toy Story characters: 12 is *bookworm*, 13 *trixie*. The current
one is in `/etc/os-release`:

```bash
cat /etc/os-release
```

Newer versions of a few programs are in **backports**; the Debian wiki explains how to use them
(wiki.debian.org/Backports). Debian ships only free software by default. Some drivers and fonts
are in `non-free`; most people in WSL never need them.

## Common problems

| Problem | Fix |
| --- | --- |
| `sudo: command not found` | Systems made by EasyWSL have sudo. Otherwise, as root: `apt install sudo`. |
| A program is older than you need | Look in backports, use Flatpak, or use Ubuntu or Arch for it. |

## Learn more

- The Debian wiki: https://wiki.debian.org
- The Debian administrator's handbook: https://debian-handbook.info
