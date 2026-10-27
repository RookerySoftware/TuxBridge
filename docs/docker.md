# Docker

The Docker page turns on Docker with one button, manages containers, installs ready-made apps,
and makes `docker` work in PowerShell and in the user's Linux systems. Everything here was
tested on 2026-10-26 (Docker Engine 29.9.2, Alpine 3.24, WSL 2.6).

## The engine

`containers.rs` and `linux/docker-engine.sh`.

- **A system of its own.** The free Docker Engine runs in `easywsl-docker`, a small Alpine Linux
  system (Alpine's official minimal root filesystem, checked against the SHA-256 in Alpine's
  release list, then `apk add docker`). It is hidden from My systems. Its disk goes to the
  folder for new systems (Settings), since images and app data can grow large.
- **Running.** The app starts `wsl.exe -d easywsl-docker -e easywsl-docker run` as a detached
  process (out of the app's job object), so Docker keeps running when the app closes; that
  wsl.exe also keeps WSL from stopping the system as idle. `easywsl-docker stop` stops dockerd,
  which stops the containers first. "Start Docker when EasyWSL opens" (Settings, on by default)
  starts it again after Windows restarts.
- **Reaching it from Windows.** dockerd listens on its Unix socket and on `127.0.0.1:2376`,
  which WSL forwards to Windows' localhost. That port requires TLS client certificates
  (`--tlsverify`): when Docker is turned on, the app makes a certificate authority with rcgen,
  signs a server certificate (localhost, 127.0.0.1) and a client certificate, and throws the
  authority's key away. The client's files stay in `%LOCALAPPDATA%\EasyWSL\docker\tls`, readable
  by this Windows account only, so other accounts and programs without them cannot use the
  engine (which amounts to root on it). A plain request is refused ("Client sent an HTTP
  request to an HTTPS server").
- **Published ports.** `-p 8080:80` binds 0.0.0.0:8080 in WSL's VM; WSL forwards it to Windows'
  localhost:8080, which is what "Open" uses.

## The docker command

- `docker.exe` is the static Windows build from download.docker.com (the newest in its
  listing; unsigned, so HTTPS from Docker's server is the trust, and the log records its
  SHA-256), unpacked with Windows' own `tar.exe` into `%LOCALAPPDATA%\EasyWSL\bin`, which is
  added to the user's PATH (HKCU, with a WM_SETTINGCHANGE broadcast) unless Docker Desktop is
  installed.
- Docker Compose v2 is put in `%USERPROFILE%\.docker\cli-plugins` unless one is there already
  (GitHub's latest release, checked against its `.sha256`).
- A context named `easywsl` holds the address and certificates; it becomes the current context
  unless Docker Desktop is installed (then `docker context use easywsl` switches).
- The app's own calls pass the address and certificates on the command line and use
  `%LOCALAPPDATA%\EasyWSL\docker\config` as `DOCKER_CONFIG`, so the user's settings (a
  credential helper of an uninstalled Docker Desktop, say) cannot break them.

## In the user's Linux systems

"docker in your Linux systems" on the Docker page runs `linux/docker-client.sh` in a system:
static Linux builds of docker and docker compose (downloaded once, same version) go to
`/usr/local/lib`, the client certificate to `/etc/easywsl/docker-client` (the key owned by the
system's user, mode 600), and `/usr/local/bin/docker` becomes a small script setting
`DOCKER_HOST`, `DOCKER_TLS_VERIFY` and `DOCKER_CERT_PATH` unless `DOCKER_HOST` is set already.
A system with a docker of its own is left alone.

Limitation: bind mounts are resolved in the engine's system. From Windows, `C:\...` paths (and
the absolute Windows paths compose makes of `./dir`) are not translated as Docker Desktop
does; `/mnt/c/...` works. Windows folders (`/mnt/c/...`)
and named volumes work from every system; a folder in a Linux system's home does not
(`docker run -v ~/project:/app` mounts an empty folder). Docker Desktop solves this with an API
proxy that rewrites such paths to a shared mount under `/mnt/wsl`; doing the same is the next
step here.

## Containers

`docker ps -a --format '{{json .}}'` on EasyWSL's engine and, when its pipe
(`\\.\pipe\dockerDesktopLinuxEngine`) exists, on Docker Desktop's. Start, stop, restart,
remove (`rm -f`: named volumes stay), the last 400 lines of the log, and "Open" for published
TCP ports. "Disk use and clean-up" shows `docker system df` and runs `docker system prune -f`
(optionally `-a`); volumes are never pruned.

## App gallery

`apps.rs`: 12 apps (Jellyfin, Nextcloud, Home Assistant, Open WebUI, Ollama, Uptime Kuma,
Stirling PDF, Vaultwarden, code-server, PostgreSQL, MariaDB, Redis), every image checked with
`docker manifest inspect`. An app asks only what it needs (a password, a Windows folder that
is mounted through `/mnt/c`). Installing pulls the image (its progress in the job card), then
`docker run -d --restart unless-stopped` with the container named after the app, labelled
`easywsl.app=<id>`, its data in volumes named `easywsl-<id>-<name>`, and the first free port
from the app's usual one. Tested: Uptime Kuma, PostgreSQL (password), Jellyfin (Windows
folder).

Not included: ad blocking (Pi-hole, AdGuard Home). WSL forwards TCP to localhost only, so a DNS
server in WSL cannot serve the network or even this PC's resolver.
