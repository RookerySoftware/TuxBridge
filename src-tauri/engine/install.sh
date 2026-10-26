#!/bin/bash
# Installs Hyprland under WSL2, plus everything needed to show it on the Windows desktop.
# Supports Arch-based distros (pacman) and Debian/Ubuntu-based distros (apt).
# Run from a clone of this repository:  sudo ./install.sh
#
# Environment:
#   HYPR_USER   user that Hyprland runs as (default: the invoking user, else the first
#               regular user, else a new user called "hypr")

set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PREFIX=/opt/hyprland-wsl
CONF=/etc/hyprland-wsl.conf

die() { echo "error: $*" >&2; exit 1; }
step() { printf '\n==> %s\n' "$*"; }

if [ "$(id -u)" != 0 ]; then
    command -v sudo >/dev/null || die "run this as root (sudo is not installed)"
    exec sudo env HYPR_USER="${HYPR_USER:-$USER}" WSL_DISTRO_NAME="${WSL_DISTRO_NAME:-}" \
        LANG="${LANG:-}" "$0" "$@"
fi

step "Checking the environment"
. /etc/os-release
case " $ID ${ID_LIKE:-} " in
    *" arch "*) FAMILY=arch ;;
    *" debian "*|*" ubuntu "*) FAMILY=debian ;;
    *) die "unsupported distro '$ID'; supported are Arch-based and Debian/Ubuntu-based ones" ;;
esac
echo "$PRETTY_NAME ($FAMILY family)"
grep -qi microsoft /proc/version || die "this does not look like WSL2"
ls "$HERE"/patches/headless-drm*.patch >/dev/null 2>&1 && [ -f "$HERE/hyprland-wsl" ] ||
    die "run install.sh from a clone of the repository"
modinfo vgem >/dev/null 2>&1 || die "the WSL kernel has no vgem module; update WSL with 'wsl --update'"
[ -d /mnt/wslg ] || echo "warning: WSLg not found; the viewer window will not work, use 'hyprland-wsl --no-viewer'"

# Ubuntu 24.04 and what is built on it (Linux Mint 22.x) have no hyprland package. A
# third-party PPA has one; it is only added with the user's consent.
add_ppa() {
    local ppa=cppiber/hyprland series=${UBUNTU_CODENAME:-} answer=${HYPRLAND_PPA:-} fp
    local url=https://ppa.launchpadcontent.net/$ppa/ubuntu
    [ -n "$series" ] && curl -fsI "$url/dists/$series/Release" >/dev/null 2>&1 ||
        die "this release of $NAME has no hyprland package, and no PPA is known for it"
    echo "This release of $NAME has no hyprland package."
    echo "The third-party PPA ppa:$ppa (not part of $NAME, not checked by this project) provides one."
    if [ -z "$answer" ] && [ -t 0 ]; then
        read -r -p "Add that PPA and install Hyprland from it? [y/N] " answer
    fi
    case $answer in
        1|y|Y|yes) ;;
        *) die "not adding the PPA; rerun with HYPRLAND_PPA=1 to allow it" ;;
    esac
    fp=$(curl -fsSL "https://api.launchpad.net/1.0/~${ppa%/*}/+archive/ubuntu/${ppa#*/}" |
        sed -n 's/.*"signing_key_fingerprint": "\([0-9A-F]*\)".*/\1/p')
    [ -n "$fp" ] || die "could not look up the PPA's signing key"
    mkdir -p /etc/apt/keyrings
    curl -fsSL "https://keyserver.ubuntu.com/pks/lookup?op=get&search=0x$fp" -o /etc/apt/keyrings/hyprland-wsl-ppa.asc
    echo "deb [signed-by=/etc/apt/keyrings/hyprland-wsl-ppa.asc] $url $series main" \
        > /etc/apt/sources.list.d/hyprland-wsl-ppa.list
    apt-get update
    apt-cache show hyprland >/dev/null 2>&1 || die "the PPA did not provide a hyprland package"
}

# Rendering on the GPU needs Mesa 26 or newer: with Mesa 25.2 (Ubuntu 24.04, Linux Mint 22.x)
# the viewer stays blank, and the launcher then falls back to the CPU. The third-party
# kisak-mesa PPA has a current Mesa for those releases; it is only added with the user's consent.
newer_mesa() {
    local ppa=kisak/kisak-mesa series=${UBUNTU_CODENAME:-} answer=${HYPRLAND_MESA_PPA:-} fp mesa pkgs
    local url=https://ppa.launchpadcontent.net/$ppa/ubuntu
    [ -e /dev/dxg ] || return 0
    mesa=$(ls /usr/lib/*/libgallium-*.so 2>/dev/null | sed -n 's/.*libgallium-\([0-9]*\)\..*/\1/p' | sort -n | tail -n 1)
    [ "${mesa:-0}" -lt 26 ] || return 0
    [ -n "$series" ] && curl -fsI "$url/dists/$series/Release" >/dev/null 2>&1 || {
        echo "Mesa ${mesa:-?} is too old for GPU rendering and no newer one is known for this release; Hyprland will render on the CPU."
        return 0
    }
    echo "Mesa ${mesa:-?} is too old for GPU rendering; Hyprland would render on the CPU."
    echo "The third-party PPA ppa:$ppa (not part of $NAME, not checked by this project) provides a current Mesa."
    if [ -z "$answer" ] && [ -t 0 ]; then
        read -r -p "Add that PPA and upgrade Mesa from it? [y/N] " answer
    fi
    case $answer in
        1|y|Y|yes) ;;
        *) echo "Not adding it. Rerun with HYPRLAND_MESA_PPA=1 to allow it."; return 0 ;;
    esac
    fp=$(curl -fsSL "https://api.launchpad.net/1.0/~${ppa%/*}/+archive/ubuntu/${ppa#*/}" |
        sed -n 's/.*"signing_key_fingerprint": "\([0-9A-F]*\)".*/\1/p')
    [ -n "$fp" ] || die "could not look up the Mesa PPA's signing key"
    mkdir -p /etc/apt/keyrings
    curl -fsSL "https://keyserver.ubuntu.com/pks/lookup?op=get&search=0x$fp" -o /etc/apt/keyrings/hyprland-wsl-mesa.asc
    echo "deb [signed-by=/etc/apt/keyrings/hyprland-wsl-mesa.asc] $url $series main" \
        > /etc/apt/sources.list.d/hyprland-wsl-mesa.list
    apt-get update
    # Only Mesa's own packages are moved to the PPA's version, not everything it carries.
    pkgs=$(dpkg-query -W -f='${binary:Package}\n' 'libgl1-mesa-dri*' 'libegl-mesa0*' 'libgbm1*' 'libgbm-dev*' \
        'libglx-mesa0*' 'libglapi-mesa*' 'mesa-*' 'libegl1-mesa-dev*' 2>/dev/null | sort -u || true)
    [ -n "$pkgs" ] || die "found no installed Mesa packages to upgrade"
    apt-get install -y --no-install-recommends $pkgs
}

# A root filesystem imported from a hand-made tarball can end up with / readable by root only.
# Nothing then works for a regular user, and the failure shows up far from its cause.
if [ "$(stat -c %a /)" = 700 ]; then
    echo "warning: / has mode 700, which locks every regular user out; setting it to 755"
    chmod 755 /
fi

step "Installing packages"
case $FAMILY in
arch)
    pacman-key --init
    pacman-key --populate
    pkgs=(hyprland foot mesa wayvnc tigervnc sudo ttf-dejavu curl
        base-devel cmake ninja hyprwayland-scanner wayland-protocols)
    if pacman -Q omarchy >/dev/null 2>&1; then
        # Omarchy upgrades through "omarchy update" only. Refreshing the package lists here and
        # installing would upgrade some packages without the rest (a partial upgrade), so only
        # what is missing is installed, from the lists Omarchy last synced.
        missing=($(pacman -T "${pkgs[@]}" || true))
        [ ${#missing[@]} = 0 ] || pacman -S --noconfirm --needed "${missing[@]}"
    else
        # A full upgrade, because Arch does not support installing onto a partly updated system.
        pacman -Syu --noconfirm --needed "${pkgs[@]}"
    fi
    # Already there as dependencies of something else, they would count as unneeded once that
    # goes, and Omarchy's update offers to remove such packages. The patched aquamarine is
    # rebuilt with them after updates.
    pacman -D --asexplicit cmake ninja hyprwayland-scanner wayland-protocols > /dev/null
    ;;
debian)
    export DEBIAN_FRONTEND=noninteractive
    apt-get update
    apt-get install -y --no-install-recommends curl ca-certificates
    apt-cache show hyprland >/dev/null 2>&1 || add_ppa
    apt-get install -y --no-install-recommends \
        hyprland foot wayvnc tigervnc-viewer sudo fonts-dejavu-core \
        libgl1-mesa-dri libegl1 libgles2 xkb-data \
        curl ca-certificates patch build-essential cmake ninja-build pkg-config \
        hyprwayland-scanner libhyprutils-dev libwayland-dev wayland-protocols libseat-dev \
        libinput-dev libpixman-1-dev libdrm-dev libgbm-dev libudev-dev libdisplay-info-dev \
        hwdata libegl-dev libgles-dev libgl-dev libopengl-dev
    newer_mesa
    ;;
esac

# WSL passes Windows' language into every session (LANG=en_US.UTF-8, for example), but the
# distro images generate no locale besides C.UTF-8, so programs warn "Failed to set locale"
# or "Cannot set LC_CTYPE" and fall back to C.
lang=${LANG:-}
if [ -n "$lang" ] && [ "$lang" != C.UTF-8 ] && [ "$lang" != C ] && [ "$lang" != POSIX ] &&
    ! locale -a 2>/dev/null | grep -qix "$(echo "$lang" | sed 's/UTF-8$/utf8/I')"; then
    step "Generating the locale $lang"
    if [ -f /etc/locale.gen ]; then
        entry="$lang ${lang##*.}"
        grep -q "^$entry" /etc/locale.gen || sed -i "s/^#[[:space:]]*$entry/$entry/" /etc/locale.gen
        grep -q "^$entry" /etc/locale.gen || echo "$entry" >> /etc/locale.gen
        locale-gen > /dev/null || echo "warning: could not generate $lang"
    elif command -v locale-gen > /dev/null; then
        locale-gen "$lang" > /dev/null || echo "warning: could not generate $lang"
    else
        echo "no locale-gen here; programs keep falling back to the C locale"
    fi
fi

step "Choosing the user"
# HYPR_USER=- leaves the choice to a first-start setup (see the launcher's "first-start"):
# the user does not exist yet, as on a machine set up for its future owner.
if [ "${HYPR_USER:-}" = - ]; then
    echo "no user yet; it is created by the first-start setup"
    U= H=
else
U=${HYPR_USER:-${SUDO_USER:-}}
[ "$U" = root ] && U=
[ -n "$U" ] || U=$(awk -F: '$3 >= 1000 && $3 < 60000 { print $1; exit }' /etc/passwd)
[ -n "$U" ] || U=hypr
if ! id "$U" >/dev/null 2>&1; then
    useradd -m -s /bin/bash "$U"
    echo "created user $U"
fi
H=$(getent passwd "$U" | cut -d: -f6)
echo "Hyprland will run as $U"
fi

step "Building the patched aquamarine"
# Installed with its patches, so that package manager hooks and the launcher can rebuild it
# when an update brings a new aquamarine.
rm -rf $PREFIX/patches
mkdir -p $PREFIX/patches
install -m 644 "$HERE"/patches/headless-drm*.patch $PREFIX/patches/
install -m 755 "$HERE/tools/build-aquamarine.sh" $PREFIX/build-aquamarine
$PREFIX/build-aquamarine
case $FAMILY in
arch)
    mkdir -p /etc/pacman.d/hooks
    cat > /etc/pacman.d/hooks/95-hyprland-wsl-aquamarine.hook <<'HOOK'
[Trigger]
Operation = Install
Operation = Upgrade
Type = Package
Target = aquamarine

[Action]
Description = Rebuilding hyprland-wsl's patched aquamarine...
When = PostTransaction
Exec = /opt/hyprland-wsl/build-aquamarine --hook
HOOK
    ;;
debian)
    # apt has no per-package hooks; this runs after every dpkg run and returns at once
    # when the patched copy is current.
    cat > /etc/apt/apt.conf.d/99hyprland-wsl <<'HOOK'
DPkg::Post-Invoke { "if [ -x /opt/hyprland-wsl/build-aquamarine ]; then /opt/hyprland-wsl/build-aquamarine --hook; fi"; };
HOOK
    ;;
esac

step "Installing the launcher"
install -m 755 "$HERE/hyprland-wsl" /usr/local/bin/hyprland-wsl
mkdir -p $PREFIX/lib
cc -shared -fPIC -O2 -o $PREFIX/lib/no-cursor-image.so "$HERE/src/no-cursor-image.c" -ldl ||
    echo "warning: could not build no-cursor-image.so; the Windows viewer may show a square on the pointer"
echo "HYPR_USER=$U" > $CONF

# All WSL distros share one kernel and with it one tree of control groups. A user's systemd
# session goes to /user.slice/user-<uid>.slice/user@<uid>.service, the same path in every
# distro, and the first user is uid 1000 almost everywhere: while one distro runs that user's
# session (a Windows Terminal tab is enough), another cannot start its own ("Failed to spawn
# executor: Device or resource busy"), and apps that need it do not open. A slice named
# after this distro gives its user sessions a path of their own. Takes effect at the next
# start of the instance.
if [ -d /run/systemd/system ]; then
    tag=$(printf '%s' "${WSL_DISTRO_NAME:-}" | tr -cd '[:alnum:]' | tr '[:upper:]' '[:lower:]')
    [ -n "$tag" ] || tag=$(cut -c1-12 /etc/machine-id)
    mkdir -p /etc/systemd/system/user@.service.d
    printf '%s\n' "# hyprland-wsl: this distro's user sessions get a control group path of their own;" \
        "# the path below user.slice is shared by every WSL distro." \
        '[Service]' "Slice=wsl$tag-user-%i.slice" > /etc/systemd/system/user@.service.d/50-hyprland-wsl-slice.conf
    systemctl daemon-reload
    # The user's systemd session is kept running ("linger"), and the launcher starts it before
    # Hyprland. Without it nothing owned /run/user/<uid> while the desktop ran: the first login
    # as the user (a terminal window is enough) mounted a fresh one over the desktop's sockets,
    # and logging out removed it, which ended the desktop.
    if [ -n "$U" ]; then
        loginctl enable-linger "$U" && echo "the session of $U is kept running (linger)"
    fi
fi

# An Omarchy installed by tools/omarchy-wsl.sh gets the current version of its WSL settings
# and of the hooks that keep them through "omarchy update"; running install.sh again is how
# an existing Omarchy instance is brought up to date.
if [ $FAMILY = arch ] && pacman -Q omarchy >/dev/null 2>&1 && [ -f "$HERE/tools/omarchy-wsl-system.sh" ]; then
    step "Omarchy's settings for WSL"
    install -m 755 "$HERE/tools/omarchy-wsl-system.sh" $PREFIX/omarchy-wsl-system
    install -m 755 "$HERE/tools/omarchy-wsl-user.sh" $PREFIX/omarchy-wsl-user
    install -D -m 644 "$HERE/tools/99-hyprland-wsl-omarchy.hook" /etc/pacman.d/hooks/99-hyprland-wsl-omarchy.hook
    $PREFIX/omarchy-wsl-system
    [ -z "$U" ] || $PREFIX/omarchy-wsl-user "$H" "$U"
fi

step "Preparing the Hyprland config"
if [ -z "$U" ]; then
    echo "skipped until the user exists"
else
lua=$H/.config/hypr/hyprland.lua
conf=$H/.config/hypr/hyprland.conf
if [ -e "$lua" ] || [ -e "$conf" ]; then
    echo "existing config left untouched"
else
    # Hyprland writes its default config when asked to verify a missing one.
    tmp=$(mktemp -d) && chown "$U": "$tmp"
    sudo -u "$U" env -u WAYLAND_DISPLAY -u DISPLAY HOME="$H" XDG_RUNTIME_DIR="$tmp" \
        sh -c 'cd "$HOME" && Hyprland --verify-config' > /dev/null 2>&1 || true
    rm -rf "$tmp"
    if [ -f "$lua" ]; then
        # Hyprland 0.55 and later: Lua config.
        sed -i -e '/autogenerated = true/d' \
               -e 's/^local terminal    = "kitty"/local terminal    = "foot"/' \
               -e 's/^local mainMod = "SUPER".*/local mainMod = "ALT" -- ALT, because Windows keeps the Windows key for itself/' \
               -e 's/^    scale    = "auto",/    scale    = 1,/' "$lua"
        echo 'hl.on("hyprland.start", function() hl.exec_cmd(terminal) end)' >> "$lua"
        echo "wrote $lua (main modifier: ALT, terminal: foot)"
    elif [ -f "$conf" ]; then
        # Older Hyprland: hyprland.conf.
        sed -i -e '/^autogenerated *= *1/d' \
               -e 's/^\$terminal = kitty/$terminal = foot/' \
               -e 's/^\$mainMod = SUPER.*/$mainMod = ALT # ALT, because Windows keeps the Windows key for itself/' \
               -e 's/^monitor=,preferred,auto,auto/monitor=,preferred,auto,1/' "$conf"
        echo 'exec-once = $terminal' >> "$conf"
        echo "wrote $conf (main modifier: ALT, terminal: foot)"
    else
        die "Hyprland did not generate a config in $H/.config/hypr"
    fi
fi

fi

step "Done"
echo "Start Hyprland with:  hyprland-wsl"
echo "From Windows:         wsl -d ${WSL_DISTRO_NAME:-<distro>} hyprland-wsl"
echo "A menu of your distros for Windows: https://github.com/ItsOverk1ll/hyprland-wsl/tree/Launcher"
