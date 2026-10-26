#!/bin/bash
# First setup of a distro that windows/new-distro.ps1 has just created, run as root inside it:
# makes the account (with sudo), sets the hostname and the default user. install.sh runs after.
#
#   NEW_USER=name NEW_HOSTNAME=host [NEW_MINT=1] bash new-distro-setup.sh < password
#
# The password comes on stdin so that it never shows in a process list. NEW_MINT=1 first turns
# Linux Mint's Docker image (which still identifies as Ubuntu) into Linux Mint.
set -euo pipefail
U=${NEW_USER:?}; HOST=${NEW_HOSTNAME:?}
IFS= read -r PW || true
PW=${PW%$'\r'}    # PowerShell ends what it pipes with CRLF
[ -n "$PW" ] || { echo "no password given" >&2; exit 1; }

step() { printf '\n==> %s\n' "$*"; }

if [ -f /etc/arch-release ]; then
    step "Preparing pacman"
    pacman-key --init > /dev/null
    pacman-key --populate > /dev/null
    # A full upgrade, as Arch requires; install.sh's own one is then quick.
    pacman -Syu --noconfirm --needed sudo
    GROUP=wheel
    echo '%wheel ALL=(ALL:ALL) ALL' > /etc/sudoers.d/10-wheel
    chmod 440 /etc/sudoers.d/10-wheel
else
    export DEBIAN_FRONTEND=noninteractive
    # Docker images leave apt's download folder owned by root, which apt's sandbox user cannot use.
    chmod 755 /var/lib/apt/lists/partial 2> /dev/null && chown _apt /var/lib/apt/lists/partial 2> /dev/null || true
    step "Updating the package lists"
    apt-get update
    if [ "${NEW_MINT:-}" = 1 ]; then
        step "Installing Linux Mint's base packages"
        mintbase=$(apt-cache madison base-files | awk '/mint/ {print $3; exit}')
        apt-get install -y --allow-downgrades --no-install-recommends \
            base-files="$mintbase" mintsystem mintsources mint-mirrors
    fi
    apt-get install -y sudo passwd
    GROUP=sudo
fi

step "Creating the account $U"
if id "$U" > /dev/null 2>&1; then
    usermod -aG "$GROUP" "$U"
else
    useradd -m -s /bin/bash -G "$GROUP" "$U"
fi
printf '%s:%s\n' "$U" "$PW" | chpasswd
echo "$U can use sudo with that password"

set_wsl_conf() {    # set_wsl_conf SECTION KEY VALUE
    local f=/etc/wsl.conf
    touch $f
    if ! grep -q "^\[$1\]" $f; then
        printf '\n[%s]\n%s=%s\n' "$1" "$2" "$3" >> $f
    elif awk -v s="[$1]" -v k="$2" '$0 == s { in_s = 1; next } /^\[/ { in_s = 0 } in_s && index($0, k "=") == 1 { found = 1 } END { exit !found }' $f; then
        awk -v s="[$1]" -v k="$2" -v v="$3" '$0 == s { in_s = 1; print; next } /^\[/ { in_s = 0 } in_s && index($0, k "=") == 1 { print k "=" v; next } { print }' $f > $f.new && mv $f.new $f
    else
        awk -v s="[$1]" -v k="$2" -v v="$3" '{ print } $0 == s { print k "=" v }' $f > $f.new && mv $f.new $f
    fi
}
set_wsl_conf user default "$U"
set_wsl_conf network hostname "$HOST"
echo "$HOST" > /etc/hostname
echo "hostname $HOST, default user $U (both take effect when the distro next starts)"
