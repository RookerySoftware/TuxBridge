---
title: Kali Linux
summary: The security-testing distribution: what it is for, its tool sets, and using it responsibly.
section: Your system
order: 43
---

# Kali Linux

Kali is made for security testing and digital forensics: it packages hundreds of tools for
finding weaknesses in networks, websites and programs. It is based on Debian and updates
continuously (**rolling**). It is meant as a work tool, not an everyday system.

> Only test computers, networks and accounts that are yours, or that you have written
> permission to test. Attacking others' systems is illegal in most countries, even "just to
> see".

## Everyday commands

```bash
sudo apt update && sudo apt full-upgrade   # update everything (Kali uses full-upgrade)
apt search <word>                          # find a tool
sudo apt install <name>                    # install it
```

**Update** in the system's **⋯** menu on My systems does the same.

## Getting the tools

A new Kali in WSL is small. Kali groups its tools in **metapackages**:

| Package | Has | Size |
| --- | --- | --- |
| `kali-linux-headless` | The usual command-line tools | a few GB |
| `kali-linux-default` | What Kali's desktop installer includes | larger |
| `kali-tools-forensics` | Forensics tools | varies |
| `kali-tools-web`, `kali-tools-wireless`, ... | One subject each | varies |

```bash
sudo apt install kali-linux-headless
apt search kali-tools      # every group
```

Some tools need direct access to hardware (Wi-Fi adapters in monitor mode, USB devices), which
WSL does not give by default.

## What people use it for

- **Learning security** on legal practice platforms and labs built for it (Hack The Box,
  TryHackMe, OverTheWire, deliberately vulnerable virtual machines)
- **Capture the flag (CTF) competitions**, where teams solve security puzzles
- **Checking their own setup**: which devices and services are on their home network, whether
  their own website is set up safely
- **Professional security testing** by testers with a written agreement from the owner
- **Forensics**: looking into disk images, files and logs after an incident
- **Studying for certifications** in security

## The desktop: Xfce

Kali's Xfce desktop with its own theme and tool menus, ordered by testing phase. Ctrl+Alt+T
opens a terminal.

## Common problems

| Problem | Fix |
| --- | --- |
| Windows Security deletes downloaded tools | Some security tools look like malware to antivirus programs. Keep them inside Kali; do not copy them to Windows folders. |
| A tool says it needs a Wi-Fi adapter in monitor mode | WSL cannot do this; use a separate PC or virtual machine. |
| `apt upgrade` holds packages back | Use `sudo apt full-upgrade`, as Kali recommends. |

## Learning

- Kali's documentation: https://www.kali.org/docs
- Kali's free training: https://www.kali.org/training
- The **Learn** page has a guided tour of Kali.
