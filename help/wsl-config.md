---
title: Configuring WSL
summary: .wslconfig for memory, processors and networking; wsl.conf for systemd, drives, PATH and the default user.
section: WSL and Windows
order: 51
---

# Configuring WSL

WSL has two settings files:

| File | Where | Applies to |
| --- | --- | --- |
| **.wslconfig** | Windows: `C:\Users\you\.wslconfig` | All systems: the virtual machine (memory, processors, network) |
| **wsl.conf** | Inside each system: `/etc/wsl.conf` | That system only (systemd, drives, user, PATH) |

Both are plain text with `[sections]` and `key=value` lines. **Changes apply after**
`wsl --shutdown` (or **Restart WSL** in EasyWSL) and opening the system again.

Microsoft's **WSL Settings** app (search for it in the Start menu) edits `.wslconfig` with a
window, if you prefer.

## .wslconfig (Windows side)

Open it with `notepad $HOME\.wslconfig` in PowerShell (it may not exist yet; Notepad offers to
create it).

```ini
[wsl2]
# the most memory Linux may use (default: half of the PC's)
memory=8GB
# how many processor cores (default: all)
processors=4
# swap space (0 turns it off)
swap=4GB
# Linux shares Windows' network addresses (see Networking)
networkingMode=mirrored
# DNS through Windows; helps with VPNs
dnsTunneling=true
# use Windows' proxy settings
autoProxy=true
# Linux apps with windows (WSLg)
guiApplications=true

[experimental]
# give unused memory back to Windows over time
autoMemoryReclaim=gradual
# new virtual disks shrink when files are deleted
sparseVhd=true
```

Only add what you want to change. Some keys move between `[wsl2]` and `[experimental]` across
WSL versions; Microsoft's WSL documentation (search "wslconfig") lists the current ones.

**A typical tweak:** if Linux and Windows together feel short of memory, set `memory=` to about
half or a little less of your PC's memory, and turn on `autoMemoryReclaim`.

## /etc/wsl.conf (inside a system)

```bash
sudo nano /etc/wsl.conf
```

```ini
[boot]
# run systemd (EasyWSL sets this for its systems)
systemd=true

[user]
# who you sign in as
default=you

[automount]
# Windows drives under /mnt
enabled=true
# where they go
root=/mnt/
# Linux permissions on Windows files
options="metadata,umask=22,fmask=11"

[interop]
# running Windows programs from Linux
enabled=true
# Windows' PATH added to Linux's (false = faster, cleaner)
appendWindowsPath=true

[network]
# the system's name on the network
hostname=my-linux
generateHosts=true
# false if you want to write /etc/resolv.conf yourself
generateResolvConf=true
```

Then `wsl --terminate <system>` (or **Stop** in EasyWSL) and open it again.

### Common changes

- **Stop Windows' programs cluttering PATH** (faster tab completion, no Windows `node` or
  `python` by mistake): `appendWindowsPath=false`. You can still run `explorer.exe` by its
  full path, `/mnt/c/Windows/explorer.exe`.
- **Make `chmod` work on Windows files:** `options="metadata"` under `[automount]`.
- **Fix DNS yourself** (some VPNs): `generateResolvConf=false`, then write
  `nameserver 1.1.1.1` into `/etc/resolv.conf`.
- **Change who signs in:** `[user] default=name`.

## Systemd

systemd starts services (databases, Docker, SSH) and timers. EasyWSL turns it on for the systems
it installs. Check with:

```bash
ps -p 1 -o comm=      # prints "systemd" when it's on
```

## Changing the kernel

`.wslconfig` can point to your own kernel (`kernel=C:\\path\\to\\bzImage`) or add kernel
modules. Most people never need this; `wsl --update` keeps Microsoft's kernel current.

## Undoing a change

Remove the line (or the whole file), then `wsl --shutdown`. If a system won't start after
editing `wsl.conf`, fix it from Windows: `wsl -d <system> -u root -e nano /etc/wsl.conf`.
