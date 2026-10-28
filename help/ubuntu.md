---
title: Ubuntu
summary: The most popular Linux: first steps, apt in depth, PPAs, snaps and Flatpak, GNOME, upgrades, servers and common problems.
section: Your system
order: 40
---

# Ubuntu

Ubuntu is the Linux most guides and answers online are written for, so help is easy to find.
TuxBridge installs the **LTS** release (Long Term Support), which gets security updates for five
years; a new LTS comes every two years, in April of even years (24.04, 26.04…). Ubuntu is built
on Debian, and Linux Mint is built on Ubuntu, so what you learn here works there too.

**Good for:** a first Linux, programming, servers (most cloud servers run Ubuntu), following
tutorials, machine learning (NVIDIA's and others' guides assume Ubuntu).

## First things to do

1. **Update it:** **Update** in the system's **⋯** menu on My systems, or in a terminal:
   ```bash
   sudo apt update && sudo apt upgrade
   ```
2. **Install the basics** most tutorials assume:
   ```bash
   sudo apt install build-essential git curl wget unzip htop tree
   ```
3. **Set up Git** if you'll program: see [Git and SSH keys](git-ssh.md).
4. **Open its files in Windows:** **Files** on the system's card.
5. **Check which version you have:**
   ```bash
   lsb_release -a        # or: cat /etc/os-release
   ```

## apt, the package manager

```bash
sudo apt update                     # refresh the list of available packages (do this first)
sudo apt upgrade                    # install newer versions of everything
sudo apt full-upgrade               # the same, also adding/removing packages when needed
apt search <word>                   # find a package
apt show <name>                     # what it is, its version and size
sudo apt install <name>             # install it
sudo apt install ./file.deb         # install a downloaded .deb (note the ./)
sudo apt remove <name>              # remove it (keeps its settings)
sudo apt purge <name>               # remove it and its system-wide settings
sudo apt autoremove                 # remove what nothing needs any more
apt list --installed | grep <word>  # is it installed?
apt list --upgradable               # what an upgrade would change
apt-cache policy <name>             # which versions exist and where from
```

`apt` is for people; `apt-get` and `apt-cache` are the older commands scripts use. Both work.

### Finding the right package name

Package names are not always the program's name: `python3-pip`, `libssl-dev`,
`fonts-firacode`. Try `apt search <word>`, or search packages.ubuntu.com, which also tells you
which package contains a file. Once installed, `dpkg -L <name>` lists its files.

### "-dev" packages

Building software (`pip install` of some packages, compiling C code) often asks for headers,
which come in `-dev` packages: an error about `openssl/ssl.h` means
`sudo apt install libssl-dev`; `Python.h` means `python3-dev`.

## More software: PPAs, Flatpak, snaps, .deb files

| Source | When | How |
| --- | --- | --- |
| **apt** (Ubuntu's repositories) | First choice: tested with your Ubuntu | `sudo apt install <name>` |
| **A PPA** | A project's own newer versions | `sudo add-apt-repository ppa:<owner>/<name>` then `sudo apt update` |
| **A vendor repository** | Docker, VS Code, Chrome, Node… | Follow the vendor's "Ubuntu" instructions (they add a signed source) |
| **Flatpak** | Desktop apps, newest versions | `sudo apt install flatpak`, then `flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo` and `flatpak install flathub <app>` |
| **.deb file** | A download from a website | `sudo apt install ./file.deb` |
| **Snap** | Ubuntu's own app format | Often doesn't work in WSL (see below) |

PPAs and vendor sources are run by others: add only ones you trust. To remove a PPA:
`sudo add-apt-repository --remove ppa:<owner>/<name>`.

### Snaps in WSL

Some Ubuntu packages, including **Firefox** and **Chromium**, install as *snaps*, which depend on
things WSL may not provide, so they can fail to start. Use another source:

- **Firefox:** Mozilla's own apt repository (see support.mozilla.org, "Install Firefox on
  Linux"), or `flatpak install flathub org.mozilla.firefox`.
- **Anything else:** look for a Flatpak or the vendor's `.deb`.

`snap list` shows installed snaps; `sudo snap remove <name>` removes one.

## The GNOME desktop

Ubuntu's desktop is GNOME with Ubuntu's look: the dock on the left, the Yaru theme. Press the
**Windows key** and type to start a program. Full guide: [The GNOME desktop](gnome.md).

Things that need a real PC (Wi-Fi, Bluetooth, power, other screens) show in Settings but do
nothing here: Windows handles them.

## Services and servers

Ubuntu is the usual base for servers, and TuxBridge turns on systemd, so services work as on a
real server:

```bash
sudo apt install nginx              # a web server
sudo systemctl status nginx         # is it running?
sudo systemctl enable --now nginx   # start it, and at every start
```

Open `http://localhost` in your Windows browser to see it. Databases, Redis, SSH servers work the
same way. For trying things out, [Docker](docker.md) is often cleaner.

## Language and time

```bash
sudo dpkg-reconfigure locales       # add languages (e.g. de_DE.UTF-8)
sudo update-locale LANG=en_GB.UTF-8 # make one the default (next start)
timedatectl                          # time and time zone (WSL follows Windows' clock)
sudo timedatectl set-timezone Europe/Berlin
```

## Upgrading to the next LTS

Two ways:

- **In place:** `sudo do-release-upgrade` once it's offered (usually a few months after
  release, with the first point release such as 26.04.1). It takes a while and asks questions;
  answer the defaults if unsure. Back up first (see [Backing up a system](backups.md)).
- **Side by side (safer):** install the new version with **Get Linux** under another name, move
  your files and projects over, then remove the old one.

`sudo do-release-upgrade -d` would upgrade to a development version: avoid that.

## Common problems

| Problem | Fix |
| --- | --- |
| `Could not get lock /var/lib/dpkg/lock-frontend` | Another update is running (maybe **Update** from the app, or `unattended-upgrades`). Wait a few minutes. |
| `E: Unable to locate package` | Run `sudo apt update` first; check the name with `apt search`; it may be in `universe` (`sudo add-apt-repository universe`). |
| `dpkg was interrupted` | `sudo dpkg --configure -a`, then `sudo apt install -f` |
| `Unmet dependencies` | `sudo apt install -f`; if a PPA caused it, remove the PPA and update. |
| A snap program does not start | Use the apt, Flatpak or vendor version (above). |
| `sudo: unable to resolve host` | Harmless; restart the system (**Stop**, then open it). |
| `GPG error … NO_PUBKEY` | A third-party source's key is missing or expired: reinstall it following the vendor's current instructions. |
| `Release file … is not valid yet` | The clock is off after sleep: `sudo hwclock -s`, or restart WSL. |
| Very slow `apt` downloads | Choose a nearer mirror: edit `/etc/apt/sources.list.d/ubuntu.sources` (or `/etc/apt/sources.list` on older releases). |

## Use cases

- **Learning to program:** Python, Node, Go and C all install with `apt`; see the
  *Programming* guides.
- **Practising server administration:** nginx, PostgreSQL, SSH, firewalls (`ufw`), systemd —
  the same commands as on a cloud server.
- **Machine learning:** CUDA, PyTorch and Jupyter; see [Graphics, the GPU and Linux apps](gpu.md).
- **A Linux desktop on Windows:** GNOME in a window, with Linux-only apps.

## Learn more

- Ubuntu's documentation: https://help.ubuntu.com and https://documentation.ubuntu.com
- Ask Ubuntu (questions and answers): https://askubuntu.com
- Package search: https://packages.ubuntu.com
- The **Learn** page's lessons all work on Ubuntu.
