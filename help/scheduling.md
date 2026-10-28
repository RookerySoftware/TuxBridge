---
title: Running things on a schedule
summary: cron and systemd timers, and what to know about schedules in WSL, which stops when idle.
section: Using Linux
order: 24
---

# Running things on a schedule

Linux has two ways to run something regularly: **cron** (simple, everywhere) and **systemd
timers** (more control). First, one thing that is different in WSL.

## WSL stops when nothing uses it

A WSL system shuts down a short while after its last terminal or program closes. A job scheduled
for 3 a.m. only runs if the system is running then. For things that must happen even when
Linux isn't open, use **Windows' Task Scheduler** to run a Linux command:

- Program: `wsl.exe`
- Arguments: `-d <system name> -e bash -lc "/home/you/bin/backup"`

Task Scheduler starts the system, runs the command, and WSL stops it again afterwards.

## cron

Each user has a *crontab*, a list of times and commands:

```bash
crontab -e        # edit yours (first time: pick an editor; nano is easiest)
crontab -l        # show it
```

Each line is five time fields and a command:

```
# minute hour day-of-month month day-of-week  command
  30     2    *            *     *            /home/you/bin/backup
  */15   *    *            *     *            /home/you/bin/check-site
  0      9    *            *     1-5          echo "Weekday!" >> /home/you/log.txt
  @reboot                                     /home/you/bin/on-start
```

| Field | Values |
| --- | --- |
| minute | 0–59 |
| hour | 0–23 |
| day of month | 1–31 |
| month | 1–12 |
| day of week | 0–7 (0 and 7 are Sunday) |

`*` means every; `*/15` every 15; `1-5` a range; `1,15` a list. **crontab.guru** explains any
line in plain words.

Tips:

- Use full paths: cron runs with a very short `PATH`.
- Save the output, or it is lost: `command >> ~/cron.log 2>&1`.
- cron needs to be running. With systemd (Pengi turns it on), it usually is:
  `systemctl status cron` (Ubuntu, Mint, Debian) or `systemctl status cronie` (Arch; install
  `cronie` and run `sudo systemctl enable --now cronie`).

## systemd timers

A timer starts a *service* at set times, with logs in the journal. Make two files:

`~/.config/systemd/user/backup.service`

```ini
[Unit]
Description=Back up my projects

[Service]
Type=oneshot
ExecStart=/home/you/bin/backup
```

`~/.config/systemd/user/backup.timer`

```ini
[Unit]
Description=Back up every day

[Timer]
OnCalendar=daily
Persistent=true

[Install]
WantedBy=timers.target
```

```bash
systemctl --user daemon-reload
systemctl --user enable --now backup.timer
systemctl --user list-timers          # when it runs next
journalctl --user -u backup.service   # what it printed
```

`Persistent=true` runs a missed job as soon as the system starts again, which suits WSL well.
`OnCalendar` also takes things like `hourly`, `weekly`, `Mon *-*-* 09:00` or `*-*-* *:00/15`.

## Running something once, later

```bash
sleep 600 && echo "Tea is ready"        # in 10 minutes, in this terminal
```

`at` runs a command at a time (`sudo apt install at`, then `echo "command" | at 18:00`), as
long as the system is still running then.
