---
title: The Xfce desktop
summary: Debian's light and quick desktop: the panel, Whisker menu, settings and shortcuts.
section: Desktops
order: 24
---

# The Xfce desktop

Xfce is a light, quick, traditional desktop: menus, panels, windows with buttons. It uses little
memory, so it starts fast and stays responsive. It is the desktop TuxBridge gives Debian.

## The panels and the menu

A new Xfce usually has a panel at the top (menu, window buttons, clock, tray) and a small dock
at the bottom (terminal, files, browser).

- **Applications** (top left) opens the menu. Many people prefer the **Whisker Menu**, with
  search: `sudo apt install xfce4-whiskermenu-plugin`, then right-click the panel →
  **Panel → Add New Items… → Whisker Menu**.
- Right-click a panel → **Panel → Panel Preferences** to move it, size it, make it auto-hide, or
  add items (a CPU graph, a workspace switcher, launchers).
- **Alt+F2** (or **Alt+F3**) opens the application finder.

## Windows

| Do | How |
| --- | --- |
| Maximise | Double-click the title bar, or **Alt+F10** |
| Tile left / right | Drag to the edge, or **Win+Left/Right** on newer Xfce |
| Switch | **Alt+Tab** |
| Move with the keyboard | **Alt+F7**, then arrow keys |
| Close | **Alt+F4** |

## Workspaces

Xfce starts with several workspaces. **Ctrl+Alt+Left/Right** switches; **Ctrl+Alt+Shift+arrow**
takes the window along. The number is set in *Settings → Workspaces*.

## Settings

Menu → **Settings → Settings Manager** has everything:

- **Appearance:** style (light/dark), icons, fonts.
- **Window Manager:** title-bar theme, keyboard shortcuts for windows.
- **Keyboard → Application Shortcuts:** your own shortcuts, e.g. **Ctrl+Alt+T** for
  `xfce4-terminal`.
- **Desktop:** wallpaper, and which icons show on the desktop.
- **Session and Startup:** programs that start with the desktop.

More themes: `sudo apt install arc-theme papirus-icon-theme`, then pick them in Appearance.

## Apps that come with it

| App | For |
| --- | --- |
| Thunar | Files; **Ctrl+L** types a path, **Ctrl+H** shows hidden files, right-click → Open Terminal Here |
| Xfce Terminal | Commands; tabs with **Ctrl+Shift+T** |
| Mousepad | Text editor |
| Ristretto | Images |
| Task Manager | Running programs |

## If something looks wrong

- Panel gone: run `xfce4-panel &` in a terminal.
- Everything back to default (files stay): close the desktop window, then in a terminal
  `mv ~/.config/xfce4 ~/.config/xfce4.old`, and open the desktop again.
- Keyboard shortcut doesn't work: TuxBridge's window passes most keys to Linux, but Windows keeps
  Ctrl+Alt+Del and Win+L.
