---
title: Setting up your shell
summary: .bashrc, aliases, PATH and environment variables, a nicer prompt, history, and zsh or fish.
section: Using Linux
order: 22
---

# Setting up your shell

The program that reads your commands is the **shell**. On most systems it is **bash**; Omarchy
and some others use it too, and some people switch to **zsh** or **fish**. You can shape it to
suit you.

`echo $SHELL` shows which one you have.

## ~/.bashrc: your settings

Bash reads `~/.bashrc` every time a terminal opens. Anything you'd type by hand to set things up
can go there.

```bash
nano ~/.bashrc       # edit it
source ~/.bashrc     # use the changes now, without opening a new terminal
```

Add your own lines at the end. If something goes wrong, the file is plain text: remove what you
added.

## Aliases: short names for long commands

```bash
alias ll='ls -lah'
alias update='sudo apt update && sudo apt upgrade'     # or: sudo pacman -Syu
alias ..='cd ..'
alias gs='git status'
alias open='explorer.exe'
```

Put them in `~/.bashrc`. `alias` alone lists them; `unalias ll` removes one for now.

## Functions: little commands of your own

```bash
mkcd() { mkdir -p "$1" && cd "$1"; }     # makes a folder and goes into it
backup() { cp "$1" "$1.bak"; }
```

## Environment variables

Variables pass settings to programs. Some common ones:

| Variable | Holds |
| --- | --- |
| `HOME` | Your home folder |
| `USER` | Your user name |
| `PATH` | The folders searched for commands |
| `EDITOR` | The editor programs open for you |
| `LANG` | Language and character set, like `en_US.UTF-8` |

```bash
echo $HOME                     # show one
env                            # show them all
export EDITOR=nano             # set one for this terminal and the programs it starts
```

Put `export` lines in `~/.bashrc` to keep them.

## PATH: where commands are found

When you type `node`, the shell looks through the folders in `PATH`, in order. A program in
another folder needs its full path, or that folder added to `PATH`:

```bash
echo $PATH | tr ':' '\n'                  # one folder per line
export PATH="$HOME/.local/bin:$PATH"      # add a folder (first = wins)
```

`~/.local/bin` is the usual place for your own programs and scripts; many systems add it to
`PATH` already when it exists.

**In WSL**, Windows' own folders are added to `PATH` too, so `notepad.exe` and `code` work. That
makes the list long and can slow down tab completion; see [Configuring WSL](wsl-config.md) to
turn it off.

## History

```bash
history              # your recent commands, numbered
!42                  # runs number 42 again
!!                   # the last command (sudo !! = again, with sudo)
```

**Ctrl+R** then typing searches backwards through history; Ctrl+R again for older matches,
Enter runs it, Esc keeps it on the line for editing.

To keep more history, add to `~/.bashrc`:

```bash
HISTSIZE=10000
HISTFILESIZE=20000
HISTCONTROL=ignoreboth      # skip duplicates and lines starting with a space
```

## A nicer prompt

The prompt (`you@ubuntu:~$`) is set by the `PS1` variable. A simple coloured one:

```bash
PS1='\[\e[32m\]\u@\h\[\e[0m\]:\[\e[34m\]\w\[\e[0m\]\$ '
```

For something fancier without learning `PS1`, **Starship** works in every shell:

```bash
curl -sS https://starship.rs/install.sh | sh
echo 'eval "$(starship init bash)"' >> ~/.bashrc
```

Starship and many terminal themes use symbols from **Nerd Fonts**. Install one on *Windows*
(the terminal draws the text) and pick it in Windows Terminal's settings.

## zsh and fish

- **zsh** is bash-compatible with better completion and themes (often with *Oh My Zsh*).
- **fish** suggests commands as you type, out of the box, but its scripting differs from bash.

```bash
sudo apt install zsh fish        # or: sudo pacman -S zsh fish
chsh -s $(which zsh)             # make zsh your shell (next terminal)
chsh -s $(which bash)            # and back
```

Each has its own settings file: `~/.zshrc`, `~/.config/fish/config.fish`.

## Other dot-files

Files starting with a dot are hidden; `ls -a` shows them. In your home folder:

| File | For |
| --- | --- |
| `~/.bashrc` | Every interactive bash |
| `~/.profile` or `~/.bash_profile` | Login shells (read once at sign-in) |
| `~/.config/` | Most programs' settings, one folder each |
| `~/.local/bin/` | Your own commands |
| `~/.ssh/` | SSH keys (see [Git and SSH keys](git-ssh.md)) |
| `~/.gitconfig` | Git's settings |

Keeping these files in a Git repository ("dotfiles") lets you set up a new system in a minute.
