---
title: The Cinnamon desktop
summary: Linux Mint's Windows-like desktop: the menu, panel, applets, themes and shortcuts.
section: Desktops
order: 23
---

# The Cinnamon desktop

Cinnamon is Linux Mint's desktop, and the most familiar one for Windows users: a **menu** at
the bottom left, a **panel** (taskbar) along the bottom, a system tray and a clock on the right.

## The menu

Click the menu button (bottom left) or press the **Windows key**, then type to search or browse
the categories. Right-click any app to **Add to panel**, **Add to desktop** or **Add to
favourites** (the column on the left of the menu).

## The panel

- Running windows show as buttons; click to switch, middle-click (or right-click) for more.
- Right-click the panel → **Panel settings** to move it, change its height, or turn on auto-hide.
- Right-click → **Applets** adds little tools: a workspace switcher, weather, a CPU graph…

## Windows

| Do | How |
| --- | --- |
| Snap to a half or quarter | Drag to the edge or corner, or **Win+arrow keys** |
| Maximise | Double-click the title bar |
| Switch | **Alt+Tab** |
| Show the desktop | **Win+D** |
| Close | **Alt+F4** |

## Workspaces

**Ctrl+Alt+Up** shows all workspaces (*Expo*); **Ctrl+Alt+Left/Right** switches between them;
**Ctrl+Alt+Down** shows all windows in this one.

## Shortcuts

| Keys | Does |
| --- | --- |
| Win | The menu |
| Ctrl+Alt+T | Terminal |
| Win+E | Home folder in Files (Nemo) |
| Win+D | Show the desktop |
| Print Screen | Screenshot |
| Ctrl+Alt+Esc | Restarts Cinnamon itself if it misbehaves (windows stay open) |

*System Settings → Keyboard → Shortcuts* changes them.

## System Settings

Menu → **System Settings** (or `cinnamon-settings`). Worth knowing:

- **Themes:** light or dark, colours, icons, mouse pointer. *Add/Remove* downloads more.
- **Backgrounds**, **Fonts**, **Effects** (animations; turn off for a snappier feel).
- **Applets, Desklets, Extensions:** add-ons for the panel, the desktop and the window manager.
- **Startup Applications:** what starts with the desktop.

## Apps that come with Mint

| App | For |
| --- | --- |
| Nemo (Files) | File manager with tabs, split view (**F3**), and an "Open in terminal" menu |
| Xed | Text editor |
| Software Manager | An app store with a window (includes Flatpaks) |
| Update Manager | Updates with a window (TuxBridge's **Update** works too) |
| Xviewer, Xreader | Images, PDFs |

## If something looks wrong

- **Ctrl+Alt+Esc** restarts Cinnamon without closing apps.
- Panel gone: right-click the desktop → **Add a new panel**, or reset all panels with
  `gsettings reset-recursively org.cinnamon` (settings go back to default; files stay).
- After a big update, close the desktop window and open it again.

More about Mint itself in [Linux Mint](mint.md).
