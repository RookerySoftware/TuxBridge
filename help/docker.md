---
title: Docker and the app gallery
summary: What containers are, the app gallery, and using docker from a terminal.
section: Docker
order: 30
---

# Docker and the app gallery

**Docker** runs programs in **containers**. A container carries a program with everything it
needs (its own small Linux, libraries, settings), so it runs the same everywhere and installs in
one step. Containers are kept apart from each other and from your systems: removing one leaves
nothing behind.

An **image** is the packed program; a **container** is one running copy of it. Images come
from Docker Hub and similar collections.

## Turning it on

On the **Docker** page, **Turn on Docker** sets up the free Docker Engine in a small hidden
Linux system of its own (2 to 4 minutes). Docker keeps running after Pengi closes, and starts
again when Pengi opens (Settings → Docker).

## The app gallery

**App gallery** has ready-made apps: a media server, your own cloud, AI chat, a password
manager, databases. **Install** asks only what the app needs (a password, a folder), downloads
it and starts it. Its card in **Containers** then has **Open**, which opens it in your browser
at an address like `http://localhost:8096`.

- **Update** (in a container's **⋯** menu) downloads the newest version of its image and starts
  it again with the same settings and data. If the new version does not start, the old one is
  put back. Containers from a Compose file are updated with Compose.
- An app's data is kept in Docker **volumes**. Removing the app keeps its data, so installing it
  again picks up where it left off.
- Apps are reachable from this PC only. Reaching them from your phone needs more set-up.
- **Disk use and clean-up** (the Docker Engine card's menu) frees space from images nothing
  uses. Apps' data is never removed by it.

## Docker in a terminal

Every Docker guide works in PowerShell or Windows Terminal:

```
docker run hello-world
docker run -d -p 8080:80 --name web nginx
docker ps
docker compose up -d
```

`-p 8080:80` makes the container's port 80 reachable at `http://localhost:8080`.

To use `docker` inside your Linux systems too, choose **Set up** for them under **docker in
your Linux systems** on the Docker page. All of them then share the same containers.

Sharing a folder with a container (`-v`): the engine runs in Linux, so name Windows folders
the Linux way, from PowerShell too: `-v /mnt/c/Users/you/site:/usr/share/nginx/html`.
Windows-style paths (`C:\Users\...`, and the `./site` of a compose file run from PowerShell)
are not translated yet, and neither are folders inside a Linux system's home. Named volumes
(`-v mydata:/data`) always work.

## Docker Desktop

If Docker Desktop is installed, its containers show on the Docker page too, marked
**Docker Desktop**. The `docker` command keeps using Docker Desktop; `docker context use
easywsl` switches it to Pengi's engine, and `docker context use desktop-linux` back.
