---
title: USB devices
summary: Give a USB device (a microcontroller, a serial adapter, a security key, a webcam) to a Linux system with usbipd-win.
section: WSL and Windows
order: 55
---

# USB devices

USB storage (flash drives, external disks) is already reachable from Linux through Windows,
under `/mnt/<drive letter>`. Other USB devices — Arduino and ESP32 boards, serial adapters,
hardware keys, some webcams — belong to Windows until you hand them over with **usbipd-win**, a
free tool from Microsoft's community.

## Install it (once)

In PowerShell:

```powershell
winget install --interactive --exact dorssel.usbipd-win
```

## Hand a device to Linux

1. Keep a Linux system running (open its terminal).
2. In PowerShell, list the devices:

```powershell
usbipd list
```

```
BUSID  VID:PID    DEVICE                                  STATE
2-3    2341:0043  Arduino Uno (COM3)                      Not shared
```

3. Share it once (needs an **administrator** PowerShell, only the first time per device):

```powershell
usbipd bind --busid 2-3
```

4. Attach it to WSL (a normal PowerShell is fine):

```powershell
usbipd attach --wsl --busid 2-3
```

While attached, Windows can't use it. In Linux it appears like on a normal PC:

```bash
lsusb                    # it's listed (sudo apt install usbutils if lsusb is missing)
ls /dev/ttyACM* /dev/ttyUSB*     # serial devices
```

5. Give it back to Windows:

```powershell
usbipd detach --busid 2-3
```

Unplugging also detaches it. `usbipd attach --wsl --busid 2-3 --auto-attach` re-attaches it
each time it's plugged in, while that PowerShell stays open.

## Serial devices: permission denied

Serial ports belong to a group (`dialout` on Ubuntu/Debian, `uucp` on Arch). Join it, then
restart the system:

```bash
sudo usermod -aG dialout $USER     # Arch: uucp
```

## Notes

- Webcams work with some drivers, but often need a WSL kernel with extra modules; using them
  from Windows apps is easier.
- Each WSL restart detaches devices; attach them again.
- `usbipd` is not part of EasyWSL; its documentation is at github.com/dorssel/usbipd-win.
