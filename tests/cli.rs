use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::{Command, Output},
    thread,
};

fn arc_dev() -> Command {
    let mut command =
        Command::new(std::env::var("CARGO_BIN_EXE_arc-dev").expect("arc-dev binary not built"));
    command.env_remove("ARC_RPC_URL");
    command
}

fn run_server(response: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let address = listener.local_addr().expect("mock server address");
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let request = read_request(&mut stream);
        stream
            .write_all(response.as_bytes())
            .expect("write response");
        request
    });
    (format!("http://{address}"), handle)
}

fn read_request(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let count = stream.read(&mut chunk).expect("read request");
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..count]);

        if let Some(header_end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&bytes[..header_end + 4]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap_or(0);
            if bytes.len() >= header_end + 4 + content_length {
                break;
            }
        }
    }
    String::from_utf8(bytes).expect("request is utf-8")
}

fn success_response(result: &str) -> String {
    let body = format!(r#"{{"jsonrpc":"2.0","id":1,"result":{result}}}"#);
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn assert_failed(output: Output, message: &str) {
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(message),
        "stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn calls_rpc_and_prints_json_result() {
    let response = Box::leak(success_response(r#""0x1""#).into_boxed_str());
    let (url, server) = run_server(response);

    let output = arc_dev()
        .args(["--rpc-url", &url, "rpc", "eth_chainId"])
        .output()
        .expect("run arc-dev");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "\"0x1\"\n");
    let request = server.join().expect("join mock server");
    assert!(request.contains(r#""method":"eth_chainId""#));
    assert!(request.contains(r#""params":[]"#));
}

#[test]
fn command_line_url_overrides_environment() {
    let response = Box::leak(success_response("7").into_boxed_str());
    let (url, server) = run_server(response);

    let output = arc_dev()
        .env("ARC_RPC_URL", "http://127.0.0.1:1")
        .args(["--rpc-url", &url, "rpc", "test_method"])
        .output()
        .expect("run arc-dev");

    assert!(output.status.success());
    server.join().expect("join mock server");
}

#[test]
fn rejects_invalid_or_scalar_params() {
    let malformed = arc_dev()
        .args([
            "--rpc-url",
            "http://localhost:8545",
            "rpc",
            "method",
            "--params",
            "{",
        ])
        .output()
        .expect("run arc-dev");
    assert_failed(malformed, "--params must be valid JSON");

    let scalar = arc_dev()
        .args([
            "--rpc-url",
            "http://localhost:8545",
            "rpc",
            "method",
            "--params",
            "42",
        ])
        .output()
        .expect("run arc-dev");
    assert_failed(scalar, "--params must be a JSON array or object");
}

#[test]
fn reports_json_rpc_errors() {
    let body = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"not found"}}"#;
    let response = Box::leak(
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_boxed_str(),
    );
    let (url, server) = run_server(response);

    let output = arc_dev()
        .args(["--rpc-url", &url, "rpc", "missing"])
        .output()
        .expect("run arc-dev");

    assert_failed(output, "RPC error -32601: not found");
    server.join().expect("join mock server");
}

#[test]
fn reports_invalid_json_without_exposing_endpoint_credentials() {
    let response = "HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nnot-json";
    let (url, server) = run_server(response);
    let credentialed_url = url.replacen("http://", "http://secret:token@", 1);

    let output = arc_dev()
        .args(["--rpc-url", &credentialed_url, "rpc", "method"])
        .output()
        .expect("run arc-dev");

    assert_failed(output.clone(), "RPC endpoint returned invalid JSON");
    assert!(!String::from_utf8_lossy(&output.stderr).contains("secret"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("token"));
    server.join().expect("join mock server");
}

#[test]
fn create_scaffolds_project_at_path() {
    let root = tempfile::tempdir().expect("temp project directory");
    let project = root.path().join("new-arc-app");
    let bin = root.path().join("bin");
    std::fs::create_dir_all(&bin).expect("create fake bin directory");
    write_fake_pnpm(&bin.join("pnpm"));
    write_fake_forge(&bin.join("forge"));

    let path = prepend_to_path(&bin);

    let output = arc_dev()
        .env("PATH", path)
        .args([
            "create",
            "all",
            "--package-manager",
            "pnpm",
            project.to_str().expect("project path"),
        ])
        .output()
        .expect("run arc-dev");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(project.join("app/package.json").is_file());
    assert!(project.join("contracts/foundry.toml").is_file());
    assert!(project.join("README.md").is_file());
}

#[test]
fn install_help_defaults_to_npm() {
    let output = arc_dev()
        .args(["install", "--help"])
        .output()
        .expect("run arc-dev");

    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("npm"));
    assert!(help.contains("pnpm"));
    assert!(help.contains("yarn"));
    assert!(help.contains("Foundry"));
    assert!(help.contains("Rust"));
    assert!(help.contains("node-version"));
    assert!(help.contains("24"));
}

#[test]
fn create_help_lists_targets_and_path() {
    let output = arc_dev()
        .args(["create", "--help"])
        .output()
        .expect("run arc-dev");

    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("app"));
    assert!(help.contains("contract"));
    assert!(help.contains("all"));
    assert!(help.contains("PATH"));
    assert!(help.contains("package-manager"));
}

#[cfg(unix)]
fn write_fake_forge(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(
        path,
        "#!/bin/sh\ncase \"$1 $2 $3\" in\n  \"install foundry-rs/forge-std\"*)\n    mkdir -p lib/forge-std\n    exit 0\n    ;;\nesac\nexit 1\n",
    )
    .expect("write fake forge");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .expect("mark fake forge executable");
}

#[cfg(unix)]
fn write_fake_pnpm(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(
        path,
        "#!/bin/sh\ncase \"$1\" in\n  install|approve-builds)\n    exit 0\n    ;;\nesac\nexit 1\n",
    )
    .expect("write fake pnpm");
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .expect("mark fake pnpm executable");
}

#[cfg(unix)]
fn prepend_to_path(directory: &std::path::Path) -> String {
    let system_path = std::env::var("PATH").unwrap_or_default();
    format!("{}:{system_path}", directory.display())
}
