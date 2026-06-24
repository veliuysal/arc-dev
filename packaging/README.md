# Publishing `arc-dev`

Repository: [`veliuysal/arc-dev`](https://github.com/veliuysal/arc-dev)

## Private source, public binaries

**Goal:** keep application source private, but let users install the CLI via Homebrew, Scoop, install script, and `cargo binstall`.

| Channel | Exposes source? | How it works |
|---------|-----------------|--------------|
| Homebrew | No | Downloads prebuilt `.tar.gz` from GitHub Releases |
| Scoop | No | Downloads prebuilt `.zip` from GitHub Releases |
| Install script | No | Downloads prebuilt binary from GitHub Releases |
| `cargo binstall` | No | Downloads prebuilt binary using `[package.metadata.binstall]` in `Cargo.toml` |
| `cargo publish` / `cargo install` | **Yes** | Publishes full source to crates.io — **do not use for private code** |

### Recommended repo layout

```text
arc-dev (PRIVATE)          ← full source, development, CI builds
  └── GitHub Actions uploads release binaries

arc-dev-releases (PUBLIC)  ← optional: releases-only public repo
  └── GitHub Releases with binaries only (no src/)

homebrew-tap (PUBLIC)      ← formula + checksums only
scoop-bucket (PUBLIC)      ← manifest + hash only
```

If the main repo is **private**, release asset URLs are not public. Use one of:

1. **Public releases repo** — CI uploads binaries there with a PAT
2. **Make only Releases public** — keep repo private, use a public CDN/S3 for binaries
3. **Public repo with no source** — push only README, install script, and release assets (no `src/`)

### crates.io note

If you already published `0.1.0` to crates.io, that source is public forever (you can [yank](https://doc.rust-lang.org/cargo/commands/cargo-yank.html) it to block new installs, but the tarball remains downloadable).

For private code going forward:

- **Do not** run `cargo publish`
- **Do not** add `CARGO_REGISTRY_TOKEN` to release CI
- Use **`cargo binstall arc-dev`** for Rust users (needs a crates.io entry with binstall metadata, or distribute via the install script)

To yank the public release:

```sh
cargo yank --vers 0.1.0 arc-dev
```

## One-time setup

1. Create GitHub repositories:
   - [`veliuysal/homebrew-tap`](https://github.com/new?repo=homebrew-tap)
   - [`veliuysal/scoop-bucket`](https://github.com/new?repo=scoop-bucket)
2. Push the local tap and bucket repos:

```sh
cd ../homebrew-tap
git push -u origin main

cd ../scoop-bucket
git push -u origin main
```

3. Add GitHub Actions secret to the **private** source repo:
   - `PACKAGE_MANAGERS_TOKEN` — GitHub PAT with `repo` scope for updating `homebrew-tap` and `scoop-bucket`

## Release checklist

1. Bump `version` in `Cargo.toml`, `packaging/homebrew/arc-dev.rb`, and `packaging/scoop/arc-dev.json`.
2. Commit and tag:

```sh
git tag v0.1.1
git push origin main
git push origin v0.1.1
```

3. GitHub Actions will:
   - Build release assets for macOS, Linux, and Windows
   - Upload them to GitHub Releases
   - Update and push Homebrew/Scoop manifests when `PACKAGE_MANAGERS_TOKEN` is set

4. Or run locally after the release finishes:

```sh
./scripts/update-package-managers.sh 0.1.1
cd ../homebrew-tap && git commit -am "arc-dev 0.1.1" && git push
cd ../scoop-bucket && git commit -am "arc-dev 0.1.1" && git push
```

## Install channels (for users)

| Channel | Command |
|---------|---------|
| Install script | `curl -fsSL https://raw.githubusercontent.com/veliuysal/arc-dev/main/scripts/install.sh \| bash` |
| Cargo binstall | `cargo binstall arc-dev` |
| Homebrew tap | `brew install veliuysal/tap/arc-dev` |
| Scoop bucket | `scoop install arc-dev` |

## Homebrew tap setup

Users install with:

```sh
brew tap veliuysal/tap
brew trust veliuysal/tap
brew install arc-dev
```

## Scoop bucket setup

Users install with:

```powershell
scoop bucket add veliuysal https://github.com/veliuysal/scoop-bucket
scoop install arc-dev
```
