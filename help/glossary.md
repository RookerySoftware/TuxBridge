---
title: Glossary
summary: Linux, WSL and TuxBridge words explained in plain language, A to Z.
section: Reference
order: 82
---

# Glossary

**Alias** — a short name you make for a longer command (`alias ll='ls -la'`).

**apt** — the package manager of Ubuntu, Mint and Debian. See [Installing software](software.md).

**Argument** — what you give a command after its name: in `cp a.txt b.txt`, the two names.

**AUR** — the Arch User Repository: recipes for extra software on Arch-based systems, written by
users. See [Arch Linux](arch.md).

**Bash** — the most common shell, the program that reads your commands.

**Binary** — a ready-to-run program (as opposed to source code). Also: counting in 0s and 1s.

**Boot** — starting a system. WSL systems boot in a second or two.

**Container** — an app packed with everything it needs, run apart from the rest of the system.
See [Docker](docker.md).

**Cron** — the classic way to run commands on a schedule. See [Running things on a schedule](scheduling.md).

**CRLF / LF** — how a text file ends its lines: Windows uses CRLF, Linux LF. Scripts need LF.

**Daemon** — a program running in the background, often a service (`sshd`, `dockerd`).

**Dependency** — something a program needs to work; package managers install them for you.

**Desktop environment** — the windows, menus and taskbar: GNOME, Cinnamon, Xfce, KDE.

**Directory** — another word for folder.

**Distribution (distro)** — one version of Linux: Ubuntu, Mint, Arch… They share the kernel and
most programs.

**Docker** — a tool to run containers. TuxBridge can set up Docker's engine for you.

**Dotfile** — a hidden settings file whose name starts with a dot (`.bashrc`).

**Environment variable** — a named setting programs read, like `PATH` or `EDITOR`.

**ext4** — the file system inside a WSL system's virtual disk.

**Flatpak** — a way to install desktop apps that works on every Linux, mostly from Flathub.

**GPU** — the graphics card. WSL can use it for 3D and AI. See [Graphics, the GPU and Linux apps](gpu.md).

**Group** — a set of users sharing permissions (`sudo`, `docker`, `dialout`).

**GUI** — graphical user interface: windows and buttons, as opposed to the terminal.

**Home folder** — your own folder, `/home/you`, written `~`.

**Host** — the computer running something; for WSL, Windows is the host. Also a computer on a
network.

**Hyprland** — a modern tiling desktop driven by the keyboard. See [The Hyprland desktop](hyprland.md).

**Kernel** — the core of an operating system that talks to the hardware. WSL 2 runs a real
Linux kernel from Microsoft.

**Localhost** — "this computer" on a network (`127.0.0.1`). Servers in WSL are reached from
Windows at `localhost`.

**LTS** — Long Term Support: a release with years of security updates (Ubuntu LTS).

**Man page** — a command's manual: `man ls`.

**Mirror** — a server with a copy of a distribution's packages; downloads come from a nearby one.

**Mount** — making a disk or folder appear somewhere in the file tree (Windows' C: is mounted at
`/mnt/c`).

**Package** — a program packed for installing, with information about what it needs.

**Package manager** — the program that installs, updates and removes packages (`apt`, `pacman`).

**pacman** — the package manager of Arch, CachyOS and Omarchy.

**Partition** — a part of a disk with its own file system.

**Path** — where a file is: `/home/you/notes.txt`. Absolute paths start with `/`; relative ones
start from where you are.

**PATH** — the variable listing the folders searched for commands.

**Permission** — who may read, write or run a file. See [Permissions](permissions.md).

**PID** — process ID: the number of a running program.

**Pipe** — `|`, sending one command's output into another. See [Pipes](pipes.md).

**Port** — a numbered door a network program listens on (web servers: 80, 443, 8080…).

**PPA** — a Personal Package Archive: extra software sources for Ubuntu.

**Process** — a running program.

**Prompt** — the text before your cursor in the terminal (`you@ubuntu:~$`): Linux is waiting.

**Repository** — a collection of packages a package manager installs from. Also: a Git project.

**Rolling release** — a distribution that updates continuously instead of in big versions (Arch).

**Root** — Linux's administrator account. Also `/`, the top of the file tree.

**Service** — a background program started and managed by systemd.

**Shell** — the program that reads your commands (bash, zsh, fish).

**Snap** — Ubuntu's own app format; often doesn't work in WSL.

**SSH** — a secure way to sign in to and copy files between computers. See [Git and SSH keys](git-ssh.md).

**Standard output / error** — the two streams a command prints to (stdout and stderr).

**sudo** — "superuser do": runs one command as administrator after asking your password.

**Symbolic link (symlink)** — a shortcut to another file or folder (`ln -s`).

**systemd** — the program that starts and manages services on most Linux systems.

**Tab completion** — pressing Tab to finish a name for you.

**Terminal** — the window where you type commands.

**Tiling** — windows arranged side by side automatically instead of overlapping.

**Upstream** — the original project something is built from (Debian is Ubuntu's upstream).

**Virtual disk (VHDX)** — the single file on Windows holding a WSL system's whole disk.

**Virtual machine** — a computer simulated by software. WSL 2 runs Linux in a small, fast one.

**Wayland / X11** — the two ways Linux programs draw windows; X11 is older, Wayland newer.

**Wildcard** — `*` and `?` in names: `*.txt` means every name ending in .txt.

**WSL** — Windows Subsystem for Linux: the part of Windows that runs Linux systems.

**WSLg** — WSL's support for Linux apps with windows and sound.

**Workspace** — one of several virtual desktops.
