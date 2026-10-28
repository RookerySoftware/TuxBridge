#!/bin/bash
# Security Basics Quest (for Kali and Parrot): a defensive, hands-on introduction to the skills
# a security analyst uses every day - reading logs, decoding data, checking file integrity, and
# spotting suspicious patterns - all on harmless practice files in ~/security-quest. Nothing here
# touches any real system or network: it is a reading-and-reasoning exercise, like a puzzle.
# Flags look like CTF{...}; the learner types each into EasyWSL's Learn page. Running it again
# starts the quest over.
#
# The flags are checked by the app against their SHA-256; keep src/learn/quests.ts in step.

set -eu
Q=$HOME/security-quest
if [ -d "$Q" ]; then chmod -R u+rwx "$Q" 2> /dev/null || true; rm -rf "$Q"; fi
mkdir -p "$Q"
cd "$Q"

cat > BRIEFING.txt << 'EOF'
                    SECURITY BASICS QUEST  (defensive)

You are a junior analyst. A folder of harmless practice "evidence" is in front
of you. Work through it to learn the everyday skills of reading data safely.
None of this touches a real computer or network - it is all make-believe files.

Every challenge hides a flag like CTF{something}. Type each into EasyWSL's Learn
page. Hints are there for every one.

GOLDEN RULE, printed on every analyst's wall:
    Only ever test systems you own or have WRITTEN permission to test.
    The skills here are for defending and understanding, not attacking.

First flag, for reading the briefing: CTF{read_the_briefing}
Start with:  ls -la
EOF

# 1. Decoding base64 - data is often encoded, not encrypted. Reading it is step one.
echo "Q1RGe2VuY29kaW5nX2lzX25vdF9zZWNyZWN5fQ==" > intercepted.b64
cat > 01-decode.txt << 'EOF'
Data you meet is often ENCODED (packed into safe characters), which is not the
same as ENCRYPTED (locked). Analysts decode first. intercepted.b64 is base64:

    cat intercepted.b64 | base64 -d

The result is your flag - and its words are worth remembering.
EOF

# 2. Reading an auth log - spotting a brute-force pattern (defensive monitoring).
{
  echo "Oct 27 09:01:11 server sshd[2011]: Accepted password for alice from 10.0.0.5 port 50122"
  for i in $(seq 1 30); do
    echo "Oct 27 09:1$((i % 6)):$((i % 60)) server sshd[$((3000 + i))]: Failed password for admin from 203.0.113.44 port 4${i}00"
  done
  echo "Oct 27 09:20:02 server sshd[4099]: Accepted password for admin from 203.0.113.44 port 44112"
  echo "Oct 27 09:21:00 server sshd[4100]: note CTF{many_failures_then_success}"
} > auth.log
cat > 02-logs.txt << 'EOF'
auth.log is a practice login record. One address tries the "admin" account over
and over (that pattern is a brute-force attempt) and then gets in. Count how many
times a login FAILED:

    grep "Failed password" auth.log | wc -l

Then read the line right after the successful admin login for your flag:

    grep -A1 "Accepted password for admin" auth.log
EOF

# 3. File integrity - verifying a checksum, the way you confirm a download was not tampered with.
echo "the quick brown fox jumps over the lazy dog" > download.bin
good=$(sha256sum download.bin | cut -d' ' -f1)
echo "$good  download.bin" > download.sha256
cat > 03-integrity.txt << 'EOF'
Before trusting a file, you check its fingerprint (a checksum) against the one
the author published. download.sha256 holds the expected fingerprint. Verify it:

    sha256sum -c download.sha256

If it says "OK", the file is intact. When a check passes, your flag is
    CTF{integrity_verified}
EOF

# 4. Finding a secret left in a config - teaching why you never hard-code passwords.
mkdir -p app/config app/logs app/src
echo "debug=false" > app/config/app.conf
echo "theme=dark" > app/config/ui.conf
echo "# TODO: move this out of the file! db_password=CTF{never_commit_secrets}" > app/config/.env
echo "starting up" > app/logs/app.log
cat > 04-secrets.txt << 'EOF'
Developers sometimes leave passwords sitting in files by mistake - a real risk.
Search the whole "app" folder for anything that looks like a password:

    grep -r -i password app

grep -r searches every file in a folder. The flag is the leaked value. (The
lesson: real projects keep secrets OUT of their files.)
EOF

# 5. Identifying a file by its contents, not its name.
printf '\x89PNG\r\n\x1a\n' > mystery.dat
printf 'the rest does not matter for this puzzle\n' >> mystery.dat
cat > 05-filetype.txt << 'EOF'
A file's name can lie; its first bytes usually tell the truth. "mystery.dat"
is not really data. Ask what it is:

    file mystery.dat

(If "file" is missing: sudo apt install file) When you know the real format,
your flag is CTF{its_really_a_png} - the tool names the picture format it is.
EOF

echo "The quest is ready in ~/security-quest. Type: cat BRIEFING.txt"
