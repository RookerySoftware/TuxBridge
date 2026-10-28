---
title: Arch Linux
summary: Always the newest software: pacman in depth, the AUR and yay, mirrors, keyrings, the Arch Wiki, keeping it healthy and fixing problems.
section: Your system
order: 44
---

# Arch Linux

Arch updates continuously (**rolling release**): there are no big version upgrades, programs
arrive in their newest versions within days. It starts small and you add what you want. In
return it expects you to update regularly and read the news when something changes.

**Good for:** the newest software, the Hyprland desktop, learning how Linux fits together,
programming with current tools.

## First things to do

```bash
sudo pacman -Syu                                       # update everything
sudo pacman -S --needed base-devel git curl htop unzip man-db bash-completion
```

`base-devel` is needed to build anything from the AUR (below). `man-db` gives you `man` pages.

## pacman, the package manager

```bash
sudo pacman -Syu              # update everything (always before installing)
pacman -Ss <word>             # search the repositories
pacman -Si <name>             # about a package (in the repositories)
sudo pacman -S <name>         # install it (several names at once are fine)
sudo pacman -S --needed <n>   # install, skipping what's already there
sudo pacman -Rs <name>        # remove it with what only it needed
sudo pacman -Rns <name>       # …and its saved settings files
pacman -Q                     # everything installed
pacman -Qe                    # what you installed yourself
pacman -Qi <name>             # about an installed package
pacman -Ql <name>             # its files
pacman -Qo /usr/bin/ls        # which package owns a file
pacman -Qdt                   # orphans: installed as dependencies, no longer needed
sudo pacman -Rns $(pacman -Qdtq)   # remove the orphans
sudo pacman -U ./file.pkg.tar.zst  # install a package file
```

**Never update just one program** (`pacman -Sy <name>`): Arch expects the whole system to move
together ("partial upgrades are unsupported"). Always `-Syu`, then install.

### Which package has the command I need?

```bash
sudo pacman -Fy                 # once: download the file lists
pacman -F <command>             # e.g. pacman -F dig -> bind
```

## The AUR and yay

The **Arch User Repository** has recipes (PKGBUILDs) for almost any program not in the official
repositories, written by users. A helper like **yay** builds and installs from it:

```bash
sudo pacman -S --needed git base-devel
git clone https://aur.archlinux.org/yay-bin.git && cd yay-bin && makepkg -si
cd .. && rm -rf yay-bin
```

```bash
yay -S <name>          # install from the repositories or the AUR
yay                    # update everything, AUR included
yay -Ss <word>         # search both
```

**AUR recipes are not checked by Arch.** Look at a package's page on aur.archlinux.org (votes,
comments, last update) and, ideally, its PKGBUILD before installing. `-bin` packages install a
ready-built program; others compile it.

## Mirrors

Packages download from *mirrors*. Slow downloads usually mean a far-away mirror. **reflector**
picks fast ones:

```bash
sudo pacman -S reflector
sudo reflector --latest 20 --protocol https --sort rate --save /etc/pacman.d/mirrorlist
sudo pacman -Syyu
```

## pacman's settings

`/etc/pacman.conf` has a few worth turning on (remove the `#` in front):

```ini
Color
ParallelDownloads = 5
VerbosePkgLists
```

## Keeping it healthy

- **Update every week or two.** A very old Arch is harder to update.
- **Read the news** at archlinux.org/news now and then; it says when an update needs a manual
  step. (`yay` shows new items before updating.)
- **Clear the package cache:** `sudo pacman -Sc`, or keep the last version of each with
  `sudo paccache -rk1` (from `pacman-contrib`).
- **Merge .pacnew files:** when an update brings a new version of a settings file you changed,
  it is saved as `file.pacnew`. Find them with `pacdiff` (in `pacman-contrib`) and merge.
- **Remove orphans** now and then (`pacman -Qdt`).

## The Arch Wiki

wiki.archlinux.org is the best Linux documentation there is, useful for every distribution. When
something doesn't work, search there first ("arch wiki <topic>"). Its *General
recommendations* page is a good tour of everything you might set up.

## Languages and locale

```bash
sudo nano /etc/locale.gen        # remove # before e.g. en_US.UTF-8 UTF-8
sudo locale-gen
sudo localectl set-locale LANG=en_US.UTF-8
```

## Hyprland

Arch has the newest Hyprland, and Pengi can add it to an Arch system. See
[The Hyprland desktop](hyprland.md).

## Common problems

| Problem | Fix |
| --- | --- |
| `invalid or corrupted package (PGP signature)` / `unknown trust` | The keyring is old: `sudo pacman -Sy archlinux-keyring && sudo pacman -Su` |
| `unable to lock database` | Another pacman is running. If none is: `sudo rm /var/lib/pacman/db.lck` |
| `target not found` | Update first (`sudo pacman -Syu`); check the name with `pacman -Ss`; it may be in the AUR. |
| `conflicting files` / `exists in filesystem` | Something installed files by hand. Read the news; if you're sure, `pacman -Qo <file>` shows the owner. Avoid `--overwrite` unless the news says so. |
| `failed to commit transaction (conflicting dependencies)` | Usually solved by the news item for that day, or by waiting a day for the mirrors. |
| Downloads fail or are slow | New mirrors with reflector (above). |
| `error while loading shared libraries` | A partial upgrade happened: `sudo pacman -Syu`. For AUR programs, rebuild them (`yay -S <name> --rebuild`). |
| `makepkg` refuses to run as root | Build AUR packages as your normal user, never with sudo. |

## Use cases

- **Newest development tools:** current compilers, languages and libraries without PPAs.
- **A keyboard-driven desktop** with Hyprland.
- **Learning Linux deeply:** you choose every piece, and the Wiki explains it.

## Learn more

- The Arch Wiki: https://wiki.archlinux.org
- News: https://archlinux.org/news
- Packages: https://archlinux.org/packages and https://aur.archlinux.org
- The Arch forums: https://bbs.archlinux.org
