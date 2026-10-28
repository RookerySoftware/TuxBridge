---
title: Omarchy
summary: DHH's ready-made Hyprland desktop on Arch: first start, its keys, the menu, themes, installing apps, updates and problems.
section: Your system
order: 46
---

# Omarchy

Omarchy, by DHH (the maker of Ruby on Rails), is Arch Linux with a finished Hyprland desktop:
themes, a launcher, a menu for everything, and keys for all of it. It is opinionated: it picks
the apps and the look for you, and you change what you like afterwards.

**Good for:** a beautiful keyboard-driven desktop that works out of the box, programming
(editors, languages and databases install from its menu).

## The first start

The first time you open its desktop, Omarchy asks for your account, keyboard and time zone, like
a new PC, and a terminal window finishes the setup. Let it run; it takes a few minutes. After
that it opens straight to the desktop.

## The keys that matter

In Pengi's window, the Windows key (written **Win** here; Omarchy calls it *Super*) goes to
Omarchy. See [Using a desktop window](desktop-window.md).

| Keys | Does |
| --- | --- |
| Win+K | **Shows every key binding** — the one to remember |
| Win+Space | Starts a program (type its name) |
| Win+Alt+Space | The Omarchy menu: install, update, themes, settings, all of it |
| Win+Return | Opens a terminal |
| Win+W | Closes the window |
| Win+1 … Win+9 | Goes to workspace 1 to 9 |
| Win+Shift+1 … 9 | Moves the window to that workspace |
| Win+arrow keys | Moves between windows |

Omarchy's keys change between releases; Win+K always shows the current ones.

## The Omarchy menu

**Win+Alt+Space** opens it. From there:

- **Install:** apps, programming languages (*Development*), editors, AI tools, games…
- **Remove:** what you installed through it.
- **Style:** themes (one theme changes the whole desktop at once), backgrounds, fonts.
- **Setup:** keyboard, monitors, default apps, config files.
- **Update:** Omarchy and the system together.
- **Learn:** the manual and the keybindings.

## Installing programs

The menu's **Install** is easiest. Underneath, Omarchy is Arch:

```bash
sudo pacman -S <name>            # from Arch's repositories
yay -S <name>                    # from the AUR (Omarchy includes yay)
```

Omarchy can also turn websites into apps of their own; look under **Install** in the menu.

## Updating

The Omarchy menu has **Update**, or run `omarchy-update` in a terminal. **Update** in the
system's **⋯** menu on My systems opens a terminal with it, because it asks for your password.
Avoid updating with plain `pacman -Syu` alone, which skips Omarchy's own migration steps.

## Changing it

Omarchy keeps your settings in plain files you're meant to edit:

| Folder | For |
| --- | --- |
| `~/.config/hypr/` | Hyprland: keys (`bindings.conf`), monitors, input, look |
| `~/.config/waybar/` | The top bar |
| `~/.config/omarchy/` | Omarchy's themes and your choices |
| `~/.local/share/omarchy/` | Omarchy itself (updated by `omarchy-update`; don't edit) |

The menu's **Setup → Config** opens them. Hyprland reloads its files as soon as you save.

## Good to know

- Everything in the [Arch Linux guide](arch.md) applies underneath, and the
  [Hyprland guide](hyprland.md) explains tiling.
- Some Omarchy features need a real PC (screen brightness, battery, Wi-Fi, Bluetooth) and do
  nothing in WSL.
- It is the largest system here (about 11 GB); install it on a drive with room.
- The manual is at https://learn.omacom.io (the Omarchy manual).

## Common problems

| Problem | Fix |
| --- | --- |
| The first-start terminal stops with an error | Close the desktop window and open it again; the setup continues. If it keeps failing, note the message and remove and reinstall. |
| A key does nothing | Check Win+K for the current binding; some keys (Win+L, Ctrl+Alt+Del) stay with Windows. |
| An update fails partway | Run `omarchy-update` again in a terminal and read its message. |
| Something looks broken after changing config | Undo your last edit; Hyprland shows config errors at the top of the screen. |
| `pacman` signature errors | `sudo pacman -Sy archlinux-keyring && sudo pacman -Su`, then `omarchy-update`. |

## Use cases

- **A ready-made developer desktop:** Neovim, VS Code, languages and Docker from one menu.
- **Learning tiling and keyboard-driven work** without building a setup yourself.

## Learn more

- The Omarchy manual: https://learn.omacom.io
- The Arch Wiki: https://wiki.archlinux.org
- The Hyprland wiki: https://wiki.hypr.land
