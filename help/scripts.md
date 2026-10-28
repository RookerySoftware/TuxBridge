---
title: Writing shell scripts
summary: Turn commands into a script: variables, ifs, loops, arguments, and making it runnable.
section: Using Linux
order: 23
---

# Writing shell scripts

A script is a text file of commands, run one after another. If you type the same commands
often, put them in a script.

## Your first script

```bash
mkdir -p ~/.local/bin
nano ~/.local/bin/hello
```

```bash
#!/usr/bin/env bash
# Says hello and shows the date.
echo "Hello, $USER!"
echo "Today is $(date +'%A %d %B')."
```

```bash
chmod +x ~/.local/bin/hello     # allow it to run
hello                           # runs it (~/.local/bin is in PATH on most systems)
./hello                         # or, from the folder it is in
```

The first line, `#!/usr/bin/env bash` (the *shebang*), says which program runs the file. Lines
starting with `#` are comments.

If `hello` alone says *command not found*, open a new terminal, or see PATH in
[Setting up your shell](shell-setup.md).

## Safer scripts

Start scripts with this, so they stop at the first error instead of carrying on:

```bash
#!/usr/bin/env bash
set -euo pipefail
```

- `-e` stops when a command fails,
- `-u` stops when you use a variable that was never set (catches typos),
- `-o pipefail` counts a failure anywhere in a pipe.

## Variables

```bash
name="Sam"                 # no spaces around =
echo "Hi $name"
echo "Hi ${name}!"         # braces when letters follow
files=$(ls | wc -l)        # a command's output
```

**Always put variables in double quotes** (`"$name"`): a value with spaces otherwise splits
into several words, and `rm $file` on "My File" removes two wrong files.

## Arguments

```bash
#!/usr/bin/env bash
echo "The script is $0"
echo "First argument: $1, second: $2"
echo "All of them: $@ ($# in total)"
```

`myscript one "two words"` gives `$1` = one and `$2` = two words.

## Asking a question

```bash
read -rp "Your name? " name
echo "Hello, $name"
```

## If

```bash
if [ -f ~/.bashrc ]; then
  echo "You have a .bashrc"
elif [ -d ~/projects ]; then
  echo "You have a projects folder"
else
  echo "Neither"
fi
```

| Test | True when |
| --- | --- |
| `[ -f path ]` | A file exists |
| `[ -d path ]` | A folder exists |
| `[ -z "$x" ]` | `x` is empty |
| `[ "$a" = "$b" ]` | Two texts are the same (`!=` not the same) |
| `[ "$n" -gt 5 ]` | A number is greater than 5 (`-lt`, `-eq`, `-ge`, `-le`, `-ne`) |
| `command -v git > /dev/null` | A command exists |

The spaces inside `[ ]` are required.

## Loops

```bash
for f in *.jpg; do
  echo "Found $f"
done

for i in {1..5}; do
  echo "Round $i"
done

while read -r line; do
  echo "Line: $line"
done < list.txt
```

## Functions

```bash
greet() {
  echo "Hello, $1"
}
greet World
```

## Did it work? Exit codes

Every command ends with a code: 0 for success, anything else for failure. `$?` holds the last
one, and `if` checks it directly:

```bash
if ping -c1 -W2 example.com > /dev/null; then
  echo "Online"
else
  echo "Offline"
fi
```

Your script can end with `exit 0` (fine) or `exit 1` (failed).

## A real example: back up a folder with the date

```bash
#!/usr/bin/env bash
set -euo pipefail
source="${1:-$HOME/projects}"            # the first argument, or ~/projects
target="/mnt/c/Users/$(cmd.exe /c 'echo %USERNAME%' 2> /dev/null | tr -d '\r')/Documents/backups"
mkdir -p "$target"
file="$target/$(basename "$source")-$(date +%Y-%m-%d).tar.gz"
tar -czf "$file" -C "$(dirname "$source")" "$(basename "$source")"
echo "Saved $file"
```

It saves a dated `.tar.gz` of a folder into your Windows Documents\backups.

## Tips

- Check scripts with **shellcheck** (`sudo apt install shellcheck`): it explains mistakes.
- `bash -x script.sh` runs a script showing each command, to see where it goes wrong.
- Write scripts in Linux, or set your Windows editor to LF line endings: CRLF breaks them
  (`$'\r': command not found`).
- For anything long or complicated, Python is often easier than bash.
