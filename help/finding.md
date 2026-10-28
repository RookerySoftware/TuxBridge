---
title: Finding files and text
summary: find, grep, locate, and the faster fd, ripgrep and fzf, with everyday examples.
section: Using Linux
order: 19
---

# Finding files and text

Two questions come up all the time: *where is that file?* and *which file says that?*

## Where is that file? `find`

`find` walks through folders and lists what matches. It is on every Linux.

```bash
find . -name "notes.txt"            # in this folder and below, exactly this name
find ~ -iname "*.pdf"               # every PDF in your home folder, any capitals
find . -type d -name "build"        # folders (d) called build
find . -type f -size +100M          # files bigger than 100 MB
find . -mtime -2                    # changed in the last 2 days
find /etc -name "*.conf" 2> /dev/null   # hide "Permission denied" messages
```

- `.` means "here"; `~` your home folder; `/` everything (slow, and full of permission errors).
- Put patterns with `*` in quotes, or the shell expands them first.
- `2> /dev/null` throws error messages away (see [Pipes and redirection](pipes.md)).

Doing something with each result:

```bash
find . -name "*.tmp" -delete                      # deletes them (check the list first!)
find . -name "*.jpg" -exec cp {} ~/photos/ \;     # copies each one
```

## Which file says that? `grep`

`grep` prints the lines that contain some text.

```bash
grep "error" app.log                 # lines with "error" in app.log
grep -i "error" app.log              # any capitals
grep -n "TODO" *.py                  # with line numbers, in every .py file here
grep -r "password" ~/project         # in every file below a folder
grep -rl "password" ~/project        # just the names of the files
grep -v "debug" app.log              # lines WITHOUT "debug"
grep -c "error" app.log              # how many lines match
grep -E "error|warning" app.log      # either word (extended patterns)
grep -w "cat" story.txt              # the whole word, not "category"
```

grep also filters what other commands print:

```bash
ps aux | grep firefox
history | grep apt
```

## Finding a program

```bash
which python3          # where the command you'd run lives
type ll                # also says if it's an alias or built into the shell
whereis nano           # the program, its manual and source
command -v git         # nothing printed = not installed
```

## Faster, friendlier tools

These are worth installing once you search often:

| Tool | Replaces | Install (apt / pacman) |
| --- | --- | --- |
| **fd** | `find` | `sudo apt install fd-find` (the command is `fdfind`) / `sudo pacman -S fd` |
| **ripgrep** (`rg`) | `grep -r` | `sudo apt install ripgrep` / `sudo pacman -S ripgrep` |
| **fzf** | searching by hand | `sudo apt install fzf` / `sudo pacman -S fzf` |
| **locate** | `find /` | `sudo apt install plocate` / `sudo pacman -S plocate` |

```bash
fd notes                     # files with "notes" in the name, skipping hidden and ignored ones
rg "TODO"                    # searches the whole folder, fast, skipping .git and node_modules
rg -i "connection refused" /var/log 2> /dev/null
locate nginx.conf            # instant, from a database (sudo updatedb refreshes it)
```

**fzf** is a fuzzy finder: type part of a name and pick from a live list.

```bash
nano $(fzf)                  # pick a file, then edit it
```

After installing fzf, **Ctrl+R** in the terminal can search your history with it; the fzf
package explains how to turn that on (`/usr/share/doc/fzf`).

## Searching Windows files from Linux

`find` and `grep` work under `/mnt/c` too, but Windows' drives are much slower to search from
Linux. Narrow it down (`/mnt/c/Users/you/Documents`, not `/mnt/c`), or search with Windows
itself.

## Searching inside a long output

Pipe it into `less`, then search with `/word` (Enter, then `n` for the next match, `q` to leave):

```bash
journalctl | less
```
