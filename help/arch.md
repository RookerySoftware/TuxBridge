---
title: Arch Linux
summary: Always the newest software, pacman, the AUR, the Arch Wiki and keeping it healthy.
section: Your system
order: 44
---

# Arch Linux

Arch updates continuously (**rolling release**): there are no big version upgrades, programs
arrive in their newest versions within days. It starts small and you add what you want. In
return it expects you to update regularly and read the news when something changes.

**Good for:** the newest software, the Hyprland desktop, learning how Linux fits together.

## First things to do

```bash
sudo pacman -Syu
sudo pacman -S --needed base-devel git curl htop unzip
```

## Everyday commands

```bash
sudo pacman -Syu          # update everything (do this before installing)
pacman -Ss <word>         # find a program
sudo pacman -S <name>     # install it
sudo pacman -Rs <name>    # remove it with what only it needed
pacman -Qe                # what you installed yourself
pacman -Qi <name>         # about an installed package
```

Never update just one program (`pacman -Sy <name>`): Arch expects the whole system to move
together.

## The AUR

The **Arch User Repository** has recipes for almost any program not in the official
repositories, written by users. A helper like **yay** builds and installs from it:

```bash
sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay-bin.git && cd yay-bin && makepkg -si
yay -S <name>
```

AUR recipes are not checked by Arch: look at a package's page (votes, comments) before
installing it.

## The Arch Wiki

wiki.archlinux.org is the best Linux documentation there is, useful for every distribution.
When something does not work, search there first.

## Keeping it healthy

- Update at least every few weeks; a very old Arch can be harder to update.
- Read https://archlinux.org/news now and then; it says when an update needs a manual step.
- `sudo pacman -Sc` clears the cache of old package versions, which grows over time.
- Files ending in `.pacnew` in `/etc` are new versions of settings files you changed; compare
  and merge them when you have time.

## Common problems

| Problem | Fix |
| --- | --- |
| `invalid or corrupted package (PGP signature)` | The keyring is old: `sudo pacman -Sy archlinux-keyring && sudo pacman -Su` |
| `unable to lock database` | Another pacman is running. If none is, `sudo rm /var/lib/pacman/db.lck` |
| `target not found` | Update first (`sudo pacman -Syu`); the program may be in the AUR. |

## Hyprland

Arch has the newest Hyprland. See [The Hyprland desktop](hyprland.md).
