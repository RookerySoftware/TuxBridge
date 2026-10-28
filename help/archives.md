---
title: Zip, tar and other archives
summary: Pack and unpack .zip, .tar.gz, .tar.xz, .7z and .rar files from the terminal.
section: Using Linux
order: 21
---

# Zip, tar and other archives

Linux software and backups often come as `.tar.gz` (a "tarball") rather than `.zip`. Both are
easy from the terminal.

## Unpacking

| File | Command |
| --- | --- |
| `.tar.gz`, `.tgz` | `tar -xzf file.tar.gz` |
| `.tar.xz` | `tar -xJf file.tar.xz` |
| `.tar.bz2` | `tar -xjf file.tar.bz2` |
| `.tar.zst` | `tar --zstd -xf file.tar.zst` |
| `.tar` | `tar -xf file.tar` |
| `.zip` | `unzip file.zip` |
| `.gz` (one file) | `gunzip file.gz` |
| `.7z` | `7z x file.7z` |
| `.rar` | `unrar x file.rar` (or `7z x file.rar`) |

Modern `tar` recognises the compression itself, so `tar -xf anything.tar.*` usually works.

```bash
tar -xf app.tar.gz -C ~/apps     # unpack into another folder
tar -tf app.tar.gz               # list what's inside without unpacking
unzip -l photos.zip              # the same for zip
unzip photos.zip -d photos       # unpack into a new folder
```

**Remembering tar's letters:** `x` e*x*tract, `c` *c*reate, `t` lis*t*, `f` *f*ile (always last,
followed by the name), `v` *v*erbose (show each file), `z` gzip, `J` xz, `j` bzip2.

## Packing

```bash
tar -czf backup.tar.gz ~/projects          # a folder into a .tar.gz
tar -cJf backup.tar.xz ~/projects          # smaller, slower (xz)
tar -czf site.tar.gz --exclude=node_modules site/
zip -r photos.zip Photos/                  # a .zip, which Windows opens directly
7z a archive.7z folder/                    # 7-Zip
```

To send something to a Windows user, `.zip` is the safest choice: Windows opens it without
extra software. Windows 11 also opens `.tar.gz`, `.7z` and `.rar` in File Explorer.

## Installing the tools

`tar` and `gzip` are always there. The others:

```bash
sudo apt install zip unzip p7zip-full unrar     # Ubuntu, Mint, Debian (unrar needs non-free on Debian)
sudo pacman -S zip unzip 7zip unrar             # Arch, CachyOS, Omarchy
```

## Good to know

- `tar` keeps Linux permissions and owners; `zip` does not always. For backups of Linux
  folders, prefer `tar`.
- Unpack into an empty folder when you're not sure how the archive is laid out: some put
  everything in one folder, others scatter files into the current one.
- Archives from the internet are not checked by anyone. Unpacking is safe; *running* what came
  out is a choice, as on Windows.
- To check a download, compare its checksum with the one on the website:
  `sha256sum file.tar.gz`.
