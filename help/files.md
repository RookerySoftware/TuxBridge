---
title: Files between Windows and Linux
summary: Where your files are, how to move them, and which side to keep projects on.
section: Using Linux
order: 11
---

# Files between Windows and Linux

Each Linux system has its own disk, separate from Windows. Both sides can still reach each
other's files.

## Linux files, from Windows

- On the system's card in **My systems**, choose **Open files**: File Explorer opens your Linux
  home folder.
- In File Explorer, **Linux** in the sidebar lists every system. The address is
  `\\wsl.localhost\<system name>\home\<you>`.
- From the Linux terminal, `explorer.exe .` opens the folder you are in.

You can copy, open and edit these files with Windows programs as usual.

## Windows files, from Linux

Windows' drives appear under `/mnt`:

| Windows | Linux |
| --- | --- |
| `C:\` | `/mnt/c` |
| `C:\Users\you\Downloads` | `/mnt/c/Users/you/Downloads` |
| `D:\Photos` | `/mnt/d/Photos` |

```
cp /mnt/c/Users/you/Downloads/report.pdf ~/
```

copies a download into your Linux home folder. Names with spaces need quotes:
`cd "/mnt/c/Users/you/My Projects"`.

## Which side should my files be on?

- **Projects you work on in Linux** (code, builds): keep them in Linux, under `~`. Linux reads
  its own disk many times faster than Windows' drives, which matters for builds and for tools
  like `git` and `npm`.
- **Documents, photos, downloads**: keep them on Windows. They are backed up with the rest of
  your Windows files, and Linux reaches them under `/mnt/c` when needed.

## Good to know

- **Removing a system deletes its files.** Copy anything you need to Windows first.
- **Line endings.** Windows text files end lines differently (CRLF) from Linux (LF). Scripts
  written in Notepad may fail in Linux with `bad interpreter` or `$'\r': command not found`.
  Editors like VS Code show and change this in their status bar.
- **Permissions.** Files under `/mnt/c` follow Windows' permissions, not Linux's; making a
  script there executable with `chmod` may not stick. Keep scripts in Linux.
