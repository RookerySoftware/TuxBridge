# The assistant

An optional AI on the PC that answers questions about EasyWSL, Linux, WSL, Windows and Docker
(the **Assistant** page), and an EasyWSL skill for people who use Claude or Codex instead
(Settings → AI apps you already use).

## Running the model (src-tauri/src/ai.rs)

- **Ollama**, EasyWSL's own copy: the official `ollama-windows-amd64.zip` from Ollama's newest
  GitHub release (1.6 GB, with CUDA; its SHA-256 is checked against the release's
  `sha256sum.txt`), unpacked with Windows' `tar` into `<systems folder>\easywsl-ai\ollama`.
  Models go to `easywsl-ai\models`. It is apart from any Ollama the person installed.
- The server listens on `127.0.0.1:11435` only, with `OLLAMA_NO_CLOUD=1`, a 16K context and a
  10-minute keep-alive (the model leaves the graphics memory when idle). It starts on the
  first question (the Assistant page warms it up when it opens) and is in a Windows job object
  that ends it with the app, even when the app crashes.
- **Models**: Qwen 3.5 (Apache 2.0) at q4_K_M, with thinking turned off for quick answers.
  Small 2B (1.9 GB), Standard 4B (3.3 GB), Large 9B (6.6 GB). Recommended from the hardware
  (`nvidia-smi`, installed memory): Large only with 12 GB of graphics memory, since on an
  8 GB RTX 4060 three of its layers run on the processor and it writes about 2 tokens a
  second, while the 4B fits whole and writes about 50.
- Measured on an RTX 4060: the first load after a download takes up to a minute (Windows
  Defender checks the new files once); after that, starting the server and loading the model
  takes about 8 s, and typical answers 3 to 15 s.

## What makes it know EasyWSL (src/assistant)

Every question is sent with instructions built in the window (`context.ts`):

1. `knowledge.md`: what EasyWSL is, every page and button, common tasks with the simplest way
   to do them, facts that are easy to get wrong (folder sharing with Docker, clipboard), and
   how to answer (one way, commands marked bash or PowerShell, warnings only for destructive
   things, never invent features).
2. **This PC right now**: the systems with their user, desktops and state, Docker's state and
   the containers, from the same calls the pages use.
3. **The guides that match**: `help/*.md` split at their `##` headings and ranked with BM25 on
   the last two questions (a follow-up like "and on Arch?" keeps its subject); up to about
   9,000 characters.

The conversation (the last 12 messages) is kept in localStorage. In a browser preview,
`?realai` makes the window talk to a running server on port 11435 directly, to try prompts
without building the app.

## The skill (src-tauri/src/skills.rs, src-tauri/skill/SKILL.md)

A folder with `SKILL.md` (when to use it, how EasyWSL is built, read-only commands to look at
the PC, what to ask before doing) and `references/` (the guides and the app part of
`knowledge.md`). Installed to `~/.claude/skills/easywsl` (Claude Code) and
`~/.agents/skills/easywsl` (Codex), or saved as `easywsl-skill.zip` in Downloads for claude.ai
and Claude Desktop. EasyWSL rewrites installed copies when it starts, and never touches a
folder named easywsl that it did not write (SKILL.md carries a mark).
