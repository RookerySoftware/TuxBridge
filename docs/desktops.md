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

## Not working yet: Wayland-only desktops

KDE Plasma 6.8 (Arch, CachyOS) and current GNOME (Ubuntu 26.04) no longer have an X11
session, so they cannot run on Xvnc. Tried on 2026-10-26 on CachyOS with Plasma 6.8.1:

| Approach | Result |
| --- | --- |
| KWin `--virtual` + KRdp (`--plasma`), Windows' Remote Desktop | KRdp needs a TLS certificate (its promised temporary one is not made; exits 255 silently). With one it listens, but KWin's virtual backend segfaulted (in libkwin) around virtual-monitor creation and removal. |
| Windows' Remote Desktop (`mstsc`) as viewer | A `.rdp` file now triggers Windows 11's "could harm your computer / unknown publisher" warning on every launch; KRdp only accepts NLA. The NLA attempt closed without a connection. `mstsc /v:` or hosting the RDP ActiveX control would avoid the file warning. |
| KWin `--virtual` + `krfb-virtualmonitor` (VNC) | Listens (on all interfaces, which would need restricting); crashed in libvncserver when the viewer connected. |
| KWin nested in a WSLg window | KWin requires `wp_single_pixel_buffer_manager_v1`, which WSLg's compositor lacks (the same wall Hyprland hit). |
| KWin nested in headless cage (pixman) + wayvnc | cage and wayvnc work at 1600x900; KWin exits at once without output. |
| KWin nested in hyprland-wsl's headless Hyprland + wayvnc | With GPU rendering KWin aborts (`invalid arguments for wl_surface.attach`); with `KWIN_COMPOSE=Q` it and plasmashell run but no window appears. |

Not tried yet: GNOME's headless mode (`gnome-shell --headless` with gnome-remote-desktop),
which is designed for exactly this; Plasma under a nested compositor that offers dmabuf with a
real GPU path; following KWin's crash with a debugger.

Until one of these works, CachyOS offers Hyprland or a terminal, and Ubuntu gets Xfce.
