#!/bin/bash
# Builds the patched aquamarine (patches/headless-drm*.patch) for the aquamarine that is
# installed, into /opt/hyprland-wsl/aquamarine. Does nothing when that copy is already current.
#
# install.sh runs it, and installs it as /opt/hyprland-wsl/build-aquamarine together with the
# patches, plus a package manager hook that runs it after every update: a system update that
# brings a new aquamarine then also brings a matching patched copy, and the launcher does not
# refuse to start. The launcher runs it too when it finds the copy out of date.
#
#   --hook   called from a package manager hook: never fails the transaction, only warns

set -euo pipefail

PREFIX=/opt/hyprland-wsl
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# From a clone the patches are next to tools/; installed, they are in $PREFIX/patches.
PATCHES=$HERE/patches
[ -d "$PATCHES" ] || PATCHES=$HERE/../patches
HOOK=
[ "${1:-}" = --hook ] && HOOK=1

fail() {
    if [ -n "$HOOK" ]; then
        echo "hyprland-wsl: warning: $*" >&2
        echo "hyprland-wsl: the desktop will not start until this works; run $PREFIX/build-aquamarine as root to retry" >&2
        exit 0
    fi
    echo "error: $*" >&2
    exit 1
}

[ "$(id -u)" = 0 ] || fail "building the patched aquamarine needs root"
ls "$PATCHES"/headless-drm*.patch >/dev/null 2>&1 || fail "no patches found in $PATCHES"

# The patched copy has to be the exact version Hyprland was built against, so the version
# comes from the installed library's file name (libaquamarine.so.X.Y.Z).
syslib=$(ldconfig -p | awk '/libaquamarine\.so\.[0-9]+ / { print $NF; exit }')
# In a package manager hook the library may not be in ld.so's cache yet.
[ -n "$syslib" ] || { ldconfig; syslib=$(ldconfig -p | awk '/libaquamarine\.so\.[0-9]+ / { print $NF; exit }'); }
[ -n "$syslib" ] || fail "libaquamarine is not installed; the hyprland package should have pulled it in"
srcver=$(basename "$(readlink -f "$syslib")")
srcver=${srcver#libaquamarine.so.}
stamp="$srcver $(cat "$PATCHES"/headless-drm*.patch | sha256sum | cut -d' ' -f1)"
if [ "$(cat $PREFIX/aquamarine/STAMP 2>/dev/null)" = "$stamp" ]; then
    [ -n "$HOOK" ] || echo "already built for aquamarine $srcver"
    exit 0
fi

echo "hyprland-wsl: building the patched aquamarine $srcver (a minute or two)"
build=$(mktemp -d)
trap 'rm -rf "$build"' EXIT
curl -fsSL "https://github.com/hyprwm/aquamarine/archive/refs/tags/v$srcver.tar.gz" | tar xz -C "$build" ||
    fail "could not download aquamarine $srcver from GitHub"
src=$build/aquamarine-$srcver
# The patch exists in one variant per layout of the patched file; use the one that fits.
applied=
for p in "$PATCHES"/headless-drm*.patch; do
    if patch -d "$src" -p1 --dry-run -s -f < "$p" >/dev/null 2>&1; then
        patch -d "$src" -p1 -s < "$p"
        applied=$(basename "$p")
        break
    fi
done
[ -n "$applied" ] || fail "no patch in $PATCHES applies to aquamarine $srcver; update hyprland-wsl"
echo "aquamarine $srcver, patched with $applied"
{
    cmake -S "$src" -B "$src/build" -G Ninja -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX=$build/out -DCMAKE_INSTALL_LIBDIR=lib &&
    cmake --build "$src/build" &&
    cmake --install "$src/build"
} > "$build/log" 2>&1 || { tail -n 20 "$build/log" >&2; fail "building aquamarine $srcver failed"; }
# Swapped in only once it is complete, so a failed build keeps the old copy.
echo "$srcver" > "$build/out/VERSION"
echo "$stamp" > "$build/out/STAMP"
rm -rf $PREFIX/aquamarine.old
[ -d $PREFIX/aquamarine ] && mv $PREFIX/aquamarine $PREFIX/aquamarine.old
mkdir -p $PREFIX
mv "$build/out" $PREFIX/aquamarine
rm -rf $PREFIX/aquamarine.old
echo "hyprland-wsl: patched aquamarine $srcver is ready"
