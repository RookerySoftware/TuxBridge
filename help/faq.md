---
title: Frequently asked questions
summary: Short answers to the questions newcomers ask most, with links to the full guides.
section: Reference
order: 83
---

# Frequently asked questions

## About TuxBridge and WSL

**Is this a virtual machine? Will it slow down my PC?**
WSL 2 runs Linux in a small virtual machine that Windows manages. It uses memory only while a
system runs, gives it back when idle, and stops by itself when nothing uses it.

**Can I break Windows with it?**
No. Linux systems are kept apart from Windows. The worst case is a broken Linux system, which you
remove and install again in minutes. Linux *can* change files on your Windows drives under
`/mnt/c`, like any program you run, so be careful with `rm` there.

**Do I need to keep TuxBridge open?**
No. Systems, terminals and Docker keep working without it (`wsl` in PowerShell, or Windows
Terminal's tabs); open TuxBridge when you want its buttons, for example to open a desktop.

**Where are my systems stored?**
Each in its own folder with one virtual disk file, in the folder chosen when installing
(**Settings → Folder for new systems**). See [Disk space and moving systems](disk-space.md).

**Can I use systems installed without TuxBridge?**
Yes. My systems shows every system on the PC, including ones from the Microsoft Store or
`wsl --install`.

**Does uninstalling TuxBridge remove my systems?**
No. They stay in WSL; remove them first in My systems if you want them gone.

## Choosing

**Which Linux should I pick?**
Ubuntu for a first Linux, Linux Mint if you want it to feel like Windows, Arch for the newest
software and Hyprland, Debian for something stable and light. See **Get Linux**.

**Can I have several?**
Yes, side by side, even several of the same kind with different names.

**Do I need a desktop?**
No. Most people use the terminal, and Linux apps with windows open on your Windows desktop even
without one. A desktop is nice for exploring Linux as a whole.

## Using it

**Where are my files?**
In your Linux home folder, `/home/<you>`. **Files** on the system's card opens it in File
Explorer. Windows' drives are at `/mnt/c`, `/mnt/d` in Linux. See [Files between Windows and Linux](files.md).

**What's my password? It doesn't show when I type it.**
The one you chose when installing. Nothing appears while typing it: that's normal, type it and
press Enter. Forgot it? See [Your account, sudo and passwords](accounts.md).

**How do I copy and paste in the terminal?**
Select text to copy (or Ctrl+Shift+C); right-click or Ctrl+Shift+V to paste. Ctrl+C stops a
command instead.

**How do I install programs?**
With the package manager: `sudo apt install <name>` or `sudo pacman -S <name>`. See
[Installing software](software.md).

**How do I update?**
**Update** in the system's **⋯** menu. See [Your first ten minutes](first-steps.md).

**Can I run Windows programs from Linux?**
Yes: `explorer.exe .`, `notepad.exe file`, `code .`. See [Windows and Linux together](windows-interop.md).

**Can Linux use my graphics card?**
Yes, for 3D, video and AI (CUDA). Install drivers on Windows only. See [Graphics, the GPU and Linux apps](gpu.md).

**How do I open a website I'm running in Linux?**
At `http://localhost:<port>` in your Windows browser. See [Networking](networking.md).

**Can I play games?**
Some run, but Windows runs games better on the same PC. Linux in WSL shines for programming,
servers, learning and tools.

## When something's wrong

**It says WSL is not responding.**
Use **Restart WSL** on My systems (or `wsl --shutdown` in PowerShell), or restart Windows. See
[When something goes wrong](troubleshooting.md).

**My system takes a lot of disk space.**
See [Disk space and moving systems](disk-space.md).

**A command says "permission denied".**
Either it needs `sudo`, or the file isn't yours / isn't executable. See [Permissions](permissions.md).

**A command says "command not found".**
It isn't installed (install it with the package manager), it's misspelled (Linux cares about
capitals), or it's a script in the current folder (run it as `./name`).

**I see an error I don't understand.**
Search it in these guides (paste a few words of it), check [Common error messages](errors.md),
or **Ask Pip**.
