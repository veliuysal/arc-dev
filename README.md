# arc-dev

Command-line tool for Arc network development — JSON-RPC access, dev tool
installation, and project scaffolding for app and contract workflows.

## Install

### Install script (macOS / Linux)

```sh
curl -fsSL https://raw.githubusercontent.com/veliuysal/arc-dev/main/scripts/install.sh | bash
```

Override the repository or version:

```sh
ARC_DEV_REPO=your-org/arc-dev ARC_DEV_VERSION=0.1.0 \
  curl -fsSL https://raw.githubusercontent.com/your-org/arc-dev/main/scripts/install.sh | bash
```

### Cargo (prebuilt binary)

Install [cargo-binstall](https://github.com/cargo-bins/cargo-binstall), then:

```sh
cargo binstall arc-dev
```

This downloads the release binary. It does not compile source.

### Homebrew

```sh
brew tap veliuysal/tap
brew trust veliuysal/tap
brew install arc-dev
```

### Scoop (Windows)

```sh
scoop bucket add veliuysal https://github.com/veliuysal/scoop-bucket
scoop install arc-dev
```

The executable is named `arc-dev`. After installation, verify with:

```sh
arc-dev --version
```

Publishing notes: https://github.com/veliuysal/arc-dev/blob/main/packaging/README.md

## Build

```sh
cargo build --release
```

The executable is produced at `target/release/arc-dev`.

## Endpoint selection

No configuration is required for Arc testnet:

```sh
arc-dev rpc eth_chainId
```

The default endpoint is:

```text
https://rpc.testnet.arc.network
```

Override it for one invocation:

```sh
arc-dev --rpc-url http://localhost:8545 rpc eth_chainId
```

Or set the environment variable:

```sh
export ARC_RPC_URL=http://localhost:8545
arc-dev rpc eth_chainId
```

Endpoint precedence is:

1. `--rpc-url`
2. `ARC_RPC_URL`
3. `https://rpc.testnet.arc.network`

## Pass JSON-RPC parameters

Parameters must be a JSON array or object. If omitted, they default to `[]`.

```sh
arc-dev rpc eth_getBalance --params '["0xabc", "latest"]'
```

Successful results are printed as JSON to stdout, making them suitable for
shell pipelines:

```sh
arc-dev rpc eth_chainId | jq -r .
```

Errors are written to stderr and return a non-zero exit code.

## Install development tools

Install the Arc development toolchain on your machine:

```sh
arc-dev install
arc-dev install --node-version 24
arc-dev install --package-manager pnpm
arc-dev install --package-manager yarn
```

This installs:

- **Rust** — stable toolchain via [rustup](https://rustup.rs) (skipped if already installed)
- **Foundry** — `forge`, `cast`, and `foundryup` (skipped if already installed)
- **Node.js + package manager** — Node.js 24 by default (`node@24` via Homebrew on macOS when missing); npm is the default package manager, with optional pnpm or yarn

## Create a dev environment

Prepare an Arc network development environment at a path — scaffold project
directories and install their dependencies:

```sh
arc-dev create all ./my-arc-app
arc-dev create app ./my-arc-app
arc-dev create contract ./path/to/project
```

Omit the path to use the current working directory:

```sh
mkdir my-arc-app && cd my-arc-app
arc-dev create all
```

The command scaffolds files directly in the path for `app` or `contract`. For
`all`, it creates separate `app/` and `contracts/` directories under the path:

- `app` — React + Vite frontend using Circle App Kit in the path directory
- `contract` — Foundry smart contracts in the path directory
- `all` — both under `app/` and `contracts/`

When the target includes `app`, the CLI asks which package manager to use
(`npm`, `pnpm`, or `yarn`). There is no default — you must choose. Pass
`--package-manager` to skip the prompt (required in non-interactive shells):

```sh
arc-dev create app ./my-arc-app --package-manager pnpm
```

If the project already has a lockfile, that package manager is used unless
`--package-manager` overrides it.

When `pnpm` is selected, the scaffold includes a `pnpm-workspace.yaml` with
pre-approved native builds, and the CLI runs `pnpm approve-builds --all`
automatically if needed.

Existing `package.json` or `foundry.toml` files in the target directory are
left unchanged. A project `README.md` and root `.env.example` are added when
missing.

## Current scope

This scaffold does not manage keys, sign or broadcast transactions, or persist
configuration. Arc currently exposes only the testnet endpoint above. Future
mainnet support will add an explicit network model after its URL and selection
behavior are defined.

## Development checks

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
