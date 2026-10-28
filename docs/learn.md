# Learn

The **Learn** page teaches Linux, the terminal and the basics of security, inside Pengi, so a
beginner never has to leave the app to find out how something works. It has two kinds of content
and a Kids mode.

## Lessons

Guided, step-by-step walk-throughs. Each lesson is a sequence of steps with a title, a short
explanation (Markdown) and the exact command to type, a progress bar, and a "Finish" that ticks
it off. Unlike the Help guides (which you read), a lesson leads you through doing it.

Lessons live in `src/learn/lessons.ts` as plain data (`Lesson` → `Step[]`). Each lesson names
the `systems` it suits (catalog ids, or `"any"`) and may be `kids: true`. Current lessons: first
commands, installing software, the Windows/Linux file bridge, and tours of Ubuntu, Kali and
Parrot, plus a kids' "Be a computer explorer".

## Quests

HackTheBox-style challenge sets solved in a real system. A quest:

1. **Sets up** — the app runs a bundled script (`src-tauri/linux/quest-<id>.sh`) in a system the
   person picks, through the `start_quest` command. The script builds a folder of harmless
   practice files (e.g. `~/linux-quest`) and the app opens a terminal there.
2. **Challenges** — each hides a flag (`FLAG{...}`, `CTF{...}`, `STAR{...}`). The person finds it
   in the terminal and types it back. Flags are checked **in the browser** against their SHA-256
   (`hashFlag` in `src/learn/quests.ts`), so answers never ship in the clear. Progress and hints
   are per challenge; everything is kept in `localStorage`.

Quests: **Linux Basics** (any system — ls, cd, grep, find, pipes, chmod, base64), **Security
Basics** (Kali or Parrot), and **Treasure Hunt** (kids).

### Security Basics is defensive only

The security quest teaches the everyday, defensive skills of an analyst on **make-believe local
files**: decoding base64, reading an auth log to spot a brute-force pattern, verifying a
checksum, finding a secret left in a config (so you learn not to leave one), and identifying a
file by its magic bytes. Nothing touches a real system or network, and every surface repeats the
rule: **only ever test systems you own or have written permission to test.**

## Adding a quest challenge

1. Add the flag to the quest's script (`src-tauri/linux/quest-<id>.sh`).
2. Compute `sha256(flag)` and add a `Challenge` with that `hash` to `src/learn/quests.ts`.
3. The script is bundled via `tauri.conf.json` (`linux/`) and served through `prepare_linux`, so
   a new version is picked up automatically.

## Kids mode

A toggle (kept in `localStorage`) that swaps the page to gentler, `kids: true` content: the
"Be a computer explorer" lesson and the "Treasure Hunt" quest, with friendlier wording.
