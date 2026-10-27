---
title: Arch Linux
summary: Always the newest software, pacman, the AUR and the Arch Wiki.
section: Your system
order: 44
---

# Arch Linux

Arch updates continuously (**rolling release**): there are no big version upgrades, programs
arrive in their newest versions within days. It starts small and you add what you want. In
return it expects you to update regularly and read the news when something changes.

## Everyday commands

```
sudo pacman -Syu          # update everything (do this before installing)
pacman -Ss <word>         # find a program
sudo pacman -S <name>     # install it
sudo pacman -Rs <name>    # remove it with what only it needed
```

Never update just one program (`pacman -Sy <name>`): Arch expects the whole system to move
together.

## The AUR

The **Arch User Repository** has recipes for almost any program not in the official
repositories, written by users. A helper like **yay** builds and installs from it:

```
sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay-bin.git && cd yay-bin && makepkg -si
yay -S <name>
```

AUR recipes are not checked by Arch: look at a package's page (votes, comments) before
installing it.

## The Arch Wiki

wiki.archlinux.org is the best Linux documentation there is, useful for every distribution.
When something does not work, search there first.

## Hyprland

Arch has the newest Hyprland. See [The Hyprland desktop](hyprland.md).

## Good to know

- Read archlinux.org/news before big updates now and then; it says when a manual step is
  needed.
- `sudo pacman -Sc` clears the cache of old package versions, which grows over time.
