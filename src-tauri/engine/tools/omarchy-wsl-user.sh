#!/bin/bash
# The per-user part of fitting Omarchy to WSL, for one home directory. Safe to run any number
# of times: it only changes what is not already as it should be.
#
# tools/omarchy-wsl.sh installs it as /opt/hyprland-wsl/omarchy-wsl-user and runs it for
# /etc/skel (new accounts) or the account it creates. It also adds an Omarchy post-update
# hook that runs it again after every "omarchy update", in case a migration changed these
# files back.
#
#   omarchy-wsl-user                as the user, for $HOME
#   omarchy-wsl-user HOME OWNER     as root, for another home (OWNER root: leave owners alone)

set -uo pipefail

h=${1:-$HOME}
owner=${2:-}
changed=()

# Omarchy starts apps through uwsm, which by default moves each new process into a scope of
# the user's systemd session. That move is refused here, because Hyprland was not started from
# inside that session, and nothing would open. Started as services, apps are created by the
# session itself and no move is needed.
envf=$h/.config/environment.d/50-hyprland-wsl.conf
if ! grep -qx 'UWSM_APP_UNIT_TYPE=service' "$envf" 2>/dev/null; then
    mkdir -p "${envf%/*}"
    echo 'UWSM_APP_UNIT_TYPE=service' >> "$envf"
    changed+=("$envf")
fi

# Chromium (and the web apps Omarchy builds on it) asks the GPU driver for a kind of buffer
# that Mesa's Direct3D 12 driver does not hand out, and exits. Without its own GPU process it
# draws on the CPU and Hyprland still composes on the GPU.
flags=$h/.config/chromium-flags.conf
if [ -f "$flags" ] && ! grep -q -- '--disable-gpu' "$flags"; then
    printf '%s\n' '# hyprland-wsl: Chromium cannot use the GPU in WSL (see the Omarchy guide)' '--disable-gpu' >> "$flags"
    changed+=("$flags")
fi

# Suspending would suspend the WSL instance's kernel, not the PC; Omarchy's System menu hides
# it when this file exists. Hibernate is hidden already (logind reports it as unavailable).
toggle=$h/.local/state/omarchy/toggles/suspend-off
if [ ! -e "$toggle" ]; then
    mkdir -p "${toggle%/*}"
    : > "$toggle"
    changed+=("$toggle")
fi

# Runs this again after every "omarchy update".
hook=$h/.config/omarchy/hooks/post-update.d/90-hyprland-wsl
want='# hyprland-wsl: puts back the WSL settings for this account, should an update have changed them.
if [ -x /opt/hyprland-wsl/omarchy-wsl-user ]; then /opt/hyprland-wsl/omarchy-wsl-user; fi'
if [ "$(cat "$hook" 2>/dev/null)" != "$want" ]; then
    mkdir -p "${hook%/*}"
    printf '%s\n' "$want" > "$hook"
    chmod 755 "$hook"
    changed+=("$hook")
fi

if [ -n "$owner" ] && [ "$owner" != root ]; then
    for f in "${changed[@]}"; do chown "$owner": "$f"; done
    chown "$owner": "$h/.config/environment.d" "$h/.local/state/omarchy/toggles" \
        "$h/.config/omarchy/hooks/post-update.d" 2> /dev/null
fi
for f in "${changed[@]}"; do echo "hyprland-wsl: updated $f"; done
exit 0
