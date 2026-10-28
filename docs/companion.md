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
