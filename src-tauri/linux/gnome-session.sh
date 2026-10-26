#!/bin/bash
# The user's part of gnome-start.sh: GNOME Shell headless, then easywsl-agent on this script's
# stdin and stdout. Run as the user, in gnome-start.sh's mount namespace:
#
#   gnome-session.sh RUNTIME_DIR WIDTHxHEIGHT AGENT LOG SCALE FPS

set -uo pipefail
R=$1 SIZE=$2 AGENT=$3 LOG=$4 SCALE=$5 FPS=${6:-60}
exec 2>> "$LOG"

unset WAYLAND_DISPLAY DISPLAY
export XDG_RUNTIME_DIR=$R DBUS_SESSION_BUS_ADDRESS=unix:path=$R/bus XDG_SESSION_TYPE=wayland
export XDG_CURRENT_DESKTOP=ubuntu:GNOME XDG_SESSION_DESKTOP=ubuntu GNOME_SHELL_SESSION_MODE=ubuntu
export GALLIUM_DRIVER=d3d12

# Windows' scaling for GNOME's text. No lock screen: it needs GNOME's login manager, which is not
# there, and GNOME would only show a notice saying so; Windows' own lock covers the viewer.
gsettings set org.gnome.desktop.interface text-scaling-factor "$SCALE"
gsettings set org.gnome.desktop.lockdown disable-lock-screen true
gsettings set org.gnome.desktop.screensaver lock-enabled false

# The virtual monitor refreshes at the frame rate the viewer gets.
gnome-shell --headless --virtual-monitor "$SIZE@$FPS" --mode=ubuntu >> "$LOG" 2>&1 &
shell=$!
trap 'kill $shell 2> /dev/null' EXIT

# The agent needs GNOME's remote desktop interface, which comes up with the shell.
for _ in $(seq 150); do
    gdbus introspect --session --dest org.gnome.Mutter.RemoteDesktop \
        --object-path /org/gnome/Mutter/RemoteDesktop > /dev/null 2>&1 && break
    kill -0 $shell 2> /dev/null || { echo "gnome-shell ended" >&2; exit 1; }
    sleep 0.2
done
"$AGENT" --fps "$FPS" gnome
