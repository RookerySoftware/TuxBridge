---
title: Debian
summary: The steady base of Ubuntu and Mint, with the light Xfce desktop.
section: Your system
order: 42
---

# Debian

Debian is one of the oldest Linux distributions and the base of Ubuntu and Mint. Its
**stable** release changes slowly and carefully: programs are a little older, and things rarely
break. A new stable release comes about every two years.

## Everyday commands

```
sudo apt update && sudo apt upgrade   # update everything
sudo apt install <name>               # install a program
```

## The desktop: Xfce

Xfce is light and quick, with a classic layout: **Applications** menu at the top left, open
windows in the panel, settings under **Applications → Settings**.

## Good to know

- Newer versions of a few programs are in **backports**; the Debian wiki explains how to use
  them (wiki.debian.org/Backports).
- Debian ships only free software by default. Some drivers and fonts are in `non-free`; most
  people in WSL never need them.
