---
title: Using a desktop window
summary: Full screen, the Windows key, size and smoothness, and what closing the window does.
section: Desktops
order: 20
---

# Using a desktop window

A system's desktop (GNOME, Cinnamon, Xfce or Hyprland) opens in a window of its own. The
picture comes straight from Linux through WSL; no network or remote-desktop program is
involved.

## Full screen

- **Ctrl+Alt+Enter** switches between full screen and a window, at any time.
- Or open the window's menu (its icon at the top left, or **Alt+Space**) and choose
  **Full screen**.
- A note at the top says how to leave full screen for a few seconds after it starts.

**Settings → Desktops → Open full screen** makes desktops start that way.

## Keyboard

While the window is active, keys go to Linux, including the **Windows key**, which most Linux
desktops use for their menu and shortcuts. Windows keeps a few for itself: **Ctrl+Alt+Del**
and **Windows+L** (locking the PC) always work.

## Size and smoothness

**Settings → Desktops**:

- **Resolution:** *Automatic* uses the screen's full resolution in full screen and the window's
  size in a window. A fixed size (4K, 1440p, 1080p) is scaled to the window. Text is scaled
  like Windows scales it, so things keep a comfortable size.
- **Frame rate:** the most pictures a second the desktop sends. For now desktops reach about 50
  to 60 a second at 1080p and 30 to 60 at 4K, depending on the desktop and how much of the
  screen changes.

Changes apply the next time a desktop opens.

## Closing

Closing the window **ends the desktop session**, like signing out: save your work first.
Programs started from the desktop close with it. The system itself keeps running for the
terminal; **Stop** on its card stops it completely.

## Not there yet

- Copy and paste between Windows and the desktop. Meanwhile, share text through a file, for
  example in your Windows Documents folder (`/mnt/c/Users/you/Documents` in Linux).
