// Quests for the Learn page: HackTheBox-style challenge sets that a learner works through in a
// real Linux system. A backend command (`start_quest`) runs a bundled script that builds a folder
// of practice files; each challenge hides a flag, and the learner types it back here. Flags are
// checked in the browser against their SHA-256, so the answers are never shipped in the clear.
//
// The hashes below MUST match the flags the scripts write (src-tauri/linux/quest-*.sh). To add a
// challenge: write the flag in the script, then put sha256(flag) here.

export interface Challenge {
  /** Short name shown in the list. */
  title: string;
  /** Markdown: what to do, and which command teaches it. */
  prompt: string;
  /** A nudge, revealed on request. */
  hint: string;
  /** Lowercase hex SHA-256 of the exact flag string. */
  hash: string;
}

export interface Quest {
  id: string;
  /** The bundled script name without .sh: the backend maps it to src-tauri/linux/quest-<id>.sh. */
  script: string;
  title: string;
  summary: string;
  /** The folder the script builds, shown so the learner knows where to look. */
  folder: string;
  /** The shape of the flags, e.g. "STAR{...}", shown as a hint of what to look for. */
  flagShape: string;
  /** Catalog ids it suits, or "any". */
  systems: string[];
  kids?: boolean;
  /** Shown above the challenges (the rules, the tone). */
  intro: string;
  challenges: Challenge[];
}

export const quests: Quest[] = [
  {
    id: "kids",
    script: "kids",
    title: "Treasure Hunt",
    summary: "A gentle first adventure: find hidden treasures with three simple commands.",
    folder: "~/treasure-hunt",
    flagShape: "STAR{...}",
    systems: ["any"],
    kids: true,
    intro:
      "Treasures are hidden all around the treasure-hunt folder. Use `ls` to look, `cd` to go into folders, and `cat` to read files. When you find a treasure like `STAR{...}`, type it in below!",
    challenges: [
      { title: "Read the first note", prompt: "Open a terminal in the folder and read the welcome note: `cat START-HERE.txt`. Your first treasure is right there!", hint: "Type: cat START-HERE.txt", hash: "42ff39089018eb9defc4286aecdef903c7609716d2a728f88a93023c9265ae7f" },
      { title: "Open the chest in the cave", prompt: "Go into the cave and open the chest: `cd cave` then `cat chest.txt`.", hint: "cd cave, then cat chest.txt. Go back with cd ..", hash: "734d2bdd6fe0622b2b0ce7b84cc52e8c83cb4a57bda37894b821d8aac91b8bda" },
      { title: "Smell the special flower", prompt: "Visit the garden and read about the flowers: `cd garden` then `cat flowers.txt`. One flower hides a treasure.", hint: "cd garden, then cat flowers.txt", hash: "d57a9d66157d3500fceb2837b2c1925e5a057b280fcb641030f442577a407cca" },
      { title: "Find the hidden treasure", prompt: "Some files are hidden. In the garden, show hidden files with `ls -a`, then read the buried one: `cat .buried`.", hint: "In the garden: ls -a shows a file starting with a dot. Read it with cat .buried", hash: "702314b454fbbac4d01138619347b2047686d2027ba960a7e09a7c2f872162b3" },
      { title: "Make your own treasure", prompt: "Read `LAST-STEP.txt` in the garden and follow it: make a file with `echo`, then read it back. You make this treasure yourself!", hint: "echo STAR{i_made_a_file} > my-flag.txt, then cat my-flag.txt", hash: "915ff7f0004879ca0f0dd9196a014e9bd8461eadbc9e6bfe5e1632f1b64b3401" },
    ],
  },
  {
    id: "linux",
    script: "linux",
    title: "Linux Basics",
    summary: "Eight challenges that teach the everyday terminal skills: ls, cd, grep, find, pipes, chmod and more.",
    folder: "~/linux-quest",
    flagShape: "FLAG{...}",
    systems: ["any"],
    intro:
      "A practice course in your linux-quest folder. Each challenge teaches one skill and hides a `FLAG{...}`. Read `README.txt` first. There is a hint for every one.",
    challenges: [
      { title: "First steps", prompt: "Open a terminal and read the intro: `cat README.txt`. Reading it earns your first flag.", hint: "Type: cat README.txt", hash: "37b151ed5b817bf75c0e021d09dfcc708f259feacb1651c11c8333ffbba29b12" },
      { title: "Hidden files", prompt: "In the `attic` folder, plain `ls` hides something. Use `ls -la` to see all files, then read the hidden one.", hint: "cd attic, then ls -la shows .ledger — read it with cat .ledger", hash: "2877fc8d5af233dcd93cf9044d64c28f33774a8fdf8f9548d747f3964aefb396" },
      { title: "Deep in the vault", prompt: "Follow the folders down under `basement` until you reach `prize.txt`. Use `cd` and `ls` to feel your way.", hint: "basement/vault/inner/safe/prize.txt — cd one step at a time, or cat the whole path", hash: "7114ffd9f3f8500c24279ffc934602ce88232e5e5fcc77f199d0570c13a64824" },
      { title: "Search with grep", prompt: "`haystack.txt` has 1200 lines. Find the one containing the word `key` without reading them all: `grep key haystack.txt`.", hint: "grep key haystack.txt", hash: "6c63c86e012c4812203fa57301c9e0ddb8b694f59748e15747e90ba64fd3e7f8" },
      { title: "Count with a pipe", prompt: "Count the lines in `numbers.txt` by feeding one command into another: `cat numbers.txt | wc -l`. The flag is FLAG{counted_N}, with N the number you get.", hint: "cat numbers.txt | wc -l gives 1000, so the flag is FLAG{counted_1000}", hash: "dd6d77c46c3854a0da202c68d785e18c9deccd510879d49f4d1aaff7bd8ac8d7" },
      { title: "Find a file", prompt: "Many `crate.txt` files hide under `warehouse`, but only one holds the flag. List them all with `find warehouse -name crate.txt`, then read each.", hint: "find warehouse -name crate.txt, then cat each until one has FLAG{...}", hash: "7d10eb0ebc0777d46dee02a32dcb35927ee2bb1b17e8cf775ba9378f2ea10ce5" },
      { title: "Make it runnable", prompt: "`reveal.sh` is a program that is not allowed to run yet. Make it runnable and run it: `chmod +x reveal.sh` then `./reveal.sh`.", hint: "chmod +x reveal.sh, then ./reveal.sh", hash: "90a6bd6473b8f21a4660e0499ebbad2fb4431dc396e5adaa912507608163577b" },
      { title: "Decode base64", prompt: "`scrambled.txt` is written in base64 (encoded, not secret). Turn it back: `cat scrambled.txt | base64 -d`.", hint: "cat scrambled.txt | base64 -d", hash: "773be9cfb99e43d88f1506387ef92cf0c66cb85cba08f0190712199a252b09c9" },
    ],
  },
  {
    id: "kali",
    script: "kali",
    title: "Security Basics",
    summary: "A safe, defensive intro to an analyst's everyday skills: reading logs, decoding data, checking files and spotting suspicious patterns.",
    folder: "~/security-quest",
    flagShape: "CTF{...}",
    systems: ["kali", "parrot"],
    intro:
      "You are a junior analyst with a folder of harmless practice evidence. Nothing here touches a real system or network — it is a reading-and-reasoning puzzle. **Golden rule: only ever test systems you own or have written permission to test.** Read `BRIEFING.txt` first.",
    challenges: [
      { title: "Read the briefing", prompt: "Open a terminal and read the case file: `cat BRIEFING.txt`. It sets the rules and gives your first flag.", hint: "Type: cat BRIEFING.txt", hash: "a0784487605bd2480ab883b7bf96311e3eb012d34d713aad9f6beb84c3c7befc" },
      { title: "Decode, don't guess", prompt: "`intercepted.b64` is base64-encoded. Decode it: `cat intercepted.b64 | base64 -d`. (Encoding is not the same as encryption.)", hint: "cat intercepted.b64 | base64 -d", hash: "2c6be4c25b8c21f60ac15420fd2668395b24e323519ce13e84ec49f0414b616f" },
      { title: "Read the logs", prompt: "`auth.log` shows login attempts. One address fails many times then gets in (a brute-force pattern). Count the failures with `grep \"Failed password\" auth.log | wc -l`, then read the line after the admin login: `grep -A1 \"Accepted password for admin\" auth.log`.", hint: "The flag is on the line right after the successful admin login. Use grep -A1.", hash: "4a7c6411ee683075b346e2d6551476b88aab64af3df1450d2790a6ad1c11bf0d" },
      { title: "Verify a file", prompt: "Before trusting a download you check its fingerprint. Verify `download.bin` against the published checksum: `sha256sum -c download.sha256`. A passing check earns CTF{integrity_verified}.", hint: "sha256sum -c download.sha256 — if it says OK, the flag is CTF{integrity_verified}", hash: "d939a64ecd96473b4e9138195d687fcee23b464bb92b71874de2be9454238fe5" },
      { title: "Find the leaked secret", prompt: "A password was left in a config file by mistake — a real-world risk. Search the whole `app` folder: `grep -r -i password app`. (The lesson: keep secrets out of files.)", hint: "grep -r -i password app — the flag is the leaked value in a hidden .env file", hash: "1399c31bba7660bd0503a1cf443507d3f6e940721fb9c459eb5d2496b50a7dcc" },
      { title: "Name that file", prompt: "A file's name can lie; its first bytes tell the truth. Ask what `mystery.dat` really is: `file mystery.dat`. (If `file` is missing: `sudo apt install file`.)", hint: "file mystery.dat reveals it is a PNG image. The flag is CTF{its_really_a_png}", hash: "ae5f64339353e63948ef8fd977c483dabd4ed23d46ee7ad43df4bf1580fd612a" },
    ],
  },
];

/** SHA-256 of a typed flag, lowercase hex, for comparing against a challenge's hash. */
export async function hashFlag(input: string): Promise<string> {
  const data = new TextEncoder().encode(input.trim());
  const digest = await crypto.subtle.digest("SHA-256", data);
  return Array.from(new Uint8Array(digest))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
