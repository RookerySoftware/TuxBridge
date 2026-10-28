"""Puts a TuxBridge installer on tuxbridge.com: copies it into the website repository's download/
folder and points the site at it (the Download button, the version, size and checksum, and the
"latest" redirect). Run by .github/workflows/publish-to-site.yml when a release is published.

    python publish_to_site.py --site <website checkout> --installer <setup.exe> --version 0.2.0 [--prerelease]

Keeps the newest few installers so links to recent versions keep working; older ones are removed
to keep the website repository small.
"""

import argparse
import hashlib
import re
import shutil
from pathlib import Path

KEEP = 3
NAME = re.compile(r"TuxBridge-(\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?)-setup\.exe")


def version_key(version: str):
    core, _, pre = version.partition("-")
    # A pre-release (0.2.0-beta.1) sorts before its release (0.2.0).
    return tuple(int(p) for p in core.split(".")), pre == "", pre


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--site", type=Path, required=True)
    parser.add_argument("--installer", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--prerelease", action="store_true")
    args = parser.parse_args()

    version = args.version.removeprefix("v")
    name = f"TuxBridge-{version}-setup.exe"
    downloads = args.site / "download"
    downloads.mkdir(exist_ok=True)
    target = downloads / name
    shutil.copyfile(args.installer, target)

    data = target.read_bytes()
    sha = hashlib.sha256(data).hexdigest()
    size = f"{len(data) / 1024 / 1024:.1f} MB"
    stage = " · pre-release" if args.prerelease else ""

    # The page: every mention of an installer name, the version line, and the checksum.
    page_path = args.site / "index.html"
    page = page_path.read_text(encoding="utf-8")
    before = page
    page = NAME.sub(name, page)
    page = re.sub(r"Version [^<·]+?(?: · pre-release)? · [\d.]+ MB", f"Version {version}{stage} · {size}", page)
    page = re.sub(r"(<details class=\"checksum\">[\s\S]*?<code>)[0-9a-f]{64}(</code>)", rf"\g<1>{sha}\g<2>", page)
    if page == before and name not in before:
        raise SystemExit("index.html didn't change: its download section may have been rewritten; update the patterns")
    page_path.write_text(page, encoding="utf-8")

    # The "latest" link.
    redirects_path = args.site / "_redirects"
    redirects = redirects_path.read_text(encoding="utf-8")
    redirects = re.sub(r"(/download/TuxBridge-setup\.exe\s+)/download/\S+", rf"\g<1>/download/{name}", redirects)
    redirects_path.write_text(redirects, encoding="utf-8")

    # Only the newest few installers stay.
    installers = sorted(
        (p for p in downloads.glob("TuxBridge-*-setup.exe") if NAME.fullmatch(p.name)),
        key=lambda p: version_key(NAME.fullmatch(p.name).group(1)),
        reverse=True,
    )
    for old in installers[KEEP:]:
        if old.name != name:
            old.unlink()
            print(f"removed {old.name}")

    print(f"{name}: {size}, sha256 {sha}")


if __name__ == "__main__":
    main()
