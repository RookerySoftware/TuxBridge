# Desktops

How a distro's own desktop runs under WSL and shows on Windows, and what was tried for the
ones that do not run yet.

## How every desktop is shown: EasyWSL's own viewer and agent

No VNC and no ports. `display/viewer` (Rust, on Windows) is a native window that starts the
session through wsl.exe; `display/agent` (`easywsl-agent`, Rust, in Linux) runs next to the
session, takes its frames and injects the viewer's input. wsl.exe carries both directions over
WSL's own Hyper-V socket (about 650 MB/s measured), so there is nothing to configure.

- Frames: changed 64x64 tiles, LZ4-compressed; the viewer gets the newest frame and skips any it
  could not keep up with. Real pixels, one desktop pixel per screen pixel; the desktop scales
  itself as Windows does (GNOME and Cinnamon their text, Xfce its DPI, Hyprland everything).
- Closing the window closes the agent's input; the agent then ends and the session with it.
  The viewer kills wsl.exe only if that takes over five seconds, because WSL kills the Linux
  side outright then. The scripts also keep a watcher for that case.
- The agent is built on Ubuntu 24.04 (`scripts/build-display.ps1`, distro `easywsl-build`) and
  needs glibc 2.39, so it runs on Linux Mint 22 and anything newer. It links libpipewire and
  libxkbcommon, which `desktop-setup.sh` installs.

| Desktop | Started by | Frames | Input | Pointer |
| --- | --- | --- | --- | --- |
| GNOME (Ubuntu) | `gnome-start.sh` (`gnome-shell --headless`) | Mutter ScreenCast (PipeWire) | Mutter RemoteDesktop | Metadata, drawn by the viewer |
| Xfce (Kali, Debian, Ubuntu), Cinnamon (Mint) | `desktop-start.sh` on Xvfb | X Damage + GetImage of the changed area | XTest | XFixes, drawn by the viewer |
| Hyprland (Omarchy and others) | `hyprland-wsl --agent` | wlr-screencopy (waits for damage) | Virtual pointer and keyboard (US layout; `EASYWSL_KEYBOARD_LAYOUT`) | Drawn by Hyprland |

All tested end to end from the app: Kali Xfce, Mint Cinnamon, Omarchy, Ubuntu GNOME.

### Size, frame rate and full screen

Settings has a resolution (Automatic: the screen's size in full screen, the window's in a
window; or the screen's size, 4K, 1440p, 1080p, scaled to the window) and a frame-rate limit
(60 to 240), passed as `--fps` to the agent and, where the desktop has one, as its refresh rate
(GNOME's virtual monitor `WxH@R`, Hyprland's monitor rule). Full screen is borderless; it is
switched with Ctrl+Alt+Enter or "Full screen" in the window's menu, and a hint says how to leave
it for four seconds after it starts.

Measured on an RTX 4060 PC (2026-10-26), limit 144:

| Desktop | Size | Updates a second | What limits it |
| --- | --- | --- | --- |
| Xfce (Xvfb) | 4K, 1080p window animating | about 59 | Xfce's compositor paces itself at 60 |
| Xfce (Xvfb) | 4K, whole screen animating | about 30 | Drawing on the CPU (llvmpipe) |
| Hyprland (GPU) | 4K | 37 to 39 | Hyprland's screen copy: 25 to 35 ms to read 4K back from the GPU |
| Hyprland (GPU) | 1440p / 1080p | about 48 | The copy: 26 / 16 ms |

Higher rates would need the picture to stay on the GPU (encoded there) instead of being read
back for every frame. `EASYWSL_AGENT_DEBUG=1` makes the agent log the copy and send times.

### X11 desktops

`desktop-setup.sh` installs the desktop and Xvfb; `desktop-start.sh` runs the session on it in
a private mount namespace (WSLg keeps `/tmp/.X11-unix` read-only). Xvfb listens on that
namespace's socket only: no TCP, and no abstract socket, which lives in the network all WSL
distros share.

- Docker-imported distros (Mint, CachyOS) run without systemd, so there is no system message
  bus; `desktop-start.sh` starts one, outside the session's namespace so it outlives the
  session, and checks that it answers (a socket left by a stopped bus once made Cinnamon abort).
  logind and ConsoleKit stay missing (Cinnamon warns; power and session menus have nothing to
  talk to).
- Xfce keeps its wallpaper per monitor name; Xvfb's is `screen`, so the distro's default image
  is set for it on first start.
- Kali's panel layout uses plugins that `xfce4` alone does not install (cpugraph, genmon,
  power manager); without them a dialog asks to remove them.
- Cinnamon repaints the whole screen while it animates (about 4 MB/s during its start), and
  nothing when idle.

### GNOME

- GPU: `gnome-start.sh` loads vgem so Mutter finds a render node; Mesa's Direct3D 12 driver then
  renders on the PC's GPU ("Created gbm renderer for /dev/dri/renderD128").
- About 46 screen updates a second with something moving on every frame; GNOME Terminal
  throttles its own redraws to about 20.
- Known: GNOME shows a "Screen Lock disabled" notice once per session and its orange "screen is
  being shared" icon while the agent records; the dock lists Windows' drives.

### Hyprland

The launcher in the engine (`hyprland-wsl --agent`) starts Hyprland as before and, instead of
wayvnc, the agent in Hyprland's namespace. Omarchy shows a red progress bar for its start-up
notice, which keeps the frames coming (about 40 small updates a second) until it goes.

## Not working yet: KDE Plasma (Wayland-only)

KDE Plasma 6.8 (Arch, CachyOS) no longer has an X11 session, so it cannot run on Xvfb, and its
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
