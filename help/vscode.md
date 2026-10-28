---
title: VS Code with Linux
summary: Edit and run Linux projects from VS Code on Windows, with the WSL extension.
section: Programming
order: 31
---

# VS Code with Linux

**Visual Studio Code** on Windows can work *inside* a Linux system: it opens Linux folders, runs
its terminal, extensions, debugger and language tools in Linux, and shows it all in a normal
Windows window. It is the most comfortable way to program in WSL.

## Set it up

1. Install VS Code **on Windows** from code.visualstudio.com (not inside Linux).
2. In VS Code, install the **WSL** extension (by Microsoft) from the Extensions view
   (Ctrl+Shift+X).
3. Open a terminal in your Linux system and run:

```bash
cd ~/projects/my-app
code .
```

The first time, VS Code installs a small server inside Linux (a minute). After that it opens in
seconds. The bottom-left corner shows **WSL: <system name>** when it's connected.

`code .` works because WSL adds Windows' programs to Linux's `PATH`. If `code` is not found,
open VS Code, press **Ctrl+Shift+P**, and run **WSL: Connect to WSL** (or **Connect to WSL using
Distro…** to pick a system), then **File → Open Folder**.

## What runs where

| Part | Runs in |
| --- | --- |
| The window, themes, keyboard | Windows |
| Files, terminal, Git, compilers | Linux |
| Most extensions (Python, ESLint, rust-analyzer…) | Linux; install them again "in WSL" when asked |
| UI-only extensions (themes, keymaps) | Windows |

The Extensions view shows two lists: **Local** and **WSL: <system>**. A button installs an
extension into WSL.

## The terminal

**Ctrl+`** opens a terminal in the project folder, inside Linux. Several terminals,
split terminals and tasks all run in Linux.

## Opening the browser

When your app serves a page (for example `npm run dev` on port 5173), VS Code notices and offers
to open it; WSL forwards `localhost` to Windows, so `http://localhost:5173` works in your Windows
browser.

## Dev Containers

The **Dev Containers** extension develops inside a Docker container, defined by a
`.devcontainer` folder in the project. With TuxBridge's Docker turned on, it can use TuxBridge's
engine: the `docker` command in your Linux system already talks to it. Open the project in WSL
first, then run **Dev Containers: Reopen in Container**.

## Tips

- Keep projects in Linux (`~/projects`): file watching and Git are much faster there than under
  `/mnt/c`.
- To open a Linux folder from Windows instead, use `\\wsl.localhost\<system>\home\<you>\project`
  with **WSL: Open Folder in WSL** rather than plain *Open Folder*, so tools run in Linux.
- VS Code shows CRLF or LF in its status bar: Linux files should be **LF**.
- Settings can be per place: *Settings → Remote [WSL]* holds settings just for WSL.

## Other editors

- **JetBrains IDEs** (IntelliJ, PyCharm, WebStorm…) support WSL through *Remote Development* or
  by using a WSL interpreter/SDK in a project on Windows.
- **Cursor** and other VS Code-based editors usually have a WSL connection too.
- **Neovim** or **Helix** run directly inside Linux in the terminal.
