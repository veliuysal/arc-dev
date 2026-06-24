use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
};

use thiserror::Error;

use crate::package_manager::PackageManager;

pub const DEFAULT_NODE_VERSION: &str = "24";

pub struct InstallOptions {
    pub package_manager: PackageManager,
    pub node_version: String,
}

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("failed to run `{command}`")]
    CommandFailed {
        command: String,
        status: Option<ExitStatus>,
    },
    #[error(
        "install Node.js {version} manually from https://nodejs.org (npm is required for {manager})"
    )]
    NodeRequired { version: String, manager: String },
    #[error("Homebrew is required to install Node.js on macOS (`brew install node@{0}`)")]
    HomebrewRequired(String),
    #[error("foundry installer did not create {0}")]
    FoundryInstallerMissing(PathBuf),
}

pub fn install(options: InstallOptions) -> Result<(), InstallError> {
    ensure_rust()?;
    ensure_foundry()?;
    ensure_node_package_manager(options.package_manager, &options.node_version)?;
    Ok(())
}

fn ensure_rust() -> Result<(), InstallError> {
    if command_available("rustc") && command_available("cargo") {
        println!("rust: already installed ({})", command_version("rustc")?);
        return Ok(());
    }

    println!("rust: installing stable toolchain via rustup");
    run_shell(
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable",
    )?;

    if command_available("rustc") {
        println!("rust: installed ({})", command_version("rustc")?);
    } else {
        println!("rust: installed — restart your shell or run `source \"$HOME/.cargo/env\"`");
    }

    Ok(())
}

fn ensure_foundry() -> Result<(), InstallError> {
    if command_available("forge") && command_available("cast") {
        println!("foundry: already installed ({})", command_version("forge")?);
        return Ok(());
    }

    println!("foundry: installing via foundryup");
    if !command_available("foundryup") {
        run_shell("curl -L https://foundry.paradigm.xyz | bash")?;
    }

    let foundryup = foundryup_path()?;
    run_command(&foundryup, &[])?;

    if command_available("forge") {
        println!("foundry: installed ({})", command_version("forge")?);
    } else {
        println!("foundry: installed — add \"$HOME/.foundry/bin\" to PATH if needed");
    }

    Ok(())
}

fn ensure_node_package_manager(
    manager: PackageManager,
    node_version: &str,
) -> Result<(), InstallError> {
    ensure_node(manager, node_version)?;

    match manager {
        PackageManager::Npm => {
            println!("npm: already available ({})", command_version("npm")?);
        }
        PackageManager::Pnpm => {
            if command_available("pnpm") {
                println!("pnpm: already installed ({})", command_version("pnpm")?);
                return Ok(());
            }
            println!("pnpm: installing globally via npm");
            run_command("npm", &["install", "-g", "pnpm"])?;
            println!("pnpm: installed ({})", command_version("pnpm")?);
        }
        PackageManager::Yarn => {
            if command_available("yarn") {
                println!("yarn: already installed ({})", command_version("yarn")?);
                return Ok(());
            }
            println!("yarn: installing globally via npm");
            run_command("npm", &["install", "-g", "yarn"])?;
            println!("yarn: installed ({})", command_version("yarn")?);
        }
    }

    Ok(())
}

fn ensure_node(manager: PackageManager, node_version: &str) -> Result<(), InstallError> {
    if command_available("node") && command_available("npm") {
        println!("node: already installed ({})", command_version("node")?);
        return Ok(());
    }

    if command_available("brew") {
        let formula = node_brew_formula(node_version);
        println!("node: installing {formula} via Homebrew");
        run_command("brew", &["install", &formula])?;
        let _ = run_command("brew", &["link", "--overwrite", &formula]);
        println!("node: installed ({})", command_version("node")?);
        return Ok(());
    }

    if cfg!(target_os = "macos") {
        return Err(InstallError::HomebrewRequired(node_version.to_string()));
    }

    Err(InstallError::NodeRequired {
        version: node_version.to_string(),
        manager: manager.command().to_string(),
    })
}

fn node_brew_formula(version: &str) -> String {
    format!("node@{version}")
}

fn foundryup_path() -> Result<PathBuf, InstallError> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let foundryup = home.join(".foundry/bin/foundryup");
    if foundryup.is_file() {
        Ok(foundryup)
    } else {
        Err(InstallError::FoundryInstallerMissing(foundryup))
    }
}

fn command_available(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {command}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn command_version(command: &str) -> Result<String, InstallError> {
    let output = Command::new(command)
        .arg("--version")
        .output()
        .map_err(|_| InstallError::CommandFailed {
            command: format!("{command} --version"),
            status: None,
        })?;

    if !output.status.success() {
        return Err(InstallError::CommandFailed {
            command: format!("{command} --version"),
            status: Some(output.status),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_shell(script: &str) -> Result<(), InstallError> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(script)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|_| InstallError::CommandFailed {
            command: script.to_string(),
            status: None,
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(InstallError::CommandFailed {
            command: script.to_string(),
            status: Some(status),
        })
    }
}

fn run_command(command: impl AsRef<OsStr>, args: &[&str]) -> Result<(), InstallError> {
    let command = command.as_ref();
    let label = if args.is_empty() {
        command.to_string_lossy().into_owned()
    } else {
        format!("{} {}", command.to_string_lossy(), args.join(" "))
    };

    let status = Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|_| InstallError::CommandFailed {
            command: label.clone(),
            status: None,
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(InstallError::CommandFailed {
            command: label,
            status: Some(status),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_available_commands() {
        assert!(command_available("sh"));
    }

    #[test]
    fn node_brew_formula_uses_versioned_keg() {
        assert_eq!(node_brew_formula("24"), "node@24");
    }
}
