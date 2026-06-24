use std::{
    fs,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::create::{CreateComponent, CreateTarget};

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("project path is not a directory: {0}")]
    NotDirectory(PathBuf),
    #[error("failed to create project directory {0}: {1}")]
    CreateRoot(PathBuf, std::io::Error),
    #[error("failed to write {0}: {1}")]
    Write(PathBuf, std::io::Error),
}

pub fn scaffold(target: CreateTarget, project_root: &Path) -> Result<(), ProjectError> {
    if project_root.exists() && !project_root.is_dir() {
        return Err(ProjectError::NotDirectory(project_root.to_path_buf()));
    }

    if !project_root.exists() {
        fs::create_dir_all(project_root)
            .map_err(|error| ProjectError::CreateRoot(project_root.to_path_buf(), error))?;
    }

    let project_name = project_name(project_root);

    for component in target.components() {
        let directory = crate::create::component_path(target, project_root, *component);
        match component {
            CreateComponent::App => scaffold_app(&directory, &project_name)?,
            CreateComponent::Contract => scaffold_contracts(&directory, &project_name)?,
        }
    }

    if !project_root.join("README.md").is_file() {
        write_file(
            &project_root.join("README.md"),
            &root_readme(&project_name, target),
        )?;
    }

    if target.components().contains(&CreateComponent::Contract)
        && !project_root.join(".env.example").is_file()
    {
        write_file(&project_root.join(".env.example"), ROOT_ENV)?;
    }

    Ok(())
}

fn scaffold_app(directory: &Path, project_name: &str) -> Result<(), ProjectError> {
    if directory.join("package.json").is_file() {
        return Ok(());
    }

    fs::create_dir_all(directory.join("src/config"))
        .map_err(|error| ProjectError::CreateRoot(directory.to_path_buf(), error))?;
    fs::create_dir_all(directory.join("src/lib"))
        .map_err(|error| ProjectError::CreateRoot(directory.to_path_buf(), error))?;

    write_file(
        &directory.join("package.json"),
        &app_package_json(project_name),
    )?;
    write_file(&directory.join("index.html"), APP_INDEX_HTML)?;
    write_file(&directory.join("vite.config.ts"), APP_VITE_CONFIG)?;
    write_file(&directory.join("tsconfig.json"), APP_TSCONFIG)?;
    write_file(&directory.join("tsconfig.app.json"), APP_TSCONFIG_APP)?;
    write_file(&directory.join("tsconfig.node.json"), APP_TSCONFIG_NODE)?;
    write_file(&directory.join("src/main.tsx"), APP_MAIN)?;
    write_file(&directory.join("src/App.tsx"), APP_APP)?;
    write_file(&directory.join("src/styles.css"), APP_STYLES)?;
    write_file(&directory.join("src/vite-env.d.ts"), APP_VITE_ENV)?;
    write_file(&directory.join("src/config/arc.ts"), APP_ARC_CONFIG)?;
    write_file(&directory.join("src/lib/circle.ts"), APP_CIRCLE)?;
    write_file(&directory.join(".env.example"), APP_ENV)?;
    write_file(
        &directory.join(".nvmrc"),
        crate::install::DEFAULT_NODE_VERSION,
    )?;
    write_file(&directory.join("pnpm-workspace.yaml"), APP_PNPM_WORKSPACE)?;
    Ok(())
}

fn scaffold_contracts(directory: &Path, _project_name: &str) -> Result<(), ProjectError> {
    if directory.join("foundry.toml").is_file() {
        return Ok(());
    }

    fs::create_dir_all(directory.join("src"))
        .map_err(|error| ProjectError::CreateRoot(directory.to_path_buf(), error))?;
    fs::create_dir_all(directory.join("test"))
        .map_err(|error| ProjectError::CreateRoot(directory.to_path_buf(), error))?;
    fs::create_dir_all(directory.join("script"))
        .map_err(|error| ProjectError::CreateRoot(directory.to_path_buf(), error))?;

    write_file(&directory.join("foundry.toml"), CONTRACTS_FOUNDRY_TOML)?;
    write_file(
        &directory.join("src/ArcFirstContract.sol"),
        CONTRACTS_SOURCE,
    )?;
    write_file(
        &directory.join("test/ArcFirstContract.t.sol"),
        CONTRACTS_TEST,
    )?;
    write_file(
        &directory.join("script/ArcFirstContract.s.sol"),
        CONTRACTS_SCRIPT,
    )?;
    Ok(())
}

fn write_file(path: &Path, contents: &str) -> Result<(), ProjectError> {
    fs::write(path, contents).map_err(|error| ProjectError::Write(path.to_path_buf(), error))
}

fn project_name(project_root: &Path) -> String {
    project_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("arc-project")
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn app_package_json(project_name: &str) -> String {
    format!(
        r#"{{
  "name": "{project_name}-app",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "engines": {{
    "node": ">=24"
  }},
  "scripts": {{
    "dev": "vite --host 127.0.0.1",
    "build": "tsc -b && vite build",
    "preview": "vite preview --host 127.0.0.1"
  }},
  "dependencies": {{
    "@circle-fin/adapter-viem-v2": "^1.11.2",
    "@circle-fin/app-kit": "^1.7.0",
    "react": "latest",
    "react-dom": "latest",
    "viem": "latest"
  }},
  "devDependencies": {{
    "@types/react": "latest",
    "@types/react-dom": "latest",
    "@vitejs/plugin-react": "latest",
    "typescript": "latest",
    "vite": "latest"
  }}
}}
"#
    )
}

fn root_readme(project_name: &str, target: CreateTarget) -> String {
    let mut sections = Vec::new();

    if target.components().contains(&CreateComponent::Contract) {
        let contract_section = if target == CreateTarget::All {
            "## Contracts\n\n```sh\ncd contracts\nforge build\nforge test\n```\n"
        } else {
            "## Contracts\n\n```sh\nforge build\nforge test\n```\n"
        };
        sections.push(contract_section.to_string());
    }
    if target.components().contains(&CreateComponent::App) {
        let app_section = if target == CreateTarget::All {
            "## App\n\n```sh\ncd app\ncp .env.example .env\npnpm dev\n```\n"
        } else {
            "## App\n\n```sh\ncp .env.example .env\npnpm dev\n```\n"
        };
        sections.push(app_section.to_string());
    }

    format!(
        "# {project_name}\n\nArc development environment created with `arc-dev create`.\n\n{}",
        sections.join("\n")
    )
}

const ROOT_ENV: &str = r#"ARC_TESTNET_RPC_URL="https://rpc.testnet.arc.network"
"#;

const APP_INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Arc App</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#;

const APP_VITE_CONFIG: &str = r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()]
})
"#;

const APP_TSCONFIG: &str = r#"{
  "files": [],
  "references": [
    { "path": "./tsconfig.app.json" },
    { "path": "./tsconfig.node.json" }
  ]
}
"#;

const APP_TSCONFIG_APP: &str = r#"{
  "compilerOptions": {
    "tsBuildInfoFile": "./node_modules/.tmp/tsconfig.app.tsbuildinfo",
    "target": "ES2022",
    "useDefineForClassFields": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "allowJs": false,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "allowSyntheticDefaultImports": true,
    "strict": true,
    "forceConsistentCasingInFileNames": true,
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx"
  },
  "include": ["src"]
}
"#;

const APP_TSCONFIG_NODE: &str = r#"{
  "compilerOptions": {
    "tsBuildInfoFile": "./node_modules/.tmp/tsconfig.node.tsbuildinfo",
    "target": "ES2023",
    "lib": ["ES2023"],
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true
  },
  "include": ["vite.config.ts"]
}
"#;

const APP_MAIN: &str = r#"import React from 'react'
import ReactDOM from 'react-dom/client'
import { App } from './App'
import './styles.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)
"#;

const APP_APP: &str = r#"import { arcChain } from './config/arc'

export function App() {
  return (
    <main className="appShell">
      <h1>{arcChain.name}</h1>
      <p>Arc app scaffold. Connect Circle App Kit from `src/lib/circle.ts`.</p>
    </main>
  )
}
"#;

const APP_STYLES: &str = r#":root {
  font-family: Inter, system-ui, sans-serif;
  color: #111827;
  background: #f8fafc;
}

body {
  margin: 0;
}

.appShell {
  max-width: 48rem;
  margin: 3rem auto;
  padding: 0 1.5rem;
}
"#;

const APP_VITE_ENV: &str = r#"/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_ARC_FIRST_CONTRACT_ADDRESS?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
"#;

const APP_ARC_CONFIG: &str = r#"import { ArcTestnet } from '@circle-fin/app-kit/chains'
import type { Address } from 'viem'

export const arcChain = ArcTestnet

export const arcFirstContractAddress = (
  import.meta.env.VITE_ARC_FIRST_CONTRACT_ADDRESS || '0x0000000000000000000000000000000000000000'
) as Address
"#;

const APP_CIRCLE: &str = r#"import { createViemAdapterFromProvider } from '@circle-fin/adapter-viem-v2'
import type { EIP1193Provider } from 'viem'
import { arcChain } from '../config/arc'

export async function createCircleWallet(provider: EIP1193Provider) {
  return createViemAdapterFromProvider({
    provider,
    capabilities: {
      addressContext: 'user-controlled',
      supportedChains: [arcChain]
    }
  })
}
"#;

const APP_ENV: &str = r#"VITE_ARC_FIRST_CONTRACT_ADDRESS="0x0000000000000000000000000000000000000000"
"#;

const APP_PNPM_WORKSPACE: &str = r#"allowBuilds:
  bufferutil: true
  esbuild: true
  utf-8-validate: true

peerDependencyRules:
  allowedVersions:
    bufferutil: "4"
    utf-8-validate: "6"

overrides:
  uuid: ">=14.0.0"
"#;

const CONTRACTS_FOUNDRY_TOML: &str = r#"[profile.default]
src = "src"
out = "out"
libs = ["lib"]

[rpc_endpoints]
arc_testnet = "${ARC_TESTNET_RPC_URL}"
"#;

const CONTRACTS_SOURCE: &str = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract ArcFirstContract {
    address public owner;

    constructor() {
        owner = msg.sender;
    }

    function contract_deployer() public view returns (string memory) {
        return string(abi.encodePacked("Deployed by ", owner));
    }
}
"#;

const CONTRACTS_TEST: &str = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {ArcFirstContract} from "../src/ArcFirstContract.sol";

contract ArcFirstContractTest is Test {
    ArcFirstContract public arcFirstContract;

    function setUp() public {
        arcFirstContract = new ArcFirstContract();
    }

    function test_OwnerIsDeployer() public view {
        assertEq(arcFirstContract.owner(), address(this));
    }
}
"#;

const CONTRACTS_SCRIPT: &str = r#"// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script} from "forge-std/Script.sol";
import {ArcFirstContract} from "../src/ArcFirstContract.sol";

contract ArcFirstContractScript is Script {
    function run() public {
        vm.startBroadcast();
        new ArcFirstContract();
        vm.stopBroadcast();
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scaffolds_app_project_at_path() {
        let root = tempfile::tempdir().expect("temp project directory");
        let project = root.path().join("my-arc-app");

        scaffold(CreateTarget::App, &project).expect("scaffold app");

        assert!(project.join("README.md").is_file());
        assert!(project.join("package.json").is_file());
        assert!(project.join("pnpm-workspace.yaml").is_file());
        assert!(project.join("src/App.tsx").is_file());
        assert!(!project.join("app").exists());
        assert!(!project.join("contracts").exists());
    }

    #[test]
    fn scaffolds_contract_project_at_path() {
        let root = tempfile::tempdir().expect("temp project directory");
        let project = root.path().join("my-contract");

        scaffold(CreateTarget::Contract, &project).expect("scaffold contract");

        assert!(project.join("foundry.toml").is_file());
        assert!(project.join("src/ArcFirstContract.sol").is_file());
        assert!(!project.join("contracts").exists());
    }

    #[test]
    fn scaffolds_all_project_layout() {
        let root = tempfile::tempdir().expect("temp project directory");
        let project = root.path().join("full-app");

        scaffold(CreateTarget::All, &project).expect("scaffold all");

        assert!(project.join("app/package.json").is_file());
        assert!(project.join("contracts/foundry.toml").is_file());
        assert!(project.join("contracts/src/ArcFirstContract.sol").is_file());
        assert!(project.join(".env.example").is_file());
    }

    #[test]
    fn does_not_overwrite_existing_app_package_json() {
        let root = tempfile::tempdir().expect("temp project directory");
        fs::write(root.path().join("package.json"), r#"{"name":"custom"}"#).expect("write package");

        scaffold(CreateTarget::App, root.path()).expect("scaffold app");

        let package_json =
            fs::read_to_string(root.path().join("package.json")).expect("read package");
        assert!(package_json.contains("custom"));
    }

    #[test]
    fn component_path_uses_subfolders_only_for_all() {
        let root = tempfile::tempdir().expect("temp project directory");
        let project = root.path();

        assert_eq!(
            crate::create::component_path(CreateTarget::App, project, CreateComponent::App),
            project
        );
        assert_eq!(
            crate::create::component_path(
                CreateTarget::Contract,
                project,
                CreateComponent::Contract
            ),
            project
        );
        assert_eq!(
            crate::create::component_path(CreateTarget::All, project, CreateComponent::App),
            project.join("app")
        );
    }
}
