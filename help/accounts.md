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

## Several accounts

```
sudo adduser anna          # Ubuntu, Mint, Debian, Kali
sudo useradd -m -G wheel anna && sudo passwd anna   # Arch, CachyOS
```

The terminal from EasyWSL opens with the account you made when installing.
