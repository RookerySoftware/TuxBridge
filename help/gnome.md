---
title: The GNOME desktop
summary: Activities, the dock, settings, extensions and shortcuts of Ubuntu's desktop.
section: Desktops
order: 22
---

# The GNOME desktop

GNOME is Ubuntu's desktop: clean, with one bar at the top and a dock. Ubuntu adds its own look
(the Yaru theme, the dock on the left). It works differently from Windows in one main way:
instead of a Start menu, you press the **Windows key** and type.

## Finding and starting programs

- Press the **Windows key** (or click **Activities** at the top left). You see your open windows
  and a search box.
- **Type** part of a program's name (`files`, `term`, `fire`) and press **Enter**.
- **Show Apps** (the grid at the bottom of the dock) lists everything installed.
- Right-click an app in the dock → **Pin to Dash** (or *Add to Favorites*) keeps it there.

## Windows

| Do | How |
| --- | --- |
| Maximise | Double-click the title bar, or **Win+Up** |
| Snap left / right half | **Win+Left** / **Win+Right**, or drag to the edge |
| Minimise | **Win+H** (there is no minimise button by default; see Tweaks) |
| Switch windows | **Alt+Tab**; **Alt+`** between windows of the same app |
| Close | **Ctrl+Q** in most apps, or **Alt+F4** |

## Workspaces

GNOME makes extra desktops (workspaces) as you need them: in Activities, drag a window to the
right. **Ctrl+Alt+Left/Right** (Ubuntu: also **Win+Page Up/Down**) moves between them.

## Shortcuts

| Keys | Does |
| --- | --- |
| Win | Activities and search |
| Win+A | All apps |
| Ctrl+Alt+T | Terminal (Ubuntu) |
| Win+V | Notifications and calendar |
| Win+L | Locks the screen (in the desktop window; Windows' own Win+L locks the PC) |
| Print Screen | Screenshot tool |
| Alt+F2 | Run a command |

*Settings → Keyboard → View and Customize Shortcuts* lists and changes all of them.

## Settings

The menu at the **top right** (network, sound, power icons) has **Settings**. Useful parts:

- **Appearance:** light or dark style, accent colour, the dock's position and size (Ubuntu).
- **Background:** the wallpaper.
- **Displays:** scaling (in EasyWSL, size is set by the window; see
  [Using a desktop window](desktop-window.md)).
- **Keyboard:** layouts and shortcuts.

Wi-Fi, Bluetooth, power and printers show in Settings but do nothing here: Windows handles them.

## Tweaks and extensions

```bash
sudo apt install gnome-tweaks gnome-shell-extension-manager
```

- **Tweaks** adds minimise/maximise buttons (*Windows → Titlebar Buttons*), fonts and more.
- **Extension Manager** installs GNOME extensions: for example *Dash to Panel* (a Windows-like
  taskbar) or *ArcMenu* (a Start-style menu). Extensions can break with GNOME updates; turn one
  off if the desktop misbehaves.

## Apps that come with it

| App | For |
| --- | --- |
| Files (Nautilus) | Browsing files; **Ctrl+L** types a path, **Ctrl+H** shows hidden files |
| Terminal | Commands |
| Text Editor | Plain text |
| Settings | Everything above |

More with `sudo apt install`: `firefox`, `gnome-software` (an app store with a window),
`eog` (images), `evince` (PDFs), `gnome-system-monitor`.

## Ubuntu and snaps

On Ubuntu, Firefox and the App Center are *snaps*, which need things WSL lacks and may not start.
Use apt versions or Flatpak instead (see [Ubuntu](ubuntu.md)).

## If something looks wrong

- Panel or dock missing after an update: close the desktop window and open it again.
- A frozen desktop: close the window; if it won't reopen, **Stop** the system and open it again.
- Reset GNOME's settings to defaults (your files stay): `dconf reset -f /org/gnome/` in a
  terminal, then reopen the desktop.
