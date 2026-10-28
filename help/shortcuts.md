---
title: Keyboard shortcuts
summary: The terminal, nano, less and man, Windows Terminal, EasyWSL's desktop window, and each desktop.
section: Reference
order: 81
---

# Keyboard shortcuts

## In the terminal (bash)

| Keys | Does |
| --- | --- |
| Tab | Completes a command or file name; twice shows the choices |
| Up / Down | Earlier / later commands |
| Ctrl+R | Searches earlier commands (again for older ones; Esc to edit) |
| Ctrl+C | Stops the running command (or abandons the line) |
| Ctrl+D | Ends input; on an empty line, closes the shell |
| Ctrl+Z | Pauses the running command (`fg` resumes it, `bg` sends it to the background) |
| Ctrl+L | Clears the screen |
| Ctrl+A / Ctrl+E | Start / end of the line |
| Alt+B / Alt+F | Back / forward one word |
| Ctrl+W | Deletes the word before the cursor |
| Ctrl+U / Ctrl+K | Deletes to the start / end of the line |
| Ctrl+Y | Pastes what Ctrl+W, U or K deleted |
| Alt+. | Inserts the last word of the previous command |
| Ctrl+X Ctrl+E | Edits the current command in your editor |
| `!!` | The previous command (`sudo !!`) |

## Windows Terminal

| Keys | Does |
| --- | --- |
| Ctrl+Shift+C / Ctrl+Shift+V | Copy / paste (or right-click) |
| Ctrl+Shift+T | New tab (of the default profile) |
| Ctrl+Shift+1…9 | New tab of profile 1…9 (each Linux system has one) |
| Ctrl+Tab | Next tab |
| Alt+Shift+D | Split the pane |
| Alt+Shift+- / Alt+Shift++ | Split horizontally / vertically |
| Ctrl+Shift+W | Close the pane or tab |
| Ctrl+= / Ctrl+- / Ctrl+0 | Bigger / smaller / normal text |
| Ctrl+Shift+F | Find in the output |
| F11 | Full screen |
| Ctrl+, | Settings |

## nano

| Keys | Does |
| --- | --- |
| Ctrl+O, Enter | Save |
| Ctrl+X | Exit |
| Ctrl+W | Search; Alt+W next |
| Ctrl+\\ | Replace |
| Ctrl+K / Ctrl+U | Cut line / paste |
| Alt+U / Alt+E | Undo / redo |
| Ctrl+_ | Go to line |
| Alt+N | Line numbers |

## less and man

| Keys | Does |
| --- | --- |
| Space / b | Page down / up |
| /word, then n / N | Search forward, next / previous match |
| g / G | Top / bottom |
| q | Quit |

## vim (the minimum)

**Esc** then `:q!` Enter quits without saving; `:wq` Enter saves and quits; `i` starts typing.
More in [Editing text files](editors.md).

## tmux

All after **Ctrl+B**: `c` new window, `n`/`p` next/previous, `%` split side by side, `"` split
top/bottom, arrows move between panes, `d` detach (`tmux attach` to return).

## EasyWSL's desktop window

| Keys | Does |
| --- | --- |
| Ctrl+Alt+Enter | Full screen ↔ window |
| Alt+Space | The window's menu |
| Ctrl+Alt+Del, Win+L | Always go to Windows |

Other keys, including the Windows key, go to Linux while the window is active. See
[Using a desktop window](desktop-window.md).

## GNOME (Ubuntu)

| Keys | Does |
| --- | --- |
| Win | Activities and search |
| Win+A | All apps |
| Ctrl+Alt+T | Terminal (Ubuntu) |
| Win+Left / Right / Up | Snap left / right, maximise |
| Win+H | Minimise |
| Alt+Tab, Alt+` | Switch apps, windows of one app |
| Ctrl+Alt+Left / Right | Switch workspace |
| Print Screen | Screenshot |

## Cinnamon (Linux Mint)

| Keys | Does |
| --- | --- |
| Win | Menu |
| Ctrl+Alt+T | Terminal |
| Win+E | Files |
| Win+D | Show desktop |
| Win+arrows | Snap windows |
| Ctrl+Alt+Up / Down | All workspaces / all windows |
| Ctrl+Alt+Esc | Restart Cinnamon |

## Xfce (Debian)

| Keys | Does |
| --- | --- |
| Alt+F2 / Alt+F3 | Run / application finder |
| Alt+F10 | Maximise |
| Alt+F7 | Move window |
| Ctrl+Alt+Left / Right | Switch workspace |
| Ctrl+Alt+D | Show desktop |

## Hyprland (EasyWSL's setup)

| Keys | Does |
| --- | --- |
| Win+Q | Terminal |
| Win+C | Close window |
| Win+arrows | Move focus |
| Win+1…9 | Workspace 1…9 |
| Win+Shift+1…9 | Move window to workspace |
| Win+V | Float / tile |
| Win+M | Leave Hyprland |

Omarchy has its own: **Win+K** shows them all. See [Omarchy](omarchy.md) and
[The Hyprland desktop](hyprland.md).
