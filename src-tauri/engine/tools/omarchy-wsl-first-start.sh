#!/bin/bash
# First start of an Omarchy instance installed by tools/omarchy-wsl.sh in setup mode, installed
# as /etc/hyprland-wsl/first-start and run by "hyprland-wsl first-start" (the Windows-side
# launcher calls it before every start; it removes itself once the setup is done).
#
# Runs Omarchy's own first-boot setup, the one a machine installed "for another owner" shows on
# its first boot: the greeting, keyboard, account, hostname and timezone, then the account is
# created and set up. Afterwards the parts WSL needs are done for that account.

set -uo pipefail
PROVISIONING=/var/lib/omarchy/provisioning
SELF=/etc/hyprland-wsl/first-start

[ "$(id -u)" = 0 ] || { echo "first-start: run as root" >&2; exit 1; }
[ -t 0 ] || { echo "first-start: needs a terminal; run 'hyprland-wsl first-start' in one" >&2; exit 1; }

if [ -f $PROVISIONING/pending ]; then
    # Omarchy's first-boot setup. Its console-only parts (fonts, keymap on the console) switch
    # themselves off in a terminal. On failure it offers a retry or a root shell.
    TERM=${TERM:-xterm-256color} /usr/share/omarchy/bin/omarchy-provision-owner
    clear
fi
if [ -f $PROVISIONING/pending ]; then
    echo "Omarchy's setup did not finish. Start it again to retry." >&2
    exit 1
fi

# The account the setup created: the regular user in group wheel.
U=
while IFS=: read -r name _ uid _; do
    [ "$uid" -ge 1000 ] && [ "$uid" -lt 60000 ] && id -nG "$name" | grep -qw wheel && { U=$name; break; }
done < /etc/passwd
[ -n "$U" ] || { echo "first-start: no account found after Omarchy's setup" >&2; exit 1; }

echo "HYPR_USER=$U" > /etc/hyprland-wsl.conf
# Omarchy's shell and app launching need the user's systemd session without a login.
loginctl enable-linger "$U"
systemctl start "user@$(id -u "$U").service" 2> /dev/null || true

# WSL would otherwise reset these at every start: the user it opens as, the hostname (from
# Windows) and the timezone (from Windows). Keep what was chosen in the setup.
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
set_wsl_conf network hostname "$(cat /etc/hostname 2> /dev/null || echo omarchy)"
set_wsl_conf time useWindowsTimezone false

rm -f "$SELF"
echo "Setup finished for $U. Starting Omarchy..."
