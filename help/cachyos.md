---
title: CachyOS
summary: Arch Linux tuned for speed, with its own optimised packages.
section: Your system
order: 45
---

# CachyOS

CachyOS is Arch Linux with packages built for modern processors and tuned for speed. Everything
in the [Arch Linux guide](arch.md) applies: `pacman`, the AUR, the Arch Wiki.

## Everyday commands

```
sudo pacman -Syu          # update everything
sudo pacman -S <name>     # install a program
```

CachyOS adds its own repositories in front of Arch's; `pacman` picks their optimised builds
automatically.

## Good to know

- CachyOS's kernels and gaming tweaks do not apply in WSL, which runs Microsoft's kernel; the
  optimised programs still do.
- Its wiki (wiki.cachyos.org) explains what CachyOS changes compared to Arch.
