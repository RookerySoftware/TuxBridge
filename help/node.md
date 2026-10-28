---
title: Node.js and JavaScript
summary: Install Node.js with a version manager, npm, pnpm and bun, and run web apps you open from Windows.
section: Programming
order: 33
---

# Node.js and JavaScript

The Node.js in a system's package manager is often old. A **version manager** gets current
versions, installs into your home folder (no `sudo`), and switches between versions per project.

## Install Node.js

**fnm** (fast) or **nvm** (the classic) — pick one.

```bash
# fnm
curl -fsSL https://fnm.vercel.app/install | bash
# open a new terminal, then:
fnm install --lts
fnm use lts-latest
node --version
```

```bash
# nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/master/install.sh | bash
# open a new terminal, then:
nvm install --lts
node --version
```

Both read a `.nvmrc` or `.node-version` file in a project to pick its version
(`fnm use` / `nvm use`).

From the package manager instead (simpler, older):
`sudo apt install nodejs npm` or `sudo pacman -S nodejs npm`.

## npm, pnpm, yarn, bun

```bash
npm install                  # a project's packages (from package.json)
npm run dev                  # a script from package.json
npx create-vite@latest       # run a package's command without installing it
```

**pnpm** and **yarn** come with Node through *corepack*:

```bash
corepack enable
pnpm install
```

**bun** is a fast all-in-one runtime and package manager:

```bash
curl -fsSL https://bun.sh/install | bash
```

Avoid `sudo npm install -g`: with fnm or nvm, global packages install into your home folder
without sudo.

## Opening your app from Windows

```bash
npm run dev
#  ➜  Local:   http://localhost:5173/
```

Open that address in your **Windows** browser: WSL forwards `localhost` to Windows. If a dev
server only listens on `127.0.0.1` inside Linux and Windows can't reach it, start it with
`--host` (Vite) or set `HOST=0.0.0.0`; see [Networking](networking.md).

## Speed: keep projects in Linux

`node_modules` holds thousands of small files. Under `/mnt/c` installs and builds are many times
slower and file watching (hot reload) may not notice changes. Keep projects in `~/projects`.

## Common problems

| Problem | Fix |
| --- | --- |
| `node: command not found` after installing fnm/nvm | Open a new terminal (they add themselves to `~/.bashrc`). |
| `EACCES` on `npm install -g` | You're using the system Node: use fnm/nvm, or set a user prefix (`npm config set prefix ~/.local`). |
| Hot reload doesn't notice changes | The project is under `/mnt/c`: move it into Linux. |
| `ENOSPC: System limit for number of file watchers reached` | `echo fs.inotify.max_user_watches=524288 \| sudo tee /etc/sysctl.d/90-watches.conf && sudo sysctl --system` |
| Windows' `node` runs instead of Linux's | `which node` should be under `/home`. Install Node in Linux, or stop adding Windows' PATH (see [Configuring WSL](wsl-config.md)). |
