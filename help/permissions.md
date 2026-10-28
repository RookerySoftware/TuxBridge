---
title: Permissions and owners
summary: Who may read, change or run a file; chmod, chown, and "Permission denied".
section: Using Linux
order: 15
---

# Permissions and owners

Every file in Linux belongs to a user and a group, and says who may **read** it, **write**
(change) it and **execute** (run) it. This is why you need `sudo` to change system files but
not your own.

## Reading them

```bash
ls -l
```

```
-rwxr-xr-- 1 you you  220 Oct 27 10:00 backup.sh
```

The first column, after the type (`-` file, `d` folder), is three groups of three:

| Part | Who | Here |
| --- | --- | --- |
| `rwx` | the owner (`you`) | may read, write and run it |
| `r-x` | the group (`you`) | may read and run it |
| `r--` | everyone else | may only read it |

## Changing them

```bash
chmod +x backup.sh        # make a script runnable
chmod u+w notes.txt       # let the owner (u) write it
chmod go-r private.txt    # take reading away from group (g) and others (o)
chmod 644 notes.txt       # the usual for a file: owner reads and writes, others read
chmod 755 tool.sh         # the usual for a program: owner everything, others read and run
sudo chown you:you file   # give a file to you (often needed after making it with sudo)
```

The numbers add up per group: read 4, write 2, execute 1. So 7 = rwx, 6 = rw-, 5 = r-x,
4 = r--.

## "Permission denied"

| When | Usually means | Fix |
| --- | --- | --- |
| Running `./script.sh` | It is not executable | `chmod +x script.sh` |
| Editing a file in `/etc` | It belongs to root | `sudo nano /etc/...` |
| A file in your home | You made it with `sudo`, so root owns it | `sudo chown you:you file` |
| On `/mnt/c/...` | Windows' permissions apply there | Keep the file in your Linux home |

Do not "fix" errors with `chmod 777` or by running everything with `sudo`: it hides the
real problem and lets anything change the file.
