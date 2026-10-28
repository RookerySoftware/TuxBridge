#!/bin/bash
# Installs a distro's own desktop (Cinnamon, KDE Plasma, Xfce, GNOME) and what TuxBridge's display
# agent needs: a virtual X server (Xvfb) for the X11 desktops, which desktop-start.sh runs them
# on, and the libraries the agent links (PipeWire's and xkbcommon). Run as root inside the distro:
#
#   desktop-setup.sh DESKTOP USER        DESKTOP: cinnamon | kde | xfce | gnome
#
# The desktop is shown on Windows through TuxBridge's viewer, like the Hyprland desktop. No login
# screen or display manager is installed: TuxBridge starts the session.

set -euo pipefail
DESKTOP=${1:?desktop}
U=${2:?user}

step() { printf '\n==> %s\n' "$*"; }
id "$U" > /dev/null 2>&1 || { echo "no user $U" >&2; exit 1; }

if command -v pacman > /dev/null; then
    case $DESKTOP in
        kde) pkgs=(plasma-desktop konsole dolphin kate ark spectacle breeze breeze-gtk
                   kde-gtk-config plasma-pa systemsettings xdg-desktop-portal-kde) ;;
        xfce) pkgs=(xfce4 xfce4-terminal) ;;
        cinnamon) pkgs=(cinnamon nemo gnome-terminal) ;;
        *) echo "unknown desktop $DESKTOP" >&2; exit 1 ;;
    esac
    # CachyOS's own look for Plasma, where its repositories have it.
    if [ "$DESKTOP" = kde ] && pacman -Si cachyos-kde-settings > /dev/null 2>&1; then
        pkgs+=(cachyos-kde-settings)
    fi
    step "Installing the desktop"
    pacman -Syu --noconfirm --needed "${pkgs[@]}" xorg-server-xvfb xorg-xauth dbus noto-fonts ttf-dejavu \
        pipewire pipewire-pulse libxkbcommon
else
    export DEBIAN_FRONTEND=noninteractive
    . /etc/os-release
    case $DESKTOP in
        cinnamon)
            pkgs=(cinnamon-core nemo gnome-terminal)
            # Mint's own themes, icons and wallpapers, where the repositories have them.
            for p in mint-themes mint-y-icons mint-x-icons mint-artwork mintlocale; do
                apt-cache show "$p" > /dev/null 2>&1 && pkgs+=("$p")
            done
            ;;
        xfce)
            pkgs=(xfce4 xfce4-terminal xfce4-whiskermenu-plugin xfce4-pulseaudio-plugin xfce4-notifyd
                  mousepad thunar-archive-plugin xfce4-screenshooter xfce4-taskmanager)
            case ${ID:-} in
                # Kali's own look for Xfce, and the panel plugins its layout uses.
                kali) pkgs+=(kali-desktop-base kali-themes kali-menu xfce4-cpugraph-plugin xfce4-genmon-plugin
                             xfce4-power-manager-plugins) ;;
                # Xubuntu's look, as Ubuntu's own GNOME cannot run this way.
                ubuntu) apt-cache show xubuntu-default-settings > /dev/null 2>&1 && pkgs+=(xubuntu-default-settings xubuntu-wallpapers) ;;
            esac
            ;;
        kde)
            pkgs=(kde-plasma-desktop konsole dolphin)
            # Parrot's own look for Plasma: its theme, wallpapers and menu of tools.
            # Its panel also starts System Settings and Firefox.
            [ "${ID:-}" = parrot ] && for p in echo-themes-kde parrot-themes parrot-wallpapers parrot-menu systemsettings firefox-esr; do
                apt-cache show "$p" > /dev/null 2>&1 && pkgs+=("$p")
            done
            ;;
        # GNOME runs headless, with no X server of its own (gnome-start.sh).
        gnome)
            pkgs=(gnome-shell gnome-session gnome-terminal nautilus gnome-control-center gnome-text-editor
                  pipewire wireplumber pipewire-pulse dbus-user-session xwayland libglib2.0-bin util-linux)
            # Ubuntu's own look: its session, settings, theme, wallpapers and font.
            for p in ubuntu-session ubuntu-settings yaru-theme-gnome-shell yaru-theme-gtk yaru-theme-icon \
                     ubuntu-wallpapers fonts-ubuntu; do
                apt-cache show "$p" > /dev/null 2>&1 && pkgs+=("$p")
            done
            ;;
        *) echo "unknown desktop $DESKTOP" >&2; exit 1 ;;
    esac
    step "Installing the desktop"
    apt-get update
    # GNOME needs no X server of its own; the others run on Xvfb.
    x=(xvfb dbus-x11 xauth)
    [ "$DESKTOP" = gnome ] && x=()
    # The agent's libraries; PipeWire's is named after the t64 transition in newer releases.
    for p in libpipewire-0.3-0t64 libpipewire-0.3-0; do
        apt-cache show "$p" > /dev/null 2>&1 && { x+=("$p"); break; }
    done
    apt-get install -y --no-install-recommends "${pkgs[@]}" "${x[@]}" libxkbcommon0 \
        fonts-dejavu-core fonts-noto-core adwaita-icon-theme pulseaudio-utils at-spi2-core
fi

step "Preparing the session"
# Parrot's panel has a VSCodium button, which is a 600 MB download of its own; without it, the
# button goes. The account's copy of the layout was made from /etc/skel already.
if [ "$DESKTOP" = kde ] && ! command -v codium > /dev/null; then
    H=$(getent passwd "$U" | cut -d: -f6)
    for f in /etc/skel/.config/plasma-org.kde.plasma.desktop-appletsrc "$H/.config/plasma-org.kde.plasma.desktop-appletsrc"; do
        [ -f "$f" ] && sed -i 's/,applications:codium\.desktop//' "$f"
    done
fi
# The user's systemd session is kept running, so that /run/user/<uid> exists before the desktop
# starts and a later login cannot replace it (see hyprland-wsl's install.sh).
if [ -d /run/systemd/system ]; then
    loginctl enable-linger "$U" || true
fi
mkdir -p /etc/easywsl
echo "$DESKTOP" > /etc/easywsl/desktop
# Whose desktop it is, for TuxBridge to find again (a system can have several accounts).
echo "$U" > /etc/easywsl/desktop-user
echo "The $DESKTOP desktop is installed for $U."
