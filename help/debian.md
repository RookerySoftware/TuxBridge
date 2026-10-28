---
title: Debian
summary: The steady base of Ubuntu and Mint: apt, sources and components, backports, Xfce, upgrades between releases and common problems.
section: Your system
order: 42
---

# Debian

Debian is one of the oldest Linux distributions and the base of Ubuntu, Mint and many others.
Its **stable** release changes slowly and carefully: programs are a little older, and things
rarely break. A new stable release comes about every two years, and each gets about three years
of full support (more with LTS). Debian is made by volunteers, with no company behind it.

**Good for:** servers, things that must keep working, light systems, learning what the others
are built on.

## First things to do

```bash
sudo apt update && sudo apt upgrade
sudo apt install curl wget git htop unzip build-essential
cat /etc/os-release          # which release you have
```

## apt, the package manager

```bash
sudo apt update && sudo apt upgrade   # update everything
apt search <word>                     # find a program
apt show <name>                       # about it, with its version
sudo apt install <name>               # install it
sudo apt install ./file.deb           # install a downloaded .deb
sudo apt remove <name>                # remove it (purge also removes settings)
sudo apt autoremove                   # tidy up
apt-cache policy <name>               # versions available and where from
```

packages.debian.org searches every package and file in every release.

## Releases

Debian names releases after Toy Story characters: 12 is *bookworm*, 13 *trixie*, 14 will be
*forky*. There are three branches:

| Branch | Is |
| --- | --- |
| **stable** | The current release: what TuxBridge installs. Security fixes only. |
| **testing** | The next release, being prepared: newer, occasionally broken. |
| **unstable** (*sid*) | Where new packages arrive first. |

Stay on stable unless you know why you want the others.

## Sources and components

Where `apt` gets packages is set in `/etc/apt/sources.list.d/debian.sources` (newer releases)
or `/etc/apt/sources.list` (older ones). Each source has **components**:

| Component | Holds |
| --- | --- |
| `main` | Free software (the default) |
| `contrib` | Free software that needs something non-free |
| `non-free` | Software with restrictions (some codecs, `unrar`) |
| `non-free-firmware` | Firmware for hardware (not needed in WSL) |

To add `contrib non-free`, edit the file (`sudo nano /etc/apt/sources.list.d/debian.sources`)
and extend the `Components:` line to `main contrib non-free non-free-firmware`, then
`sudo apt update`.

## Backports: newer versions of a few programs

Backports are newer programs rebuilt for stable. Add the source once (replace *trixie* with your
release's name):

```bash
echo "deb http://deb.debian.org/debian trixie-backports main" | sudo tee /etc/apt/sources.list.d/backports.list
sudo apt update
sudo apt install -t trixie-backports <name>
```

Only packages you ask for come from backports; the rest stays stable.

Other ways to get newer software: Flatpak (`sudo apt install flatpak`, then add Flathub as in
the [Ubuntu guide](ubuntu.md)), a language's own installer (rustup, uv, nvm), or Docker.

## The desktop: Xfce

TuxBridge gives Debian the light, quick Xfce desktop. Full guide: [The Xfce desktop](xfce.md).

## Upgrading to the next release

When a new stable release comes out:

1. Update fully: `sudo apt update && sudo apt full-upgrade`.
2. Back up (see [Backing up a system](backups.md)).
3. Replace the old name with the new one in the sources (e.g. `bookworm` → `trixie`):
   ```bash
   sudo sed -i 's/bookworm/trixie/g' /etc/apt/sources.list /etc/apt/sources.list.d/*
   ```
4. `sudo apt update && sudo apt upgrade --without-new-pkgs`, then `sudo apt full-upgrade`.
5. Stop and open the system again.

Debian's release notes describe each upgrade; read them first. Installing the new release
beside the old one with **Get Linux** is the safe alternative.

## Small differences from Ubuntu

- No snaps, no PPAs (PPAs are built for Ubuntu and may break Debian).
- Some commands live in `/usr/sbin`, which normal users' `PATH` may lack: if `adduser` or
  `ip` is "not found", try `sudo` or the full path.
- Fewer things installed by default: you may need `sudo apt install man-db less bash-completion`.

## Common problems

| Problem | Fix |
| --- | --- |
| `sudo: command not found` | Systems made by TuxBridge have sudo. Otherwise, from PowerShell: `wsl -d <system> -u root apt install sudo`, then `usermod -aG sudo <you>`. |
| `<you> is not in the sudoers file` | `wsl -d <system> -u root usermod -aG sudo <you>`, then restart the system. |
| A program is older than you need | Backports, Flatpak, the language's own installer, or Docker. |
| `Package has no installation candidate` | It may need `contrib`/`non-free` (above), or doesn't exist in your release. |
| `The repository … does not have a Release file` | The release is old and moved to archive.debian.org, or a name is misspelled in sources. |
| `command not found` for system tools | They're in `/usr/sbin`: use `sudo`, or add `/usr/sbin` to PATH. |

## Use cases

- **A small, stable server** for web sites, databases or home tools.
- **A light desktop** with Xfce.
- **Understanding Ubuntu and Mint**: what they add on top.

## Learn more

- The Debian wiki: https://wiki.debian.org
- The Debian administrator's handbook: https://debian-handbook.info
- Package search: https://packages.debian.org
