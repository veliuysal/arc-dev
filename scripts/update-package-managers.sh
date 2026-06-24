#!/usr/bin/env bash
set -euo pipefail

REPO="${ARC_DEV_REPO:-veliuysal/arc-dev}"
VERSION="${1:-}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HOMEBREW_FORMULA="${HOMEBREW_FORMULA:-$ROOT/homebrew-tap/Formula/arc-dev.rb}"
SCOOP_MANIFEST="${SCOOP_MANIFEST:-$ROOT/scoop-bucket/arc-dev.json}"

usage() {
  cat <<EOF
Update Homebrew and Scoop manifests with release asset checksums.

Usage:
  $0 <version>

Example:
  $0 0.1.0
EOF
}

if [[ -z "$VERSION" ]]; then
  usage >&2
  exit 1
fi

if [[ ! -f "$HOMEBREW_FORMULA" ]]; then
  echo "Homebrew formula not found: $HOMEBREW_FORMULA" >&2
  exit 1
fi

if [[ ! -f "$SCOOP_MANIFEST" ]]; then
  echo "Scoop manifest not found: $SCOOP_MANIFEST" >&2
  exit 1
fi

declare -A ASSETS=(
  ["aarch64-apple-darwin"]="arc-dev-${VERSION}-aarch64-apple-darwin.tar.gz"
  ["x86_64-apple-darwin"]="arc-dev-${VERSION}-x86_64-apple-darwin.tar.gz"
  ["aarch64-unknown-linux-gnu"]="arc-dev-${VERSION}-aarch64-unknown-linux-gnu.tar.gz"
  ["x86_64-unknown-linux-gnu"]="arc-dev-${VERSION}-x86_64-unknown-linux-gnu.tar.gz"
  ["x86_64-pc-windows-msvc"]="arc-dev-${VERSION}-x86_64-pc-windows-msvc.zip"
)

echo "Fetching checksums for arc-dev v${VERSION}"
CHECKSUM_LINES=()
for target in "${!ASSETS[@]}"; do
  asset="${ASSETS[$target]}"
  url="https://github.com/${REPO}/releases/download/v${VERSION}/${asset}"
  echo "  ${asset}"
  checksum="$(curl -fsSL "$url" | shasum -a 256 | awk '{print $1}')"
  CHECKSUM_LINES+=("${target}=${checksum}")
done

export REPO VERSION HOMEBREW_FORMULA SCOOP_MANIFEST
export CHECKSUM_DATA="${CHECKSUM_LINES[*]}"

python3 <<'PY'
import os
import re
from pathlib import Path

version = os.environ["VERSION"]
repo = os.environ["REPO"]
formula = Path(os.environ["HOMEBREW_FORMULA"])
scoop = Path(os.environ["SCOOP_MANIFEST"])
checksums = dict(item.split("=", 1) for item in os.environ["CHECKSUM_DATA"].split())

text = formula.read_text()
text = re.sub(r'version "[^"]+"', f'version "{version}"', text, count=1)
text = re.sub(
    r"/releases/download/v[^/]+/",
    f"/releases/download/v{version}/",
    text,
)
text = re.sub(
    r"arc-dev-[^-]+-[^-]+-(aarch64-apple-darwin|x86_64-apple-darwin|aarch64-unknown-linux-gnu|x86_64-unknown-linux-gnu)\.tar\.gz",
    lambda match: f"arc-dev-{version}-{match.group(1)}.tar.gz",
    text,
)

for target, key in [
    ("aarch64-apple-darwin", "aarch64-apple-darwin"),
    ("x86_64-apple-darwin", "x86_64-apple-darwin"),
    ("aarch64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"),
    ("x86_64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"),
]:
    pattern = (
        rf'(url "https://github.com/{re.escape(repo)}/releases/download/v{re.escape(version)}/'
        rf'arc-dev-{re.escape(version)}-{target}\.tar\.gz"\n\s+sha256 ")[^"]+(")'
    )
    text = re.sub(pattern, rf'\1{checksums[key]}\2', text)

formula.write_text(text)

scoop_text = scoop.read_text()
scoop_text = re.sub(r'"version": "[^"]+"', f'"version": "{version}"', scoop_text, count=1)
scoop_text = re.sub(
    r'"url": "https://github.com/[^/]+/arc-dev/releases/download/v[^/]+/[^"]+"',
    f'"url": "https://github.com/{repo}/releases/download/v{version}/arc-dev-{version}-x86_64-pc-windows-msvc.zip"',
    scoop_text,
    count=1,
)
scoop_text = re.sub(
    r'"hash": "[^"]+"',
    f'"hash": "{checksums["x86_64-pc-windows-msvc"]}"',
    scoop_text,
    count=1,
)
scoop.write_text(scoop_text)
PY

echo "Updated:"
echo "  $HOMEBREW_FORMULA"
echo "  $SCOOP_MANIFEST"
