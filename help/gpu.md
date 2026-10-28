---
title: Graphics, the GPU and Linux apps
summary: Linux apps with windows, using the graphics card for 3D, CUDA and AI, and fixing blurry or slow windows.
section: WSL and Windows
order: 54
---

# Graphics, the GPU and Linux apps

## Linux apps with windows (WSLg)

WSL can show Linux programs as normal windows on your Windows desktop, with sound, even when
the system has no desktop. Install a program and start it from the terminal:

```bash
sudo apt install gimp          # or: sudo pacman -S gimp
gimp &                         # & gives the terminal back
```

Installed apps with a menu entry also appear in Windows' **Start menu**, in a folder named after
the system.

Inside a desktop opened from Pengi (GNOME, Cinnamon, Xfce, Hyprland), programs open on that
desktop instead.

## The graphics card (GPU)

WSL passes your graphics card through to Linux. **Install drivers on Windows only** — the normal
NVIDIA, AMD or Intel driver. Never install a Linux display driver inside WSL: it replaces the
WSL one and breaks things.

Check that Linux sees it:

```bash
sudo apt install mesa-utils     # or: sudo pacman -S mesa-utils
glxinfo -B                      # "OpenGL renderer string" should name your card (via D3D12)
nvidia-smi                      # NVIDIA only: shows the card, driver and memory
```

If the renderer says `llvmpipe`, drawing is done by the processor: update the Windows driver
and WSL (`wsl --update`), and on Ubuntu/Debian make sure the system's Mesa is recent enough.

## CUDA and AI (NVIDIA)

1. Install the current NVIDIA driver on **Windows** (it includes WSL support).
2. In Linux, install the **CUDA toolkit for WSL** only if you compile CUDA code yourself
   (NVIDIA offers a "WSL-Ubuntu" package that leaves the driver alone). Most people skip this.
3. Install the CUDA build of your library, for example:

```bash
uv pip install torch            # PyTorch's Linux wheels include CUDA
python3 -c "import torch; print(torch.cuda.is_available())"   # True
```

Ollama, llama.cpp, TensorFlow and others use the GPU the same way. AMD (ROCm) and Intel GPUs
have WSL support for some cards; check their documentation for which models.

## Video, sound and the microphone

- **Sound** plays through Windows automatically (WSLg's PulseAudio server).
- **Microphone** input works for most apps through the same route.
- **Webcams** are USB devices: see [USB devices](usb.md).
- **Hardware video decoding** works with some drivers through VA-API; software decoding always
  works.

## Fixing how windows look

| Problem | Fix |
| --- | --- |
| Tiny text on a high-resolution screen | `export GDK_SCALE=2` (GTK apps) or `QT_SCALE_FACTOR=2` (Qt apps) before starting it; put it in `~/.bashrc` to keep it. |
| Blurry text | The opposite: `GDK_SCALE=1`, and Windows' display scaling at 100% or 200% helps. |
| Missing letters or boxes for symbols | Install fonts: `sudo apt install fonts-noto fonts-noto-color-emoji` (Arch: `noto-fonts noto-fonts-emoji`). |
| App opens nothing, `cannot open display` | Restart WSL; check `echo $DISPLAY` shows `:0`; `wsl --update`. |
| Window menus look old | Install a theme, e.g. `sudo apt install gnome-themes-extra`, or use the system's desktop. |

## Games and 3D

3D programs use the GPU through Direct3D 12 (Mesa's D3D12 driver). Many work well; demanding
games are better played on Windows itself. Steam's Linux client runs, but Windows' own Steam is
usually faster on the same PC.
