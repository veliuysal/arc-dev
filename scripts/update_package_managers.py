#!/usr/bin/env python3
"""Update Homebrew and Scoop manifests with release asset checksums."""

from __future__ import annotations

import argparse
import hashlib
import sys
import urllib.request
from pathlib import Path

DEFAULT_REPO = "veliuysal/arc-dev"

LINUX_AND_MAC_TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
)
WINDOWS_TARGET = "x86_64-pc-windows-msvc"


def asset_name(version: str, target: str) -> str:
    if target == WINDOWS_TARGET:
        return f"arc-dev-{version}-{target}.zip"
    return f"arc-dev-{version}-{target}.tar.gz"


def release_url(repo: str, version: str, target: str) -> str:
    return (
        f"https://github.com/{repo}/releases/download/v{version}/"
        f"{asset_name(version, target)}"
    )


def download_sha256(url: str) -> str:
    with urllib.request.urlopen(url) as response:
        digest = hashlib.sha256()
        while chunk := response.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def replace_quoted_value(text: str, prefix: str, value: str, start: int = 0) -> str:
    index = text.find(prefix, start)
    if index == -1:
        raise ValueError(f"could not find {prefix!r}")
    value_start = index + len(prefix)
    value_end = text.find('"', value_start)
    if value_end == -1:
        raise ValueError(f"could not find closing quote after {prefix!r}")
    return text[:value_start] + value + text[value_end:]


def update_homebrew(formula: Path, repo: str, version: str, checksums: dict[str, str]) -> None:
    text = formula.read_text()
    text = replace_quoted_value(text, 'version "', version, 0)

    for target in LINUX_AND_MAC_TARGETS:
        marker = f"-{target}.tar.gz\""
        marker_index = text.find(marker)
        if marker_index == -1:
            raise ValueError(f"could not find homebrew asset marker for {target}")

        url_start = text.rfind('url "', 0, marker_index)
        if url_start == -1:
            raise ValueError(f"could not find url field for {target}")
        url_line_end = text.find("\n", url_start)

        new_url_line = f'      url "{release_url(repo, version, target)}"'
        text = text[:url_start] + new_url_line + text[url_line_end:]

        marker_index = text.find(marker)
        sha_prefix = 'sha256 "'
        sha_start = text.find(sha_prefix, marker_index)
        if sha_start == -1:
            raise ValueError(f"could not find sha256 field for {target}")
        text = replace_quoted_value(text, sha_prefix, checksums[target], sha_start)

    formula.write_text(text)


def update_scoop(manifest: Path, repo: str, version: str, checksums: dict[str, str]) -> None:
    text = manifest.read_text()
    text = replace_quoted_value(text, '"version": "', version, 0)
    text = replace_quoted_value(
        text,
        '"url": "',
        release_url(repo, version, WINDOWS_TARGET),
        0,
    )
    text = replace_quoted_value(
        text,
        '"hash": "',
        checksums[WINDOWS_TARGET],
        0,
    )
    manifest.write_text(text)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="Release version without v prefix, e.g. 0.1.1")
    parser.add_argument(
        "--repo",
        default=DEFAULT_REPO,
        help=f"GitHub repository (default: {DEFAULT_REPO})",
    )
    parser.add_argument(
        "--homebrew-formula",
        type=Path,
        required=True,
        help="Path to arc-dev.rb",
    )
    parser.add_argument(
        "--scoop-manifest",
        type=Path,
        required=True,
        help="Path to arc-dev.json",
    )
    args = parser.parse_args()

    checksums: dict[str, str] = {}
    print(f"Fetching checksums for arc-dev v{args.version}")
    for target in (*LINUX_AND_MAC_TARGETS, WINDOWS_TARGET):
        url = release_url(args.repo, args.version, target)
        print(f"  {asset_name(args.version, target)}")
        checksums[target] = download_sha256(url)

    update_homebrew(args.homebrew_formula, args.repo, args.version, checksums)
    update_scoop(args.scoop_manifest, args.repo, args.version, checksums)

    print("Updated:")
    print(f"  {args.homebrew_formula}")
    print(f"  {args.scoop_manifest}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
