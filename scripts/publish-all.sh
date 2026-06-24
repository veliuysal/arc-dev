#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.1.0}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

echo "==> Tagging and pushing arc-dev v${VERSION}"
git -C "$ROOT/arc-dev" tag -a "v${VERSION}" -m "Release v${VERSION}" 2>/dev/null || \
  git -C "$ROOT/arc-dev" tag -f "v${VERSION}" -m "Release v${VERSION}"
git -C "$ROOT/arc-dev" push origin "v${VERSION}"

echo
echo "==> Waiting for GitHub release assets"
release_url="https://github.com/veliuysal/arc-dev/releases/tag/v${VERSION}"
for _ in $(seq 1 60); do
  if curl -fsSL "https://api.github.com/repos/veliuysal/arc-dev/releases/tags/v${VERSION}" \
    | rg -q "arc-dev-${VERSION}-x86_64-apple-darwin.tar.gz"; then
    echo "Release assets are ready"
    break
  fi
  echo "  waiting..."
  sleep 15
done

echo
echo "==> Updating Homebrew and Scoop manifests"
bash "$ROOT/arc-dev/scripts/update-package-managers.sh" "$VERSION"

echo
echo "==> Pushing homebrew-tap"
git -C "$ROOT/homebrew-tap" add Formula/arc-dev.rb
git -C "$ROOT/homebrew-tap" commit -m "arc-dev ${VERSION}" || true
git -C "$ROOT/homebrew-tap" push origin main

echo
echo "==> Pushing scoop-bucket"
git -C "$ROOT/scoop-bucket" add arc-dev.json
git -C "$ROOT/scoop-bucket" commit -m "arc-dev ${VERSION}" || true
git -C "$ROOT/scoop-bucket" push origin main

echo
echo "Done."
echo "Cargo publish runs in GitHub Actions when CARGO_REGISTRY_TOKEN is configured."
echo "Release: ${release_url}"
