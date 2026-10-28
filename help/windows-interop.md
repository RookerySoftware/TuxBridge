---
title: Windows and Linux together
summary: Run Windows programs from Linux and Linux commands from Windows, convert paths, use the clipboard.
section: WSL and Windows
order: 52
---

# Windows and Linux together

WSL is not a separate computer: Windows and Linux can run each other's programs and share files.

## Windows programs from Linux

Any Windows program runs from a Linux terminal by its `.exe` name:

```bash
explorer.exe .                  # this Linux folder in File Explorer
notepad.exe todo.txt            # edit a Linux file in Notepad
code .                          # VS Code (see VS Code with Linux)
cmd.exe /c ver                  # a cmd command
powershell.exe -c "Get-Date"    # a PowerShell command
ipconfig.exe                    # Windows' network settings
wslview https://example.com     # open a link in the Windows browser (from the wslu package)
```

Windows programs see Linux files through `\\wsl.localhost\<system>\...`; WSL converts the path
for the current folder automatically.

## Linux commands from Windows

In PowerShell or cmd:

```powershell
wsl ls -la                              # in the default system
wsl -d Ubuntu -e grep -i error log.txt  # a file in the current Windows folder
Get-Content big.log | wsl grep error    # pipe Windows output through Linux tools
```

The current Windows folder becomes the Linux folder (`/mnt/c/...`), so Linux tools work on
Windows files directly.

## Converting paths: wslpath

```bash
wslpath 'C:\Users\you\Documents'        # -> /mnt/c/Users/you/Documents
wslpath -w ~/projects                   # -> \\wsl.localhost\Ubuntu\home\you\projects
wslpath -m ~/projects                   # the same with forward slashes
```

## The clipboard

```bash
cat ~/.ssh/id_ed25519.pub | clip.exe     # copy into the Windows clipboard
powershell.exe -c Get-Clipboard          # paste from it
```

In Windows Terminal, **Ctrl+Shift+C** / **Ctrl+Shift+V** (or right-click) copy and paste;
selecting text with the mouse also copies it by default.

## Your Windows user name from Linux

```bash
cmd.exe /c 'echo %USERNAME%' 2> /dev/null | tr -d '\r'
```

Handy in scripts that save into `/mnt/c/Users/<name>/...`.

## Sharing environment variables: WSLENV

`WSLENV` lists variables to pass between the two sides. In PowerShell:

```powershell
$env:PROJECT = "C:\work\app"
$env:WSLENV = "PROJECT/p"       # /p converts the path for Linux
wsl echo '$PROJECT'             # prints /mnt/c/work/app
```

Flags: `/p` converts a path, `/l` a list of paths, `/u` only Windows → Linux, `/w` only
Linux → Windows.

## Opening Linux in Windows Terminal

Windows Terminal adds a profile for each system. Set your favourite as the default
profile in its settings, or start a tab with `wt -p "Ubuntu"`. Pengi's **Terminal** button
uses Windows Terminal when it's installed.

## Shortcuts in File Explorer

- Every system is under **Linux** in File Explorer's sidebar. Right-click your home folder →
  **Pin to Quick access**.
- In File Explorer's address bar, type `wsl` and press Enter to open a Linux terminal in that
  folder.

## Things that don't cross over

- Linux programs can't use Windows drivers directly (except the GPU, see
  [Graphics, the GPU and Linux apps](gpu.md), and USB, see [USB devices](usb.md)).
- Windows' antivirus doesn't scan inside Linux disks the same way; install from trusted sources.
- Running a Windows `.exe` from Linux runs it *as Windows*, with your Windows rights, not
  inside Linux.
