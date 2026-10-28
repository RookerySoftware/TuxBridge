---
title: Pipes, redirection and combining commands
summary: Send output to files and other commands, chain commands, and the handy small tools that go with them.
section: Using Linux
order: 20
---

# Pipes, redirection and combining commands

Linux commands are small tools that do one thing. The power comes from joining them.

## Output into a file: `>` and `>>`

```bash
ls > list.txt              # writes ls's output into list.txt (replacing what was there)
echo "one more" >> list.txt   # adds to the end instead
```

**`>` replaces the file without asking.** Use `>>` when you mean to add.

Commands print two streams: normal output (*stdout*, number 1) and errors (*stderr*, number 2).

```bash
command 2> errors.txt          # only the errors into a file
command > all.txt 2>&1         # both into one file
command 2> /dev/null           # throw errors away (/dev/null swallows everything)
command &> /dev/null           # throw everything away (bash)
```

## A file into a command: `<`

```bash
sort < names.txt
```

## One command into another: the pipe `|`

`|` sends what the left command prints into the right one:

```bash
ls -l | less                 # page through a long listing
history | grep docker        # earlier commands that mention docker
ps aux | grep -i chrome      # running programs called chrome
cat access.log | sort | uniq -c | sort -rn | head   # most common lines, top 10
```

## The small tools that go with pipes

| Command | Does | Example |
| --- | --- | --- |
| `less` | Pages through text (space, b, /search, q) | `dmesg \| less` |
| `head` / `tail` | First / last lines (`-n 20`) | `tail -n 50 app.log` |
| `tail -f` | Keeps showing new lines as they arrive | `tail -f app.log` |
| `sort` | Sorts lines (`-n` numbers, `-r` reverse, `-h` sizes like 2K) | `du -sh * \| sort -h` |
| `uniq` | Removes repeated neighbours (`-c` counts them) | `sort words \| uniq -c` |
| `wc` | Counts lines, words, bytes (`-l` lines) | `ls \| wc -l` |
| `cut` | Picks columns | `cut -d: -f1 /etc/passwd` |
| `tr` | Changes characters | `echo hi \| tr a-z A-Z` |
| `sed` | Finds and replaces | `sed 's/cat/dog/g' story.txt` |
| `awk` | Works with columns | `awk '{print $1}' access.log` |
| `tee` | Writes to a file *and* the screen | `make 2>&1 \| tee build.log` |
| `xargs` | Turns lines into arguments | `find . -name "*.log" \| xargs rm` |
| `column -t` | Lines up columns | `mount \| column -t` |

## Running several commands

```bash
mkdir demo; cd demo              # ; runs one after another, no matter what
sudo apt update && sudo apt upgrade   # && runs the second only if the first worked
ping -c1 example.com || echo "offline" # || runs the second only if the first failed
```

## A command's output inside another: `$( )`

```bash
echo "Today is $(date +%A)"
cd $(dirname $(which python3))
```

## Running in the background

```bash
long-task &            # runs in the background; the terminal is free again
jobs                   # lists background jobs
fg                     # brings the last one back to the front
```

**Ctrl+Z** pauses the command in front; `bg` then continues it in the background. To keep
something running after the terminal closes, use `nohup long-task &`, or better a terminal
multiplexer like `tmux` (`sudo apt install tmux`): start `tmux`, run things, press **Ctrl+B**
then **D** to leave it running, and `tmux attach` to come back.

## Quoting

The shell treats some characters specially: spaces split words, `*` matches files, `$` starts a
variable.

| Write | Means |
| --- | --- |
| `"My File.txt"` | One name with a space; `$VARIABLES` still work inside |
| `'cost: $5'` | Exactly as written; nothing special inside |
| `My\ File.txt` | A backslash makes the next character ordinary |

When a command from a guide has quotes, copy them exactly.

## Wildcards

```bash
ls *.txt          # every name ending in .txt
ls photo?.jpg     # photo1.jpg, photoA.jpg (one character)
ls file[1-3].txt  # file1.txt, file2.txt, file3.txt
cp ~/docs/{a,b}.txt .   # a.txt and b.txt
mkdir -p project/{src,tests,docs}
```

The shell expands wildcards before the command runs, so `rm *` really means every file here.
Check with `ls` first, with the same pattern.
