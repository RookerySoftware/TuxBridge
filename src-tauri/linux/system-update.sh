#!/bin/sh
# Updates a distro's programs with its own package manager, without questions. Run as root:
#
#   system-update.sh check    prints how this system is updated: apt, pacman, dnf, zypper, apk,
#                             omarchy (which has its own tool, run by the user in a terminal)
#                             or none
#   system-update.sh run      updates everything; the last line says how many packages changed
#                             ("easywsl-updated: N"), when the package manager can tell
#
# Changed configuration files keep the system's current version (the new one is saved next to
# it), as a careful person would choose when asked.

set -eu

manager() {
    if command -v omarchy-update > /dev/null || ls -d /home/*/.local/share/omarchy/bin > /dev/null 2>&1; then echo omarchy
    elif command -v apt-get > /dev/null; then echo apt
    elif command -v pacman > /dev/null; then echo pacman
    elif command -v dnf > /dev/null; then echo dnf
    elif command -v zypper > /dev/null; then echo zypper
    elif command -v apk > /dev/null; then echo apk
    else echo none
    fi
}

case ${1:-} in
check)
    manager
    ;;
run)
    count=""
    case $(manager) in
    apt)
        export DEBIAN_FRONTEND=noninteractive NEEDRESTART_MODE=a
        apt-get update
        count=$(apt-get -s dist-upgrade | grep -c '^Inst ' || true)
        apt-get -y -o Dpkg::Options::=--force-confdef -o Dpkg::Options::=--force-confold dist-upgrade
        ;;
    pacman)
        # The keyring first: packages signed by new keys fail to install with an old keyring.
        pacman -Sy --noconfirm --needed archlinux-keyring
        if pacman -Si cachyos-keyring > /dev/null 2>&1; then pacman -S --noconfirm --needed cachyos-keyring; fi
        count=$(pacman -Qu 2> /dev/null | wc -l)
        pacman -Su --noconfirm
        ;;
    dnf)
        dnf -y upgrade --refresh
        ;;
    zypper)
        zypper --non-interactive refresh
        zypper --non-interactive update
        ;;
    apk)
        apk update
        count=$(apk -u list 2> /dev/null | wc -l)
        apk upgrade
        ;;
    omarchy)
        echo "Omarchy updates with its own tool: run omarchy-update in a terminal." >&2
        exit 4
        ;;
    *)
        echo "this system has no package manager Pengi knows" >&2
        exit 3
        ;;
    esac
    if command -v flatpak > /dev/null; then flatpak update -y --noninteractive || true; fi
    if [ -n "$count" ]; then echo "easywsl-updated: $count"; fi
    ;;
*)
    echo "usage: system-update.sh check | run" >&2
    exit 2
    ;;
esac
