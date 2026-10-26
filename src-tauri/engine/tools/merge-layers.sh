#!/bin/bash
# Flatten the layers of a Docker image into one rootfs tarball for "wsl --import".
# usage: merge-layers.sh <dir with layerNN.tar.gz> <output tar.gz>
set -e
IN=$1; OUT=$2
ROOT=$(mktemp -d /var/tmp/rootfs.XXXXXX)
chmod 755 "$ROOT"   # mktemp makes it 0700, which would become the mode of / in the distro
for l in "$IN"/layer*.tar.gz; do
    # Whiteouts: ".wh..wh..opq" empties its directory, ".wh.NAME" deletes NAME.
    tar -tzf "$l" | grep -E '(^|/)\.wh\.' | while IFS= read -r w; do
        d=$(dirname "$w"); b=$(basename "$w")
        if [ "$b" = .wh..wh..opq ]; then
            find "$ROOT/$d" -mindepth 1 -delete 2>/dev/null || true
        else
            rm -rf "$ROOT/$d/${b#.wh.}"
        fi
    done
    tar -xzf "$l" -C "$ROOT" --numeric-owner --exclude='.wh.*'
    echo "applied $(basename "$l")"
done
tar -czf "$OUT" -C "$ROOT" --numeric-owner .
rm -rf "$ROOT"
ls -la "$OUT"
