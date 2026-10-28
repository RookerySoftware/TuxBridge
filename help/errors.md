---
title: Common error messages
summary: What the messages you'll meet most actually mean, and what to do about each.
section: When something goes wrong
order: 91
---

# Common error messages

Paste a few words of any message into the search above to find it. The most common ones:

## In the terminal

| Message | Means | Do |
| --- | --- | --- |
| `command not found` | No program by that name | Check spelling and capitals; install it (`apt search` / `pacman -Ss`); for a script here, run `./name`. Ubuntu often suggests the package. |
| `Permission denied` | You may not read, write or run it | Needs `sudo`? Not executable? (`chmod +x file`). Not yours? (`ls -l`). See [Permissions](permissions.md). |
| `No such file or directory` | The path doesn't exist | Check with `ls`, Tab completion and capitals; quote names with spaces. For a script: maybe CRLF line endings (below). |
| `Is a directory` / `Not a directory` | A folder where a file was expected, or the reverse | Check the path; `cp -r` for folders. |
| `Directory not empty` | `rmdir` only removes empty folders | `rm -r folder` (for good!) |
| `Operation not permitted` | Even sudo can't, or it's a Windows file | Under `/mnt/c`, Windows decides; close Windows programs using the file. |
| `Text file busy` | The file is running | Stop the program first. |
| `Argument list too long` | A `*` matched too many files | Use `find … -exec` or `xargs`. |
| `$'\r': command not found` / `bad interpreter: /bin/bash^M` | Windows line endings in a script | `sed -i 's/\r$//' script.sh` |
| `Segmentation fault` | The program crashed | Update it; report it to its makers. |
| `Killed` | Linux ran out of memory and stopped it | Close other things; raise `memory=` in `.wslconfig` (see [Configuring WSL](wsl-config.md)). |
| `No space left on device` | The disk (or Windows' drive) is full | See [Disk space](disk-space.md). |
| `Read-only file system` | The disk was put in safe mode after an error | Restart WSL. If it happens again, copy your files out and reinstall the system. |
| `sudo: unable to resolve host` | The system's name isn't in `/etc/hosts` | Harmless; restart the system. |
| `<you> is not in the sudoers file` | Your account can't use sudo | From PowerShell: `wsl -d <system> -u root usermod -aG sudo <you>` (Arch: `wheel`). |
| `Sorry, try again` (sudo) | Wrong password | It's your Linux password; nothing shows while typing. Forgot it? See [Your account](accounts.md). |

## Installing software

| Message | Do |
| --- | --- |
| `Could not get lock /var/lib/dpkg/lock-frontend` | Another apt is running (maybe automatic updates, or **Update**). Wait. |
| `E: Unable to locate package` | `sudo apt update` first; check the name with `apt search`. |
| `Unmet dependencies` / `held broken packages` | `sudo apt install -f`; remove a PPA that caused it. |
| `dpkg was interrupted` | `sudo dpkg --configure -a` |
| `NO_PUBKEY` / `is not signed` | A third-party source's key is missing: redo its setup from its current instructions. |
| `externally-managed-environment` (pip) | Use a virtual environment, uv or pipx: see [Python](python.md). |
| `unable to lock database` (pacman) | Another pacman runs; if not: `sudo rm /var/lib/pacman/db.lck`. |
| `invalid or corrupted package (PGP signature)` | `sudo pacman -Sy archlinux-keyring && sudo pacman -Su` |
| `target not found` (pacman) | Update first; it may be in the AUR. |
| `conflicting files` (pacman) | Read archlinux.org/news; see [Arch Linux](arch.md). |
| `EACCES` (npm) | Don't use `sudo npm`; see [Node.js](node.md). |

## Network

| Message | Do |
| --- | --- |
| `Temporary failure in name resolution` | DNS: restart WSL; with a VPN, see [Networking](networking.md). |
| `Connection refused` | Nothing listens on that port: is the server running? Right port? |
| `Connection timed out` | A firewall or VPN blocks it, or the host is down. |
| `Address already in use` | Another program has the port: `sudo ss -tlnp` shows which. |
| `Could not resolve host: github.com` | DNS, as above. |
| `SSL certificate problem` | The clock is off (`sudo hwclock -s`) or a corporate proxy intercepts HTTPS. |
| `Permission denied (publickey)` | SSH key not added or not used; see [Git and SSH keys](git-ssh.md). |

## WSL itself (in PowerShell, or from EasyWSL)

| Message | Do |
| --- | --- |
| `HCS_E_CONNECTION_TIMEOUT`, or nothing happens | WSL is stuck: **Restart WSL**, or `wsl --shutdown`, or restart Windows. |
| `0x80370102` / *virtualization is not enabled* | Turn on virtualization (Intel VT-x / AMD SVM) in the PC's firmware settings. EasyWSL's check explains. |
| `0x8007019e` / *WSL is not installed* | Let EasyWSL's setup turn WSL on, or `wsl --install` and restart. |
| `0x80070422` | A Windows service WSL needs is disabled; restart, and make sure *Virtual Machine Platform* is on. |
| `There is no distribution with the supplied name` | Check the name with `wsl -l -v`. |
| `The attempted operation is not supported for the type of object referenced` | Often a network tool (VPN, proxy) hooking Windows' network; update or remove it. |
| `Catastrophic failure` / `E_UNEXPECTED` | `wsl --shutdown`, then `wsl --update`; restart Windows. |
| `cannot open display` (apps with windows) | `wsl --update`; restart WSL. |

## Git

| Message | Do |
| --- | --- |
| `Author identity unknown` | `git config --global user.name …` and `user.email …` |
| `detected dubious ownership` | The project is under `/mnt/c`: move it into Linux, or follow Git's `safe.directory` hint. |
| `failed to push some refs` | Someone pushed first: `git pull`, then push. |
| `CONFLICT (content)` | Edit the marked files, `git add` them, then `git commit`. |

Not listed? Search the guides with a few words of the message, or ask the **Assistant** with the
whole message pasted in.
