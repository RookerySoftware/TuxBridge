---
title: Running programs, services and memory
summary: See what is running, stop a stuck program, and understand services and memory in WSL.
section: Using Linux
order: 16
---

# Running programs, services and memory

Every running program is a **process**. Most of the time you do not think about them, but when
something hangs or a port is busy, these commands help.

## What is running?

```bash
htop            # a live, colourful view (install it first: see Installing software)
top             # the same idea, always available; press q to quit
ps aux          # a one-off list of every process
ps aux | grep firefox   # just the ones matching a name
```

In `htop`, the columns show each process's share of the processor and memory. Press `F10` or
`q` to leave.

## Stopping a program

If a program is stuck, stop it by name or by its number (PID, shown by `ps` and `htop`):

```bash
pkill firefox        # stop every process named firefox
kill 12345           # stop the process with PID 12345
kill -9 12345        # force it to stop, if it ignores the polite request
```

In `htop` you can select a process and press `F9` to stop it.

## Services

Some programs run in the background as **services** (a database, a web server). Many Linux
systems manage them with `systemctl`:

```bash
systemctl status ssh        # is it running?
sudo systemctl start ssh    # start it now
sudo systemctl enable ssh   # start it automatically in future
```

WSL turns systemd on for systems made by TuxBridge. In a very small or imported system without
systemd, start background programs yourself, or with the service's own command.

## Memory and disk

```bash
free -h              # memory in use and free
df -h                # disk space per drive
du -sh ~/*           # the size of each thing in your home folder
```

WSL shares your PC's memory with Windows and gives it back when Linux is done. If a Linux
system holds on to too much, stopping it (**Stop** on its card) frees it at once.

## Ports

A program that serves something (a website, a database) listens on a **port**. To see what is
listening, or what is using a port you need:

```bash
ss -tlnp             # everything listening (add sudo to see which program)
```

Apps you reach from Windows at `http://localhost:<port>` are the Linux side's listening ports;
WSL forwards them to Windows for you.
