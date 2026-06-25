#!/usr/bin/env bash
set -euo pipefail

REPO="${ARC_DEV_REPO:-veliuysal/arc-dev}"
VERSION="${1:-}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOMEBREW_FORMULA="${HOMEBREW_FORMULA:-$ROOT/../homebrew-tap/Formula/arc-dev.rb}"
SCOOP_MANIFEST="${SCOOP_MANIFEST:-$ROOT/../scoop-bucket/arc-dev.json}"

usage() {
  cat <<EOF
Update Homebrew and Scoop manifests with release asset checksums.

Usage:
  $0 <version>

Example:
  $0 0.1.1
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

python3 "$ROOT/scripts/update_package_managers.py" "$VERSION" \
  --repo "$REPO" \
  --homebrew-formula "$HOMEBREW_FORMULA" \
  --scoop-manifest "$SCOOP_MANIFEST"
