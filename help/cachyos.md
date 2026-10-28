---
title: CachyOS
summary: Arch Linux tuned for speed: its optimised repositories, keyrings, tools, Hyprland, and what differs from Arch.
section: Your system
order: 45
---

# CachyOS

CachyOS is Arch Linux with packages built for modern processors and tuned for speed. Everything
in the [Arch Linux guide](arch.md) applies: `pacman`, the AUR, the Arch Wiki, keeping it healthy.
This guide covers what CachyOS adds.

**Good for:** people who want Arch with faster builds of programs, and the Hyprland desktop.

## First things to do

```bash
sudo pacman -Syu
sudo pacman -S --needed base-devel git curl htop unzip man-db
```

## Everyday commands

```bash
sudo pacman -Syu          # update everything
pacman -Ss <word>         # find a program
sudo pacman -S <name>     # install a program
sudo pacman -Rs <name>    # remove it
```

CachyOS ships **paru** as its AUR helper on its installs; if it's missing,
`sudo pacman -S paru` (or install yay as in the Arch guide). `paru -S <name>` installs from the
AUR, and `paru` alone updates everything.

## The optimised repositories

CachyOS adds its own repositories in front of Arch's, built for newer processor features:

| Repository | Built for |
| --- | --- |
| `cachyos-v3`, `cachyos-core-v3`, `cachyos-extra-v3` | x86-64-v3 (most processors from about 2015 on: AVX2) |
| `cachyos-v4`, `…-v4` | x86-64-v4 (AVX-512, some newer processors) |
| `cachyos-znver4` | AMD Zen 4 and newer |
| `cachyos` | Its own tools and packages |

`pacman` picks their builds automatically. To see where a package comes from, and what your
processor supports:

```bash
pacman -Si <name> | grep Repository
/lib/ld-linux-x86-64.so.2 --help | grep supported
```

The repositories are listed in `/etc/pacman.conf`; leave their order as it is.

## What is different from Arch

- **Optimised builds**, so many programs run faster.
- **Its own keyring and mirrors:** if an update complains about signatures,
  ```bash
  sudo pacman -Sy cachyos-keyring archlinux-keyring && sudo pacman -Su
  ```
  and for slow mirrors, `sudo cachyos-rate-mirrors` (from `cachyos-rate-mirrors`).
- **Its kernels and gaming tweaks do not apply in WSL**, which runs Microsoft's kernel. The
  optimised programs still do.
- **Its own tools:** `cachyos-hello` (welcome app) and `cachyos-kernel-manager` are for real
  PCs and can be ignored here.

## The desktop

CachyOS's own KDE Plasma needs a Wayland session this set-up cannot show yet, so Pengi offers
the Hyprland desktop for it. See [The Hyprland desktop](hyprland.md).

## Shell

CachyOS sets up **fish** as the default shell on its own installs, with helpful suggestions as
you type. If your terminal shows fish, bash scripts still run (`bash script.sh`); fish has its
own syntax for its config (`~/.config/fish/config.fish`). `chsh -s /bin/bash` switches to bash.
See [Setting up your shell](shell-setup.md).

## Common problems

| Problem | Fix |
| --- | --- |
| Signature or keyring errors | Update both keyrings (above). |
| `Illegal instruction` from a program | A v4/znver4 build on a processor without those features: check which repositories `/etc/pacman.conf` uses and that your CPU supports them. |
| A guide's `export` line fails | The shell is fish: use `set -x NAME value`, or run bash. |
| Anything else | The [Arch Linux guide](arch.md) problems table applies. |

## Use cases

- **Faster builds and tools** for programming.
- **Hyprland** on an Arch base that is a little more ready-made than plain Arch.

## Learn more

- The CachyOS wiki: https://wiki.cachyos.org
- The Arch Wiki: https://wiki.archlinux.org
