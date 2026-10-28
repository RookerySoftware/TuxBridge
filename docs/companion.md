# Pip, the companion

Pip is a penguin in the bottom-right corner of the window (`src/companion/`). It is **off by
default** for everyone (Settings → **Show Pip**) and **always on in Kids mode**.

## What it does

- **Page tips** (`pageTips` in `content.ts`): the first visit to a page opens the bubble with its
  first tip; later visits only put a dot on Pip.
- **Pointing at things**: hovering a button, link or field for 0.7 s shows what it does, looked up
  by the start of its label in `explanations` (longest match wins). Settings → **Explain buttons
  when I point at them** turns it off.
- **Something else** (the shuffle button): a suggestion based on the systems on the PC, a tip,
  or a fun fact (`funFacts`, `kidsFacts`).
- **The welcome tour** (`appTour`): on first launch, a skippable tour that goes page by page and
  spotlights each sidebar item (`data-tour="nav:<page>"`). Its last step asks whether Pip should
  stay. Settings → **Show the welcome tour again**.
- **Show me around** (`systemTour`): the install form's toggle (on by default) stores the
  system's name (`requestShowAround`). Once that system exists and isn't busy, Pip walks
  through it on My systems, spotlighting its card (`data-tour="distro:<name>"`), with buttons to
  open its terminal, its files and its guides. The text comes from per-catalogue `facts`.
- **Kids mode**: Pip wears a cap and teaches — the book button lists `kidsLessons`, each a few
  cards and a quiz; a right answer earns a star.

State lives in `localStorage` (`store.ts`, `prefs.ts`); nothing leaves the PC.

## The artwork

`Penguin.tsx` is an original SVG drawn for EasyWSL: a round navy head, a feather tuft (or a
graduation cap in Kids mode), cheeks and a few moods. It is **not Tux** and not based on
Larry Ewing's Tux image, so the project owns it outright and can use it in a paid product, on the
website, or as a trademark. Keep it that way: don't paste in Tux or other penguin art.

## Pip everywhere

- **The app icon** (taskbar, installer, tray) is generated from `src-tauri/icons/pip.svg`:
  `pnpm tauri icon src-tauri/icons/pip.svg -o src-tauri/icons` (then delete the `android` and
  `ios` folders it adds).
- **The tray:** closing the window hides EasyWSL to the notification area. A left click on Pip
  opens it; a right click opens EasyWSL's own tray menu (`src/tray/TrayMenu.tsx`, a small
  borderless window, because native menus can't tell left from right clicks on an item): the
  five most recent systems (`recentSystems` in `api.ts`; click opens the desktop, or a terminal
  when there's none; right-click offers open desktop, open terminal, stop), Open EasyWSL,
  Settings and Quit. A second launch shows the running copy (single-instance plugin).
- **Ask Pip:** the local AI speaks as Pip (its instructions in `src/assistant/context.ts`).
