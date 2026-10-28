---
title: CachyOS
summary: Arch Linux tuned for speed, with its own optimised packages and repositories.
section: Your system
order: 45
---

# CachyOS

CachyOS is Arch Linux with packages built for modern processors and tuned for speed. Everything
in the [Arch Linux guide](arch.md) applies: `pacman`, the AUR, the Arch Wiki.

**Good for:** people who want Arch with faster builds of programs, and the Hyprland desktop.

## Everyday commands

```bash
sudo pacman -Syu          # update everything
pacman -Ss <word>         # find a program
sudo pacman -S <name>     # install a program
sudo pacman -Rs <name>    # remove it
```

CachyOS adds its own repositories in front of Arch's; `pacman` picks their optimised builds
automatically. To see which repository a package comes from:

```bash
pacman -Si <name> | grep Repository
```

## What is different from Arch

- **Optimised builds** for newer processors (x86-64-v3 and v4), so many programs run faster.
- **Its own keyring**: if an update complains about signatures,
  `sudo pacman -Sy cachyos-keyring archlinux-keyring && sudo pacman -Su`
- CachyOS's kernels and gaming tweaks do not apply in WSL, which runs Microsoft's kernel; the
  optimised programs still do.

## The desktop

CachyOS's own KDE Plasma needs a Wayland session this set-up cannot show yet, so EasyWSL offers
the Hyprland desktop for it. See [The Hyprland desktop](hyprland.md).

## Learn more

- The CachyOS wiki: https://wiki.cachyos.org
- The Arch Wiki: https://wiki.archlinux.org
