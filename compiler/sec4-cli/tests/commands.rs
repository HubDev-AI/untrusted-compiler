use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::Duration;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn cli_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sec4"))
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(cli_bin())
        .args(args)
        .output()
        .expect("sec4 CLI should run")
}

fn clang_available() -> bool {
    Command::new("clang")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn clang_with_openssl_available() -> bool {
    if !clang_available() {
        return false;
    }

    let probe_dir = temp_dir("sec4-clang-openssl-probe");
    let probe_source = probe_dir.join("probe.c");
    let probe_binary = probe_dir.join("probe");
    if fs::write(
        &probe_source,
        r#"#include <openssl/ssl.h>

int main(void) {
  SSL_CTX *ctx = SSL_CTX_new(TLS_client_method());
  if (ctx != NULL) {
    SSL_CTX_free(ctx);
  }
  return 0;
}
"#,
    )
    .is_err()
    {
        return false;
    }

    Command::new("clang")
        .arg(&probe_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-DSEC4_RT_ENABLE_OPENSSL_TLS")
        .arg("-o")
        .arg(&probe_binary)
        .arg("-lssl")
        .arg("-lcrypto")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    format!("{nanos}")
}

fn temp_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("{prefix}-{}", unique_suffix()));
    fs::create_dir_all(&path).expect("temp directory should be created");
    path
}

fn find_available_tcp_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("ephemeral tcp bind should work");
    listener
        .local_addr()
        .expect("listener local address should resolve")
        .port()
}

fn localhost_dot_resolves() -> bool {
    ("localhost.", 80)
        .to_socket_addrs()
        .map(|mut addrs| addrs.next().is_some())
        .unwrap_or(false)
}

fn write_minimal_project(project_dir: &PathBuf, policy_source: &str) {
    fs::create_dir_all(project_dir.join("src")).expect("src dir should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"commands-fixture\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), policy_source).expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn health() effects { net } -> Int {\n  res.text(200, \"smoke body\");\n  0\n}\n\nfn main() effects { net } -> Int {\n  let router = http.router();\n  http.get(router, \"/probe\", health);\n  0\n}\n",
    )
    .expect("entry should be written");
}

fn write_manifest_with_profile(project_dir: &PathBuf, profile: &str) {
    fs::write(
        project_dir.join("sec4.toml"),
        format!(
            "[package]\n\
name = \"commands-fixture\"\n\
version = \"0.1.0\"\n\
\n\
[build]\n\
entry = \"src/main.ut\"\n\
profile = \"{profile}\"\n"
        ),
    )
    .expect("manifest should be written");
}

#[test]
fn init_creates_project_files_successfully() {
    let root = temp_dir("sec4-init-create");
    let project_dir = root.join("created-project");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "init",
        "--path",
        &project_path,
        "--name",
        "init-fixture-create",
    ]);
    assert!(output.status.success(), "init command should succeed");

    let manifest = fs::read_to_string(project_dir.join("sec4.toml"))
        .expect("manifest should be written by init command");
    assert!(
        manifest.contains("name = \"init-fixture-create\""),
        "manifest should include explicit package name:\n{manifest}"
    );
    assert!(
        project_dir.join("sec4.policy").exists(),
        "init should write sec4.policy"
    );
    let entry = fs::read_to_string(project_dir.join("src/main.ut"))
        .expect("entry source should be written by init command");
    assert!(
        entry.contains("http.serve(8080, router);"),
        "entry source should include runnable server bootstrap:\n{entry}"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("created files:"),
        "init output should include created-files summary:\n{stdout}"
    );
    assert!(
        stdout.contains("sec4.toml")
            && stdout.contains("sec4.policy")
            && stdout.contains("src/main.ut"),
        "init output should list all generated files:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn init_then_check_succeeds() {
    let root = temp_dir("sec4-init-check");
    let project_dir = root.join("check-project");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let init_output = run_cli(&[
        "init",
        "--path",
        &project_path,
        "--name",
        "init-fixture-check",
    ]);
    assert!(init_output.status.success(), "init command should succeed");

    let check_output = run_cli(&["check", "--path", &project_path]);
    assert!(
        check_output.status.success(),
        "check command should succeed for initialized project"
    );
    let stdout = String::from_utf8(check_output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("check succeeded"),
        "check output should confirm success:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_succeeds_for_multi_file_module_project() {
    let root = temp_dir("sec4-check-multi-file-pass");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src/auth")).expect("src/auth should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"multi-file-pass\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "use auth.validate;\n\nfn main() -> Int {\n  if is_valid() {\n    0\n  } else {\n    1\n  }\n}\n",
    )
    .expect("main should be written");
    fs::write(
        project_dir.join("src/auth/validate.ut"),
        "fn is_valid() -> Bool {\n  true\n}\n",
    )
    .expect("module should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        output.status.success(),
        "check should succeed for multi-file project"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_multi_file_module_is_missing() {
    let root = temp_dir("sec4-check-multi-file-missing");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"multi-file-missing\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "use auth.validate;\n\nfn main() -> Int {\n  0\n}\n",
    )
    .expect("main should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when imported module is missing"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("M0303"),
        "missing-module diagnostic code should be reported:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_multi_file_module_graph_has_cycle() {
    let root = temp_dir("sec4-check-multi-file-cycle");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"multi-file-cycle\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "use a;\n\nfn main() -> Int {\n  0\n}\n",
    )
    .expect("main should be written");
    fs::write(
        project_dir.join("src/a.ut"),
        "use b;\n\nfn a_fn() -> Int {\n  0\n}\n",
    )
    .expect("a module should be written");
    fs::write(
        project_dir.join("src/b.ut"),
        "use a;\n\nfn b_fn() -> Int {\n  0\n}\n",
    )
    .expect("b module should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when module graph has a cycle"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("M0305"),
        "cycle diagnostic code should be reported:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn build_emit_lasm_outputs_lasm_text() {
    let root = temp_dir("sec4-build-emit-lasm");
    let project_dir = root.join("lasm-project");
    write_minimal_project(&project_dir, "");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["build", "--path", &project_path, "--emit", "lasm"]);
    assert!(
        output.status.success(),
        "build --emit lasm should succeed for minimal project"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains(".lasm v0"),
        "lasm output should contain LASM version header:\n{stdout}"
    );
    assert!(
        stdout.contains(".entry main params=0 ret=Int"),
        "lasm output should include deterministic entrypoint contract:\n{stdout}"
    );
    assert!(
        stdout.contains(".fn main"),
        "lasm output should contain lowered function:\n{stdout}"
    );
    assert!(
        stdout.contains("ret 0"),
        "lasm output should contain lowered return operation:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn build_emit_lasm_json_outputs_machine_readable_lasm() {
    let root = temp_dir("sec4-build-emit-lasm-json");
    let project_dir = root.join("lasm-json-project");
    write_minimal_project(&project_dir, "");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["build", "--path", &project_path, "--emit", "lasm-json"]);
    assert!(
        output.status.success(),
        "build --emit lasm-json should succeed for minimal project"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("lasm-json output should be valid json");
    assert_eq!(
        parsed.get("version").and_then(serde_json::Value::as_str),
        Some("v0"),
        "lasm-json should include version marker"
    );
    assert_eq!(
        parsed
            .get("entry")
            .and_then(|entry| entry.get("name"))
            .and_then(serde_json::Value::as_str),
        Some("main"),
        "lasm-json should include explicit entrypoint"
    );
    assert!(
        parsed
            .get("functions")
            .and_then(serde_json::Value::as_array)
            .map(|functions| {
                functions.iter().any(|function| {
                    function
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .map(|name| name == "main")
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false),
        "lasm-json should include lowered main function:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn build_emit_c_succeeds_for_multi_file_module_project() {
    let root = temp_dir("sec4-build-emit-c-multi-file");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src/lib")).expect("src/lib should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"multi-file-build\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "use lib.helper;\n\nfn main() -> Int {\n  helper();\n  0\n}\n",
    )
    .expect("entry should be written");
    fs::write(
        project_dir.join("src/lib/helper.ut"),
        "fn helper() -> Int {\n  0\n}\n",
    )
    .expect("module should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["build", "--path", &project_path, "--emit", "c"]);
    assert!(
        output.status.success(),
        "build --emit c should succeed for multi-file project"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("int main(void);"),
        "generated C should contain lowered main function symbol:\n{stdout}"
    );
    assert!(
        stdout.contains("int64_t helper(void);"),
        "generated C should contain lowered helper function symbol from imported module:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_serves_multi_file_module_route_and_exits() {
    if !clang_available() {
        eprintln!("skipping run-command multi-file oneshot integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-oneshot-multi-file");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src/api")).expect("src/api directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runoneshotmultifilecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"use api.health;

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("entry should be written");
    fs::write(
        project_dir.join("src/api/health.ut"),
        r#"fn health() effects { net } -> Int {
  res.setHeader(headers.name("X-From-Module"), headers.value("1"));
  res.text(200, "pong from module");
  0
}
"#,
    )
    .expect("module route should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--tls-backend",
            "auto",
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command multi-file oneshot test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command multi-file oneshot process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command multi-file oneshot process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("X-From-Module: 1"),
        "response should include module-defined header:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npong from module"),
        "response should include module-defined body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_runs_in_memory_runtime_with_compiled_entrypoint() {
    let root = temp_dir("sec4-lasm-smoke-pass");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-pass\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn health() effects { net } -> Int {\n  res.text(200, \"smoke body\");\n  0\n}\n\nfn main() effects { net } -> Int {\n  let router = http.router();\n  http.get(router, \"/probe\", health);\n  0\n}\n",
    )
    .expect("entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "lasm-smoke",
        "--path",
        &project_path,
        "--method",
        "GET",
        "--route",
        "/probe",
        "--requests",
        "5",
        "--max-steps",
        "64",
    ]);
    assert!(output.status.success(), "lasm-smoke command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("lasm smoke succeeded:"),
        "lasm-smoke output should report success summary:\n{stdout}"
    );
    assert!(
        stdout.contains("requests=5"),
        "lasm-smoke output should include requested execution count:\n{stdout}"
    );
    assert!(
        stdout.contains("ok=5") && stdout.contains("errors=0"),
        "lasm-smoke output should include deterministic success/error counts:\n{stdout}"
    );
    assert!(
        stdout.contains("entry=main"),
        "lasm-smoke output should include selected entrypoint:\n{stdout}"
    );
    assert!(
        stdout.contains("origin=handler:health"),
        "lasm-smoke output should include matched handler origin:\n{stdout}"
    );
    assert!(
        stdout.contains("status=200"),
        "lasm-smoke output should include extracted handler status:\n{stdout}"
    );
    assert!(
        stdout.contains("body=smoke body"),
        "lasm-smoke output should include extracted handler response body:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_resolves_routes_and_response_through_helper_calls() {
    let root = temp_dir("sec4-lasm-smoke-helper-call-graph");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-helper-call-graph\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn emit_error() effects { net } -> Int {\n  res.text(503, \"helper response\");\n  0\n}\n\nfn health() effects { net } -> Int {\n  emit_error();\n  0\n}\n\nfn register_routes(router: Router) effects { net } -> Int {\n  http.get(router, \"/probe\", health);\n  0\n}\n\nfn main() effects { net } -> Int {\n  let router = http.router();\n  register_routes(router);\n  0\n}\n",
    )
    .expect("entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "lasm-smoke",
        "--path",
        &project_path,
        "--method",
        "GET",
        "--route",
        "/probe",
        "--requests",
        "3",
        "--max-steps",
        "64",
    ]);
    assert!(output.status.success(), "lasm-smoke command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("origin=handler:health"),
        "lasm-smoke output should include matched handler origin:\n{stdout}"
    );
    assert!(
        stdout.contains("body=helper response"),
        "lasm-smoke output should include helper-extracted response body:\n{stdout}"
    );
    assert!(
        stdout.contains("status=503"),
        "lasm-smoke output should include helper-extracted status:\n{stdout}"
    );
    assert!(
        stdout.contains("ok=0") && stdout.contains("errors=3"),
        "lasm-smoke output should classify helper-derived 5xx responses as errors:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_extracts_res_ok_status_and_body() {
    let root = temp_dir("sec4-lasm-smoke-res-ok");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-res-ok\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn create_user() effects { net } -> Int {\n  res.ok(201, \"CreateUserResponse\", 1);\n  0\n}\n\nfn main() effects { net } -> Int {\n  let router = http.router();\n  http.post(router, \"/users\", create_user);\n  0\n}\n",
    )
    .expect("entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "lasm-smoke",
        "--path",
        &project_path,
        "--method",
        "POST",
        "--route",
        "/users",
        "--requests",
        "2",
        "--max-steps",
        "64",
    ]);
    assert!(output.status.success(), "lasm-smoke command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("origin=handler:create_user"),
        "lasm-smoke output should include matched handler origin:\n{stdout}"
    );
    assert!(
        stdout.contains("status=201"),
        "lasm-smoke output should include extracted res.ok status:\n{stdout}"
    );
    assert!(
        stdout.contains("body=ok response"),
        "lasm-smoke output should include res.ok-derived body marker:\n{stdout}"
    );
    assert!(
        stdout.contains("ok=2") && stdout.contains("errors=0"),
        "lasm-smoke output should classify res.ok status as success:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_emits_json_summary_when_requested() {
    let root = temp_dir("sec4-lasm-smoke-json-summary");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-json-summary\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn health() effects { net } -> Int {\n  let name = headers.name(\"X-Test\");\n  let value = headers.value(\"active\");\n  res.setHeader(name, value);\n  res.ok(201, \"CreateUserResponse\", 1);\n  0\n}\n\nfn main() effects { net } -> Int {\n  let router = http.router();\n  http.post(router, \"/users\", health);\n  0\n}\n",
    )
    .expect("entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "lasm-smoke",
        "--path",
        &project_path,
        "--method",
        "POST",
        "--route",
        "/users",
        "--requests",
        "2",
        "--max-steps",
        "64",
        "--format",
        "json",
    ]);
    assert!(output.status.success(), "lasm-smoke command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("lasm-smoke json output should be valid json");
    assert_eq!(
        parsed.get("ok").and_then(serde_json::Value::as_bool),
        Some(true),
        "lasm-smoke json should signal success"
    );
    assert_eq!(
        parsed.get("origin").and_then(serde_json::Value::as_str),
        Some("handler:health"),
        "lasm-smoke json should include resolved handler origin"
    );
    assert_eq!(
        parsed.get("status").and_then(serde_json::Value::as_u64),
        Some(201),
        "lasm-smoke json should include extracted status code"
    );
    assert_eq!(
        parsed.get("okCount").and_then(serde_json::Value::as_u64),
        Some(2),
        "lasm-smoke json should include success count"
    );
    assert_eq!(
        parsed.get("errorCount").and_then(serde_json::Value::as_u64),
        Some(0),
        "lasm-smoke json should include error count"
    );
    assert_eq!(
        parsed
            .get("headers")
            .and_then(|headers| headers.get("X-Test"))
            .and_then(serde_json::Value::as_str),
        Some("active"),
        "lasm-smoke json should include extracted response headers"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_captures_path_params_with_request_path_override() {
    let root = temp_dir("sec4-lasm-smoke-path-params");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-path-params\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn get_user() effects { net } -> Int {\n  res.text(200, \"user\");\n  0\n}\n\nfn main() effects { net } -> Int {\n  let router = http.router();\n  http.get(router, \"/users/:id\", get_user);\n  0\n}\n",
    )
    .expect("entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "lasm-smoke",
        "--path",
        &project_path,
        "--method",
        "GET",
        "--route",
        "/users/:id",
        "--request-path",
        "/users/42",
        "--requests",
        "1",
        "--max-steps",
        "64",
        "--format",
        "json",
    ]);
    assert!(output.status.success(), "lasm-smoke command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("lasm-smoke json output should be valid json");
    assert_eq!(
        parsed
            .get("requestPath")
            .and_then(serde_json::Value::as_str),
        Some("/users/42"),
        "lasm-smoke json should report resolved request path"
    );
    assert_eq!(
        parsed
            .get("pathParams")
            .and_then(|params| params.get("id"))
            .and_then(serde_json::Value::as_str),
        Some("42"),
        "lasm-smoke json should include captured path parameter values"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_rejects_zero_step_budget() {
    let root = temp_dir("sec4-lasm-smoke-zero-budget");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-zero-budget\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["lasm-smoke", "--path", &project_path, "--max-steps", "0"]);
    assert!(
        !output.status.success(),
        "lasm-smoke should fail with zero step budget"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("--max-steps must be >= 1"),
        "lasm-smoke failure should mention invalid step budget:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_rejects_zero_request_count() {
    let root = temp_dir("sec4-lasm-smoke-zero-requests");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-zero-requests\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["lasm-smoke", "--path", &project_path, "--requests", "0"]);
    assert!(
        !output.status.success(),
        "lasm-smoke should fail with zero request count"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("--requests must be >= 1"),
        "lasm-smoke failure should mention invalid request count:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_fails_when_step_budget_is_too_low_for_batch() {
    let root = temp_dir("sec4-lasm-smoke-step-budget-too-low");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-step-budget-too-low\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "lasm-smoke",
        "--path",
        &project_path,
        "--requests",
        "8",
        "--max-steps",
        "1",
    ]);
    assert!(
        !output.status.success(),
        "lasm-smoke should fail when step budget is too low for request batch"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("runtime remained active after step budget"),
        "lasm-smoke should emit deterministic step-budget failure:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_rejects_zero_max_in_flight() {
    let root = temp_dir("sec4-lasm-smoke-zero-max-in-flight");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-zero-max-in-flight\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "lasm-smoke",
        "--path",
        &project_path,
        "--max-in-flight",
        "0",
    ]);
    assert!(
        !output.status.success(),
        "lasm-smoke should fail for zero max in-flight limit"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "invalid max in-flight should return deterministic usage-style exit code"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("invalid max in-flight limit: must be >= 1"),
        "lasm-smoke failure should mention deterministic max in-flight guidance:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_reports_effective_max_in_flight_in_text_summary() {
    let root = temp_dir("sec4-lasm-smoke-reports-max-in-flight");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-reports-max-in-flight\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn health() effects { net } -> Int {\n  res.text(200, \"smoke body\");\n  0\n}\n\nfn main() effects { net } -> Int {\n  let router = http.router();\n  http.get(router, \"/probe\", health);\n  0\n}\n",
    )
    .expect("entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "lasm-smoke",
        "--path",
        &project_path,
        "--method",
        "GET",
        "--route",
        "/probe",
        "--max-in-flight",
        "1",
        "--requests",
        "2",
        "--max-steps",
        "64",
    ]);
    assert!(output.status.success(), "lasm-smoke command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("maxInFlight=1"),
        "lasm-smoke text summary should include effective max in-flight limit:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_internal_net_is_disabled_by_policy() {
    let root = temp_dir("sec4-check-internal-net-disabled");
    let project_dir = root.join("policy-disabled-project");
    write_minimal_project(&project_dir, "");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn call_internal(cap: InternalNetCap, url: InternalUrl) effects { net } -> Int {
  httpClient.getInternal(cap, url);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when internal net policy is disabled"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("internal net is disabled by active policy"),
        "failure should include internal-net policy diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("tags: security, policy"),
        "failure should include policy/security tags:\n{stderr}"
    );
    assert!(
        stderr.contains("[net.internal]") && stderr.contains("enabled = true"),
        "failure should include policy enable guidance snippet:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_succeeds_when_internal_net_policy_is_enabled() {
    let root = temp_dir("sec4-check-internal-net-enabled");
    let project_dir = root.join("policy-enabled-project");
    write_minimal_project(
        &project_dir,
        r#"[net.internal]
enabled = true
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn call_internal(cap: InternalNetCap, url: InternalUrl) effects { net } -> Int {
  httpClient.getInternal(cap, url);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        output.status.success(),
        "check should succeed when internal net policy is enabled and typed usage is valid"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("check succeeded"),
        "check output should confirm success:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_browser_profile_uses_server_only_intrinsics() {
    let root = temp_dir("sec4-check-browser-profile-deny");
    let project_dir = root.join("browser-profile-deny-project");
    write_minimal_project(
        &project_dir,
        r#"[net.internal]
enabled = true
"#,
    );
    write_manifest_with_profile(&project_dir, "browser");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn do_work(db: DbCap, secrets: SecretsCap, internal: InternalNetCap, internal_url: InternalUrl) effects { db.write, net, secrets.read } -> Int {
  let q = sql.q("SELECT 1", 1);
  db.exec(db, q);
  secrets.get(secrets, "API_KEY");
  httpClient.getInternal(internal, internal_url);
  http.serve(8080, http.router());
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when browser profile uses server-only intrinsics"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("database intrinsics are disabled in browser profile"),
        "failure should include db browser-profile diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("secrets intrinsics are disabled in browser profile"),
        "failure should include secrets browser-profile diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("internal network intrinsics are disabled in browser profile"),
        "failure should include internal-net browser-profile diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("inbound network listener is disabled in browser profile"),
        "failure should include inbound-listener browser-profile diagnostic:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_succeeds_when_browser_profile_uses_allowed_public_net_intrinsics() {
    let root = temp_dir("sec4-check-browser-profile-allow");
    let project_dir = root.join("browser-profile-allow-project");
    write_minimal_project(&project_dir, "");
    write_manifest_with_profile(&project_dir, "browser");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { net } -> Int {
  let raw = req.query("https://api.example.com/users");
  url.public(raw);
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        output.status.success(),
        "check should succeed when browser profile uses allowed intrinsics"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_manifest_profile_is_invalid() {
    let root = temp_dir("sec4-check-invalid-manifest-profile");
    let project_dir = root.join("invalid-manifest-profile-project");
    write_minimal_project(&project_dir, "");
    write_manifest_with_profile(&project_dir, "mobile");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when manifest build.profile is invalid"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("M0005"),
        "failure should include deterministic manifest profile code:\n{stderr}"
    );
    assert!(
        stderr.contains("build.profile must be one of `server` or `browser`"),
        "failure should include invalid profile guidance:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_browser_profile_uses_server_capability_types_and_constructors() {
    let root = temp_dir("sec4-check-browser-profile-server-cap-types");
    let project_dir = root.join("browser-profile-server-cap-types-project");
    write_minimal_project(&project_dir, "");
    write_manifest_with_profile(&project_dir, "browser");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn use_db_cap(cap: DbCap) -> Int {
  0
}

fn main() -> Int {
  DbCap();
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when browser profile uses server capability types/constructors"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("server-only capability type is disabled in browser profile"),
        "failure should include server capability type fence diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("server-only capability constructor is disabled in browser profile"),
        "failure should include server capability constructor fence diagnostic:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_url_internal_gate_is_disabled_by_policy() {
    let root = temp_dir("sec4-check-url-internal-disabled");
    let project_dir = root.join("url-internal-disabled-project");
    write_minimal_project(&project_dir, "");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { net } -> Int {
  let raw = req.query("http://127.0.0.1/service");
  url.internal(raw);
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when internal url gate is disabled by policy"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("internal net is disabled by active policy"),
        "failure should include internal-net policy diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("tags: security, policy"),
        "failure should include policy/security tags:\n{stderr}"
    );
    assert!(
        stderr.contains("[net.internal]") && stderr.contains("enabled = true"),
        "failure should include policy enable guidance snippet:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_succeeds_when_url_internal_gate_is_enabled_by_policy() {
    let root = temp_dir("sec4-check-url-internal-enabled");
    let project_dir = root.join("url-internal-enabled-project");
    write_minimal_project(
        &project_dir,
        r#"[net.internal]
enabled = true
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { net } -> Int {
  let raw = req.query("http://127.0.0.1/service");
  url.internal(raw);
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        output.status.success(),
        "check should succeed when internal url gate policy is enabled"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_sql_select_has_no_limit_under_enforce_policy() {
    let root = temp_dir("sec4-check-sql-limit-enforce");
    let project_dir = root.join("sql-limit-enforce-project");
    write_minimal_project(
        &project_dir,
        r#"[sql]
require_limit_on_select = "enforce"
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() -> Int {
  sql.q("SELECT id FROM users", 1);
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when enforce policy requires LIMIT on SELECT"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("SELECT query without LIMIT violates active SQL policy"),
        "failure should include SQL LIMIT policy diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("tags: security, policy"),
        "failure should include policy/security tags:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_succeeds_when_sql_limit_policy_is_off() {
    let root = temp_dir("sec4-check-sql-limit-off");
    let project_dir = root.join("sql-limit-off-project");
    write_minimal_project(
        &project_dir,
        r#"[sql]
require_limit_on_select = "off"
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() -> Int {
  sql.q("SELECT id FROM users", 1);
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        output.status.success(),
        "check should succeed when SQL LIMIT policy is off"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_fs_is_disabled_by_policy() {
    let root = temp_dir("sec4-check-fs-disabled");
    let project_dir = root.join("policy-disabled-project");
    write_minimal_project(&project_dir, "");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn read_and_write(fs: FsCap, path: PathSafe) effects { fs.read, fs.write } -> Int {
  fs.read(fs, path);
  fs.write(fs, path, 1);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when filesystem policy is disabled"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("filesystem is disabled by active policy"),
        "failure should include filesystem policy diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("tags: security, policy"),
        "failure should include policy/security tags:\n{stderr}"
    );
    assert!(
        stderr.contains("[fs]") && stderr.contains("enabled = true"),
        "failure should include policy enable guidance snippet:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_succeeds_when_fs_policy_is_enabled() {
    let root = temp_dir("sec4-check-fs-enabled");
    let project_dir = root.join("policy-enabled-project");
    write_minimal_project(
        &project_dir,
        r#"[fs]
enabled = true
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn read_and_write(fs: FsCap, path: PathSafe) effects { fs.read, fs.write } -> Int {
  fs.read(fs, path);
  fs.write(fs, path, 1);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        output.status.success(),
        "check should succeed when filesystem policy is enabled and typed usage is valid"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("check succeeded"),
        "check output should confirm success:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_public_url_literal_scheme_is_disallowed_by_policy() {
    let root = temp_dir("sec4-check-public-url-scheme-disallowed");
    let project_dir = root.join("policy-disallowed-scheme-project");
    write_minimal_project(
        &project_dir,
        r#"[net.public]
allowed_schemes = ["https"]
allowed_domains = []
blocked_domains = []
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { net } -> Int {
  url.public("http://api.example.com/users");
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when public URL scheme is blocked by policy"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("E2002") && stderr.contains("public URL literal violates active policy"),
        "failure should include deterministic E2002 policy diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("scheme `http` is not allowed by `[net.public].allowed_schemes`"),
        "failure should explain disallowed scheme:\n{stderr}"
    );
    assert!(
        stderr.contains("tags: security, policy"),
        "failure should include policy/security tags:\n{stderr}"
    );
    assert!(
        stderr.contains("[net.public]") && stderr.contains("sec4.policy"),
        "failure should include fix guidance pointing to [net.public] in sec4.policy:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_public_url_literal_host_is_blocked_by_policy() {
    let root = temp_dir("sec4-check-public-url-blocked-domain");
    let project_dir = root.join("policy-blocked-domain-project");
    write_minimal_project(
        &project_dir,
        r#"[net.public]
allowed_schemes = ["https"]
allowed_domains = []
blocked_domains = ["blocked.example.com"]
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { net } -> Int {
  url.public("https://blocked.example.com/private");
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when host is blocked by net.public policy"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("E2002") && stderr.contains("public URL literal violates active policy"),
        "failure should include deterministic E2002 policy diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("host `blocked.example.com` is blocked by `[net.public].blocked_domains`"),
        "failure should explain blocked host:\n{stderr}"
    );
    assert!(
        stderr.contains("tags: security, policy"),
        "failure should include policy/security tags:\n{stderr}"
    );
    assert!(
        stderr.contains("[net.public]") && stderr.contains("sec4.policy"),
        "failure should include fix guidance pointing to [net.public] in sec4.policy:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_public_url_literal_host_is_outside_allowlist() {
    let root = temp_dir("sec4-check-public-url-allowlist");
    let project_dir = root.join("policy-allowlist-project");
    write_minimal_project(
        &project_dir,
        r#"[net.public]
allowed_schemes = ["https"]
allowed_domains = ["allowed.example.com"]
blocked_domains = []
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { net } -> Int {
  url.public("https://outside.example.com/private");
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when literal host is outside allowed_domains"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("E2002") && stderr.contains("public URL literal violates active policy"),
        "failure should include deterministic E2002 policy diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("host `outside.example.com` is outside `[net.public].allowed_domains`"),
        "failure should explain allowlist mismatch:\n{stderr}"
    );
    assert!(
        stderr.contains("tags: security, policy"),
        "failure should include policy/security tags:\n{stderr}"
    );
    assert!(
        stderr.contains("[net.public]") && stderr.contains("sec4.policy"),
        "failure should include fix guidance pointing to [net.public] in sec4.policy:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_fails_when_public_url_literal_port_is_disallowed_by_policy() {
    let root = temp_dir("sec4-check-public-url-port-disallowed");
    let project_dir = root.join("policy-port-disallowed-project");
    write_minimal_project(
        &project_dir,
        r#"[net.public]
allowed_schemes = ["https"]
allowed_domains = []
blocked_domains = []
allowed_ports = [443]
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { net } -> Int {
  url.public("https://api.example.com:8443/private");
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "check should fail when literal port is outside net.public allowed_ports"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("E2002") && stderr.contains("public URL literal violates active policy"),
        "failure should include deterministic E2002 policy diagnostic:\n{stderr}"
    );
    assert!(
        stderr.contains("port `8443` is not allowed by `[net.public].allowed_ports`"),
        "failure should explain disallowed port:\n{stderr}"
    );
    assert!(
        stderr.contains("tags: security, policy"),
        "failure should include policy/security tags:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_succeeds_when_public_url_literal_port_is_allowed_by_policy() {
    let root = temp_dir("sec4-check-public-url-port-allowed");
    let project_dir = root.join("policy-port-allowed-project");
    write_minimal_project(
        &project_dir,
        r#"[net.public]
allowed_schemes = ["https"]
allowed_domains = []
blocked_domains = []
allowed_ports = [8443]
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { net } -> Int {
  url.public("https://api.example.com:8443/private");
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        output.status.success(),
        "check should succeed when literal port is listed in net.public allowed_ports"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("check succeeded"),
        "check output should confirm success:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn check_succeeds_for_allowed_public_url_policy_usage() {
    let root = temp_dir("sec4-check-public-url-policy-allowed");
    let project_dir = root.join("policy-allowed-project");
    write_minimal_project(
        &project_dir,
        r#"[net.public]
allowed_schemes = ["https"]
allowed_domains = ["allowed.example.com"]
blocked_domains = ["blocked.example.com"]
"#,
    );
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn use_public(raw: Untrusted<String>) effects { net } -> Int {
  url.public(raw);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &project_path]);
    assert!(
        output.status.success(),
        "check should succeed when public URL policy is configured and usage remains valid"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("check succeeded"),
        "check output should confirm success:\n{stdout}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn init_then_run_oneshot_serves_request_and_exits() {
    if !clang_available() {
        eprintln!("skipping init run-command oneshot integration test: clang not available");
        return;
    }

    let root = temp_dir("sec4-init-run-oneshot");
    let project_dir = root.join("run-project");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let port = find_available_tcp_port();

    let init_output = run_cli(&[
        "init",
        "--path",
        &project_path,
        "--name",
        "init-fixture-run",
    ]);
    assert!(init_output.status.success(), "init command should succeed");

    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            project_path.as_str(),
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("init oneshot test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("init oneshot process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command oneshot process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nhello from sec4"),
        "response should include expected bootstrap body:\n{response}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn init_fails_deterministically_on_existing_initialized_project() {
    let project_dir = temp_dir("sec4-init-existing-project");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let first = run_cli(&[
        "init",
        "--path",
        &project_path,
        "--name",
        "init-fixture-existing",
    ]);
    assert!(first.status.success(), "first init command should succeed");

    let second = run_cli(&["init", "--path", &project_path]);
    assert!(
        !second.status.success(),
        "init command should fail for already initialized project"
    );
    assert_eq!(
        second.status.code(),
        Some(1),
        "re-initialization should exit with deterministic status code"
    );

    let stderr = String::from_utf8(second.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("already contains sec4.toml"),
        "init failure should clearly indicate existing initialized project:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn test_command_success_path() {
    if !clang_available() {
        eprintln!("skipping test-command success integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-test-command-success");
    write_minimal_project(
        &project_dir,
        r#"[security_headers.csp]
enabled = true
report_only = false
"#,
    );
    fs::create_dir_all(project_dir.join("tests/nested")).expect("nested tests dir should exist");
    fs::write(
        project_dir.join("tests/alpha.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("alpha test should be written");
    fs::write(
        project_dir.join("tests/nested/beta.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("beta test should be written");

    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["test", "--path", &project_path]);
    assert!(output.status.success(), "test command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("test summary: discovered=2")
            && stdout.contains(
                "static_passed=2, static_failed=0, runtime_passed=2, runtime_failed=0, skipped_non_entry=0"
            ),
        "test summary should report all tests passing:\n{stdout}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn test_command_fails_for_invalid_test_file() {
    let project_dir = temp_dir("sec4-test-command-invalid");
    write_minimal_project(
        &project_dir,
        r#"[security_headers.csp]
enabled = true
report_only = false
"#,
    );
    fs::create_dir_all(project_dir.join("tests")).expect("tests dir should exist");
    fs::write(
        project_dir.join("tests/bad.ut"),
        "fn bad( -> Int {\n  0\n}\n",
    )
    .expect("bad test should be written");

    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["test", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "test command should fail when one test file has diagnostics"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("test summary: discovered=1")
            && stdout.contains(
                "static_passed=0, static_failed=1, runtime_passed=0, runtime_failed=0, skipped_non_entry=0"
            ),
        "test summary should report one static failing test:\n{stdout}"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("tests/bad.ut"),
        "diagnostics should reference the invalid test file:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn test_command_fails_when_test_runtime_exits_non_zero() {
    if !clang_available() {
        eprintln!("skipping test-command runtime failure integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-test-command-runtime-failure");
    write_minimal_project(
        &project_dir,
        r#"[security_headers.csp]
enabled = true
report_only = false
"#,
    );
    fs::create_dir_all(project_dir.join("tests")).expect("tests dir should exist");
    fs::write(
        project_dir.join("tests/pass.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("passing test should be written");
    fs::write(
        project_dir.join("tests/fail.ut"),
        "fn main() -> Int {\n  3\n}\n",
    )
    .expect("failing test should be written");

    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["test", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "test command should fail when one runtime exits non-zero"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("test summary: discovered=2")
            && stdout.contains(
                "static_passed=2, static_failed=0, runtime_passed=1, runtime_failed=1, skipped_non_entry=0"
            ),
        "test summary should report one runtime failing test:\n{stdout}"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("tests/fail.ut"),
        "runtime failure should reference the failing test file:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn test_command_supports_multi_file_test_modules() {
    if !clang_available() {
        eprintln!("skipping test-command module graph integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-test-command-modules");
    write_minimal_project(
        &project_dir,
        r#"[security_headers.csp]
enabled = true
report_only = false
"#,
    );
    fs::create_dir_all(project_dir.join("tests/shared"))
        .expect("nested test module directory should exist");
    fs::write(
        project_dir.join("tests/main.ut"),
        "use shared.helper;\n\nfn main() -> Int {\n  helper();\n  0\n}\n",
    )
    .expect("test entry should be written");
    fs::write(
        project_dir.join("tests/shared/helper.ut"),
        "fn helper() -> Int {\n  0\n}\n",
    )
    .expect("helper module should be written");

    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["test", "--path", &project_path]);
    assert!(
        output.status.success(),
        "test command should succeed for module-based test entry"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("test summary: discovered=2")
            && stdout.contains(
                "static_passed=1, static_failed=0, runtime_passed=1, runtime_failed=0, skipped_non_entry=1"
            ),
        "test summary should report one module-based test passing:\n{stdout}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn test_command_reports_missing_module_in_test_graph() {
    let project_dir = temp_dir("sec4-test-command-module-missing");
    write_minimal_project(
        &project_dir,
        r#"[security_headers.csp]
enabled = true
report_only = false
"#,
    );
    fs::create_dir_all(project_dir.join("tests")).expect("tests directory should exist");
    fs::write(
        project_dir.join("tests/main.ut"),
        "use shared.missing;\n\nfn main() -> Int {\n  0\n}\n",
    )
    .expect("test entry should be written");

    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["test", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "test command should fail when imported test module is missing"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("test summary: discovered=1")
            && stdout.contains(
                "static_passed=0, static_failed=1, runtime_passed=0, runtime_failed=0, skipped_non_entry=0"
            ),
        "test summary should report one static failing test:\n{stdout}"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("M0303") && stderr.contains("missing module: shared.missing"),
        "missing module diagnostic should be surfaced deterministically:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn test_command_fails_when_no_runnable_test_entrypoints_exist() {
    let project_dir = temp_dir("sec4-test-command-no-main-entries");
    write_minimal_project(
        &project_dir,
        r#"[security_headers.csp]
enabled = true
report_only = false
"#,
    );
    fs::create_dir_all(project_dir.join("tests/shared"))
        .expect("shared test module directory should exist");
    fs::write(
        project_dir.join("tests/shared/helper.ut"),
        "fn helper() -> Int {\n  0\n}\n",
    )
    .expect("helper module should be written");

    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["test", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "test command should fail when no runnable `main` test entries exist"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "missing runnable test entries should fail with deterministic code"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("no runnable test entrypoints found"),
        "failure should clearly explain missing runnable test entries:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn fmt_rewrites_trailing_whitespace() {
    let project_dir = temp_dir("sec4-fmt-command");
    fs::create_dir_all(project_dir.join("src")).expect("src dir should exist");
    let source_path = project_dir.join("src/main.ut");
    fs::write(&source_path, "fn main() -> Int {   \n  0   \n}")
        .expect("source fixture should be written");

    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["fmt", "--path", &project_path]);
    assert!(output.status.success(), "fmt command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("fmt summary: changed=1"),
        "fmt summary should report one changed file:\n{stdout}"
    );

    let formatted = fs::read_to_string(&source_path).expect("formatted source should be readable");
    assert_eq!(formatted, "fn main() -> Int {\n  0\n}\n");

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn lint_fails_on_high_risk_policy_fixture() {
    let project_dir = temp_dir("sec4-lint-command-high-risk");
    write_minimal_project(
        &project_dir,
        r#"[security_headers.csp]
enabled = false
report_only = false
"#,
    );

    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["lint", "--path", &project_path]);
    assert!(
        !output.status.success(),
        "lint should fail on HIGH-risk security audit findings"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("security audit failed: findings at or above threshold HIGH"),
        "lint should fail at HIGH threshold:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_serves_request_and_exits() {
    if !clang_available() {
        eprintln!("skipping run-command oneshot integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-oneshot");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runoneshotcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "pong");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--tls-backend",
            "auto",
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command oneshot test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command oneshot process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command oneshot process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npong"),
        "response should include expected body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_openssl_backend_serves_request_and_exits() {
    if !clang_available() {
        eprintln!("skipping run-command openssl oneshot test: clang not available");
        return;
    }
    if !clang_with_openssl_available() {
        eprintln!(
            "skipping run-command openssl oneshot test: OpenSSL headers/libs not available to clang"
        );
        return;
    }

    let project_dir = temp_dir("sec4-run-command-openssl-oneshot");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runopenssloneshotcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "pong");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--tls-backend",
            "openssl",
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command openssl oneshot test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command openssl oneshot process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command openssl oneshot process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npong"),
        "response should include expected body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_applies_cors_from_policy() {
    if !clang_available() {
        eprintln!("skipping run-command cors policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-cors-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runcorspolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[cors]
enabled = true
allowed_origins = ["https://frontend.example"]
allow_credentials = true
exposed_headers = ["x-trace-id"]
require_vary_origin = true
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "pong");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let router = cors.withCors(router, cors.fromPolicy());
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nOrigin: https://frontend.example\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command cors policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command cors policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command cors policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: https://frontend.example"),
        "response should include policy-driven allow origin header:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Credentials: true"),
        "response should include policy-driven credentials header:\n{response}"
    );
    assert!(
        response.contains("Vary: Origin"),
        "response should include Vary: Origin when policy requires it:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Expose-Headers: x-trace-id"),
        "response should include policy-driven expose headers:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_allows_private_network_preflight_when_cors_policy_enables_it() {
    if !clang_available() {
        eprintln!("skipping run-command cors private-network policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-cors-private-network-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runcorsprivatenetworkpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[cors]
enabled = true
allowed_origins = ["https://frontend.example"]
allow_credentials = true
allow_private_network = true
require_vary_origin = true
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn createUser() effects { net } -> Int {
  res.text(201, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let router = cors.withCors(router, cors.fromPolicy());
  http.post(router, "/users", createUser);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"OPTIONS /users HTTP/1.1\r\nHost: localhost\r\nOrigin: https://frontend.example\r\nAccess-Control-Request-Method: POST\r\nAccess-Control-Request-Private-Network: true\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command cors private-network policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "run command cors private-network policy process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command cors private-network policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 status line for private-network preflight:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: https://frontend.example"),
        "response should include policy-driven allow-origin header:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Private-Network: true"),
        "response should include private-network allow header from policy bridge:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_applies_cors_preflight_methods_headers_and_max_age_from_policy() {
    if !clang_available() {
        eprintln!("skipping run-command cors preflight policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-cors-preflight-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runcorspreflightpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[cors]
enabled = true
allowed_origins = ["https://frontend.example"]
allowed_methods = ["POST"]
allowed_headers = ["x-auth-token"]
max_age_seconds = 7200
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn createUser() effects { net } -> Int {
  res.text(201, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let router = cors.withCors(router, cors.fromPolicy());
  http.post(router, "/users", createUser);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"OPTIONS /users HTTP/1.1\r\nHost: localhost\r\nOrigin: https://frontend.example\r\nAccess-Control-Request-Method: POST\r\nAccess-Control-Request-Headers: x-auth-token\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command cors preflight policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command cors preflight policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command cors preflight policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 status line for cors preflight:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: https://frontend.example"),
        "response should include policy-driven allow-origin header:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Methods: POST"),
        "response should include policy-driven allowed methods:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Headers: x-auth-token"),
        "response should include policy-driven allowed headers:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Max-Age: 7200"),
        "response should include policy-driven max-age:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_applies_security_headers_hsts_and_csp_from_policy() {
    if !clang_available() {
        eprintln!(
            "skipping run-command security-headers hsts/csp policy test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-run-command-security-headers-hsts-csp-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runsecurityheadershstscsppolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[security_headers]
enabled = true
x_content_type_options = true
x_frame_options = "DENY"
referrer_policy = "strict-origin"

[security_headers.hsts]
enabled = true
max_age_seconds = 777
include_subdomains = false
preload = false

[security_headers.csp]
enabled = true
report_only = true
policy = "default-src 'none'; frame-ancestors 'none'"
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "pong");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let router = sec.withSecurityHeaders(router, sec.defaultHeaders());
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command security-headers hsts/csp policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command security-headers hsts/csp policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command security-headers hsts/csp policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("Strict-Transport-Security: max-age=777"),
        "response should include policy-driven HSTS max-age header:\n{response}"
    );
    assert!(
        response.contains(
            "Content-Security-Policy-Report-Only: default-src 'none'; frame-ancestors 'none'"
        ),
        "response should include policy-driven CSP report-only header:\n{response}"
    );
    assert!(
        response.contains("Referrer-Policy: strict-origin"),
        "response should include policy-driven referrer policy header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_disables_security_headers_from_policy() {
    if !clang_available() {
        eprintln!("skipping run-command security-headers policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-security-headers-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runsecurityheaderspolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[security_headers]
enabled = false
x_content_type_options = false
x_frame_options = "DENY"
referrer_policy = "strict-origin-when-cross-origin"

[security_headers.hsts]
enabled = false
max_age_seconds = 0
include_subdomains = false
preload = false

[security_headers.csp]
enabled = false
report_only = false
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "pong");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let router = sec.withSecurityHeaders(router, sec.defaultHeaders());
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command security-headers policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command security-headers policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command security-headers policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        !response.contains("X-Content-Type-Options:"),
        "response should omit nosniff header when policy disables security headers:\n{response}"
    );
    assert!(
        !response.contains("X-Frame-Options:"),
        "response should omit frame options header when policy disables security headers:\n{response}"
    );
    assert!(
        !response.contains("Referrer-Policy:"),
        "response should omit referrer policy header when policy disables security headers:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_disables_csrf_from_policy() {
    if !clang_available() {
        eprintln!("skipping run-command csrf policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-csrf-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runcsrfpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[csrf]
enabled = false
mode = "off"
same_site = "Lax"
secure_cookie = true
protected_methods = ["POST","PUT","PATCH","DELETE"]
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn submit() effects { net } -> Int {
  res.text(204, "");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let router = csrf.withCsrf(router, csrf.fromPolicy());
  http.post(router, "/submit", submit);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /submit HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command csrf policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command csrf policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command csrf policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 status line when csrf policy disables checks:\n{response}"
    );
    assert!(
        !response.contains("\"code\":\"CSRF.TOKEN_INVALID\""),
        "response should not include csrf token error when policy disables csrf:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_applies_csrf_cookie_and_header_names_from_policy() {
    if !clang_available() {
        eprintln!("skipping run-command csrf cookie/header policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-csrf-cookie-header-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runcsrfcookieheaderpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[csrf]
enabled = true
mode = "double_submit"
cookie_name = "sid"
header_name = "x-sid-csrf"
same_site = "Lax"
secure_cookie = true
protected_methods = ["POST","PUT","PATCH","DELETE"]
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn submit() effects { net } -> Int {
  res.text(204, "");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let router = csrf.withCsrf(router, csrf.fromPolicy());
  http.post(router, "/submit", submit);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /submit HTTP/1.1\r\nHost: localhost\r\nx-sid-csrf: token-1\r\nCookie: sid=token-1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command csrf cookie/header policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command csrf cookie/header policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command csrf cookie/header policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 status line when custom csrf names are respected:\n{response}"
    );
    assert!(
        !response.contains("\"code\":\"AUTH.CSRF_TOKEN_INVALID\""),
        "response should not include csrf token invalid error:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_applies_http_body_limit_from_policy() {
    if !clang_available() {
        eprintln!("skipping run-command http body-limit policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-http-body-limit-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runhttpbodylimitpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[http]
max_body_bytes = 32
default_timeout_ms = 20000
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn createUser() effects { net } -> Int {
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let payload = r#"{"name":"0123456789012345678901234567890123456789"}"#;
    let request = format!(
        "POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(request.as_bytes())
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command http body-limit policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command http body-limit policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command http body-limit policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 413 Payload Too Large"),
        "response should contain 413 status line when policy body limit is exceeded:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"LIMIT.BODY_BYTES\""),
        "response should include deterministic body-limit code:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_applies_http_header_limit_from_policy() {
    if !clang_available() {
        eprintln!("skipping run-command http header-limit policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-http-header-limit-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runhttpheaderlimitpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[http]
max_body_bytes = 4096
max_header_bytes = 128
default_timeout_ms = 20000
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let large_header_value = "a".repeat(220);
    let request = format!(
        "GET /health HTTP/1.1\r\nHost: localhost\r\nX-Fill: {}\r\nConnection: close\r\n\r\n",
        large_header_value
    );

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(request.as_bytes())
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command http header-limit policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command http header-limit policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command http header-limit policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 431 Request Header Fields Too Large"),
        "response should contain 431 status line when policy header limit is exceeded:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nrequest headers too large"),
        "response should include deterministic oversized-header message:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_applies_http_multipart_limit_from_policy() {
    if !clang_available() {
        eprintln!("skipping run-command http multipart-limit policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-http-multipart-limit-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runhttpmultipartlimitpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[http]
max_body_bytes = 4096
max_header_bytes = 8191
max_multipart_bytes = 64
default_timeout_ms = 20000
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn upload() effects { net } -> Int {
  res.text(201, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.post(router, "/upload", upload);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let boundary = "----sec4boundary";
    let multipart_body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.txt\"\r\nContent-Type: text/plain\r\n\r\n{}\r\n--{boundary}--\r\n",
        "a".repeat(120)
    );
    let request = format!(
        "POST /upload HTTP/1.1\r\nHost: localhost\r\nContent-Type: multipart/form-data; boundary={boundary}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        multipart_body.len(),
        multipart_body
    );

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(request.as_bytes())
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command http multipart-limit policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "run command http multipart-limit policy process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command http multipart-limit policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 413 Payload Too Large"),
        "response should contain 413 status line when policy multipart limit is exceeded:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nmultipart payload exceeds runtime limit"),
        "response should include deterministic multipart limit message:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_applies_http_max_concurrency_from_policy() {
    if !clang_available() {
        eprintln!("skipping run-command http max-concurrency policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-http-max-concurrency-policy");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runhttpmaxconcurrencypolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[http]
max_body_bytes = 4096
max_concurrency = 1
max_header_bytes = 8191
max_multipart_bytes = 4096
default_timeout_ms = 20000
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let request = b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";

    let mut matched = false;
    let mut last_responses = String::new();
    for attempt in 0..8 {
        let port = find_available_tcp_port();
        let port_value = port.to_string();
        let mut child = Command::new(cli_bin())
            .args([
                "run",
                "--path",
                path.as_str(),
                "--oneshot",
                "--port",
                port_value.as_str(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("sec4 run command should start");

        let first_handle = thread::spawn(move || {
            for _ in 0..800 {
                match TcpStream::connect(("127.0.0.1", port)) {
                    Ok(stream) => return Some(stream),
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            }
            None
        });
        let second_handle = thread::spawn(move || {
            for _ in 0..800 {
                match TcpStream::connect(("127.0.0.1", port)) {
                    Ok(stream) => return Some(stream),
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            }
            None
        });

        let mut first_stream = match first_handle
            .join()
            .expect("first connector thread should join")
        {
            Some(stream) => stream,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("run command max-concurrency test could not establish first connection");
            }
        };
        let mut second_stream = match second_handle
            .join()
            .expect("second connector thread should join")
        {
            Some(stream) => stream,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("run command max-concurrency test could not establish second connection");
            }
        };

        first_stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("first stream read timeout should be set");
        second_stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("second stream read timeout should be set");

        first_stream
            .write_all(request)
            .expect("first request should be written");
        second_stream
            .write_all(request)
            .expect("second request should be written");

        let mut first_response = String::new();
        let mut second_response = String::new();
        let _ = first_stream.read_to_string(&mut first_response);
        let _ = second_stream.read_to_string(&mut second_response);
        last_responses = format!(
            "attempt={attempt}\nfirst_response=\n{}\nsecond_response=\n{}",
            first_response, second_response
        );

        let mut status = None;
        for _ in 0..240 {
            match child.try_wait().expect("run command wait should succeed") {
                Some(next) => {
                    status = Some(next);
                    break;
                }
                None => thread::sleep(Duration::from_millis(25)),
            }
        }
        let status = match status {
            Some(status) => status,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("run command max-concurrency process did not exit in expected window");
            }
        };

        let has_success = first_response.contains("HTTP/1.1 200 OK")
            || second_response.contains("HTTP/1.1 200 OK");
        let has_throttle = first_response.contains("HTTP/1.1 503 Service Unavailable")
            || second_response.contains("HTTP/1.1 503 Service Unavailable");
        if status.success() && has_success && has_throttle {
            matched = true;
            break;
        }
    }

    assert!(
        matched,
        "response set should include both success and deterministic max-concurrency throttle response:\n{last_responses}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_enforces_body_limit_for_non_json_handler_paths() {
    if !clang_available() {
        eprintln!("skipping run-command generic body-limit test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-http-generic-body-limit-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runhttpgenericbodylimitpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[http]
max_body_bytes = 32
max_header_bytes = 8191
max_multipart_bytes = 4096
default_timeout_ms = 20000
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn upload() effects { net } -> Int {
  res.text(201, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.post(router, "/upload", upload);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let payload = "a".repeat(120);
    let request = format!(
        "POST /upload HTTP/1.1\r\nHost: localhost\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(request.as_bytes())
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command generic body-limit test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command generic body-limit process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command generic body-limit process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 413 Payload Too Large"),
        "response should contain 413 status line when generic body limit is exceeded:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nrequest body exceeds runtime limit"),
        "response should include deterministic generic body-limit message:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_cli_max_body_bytes_overrides_policy_limit() {
    if !clang_available() {
        eprintln!("skipping run-command http body-limit override test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-http-body-limit-override");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runhttpbodylimitoverridecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[http]
max_body_bytes = 32
default_timeout_ms = 20000
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn createUser() effects { net } -> Int {
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--max-body-bytes",
            "4096",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let payload = r#"{"name":"0123456789012345678901234567890123456789"}"#;
    let request = format!(
        "POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(request.as_bytes())
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command http body-limit override test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command http body-limit override process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command http body-limit override process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 201 Created"),
        "response should contain 201 status line when CLI body limit override applies:\n{response}"
    );
    assert!(
        !response.contains("\"code\":\"LIMIT.BODY_BYTES\""),
        "response should not include body-limit error when CLI override is higher:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_cli_serve_timeout_overrides_policy_timeout() {
    if !clang_available() {
        eprintln!("skipping run-command http timeout override test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-http-timeout-override");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runhttptimeoutoverridecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[http]
max_body_bytes = 4096
default_timeout_ms = 20000
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let started_at = Instant::now();
    let output = run_cli(&[
        "run",
        "--path",
        path,
        "--oneshot",
        "--serve-timeout-ms",
        "1",
    ]);

    assert!(
        output.status.success(),
        "run command should exit successfully when CLI timeout override is tiny"
    );
    assert!(
        started_at.elapsed() < Duration::from_secs(15),
        "run command should exit well before policy timeout when CLI timeout override is applied"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_cli_max_concurrency_overrides_policy_limit() {
    if !clang_available() {
        eprintln!("skipping run-command http max-concurrency override test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-http-max-concurrency-override");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runhttpmaxconcurrencyoverridecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[http]
max_body_bytes = 4096
max_concurrency = 1
max_header_bytes = 8191
max_multipart_bytes = 4096
default_timeout_ms = 20000
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let request = b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";

    let mut matched = false;
    let mut last_responses = String::new();
    for attempt in 0..8 {
        let port = find_available_tcp_port();
        let port_value = port.to_string();
        let mut child = Command::new(cli_bin())
            .args([
                "run",
                "--path",
                path.as_str(),
                "--port",
                port_value.as_str(),
                "--max-concurrency",
                "2",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("sec4 run command should start");

        let first_handle = thread::spawn(move || {
            for _ in 0..800 {
                match TcpStream::connect(("127.0.0.1", port)) {
                    Ok(stream) => return Some(stream),
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            }
            None
        });
        let second_handle = thread::spawn(move || {
            for _ in 0..800 {
                match TcpStream::connect(("127.0.0.1", port)) {
                    Ok(stream) => return Some(stream),
                    Err(_) => thread::sleep(Duration::from_millis(10)),
                }
            }
            None
        });

        let mut first_stream = match first_handle
            .join()
            .expect("first connector thread should join")
        {
            Some(stream) => stream,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("run command max-concurrency override test could not establish first connection");
            }
        };
        let mut second_stream = match second_handle
            .join()
            .expect("second connector thread should join")
        {
            Some(stream) => stream,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("run command max-concurrency override test could not establish second connection");
            }
        };

        first_stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("first stream read timeout should be set");
        second_stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("second stream read timeout should be set");

        first_stream
            .write_all(request)
            .expect("first request should be written");
        second_stream
            .write_all(request)
            .expect("second request should be written");

        let mut first_response = String::new();
        let mut second_response = String::new();
        let _ = first_stream.read_to_string(&mut first_response);
        let _ = second_stream.read_to_string(&mut second_response);
        last_responses = format!(
            "attempt={attempt}\nfirst_response=\n{}\nsecond_response=\n{}",
            first_response, second_response
        );

        if let Some(status) = child.try_wait().expect("run command wait should succeed") {
            last_responses = format!("{last_responses}\nprocess_status={status}");
            continue;
        }

        let _ = child.kill();
        let _ = child.wait();

        let first_success = first_response.contains("HTTP/1.1 200 OK");
        let second_success = second_response.contains("HTTP/1.1 200 OK");
        let has_throttle = first_response.contains("HTTP/1.1 503 Service Unavailable")
            || second_response.contains("HTTP/1.1 503 Service Unavailable");
        if first_success && second_success && !has_throttle {
            matched = true;
            break;
        }
    }

    assert!(
        matched,
        "responses should include two successful replies and no throttle when CLI max-concurrency override is higher than policy:\n{last_responses}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_applies_net_public_policy_env() {
    if !clang_available() {
        eprintln!("skipping run-command net.public policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-net-public-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runnetpublicpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[net.public]
allowed_schemes = ["https"]
blocked_domains = ["blocked.example"]
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let raw = req.query("https://blocked.example/resource");
  url.public(raw);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command net.public policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command net.public policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command net.public policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain 400 status line:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"NET.URL_PUBLIC_INVALID\""),
        "response should include deterministic URL policy code:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_rejects_public_url_when_port_is_not_allowed() {
    if !clang_available() {
        eprintln!("skipping run-command net.public port policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-net-public-port-disallowed");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runnetpublicportdisallowed"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[net.public]
allowed_schemes = ["https"]
allowed_domains = []
blocked_domains = ["blocked.example.com"]
allowed_ports = [443]
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let raw = req.query("https://blocked.example.com:8443/private");
  url.public(raw);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command net.public port policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command net.public port policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command net.public port policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain 400 status line:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"NET.URL_PUBLIC_INVALID\""),
        "response should include deterministic URL policy code:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_allows_public_url_when_port_is_allowed() {
    if !clang_available() {
        eprintln!("skipping run-command net.public allowed-port policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-net-public-port-allowed");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runnetpublicportallowed"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[net.public]
allowed_schemes = ["https"]
allowed_domains = ["api.example.com"]
blocked_domains = []
allowed_ports = [8443]
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let raw = req.query("https://api.example.com:8443/private");
  url.public(raw);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command net.public allowed-port policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "run command net.public allowed-port policy process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command net.public allowed-port policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 status line:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_allows_public_url_when_dns_resolution_is_disabled_by_policy() {
    if !clang_available() {
        eprintln!("skipping run-command dns-resolution policy test: clang not available");
        return;
    }
    if !localhost_dot_resolves() {
        eprintln!(
            "skipping run-command dns-resolution policy test: localhost. does not resolve here"
        );
        return;
    }

    let project_dir = temp_dir("sec4-run-command-net-public-dns-resolve-disabled");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runnetpublicdnsdisabled"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[net.public]
allowed_schemes = ["http", "https"]
allowed_domains = []
blocked_domains = []
allowed_ports = []

[net.ssrf]
resolve_dns = false
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let raw = req.query("http://localhost./private");
  url.public(raw);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command dns-resolution policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command dns-resolution policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command dns-resolution policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 status line when DNS resolve checks are disabled:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_allows_public_loopback_when_ssrf_block_toggles_are_disabled_by_policy() {
    if !clang_available() {
        eprintln!("skipping run-command ssrf block-toggle policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-net-public-ssrf-block-toggles");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runnetpublicssrftoggles"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[net.public]
allowed_schemes = ["http", "https"]
allowed_domains = []
blocked_domains = []
allowed_ports = []

[net.ssrf]
block_private_ranges = false
block_loopback = false
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let raw = req.query("http://127.0.0.1/private");
  url.public(raw);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command ssrf block-toggle policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command ssrf block-toggle policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command ssrf block-toggle policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 status line when ssrf block toggles are disabled:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_applies_net_internal_allowed_domains_policy_env() {
    if !clang_available() {
        eprintln!("skipping run-command net.internal policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-net-internal-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runnetinternalpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[net.internal]
enabled = true
allowed_domains = ["internal.service"]
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let raw = req.query("http://127.0.0.1/service");
  url.internal(raw);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command net.internal policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command net.internal policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command net.internal policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain 400 status line:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"NET.URL_INTERNAL_INVALID\""),
        "response should include deterministic internal URL policy code:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_allows_internal_url_when_cidr_allowlist_matches() {
    if !clang_available() {
        eprintln!("skipping run-command net.internal cidr policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-net-internal-cidr-policy");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runnetinternalcidrpolicycommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[net.internal]
enabled = true
allowed_domains = ["internal.service"]
allowed_cidrs = ["127.0.0.0/8"]
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let raw = req.query("http://127.0.0.1/service");
  url.internal(raw);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command net.internal cidr policy test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command net.internal cidr policy process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command net.internal cidr policy process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 status line:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_allows_cookie_auth_mode_request_and_exits() {
    if !clang_available() {
        eprintln!("skipping run-command cookie-auth integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-cookie-auth-oneshot");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runcookieauthoneshotcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[auth]
mode = "cookie"
cross_site_frontend = false

[auth.cookie]
cookie_name = "sid"
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "pong");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  let authCfg = auth.fromPolicy();
  let withAuth = auth.withAuth(router, authCfg);
  http.serve(8080, withAuth);
  0
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let port_value = port.to_string();
    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            &project_path,
            "--oneshot",
            "--port",
            &port_value,
            "--serve-timeout-ms",
            "20000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nCookie: sid=s123\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command cookie-auth test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..240 {
        match child.try_wait().expect("run command wait should succeed") {
            Some(next) => {
                status = Some(next);
                break;
            }
            None => thread::sleep(Duration::from_millis(25)),
        }
    }
    let status = match status {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command cookie-auth process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command cookie-auth process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npong"),
        "response should include expected route body:\n{response}"
    );
}

#[test]
fn run_command_fails_for_invalid_runtime_port() {
    if !clang_available() {
        eprintln!("skipping run-command invalid-port test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-invalid-port");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runinvalidport"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.text(200, "ok");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.serve(0, router)
}
"#,
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "run",
        "--path",
        &project_path,
        "--oneshot",
        "--serve-timeout-ms",
        "150",
    ]);
    assert!(
        !output.status.success(),
        "run command should fail when runtime startup uses invalid port"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "runtime should exit with deterministic invalid-port status"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("run failed: binary"),
        "stderr should include non-zero run diagnostics:\n{stderr}"
    );
    assert!(
        stderr.contains("exited with status 1"),
        "stderr should include runtime exit code:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_rejects_zero_max_body_bytes_override() {
    let project_dir = temp_dir("sec4-run-command-zero-max-body-bytes");
    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();

    let output = run_cli(&["run", "--path", &project_path, "--max-body-bytes", "0"]);
    assert!(
        !output.status.success(),
        "run command should fail for zero --max-body-bytes override"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "run command should exit with deterministic invalid-flag status"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("run failed: --max-body-bytes must be >= 1"),
        "stderr should include deterministic max-body-bytes validation message:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_rejects_zero_serve_timeout_ms_override() {
    let project_dir = temp_dir("sec4-run-command-zero-serve-timeout");
    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();

    let output = run_cli(&["run", "--path", &project_path, "--serve-timeout-ms", "0"]);
    assert!(
        !output.status.success(),
        "run command should fail for zero --serve-timeout-ms override"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "run command should exit with deterministic invalid-flag status"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("run failed: --serve-timeout-ms must be >= 1"),
        "stderr should include deterministic serve-timeout-ms validation message:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_rejects_zero_max_concurrency_override() {
    let project_dir = temp_dir("sec4-run-command-zero-max-concurrency");
    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();

    let output = run_cli(&["run", "--path", &project_path, "--max-concurrency", "0"]);
    assert!(
        !output.status.success(),
        "run command should fail for zero --max-concurrency override"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "run command should exit with deterministic invalid-flag status"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("run failed: --max-concurrency must be >= 1"),
        "stderr should include deterministic max-concurrency validation message:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}
