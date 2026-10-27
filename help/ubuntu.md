---
title: Ubuntu
summary: The most popular Linux, its GNOME desktop, and keeping it updated.
section: Your system
order: 40
---

# Ubuntu

Ubuntu is the Linux most guides and answers online are written for, so help is easy to find.
EasyWSL installs the **LTS** release (Long Term Support), which gets security updates for five
years; a new LTS comes every two years.

## Everyday commands

```
sudo apt update && sudo apt upgrade   # update everything
apt search <word>                     # find a program
sudo apt install <name>               # install it
```

See [Installing software](software.md).

## The desktop: GNOME

Ubuntu's own desktop is GNOME, with Ubuntu's look (the dock on the left, the Yaru theme).

- **Activities** (top left, or the Windows key) shows your windows and a search: type a
  program's name and press Enter.
- The **dock** holds favourite apps; right-click one to add or remove it.
- **Settings** (in the top-right menu) has appearance, the background and more.

Things that need a real PC (Wi-Fi, Bluetooth, power, other screens) show in Settings but do
nothing here: WSL and Windows handle them.

## Good to know

- `snap` packages (Ubuntu's own app format) may be slow to start or not work in WSL; prefer
  `apt` or Flatpak.
- Moving to the next LTS: `sudo do-release-upgrade`, once it is offered (usually a few months
  after release). Or install the new version beside the old one and move your files over.
