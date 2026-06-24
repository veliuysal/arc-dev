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
declare -A CHECKSUMS=()
for target in "${!ASSETS[@]}"; do
  asset="${ASSETS[$target]}"
  url="https://github.com/${REPO}/releases/download/v${VERSION}/${asset}"
  echo "  ${asset}"
  CHECKSUMS[$target]="$(curl -fsSL "$url" | shasum -a 256 | awk '{print $1}')"
done

json_entries=()
for target in "${!CHECKSUMS[@]}"; do
  json_entries+=("\"${target}\": \"${CHECKSUMS[$target]}\"")
done
CHECKSUM_JSON="{ $(IFS=','; echo "${json_entries[*]}") }"

export REPO VERSION HOMEBREW_FORMULA SCOOP_MANIFEST CHECKSUM_JSON
python3 - "$HOMEBREW_FORMULA" "$SCOOP_MANIFEST" <<'PY'
import json
import os
import re
import sys
from pathlib import Path

formula = Path(sys.argv[1])
scoop = Path(sys.argv[2])
version = os.environ["VERSION"]
repo = os.environ["REPO"]
checksums = json.loads(os.environ["CHECKSUM_JSON"])

text = formula.read_text()
text = re.sub(r'version "[^"]+"', f'version "{version}"', text, count=1)
text = re.sub(
    r"/releases/download/v[^/]+/",
    f"/releases/download/v{version}/",
    text,
)
text = re.sub(
    r"arc-dev-[\d.]+-(aarch64-apple-darwin|x86_64-apple-darwin|aarch64-unknown-linux-gnu|x86_64-unknown-linux-gnu)\.tar\.gz",
    lambda match: f"arc-dev-{version}-{match.group(1)}.tar.gz",
    text,
)

for target in (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
):
    marker = f'arc-dev-{version}-{target}.tar.gz"'
    start = text.find(marker)
    if start == -1:
        raise SystemExit(f"could not find homebrew asset marker for {target}")
    sha_start = text.find('sha256 "', start)
    if sha_start == -1:
        raise SystemExit(f"could not find sha256 field for {target}")
    sha_value_start = sha_start + len('sha256 "')
    sha_end = text.find('"', sha_value_start)
    text = text[:sha_value_start] + checksums[target] + text[sha_end:]

formula.write_text(text)

scoop_text = scoop.read_text()
scoop_text = re.sub(r'"version": "[^"]+"', f'"version": "{version}"', scoop_text, count=1)
scoop_text = re.sub(
    r'"url": "https://github.com/[^/]+/arc-dev/releases/download/v[^/]+/[^"]+"',
    f'"url": "https://github.com/{repo}/releases/download/v{version}/arc-dev-{version}-x86_64-pc-windows-msvc.zip"',
    scoop_text,
    count=1,
)
hash_start = scoop_text.find('"hash": "')
if hash_start == -1:
    raise SystemExit("could not find scoop hash field")
hash_value_start = hash_start + len('"hash": "')
hash_end = scoop_text.find('"', hash_value_start)
scoop_text = (
    scoop_text[:hash_value_start]
    + checksums["x86_64-pc-windows-msvc"]
    + scoop_text[hash_end:]
)
scoop.write_text(scoop_text)
PY

echo "Updated:"
echo "  $HOMEBREW_FORMULA"
echo "  $SCOOP_MANIFEST"
