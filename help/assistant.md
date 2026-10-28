---
title: Ask Pip, and AI apps
summary: Pengi's own AI on your PC, and adding Pengi to Claude or Codex instead.
section: Start here
order: 4
---

# Ask Pip, and AI apps

Pengi can answer questions in your own words about Pengi, Linux, WSL, Windows and Docker.
There are two ways: **Ask Pip**, Pengi's own assistant (Pip, with an AI that runs on your PC), or an AI app you already use.

## Ask Pip

**Ask Pip** (on the left) sets it up: pick a size and **Install Pip's AI**. It downloads
Ollama, the free program that runs AI models, and a model (Qwen 3.5), about 5 GB in all.

- **Private:** it runs on this PC. Nothing you ask leaves it, and it works without internet.
- **It knows your PC:** with each question it reads the guides that match and sees your
  systems and containers, so its answers use your system names and the app's own buttons.
- **Failed tasks:** a card that did not finish has **Ask Pip**, which sends it the
  error and the end of the log.
- **Sizes:** *Small* for PCs with little memory, *Standard* for most PCs, *Large* for graphics
  cards with 12 GB or more. Change it, or remove it, in **Settings → Ask Pip (the AI)**.
- The first answer after opening the app takes a few seconds longer while the model loads.

Like any AI it can be wrong. Read commands before running them, especially ones that delete
things.

## Claude or Codex instead

If you pay for Claude or ChatGPT, you can skip the local assistant. **Settings → AI apps you
already use** adds an Pengi *skill* to:

- **Claude Code**, in `%USERPROFILE%\.claude\skills\easywsl`
- **Codex**, in `%USERPROFILE%\.agents\skills\easywsl`
- **Claude on the web or Claude Desktop:** **Save .zip**, then upload it in Claude under
  Settings → Capabilities → Skills.

A skill is a few text files that teach the AI how Pengi works, with these guides. Ask about
Pengi and it uses the skill by itself, or call it: `/pengi` in Claude Code, `$pengi` in
Codex. Claude Code and Codex can also look at your systems, logs and Docker with commands, but
only as you allow them. Pengi keeps the skill up to date when it updates.
