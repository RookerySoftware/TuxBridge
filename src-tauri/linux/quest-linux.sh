#!/bin/bash
# Linux Basics Quest: a practice course in ~/linux-quest that teaches the everyday terminal
# skills (ls, cd, cat, grep, find, pipes, chmod, base64) by hiding a flag in each challenge.
# Flags look like FLAG{...}; the learner types each into EasyWSL's Learn page. Needs only bash
# and coreutils, so it runs on every system. Running it again starts the quest over.
#
# The flags are checked by the app against their SHA-256; keep src/learn/quests.ts in step.

set -eu
Q=$HOME/linux-quest
if [ -d "$Q" ]; then chmod -R u+rwx "$Q" 2> /dev/null || true; rm -rf "$Q"; fi
mkdir -p "$Q"
cd "$Q"

# 1. Reading a file.
cat > README.txt << 'EOF'
                       LINUX BASICS QUEST

Each challenge hides a flag that looks like FLAG{something}. Type each flag
into EasyWSL's Learn page to tick it off. Every challenge has a hint there if
you get stuck.

Handy commands to start:
    ls            what is in this folder?   (ls -la shows hidden files and details)
    cat FILE      show a file
    cd FOLDER     go into a folder          (cd .. goes back up)

Challenge 1 (done!): reading this file earns   FLAG{first_steps_taken}
Now run  ls -la  to see everything here, including hidden files.
EOF

# 2. Hidden files.
mkdir -p attic
echo "Dusty boxes. Nothing to see with a plain ls..." > attic/boxes.txt
echo "Behind the boxes, a hidden file: FLAG{hidden_in_plain_sight}" > attic/.ledger

# 3. Going deep with cd.
mkdir -p basement/vault/inner/safe
echo "FLAG{deep_in_the_vault}" > basement/vault/inner/safe/prize.txt
echo "Keep going down: cd vault, then look around." > basement/hint.txt

# 4. Searching a file with grep.
{
  for i in $(seq 1 600); do echo "line $i: the cat sat on the mat"; done
  echo "line 601: the key is FLAG{grep_found_the_line}"
  for i in $(seq 602 1200); do echo "line $i: the cat sat on the mat"; done
} > haystack.txt
cat > SEARCH-ME.txt << 'EOF'
haystack.txt has 1200 lines and one of them contains the word "key".
Reading it all would be slow. Search instead:

    grep key haystack.txt

grep prints only the lines that match.
EOF

# 5. Counting with a pipe.
seq 1 1000 > numbers.txt
cat > COUNT-ME.txt << 'EOF'
How many lines are in numbers.txt? Let one command feed another with a pipe (|):

    cat numbers.txt | wc -l

(wc -l counts lines.) When you have the number N, your flag is  FLAG{counted_N}
For example if it were 42, the flag would be FLAG{counted_42}.
EOF

# 6. find.
mkdir -p warehouse/aisle3/shelf7
echo "FLAG{find_located_it}" > warehouse/aisle3/shelf7/crate.txt
for a in 1 2 4 5; do mkdir -p "warehouse/aisle$a"; echo "empty" > "warehouse/aisle$a/crate.txt"; done
cat > LOCATE-ME.txt << 'EOF'
Somewhere under "warehouse" is a file named crate.txt with a flag in it, but
there are many crate.txt files. Let the computer find them all:

    find warehouse -name crate.txt

Then read each one (cat ...) until you find the flag. Only one has it.
EOF

# 7. Permissions and chmod.
printf '#!/bin/sh\necho "FLAG{made_it_runnable}"\n' > reveal.sh
chmod 600 reveal.sh
cat > RUN-ME.txt << 'EOF'
reveal.sh is a little program, but it is not allowed to run yet. Make it
runnable, then run it:

    chmod +x reveal.sh
    ./reveal.sh

It prints your flag.
EOF

# 8. Decoding base64 (not secret, just encoded - a very common thing to meet).
echo "RkxBR3tiYXNlNjRfZGVjb2RlZH0=" > scrambled.txt
cat > DECODE-ME.txt << 'EOF'
scrambled.txt is written in base64 - a common way to pack text so it survives
being copied around. It is NOT a secret code; anyone can turn it back:

    cat scrambled.txt | base64 -d

That prints the real text, which is your flag.
EOF

echo "The quest is ready in ~/linux-quest. Type: cat README.txt"
