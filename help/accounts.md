---
title: Your account, sudo and passwords
summary: Your Linux account, administrator rights, and what to do if you forget the password.
section: Using Linux
order: 13
---

# Your account, sudo and passwords

When you installed the system, EasyWSL made a Linux account with the name and password you
chose. It is separate from your Windows account: the password is only for this system.

## sudo

Your account is a normal user that can become administrator for one command with `sudo`:

```
sudo apt install htop
```

Linux asks for **your** password (not a separate administrator password) and remembers it for
a few minutes. Nothing shows while you type it; press Enter at the end.

The real administrator account is called `root`. You do not need to sign in as root; `sudo`
is safer because you only use those rights when you mean to.

## Forgot the password?

WSL can open any system as root without a password, from Windows. In PowerShell or Windows
Terminal:

```
wsl -d <system name> -u root passwd <your linux name>
```

for example `wsl -d ubuntu -u root passwd me`, then type the new password twice.

## Changing your password

```bash
passwd               # asks for the old one, then the new one twice
```

## Several accounts

```bash
sudo adduser anna                                    # Ubuntu, Mint, Debian (asks the rest)
sudo useradd -m -G wheel anna && sudo passwd anna    # Arch, CachyOS, Omarchy
su - anna                                            # become anna in this terminal (exit returns)
sudo deluser --remove-home anna                      # remove (Arch: sudo userdel -r anna)
```

The terminal from EasyWSL opens with the account you made when installing. To sign in as
someone else from PowerShell: `wsl -d <system> -u anna`. The default account is set in
`/etc/wsl.conf` (see [Configuring WSL](wsl-config.md)).

## Groups

Groups give several users the same rights. A few that matter:

| Group | Lets its members |
| --- | --- |
| `sudo` (Ubuntu, Mint, Debian) / `wheel` (Arch) | Use sudo |
| `docker` | Use Docker without sudo |
| `dialout` (Arch: `uucp`) | Use serial ports (microcontrollers) |
| `video`, `render` | Use the GPU directly (rarely needed in WSL) |

```bash
groups                          # your groups
sudo usermod -aG docker $USER   # add yourself to one (-a = add, don't replace!)
```

A new group takes effect in a **new** session: close the terminals and **Stop** the system, or
run `newgrp docker` for one terminal.

## Who can use sudo

The rules live in `/etc/sudoers` and `/etc/sudoers.d/`. Never edit them with a normal editor; a
mistake can lock you out. Use `sudo visudo`, which checks before saving. If you ever do lock
yourself out, WSL's root access from PowerShell (above) still works.

## Root's own password

`root` usually has no password (you can't sign in as root directly, only through sudo). That's
the safe default; leave it that way.
