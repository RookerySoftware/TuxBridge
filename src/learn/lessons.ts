// Guided lessons for the Learn page: short, step-by-step walk-throughs the learner follows in a
// real system, with the exact commands to type. Unlike the Help guides (which you read), a lesson
// is a sequence of steps with a "done" tick, kept in the browser. Kids' lessons are gentler and
// marked kids: true.

export interface Step {
  title: string;
  /** Markdown. Commands in it run inside the chosen Linux system unless the step says otherwise. */
  body: string;
  /** Shown as a copyable command block under the step. */
  command?: string;
}

export interface Lesson {
  id: string;
  title: string;
  summary: string;
  /** Who it suits: catalog ids it is written for, or "any". */
  systems: string[];
  minutes: number;
  kids?: boolean;
  steps: Step[];
}

export const lessons: Lesson[] = [
  {
    id: "first-commands",
    title: "Your first commands",
    summary: "Move around, look at files, and get comfortable typing in the terminal.",
    systems: ["any"],
    minutes: 10,
    steps: [
      {
        title: "Open a terminal",
        body: "On **My systems**, open your system and choose **Terminal** (or the arrow next to **Open desktop**). A window appears with a line ending in `$`. That is the prompt: Linux is waiting for you.",
      },
      {
        title: "Who am I, and where am I?",
        body: "Type each line and press Enter. `whoami` shows your user name; `pwd` shows the folder you are in (your home folder, `/home/you`).",
        command: "whoami\npwd",
      },
      {
        title: "Look around",
        body: "`ls` lists what is in the folder. Add `-la` to see hidden files (the ones starting with a dot) and details like size and date.",
        command: "ls -la",
      },
      {
        title: "Make a folder and go into it",
        body: "`mkdir` makes a folder; `cd` goes into it. `cd ..` goes back up one level.",
        command: "mkdir practice\ncd practice\npwd",
      },
      {
        title: "Create and read a file",
        body: "`echo` prints text; the `>` sends it into a file instead of the screen. `cat` shows a file's contents.",
        command: "echo \"hello from Linux\" > note.txt\ncat note.txt",
      },
      {
        title: "Tidy up",
        body: "`rm` deletes a file — **for good**, there is no Recycle Bin. Go home with `cd` on its own.\n\nThat is the core of it: `ls`, `cd`, `cat`, `mkdir`, `echo`, `rm`. You now know your way around.",
        command: "rm note.txt\ncd ..\nrmdir practice",
      },
    ],
  },
  {
    id: "install-software",
    title: "Installing software",
    summary: "Use the package manager to find, install and remove programs.",
    systems: ["any"],
    minutes: 10,
    steps: [
      {
        title: "Update first",
        body: "Before installing anything, refresh the system. Use the first line on Ubuntu, Mint, Debian, Kali and Parrot; the second on Arch, CachyOS and Omarchy. (Or just use **Update** in the system's ⋯ menu.)",
        command: "sudo apt update && sudo apt upgrade     # apt systems\nsudo pacman -Syu                        # pacman systems",
      },
      {
        title: "Install a small program",
        body: "`htop` is a live view of what your system is doing. Install it with your system's package manager.",
        command: "sudo apt install htop     # apt systems\nsudo pacman -S htop       # pacman systems",
      },
      {
        title: "Run it",
        body: "Type `htop` and press Enter. Press `q` to quit. You just installed and ran your first Linux program.",
        command: "htop",
      },
      {
        title: "Find programs",
        body: "Search the system's collection by keyword.",
        command: "apt search markdown      # apt systems\npacman -Ss markdown      # pacman systems",
      },
      {
        title: "Remove a program",
        body: "Remove what you no longer want. Programs come from a trusted collection, so this is safe and clean.",
        command: "sudo apt remove htop      # apt systems\nsudo pacman -Rs htop      # pacman systems",
      },
    ],
  },
  {
    id: "files-bridge",
    title: "Files between Windows and Linux",
    summary: "Reach your Windows files from Linux, and your Linux files from Windows.",
    systems: ["any"],
    minutes: 8,
    steps: [
      {
        title: "Find your Windows drive from Linux",
        body: "Windows drives live under `/mnt`. Your `C:` drive is `/mnt/c`. List your Windows user folder:",
        command: "ls /mnt/c/Users",
      },
      {
        title: "Copy a Windows file into Linux",
        body: "Replace `you` with your Windows user name. This copies a file from your Downloads into your Linux home folder. Working on files inside Linux is much faster than on `/mnt/c`.",
        command: "cp /mnt/c/Users/you/Downloads/somefile.txt ~/",
      },
      {
        title: "Open the Linux folder in Windows",
        body: "From Linux, this opens File Explorer at the folder you are in. You can also use **Files** on the system's card.",
        command: "explorer.exe .",
      },
      {
        title: "Remember",
        body: "- Keep **projects you work on in Linux** inside Linux (`~`) for speed.\n- Keep **documents and photos** on Windows, where they are backed up.\n- **Removing a system deletes its files** — copy anything important to Windows first.",
      },
    ],
  },
  {
    id: "ubuntu-tour",
    title: "A tour of Ubuntu",
    summary: "Get to know the GNOME desktop and set Ubuntu up the way you like.",
    systems: ["ubuntu", "mint", "debian"],
    minutes: 12,
    steps: [
      {
        title: "Open the desktop",
        body: "On the system's card, choose **Open desktop**. It opens in a window. **Ctrl+Alt+Enter** switches to full screen and back.",
      },
      {
        title: "Find your programs",
        body: "Press the **Windows key** (or click **Activities**, top left). Start typing a program's name and press Enter to open it. This is how you launch anything.",
      },
      {
        title: "Install a program with a window",
        body: "In a terminal, install the GIMP image editor, then find it in Activities. Programs with a window appear there automatically.",
        command: "sudo apt install gimp",
      },
      {
        title: "Make it yours",
        body: "Open **Settings** (top-right menu, or search for it). Try **Appearance** for a dark theme and a background. **Keyboard** has shortcuts.",
      },
      {
        title: "What does not work here",
        body: "Wi-Fi, Bluetooth, power and extra screens show in Settings but do nothing — Windows handles the hardware. That is normal for Linux in WSL.",
      },
    ],
  },
  {
    id: "kali-tour",
    title: "Getting started with Kali",
    summary: "Set up Kali responsibly, add tool sets, and find safe places to practise.",
    systems: ["kali"],
    minutes: 12,
    steps: [
      {
        title: "The one rule",
        body: "Kali's tools are powerful. **Only ever test systems you own, or have written permission to test.** Testing others' systems is illegal in most places. Everything below is about learning and defending.",
      },
      {
        title: "Update Kali",
        body: "Kali updates continuously and uses `full-upgrade` (not plain `upgrade`).",
        command: "sudo apt update && sudo apt full-upgrade",
      },
      {
        title: "Add a set of tools",
        body: "A fresh Kali is small. Install the headless tool set (command-line tools, a few GB), or browse the groups with `apt search kali-tools`.",
        command: "sudo apt install kali-linux-headless",
      },
      {
        title: "Practise legally",
        body: "Learn on systems made for it: **OverTheWire** (wargames by SSH), **TryHackMe** and **Hack The Box** (guided labs), and deliberately vulnerable practice machines. These are legal, free to start, and built to be attacked.",
      },
      {
        title: "Try the Security Basics quest",
        body: "On the **Challenges** tab of this page, run the **Security Basics** quest in Kali. It is a safe, defensive puzzle set — reading logs, decoding data, checking files — the everyday skills of an analyst, on harmless practice files.",
      },
      {
        title: "What WSL cannot do",
        body: "Some tools need direct hardware: putting a Wi-Fi adapter into monitor mode, or raw USB access. WSL does not allow that. For those, use a separate PC or a virtual machine.",
      },
    ],
  },
  {
    id: "parrot-tour",
    title: "Getting started with Parrot",
    summary: "Set up Parrot, learn its KDE desktop, and add the tool groups you want.",
    systems: ["parrot"],
    minutes: 12,
    steps: [
      {
        title: "The one rule",
        body: "Like Kali, Parrot is for security work. **Only test systems you own or have written permission to test.** The skills are for understanding and defending.",
      },
      {
        title: "Update Parrot",
        body: "Parrot is Debian-based and recommends `full-upgrade`.",
        command: "sudo apt update && sudo apt full-upgrade",
      },
      {
        title: "Meet the desktop",
        body: "Open the desktop from the system's card. Parrot uses **KDE Plasma** with its own dark theme. The Parrot logo at the top-left opens the menu; next to it are System Settings, the file manager, Firefox and the terminal.",
      },
      {
        title: "Add tool groups",
        body: "Parrot keeps tools in groups, so you install only what you need. See them all with `apt search parrot-tools`. For example, the forensics group:",
        command: "sudo apt install parrot-tools-forensics",
      },
      {
        title: "Parrot also makes a fine dev machine",
        body: "Beyond security, Parrot has strong development tool groups. Try `parrot-meta-devel` for editors and compilers, and the privacy tools in `parrot-meta-privacy`.",
        command: "sudo apt install parrot-meta-devel",
      },
      {
        title: "Try the Security Basics quest",
        body: "On the **Challenges** tab, run the **Security Basics** quest in Parrot — a safe, defensive set of puzzles that teaches how analysts read evidence.",
      },
    ],
  },
  {
    id: "kids-explorer",
    title: "Be a computer explorer",
    summary: "A gentle first adventure in the terminal, with treasures to find.",
    systems: ["any"],
    minutes: 15,
    kids: true,
    steps: [
      {
        title: "What is this black window?",
        body: "It is called a **terminal**. Instead of clicking, you *type* what you want the computer to do, and it does it. Computer experts use it every day. You are about to learn it too!",
      },
      {
        title: "Say hello",
        body: "Type this and press Enter. The computer will say hello back with your name!",
        command: "echo Hello, I am an explorer",
      },
      {
        title: "Look around you",
        body: "Type `ls` (that is a small L and a small S). It shows what is in the folder you are standing in, like looking around a room.",
        command: "ls",
      },
      {
        title: "Start the treasure hunt",
        body: "Go to the **Challenges** tab on this page and start the **Treasure Hunt**. It fills a folder with treasures to find. Each treasure is a secret word like `STAR{...}` — type the words you find into the page to collect them. Have fun, explorer!",
      },
    ],
  },
];
