#!/bin/bash
# The system-wide part of fitting Omarchy to WSL. Safe to run any number of times: it only
# changes what is not already as it should be.
#
# tools/omarchy-wsl.sh runs it once at install and installs it as
# /opt/hyprland-wsl/omarchy-wsl-system, together with a pacman hook that runs it after every
# package transaction. "omarchy update" can then bring new packages and migrations without
# undoing any of this, and without stopping at things WSL does not have (a boot partition,
# btrfs snapshots, a kernel package).
#
#   --quiet   only report what was changed (the pacman hook uses this)

set -uo pipefail

QUIET=
[ "${1:-}" = --quiet ] && QUIET=1
say() { echo "hyprland-wsl: $*"; }

[ "$(id -u)" = 0 ] || { echo "run this as root" >&2; exit 1; }

# --- Services ---------------------------------------------------------------------------
# No login screen: hyprland-wsl starts the session. WSL does name resolution itself, and a
# firewall in the instance would stand between Windows and the session. WSL has no btrfs, so
# snapshots and their boot entries have nothing to work with.
for s in sddm.service ufw.service snapper-cleanup.timer snapper-timeline.timer \
         snapper-boot.timer limine-snapper-sync.service; do
    if systemctl is-enabled --quiet "$s" 2>/dev/null; then
        systemctl disable --now "$s" > /dev/null 2>&1 && say "disabled $s"
    fi
done
if [ "$(systemctl is-enabled systemd-resolved.service 2>/dev/null)" != masked ]; then
    systemctl disable --now systemd-resolved.service > /dev/null 2>&1
    systemctl mask systemd-resolved.service > /dev/null 2>&1 && say "masked systemd-resolved.service"
fi
command -v ufw > /dev/null && ufw status 2>/dev/null | grep -q 'Status: active' && ufw disable > /dev/null 2>&1

# NetworkManager only watches: WSL sets up the network and /etc/resolv.conf itself. Running,
# it lets Omarchy's bar and menus see the connection, which they otherwise show as missing.
# dns=none also keeps Omarchy's own DNS setting (20-omarchy-dns.conf) out of resolv.conf.
if [ -d /etc/NetworkManager ]; then
    nm=/etc/NetworkManager/conf.d/99-hyprland-wsl.conf
    want='# hyprland-wsl: WSL sets up the network and /etc/resolv.conf; NetworkManager only reports them.
[main]
dns=none
rc-manager=unmanaged
no-auto-default=*

[device]
keep-configuration=yes'
    if [ "$(cat "$nm" 2>/dev/null)" != "$want" ]; then
        mkdir -p "${nm%/*}"
        printf '%s\n' "$want" > "$nm"
        say "NetworkManager set to watch only"
        systemctl is-active --quiet NetworkManager.service && systemctl restart NetworkManager.service
    fi
    if ! systemctl is-enabled --quiet NetworkManager.service 2>/dev/null; then
        systemctl enable --now NetworkManager.service > /dev/null 2>&1 && say "enabled NetworkManager (watch only)"
    fi
fi

# --- Power off and reboot without a password -------------------------------------------
rule=/etc/polkit-1/rules.d/50-hyprland-wsl-power.rules
want='// hyprland-wsl: the session is not started through a login manager, so logind does not
// count it as an active local session, and power off and reboot would ask for a password
// that no prompt can show. Let the desktop'"'"'s users do both, as on a normal login.
polkit.addRule(function (action, subject) {
    if (/^org\.freedesktop\.login1\.(power-off|reboot)(-multiple-sessions|-ignore-inhibit)?$/.test(action.id) &&
        subject.isInGroup("wheel")) {
        return polkit.Result.YES;
    }
});'
if [ -d /etc/polkit-1/rules.d ] && [ "$(cat "$rule" 2>/dev/null)" != "$want" ]; then
    printf '%s\n' "$want" > "$rule"
    say "wrote $rule"
fi

# --- The file index skips the Windows drives -------------------------------------------
if [ -f /etc/updatedb.conf ] && ! grep -q '/mnt' /etc/updatedb.conf; then
    sed -i 's|^PRUNEPATHS = "|PRUNEPATHS = "/mnt |' /etc/updatedb.conf
    say "the file index (locate) skips /mnt"
fi

# --- No boot chain ----------------------------------------------------------------------
# WSL boots Microsoft's kernel from Windows. Omarchy's boot tools (Limine, mkinitcpio, the
# snapshot boot entries) would look for a boot partition that does not exist and fail, and a
# failing step stops "omarchy update" halfway. Their pacman hooks are kept from being
# installed, and their commands are replaced, first on PATH, by ones that do nothing.
noextract='NoExtract = usr/share/libalpm/hooks/*limine* usr/share/libalpm/hooks/*mkinitcpio* etc/pacman.d/hooks/90-mkinitcpio-install.hook'
if ! grep -qF '# hyprland-wsl: no boot chain in WSL' /etc/pacman.conf; then
    # Two appends right after [options]: the second lands above the first.
    sed -i -e "/^\[options\]/a $noextract" /etc/pacman.conf
    sed -i -e "/^\[options\]/a # hyprland-wsl: no boot chain in WSL" /etc/pacman.conf
    say "pacman no longer installs the boot loader's hooks"
fi
for h in /usr/share/libalpm/hooks/*limine*.hook /usr/share/libalpm/hooks/*mkinitcpio*.hook \
         /etc/pacman.d/hooks/90-mkinitcpio-install.hook; do
    [ -f "$h" ] && rm -f "$h" && say "removed $h"
done

stub() {    # stub NAME TEXT: a command in /usr/local/bin that takes precedence on PATH
    local f=/usr/local/bin/$1 body
    body="#!/bin/bash
# hyprland-wsl: $2
$3"
    if [ "$(cat "$f" 2>/dev/null)" != "$body" ]; then
        printf '%s\n' "$body" > "$f"
        chmod 755 "$f"
        say "installed $f"
    fi
}
for c in limine-install limine-update limine-mkinitcpio limine-snapper-sync limine-snapper-restore mkinitcpio; do
    stub "$c" "WSL boots Microsoft's kernel from Windows; there is no boot partition or boot image to update." 'exit 0'
done
# 127 is what "omarchy update" takes as "snapshots are deliberately not set up".
stub omarchy-snapshot "snapshots need a btrfs root, and WSL's disk is ext4. Export the instance instead: wsl --export <name> <file>.tar" \
'[ "${1:-}" = create ] && exit 127
echo "Snapshots are not available in WSL. Back up the instance from Windows: wsl --export <name> <file>.tar" >&2
exit 1'

# After an update Omarchy offers a reboot when the running kernel is not one of the installed
# kernel packages, which under WSL is always the case. This copy of its last update step
# leaves that check out and is otherwise Omarchy's own, made again after every update.
src=/usr/share/omarchy/bin/omarchy-update-restart
dst=/usr/local/bin/omarchy-update-restart
if [ -f $src ]; then
    if grep -q '^kernel_updated=true$' $src; then
        new=$(sed 's/^kernel_updated=true$/kernel_updated=false  # hyprland-wsl: WSL runs Microsoft'"'"'s kernel, which no package provides/' $src)
        if [ "$(cat $dst 2>/dev/null)" != "$new" ]; then
            printf '%s\n' "$new" > $dst
            chmod 755 $dst
            say "installed $dst (without the kernel check)"
        fi
    elif [ -f $dst ]; then
        # Omarchy changed the step; its own version is better than a stale copy.
        rm -f $dst
        say "warning: Omarchy's omarchy-update-restart changed; its kernel check may ask for a reboot after updates"
    fi
fi

[ -n "$QUIET" ] || say "Omarchy's system settings for WSL are in place"
exit 0
