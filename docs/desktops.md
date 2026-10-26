# Desktops

How a distro's own desktop runs under WSL and shows on Windows, and what was tried for the
ones that do not run yet.

## What works: X11 desktops on TigerVNC's X server

`src-tauri/linux/desktop-setup.sh` installs the desktop and TigerVNC's X server (Xvnc);
`desktop-start.sh` runs the session on it, in a private mount namespace (WSLg keeps
`/tmp/.X11-unix` read-only), and EasyWSL's viewer (the same TigerVNC viewer as for Hyprland)
shows it.

| Desktop | Distros | State |
| --- | --- | --- |
| Xfce | Kali (with Kali's look), Debian, Ubuntu (as in Xubuntu) | Kali tested end to end |
| Cinnamon | Linux Mint | Tested end to end |

Details that matter:

- Docker-imported distros (Mint, CachyOS) run without systemd, so there is no system message
  bus; `desktop-start.sh` starts one. logind and ConsoleKit stay missing (Cinnamon warns, and
  power and session menus have nothing to talk to).
- Xfce keeps its wallpaper per monitor name, and the distros name physical monitors; the
  virtual screen is `VNC-0`, so the distro's default image is set for it on first start.
- Kali's panel layout uses plugins that `xfce4` alone does not install (cpugraph, genmon,
  power manager); without them a dialog asks to remove them.

## GNOME: headless, in EasyWSL's own viewer

Ubuntu's GNOME runs headless (`gnome-shell --headless --virtual-monitor WxH`) and shows in
EasyWSL's own viewer, not over VNC: `display/agent` (Rust, in Linux) takes the frames from
Mutter's ScreenCast interface (PipeWire) and injects input through its RemoteDesktop interface;
`display/viewer` (Rust, on Windows) is a native window that starts the agent through wsl.exe
and draws what it sends. wsl.exe carries both directions over WSL's own Hyper-V socket (about
650 MB/s measured), so there is no network, no port and nothing to configure.

- Frames: changed 64x64 tiles, LZ4-compressed; the pointer comes as metadata and the viewer
  shows it as its own Windows pointer, so moving the mouse needs no round trip.
- GPU: `gnome-start.sh` loads vgem so Mutter finds a render node; Mesa's Direct3D 12 driver then
  renders on the PC's GPU ("Created gbm renderer for /dev/dri/renderD128").
- About 46 screen updates a second with something moving on every frame (pointer drawn in the
  frames, before the metadata pointer); GNOME Terminal throttles its own redraws to about 20.
- The agent is built on Ubuntu 26.04 and links its C library, so it runs there and on newer
  releases; built on an older release it would run on more.
- Known: GNOME shows a "Screen Lock disabled" notice once per session and its orange "screen is
  being shared" icon while the agent records.

## Not working yet: KDE Plasma (Wayland-only)

KDE Plasma 6.8 (Arch, CachyOS) no longer has an X11 session, so it cannot run on Xvnc, and its
own remote paths crashed. Tried on 2026-10-26 on CachyOS with Plasma 6.8.1 (before GNOME's
headless route above existed; KDE has no equivalent of GNOME's ScreenCast interface that worked):

| Approach | Result |
| --- | --- |
| KWin `--virtual` + KRdp (`--plasma`), Windows' Remote Desktop | KRdp needs a TLS certificate (its promised temporary one is not made; exits 255 silently). With one it listens, but KWin's virtual backend segfaulted (in libkwin) around virtual-monitor creation and removal. |
| Windows' Remote Desktop (`mstsc`) as viewer | A `.rdp` file now triggers Windows 11's "could harm your computer / unknown publisher" warning on every launch; KRdp only accepts NLA. The NLA attempt closed without a connection. `mstsc /v:` or hosting the RDP ActiveX control would avoid the file warning. |
| KWin `--virtual` + `krfb-virtualmonitor` (VNC) | Listens (on all interfaces, which would need restricting); crashed in libvncserver when the viewer connected. |
| KWin nested in a WSLg window | KWin requires `wp_single_pixel_buffer_manager_v1`, which WSLg's compositor lacks (the same wall Hyprland hit). |
| KWin nested in headless cage (pixman) + wayvnc | cage and wayvnc work at 1600x900; KWin exits at once without output. |
| KWin nested in hyprland-wsl's headless Hyprland + wayvnc | With GPU rendering KWin aborts (`invalid arguments for wl_surface.attach`); with `KWIN_COMPOSE=Q` it and plasmashell run but no window appears. |

Not tried yet for KDE: Plasma under a nested compositor that offers dmabuf with a
real GPU path; following KWin's crash with a debugger.

Until one of these works, CachyOS offers Hyprland or a terminal.
