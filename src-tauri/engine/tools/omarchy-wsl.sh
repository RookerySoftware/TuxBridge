#!/bin/bash
# Installs Omarchy (https://omarchy.org) into a fresh Arch Linux WSL instance and puts
# hyprland-wsl on top, so that Omarchy's desktop runs in a window on Windows.
#
# Omarchy is normally installed from its own ISO onto a whole disk. This does what the ISO
# does after partitioning, minus everything about booting and real hardware:
#   packages from Omarchy's repository -> its system setup -> its per-user setup.
#
# Run as root in a NEW Arch instance, from a clone of this repository:
#   wsl --install archlinux --name omarchy
#   wsl -d omarchy -u root -- bash /mnt/c/path/to/hyprland-wsl/tools/omarchy-wsl.sh
# It stops once and asks for the instance to be restarted (exit code 75); run it again after
# that. windows\install-omarchy.ps1 does all of this in one go.
#
# By default no user is created: the first start shows Omarchy's own first-boot setup
# (keyboard, account, hostname, timezone), as a machine installed from the ISO "for another
# owner" does. See tools/omarchy-wsl-first-start.sh.
#
# Environment, for an install without that setup:
#   OMARCHY_USER       the user to create now
#   OMARCHY_PASSWORD   that user's password, also asked for by Omarchy's lock screen
#                      (default: the same as the user name; change it with passwd)

set -uo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
U=${OMARCHY_USER:-}
PASS=${OMARCHY_PASSWORD:-$U}
# Setup mode: the user is created by Omarchy's first-boot setup at the first start.
SETUP=0; [ -z "$U" ] && SETUP=1
KEY=40DFB630FF42BCFFB047046CF0134EE680CAC571      # Omarchy <pkgs@omarchy.org>
PROVISIONING=/var/lib/omarchy/provisioning

die() { echo "error: $*" >&2; exit 1; }
step() { printf '\n==> %s\n' "$*"; }

[ "$(id -u)" = 0 ] || die "run this as root"
. /etc/os-release
# A rerun finds the instance already calling itself Omarchy.
[ "$ID" = arch ] || [ "$ID" = omarchy ] || die "this needs plain Arch Linux, found '$ID'"
grep -qi microsoft /proc/version || die "this does not look like WSL2"
[ "$(ps -p 1 -o comm=)" = systemd ] || die "systemd is not running; enable it in /etc/wsl.conf ([boot] systemd=true) and restart the instance"
[ -f "$HERE/install.sh" ] || die "run this from a clone of the hyprland-wsl repository"

step "Omarchy's package repository"
pacman-key --init
pacman-key --populate
[ -f /etc/pacman.conf.arch ] || cp /etc/pacman.conf /etc/pacman.conf.arch
[ -f /etc/pacman.d/mirrorlist.arch ] || cp /etc/pacman.d/mirrorlist /etc/pacman.d/mirrorlist.arch
# Omarchy pairs its own repository with a snapshot of Arch's, so that the two always fit.
echo 'Server = https://stable-mirror.omarchy.org/$repo/os/$arch' > /etc/pacman.d/mirrorlist
if ! grep -q '^\[omarchy\]' /etc/pacman.conf; then
    awk '/^\[core\]/ && !done { print "[omarchy]\nServer = https://pkgs.omarchy.org/stable/$arch\n"; done=1 } { print }' \
        /etc/pacman.conf.arch > /etc/pacman.conf
fi
pacman-key --recv-keys $KEY --keyserver keys.openpgp.org ||
    pacman-key --recv-keys $KEY --keyserver keyserver.ubuntu.com || die "could not fetch Omarchy's signing key"
pacman-key --lsign-key $KEY

step "Bringing the system to Omarchy's snapshot"
# -uu: the snapshot can be older than what the WSL image shipped with.
OMARCHY_ALLOW_DIRECT_PACMAN=1 pacman -Syyuu --noconfirm || die "system sync failed"
pacman -S --noconfirm --needed omarchy-keyring sudo || die "could not install omarchy-keyring"

step "Omarchy's settings, the user, Omarchy itself"
# omarchy-settings fills /etc/skel, so it has to be installed before the user is created.
pacman -S --noconfirm --needed omarchy-settings || die "could not install omarchy-settings"
if [ $SETUP = 1 ]; then
    echo "no user yet: Omarchy's setup creates it at the first start"
elif ! id "$U" > /dev/null 2>&1; then
    useradd -m -G wheel -s /bin/bash "$U" || die "could not create user $U"
    echo "$U:$PASS" | chpasswd
    echo "created user $U"
fi
# Omarchy's setup calls sudo as the user. Removed again at the end.
if [ $SETUP = 0 ]; then
    echo "$U ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/00-omarchy-wsl-install
    chmod 440 /etc/sudoers.d/00-omarchy-wsl-install
fi
pacman -S --noconfirm --needed omarchy || die "could not install omarchy"
echo "Omarchy $(pacman -Q omarchy | cut -d" " -f2)"

step "Omarchy's base packages"
# The list the ISO installs. Its second list (kernel, bootloader, drivers) is left out:
# WSL boots Microsoft's kernel and has no bootloader.
mapfile -t listed < <(sed -e 's/[[:space:]]*#.*$//' -e '/^[[:space:]]*$/d' /usr/share/omarchy/install/omarchy-base.packages)
want=() skipped=()
for p in "${listed[@]}"; do
    if pacman -Si "$p" > /dev/null 2>&1 || pacman -Sg "$p" > /dev/null 2>&1; then want+=("$p"); else skipped+=("$p"); fi
done
[ ${#skipped[@]} = 0 ] || echo "not in any repository, skipped: ${skipped[*]}"
pacman -S --noconfirm --needed "${want[@]}" || die "could not install the base packages"

step "Omarchy's system setup"
export OMARCHY_INSTALL_USER="$U" OMARCHY_FIRST_INSTALL=1 OMARCHY_UPGRADE=0
export OMARCHY_PATH=/usr/share/omarchy OMARCHY_INSTALL=/usr/share/omarchy/install
export OMARCHY_INSTALL_LOG_FILE=/var/log/omarchy-install.log
export PATH="$OMARCHY_PATH/bin:$PATH"
# Snapshots need btrfs; this makes the step write its config without creating any.
export OMARCHY_SNAPPER_CONFIGURE_TEST=1
# The Arch WSL image masks systemd-resolved, because WSL writes /etc/resolv.conf itself.
# Omarchy's setup enables it and stops if it cannot, so it is unmasked for the run.
systemctl unmask systemd-resolved.service > /dev/null 2>&1
: > $OMARCHY_INSTALL_LOG_FILE
(
    set +e
    source "$OMARCHY_INSTALL/helpers/logging.sh"
    start_install_log
    source "$OMARCHY_INSTALL/config/all.sh"
    # Of the hardware steps only the package-manager one applies; the rest are fixes for
    # particular laptops, GPUs and network cards.
    run_logged "$OMARCHY_INSTALL/hardware/pacman.sh"
    source "$OMARCHY_INSTALL/login/all.sh"
    run_logged "$OMARCHY_INSTALL/post-install/pacman.sh"
    run_logged "$OMARCHY_INSTALL/post-install/udev.sh"
    stop_install_log
) > /dev/null 2>&1
# The post-install step replaced pacman.conf with Omarchy's, which names one more repository.
pacman -Sy > /dev/null || die "could not refresh the package databases"
if grep -q 'Failed:' $OMARCHY_INSTALL_LOG_FILE; then
    echo "these steps failed (see $OMARCHY_INSTALL_LOG_FILE):"
    grep 'Failed:' $OMARCHY_INSTALL_LOG_FILE | sed 's/.*Failed: /  /'
fi
echo "$(grep -c 'Completed:' $OMARCHY_INSTALL_LOG_FILE) steps completed"
# Omarchy's last post-install step indexes every mounted file system for "locate". Under WSL
# that would be every Windows drive, so /mnt is excluded first.
if [ -f /etc/updatedb.conf ] && ! grep -q '/mnt' /etc/updatedb.conf; then
    sed -i 's|^PRUNEPATHS = "|PRUNEPATHS = "/mnt |' /etc/updatedb.conf
fi
updatedb --add-prunepaths="/mnt /.snapshots" 2>/dev/null || true

step "Adjusting for WSL"
# The WSL settings live in two scripts that are installed and run again after updates (from a
# pacman hook and an Omarchy post-update hook), so that "omarchy update" keeps them.
mkdir -p /opt/hyprland-wsl
install -m 755 "$HERE/tools/omarchy-wsl-system.sh" /opt/hyprland-wsl/omarchy-wsl-system
install -m 755 "$HERE/tools/omarchy-wsl-user.sh" /opt/hyprland-wsl/omarchy-wsl-user
install -D -m 644 "$HERE/tools/99-hyprland-wsl-omarchy.hook" /etc/pacman.d/hooks/99-hyprland-wsl-omarchy.hook
/opt/hyprland-wsl/omarchy-wsl-system

# Per-user settings WSL needs, written into a home directory. In setup mode they go into
# /etc/skel, from which Omarchy's setup creates the new user's home.
wsl_user_settings() {    # wsl_user_settings HOME OWNER
    local h=$1 owner=$2
    /opt/hyprland-wsl/omarchy-wsl-user "$h" "$owner"
    # Omarchy assumes a high-density laptop screen and draws everything twice the size. The
    # session here has the size the launcher is given, so start at scale 1; change it from
    # Omarchy's menu (Setup > Monitors) or with the launcher's --scale. Only at install: the
    # user may change it later.
    if [ -f "$h/.config/hypr/monitors.lua" ]; then
        sed -i -e 's/^local omarchy_monitor_scale = .*/local omarchy_monitor_scale = 1/' \
            -e 's/^local omarchy_gdk_scale = .*/local omarchy_gdk_scale = 1/' "$h/.config/hypr/monitors.lua"
    fi
    return 0
}

if [ $SETUP = 1 ]; then
    wsl_user_settings /etc/skel root
    echo "WSL settings for new users written to /etc/skel"
else
    # Omarchy's desktop shell and its way of starting apps need the user's systemd session,
    # which normally comes with logging in.
    loginctl enable-linger "$U"
    # The packages just installed replaced systemd under the running one; without this the
    # user's session cannot be started until the instance is restarted.
    systemctl daemon-reexec
    systemctl start "user@$(id -u "$U").service"
    for _ in $(seq 50); do [ -S "/run/user/$(id -u "$U")/bus" ] && break; sleep 0.2; done
    if [ ! -S "/run/user/$(id -u "$U")/bus" ]; then
        cat >&2 <<EOF

The instance has to be restarted before the rest can run: the packages just installed
replaced systemd, and the user's session does not start under the old one.
In PowerShell:

    wsl -t ${WSL_DISTRO_NAME:-<instance>}

Then run this script again. It picks up where it stopped. (windows\\install-omarchy.ps1 does
both by itself.)
EOF
        exit 75
    fi
    # "wsl -d <instance>" should open as the user, not as root.
    if ! grep -q '^\[user\]' /etc/wsl.conf 2>/dev/null; then
        printf '\n[user]\ndefault=%s\n' "$U" >> /etc/wsl.conf
    fi

    step "Omarchy's per-user setup"
    R=/run/user/$(id -u "$U")
    sudo -u "$U" -i env OMARCHY_PATH=/usr/share/omarchy PATH="/usr/share/omarchy/bin:/usr/local/bin:/usr/bin" \
        XDG_RUNTIME_DIR="$R" DBUS_SESSION_BUS_ADDRESS="unix:path=$R/bus" \
        omarchy-provision-user --force || die "Omarchy's per-user setup failed"
    # A first install counts every shipped migration as already applied.
    sudo -u "$U" bash -c 'mkdir -p ~/.local/state/omarchy/migrations
        for m in /usr/share/omarchy/migrations/*.sh; do [ -f "$m" ] && touch ~/.local/state/omarchy/migrations/"$(basename "$m")"; done'
    wsl_user_settings "/home/$U" "$U"
    systemctl restart "user@$(id -u "$U").service"
fi

step "hyprland-wsl"
if [ $SETUP = 1 ]; then
    HYPR_USER=- bash "$HERE/install.sh" || die "the hyprland-wsl installer failed"
    # The first start runs Omarchy's first-boot setup in the terminal. Omarchy would run it on
    # the console at boot through its own service; that stays off, WSL has no console.
    mkdir -p /etc/hyprland-wsl $PROVISIONING
    install -m 755 "$HERE/tools/omarchy-wsl-first-start.sh" /etc/hyprland-wsl/first-start
    touch $PROVISIONING/pending
    systemctl disable omarchy-provision-owner.service > /dev/null 2>&1 || true
else
    HYPR_USER=$U bash "$HERE/install.sh" || die "the hyprland-wsl installer failed"
fi
rm -f /etc/sudoers.d/00-omarchy-wsl-install

step "Done"
if [ $SETUP = 1 ]; then
    cat <<EOF
Omarchy $(pacman -Q omarchy | cut -d" " -f2) is installed. The first start opens Omarchy's setup, where you choose the
keyboard, your account, the hostname and the timezone:

    windows\\hyprland-wsl.cmd -Distro ${WSL_DISTRO_NAME:-<instance>}

Restart the instance first if this script was just run (wsl -t ${WSL_DISTRO_NAME:-<instance>}).
Update with "omarchy update", not with pacman -Syu.
EOF
    exit 0
fi
cat <<EOF
Omarchy $(pacman -Q omarchy | cut -d" " -f2) is installed for user "$U", password "$PASS" (change it: passwd $U).
Omarchy's keys use the Windows key, which only reaches it through the Windows-side launcher
in full screen:

    windows\\hyprland-wsl.cmd -Distro ${WSL_DISTRO_NAME:-<instance>}

Then: Super+Return opens a terminal, Super+Space the Omarchy menu, Super+K the key list.
Update with "omarchy update", not with pacman -Syu.
EOF
