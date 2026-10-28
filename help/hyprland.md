---
title: The Hyprland desktop
summary: How a tiling desktop works, and the keys that drive it.
section: Desktops
order: 21
---

# The Hyprland desktop

Hyprland is a **tiling** desktop: instead of windows lying over each other, each new window
takes a share of the screen, and the others make room. Almost everything is done with the
keyboard, which feels strange for a day and then fast.

The **main key** is the Windows key, or Alt if you chose that when installing. Below it is
written as **Win**.

## The keys of a new Hyprland

| Keys | Does |
| --- | --- |
| Win+Q | Opens a terminal |
| Win+C | Closes the active window |
| Win+arrow keys | Moves between windows |
| Win+1 ... Win+9 | Goes to workspace 1 to 9 (like several desktops) |
| Win+Shift+1 ... 9 | Moves the window to that workspace |
| Win+V | Lets the window float over the others (and back) |
| Win+J | Splits the next window the other way |
| Win+mouse drag | Moves a window; with the right button, resizes it |
| Win+M | Leaves Hyprland (the window closes) |

The first window, a terminal, opens by itself.

[Omarchy](omarchy.md) has its own, richer set of keys: Win+K shows them.

## Installing programs for it

A new Hyprland has a terminal and little else. Install what you like from the terminal, for
example on Arch:

```
sudo pacman -S firefox thunar wofi
```

Then start them from the terminal (`firefox &`), or bind a key to them.

## Changing it

Hyprland is configured in a text file in your home folder,
`~/.config/hypr/hyprland.conf` (or `hyprland.lua` in the newest versions). It reloads by itself
when you save it. The Hyprland wiki (wiki.hypr.land) explains every setting.
