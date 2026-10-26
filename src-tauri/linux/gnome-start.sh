#!/bin/bash
# Runs a headless GNOME session for a user and hands it to EasyWSL's viewer through
# easywsl-agent, whose stdin and stdout are this script's (wsl.exe connects them to the viewer).
# Run as root inside the distro:
#
#   gnome-start.sh USER WIDTHxHEIGHT [AGENT]
#
# Nothing but the agent may write to stdout: everything else goes to /tmp/easywsl-gnome-*.log.

set -uo pipefail
U=${1:?user}
SIZE=${2:?size}
AGENT=${3:-easywsl-agent}

UID_=$(id -u "$U") || exit 1
R=/run/user/$UID_
# In a folder of root's: /tmp would keep root from replacing a log the user owns.
mkdir -p /run/easywsl
log=/run/easywsl/gnome-$U.log
rm -f "$log"
: > "$log" && chown "$U": "$log"
exec 2> "$log"

# GNOME lives in the user's systemd session: kept running (linger), started now if needed.
loginctl enable-linger "$U"
systemctl is-active --quiet "user@$UID_.service" || systemctl start "user@$UID_.service"
for _ in $(seq 50); do [ -S "$R/bus" ] && break; sleep 0.1; done
[ -S "$R/bus" ] || { echo "the user's session bus did not start" >&2; exit 1; }

# WSL gives Linux no display device, and without one GNOME draws on the CPU. The kernel's
# virtual GEM device gives it a render node; Mesa's Direct3D 12 driver (GALLIUM_DRIVER below)
# then draws on the PC's GPU, as for hyprland-wsl's Hyprland.
if modprobe vgem 2> /dev/null; then
    chmod 666 /dev/dri/card* /dev/dri/renderD* 2> /dev/null
fi

# A private mount namespace with its own /tmp/.X11-unix for GNOME's Xwayland (WSLg keeps the
# real one read-only). The session itself runs as the user.
exec unshare --mount --propagation private -- bash -c '
    mount -t tmpfs -o mode=1777 tmpfs /tmp/.X11-unix
    exec runuser -u "$1" -- env -u WAYLAND_DISPLAY -u DISPLAY XDG_RUNTIME_DIR="$2" \
        DBUS_SESSION_BUS_ADDRESS="unix:path=$2/bus" XDG_SESSION_TYPE=wayland \
        XDG_CURRENT_DESKTOP=ubuntu:GNOME XDG_SESSION_DESKTOP=ubuntu GNOME_SHELL_SESSION_MODE=ubuntu \
        GALLIUM_DRIVER=d3d12 bash -c "
            gnome-shell --headless --virtual-monitor \"\$1\" --mode=ubuntu >> \"\$3\" 2>&1 &
            shell=\$!
            trap \"kill \$shell 2> /dev/null\" EXIT
            # The agent needs GNOME'\''s remote desktop interface, which comes up with the shell.
            for _ in \$(seq 150); do
                gdbus introspect --session --dest org.gnome.Mutter.RemoteDesktop \
                    --object-path /org/gnome/Mutter/RemoteDesktop > /dev/null 2>&1 && break
                kill -0 \$shell 2> /dev/null || { echo \"gnome-shell ended\" >&2; exit 1; }
                sleep 0.2
            done
            \"\$2\" gnome
        " gnome "$3" "$4" "$5"
' gnome-start "$U" "$R" "$SIZE" "$AGENT" "$log"
