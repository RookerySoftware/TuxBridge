---
title: Command reference
summary: The commands you'll meet most, grouped by task, each with what it does and an example.
section: Reference
order: 80
---

# Command reference

A quick lookup. For any command, `man <command>` has the full manual and `<command> --help` a
summary. `tldr <command>` (install `tldr`) shows short examples.

## Moving around

| Command | Does | Example |
| --- | --- | --- |
| `pwd` | Shows where you are | `pwd` |
| `ls` | Lists a folder | `ls -lah` (details, hidden, sizes) |
| `cd` | Changes folder | `cd ~/projects`, `cd ..`, `cd -` (back) |
| `tree` | Shows folders as a tree | `tree -L 2` |
| `pushd` / `popd` | Go somewhere and come back | `pushd /etc; popd` |

## Files and folders

| Command | Does | Example |
| --- | --- | --- |
| `mkdir` | Makes folders | `mkdir -p a/b/c` |
| `touch` | Makes an empty file (or updates its time) | `touch notes.txt` |
| `cp` | Copies | `cp -r src/ backup/` |
| `mv` | Moves or renames | `mv old.txt new.txt` |
| `rm` | Deletes, for good | `rm file`, `rm -r folder`, `rm -i *.log` (asks) |
| `rmdir` | Deletes an empty folder | `rmdir empty` |
| `ln -s` | Makes a shortcut (symbolic link) | `ln -s /mnt/c/Users/you/Downloads ~/dl` |
| `file` | Says what kind of file it is | `file photo` |
| `stat` | Details: size, times, permissions | `stat notes.txt` |
| `du` | Space used | `du -sh *` |
| `df` | Free space per disk | `df -h` |
| `rsync` | Copies smartly, resumes, mirrors | `rsync -av src/ /mnt/d/backup/` |

## Reading files

| Command | Does | Example |
| --- | --- | --- |
| `cat` | Prints a file | `cat notes.txt` |
| `less` | Pages through it (q quits, /search) | `less big.log` |
| `head` / `tail` | Start / end of a file | `tail -n 100 -f app.log` |
| `wc` | Counts lines, words, bytes | `wc -l *.py` |
| `diff` | Differences between files | `diff -u old new` |
| `nano`, `vim`, `micro` | Edit (see [Editing text files](editors.md)) | `nano notes.txt` |

## Searching

| Command | Does | Example |
| --- | --- | --- |
| `find` | Finds files by name, size, date | `find . -name "*.pdf"` |
| `grep` | Finds text in files | `grep -rin "todo" .` |
| `which` / `type` | Where a command lives | `which python3` |
| `locate` | Instant name search (plocate) | `locate nginx.conf` |

More in [Finding files and text](finding.md).

## Text tools

`sort`, `uniq`, `cut`, `tr`, `sed`, `awk`, `tee`, `xargs`, `column` — see
[Pipes, redirection and combining commands](pipes.md).

## Permissions and users

| Command | Does | Example |
| --- | --- | --- |
| `sudo` | Runs one command as administrator | `sudo apt update` |
| `chmod` | Changes permissions | `chmod +x script.sh`, `chmod 600 key` |
| `chown` | Changes owner | `sudo chown -R you:you folder` |
| `whoami` / `id` | Who you are, your groups | `id` |
| `passwd` | Changes your password | `passwd` |
| `su` | Becomes another user | `sudo su -` (root shell; careful) |
| `adduser`, `usermod` | Manage accounts | `sudo usermod -aG docker you` |

More in [Permissions](permissions.md) and [Your account, sudo and passwords](accounts.md).

## Software

| Task | apt (Ubuntu, Mint, Debian) | pacman (Arch, CachyOS, Omarchy) |
| --- | --- | --- |
| Update everything | `sudo apt update && sudo apt upgrade` | `sudo pacman -Syu` |
| Search | `apt search x` | `pacman -Ss x` |
| Install | `sudo apt install x` | `sudo pacman -S x` |
| Remove | `sudo apt remove x` | `sudo pacman -Rs x` |
| About | `apt show x` | `pacman -Si x` |
| List installed | `apt list --installed` | `pacman -Q` |
| Which package owns a file | `dpkg -S /path` | `pacman -Qo /path` |
| Files of a package | `dpkg -L x` | `pacman -Ql x` |

Also `flatpak`, `snap`, `pip`/`uv`, `npm`, `cargo` — see [Installing software](software.md).

## Processes and the system

| Command | Does | Example |
| --- | --- | --- |
| `ps` | Lists processes | `ps aux \| grep node` |
| `top` / `htop` / `btop` | Live view | `htop` |
| `kill` / `pkill` | Stops a process | `kill 1234`, `pkill -f "npm run"` |
| `systemctl` | Manages services | `systemctl status ssh` |
| `journalctl` | System logs | `journalctl -u nginx -f` |
| `free` | Memory | `free -h` |
| `uptime` | How long it has run, load | `uptime` |
| `uname` | Kernel version | `uname -a` |
| `lsb_release`, `/etc/os-release` | Which Linux | `cat /etc/os-release` |
| `lscpu`, `nproc` | Processor | `nproc` |
| `env` / `export` | Environment variables | `export EDITOR=nano` |
| `history` | Earlier commands | `history \| tail` |
| `date` / `timedatectl` | Date, time zone | `date +%F` |

More in [Running programs, services and memory](processes.md).

## Network

| Command | Does | Example |
| --- | --- | --- |
| `ping` | Is a host reachable? | `ping -c 3 example.com` |
| `curl` | Fetches a URL | `curl -LO https://…/file` |
| `wget` | Downloads | `wget https://…/file` |
| `ip` | Addresses and routes | `ip addr`, `ip route` |
| `ss` | Listening ports | `ss -tlnp` |
| `dig` / `nslookup` | Looks up names | `dig example.com` |
| `ssh` / `scp` | Remote sign-in / copy | `ssh you@server` |

More in [Networking, ports and the internet](networking.md).

## Archives

| Command | Does | Example |
| --- | --- | --- |
| `tar` | .tar, .tar.gz, .tar.xz | `tar -xf a.tar.gz`, `tar -czf b.tar.gz dir/` |
| `zip` / `unzip` | .zip | `unzip a.zip -d out` |
| `7z` | .7z and most others | `7z x a.7z` |

More in [Zip, tar and other archives](archives.md).

## WSL and Windows

| Command | Does | Example |
| --- | --- | --- |
| `explorer.exe` | Opens File Explorer | `explorer.exe .` |
| `wslpath` | Converts paths | `wslpath -w ~` |
| `clip.exe` | Copies to Windows' clipboard | `cat key.pub \| clip.exe` |
| `cmd.exe`, `powershell.exe` | Runs Windows commands | `powershell.exe -c Get-Date` |
| `wsl.exe` (in Windows) | Manages systems | `wsl -l -v`, `wsl --shutdown` |

More in [Windows and Linux together](windows-interop.md) and [The wsl command](wsl-commands.md).

## Getting help

| Command | Does |
| --- | --- |
| `man x` | The manual (q quits, /search) |
| `x --help` | A short summary |
| `tldr x` | Examples (install `tldr`) |
| `apropos word` | Commands whose description mentions *word* |
| `info x` | Longer manuals for GNU tools |
