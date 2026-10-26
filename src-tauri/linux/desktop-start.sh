#!/bin/bash
# Runs a distro's own desktop (installed by desktop-setup.sh) on a virtual X server (Xvfb) and
# hands it to EasyWSL's viewer through easywsl-agent, whose stdin and stdout are this script's
# (wsl.exe connects them to the viewer). Run as root inside the distro:
#
#   desktop-start.sh run USER DESKTOP WIDTHxHEIGHT AGENT [SCALE [FPS]]   until the viewer closes or
#                                                                   the session ends
#   desktop-start.sh stop                                           ends a running session
#
# SCALE is Windows' display scaling (1.5 for 150 %); the desktop's text is scaled to match. FPS
# is the most frames a second the agent sends (default 60).
# Nothing but the agent may write to stdout: everything else goes to
# /run/easywsl/desktop-USER.log. Everything runs in a private mount namespace: WSLg keeps
# /tmp/.X11-unix read-only, and the X server needs its own socket there.

set -uo pipefail
STATE=/run/easywsl-desktop

case ${1:-} in
stop)
    [ -f $STATE/pid ] && kill "$(cat $STATE/pid)" 2> /dev/null
    exit 0
    ;;
run) ;;
inner) ;;
*) echo "usage: desktop-start.sh run USER DESKTOP SIZE AGENT [SCALE] | stop" >&2; exit 2 ;;
esac

if [ "$1" = run ]; then
    shift
    mkdir -p $STATE /run/easywsl
    # The viewer's channel moves to fd 3 (out) and 4 (in); the rest goes to the log.
    exec 3>&1 4<&0 > "/run/easywsl/desktop-${1:?user}.log" 2>&1 < /dev/null
    UID_=$(id -u "$1") || exit 1

    # What outlives the session is started here, outside its namespace (the session's processes
    # are all ended with it). The user's session first (linger), so it owns the runtime directory.
    if [ -e "/var/lib/systemd/linger/$1" ] && [ -d /run/systemd/system ]; then
        systemctl is-active --quiet "user@$UID_.service" || systemctl start "user@$UID_.service" || true
    fi
    # Desktops talk to the system through the system message bus. Distros imported from Docker
    # images run without systemd, which would otherwise start it. A socket left by a bus that
    # was stopped does not count.
    if command -v dbus-daemon > /dev/null && ! dbus-send --system --print-reply --dest=org.freedesktop.DBus /             org.freedesktop.DBus.GetId > /dev/null 2>&1; then
        mkdir -p /run/dbus
        rm -f /run/dbus/pid /run/dbus/system_bus_socket
        dbus-daemon --system --fork 3>&- 4<&- || echo "warning: the system message bus did not start"
    fi

    # The private namespace; this script runs again inside it as "inner".
    exec unshare --mount --propagation private -- "$0" inner "$@"
fi

shift
U=${1:?user} DESKTOP=${2:?desktop} SIZE=${3:?size} AGENT=${4:?agent} SCALE=${5:-1} FPS=${6:-60}
UID_=$(id -u "$U") || exit 1
R=/run/user/$UID_
DISPLAY_NUM=1
DPI=$(awk -v s="$SCALE" 'BEGIN { printf "%d", 96 * s + 0.5 }')

mount -t tmpfs -o mode=1777 tmpfs /tmp/.X11-unix

mkdir -p "$R" && chown "$U": "$R" && chmod 700 "$R"

# Each desktop's own text scaling, set inside the session before it starts.
case $DESKTOP in
    cinnamon) session=cinnamon-session
              scaling="gsettings set org.cinnamon.desktop.interface text-scaling-factor $SCALE" ;;
    kde) session=startplasma-x11 scaling=: ;;
    xfce) session=startxfce4
          scaling="xfconf-query -c xsettings -p /Xft/DPI -n -t int -s $DPI" ;;
    *) echo "unknown desktop $DESKTOP"; exit 1 ;;
esac
command -v "$session" > /dev/null || { echo "the $DESKTOP desktop is not installed"; exit 1; }

# Settings that make these desktops behave on a virtual screen without a GPU: KDE's compositor
# off (it would draw on the CPU at every frame), no screen locker (the viewer is the lock).
H=$(getent passwd "$U" | cut -d: -f6)
if [ "$DESKTOP" = kde ]; then
    su "$U" -c "kwriteconfig6 --file kwinrc --group Compositing --key Enabled false; \
                kwriteconfig6 --file kscreenlockerrc --group Daemon --key Autolock false" || true
fi

# Kali names qterminal as Xfce's terminal, which is not installed here; Ctrl+Alt+T would ask
# which terminal to use. The user gets Xfce's own instead, whenever the named one is missing.
if [ "$DESKTOP" = xfce ]; then
    term=$(sed -n 's/^TerminalEmulator=//p' /etc/xdg/xfce4/helpers.rc 2> /dev/null)
    if [ -n "$term" ] && ! command -v "$term" > /dev/null && [ ! -e "$H/.config/xfce4/helpers.rc" ]; then
        su "$U" -c "mkdir -p '$H/.config/xfce4' && echo TerminalEmulator=xfce4-terminal > '$H/.config/xfce4/helpers.rc'"
    fi
fi

# Xfce keeps its wallpaper per monitor name, and the distro's default names physical monitors;
# Xvfb's screen is "screen". The distro's default image is set for it once.
desktop_xml=$H/.config/xfce4/xfconf/xfce-perchannel-xml/xfce4-desktop.xml
if [ "$DESKTOP" = xfce ] && [ ! -e "$desktop_xml" ]; then
    img=$(grep -ho '/usr/share/backgrounds/[^"<]*' /etc/xdg/xfce4/xfconf/xfce-perchannel-xml/xfce4-desktop.xml 2> /dev/null | head -n 1)
    if [ -n "$img" ] && [ -e "$img" ]; then
        su "$U" -c "mkdir -p '${desktop_xml%/*}'"
        su "$U" -c "cat > '$desktop_xml'" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<channel name="xfce4-desktop" version="1.0">
  <property name="backdrop" type="empty">
    <property name="single-workspace-mode" type="bool" value="true"/>
    <property name="single-workspace-number" type="int" value="0"/>
    <property name="screen0" type="empty">
      <property name="monitorscreen" type="empty">
        <property name="workspace0" type="empty">
          <property name="last-image" type="string" value="$img"/>
          <property name="image-style" type="int" value="5"/>
          <property name="color-style" type="int" value="0"/>
        </property>
      </property>
    </property>
  </property>
</channel>
EOF
    fi
fi

echo $$ > $STATE/pid
agent= session_pid=
# Everything the session started lives in this mount namespace, and goes with it.
cleanup() {
    own=$(readlink /proc/$$/ns/mnt)
    for p in /proc/[0-9]*; do
        p=${p#/proc/}
        [ "$p" != $$ ] && [ "$p" != "$BASHPID" ] && [ "$(readlink "/proc/$p/ns/mnt" 2> /dev/null)" = "$own" ] &&
            kill -9 "$p" 2> /dev/null
    done
    rm -f $STATE/pid
}
trap cleanup EXIT
trap 'exit 0' TERM INT HUP
# When the viewer is killed, WSL kills this script outright, with no chance to clean up; a
# watcher does it then.
( trap '' TERM INT HUP; while kill -0 $$ 2> /dev/null; do sleep 1; done; cleanup ) 3>&- 4<&- &

# Only the socket in this namespace: no TCP, and no abstract socket, which lives in the network
# all WSL distros share (and where WSLg's X server is).
su "$U" -c "Xvfb :$DISPLAY_NUM -screen 0 ${SIZE}x24 -dpi $DPI -nolisten tcp -nolisten local" 3>&- 4<&- &
for _ in $(seq 50); do [ -S /tmp/.X11-unix/X$DISPLAY_NUM ] && break; sleep 0.1; done
[ -S /tmp/.X11-unix/X$DISPLAY_NUM ] || { echo "the X server did not start"; exit 1; }
echo "The $DESKTOP desktop is starting at $SIZE ($DPI dpi)."

# WSLg's Wayland and X displays are left out, so apps open on this desktop; sound goes to
# Windows through WSLg's PulseAudio server.
su - "$U" -c "env -u WAYLAND_DISPLAY DISPLAY=:$DISPLAY_NUM XDG_RUNTIME_DIR=$R XDG_SESSION_TYPE=x11 \
    GDK_BACKEND=x11 QT_QPA_PLATFORM=xcb PULSE_SERVER=unix:/mnt/wslg/PulseServer \
    XDG_CURRENT_DESKTOP= dbus-run-session -- sh -c '$scaling; exec $session'" 3>&- 4<&- &
session_pid=$!

runuser -u "$U" -- env DISPLAY=:$DISPLAY_NUM "$AGENT" --fps "$FPS" x11 <&4 >&3 &
agent=$!
exec 3>&- 4<&-

# The viewer closed (the agent ends) or the desktop logged out (the session ends).
wait -n $agent $session_pid
