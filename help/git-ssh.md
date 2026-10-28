---
title: Git and SSH keys
summary: Set up Git, make an SSH key, connect to GitHub or GitLab, and share credentials with Windows.
section: Programming
order: 30
---

# Git and SSH keys

**Git** keeps the history of a project and syncs it with sites like GitHub and GitLab. It is
usually installed already; if not: `sudo apt install git` or `sudo pacman -S git`.

## Set it up once

```bash
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
git config --global init.defaultBranch main
git config --global core.editor nano          # the editor for commit messages
git config --global pull.rebase false         # the usual pull behaviour, without asking
```

The email is public in every commit you push. GitHub can give you a private one
(*Settings → Emails → Keep my email address private*), shaped like
`12345+name@users.noreply.github.com`.

## Everyday Git

```bash
git clone https://github.com/someone/project.git   # copy a project
cd project
git status                     # what changed
git add file.txt               # stage a change (git add . for everything)
git commit -m "Fix the typo"   # record it
git push                       # send commits to the server
git pull                       # get others' commits
git log --oneline --graph      # history
git diff                       # changes not staged yet
git switch -c new-idea         # a new branch
git switch main                # back to main
```

**Keep projects inside Linux** (`~/projects`), not under `/mnt/c`: Git is many times faster on
Linux's own disk, and file permissions behave.

## SSH keys: signing in without a password

An SSH key is a pair of files: a **private key** that stays on your PC, and a **public key** you
give to GitHub, servers and so on.

```bash
ssh-keygen -t ed25519 -C "you@example.com"
```

Press Enter for the default place (`~/.ssh/id_ed25519`). A passphrase is optional but protects
the key if someone copies the file.

```bash
cat ~/.ssh/id_ed25519.pub        # the PUBLIC key: this is the one you share
```

**Never share the file without `.pub`.**

### Add it to GitHub

1. Copy the public key: `cat ~/.ssh/id_ed25519.pub | clip.exe` puts it on the Windows clipboard.
2. GitHub → *Settings → SSH and GPG keys → New SSH key*, paste, save.
3. Test: `ssh -T git@github.com` (answer `yes` the first time). It greets you by name.

Then clone with the SSH address (`git@github.com:someone/project.git`). To switch an existing
project: `git remote set-url origin git@github.com:someone/project.git`.

GitLab, Bitbucket and Codeberg work the same way, under their profile's *SSH Keys*.

### Not typing the passphrase every time

```bash
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519
```

The agent remembers it until the system stops. `keychain` (`sudo apt install keychain`) keeps
one agent across terminals: add `eval "$(keychain --eval --quiet id_ed25519)"` to `~/.bashrc`.

## HTTPS with Windows' Git Credential Manager

If Git for Windows is installed, Linux's Git can use its credential manager, which signs in
through the browser and stores the token in Windows:

```bash
git config --global credential.helper "/mnt/c/Program\ Files/Git/mingw64/bin/git-credential-manager.exe"
```

(The path is where Git for Windows installs it by default.)

## Signing in to servers with SSH

```bash
ssh user@server.example.com                  # sign in
ssh-copy-id user@server.example.com          # install your public key there (then no password)
scp file.txt user@server:/home/user/         # copy a file over
```

`~/.ssh/config` saves typing:

```
Host myserver
    HostName server.example.com
    User sam
    IdentityFile ~/.ssh/id_ed25519
```

Then just `ssh myserver`.

## Using the same keys as Windows

Windows keeps its keys in `C:\Users\you\.ssh`. To use them in Linux, copy them and fix the
permissions (SSH refuses keys others can read):

```bash
cp /mnt/c/Users/you/.ssh/id_ed25519* ~/.ssh/
chmod 700 ~/.ssh
chmod 600 ~/.ssh/id_ed25519
chmod 644 ~/.ssh/id_ed25519.pub
```

## Common problems

| Message | Fix |
| --- | --- |
| `Permission denied (publickey)` | The public key isn't added to the site, or the wrong key is used. `ssh -vT git@github.com` shows which keys it tries. |
| `WARNING: UNPROTECTED PRIVATE KEY FILE!` | `chmod 600 ~/.ssh/id_ed25519` |
| `Author identity unknown` | Set `user.name` and `user.email` (top of this guide). |
| Every file shows as changed | Line endings or permissions from Windows: `git config core.autocrlf input` and `git config core.fileMode false`, or move the project into Linux. |
| `fatal: detected dubious ownership` | A project under `/mnt/c`. Move it into Linux, or run the `git config --global --add safe.directory` command Git suggests. |
