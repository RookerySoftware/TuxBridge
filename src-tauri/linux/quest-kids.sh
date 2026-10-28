#!/bin/bash
# Kids' Treasure Hunt: a gentle first taste of the terminal. It builds a play folder in
# ~/treasure-hunt full of little challenges that teach ls, cd, cat and making files. Every
# treasure is a word like STAR{...}; the child types it into Pengi's Learn page. Needs only
# bash and coreutils, so it works on every system. Running it again starts the hunt over.
#
# The treasures are checked by the app against their SHA-256; keep src/learn/quests.ts in step.

set -eu
Q=$HOME/treasure-hunt
if [ -d "$Q" ]; then chmod -R u+rwx "$Q" 2> /dev/null || true; rm -rf "$Q"; fi
mkdir -p "$Q"
cd "$Q"

cat > START-HERE.txt << 'EOF'
                    ~ THE TREASURE HUNT ~

Hello, explorer! Treasures are hidden all around this folder. Each one looks
like STAR{something}. When you find a treasure, type it into the Learn page in
Pengi to collect it.

Three commands do everything:
    ls              look around   (what is here?)
    cd NAME         go inside a folder   (cd .. goes back)
    cat NAME        read a file

Your first treasure, just for reading this: STAR{hello_explorer}

Now type:   ls
...to see what is here.
EOF

# A plain file to read.
cat > the-map.txt << 'EOF'
The map says: a treasure chest sits inside the folder called "cave".
Type:   cd cave      then   ls      to look inside it.
EOF

# A folder to go into.
mkdir -p cave
cat > cave/chest.txt << 'EOF'
You opened the chest! Inside, glittering: STAR{gold_in_the_cave}

There is also a door to the "garden". Go back up first:  cd ..
then:  cd garden
EOF

# A second folder, with a treasure in a file you have to read.
mkdir -p garden
cat > garden/flowers.txt << 'EOF'
Roses, daisies, sunflowers. One flower is special and hides a treasure:
    the sunflower whispers STAR{sunny_flower}
EOF

# A tiny bit of hidden-file magic, explained kindly.
cat > garden/secret-note.txt << 'EOF'
Psst... some files are HIDDEN. They start with a dot (.) and ls does not show
them. To see hidden files too, type:   ls -a
There is a hidden treasure right here in the garden. Can you find it?
EOF
echo "You found the hidden treasure! STAR{dot_files_hide}" > garden/.buried

# A make-your-own step: the child creates a file to finish.
cat > garden/LAST-STEP.txt << 'EOF'
Last one! This treasure you MAKE yourself.
Type this exactly:

    echo STAR{i_made_a_file} > my-flag.txt

Then read it back:   cat my-flag.txt
...and type the treasure into the Learn page. Well done, explorer!
EOF

echo "The treasure hunt is ready in ~/treasure-hunt. Type: cat START-HERE.txt"
