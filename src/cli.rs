use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::create::CreateTarget;
use crate::package_manager::PackageManager;

/// Command-line tools for Arc network development.
#[derive(Debug, Parser)]
#[command(name = "arc-dev", version, about, long_about = None)]
#[command(
    after_help = "Endpoint precedence: --rpc-url, ARC_RPC_URL, then https://rpc.testnet.arc.network\n\nExamples:\n  arc-dev install\n  arc-dev install --package-manager pnpm\n  arc-dev rpc eth_chainId\n  arc-dev create all ./my-arc-app"
)]
pub struct Cli {
    /// JSON-RPC endpoint. Overrides ARC_RPC_URL and the Arc testnet default.
    #[arg(long, global = true, value_name = "URL")]
    pub rpc_url: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Call a JSON-RPC method and print its result as JSON.
    Rpc {
        /// JSON-RPC method name, such as eth_chainId.
        method: String,

        /// JSON array or object passed as the method parameters.
        #[arg(long, default_value = "[]", value_name = "JSON")]
        params: String,
    },

    /// Install Arc development tools (Rust, Foundry, and a JS package manager).
    Install {
        /// Node.js major version to install when missing. Defaults to 24.
        #[arg(long, value_name = "VERSION", default_value = "24")]
        node_version: String,

        /// JavaScript package manager to install. Defaults to npm.
        #[arg(long, value_name = "MANAGER", default_value = "npm")]
        package_manager: PackageManager,
    },

    /// Prepare an Arc network development environment.
    Create {
        /// Which parts of the dev environment to scaffold and prepare.
        target: CreateTarget,

        /// Package manager for the app. Prompts interactively when omitted.
        #[arg(long, value_name = "MANAGER")]
        package_manager: Option<PackageManager>,

        /// Project location. Defaults to the current working directory.
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },
}
