---
title: Parrot Security
summary: Parrot OS: what it is for, its KDE desktop, its tool groups and keeping it updated.
section: Your system
order: 47
---

# Parrot Security

Parrot (Parrot OS) is a Debian-based Linux made for security work, privacy and programming. It
is similar to Kali, with a calmer, everyday feel. EasyWSL installs **Parrot Core**, a small
base, and you add the tool groups you want. Like Kali, it updates continuously.

> Only test computers, networks and accounts that are yours, or that you have written
> permission to test.

## Everyday commands

```bash
sudo apt update && sudo apt full-upgrade   # update everything
apt search <word>                          # find a program
sudo apt install <name>                    # install it
```

Parrot's own advice is `full-upgrade`, not `upgrade`: its updates sometimes add or remove
packages. **Update** in the system's **⋯** menu on My systems does the same.

## Tool groups

Parrot packs its tools in groups (metapackages), so you install only what you need:

| Package | Has |
| --- | --- |
| `parrot-tools-full` | Everything below (large, many GB) |
| `parrot-tools-forensics` | Investigating disks and files |
| `parrot-tools-reversing` | Taking programs apart to understand them |
| `parrot-tools-reporting` | Writing up findings |
| `parrot-meta-privacy` | Privacy tools |
| `parrot-meta-crypto` | Encryption tools |
| `parrot-meta-devel` | Programming tools and editors |

```bash
apt search parrot-tools     # every group, with a description
sudo apt install parrot-meta-devel
```

## The desktop: KDE Plasma

Parrot 7 uses KDE Plasma with Parrot's own theme. The panel is at the top:

- the **Parrot logo** (top left) opens the menu, with the installed tools sorted by group
- next to it: **System Settings**, the **file manager** (Dolphin), **Firefox** and the
  **terminal** (Konsole)

Plasma needs more memory than Xfce; give it a moment to start. Things that need a real PC
(Wi-Fi, Bluetooth, power, screens) are handled by Windows and do nothing here.

## Parrot or Kali?

Both are Debian-based and have mostly the same tools.

- **Kali** is the industry standard, and most course material and write-ups use it.
- **Parrot** adds privacy and development tools, and is pleasant as a daily system.

Many people keep both. With EasyWSL each is a separate system, so neither gets in the way of
the other.

## What people use it for

- **Learning security** on practice platforms and in labs built for it
- **Checking their own setup**: their home network, their own website, their own passwords
- **Forensics**: looking into files and disk images
- **Privacy**: encryption and privacy tools
- **Programming**: Parrot comes with good development tool groups

WSL has limits for some security work: it cannot put a Wi-Fi adapter in monitor mode, and USB
devices need extra steps. Use a separate PC or a virtual machine for that.

## Learn more

- Parrot's documentation: https://parrotsec.org/docs
- The **Learn** page has a guided tour of Parrot.
