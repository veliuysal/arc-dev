use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

use clap::ValueEnum;
use thiserror::Error;

use crate::package_manager::PackageManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CreateTarget {
    /// Scaffold and prepare a React app in the path directory.
    App,
    /// Scaffold and prepare Foundry contracts in the path directory.
    Contract,
    /// Scaffold and prepare both `app/` and `contracts/` under the path.
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateComponent {
    App,
    Contract,
}

pub struct CreateOptions {
    pub package_manager: Option<PackageManager>,
}

impl CreateTarget {
    pub fn components(self) -> &'static [CreateComponent] {
        match self {
            Self::App => &[CreateComponent::App],
            Self::Contract => &[CreateComponent::Contract],
            Self::All => &[CreateComponent::App, CreateComponent::Contract],
        }
    }
}

impl CreateComponent {
    pub fn subdirectory(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Contract => "contracts",
        }
    }
}

pub fn component_path(
    target: CreateTarget,
    project_root: &Path,
    component: CreateComponent,
) -> PathBuf {
    match target {
        CreateTarget::All => project_root.join(component.subdirectory()),
        CreateTarget::App | CreateTarget::Contract => project_root.to_path_buf(),
    }
}

#[derive(Debug, Error)]
pub enum CreateError {
    #[error(transparent)]
    Project(#[from] crate::project::ProjectError),
    #[error("project directory not found: {0}")]
    MissingDirectory(PathBuf),
    #[error("no package.json in {0}")]
    MissingPackageJson(PathBuf),
    #[error("no foundry.toml in {0}")]
    MissingFoundryProject(PathBuf),
    #[error("package manager selection was cancelled")]
    SelectionCancelled,
    #[error(
        "app setup requires --package-manager when stdin is not interactive and no lockfile exists"
    )]
    PackageManagerRequired,
    #[error("package manager `{program}` is not installed or not in PATH")]
    PackageManagerNotFound { program: String },
    #[error("forge is not installed or not in PATH")]
    ForgeNotFound,
    #[error("failed to read package manager choice")]
    ReadChoice(#[source] io::Error),
    #[error("`{program} {command}` failed in {directory}")]
    PackageCommandFailed {
        program: String,
        command: String,
        directory: PathBuf,
        status: ExitStatus,
    },
    #[error("`forge install` failed in {directory}")]
    ForgeInstallFailed {
        directory: PathBuf,
        status: ExitStatus,
    },
}

pub fn create(
    target: CreateTarget,
    project_root: &Path,
    options: CreateOptions,
) -> Result<(), CreateError> {
    crate::project::scaffold(target, project_root)?;

    for component in target.components() {
        prepare_component(target, *component, project_root, &options)?;
    }
    Ok(())
}

fn prepare_component(
    target: CreateTarget,
    component: CreateComponent,
    project_root: &Path,
    options: &CreateOptions,
) -> Result<(), CreateError> {
    let directory = component_path(target, project_root, component);

    match component {
        CreateComponent::App => prepare_app(&directory, options.package_manager),
        CreateComponent::Contract => prepare_contracts(&directory),
    }
}

fn prepare_app(directory: &Path, selected: Option<PackageManager>) -> Result<(), CreateError> {
    if !directory.is_dir() {
        return Err(CreateError::MissingDirectory(directory.to_path_buf()));
    }

    if !directory.join("package.json").is_file() {
        return Err(CreateError::MissingPackageJson(directory.to_path_buf()));
    }

    let manager = resolve_package_manager(directory, selected)?;
    run_package_install(manager, directory)
}

fn resolve_package_manager(
    directory: &Path,
    selected: Option<PackageManager>,
) -> Result<PackageManager, CreateError> {
    if let Some(manager) = selected {
        return Ok(manager);
    }

    if let Some(manager) = package_manager_from_lockfile(directory) {
        return Ok(manager);
    }

    prompt_package_manager()
}

fn package_manager_from_lockfile(directory: &Path) -> Option<PackageManager> {
    if directory.join("pnpm-lock.yaml").is_file() {
        return Some(PackageManager::Pnpm);
    }
    if directory.join("yarn.lock").is_file() {
        return Some(PackageManager::Yarn);
    }
    if directory.join("package-lock.json").is_file() {
        return Some(PackageManager::Npm);
    }

    None
}

fn prompt_package_manager() -> Result<PackageManager, CreateError> {
    if !io::stdin().is_terminal() {
        return Err(CreateError::PackageManagerRequired);
    }

    eprintln!("Which package manager should the app use?");
    eprintln!("  1) npm");
    eprintln!("  2) pnpm");
    eprintln!("  3) yarn");

    loop {
        eprint!("Enter choice [1-3]: ");
        io::stderr().flush().ok();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(CreateError::ReadChoice)?;

        match input.trim() {
            "1" => return Ok(PackageManager::Npm),
            "2" => return Ok(PackageManager::Pnpm),
            "3" => return Ok(PackageManager::Yarn),
            "" => return Err(CreateError::SelectionCancelled),
            _ => eprintln!("Invalid choice. Enter 1, 2, or 3."),
        }
    }
}

fn prepare_contracts(directory: &Path) -> Result<(), CreateError> {
    if !directory.is_dir() {
        return Err(CreateError::MissingDirectory(directory.to_path_buf()));
    }

    if !directory.join("foundry.toml").is_file() {
        return Err(CreateError::MissingFoundryProject(directory.to_path_buf()));
    }

    if directory.join("lib/forge-std").exists() {
        return Ok(());
    }

    run_forge_install(directory)
}

fn run_package_install(manager: PackageManager, directory: &Path) -> Result<(), CreateError> {
    match manager {
        PackageManager::Pnpm => run_pnpm_setup(directory),
        _ => run_package_command(manager.command(), "install", &[], directory),
    }
}

fn run_pnpm_setup(directory: &Path) -> Result<(), CreateError> {
    // pnpm may exit non-zero when dependencies need build-script approval.
    let _ = run_package_command("pnpm", "install", &[], directory);
    run_package_command("pnpm", "approve-builds", &["--all"], directory)?;
    run_package_command("pnpm", "install", &[], directory)
}

fn run_package_command(
    program: &str,
    command: &str,
    args: &[&str],
    directory: &Path,
) -> Result<(), CreateError> {
    let status = Command::new(program)
        .arg(command)
        .args(args)
        .current_dir(directory)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|_| CreateError::PackageManagerNotFound {
            program: program.to_string(),
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(CreateError::PackageCommandFailed {
            program: program.to_string(),
            command: command.to_string(),
            directory: directory.to_path_buf(),
            status,
        })
    }
}

fn run_forge_install(directory: &Path) -> Result<(), CreateError> {
    let status = Command::new("forge")
        .args(["install", "foundry-rs/forge-std", "--no-git", "--no-commit"])
        .current_dir(directory)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|_| CreateError::ForgeNotFound)?;

    if status.success() {
        Ok(())
    } else {
        Err(CreateError::ForgeInstallFailed {
            directory: directory.to_path_buf(),
            status,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_project() -> tempfile::TempDir {
        tempfile::tempdir().expect("temp project directory")
    }

    fn write_package_json(directory: &Path) {
        fs::create_dir_all(directory).expect("create package directory");
        fs::write(
            directory.join("package.json"),
            r#"{"name":"test","private":true}"#,
        )
        .expect("write package.json");
    }

    #[test]
    fn scaffolds_before_preparing_app() {
        let root = temp_project();
        crate::project::scaffold(CreateTarget::App, root.path()).expect("scaffold app");

        assert!(root.path().join("package.json").is_file());
    }

    #[test]
    fn uses_lockfile_when_no_flag_is_set() {
        let root = temp_project();
        let app = root.path().join("app");
        write_package_json(&app);
        fs::write(app.join("yarn.lock"), "").expect("write yarn.lock");

        let manager = resolve_package_manager(&app, None).expect("resolve manager");
        assert_eq!(manager, PackageManager::Yarn);
    }

    #[test]
    fn flag_overrides_lockfile_detection() {
        let root = temp_project();
        let app = root.path().join("app");
        write_package_json(&app);
        fs::write(app.join("yarn.lock"), "").expect("write yarn.lock");

        let manager =
            resolve_package_manager(&app, Some(PackageManager::Pnpm)).expect("resolve manager");
        assert_eq!(manager, PackageManager::Pnpm);
    }

    #[test]
    fn requires_package_manager_flag_when_not_interactive() {
        if io::stdin().is_terminal() {
            return;
        }

        let root = temp_project();
        let app = root.path().join("app");
        write_package_json(&app);

        let error = resolve_package_manager(&app, None).unwrap_err();
        assert!(matches!(error, CreateError::PackageManagerRequired));
    }

    #[test]
    fn all_prepares_app_before_contracts() {
        assert_eq!(
            CreateTarget::All.components(),
            &[CreateComponent::App, CreateComponent::Contract]
        );
    }

    #[test]
    fn skips_forge_install_when_dependency_exists() {
        let root = temp_project();
        let contracts = root.path().join("contracts");
        fs::create_dir_all(contracts.join("lib/forge-std")).expect("create forge-std");
        fs::write(contracts.join("foundry.toml"), "[profile.default]\n").expect("write foundry");

        prepare_contracts(&contracts).expect("skip existing dependency");
    }
}
