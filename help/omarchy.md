---
title: Omarchy
summary: DHH's ready-made Hyprland desktop on Arch, and its keys.
section: Your system
order: 46
---

# Omarchy

Omarchy, by DHH (the maker of Ruby on Rails), is Arch Linux with a finished Hyprland desktop:
themes, a launcher, a menu for everything, and keys for all of it. It is in beta.

The first time you open its desktop, a terminal window finishes the setup (your account and a
few questions).

## The keys that matter

| Keys | Does |
| --- | --- |
| Win+Space | Starts a program (type its name) |
| Win+Alt+Space | The Omarchy menu: install, update, themes, settings, all of it |
| Win+K | Shows every key binding |
| Win+Return | Opens a terminal |
| Win+W | Closes the window |
| Win+1 ... Win+9 | Goes to workspace 1 to 9 |

Win+K is the one to remember: it shows the rest.

## Updating

The Omarchy menu (Win+Alt+Space) has **Update**, which updates Omarchy and the system together;
or run `omarchy-update` in a terminal. Avoid updating with plain `pacman -Syu` alone, which
skips Omarchy's own steps.

## Good to know

- Everything in the [Arch Linux guide](arch.md) applies underneath.
- The manual is at learn.omacom.io (the Omarchy manual).
- Some Omarchy features need a real PC (screen brightness, power, Wi-Fi) and do nothing in
  WSL.
