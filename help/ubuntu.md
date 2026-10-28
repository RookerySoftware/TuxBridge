---
title: Ubuntu
summary: The most popular Linux, its GNOME desktop, software, upgrades and common problems.
section: Your system
order: 40
---

# Ubuntu

Ubuntu is the Linux most guides and answers online are written for, so help is easy to find.
EasyWSL installs the **LTS** release (Long Term Support), which gets security updates for five
years; a new LTS comes every two years. Ubuntu is built on Debian, and Linux Mint is built on
Ubuntu, so what you learn here works there too.

**Good for:** a first Linux, programming, servers (most cloud servers run Ubuntu), following
tutorials.

## First things to do

1. **Update it:** **Update** in the system's **⋯** menu on My systems, or in a terminal:
   ```bash
   sudo apt update && sudo apt upgrade
   ```
2. **Install a few basics** most tutorials assume:
   ```bash
   sudo apt install build-essential git curl wget unzip htop
   ```
3. **Open its files in Windows:** **Files** on the system's card.

## Everyday commands

```bash
sudo apt update && sudo apt upgrade   # update everything
apt search <word>                     # find a program
apt show <name>                       # what a package is, and its size
sudo apt install <name>               # install it
sudo apt remove <name>                # remove it
sudo apt autoremove                   # remove what nothing needs any more
```

See [Installing software](software.md).

## The desktop: GNOME

Ubuntu's own desktop is GNOME, with Ubuntu's look (the dock on the left, the Yaru theme).

- **Activities** (top left, or the Windows key) shows your windows and a search: type a
  program's name and press Enter.
- The **dock** holds favourite apps; right-click one to add or remove it.
- **Settings** (in the top-right menu) has appearance, the background and more.
- **Files** is the file manager; **Text Editor** edits text files.
- To add programs with a desktop, install them with `apt`; they appear in Activities.

Things that need a real PC (Wi-Fi, Bluetooth, power, other screens) show in Settings but do
nothing here: WSL and Windows handle them.

## Common problems

| Problem | Fix |
| --- | --- |
| `Could not get lock /var/lib/dpkg/lock` | Another update is running (maybe **Update** from the app). Wait a few minutes. |
| `E: Unable to locate package` | Run `sudo apt update` first, and check the name with `apt search`. |
| A `snap` program does not start | Snaps rely on things WSL lacks; install the `apt` version or a Flatpak. |
| `sudo: unable to resolve host` | Harmless; restart the system (**Stop** on its card, then open it). |

## Upgrading to the next LTS

`sudo do-release-upgrade` once it is offered (usually a few months after release). Or install
the new version beside the old one with **Get Linux** and move your files over, which is safer.

## Learn more

- Ubuntu's documentation: https://help.ubuntu.com
- Ask Ubuntu (questions and answers): https://askubuntu.com
- The **Learn** page's lessons all work on Ubuntu.
