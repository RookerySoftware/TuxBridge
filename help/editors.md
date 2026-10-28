---
title: Editing text files
summary: nano for quick changes, surviving vim, micro, and editing Linux files with VS Code or Notepad.
section: Using Linux
order: 18
---

# Editing text files

Much of Linux is set up in plain text files: settings, scripts, notes. You will edit them often,
so it pays to pick an editor you are comfortable with.

| Editor | Where | Good for |
| --- | --- | --- |
| **nano** | In the terminal | Quick changes. Shows its keys at the bottom. The easiest. |
| **micro** | In the terminal | Like nano, but with Ctrl+S, Ctrl+C, Ctrl+V and the mouse, as on Windows. |
| **vim** / **vi** | In the terminal | Very fast once learned; on every Linux. Worth knowing how to leave. |
| **VS Code** | On Windows, connected to Linux | Projects, programming, many files at once. |
| **Text Editor**, **Kate**, **Mousepad** | On a Linux desktop | Like Notepad, with a window. |

## nano

```bash
nano notes.txt              # opens (or creates) notes.txt
sudo nano /etc/hosts        # a system file needs sudo
```

The bottom two lines list the keys; `^` means **Ctrl**, and `M-` means **Alt**.

| Keys | Does |
| --- | --- |
| Ctrl+O, then Enter | Saves (*Write Out*) |
| Ctrl+X | Leaves (asks to save if you changed something) |
| Ctrl+W | Searches (*Where is*); Alt+W finds the next one |
| Ctrl+\\ | Search and replace |
| Ctrl+K / Ctrl+U | Cuts the line / pastes it |
| Alt+U | Undoes |
| Ctrl+_ | Goes to a line number |
| Alt+N | Shows line numbers |

To paste from Windows into the terminal, right-click (Windows Terminal) or Ctrl+Shift+V.

If nano is missing: `sudo apt install nano` or `sudo pacman -S nano`.

## micro

micro behaves like a Windows editor: **Ctrl+S** saves, **Ctrl+Q** quits, **Ctrl+C**/**Ctrl+V**
copy and paste, **Ctrl+Z** undoes, **Ctrl+F** finds, and the mouse selects text.

```bash
sudo apt install micro      # Ubuntu, Mint, Debian
sudo pacman -S micro        # Arch, CachyOS, Omarchy
micro notes.txt
```

## vim: how to get out (and a little more)

Some commands open `vim` (or `vi`) for you, for example `git commit` without a message, or
`sudo visudo`. vim has **modes**: you start in *normal* mode, where letters are commands, not
text. That is why typing seems to do strange things.

**To leave:** press **Esc**, then type `:q!` and Enter to quit without saving, or `:wq` and Enter
to save and quit.

Enough vim to edit a file:

| Keys (in normal mode) | Does |
| --- | --- |
| `i` | Starts typing text (insert mode); **Esc** goes back to normal mode |
| `:w` Enter | Saves |
| `:q` Enter | Quits (`:q!` throws changes away) |
| `/word` Enter | Finds *word*; `n` finds the next |
| `dd` | Deletes (cuts) the line |
| `p` | Pastes after the cursor |
| `u` / Ctrl+R | Undo / redo |
| `gg` / `G` | Goes to the first / last line |

`vimtutor` (in the terminal) is a 30-minute lesson, if you want to learn it properly.

### Make another editor the default

Programs like `git` open the editor named in the `EDITOR` variable. To use nano everywhere, add
this line to `~/.bashrc` (see [Setting up your shell](shell-setup.md)):

```bash
export EDITOR=nano VISUAL=nano
```

and for `git` alone: `git config --global core.editor nano`.

On Ubuntu, Mint and Debian, `sudo update-alternatives --config editor` picks the system-wide
default.

## VS Code (recommended for projects)

VS Code on Windows can open folders inside Linux and run its terminal, extensions and debugger
there, so it feels like a Linux editor with a Windows window. See
[VS Code with Linux](vscode.md). In short, in a Linux terminal:

```bash
cd ~/my-project
code .
```

## Notepad and other Windows editors

Windows programs can open Linux files: `notepad.exe notes.txt` in a Linux terminal, or through
**Files** on the system's card. Two things to watch:

- **Line endings.** Windows editors may save lines ending in CRLF; Linux expects LF. Shell
  scripts saved that way fail with `$'\r': command not found`. Fix a file with
  `sed -i 's/\r$//' script.sh` (or `dos2unix script.sh`, after installing `dos2unix`).
- **System files** (under `/etc`) belong to root and cannot be saved from Windows. Edit those
  with `sudo nano` instead.

## Editors on a Linux desktop

Each desktop has a simple editor with a window: **Text Editor** on GNOME, **Xed** on Linux
Mint, **Mousepad** on Xfce, **Kate** or **KWrite** on KDE. Start them from the menu, or from the
terminal (`gnome-text-editor notes.txt &`). Install others with the package manager
(`gedit`, `kate`, `geany`).
