// What Pip says: tips for each page, explanations of the buttons you point at, fun facts, the
// first-launch tour, the tour of a new system, and (in Kids mode) little lessons with a quiz.

import type { Mood } from "./Penguin";

export type Page = "systems" | "get" | "docker" | "learn" | "assistant" | "help" | "settings";

/** A button in Pip's bubble. */
export interface Action {
  label: string;
  /** Go to a page. */
  go?: Page;
  /** Open a guide. */
  help?: string;
  /** Do something with the system being shown around. */
  run?: "terminal" | "files";
}

export interface Bubble {
  title?: string;
  /** Markdown: **bold** and `code`. */
  text: string;
  mood?: Mood;
  actions?: Action[];
}

export interface TourStep extends Bubble {
  /** The data-tour of the element to point at. */
  target?: string;
  /** The page to show for this step. */
  page?: Page;
}

// ---- Tips for each page ----

export const pageTips: Record<Page, { normal: string[]; kids: string[] }> = {
  systems: {
    normal: [
      "This is **My systems**: every Linux on this PC, including ones installed without EasyWSL. **Terminal** opens a command line, **Files** opens its files in Windows.",
      "The **⋯** menu on a card has **Update**, **Stop**, **Make default**, adding a desktop, and the guide for that system.",
      "A system that is not in use stops by itself after a short while, and gives its memory back to Windows.",
      "Typing `wsl` in PowerShell opens the **default** system. **Make default** in the ⋯ menu picks which one.",
    ],
    kids: [
      "These are your **Linux computers**! They live inside this PC, like a computer inside a computer.",
      "Press **Terminal** to talk to Linux by typing. Try typing `ls` and pressing Enter!",
      "**Files** shows what is inside, like opening a toy box.",
    ],
  },
  get: {
    normal: [
      "Not sure which to choose? **Ubuntu** is the safest first pick: most guides online are written for it.",
      "You can install several systems side by side, even two of the same kind with different names.",
      "**Arch Linux** is the best base for the Hyprland desktop; **Linux Mint** feels most like Windows.",
      "Each system needs a few GB. **More options** in the install form lets you put it on another drive.",
    ],
    kids: [
      "Pick a Linux to install! **Ubuntu** is a great first one.",
      "Ask a grown-up before installing: it downloads a big file and takes a few minutes.",
    ],
  },
  docker: {
    normal: [
      "**Docker** runs apps in containers: a whole app (a media server, a database) in one box, without installing it into a system.",
      "Turn Docker on once, then install apps from the **Apps** tab with one click. They open at `http://localhost:<port>`.",
      "`docker` works from PowerShell and from your Linux systems too, all talking to the same engine.",
    ],
    kids: ["Docker keeps apps in little boxes called **containers**, so they don't make a mess. This page is for grown-ups!"],
  },
  learn: {
    normal: [
      "**Lessons** walk you through doing something, step by step. **Challenges** are puzzles you solve in a real terminal.",
      "Challenges build practice files in a system you pick, so you can't break anything that matters.",
      "Stuck on a challenge? Each one has a hint.",
    ],
    kids: [
      "Here are adventures just for you! Try the **Treasure Hunt** in Challenges.",
      "Every flag you find is a little secret word. Type it in to win a star!",
      "I have my own lessons too: press the **book** in my bubble.",
    ],
  },
  assistant: {
    normal: [
      "**Ask Pip** is me, with an AI that runs on this PC: your questions never leave it.",
      "I read the guides and see which systems you have, so you can ask me things like *how do I update my Ubuntu?*",
      "Prefer Claude or Codex? **Settings** can install an EasyWSL skill for them instead.",
    ],
    kids: ["Here you can ask me questions by typing! Ask a grown-up to help you."],
  },
  help: {
    normal: [
      "The search understands **operators**: `\"exact words\"`, `-word` to leave out, `a OR b`, `title:`, `in:ubuntu`, `section:docker`, `code:apt` and `word*`. Press **?** next to the search to see them all.",
      "Press **Ctrl+K** or **/** anywhere on this page to jump to the search.",
      "Each system has its own guide under **Your system**, and the guides link to each other.",
    ],
    kids: ["These are guides, like an instruction book. Try searching for **files**!"],
  },
  settings: {
    normal: [
      "Here you can turn me on or off, switch on **Kids mode**, or watch the welcome tour again.",
      "**Folder for new systems** decides where new systems keep their disks. A fast SSD helps.",
    ],
    kids: ["Settings change how the app works. Best to ask a grown-up before changing things here."],
  },
};

// ---- What a button does, when you point at it ----

/** Keys are matched against the start of a button's label, ignoring case. */
export const explanations: [string, string][] = [
  ["My systems", "Your Linux systems: open them, update them, add a desktop or remove them."],
  ["Get Linux", "Choose a Linux to install. Each one is described in plain words."],
  ["Docker", "Run ready-made apps in containers, and use the docker command."],
  ["Learn", "Guided lessons, hands-on challenges and Kids mode."],
  ["Ask Pip", "Ask me anything about EasyWSL, Linux, WSL, Windows and Docker. My AI runs on this PC."],
  ["Help", "Guides for everything in EasyWSL and Linux, with a search."],
  ["Settings", "Where new systems go, desktop options, Docker, my AI, Kids mode and me."],
  ["Terminal", "Opens a command line in this system: you type a command, press Enter, and it runs."],
  ["Open desktop", "Opens this system's desktop in its own window. Close the window to end the session."],
  ["Set up and open", "Finishes setting up the desktop the first time, then opens it."],
  ["Files", "Opens this system's files in Windows File Explorer. Your home folder is in /home/<your name>."],
  ["Update", "Installs the newest versions of everything. It uses the system's own package manager."],
  ["Stop", "Shuts it down now and frees its memory. It starts again when you open it."],
  ["Start", "Starts it."],
  ["Restart WSL", "Stops every Linux system at once and starts WSL afresh. Save your work first."],
  ["Restart", "Stops it and starts it again."],
  ["Make default", "The default system is the one `wsl` opens when you type it in PowerShell."],
  ["Make it the default system", "The default system is the one `wsl` opens when you type it in PowerShell."],
  ["Add the Hyprland desktop", "Adds Hyprland: a modern tiling desktop driven by the keyboard. You keep the terminal and any other desktop."],
  ["Add the", "Adds this system's own desktop, with a menu, taskbar and windows."],
  ["Guide to", "Opens the guide for this system: first steps, software, common problems."],
  ["Remove", "Deletes it and everything in it. Copy out what you want to keep first."],
  ["Install", "Starts installing. You can keep using EasyWSL while it works."],
  ["Show me around", "When the system is ready, I'll walk you through it."],
  ["More options", "Where its disk goes, and whether it becomes the default system."],
  ["Use the Windows key", "Hyprland's shortcuts use a main key. The Windows key is easiest; Alt works too."],
  ["Allow the community package sources", "Hyprland comes from community package sources on this system; this allows them."],
  ["Folder for its disk", "Where its virtual disk goes. A fast drive (SSD) makes Linux much quicker."],
  ["Name", "What this system is called on your PC. Any name, as long as it is not already used."],
  ["User name", "Your Linux account. Lower-case letters and digits, like `sam`."],
  ["Password", "Linux asks for it when you install software or change the system (sudo). Pick one you'll remember."],
  ["Show its log", "What the app inside the container has been printing: useful when something is wrong."],
  ["Disk use and clean-up", "See how much space Docker uses and free what is not needed."],
  ["Remove Docker", "Removes Docker's engine and everything in it: apps, images and their data."],
  ["Browse apps", "Ready-made apps you can install in one click."],
  ["Open", "Opens the app in your browser."],
  ["Refresh", "Checks again."],
  ["Kids mode", "Switches to friendly projects for children, and turns me into a teacher!"],
  ["Search the guides", "Search all the guides. Try `\"exact words\"`, `-leave-out`, `title:`, `in:ubuntu` or `code:apt`."],
  ["Other ways to open", "More ways to open this system: its terminal, or another desktop."],
  ["Send", "Sends your question to me."],
  ["Hint", "Shows a clue. No penalty for peeking!"],
  ["Check", "Checks the flag you typed."],
  ["Set up and open terminal", "Builds the practice files in the system you picked, then opens a terminal right there."],
  ["Next", "The next step."],
  ["Back", "Goes back."],
  ["Finish", "Done! Marks it as finished."],
];

// ---- Fun facts ----

export const funFacts: string[] = [
  "Linus Torvalds announced Linux in 1991 as *just a hobby*. Today it runs most of the world's servers.",
  "Every one of the world's 500 fastest supercomputers runs Linux, and has since 2017.",
  "Android phones run on the Linux kernel, so Linux is probably in your pocket.",
  "**grep** gets its name from an old editor command, `g/re/p`: *globally search for a regular expression and print*.",
  "**sudo** means *superuser do*: run one command as the all-powerful root user.",
  "Debian's releases are named after Toy Story characters: Buzz, Woody, Bookworm, Trixie…",
  "Ubuntu's releases have alliterative animal names, like *Noble Numbat* and *Jammy Jellyfish*.",
  "**Ubuntu** is a Nguni Bantu word often translated as *humanity to others*.",
  "**Debian** is named after its founder Ian Murdock and his then-girlfriend Debra.",
  "Linus Torvalds also wrote **Git**, in 2005, to manage the Linux kernel's code.",
  "NASA's Ingenuity helicopter, which flew on Mars, ran Linux.",
  "WSL 2 runs a real Linux kernel in a small, fast virtual machine that Windows manages for you.",
  "In Linux, *everything is a file*: even disks and keyboards appear as files under `/dev`.",
  "Press **Tab** in a terminal to finish a command or file name for you. Press it twice to see the choices.",
  "Press **Ctrl+R** in a terminal and type part of an old command to find it again.",
  "`cd -` jumps back to the folder you were in before. `cd` alone goes home.",
  "`sudo !!` runs your last command again, this time with sudo.",
  "**Arch Linux** is a *rolling release*: there are no big upgrades, just a steady flow of small ones.",
  "Files that start with a dot, like `.bashrc`, are hidden. `ls -a` shows them.",
  "`man <command>` opens the manual for almost any command. Press **q** to leave it.",
  "Linux file names care about capitals: `Notes.txt` and `notes.txt` are two different files.",
  "The **root** folder `/` is the top of everything in Linux; there is no C: drive. Windows' C: is at `/mnt/c`.",
];

export const kidsFacts: string[] = [
  "Penguins can't fly in the air, but they *fly* underwater with their flippers!",
  "The **emperor penguin** is the tallest penguin: about as tall as a 5-year-old.",
  "The smallest penguin is the **little penguin**, only about 30 cm tall. That's about one ruler!",
  "Almost all penguins live in the southern half of the world. The Galápagos penguin lives right by the equator!",
  "Emperor penguin dads keep their egg warm on their feet for about two months, in the middle of winter.",
  "Penguins are black on the back and white on the front, so they are hard to spot from above *and* below the water.",
  "A group of penguins swimming together is called a **raft**.",
  "Linux has a penguin mascot too! That's why I'm here.",
  "A computer only really understands 0s and 1s. Everything you see is made of them!",
  "The first computer *bug* was a real moth stuck inside a computer in 1947.",
  "A **password** is like a toothbrush: don't share it, and change it if someone sees it.",
  "Linux runs on tiny computers like the Raspberry Pi, and on giant supercomputers too.",
];

// ---- The first-launch tour ----

export const appTour: TourStep[] = [
  {
    title: "Hi! I'm Pip",
    text: "I'm EasyWSL's penguin. Want a quick tour? It takes about a minute, and you can skip it any time.",
    mood: "happy",
  },
  {
    title: "My systems",
    target: "nav:systems",
    page: "systems",
    text: "Your Linux systems live here. Each one has a **Terminal**, **Files**, and a **⋯** menu with updates, desktops and more.",
  },
  {
    title: "Get Linux",
    target: "nav:get",
    page: "get",
    text: "Pick a Linux to install. Each is described in plain words; **Ubuntu** is a great first choice. Tick **Show me around** and I'll explain your new system when it's ready.",
  },
  {
    title: "Docker",
    target: "nav:docker",
    page: "docker",
    text: "One button turns on Docker. Then you can install ready-made apps (a media server, cloud storage, an AI chat) with one click.",
  },
  {
    title: "Learn",
    target: "nav:learn",
    page: "learn",
    text: "Step-by-step lessons and puzzles you solve in a real terminal. There's a **Kids mode** too.",
  },
  {
    title: "Ask Pip",
    target: "nav:assistant",
    page: "assistant",
    text: "Ask me anything, in your own words. My AI is optional and runs on your own PC, so your questions stay private. I know the guides and your systems.",
  },
  {
    title: "Help",
    target: "nav:help",
    page: "help",
    text: "Guides for everything, so you never have to leave the app. The search is clever: try `\"exact words\"`, `-leave-out` or `in:ubuntu`.",
  },
  {
    title: "Settings",
    target: "nav:settings",
    page: "settings",
    text: "Where new systems go, desktop options, Kids mode, and me. You can bring this tour back from here too.",
  },
  {
    title: "That's it!",
    text: "Shall I stay around? I'll explain buttons when you point at them, give tips for each page, and share fun facts. You can change your mind in **Settings**.",
    mood: "happy",
  },
];

// ---- Showing a new system around ----

interface SystemFacts {
  /** Its short name: "Linux Mint". */
  name: string;
  /** What it is, in a sentence. */
  about: string;
  install: string;
  update: string;
  search: string;
  /** Something worth knowing about this one. */
  tip: string;
}

const facts: Record<string, SystemFacts> = {
  ubuntu: {
    name: "Ubuntu",
    about: "**Ubuntu** is the Linux most guides online are written for, so answers are easy to find.",
    install: "sudo apt install htop",
    update: "sudo apt update && sudo apt upgrade",
    search: "apt search <word>",
    tip: "Ubuntu prefers *snaps* for some programs, but snaps don't work well in WSL. Use `apt` (or Flatpak) instead.",
  },
  mint: {
    name: "Linux Mint",
    about: "**Linux Mint** is built on Ubuntu and made to feel familiar to Windows users, with the Cinnamon desktop.",
    install: "sudo apt install htop",
    update: "sudo apt update && sudo apt upgrade",
    search: "apt search <word>",
    tip: "Almost every Ubuntu guide works on Mint too: it uses the same software.",
  },
  debian: {
    name: "Debian",
    about: "**Debian** is the steady, reliable Linux that Ubuntu, Mint and many others are built on.",
    install: "sudo apt install htop",
    update: "sudo apt update && sudo apt upgrade",
    search: "apt search <word>",
    tip: "Debian's software is a little older but very well tested. Need newer versions? Look at *backports* in its guide.",
  },
  arch: {
    name: "Arch Linux",
    about: "**Arch Linux** is a lean, always-up-to-date Linux you build up yourself, with a huge wiki.",
    install: "sudo pacman -S htop",
    update: "sudo pacman -Syu",
    search: "pacman -Ss <word>",
    tip: "Update often, and always update everything at once (`pacman -Syu`). Never install with `-Sy` alone.",
  },
  cachyos: {
    name: "CachyOS",
    about: "**CachyOS** is Arch Linux tuned for speed, with its own optimised packages.",
    install: "sudo pacman -S htop",
    update: "sudo pacman -Syu",
    search: "pacman -Ss <word>",
    tip: "Everything from the Arch guides works here, and the Arch Wiki is your best friend.",
  },
  omarchy: {
    name: "Omarchy",
    about: "**Omarchy** is a ready-made, beautiful Hyprland setup on Arch Linux, by DHH.",
    install: "sudo pacman -S htop",
    update: "omarchy-update",
    search: "pacman -Ss <word>",
    tip: "The first time you open it, Omarchy asks for your account, keyboard and time zone, like a new PC. Then **Win+K** lists all its keyboard shortcuts.",
  },
};

const generic: SystemFacts = {
  name: "",
  about: "This is a Linux system, running right next to Windows.",
  install: "sudo apt install htop",
  update: "sudo apt update && sudo apt upgrade",
  search: "apt search <word>",
  tip: "The **Update** button in the ⋯ menu always uses the right command for this system.",
};

const desktopGuides: Record<string, string> = { GNOME: "gnome", Cinnamon: "cinnamon", Xfce: "xfce" };

/** The tour of a new system, `name`, installed from catalogue entry `catalogId`. */
export function systemTour(name: string, catalogId: string, title: string, desktop: string | null, hyprland: boolean): TourStep[] {
  const f = facts[catalogId] ?? generic;
  const short = f.name || title;
  const target = `distro:${name}`;
  const steps: TourStep[] = [
    { title: `${name} is ready!`, text: `${f.about} Let me show you around.`, mood: "happy", page: "systems", target },
    {
      title: "Opening it",
      target,
      text: `**Terminal** opens a command line in ${name}. ${desktop || hyprland ? "The desktop button opens its desktop in a window of its own." : "Linux apps you install still open in their own windows."} **Files** shows its files in Windows Explorer.`,
      actions: [{ label: "Open a terminal", run: "terminal" }],
    },
    {
      title: "Your account",
      target,
      text: "You log in automatically. When a command needs to change the system, start it with `sudo` and type **your Linux password**. Nothing shows while you type it: that's normal.",
    },
    {
      title: "Installing software",
      target,
      text: `Software comes from ${short}'s own app store, through its *package manager*. To install a program:\n\n\`${f.install}\`\n\nTo look for one: \`${f.search}\``,
      actions: [{ label: "Installing software", help: "software" }],
    },
    {
      title: "Staying up to date",
      target,
      text: `**Update** in the **⋯** menu updates everything. In a terminal it is:\n\n\`${f.update}\``,
    },
    {
      title: "Your files",
      target,
      text: "Your own files go in your home folder, `~`. Windows' drives are inside Linux at `/mnt/c`, `/mnt/d`… Linux files are fastest kept in Linux.",
      actions: [{ label: "Open its files", run: "files" }, { label: "Files guide", help: "files" }],
    },
  ];
  if (hyprland) {
    steps.push({
      title: "The Hyprland desktop",
      target,
      text: "Windows arrange themselves in tiles. **Win+Q** opens a terminal, **Win+C** closes a window, **Win+1…9** switches workspaces. The first open sets the desktop up.",
      actions: [{ label: "Hyprland guide", help: "hyprland" }],
    });
  } else if (desktop) {
    steps.push({
      title: `The ${desktop} desktop`,
      target,
      text: "It opens in a window of its own. **Ctrl+Alt+Enter** switches between full screen and a window, and closing the window ends the session, so save your work first.",
      actions: [{ label: `${desktop} guide`, help: desktopGuides[desktop] ?? "desktop-window" }],
    });
  }
  steps.push(
    { title: "Good to know", target, text: f.tip },
    {
      title: "Where to learn more",
      text: `The guide to ${short} (in the **⋯** menu, or under **Help**) has first steps, software and common problems. **Learn** has lessons you can follow in ${name}. Have fun!`,
      mood: "happy",
      actions: [
        { label: "Read the guide", help: catalogId in facts ? catalogId : "first-steps" },
        { label: "Go to Learn", go: "learn" },
      ],
    },
  );
  return steps;
}

// ---- Kids mode: Pip's lessons ----

export interface KidsLesson {
  id: string;
  title: string;
  cards: string[];
  quiz: { question: string; options: string[]; answer: number; why: string };
}

export const kidsLessons: KidsLesson[] = [
  {
    id: "computer",
    title: "What is a computer?",
    cards: [
      "A computer is a machine that follows **instructions**, super fast. Millions of them every second!",
      "The instructions are called **programs**. A game is a program. A web browser is a program.",
      "Computers only understand **0** and **1** deep down. People write programs in languages that get turned into 0s and 1s.",
    ],
    quiz: { question: "What do we call the instructions a computer follows?", options: ["Programs", "Pancakes", "Pebbles"], answer: 0, why: "Programs are lists of instructions for the computer." },
  },
  {
    id: "os",
    title: "What is Linux?",
    cards: [
      "Every computer needs a boss program that runs everything else. It's called an **operating system**.",
      "Windows is an operating system. **Linux** is another one, and it's free for everybody!",
      "Linux runs phones, robots, TVs, cars and even helicopters on Mars. Its mascot is a penguin, like me!",
    ],
    quiz: { question: "Windows and Linux are both…", options: ["Video games", "Operating systems", "Keyboards"], answer: 1, why: "They are operating systems: the boss program of a computer." },
  },
  {
    id: "files",
    title: "Files and folders",
    cards: [
      "A **file** holds one thing: a story, a picture, a song.",
      "A **folder** holds files, and other folders too. Like boxes inside boxes!",
      "In Linux your own folder is called **home**, and its short name is `~` (a squiggle called a tilde).",
    ],
    quiz: { question: "What holds files?", options: ["A folder", "A mouse", "A screen"], answer: 0, why: "Folders hold files and other folders." },
  },
  {
    id: "terminal",
    title: "Talking to the computer",
    cards: [
      "The **terminal** lets you talk to the computer by typing words called **commands**.",
      "Type a command, then press **Enter**. The computer answers with words.",
      "Try `echo Hello Pip!` The computer says it back to you. `echo` means *repeat after me*.",
    ],
    quiz: { question: "Which key sends your command to the computer?", options: ["Space", "Enter", "Shift"], answer: 1, why: "Enter tells the computer: go!" },
  },
  {
    id: "look",
    title: "Looking around",
    cards: [
      "`ls` means **list**. It shows what's in the folder you are in.",
      "`pwd` means **print where I am**. It tells you which folder you're in.",
      "`cd` means **change directory** (directory is another word for folder). `cd Documents` walks into Documents, and `cd ..` walks back out.",
    ],
    quiz: { question: "Which command shows what's in a folder?", options: ["ls", "cd", "pwd"], answer: 0, why: "ls lists the things in a folder." },
  },
  {
    id: "make",
    title: "Making things",
    cards: [
      "`mkdir treasure` makes a new folder called **treasure**. (*mkdir* = make directory.)",
      "`echo gold > treasure/chest.txt` writes the word *gold* into a new file. The `>` arrow sends the words into the file!",
      "`cat treasure/chest.txt` shows what's in the file. (It's not a real cat, it's short for *concatenate*.)",
    ],
    quiz: { question: "What does mkdir make?", options: ["A file", "A folder", "A sandwich"], answer: 1, why: "mkdir makes a directory, which is a folder." },
  },
  {
    id: "tab",
    title: "Typing tricks",
    cards: [
      "Press **Tab** while typing a name, and the computer finishes it for you!",
      "Press the **Up arrow** to bring back the command you typed before.",
      "If a command keeps going and won't stop, press **Ctrl+C** to stop it.",
    ],
    quiz: { question: "Which key finishes a word for you?", options: ["Tab", "Esc", "Delete"], answer: 0, why: "Tab completes names for you." },
  },
  {
    id: "safe",
    title: "Staying safe",
    cards: [
      "Your **password** is a secret. Only share it with your grown-up.",
      "`sudo` gives a command super-powers. Super-powers need care, so ask a grown-up before using it.",
      "If something on the screen asks you to type a command you don't understand, ask a grown-up first.",
    ],
    quiz: { question: "Who can you share your password with?", options: ["Everyone online", "Your grown-up", "A stranger in a game"], answer: 1, why: "Passwords are secret. Only your grown-up should know it." },
  },
  {
    id: "binary",
    title: "Counting like a computer",
    cards: [
      "We count with ten digits, 0 to 9. Computers count with only two: **0** and **1**. That's called **binary**.",
      "In binary: 0 is 0, 1 is 1, **10** is two, **11** is three, **100** is four!",
      "Each 0 or 1 is called a **bit**. Eight bits make a **byte**, enough for one letter.",
    ],
    quiz: { question: "In binary, what is 10?", options: ["Ten", "Two", "One"], answer: 1, why: "In binary, 10 means two." },
  },
];
