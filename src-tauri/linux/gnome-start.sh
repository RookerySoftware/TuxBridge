#!/bin/bash
# Runs a headless GNOME session for a user and hands it to EasyWSL's viewer through
# easywsl-agent, whose stdin and stdout are this script's (wsl.exe connects them to the viewer).
# Run as root inside the distro:
#
#   gnome-start.sh USER WIDTHxHEIGHT [AGENT [SCALE]]
#
# SCALE is Windows' display scaling (1.5 for 150 %); GNOME's text is scaled to match. The part
# that runs as the user is gnome-session.sh, next to this script.
#
# Nothing but the agent may write to stdout: everything else goes to /run/easywsl/gnome-USER.log.

set -uo pipefail
U=${1:?user}
SIZE=${2:?size}
AGENT=${3:-easywsl-agent}
SCALE=${4:-1}
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

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
# virtual GEM device gives it a render node; Mesa's Direct3D 12 driver then draws on the PC's
# GPU, as for hyprland-wsl's Hyprland.
if modprobe vgem 2> /dev/null; then
    chmod 666 /dev/dri/card* /dev/dri/renderD* 2> /dev/null
fi

# A private mount namespace with its own /tmp/.X11-unix for GNOME's Xwayland (WSLg keeps the
# real one read-only); the session runs as the user.
exec unshare --mount --propagation private -- /bin/sh -c '
    mount -t tmpfs -o mode=1777 tmpfs /tmp/.X11-unix && exec runuser -u "$1" -- "$2" "$3" "$4" "$5" "$6" "$7"
' gnome-start "$U" "$HERE/gnome-session.sh" "$R" "$SIZE" "$AGENT" "$log" "$SCALE"
