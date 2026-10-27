---
title: The terminal, the commands you will use
summary: Moving around, files and folders, editing, stopping a command, with Windows equivalents.
section: Using Linux
order: 10
---

# The terminal: the commands you will use

The terminal looks old-fashioned, but most Linux guides use it because a command can be
copied exactly, while "click the third button" cannot. You only need a dozen commands to start.

## Where am I, and what is here?

| Command | Does | On Windows |
| --- | --- | --- |
| `pwd` | Shows the folder you are in | `cd` |
| `ls` | Lists what is in it (`ls -la` with hidden files and details) | `dir` |
| `cd Documents` | Goes into a folder | `cd Documents` |
| `cd ..` | Goes up one folder | `cd ..` |
| `cd` or `cd ~` | Goes home (`/home/you`) | `cd %USERPROFILE%` |

Paths use `/`, not `\`. The top of everything is `/`, and `~` is short for your home folder.

## Files and folders

| Command | Does |
| --- | --- |
| `mkdir projects` | Makes a folder |
| `cp notes.txt backup.txt` | Copies a file (`cp -r` for a folder) |
| `mv notes.txt Documents/` | Moves (or renames) a file |
| `rm notes.txt` | Deletes a file, **for good**: there is no Recycle Bin |
| `rm -r old-folder` | Deletes a folder and everything in it, for good |
| `cat notes.txt` | Shows a file's text |
| `nano notes.txt` | Edits a file in the terminal (Ctrl+O saves, Ctrl+X leaves) |

## Things that save time

- **Tab** completes names: type `cd Doc` and press Tab.
- **Up arrow** brings back earlier commands.
- **Ctrl+C** stops the command that is running. (To copy, select text and use the terminal's
  own menu, or Ctrl+Shift+C in most Linux terminals.)
- **Ctrl+L** or `clear` clears the screen.
- `man ls` shows the manual of a command (`q` leaves it); `ls --help` is shorter.

## Being administrator: sudo

Changing the system (installing software, editing settings outside your home) needs `sudo` in
front of the command, and your password. It is Linux's version of the "Do you want to allow
this app to make changes?" prompt. Use it only when a command needs it.

## Running Windows programs from Linux

Windows programs work from the Linux terminal by their file name:

```
explorer.exe .        # opens the current Linux folder in File Explorer
notepad.exe notes.txt # edits a file in Notepad
code .                # opens the folder in VS Code, if it is installed on Windows
```
