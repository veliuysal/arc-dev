#!/usr/bin/env bash
set -euo pipefail

REPO="${ARC_DEV_REPO:-veliuysal/arc-dev}"
VERSION="${ARC_DEV_VERSION:-latest}"
INSTALL_DIR="${ARC_DEV_INSTALL_DIR:-${HOME}/.local/bin}"

usage() {
  cat <<EOF
Install the arc-dev CLI from GitHub releases.

Environment variables:
  ARC_DEV_REPO         GitHub repository (default: veliuysal/arc-dev)
  ARC_DEV_VERSION      Release tag or "latest" (default: latest)
  ARC_DEV_INSTALL_DIR  Install directory (default: ~/.local/bin)
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

detect_target() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"

  case "$arch" in
    x86_64 | amd64) arch="x86_64" ;;
    arm64 | aarch64) arch="aarch64" ;;
    *)
      echo "unsupported architecture: $arch" >&2
      exit 1
      ;;
  esac

  case "$os" in
    darwin) echo "${arch}-apple-darwin" ;;
    linux) echo "${arch}-unknown-linux-gnu" ;;
    *)
      echo "unsupported operating system: $os" >&2
      exit 1
      ;;
  esac
}

resolve_version() {
  if [[ "$VERSION" != "latest" ]]; then
    echo "${VERSION#v}"
    return
  fi

  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name": "v\(.*\)".*/\1/p'
}

main() {
  local target version archive url tmpdir

  target="$(detect_target)"
  version="$(resolve_version)"
  archive="arc-dev-${version}-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/v${version}/${archive}"

  command -v curl >/dev/null || {
    echo "curl is required" >&2
    exit 1
  }

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  echo "Downloading arc-dev v${version} for ${target}"
  curl -fsSL "$url" -o "${tmpdir}/${archive}"
  tar xzf "${tmpdir}/${archive}" -C "$tmpdir"

  mkdir -p "$INSTALL_DIR"
  install -m 755 "${tmpdir}/arc-dev" "${INSTALL_DIR}/arc-dev"

  echo "Installed arc-dev to ${INSTALL_DIR}/arc-dev"
  if ! command -v arc-dev >/dev/null 2>&1; then
    echo "Add ${INSTALL_DIR} to your PATH if needed"
  fi
}

main "$@"
