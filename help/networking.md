---
title: Networking, ports and the internet
summary: localhost between Windows and Linux, running servers, reaching them from other devices, VPNs, DNS and proxies.
section: WSL and Windows
order: 53
---

# Networking, ports and the internet

Linux in WSL shares your PC's internet connection. Most things just work; this guide is for when
you run servers or something doesn't connect.

## Checking the connection

```bash
ping -c 3 example.com              # can Linux reach the internet? (Ctrl+C stops it)
curl -I https://example.com        # does a website answer? (headers only)
ip addr                            # Linux's network addresses
ip route                           # the way out (default gateway)
cat /etc/resolv.conf               # which DNS server Linux asks
```

## Running a server, opening it from Windows

When a program in Linux listens on a port, Windows reaches it at `http://localhost:<port>`.

```bash
python3 -m http.server 8000        # serves the current folder
```

Then open `http://localhost:8000` in your Windows browser. This is how web apps, Jupyter and
Docker containers work.

If `localhost` doesn't answer from Windows:

- Make the program listen on all addresses, not only Linux's own localhost: Vite `--host`,
  Flask `--host=0.0.0.0`, Django `runserver 0.0.0.0:8000`, Node `HOST=0.0.0.0`.
- Restart WSL (**Restart WSL**, or `wsl --shutdown`) after the PC slept; forwarding sometimes
  stops.
- Try **mirrored networking** (below).

## Which port is in use?

```bash
ss -tlnp                           # every port listening (sudo shows which program)
sudo ss -tlnp | grep 8000          # just one port
```

`Address already in use` means another program has that port: stop it, or pick another port.

## Linux reaching a program on Windows

With the default (NAT) networking, Windows is not `localhost` from Linux; it's the gateway:

```bash
ip route show default | awk '{print $3}'     # Windows' address as Linux sees it
```

The Windows program must listen on that interface, and Windows Firewall must allow it. With
**mirrored** networking, `localhost` works both ways.

## Mirrored networking

In newer WSL, Linux can share Windows' network addresses directly (*mirrored mode*). It makes
localhost work both ways, helps with VPNs and IPv6, and lets other devices reach Linux servers.
In `C:\Users\you\.wslconfig`:

```ini
[wsl2]
networkingMode=mirrored
dnsTunneling=true
```

then `wsl --shutdown`. See [Configuring WSL](wsl-config.md).

## Reaching a Linux server from your phone or another PC

- **Mirrored mode:** use the PC's network address (`ipconfig` in PowerShell, e.g. `192.168.1.20`)
  and the port. Windows Firewall must allow it; the first time, Windows may ask.
- **Default (NAT) mode:** forward the port from Windows to Linux, in an administrator
  PowerShell:

```powershell
netsh interface portproxy add v4tov4 listenport=8000 listenaddress=0.0.0.0 connectport=8000 connectaddress=(wsl hostname -I).Trim().Split(' ')[0]
New-NetFirewallRule -DisplayName "WSL 8000" -Direction Inbound -LocalPort 8000 -Protocol TCP -Action Allow
```

Linux's address changes when WSL restarts, so mirrored mode is simpler.

## VPNs

Corporate VPNs often break WSL's network (no internet, or names not found). In order of
effort:

1. Turn on `dnsTunneling=true` and `networkingMode=mirrored` in `.wslconfig`, then
   `wsl --shutdown`.
2. Set DNS by hand: in `/etc/wsl.conf` set `generateResolvConf=false` under `[network]`, then
   write your VPN's DNS server into `/etc/resolv.conf` (`nameserver 10.x.x.x`).
3. Ask the VPN's administrators: some VPN clients have a WSL setting.

## Proxies

`autoProxy=true` in `.wslconfig` passes Windows' proxy to Linux. Otherwise, for one terminal:

```bash
export http_proxy=http://proxy.example.com:8080 https_proxy=$http_proxy no_proxy=localhost,127.0.0.1
```

`apt` needs its own setting: a file `/etc/apt/apt.conf.d/95proxy` with
`Acquire::http::Proxy "http://proxy.example.com:8080";`.

## Downloading from the terminal

```bash
curl -LO https://example.com/file.zip      # save with its own name (-L follows redirects)
wget https://example.com/file.zip          # the same with wget
curl https://api.example.com/data | less   # look at what an address returns
```

## Common problems

| Problem | Fix |
| --- | --- |
| `Temporary failure in name resolution` | DNS: restart WSL; with a VPN see above; or set `nameserver 1.1.1.1` as shown. |
| No internet at all in Linux | Restart WSL; try with the VPN off; `wsl --update`. |
| Works in Linux, not from Windows | Listen on `0.0.0.0`; try mirrored mode. |
| Slow downloads only in Linux | Try `networkingMode=mirrored`; some VPNs and security software slow WSL's NAT. |
