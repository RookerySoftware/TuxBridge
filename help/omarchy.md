---
title: Omarchy
summary: DHH's ready-made Hyprland desktop on Arch: its keys, menu, themes and updates.
section: Your system
order: 46
---

# Omarchy

Omarchy, by DHH (the maker of Ruby on Rails), is Arch Linux with a finished Hyprland desktop:
themes, a launcher, a menu for everything, and keys for all of it. It is in beta.

**Good for:** a beautiful keyboard-driven desktop that works out of the box, programming.

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
| Win+Shift+1 ... 9 | Moves the window to that workspace |
| Win+arrow keys | Moves between windows |

Win+K is the one to remember: it shows the rest. In EasyWSL's window, the Windows key goes to
Omarchy (see [Using a desktop window](desktop-window.md)).

## The Omarchy menu

Win+Alt+Space opens it. From there:

- **Install**: apps, programming languages, editors
- **Style**: themes (Omarchy's themes change the whole desktop at once), backgrounds, fonts
- **Update**: Omarchy and the system together
- **Setup**: keyboard, monitors, default apps

## Updating

The Omarchy menu has **Update**, or run `omarchy-update` in a terminal. **Update** in the
system's **⋯** menu on My systems opens a terminal with it, because it asks for your password.
Avoid updating with plain `pacman -Syu` alone, which skips Omarchy's own steps.

## Good to know

- Everything in the [Arch Linux guide](arch.md) applies underneath.
- The manual is at https://learn.omacom.io (the Omarchy manual).
- Some Omarchy features need a real PC (screen brightness, power, Wi-Fi) and do nothing in
  WSL.
- It is the largest system here (about 11 GB); install it on a drive with room.
