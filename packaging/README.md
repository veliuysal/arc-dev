# Publishing `arc-dev`

Repository: [`veliuysal/arc-dev`](https://github.com/veliuysal/arc-dev)

## One-time setup

1. Create GitHub repositories:
   - [`veliuysal/homebrew-tap`](https://github.com/new?repo=homebrew-tap)
   - [`veliuysal/scoop-bucket`](https://github.com/new?repo=scoop-bucket)
2. Push the local tap and bucket repos from the parent `Arc/` directory:

```sh
cd ../homebrew-tap
git init
git add .
git commit -m "Initial homebrew tap"
git branch -M main
git remote add origin https://github.com/veliuysal/homebrew-tap.git
git push -u origin main

cd ../scoop-bucket
git init
git add .
git commit -m "Initial scoop bucket"
git branch -M main
git remote add origin https://github.com/veliuysal/scoop-bucket.git
git push -u origin main
```

3. Add GitHub Actions secrets to `arc-dev`:
   - `CARGO_REGISTRY_TOKEN` — create at [crates.io/settings/tokens](https://crates.io/settings/tokens)
   - `PACKAGE_MANAGERS_TOKEN` — GitHub PAT with `repo` scope for updating `homebrew-tap` and `scoop-bucket`

## Release checklist

1. Bump `version` in `Cargo.toml`, `packaging/homebrew/arc-dev.rb`, and `packaging/scoop/arc-dev.json`.
2. Commit and tag:

```sh
git tag v0.1.0
git push origin main
git push origin v0.1.0
```

3. GitHub Actions will:
   - Build release assets for macOS, Linux, and Windows
   - Publish to crates.io when `CARGO_REGISTRY_TOKEN` is set
   - Update and push Homebrew/Scoop manifests when `PACKAGE_MANAGERS_TOKEN` is set

4. Or run locally after the release finishes:

```sh
./scripts/update-package-managers.sh 0.1.0
cd ../homebrew-tap && git commit -am "arc-dev 0.1.0" && git push
cd ../scoop-bucket && git commit -am "arc-dev 0.1.0" && git push
cargo publish --locked
```

## Install channels

| Channel | Command |
|---------|---------|
| Install script | `curl -fsSL https://raw.githubusercontent.com/veliuysal/arc-dev/main/scripts/install.sh \| bash` |
| Cargo | `cargo install arc-dev --locked` |
| Homebrew tap | `brew install veliuysal/tap/arc-dev` |
| Scoop bucket | `scoop install arc-dev` |
| Local checkout | `cargo install --path . --locked` |

## Homebrew tap setup

Users install with:

```sh
brew tap veliuysal/tap
brew install arc-dev
```

## Scoop bucket setup

Users install with:

```powershell
scoop bucket add veliuysal https://github.com/veliuysal/scoop-bucket
scoop install arc-dev
```
