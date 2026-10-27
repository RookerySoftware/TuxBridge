---
title: Kali Linux
summary: The security-testing distribution, its tool sets, and using it responsibly.
section: Your system
order: 43
---

# Kali Linux

Kali is made for security testing and digital forensics: it packages hundreds of tools for
finding weaknesses in networks, websites and programs. It is based on Debian and updates
continuously (**rolling**).

> Only test computers, networks and accounts that are yours, or that you have written
> permission to test. Attacking others' systems is illegal in most countries, even "just to
> see".

## Everyday commands

```
sudo apt update && sudo apt full-upgrade   # update everything (Kali uses full-upgrade)
sudo apt install <name>                    # install a tool
```

## Getting the tools

A new Kali in WSL is small. Kali groups its tools in **metapackages**:

| Package | Has | Size |
| --- | --- | --- |
| `kali-linux-headless` | The usual command-line tools | a few GB |
| `kali-linux-default` | What Kali's desktop installer includes | larger |
| `kali-tools-web`, `kali-tools-wireless`, ... | One subject each | varies |

```
sudo apt install kali-linux-headless
```

Some tools need direct access to hardware (Wi-Fi adapters in monitor mode, USB devices), which
WSL does not give by default.

## The desktop: Xfce

Kali's Xfce desktop with its own theme and tool menus, ordered by testing phase. Ctrl+Alt+T
opens a terminal.

## Learning

Kali's documentation (kali.org/docs) and the free training at kali.org/training are good
starting points; practise on deliberately vulnerable labs (Hack The Box, TryHackMe, or VMs
made for it) rather than real systems.
