mod cli;
mod config;
mod create;
mod install;
mod package_manager;
mod project;
mod rpc;

use std::env;

use clap::Parser;
use cli::{Cli, Command};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
enum AppError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("--params must be valid JSON: {0}")]
    InvalidParams(serde_json::Error),
    #[error("--params must be a JSON array or object")]
    InvalidParamsShape,
    #[error(transparent)]
    Rpc(#[from] rpc::RpcError),
    #[error("failed to serialize RPC result")]
    Output,
    #[error(transparent)]
    Create(#[from] create::CreateError),
    #[error(transparent)]
    Install(#[from] install::InstallError),
    #[error("could not determine current working directory")]
    WorkingDirectory,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    match cli.command {
        Command::Install {
            package_manager,
            node_version,
        } => {
            install::install(install::InstallOptions {
                package_manager,
                node_version,
            })?;
        }
        Command::Rpc { method, params } => {
            let rpc_url = config::resolve_rpc_url(cli.rpc_url)?;
            let params: Value = serde_json::from_str(&params).map_err(AppError::InvalidParams)?;
            if !params.is_array() && !params.is_object() {
                return Err(AppError::InvalidParamsShape);
            }

            let result = rpc::call(&reqwest::Client::new(), rpc_url, &method, &params).await?;
            let output = serde_json::to_string_pretty(&result).map_err(|_| AppError::Output)?;
            println!("{output}");
        }
        Command::Create {
            target,
            package_manager,
            path,
        } => {
            let project_root = match path {
                Some(path) => path,
                None => env::current_dir().map_err(|_| AppError::WorkingDirectory)?,
            };
            create::create(
                target,
                &project_root,
                create::CreateOptions { package_manager },
            )?;
        }
    }

    Ok(())
}
