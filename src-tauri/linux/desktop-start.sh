#!/bin/bash
# Runs a distro's own desktop (installed by desktop-setup.sh) on TigerVNC's X server, for
# EasyWSL's viewer on Windows. Run as root inside the distro:
#
#   desktop-start.sh run USER DESKTOP WIDTHxHEIGHT PORT     until the session ends
#   desktop-start.sh stop                                   ends a running session
#
# The X server listens on 127.0.0.1 only; WSL forwards that port to Windows' localhost and
# nowhere else. Everything runs in a private mount namespace: WSLg keeps /tmp/.X11-unix
# read-only, and the X server needs its own socket there.

set -uo pipefail
STATE=/run/easywsl-desktop

case ${1:-} in
stop)
    [ -f $STATE/pid ] && kill "$(cat $STATE/pid)" 2> /dev/null
    exit 0
    ;;
run) ;;
inner) ;;
*) echo "usage: desktop-start.sh run USER DESKTOP SIZE PORT | stop" >&2; exit 2 ;;
esac

if [ "$1" = run ]; then
    shift
    mkdir -p $STATE
    # The private namespace; this script runs again inside it as "inner".
    exec unshare --mount --propagation private -- "$0" inner "$@"
fi

shift
U=$1 DESKTOP=$2 SIZE=$3 PORT=$4
UID_=$(id -u "$U")
R=/run/user/$UID_
DISPLAY_NUM=1

mount -t tmpfs -o mode=1777 tmpfs /tmp/.X11-unix

# The user's session first (linger), so it owns the runtime directory.
if [ -e "/var/lib/systemd/linger/$U" ] && [ -d /run/systemd/system ]; then
    systemctl is-active --quiet "user@$UID_.service" || systemctl start "user@$UID_.service" || true
fi
mkdir -p "$R" && chown "$U": "$R" && chmod 700 "$R"

# Desktops talk to the system through the system message bus. Distros imported from Docker
# images run without systemd, which would otherwise start it.
if [ ! -S /run/dbus/system_bus_socket ] && command -v dbus-daemon > /dev/null; then
    mkdir -p /run/dbus
    rm -f /run/dbus/pid
    dbus-daemon --system --fork || echo "warning: the system message bus did not start" >&2
fi

case $DESKTOP in
    cinnamon) session=cinnamon-session ;;
    kde) session=startplasma-x11 ;;
    xfce) session=startxfce4 ;;
    *) echo "unknown desktop $DESKTOP" >&2; exit 1 ;;
esac
command -v "$session" > /dev/null || { echo "the $DESKTOP desktop is not installed" >&2; exit 1; }

# Settings that make these desktops behave on a virtual screen without a GPU: KDE's compositor
# off (it would draw on the CPU at every frame), no screen locker (the viewer is the lock).
H=$(getent passwd "$U" | cut -d: -f6)
if [ "$DESKTOP" = kde ]; then
    su "$U" -c "kwriteconfig6 --file kwinrc --group Compositing --key Enabled false; \
                kwriteconfig6 --file kscreenlockerrc --group Daemon --key Autolock false" 2> /dev/null || true
fi

# Xfce keeps its wallpaper per monitor name, and the distro's default names physical monitors;
# the virtual screen here is "VNC-0". The distro's default image is set for it once.
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
      <property name="monitorVNC-0" type="empty">
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

echo $$ > /run/easywsl-desktop/pid
cleanup() { kill "$xvnc" 2> /dev/null; rm -f /run/easywsl-desktop/pid; }
trap cleanup EXIT
trap 'exit 0' TERM INT

su "$U" -c "Xvnc :$DISPLAY_NUM -geometry $SIZE -depth 24 -rfbport $PORT -localhost -interface 127.0.0.1 \
    -SecurityTypes None -AlwaysShared -AcceptSetDesktopSize=0 -nolisten tcp" > /tmp/easywsl-xvnc.log 2>&1 &
xvnc=$!
for _ in $(seq 50); do [ -S /tmp/.X11-unix/X$DISPLAY_NUM ] && break; sleep 0.1; done
[ -S /tmp/.X11-unix/X$DISPLAY_NUM ] || { cat /tmp/easywsl-xvnc.log >&2; echo "the X server did not start" >&2; exit 1; }
echo "The $DESKTOP desktop is running at $SIZE (VNC on 127.0.0.1:$PORT)."

# WSLg's Wayland and X displays are left out, so apps open on this desktop; sound goes to
# Windows through WSLg's PulseAudio server.
su - "$U" -c "env -u WAYLAND_DISPLAY DISPLAY=:$DISPLAY_NUM XDG_RUNTIME_DIR=$R XDG_SESSION_TYPE=x11 \
    GDK_BACKEND=x11 QT_QPA_PLATFORM=xcb PULSE_SERVER=unix:/mnt/wslg/PulseServer \
    XDG_CURRENT_DESKTOP= dbus-run-session -- $session" > /tmp/easywsl-session.log 2>&1 &
wait $!
