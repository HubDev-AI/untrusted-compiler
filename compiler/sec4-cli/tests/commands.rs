use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
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

fn read_http_response(reader: &mut BufReader<&mut TcpStream>) -> String {
    let mut header_lines = Vec::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        let read = reader
            .read_line(&mut line)
            .expect("response header line should be readable");
        if read == 0 {
            break;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                content_length = value
                    .trim()
                    .parse::<usize>()
                    .expect("content-length should parse as usize");
            }
        }
        header_lines.push(trimmed);
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .expect("response body should be readable");
    }
    let mut response = header_lines.join("\r\n");
    response.push_str("\r\n\r\n");
    response.push_str(String::from_utf8_lossy(&body).as_ref());
    response
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
fn promote_dry_run_emits_deterministic_plan_for_valid_project() {
    let root = temp_dir("sec4-promote-dry-run-deterministic");
    let project_dir = root.join("project");
    write_minimal_project(&project_dir, "");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let first = run_cli(&[
        "promote",
        "--path",
        &project_path,
        "--from",
        "browser",
        "--to",
        "server",
        "--dry-run",
    ]);
    assert!(
        first.status.success(),
        "promote dry-run should succeed for valid project with non-blocking warnings"
    );

    let second = run_cli(&[
        "promote",
        "--path",
        &project_path,
        "--from",
        "browser",
        "--to",
        "server",
        "--dry-run",
    ]);
    assert!(
        second.status.success(),
        "repeated promote dry-run should succeed for unchanged project"
    );

    let first_stdout = String::from_utf8(first.stdout).expect("stdout should be utf-8");
    let second_stdout = String::from_utf8(second.stdout).expect("stdout should be utf-8");
    assert_eq!(
        first_stdout, second_stdout,
        "promote dry-run output should be byte-identical for unchanged tree"
    );

    let parsed: serde_json::Value =
        serde_json::from_str(&first_stdout).expect("promote dry-run should emit valid json");
    assert_eq!(
        parsed.get("from").and_then(serde_json::Value::as_str),
        Some("browser"),
        "promotion plan should include from target"
    );
    assert_eq!(
        parsed.get("to").and_then(serde_json::Value::as_str),
        Some("server"),
        "promotion plan should include to target"
    );
    assert_eq!(
        parsed.get("ready").and_then(serde_json::Value::as_bool),
        Some(true),
        "plan should be ready when only warning-level preconditions exist"
    );
    assert_eq!(
        parsed
            .get("changedBindings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(2),
        "promotion plan should include deterministic changed-binding entries"
    );
    assert!(
        parsed
            .get("generatedFiles")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|files| files
                .iter()
                .any(|entry| { entry.as_str() == Some("server/src/repo/db_repo.ut") })),
        "promotion plan should include deterministic generated scaffold files"
    );
    assert!(
        parsed
            .get("preconditions")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| items.iter().any(|item| {
                item.get("code").and_then(serde_json::Value::as_str) == Some("PROMOTE.P9303")
                    && item.get("severity").and_then(serde_json::Value::as_str) == Some("warning")
            })),
        "promotion plan should include deterministic warning when localdb usage is absent"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn promote_dry_run_writes_plan_artifact_when_out_is_provided() {
    let root = temp_dir("sec4-promote-dry-run-out-artifact");
    let project_dir = root.join("project");
    write_minimal_project(&project_dir, "");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let out_path = root.join("reports/promote-plan.json");
    let out_path_string = out_path
        .to_str()
        .expect("artifact path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "promote",
        "--path",
        &project_path,
        "--from",
        "browser",
        "--to",
        "server",
        "--dry-run",
        "--out",
        &out_path_string,
    ]);
    assert!(output.status.success(), "promote dry-run should succeed");
    assert!(
        out_path.exists(),
        "promote dry-run should write requested artifact path"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stdout_json: serde_json::Value =
        serde_json::from_str(&stdout).expect("promote dry-run stdout should be valid json");
    let file_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&out_path).expect("promote dry-run artifact should be readable"),
    )
    .expect("promote dry-run artifact should be valid json");
    assert_eq!(
        stdout_json, file_json,
        "promote dry-run stdout and --out artifact should be JSON-equivalent"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn promote_apply_rewrites_composition_root_and_generates_scaffold() {
    let root = temp_dir("sec4-promote-apply-rewrite");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src/feature")).expect("src/feature should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"promote-apply\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "use feature.util;\n\nfn main() -> Int {\n  // localdb.main\n  if is_valid() {\n    0\n  } else {\n    1\n  }\n}\n",
    )
    .expect("main should be written");
    fs::write(
        project_dir.join("src/feature/util.ut"),
        "// localdb.module\nfn is_valid() -> Bool {\n  true\n}\n",
    )
    .expect("feature module should be written");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "promote",
        "--path",
        &project_path,
        "--from",
        "browser",
        "--to",
        "server",
    ]);
    assert!(
        output.status.success(),
        "promote apply should succeed for valid browser->server project"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("promote apply should emit valid json report");
    assert!(
        parsed.get("mode").and_then(serde_json::Value::as_str) == Some("apply"),
        "promote apply should emit apply-mode report"
    );
    assert_eq!(
        parsed
            .get("compositionRewrite")
            .and_then(|rewrite| rewrite.get("rewrites"))
            .and_then(serde_json::Value::as_u64),
        Some(1),
        "composition-root rewrite should replace localdb references in src/main.ut only"
    );
    assert!(
        parsed
            .get("generatedFiles")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|files| files
                .iter()
                .any(|entry| { entry.as_str() == Some("server/src/repo/db_repo.ut") })),
        "apply report should list deterministic generated scaffold files"
    );
    assert!(
        parsed
            .get("generatedFiles")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|files| files
                .iter()
                .any(|entry| { entry.as_str() == Some("server/sec4.toml") })),
        "apply report should include generated standalone server manifest"
    );
    assert!(
        parsed
            .get("guardedSkippedReferences")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| items.iter().any(|item| {
                item.get("file").and_then(serde_json::Value::as_str) == Some("src/feature/util.ut")
            })),
        "apply report should include guard-skipped localdb references outside composition root"
    );

    let main_source = fs::read_to_string(project_dir.join("src/main.ut"))
        .expect("rewritten main source should exist");
    assert!(
        main_source.contains("// db.main"),
        "apply rewrite should replace localdb token in composition root:\n{main_source}"
    );
    let module_source = fs::read_to_string(project_dir.join("src/feature/util.ut"))
        .expect("feature module source should remain readable");
    assert!(
        module_source.contains("// localdb.module"),
        "composition-root guard should preserve module files outside src/main.ut:\n{module_source}"
    );

    let report_path = project_dir.join("server/reports/promote-plan.json");
    assert!(
        report_path.exists(),
        "promote apply should write deterministic report artifact"
    );
    let report_source =
        fs::read_to_string(&report_path).expect("promote apply report should be readable");
    let report_json: serde_json::Value = serde_json::from_str(&report_source)
        .expect("promote apply report file should be valid json");
    assert_eq!(
        parsed, report_json,
        "stdout report and written report artifact should be byte-equivalent JSON payloads"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn promote_e2e_apply_generates_server_project_that_checks_and_runs() {
    let root = temp_dir("sec4-promote-e2e-server-scaffold");
    let project_dir = root.join("project");
    write_minimal_project(&project_dir, "");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let dry_run = run_cli(&[
        "promote",
        "--path",
        &project_path,
        "--from",
        "browser",
        "--to",
        "server",
        "--dry-run",
    ]);
    assert!(dry_run.status.success(), "promote dry-run should succeed");

    let apply = run_cli(&[
        "promote",
        "--path",
        &project_path,
        "--from",
        "browser",
        "--to",
        "server",
    ]);
    assert!(apply.status.success(), "promote apply should succeed");

    let server_project_path = project_dir.join("server");
    let server_project_path_string = server_project_path
        .to_str()
        .expect("server project path should be valid utf-8")
        .to_string();

    let check_output = run_cli(&["check", "--path", &server_project_path_string]);
    assert!(
        check_output.status.success(),
        "generated server scaffold should pass sec4 check"
    );

    let run_output = run_cli(&["run", "--path", &server_project_path_string]);
    assert!(
        run_output.status.success(),
        "generated server scaffold should run successfully"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn promote_rejects_unsupported_route_pair() {
    let root = temp_dir("sec4-promote-unsupported-route");
    let project_dir = root.join("project");
    write_minimal_project(&project_dir, "");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "promote",
        "--path",
        &project_path,
        "--from",
        "server",
        "--to",
        "browser",
        "--dry-run",
    ]);
    assert!(
        !output.status.success(),
        "promote should reject unsupported route pair deterministically"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "unsupported route should produce deterministic usage-style exit code"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("only `browser -> server` is available"),
        "unsupported route error should include deterministic guidance:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn promote_dry_run_reports_blocking_preconditions_for_invalid_project() {
    let root = temp_dir("sec4-promote-invalid-project");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"promote-invalid\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main( -> Int {\n  0\n}\n",
    )
    .expect("invalid source should be written");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "promote",
        "--path",
        &project_path,
        "--from",
        "browser",
        "--to",
        "server",
        "--dry-run",
    ]);
    assert!(
        !output.status.success(),
        "promote dry-run should fail when blocking preconditions exist"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "blocking preconditions should produce deterministic failure exit code"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("promote dry-run should emit json even on failure");
    assert_eq!(
        parsed.get("ready").and_then(serde_json::Value::as_bool),
        Some(false),
        "plan readiness should be false when blocking preconditions exist"
    );
    assert!(
        parsed
            .get("preconditions")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| items.iter().any(|item| {
                item.get("severity").and_then(serde_json::Value::as_str) == Some("error")
                    && item
                        .get("code")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|code| code.starts_with("DIAG."))
            })),
        "blocking promote plan should include diagnostic-derived error preconditions"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn promote_dry_run_treats_semantic_diagnostics_as_non_blocking_warnings() {
    let root = temp_dir("sec4-promote-semantic-warning-non-blocking");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"promote-semantic-warning\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), "").expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  missing();\n  0\n}\n",
    )
    .expect("semantic-warning source should be written");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "promote",
        "--path",
        &project_path,
        "--from",
        "browser",
        "--to",
        "server",
        "--dry-run",
    ]);
    assert!(
        output.status.success(),
        "promote dry-run should stay non-blocking for semantic diagnostics"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("promote dry-run should emit valid json");
    assert_eq!(
        parsed.get("ready").and_then(serde_json::Value::as_bool),
        Some(true),
        "semantic diagnostics should not block promotion planning"
    );
    assert!(
        parsed
            .get("preconditions")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|items| items.iter().any(|item| {
                item.get("code")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|code| code.starts_with("DIAG.") && code != "PROMOTE.P9303")
                    && item.get("severity").and_then(serde_json::Value::as_str) == Some("warning")
                    && item.get("message").and_then(serde_json::Value::as_str)
                        == Some("unknown function or constructor")
            })),
        "semantic diagnostics should be reported as warning preconditions"
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
fn lasm_smoke_command_supports_bound_route_and_handler_aliases() {
    let root = temp_dir("sec4-lasm-smoke-bound-route-handler");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should exist");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-bound-route-handler\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn health() effects { net } -> Int {\n  res.text(200, \"alias body\");\n  0\n}\n\nfn main() effects { net } -> Int {\n  let router = http.router();\n  let route_path = \"/probe\";\n  let route_handler = health;\n  http.get(router, route_path, route_handler);\n  0\n}\n",
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&[
        "lasm-smoke",
        "--path",
        project_path,
        "--method",
        "GET",
        "--route",
        "/probe",
        "--requests",
        "1",
    ]);

    assert!(output.status.success(), "lasm-smoke command should succeed");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("origin=handler:health"),
        "lasm-smoke output should resolve aliased handler origin:\n{stdout}"
    );
    assert!(
        stdout.contains("body=alias body"),
        "lasm-smoke output should include aliased handler response body:\n{stdout}"
    );
}

#[test]
fn lasm_smoke_command_revisits_helper_route_registration_with_distinct_bindings() {
    let root = temp_dir("sec4-lasm-smoke-helper-revisit-bindings");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should exist");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-helper-revisit-bindings\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn health() effects { net } -> Int {\n  res.text(200, \"helper route\");\n  0\n}\n\nfn register_route(router: Router, path: String) effects { net } -> Int {\n  http.get(router, path, health);\n  0\n}\n\nfn main() effects { net } -> Int {\n  let router = http.router();\n  register_route(router, \"/first\");\n  register_route(router, \"/second\");\n  0\n}\n",
    )
    .expect("source should be written");

    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&[
        "lasm-smoke",
        "--path",
        project_path,
        "--method",
        "GET",
        "--route",
        "/second",
        "--requests",
        "1",
        "--max-steps",
        "64",
    ]);

    assert!(
        output.status.success(),
        "lasm-smoke command should succeed for repeated helper registration bindings"
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("origin=handler:health"),
        "lasm-smoke output should resolve helper handler origin after revisiting helper:\n{stdout}"
    );
    assert!(
        stdout.contains("body=helper route"),
        "lasm-smoke output should include helper route body from second registration:\n{stdout}"
    );
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
        stdout.contains("body={\"ok\":true,\"status\":201,\"schema\":\"CreateUserResponse\"}"),
        "lasm-smoke output should include deterministic res.ok envelope body:\n{stdout}"
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
    assert_eq!(
        parsed
            .get("headers")
            .and_then(|headers| headers.get("Content-Type"))
            .and_then(serde_json::Value::as_str),
        Some("application/json; charset=utf-8"),
        "lasm-smoke json should include default JSON content-type for res.ok handlers"
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
fn lasm_smoke_command_rejects_zero_max_pending() {
    let root = temp_dir("sec4-lasm-smoke-zero-max-pending");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-zero-max-pending\"\nversion = \"0.1.0\"\n",
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
    let output = run_cli(&["lasm-smoke", "--path", &project_path, "--max-pending", "0"]);
    assert!(
        !output.status.success(),
        "lasm-smoke should fail for zero max pending limit"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "invalid max pending should return deterministic usage-style exit code"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("invalid max pending limit: must be >= 1"),
        "lasm-smoke failure should mention deterministic max pending guidance:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_rejects_zero_max_request_ms() {
    let root = temp_dir("sec4-lasm-smoke-zero-max-request-ms");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-zero-max-request-ms\"\nversion = \"0.1.0\"\n",
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
        "--max-request-ms",
        "0",
    ]);
    assert!(
        !output.status.success(),
        "lasm-smoke should fail for zero max request duration"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "invalid max request duration should return deterministic usage-style exit code"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("invalid max request duration: must be >= 1ms"),
        "lasm-smoke failure should mention deterministic max request duration guidance:\n{stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_rejects_invalid_runtime_script_segments() {
    let root = temp_dir("sec4-lasm-smoke-invalid-runtime-script");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-invalid-runtime-script\"\nversion = \"0.1.0\"\n",
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
        "--runtime-script",
        "yield,unknown:1",
    ]);
    assert!(
        !output.status.success(),
        "lasm-smoke should fail for invalid runtime-script segments"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "invalid runtime-script should return deterministic usage-style exit code"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("invalid --runtime-script segment `unknown:1`"),
        "lasm-smoke failure should identify invalid runtime-script segment:\n{stderr}"
    );

    let empty_script_output = run_cli(&[
        "lasm-smoke",
        "--path",
        &project_path,
        "--runtime-script",
        " ,  , ",
    ]);
    assert!(
        !empty_script_output.status.success(),
        "lasm-smoke should fail when runtime-script yields no actions"
    );
    assert_eq!(
        empty_script_output.status.code(),
        Some(2),
        "empty runtime-script should return deterministic usage-style exit code"
    );
    let empty_stderr =
        String::from_utf8(empty_script_output.stderr).expect("stderr should be utf-8");
    assert!(
        empty_stderr.contains("invalid --runtime-script: expected at least one action segment"),
        "lasm-smoke failure should mention empty runtime-script guidance:\n{empty_stderr}"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_reports_queue_overflow_with_max_pending_limit() {
    let root = temp_dir("sec4-lasm-smoke-max-pending-overflow");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-max-pending-overflow\"\nversion = \"0.1.0\"\n",
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
        "--max-pending",
        "1",
        "--requests",
        "3",
        "--max-steps",
        "64",
        "--format",
        "json",
    ]);
    assert!(
        output.status.success(),
        "lasm-smoke command should still succeed with overflow responses counted"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("lasm-smoke json output should be valid json");
    assert_eq!(
        parsed
            .get("maxInFlight")
            .and_then(serde_json::Value::as_u64),
        Some(1),
        "json summary should include effective max in-flight limit"
    );
    assert_eq!(
        parsed.get("maxPending").and_then(serde_json::Value::as_u64),
        Some(1),
        "json summary should include effective max pending limit"
    );
    assert_eq!(
        parsed.get("okCount").and_then(serde_json::Value::as_u64),
        Some(2),
        "overflow scenario should keep two accepted responses successful"
    );
    assert_eq!(
        parsed.get("errorCount").and_then(serde_json::Value::as_u64),
        Some(1),
        "overflow scenario should include one deterministic error response"
    );
    assert_eq!(
        parsed
            .get("statusCounts")
            .and_then(|counts| counts.get("200"))
            .and_then(serde_json::Value::as_u64),
        Some(2),
        "statusCounts should include successful response count"
    );
    assert_eq!(
        parsed
            .get("statusCounts")
            .and_then(|counts| counts.get("503"))
            .and_then(serde_json::Value::as_u64),
        Some(1),
        "statusCounts should include overflow response count"
    );
    assert_eq!(
        parsed.get("status").and_then(serde_json::Value::as_u64),
        Some(503),
        "first emitted response should be deterministic queue-full overflow"
    );
    assert_eq!(
        parsed.get("body").and_then(serde_json::Value::as_str),
        Some("runtime queue full"),
        "overflow response body should be deterministic"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_reports_timeout_status_with_max_request_ms() {
    let root = temp_dir("sec4-lasm-smoke-max-request-timeout");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-max-request-timeout\"\nversion = \"0.1.0\"\n",
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
        "--runtime-script",
        "sleep:25,complete:0",
        "--max-request-ms",
        "10",
        "--requests",
        "1",
        "--max-steps",
        "64",
        "--format",
        "json",
    ]);
    assert!(
        output.status.success(),
        "lasm-smoke should succeed and report timeout response when fail-on-errors is disabled"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("lasm-smoke json output should be valid json");
    assert_eq!(
        parsed
            .get("maxRequestMs")
            .and_then(serde_json::Value::as_u64),
        Some(10),
        "json summary should include effective max request duration"
    );
    assert_eq!(
        parsed.get("okCount").and_then(serde_json::Value::as_u64),
        Some(0),
        "timeout scenario should emit no successful responses"
    );
    assert_eq!(
        parsed.get("errorCount").and_then(serde_json::Value::as_u64),
        Some(1),
        "timeout scenario should emit one deterministic error response"
    );
    assert_eq!(
        parsed
            .get("statusCounts")
            .and_then(|counts| counts.get("504"))
            .and_then(serde_json::Value::as_u64),
        Some(1),
        "statusCounts should include timeout status count"
    );
    assert_eq!(
        parsed.get("status").and_then(serde_json::Value::as_u64),
        Some(504),
        "first emitted response should be deterministic timeout status"
    );
    assert_eq!(
        parsed.get("body").and_then(serde_json::Value::as_str),
        Some("handler timed out after 10ms"),
        "timeout response body should include deterministic limit details"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_times_out_pending_requests_from_submit_age() {
    let root = temp_dir("sec4-lasm-smoke-timeout-from-submit-age");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-timeout-from-submit-age\"\nversion = \"0.1.0\"\n",
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
        "--runtime-script",
        "sleep:20,complete:0",
        "--max-request-ms",
        "10",
        "--max-in-flight",
        "1",
        "--requests",
        "2",
        "--max-steps",
        "64",
        "--format",
        "json",
    ]);
    assert!(
        output.status.success(),
        "lasm-smoke should report deterministic timeout results for in-flight + pending requests"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("lasm-smoke json output should be valid json");
    assert_eq!(
        parsed.get("errorCount").and_then(serde_json::Value::as_u64),
        Some(2),
        "both requests should exceed timeout budget in deterministic timeout scenario"
    );
    assert_eq!(
        parsed
            .get("statusCounts")
            .and_then(|counts| counts.get("504"))
            .and_then(serde_json::Value::as_u64),
        Some(2),
        "both responses should emit timeout status"
    );
    assert_eq!(
        parsed.get("nowMs").and_then(serde_json::Value::as_u64),
        Some(20),
        "submit-age timeout should avoid running an extra second task window after first timeout"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_reports_duration_stats_in_json_summary() {
    let root = temp_dir("sec4-lasm-smoke-duration-stats");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-duration-stats\"\nversion = \"0.1.0\"\n",
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
        "--runtime-script",
        "sleep:7,complete:0",
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
            .get("durationMs")
            .and_then(|duration| duration.get("min"))
            .and_then(serde_json::Value::as_u64),
        Some(7),
        "duration min should be deterministic for scripted sleep"
    );
    assert_eq!(
        parsed
            .get("durationMs")
            .and_then(|duration| duration.get("max"))
            .and_then(serde_json::Value::as_u64),
        Some(7),
        "duration max should be deterministic for scripted sleep"
    );
    assert_eq!(
        parsed
            .get("durationMs")
            .and_then(|duration| duration.get("avg"))
            .and_then(serde_json::Value::as_u64),
        Some(7),
        "duration avg should be deterministic for scripted sleep"
    );
    assert_eq!(
        parsed
            .get("firstDurationMs")
            .and_then(serde_json::Value::as_u64),
        Some(7),
        "firstDurationMs should align with deterministic scripted duration"
    );

    fs::remove_dir_all(&root).expect("temp project cleanup should succeed");
}

#[test]
fn lasm_smoke_command_fail_on_errors_turns_overflow_into_failure() {
    let root = temp_dir("sec4-lasm-smoke-fail-on-errors");
    let project_dir = root.join("project");
    fs::create_dir_all(project_dir.join("src")).expect("src should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"lasm-smoke-fail-on-errors\"\nversion = \"0.1.0\"\n",
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
        "--max-pending",
        "1",
        "--requests",
        "3",
        "--max-steps",
        "64",
        "--fail-on-errors",
    ]);
    assert!(
        !output.status.success(),
        "lasm-smoke should fail when --fail-on-errors is enabled and overflow emits 503"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "fail-on-errors should produce deterministic runtime failure code"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("error responses observed with --fail-on-errors")
            && stderr.contains("statusCounts=200:2,503:1"),
        "fail-on-errors should report deterministic error summary with status distribution:\n{stderr}"
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
        response.contains("Access-Control-Allow-Origin: "),
        "response should include CORS allow-origin header:\n{response}"
    );
    assert!(
        response.contains(
            "Content-Security-Policy: default-src 'self'; frame-ancestors 'none'; base-uri 'self'"
        ),
        "response should include security policy header:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npong"),
        "response should include expected body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_supports_dynamic_req_header_in_res_text() {
    if !clang_available() {
        eprintln!("skipping run-command dynamic req.header text test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-dynamic-req-header-text");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "rundynamicreqheadertextcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let request_id = validate.nonEmpty(req.header("X-Request-Id"));
  res.text(200, request_id);
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
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nX-Request-Id: req-1\r\nx-request-id: req-2\r\nConnection: close\r\n\r\n",
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
            panic!("run command dynamic req.header text test could not connect to server");
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
            panic!("run command dynamic req.header text process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command dynamic req.header text process should exit successfully"
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
        response.contains("\r\n\r\nreq-1, req-2"),
        "response should materialize merged req.header value in response body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_supports_dynamic_err_internal_message() {
    if !clang_available() {
        eprintln!("skipping run-command dynamic err.internal test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-dynamic-err-internal");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "rundynamicerrinternalcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let message = validate.nonEmpty(req.query("msg"));
  err.internal(message);
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
                        b"GET /health?msg=boom HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command dynamic err.internal test could not connect to server");
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
            panic!("run command dynamic err.internal process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command dynamic err.internal process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 500 Internal Server Error"),
        "response should contain deterministic internal-error status:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"INTERNAL.ERROR\""),
        "response should include deterministic internal error code:\n{response}"
    );
    assert!(
        response.contains("\"message\":\"boom\""),
        "response should materialize dynamic err.internal message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_supports_dynamic_err_with_path() {
    if !clang_available() {
        eprintln!("skipping run-command dynamic err.withPath test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-dynamic-err-with-path");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "rundynamicerrwithpathcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let message = validate.nonEmpty(req.query("msg"));
  let path = validate.nonEmpty(req.query("path"));
  let base = err.internal(message);
  err.withPath(base, path);
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
                        b"GET /health?msg=boom&path=%24.user.name HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command dynamic err.withPath test could not connect to server");
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
            panic!("run command dynamic err.withPath process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command dynamic err.withPath process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 500 Internal Server Error"),
        "response should contain deterministic internal-error status:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"INTERNAL.ERROR\""),
        "response should include deterministic internal error code:\n{response}"
    );
    assert!(
        response.contains("\"message\":\"boom\""),
        "response should materialize dynamic err.internal message:\n{response}"
    );
    assert!(
        response.contains("\"path\":\"$.user.name\""),
        "response should materialize dynamic err.withPath path value:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_supports_ctx_current_with_auth_require_role() {
    if !clang_available() {
        eprintln!("skipping run-command ctx.current auth.requireRole test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-ctx-current-auth-require-role");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runctxcurrentauthrequirerolecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  let ctx = ctx.current();
  let role = validate.nonEmpty(req.query("role"));
  auth.requireRole(ctx, role);
  res.text(200, "authorized");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/secure", secure);
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
                        b"GET /secure?role=admin HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer admin\r\nConnection: close\r\n\r\n",
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
            panic!("run command ctx.current auth.requireRole test could not connect to server");
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
                "run command ctx.current auth.requireRole process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command ctx.current auth.requireRole process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nauthorized"),
        "response should include expected body:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_serves_request_and_exits() {
    let project_dir = temp_dir("sec4-run-command-lasm-oneshot");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmoneshotcommand"
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
            "--backend",
            "lasm",
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
            panic!("run command LASM oneshot test could not connect to server");
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
            panic!("run command LASM oneshot process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM oneshot process should exit successfully"
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
fn run_command_oneshot_lasm_backend_resolves_alias_route_registration_helper_call() {
    let project_dir = temp_dir("sec4-run-command-lasm-alias-route-helper");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmaliasroutehelpercommand"
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

fn register_routes(router: Router) effects { net } -> Int {
  http.get(router, "/health", health);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let register = register_routes;
  register(router);
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
            "--backend",
            "lasm",
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
            panic!("run command LASM alias route helper test could not connect to server");
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
            panic!("run command LASM alias route helper process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM alias route helper process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain expected status line:\n{response}"
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
fn run_command_oneshot_lasm_backend_enforces_auth_require_role_with_ctx_current() {
    let project_dir = temp_dir("sec4-run-command-lasm-ctx-current-auth-require-role");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmctxcurrentauthcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  let ctx = ctx.current();
  let role = validate.nonEmpty(req.query("role"));
  auth.requireRole(ctx, role);
  res.text(200, "secure");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/secure", secure);
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

    let run_once = |request: &'static [u8]| -> String {
        let port = find_available_tcp_port();
        let port_value = port.to_string();
        let mut child = Command::new(cli_bin())
            .args([
                "run",
                "--path",
                path.as_str(),
                "--backend",
                "lasm",
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
                        .write_all(request)
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
                panic!("run command LASM auth.requireRole test could not connect to server");
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
                panic!("run command LASM auth.requireRole process did not exit in expected window");
            }
        };

        assert!(
            status.success(),
            "run command LASM auth.requireRole process should exit successfully"
        );
        response
    };

    let forbidden = run_once(
        b"GET /secure?role=admin HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer user\r\nConnection: close\r\n\r\n",
    );
    assert!(
        forbidden.contains("HTTP/1.1 403 Forbidden"),
        "response should contain 403 status line when bearer role is missing:\n{forbidden}"
    );
    assert!(
        forbidden.contains("\"code\":\"AUTH.FORBIDDEN\"")
            && forbidden.contains("\"message\":\"Authorization token missing required role\""),
        "response should include deterministic forbidden auth envelope:\n{forbidden}"
    );

    let allowed = run_once(
        b"GET /secure?role=admin HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer admin\r\nConnection: close\r\n\r\n",
    );
    assert!(
        allowed.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line when bearer token has required role:\n{allowed}"
    );
    assert!(
        allowed.contains("\r\n\r\nsecure"),
        "response should include expected secure body:\n{allowed}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_enforces_auth_require_role_from_helper_param() {
    let project_dir = temp_dir("sec4-run-command-lasm-helper-auth-require-role");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmhelperauthrequirerolecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn enforce(role: String) effects { net } -> Int {
  let ctx = ctx.current();
  auth.requireRole(ctx, role);
  0
}

fn secure() effects { net } -> Int {
  enforce("admin");
  res.text(200, "secure");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/secure", secure);
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

    let run_once = |request: &'static [u8]| -> String {
        let port = find_available_tcp_port();
        let port_value = port.to_string();
        let mut child = Command::new(cli_bin())
            .args([
                "run",
                "--path",
                path.as_str(),
                "--backend",
                "lasm",
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
                        .write_all(request)
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
                panic!("run command LASM helper auth.requireRole test could not connect to server");
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
                    "run command LASM helper auth.requireRole process did not exit in expected window"
                );
            }
        };

        assert!(
            status.success(),
            "run command LASM helper auth.requireRole process should exit successfully"
        );
        response
    };

    let forbidden = run_once(
        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer user\r\nConnection: close\r\n\r\n",
    );
    assert!(
        forbidden.contains("HTTP/1.1 403 Forbidden"),
        "response should contain 403 status line when bearer role is missing:\n{forbidden}"
    );
    assert!(
        forbidden.contains("\"code\":\"AUTH.FORBIDDEN\"")
            && forbidden.contains("\"message\":\"Authorization token missing required role\""),
        "response should include deterministic forbidden auth envelope:\n{forbidden}"
    );

    let allowed = run_once(
        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer user:admin\r\nConnection: close\r\n\r\n",
    );
    assert!(
        allowed.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line when bearer token has required role:\n{allowed}"
    );
    assert!(
        allowed.contains("\r\n\r\nsecure"),
        "response should include expected secure body:\n{allowed}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_enforces_auth_require_role_from_alias_callee() {
    let project_dir = temp_dir("sec4-run-command-lasm-alias-auth-require-role");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmaliasauthrequirerolecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  let ctx = ctx.current();
  let requireRole = auth.requireRole;
  requireRole(ctx, "admin");
  res.text(200, "secure");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/secure", secure);
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

    let run_once = |request: &'static [u8]| -> String {
        let port = find_available_tcp_port();
        let port_value = port.to_string();
        let mut child = Command::new(cli_bin())
            .args([
                "run",
                "--path",
                path.as_str(),
                "--backend",
                "lasm",
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
                        .write_all(request)
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
                panic!("run command LASM alias auth.requireRole test could not connect to server");
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
                    "run command LASM alias auth.requireRole process did not exit in expected window"
                );
            }
        };

        assert!(
            status.success(),
            "run command LASM alias auth.requireRole process should exit successfully"
        );
        response
    };

    let forbidden = run_once(
        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer user\r\nConnection: close\r\n\r\n",
    );
    assert!(
        forbidden.contains("HTTP/1.1 403 Forbidden"),
        "response should contain 403 status line when bearer role is missing:\n{forbidden}"
    );
    assert!(
        forbidden.contains("\"code\":\"AUTH.FORBIDDEN\"")
            && forbidden.contains("\"message\":\"Authorization token missing required role\""),
        "response should include deterministic forbidden auth envelope:\n{forbidden}"
    );

    let allowed = run_once(
        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer user:admin\r\nConnection: close\r\n\r\n",
    );
    assert!(
        allowed.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line when bearer token has required role:\n{allowed}"
    );
    assert!(
        allowed.contains("\r\n\r\nsecure"),
        "response should include expected secure body:\n{allowed}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_resolves_alias_response_helper_call() {
    let project_dir = temp_dir("sec4-run-command-lasm-alias-response-helper");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmaliasresponsehelpercommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn emit(status: Int, body: String) effects { net } -> Int {
  res.text(status, body);
  0
}

fn secure() effects { net } -> Int {
  let send = emit;
  send(201, "alias-body");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", secure);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let output = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            project_dir
                .to_str()
                .expect("project path should be valid utf-8"),
            "--backend",
            "lasm",
            "--oneshot",
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");
    let mut child = output;

    let mut response = None;
    for _ in 0..600 {
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
            panic!("run command LASM alias response helper test could not connect to server");
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
                "run command LASM alias response helper process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM alias response helper process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 201 Created"),
        "response should contain aliased helper status:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nalias-body"),
        "response should contain aliased helper body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_materializes_alias_req_query_in_res_text() {
    let project_dir = temp_dir("sec4-run-command-lasm-alias-req-query-res-text");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmaliasreqqueryrestextcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  let query = req.query;
  let nonEmpty = validate.nonEmpty;
  let name = nonEmpty(query("name"));
  res.text(200, name);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/secure", secure);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let output = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            project_dir
                .to_str()
                .expect("project path should be valid utf-8"),
            "--backend",
            "lasm",
            "--oneshot",
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");
    let mut child = output;

    let mut response = None;
    for _ in 0..600 {
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
                        b"GET /secure?name=alice HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM alias req.query res.text test could not connect to server");
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
                "run command LASM alias req.query res.text process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM alias req.query res.text process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain expected status line:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nalice"),
        "response should contain materialized aliased req.query body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_resolves_alias_response_set_header_call() {
    let project_dir = temp_dir("sec4-run-command-lasm-alias-response-set-header");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmaliasresponsesetheadercommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  let set = res.setHeader;
  set(headers.name("X-Alias"), headers.value("ok"));
  res.text(200, "secure");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/secure", secure);
  http.serve(8080, router);
  0
}
"#,
    )
    .expect("source should be written");

    let output = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            project_dir
                .to_str()
                .expect("project path should be valid utf-8"),
            "--backend",
            "lasm",
            "--oneshot",
            "--port",
            &port.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");
    let mut child = output;

    let mut response = None;
    for _ in 0..600 {
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
                        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM alias response setHeader test could not connect to server");
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
                "run command LASM alias response setHeader process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM alias response setHeader process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain expected status line:\n{response}"
    );
    assert!(
        response.contains("X-Alias: ok"),
        "response should contain aliased setHeader materialization:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nsecure"),
        "response should contain expected body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_enforces_auth_with_auth_middleware() {
    let project_dir = temp_dir("sec4-run-command-lasm-auth-middleware");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmauthmiddlewarecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  res.text(200, "secure");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let authCfg = auth.fromPolicy();
  let router = auth.withAuth(router, authCfg);
  http.get(router, "/secure", secure);
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
            "--backend",
            "lasm",
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
                        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM auth middleware test could not connect to server");
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
            panic!("run command LASM auth middleware process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM auth middleware process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line when auth middleware rejects request:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\"")
            && response.contains("\"message\":\"Authorization header missing or invalid\""),
        "response should include deterministic auth middleware error envelope:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_enforces_auth_middleware_from_alias_helper_callee() {
    let project_dir = temp_dir("sec4-run-command-lasm-auth-middleware-alias-helper");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmauthmiddlewarealiashelpercommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn apply_auth(router: Router, cfg: AuthConfig) effects { net } -> Router {
  auth.withAuth(router, cfg)
}

fn secure() effects { net } -> Int {
  res.text(200, "secure");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let authCfg = auth.fromPolicy();
  let wrap = apply_auth;
  let router = wrap(router, authCfg);
  http.get(router, "/secure", secure);
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
            "--backend",
            "lasm",
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
                        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!(
                "run command LASM auth middleware alias-helper test could not connect to server"
            );
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
                "run command LASM auth middleware alias-helper process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM auth middleware alias-helper process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line when helper-wrapped auth middleware rejects request:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\"")
            && response.contains("\"message\":\"Authorization header missing or invalid\""),
        "response should include deterministic auth middleware error envelope:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_skips_auth_middleware_when_auth_mode_env_off() {
    let project_dir = temp_dir("sec4-run-command-lasm-auth-middleware-env-off");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmauthmiddlewarepolicyoffcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  res.text(200, "secure");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let authCfg = auth.fromPolicy();
  let router = auth.withAuth(router, authCfg);
  http.get(router, "/secure", secure);
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
            "--backend",
            "lasm",
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .env("SEC4_RT_AUTH_MODE", "off")
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
                        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM auth middleware env-off test could not connect to server");
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
                "run command LASM auth middleware env-off process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM auth middleware env-off process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line when auth mode is off:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nsecure"),
        "response should include expected secure body when auth mode is off:\n{response}"
    );
    assert!(
        !response.contains("\"code\":\"AUTH.UNAUTHORIZED\""),
        "response should not include unauthorized envelope when auth mode is off:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_enforces_csrf_with_csrf_middleware() {
    let project_dir = temp_dir("sec4-run-command-lasm-csrf-middleware");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmcsrfmiddlewarecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn submit() effects { net } -> Int {
  res.text(200, "submitted");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let csrfCfg = csrf.fromPolicy();
  let router = csrf.withCsrf(router, csrfCfg);
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
            "--backend",
            "lasm",
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
            panic!("run command LASM csrf middleware test could not connect to server");
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
            panic!("run command LASM csrf middleware process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM csrf middleware process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 403 Forbidden"),
        "response should contain 403 status line when csrf middleware rejects request:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"AUTH.CSRF_TOKEN_INVALID\"")
            && response.contains("\"message\":\"CSRF token missing or invalid\""),
        "response should include deterministic csrf middleware error envelope:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_skips_csrf_middleware_when_csrf_mode_env_off() {
    let project_dir = temp_dir("sec4-run-command-lasm-csrf-middleware-env-off");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmcsrfmiddlewareenvoffcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn submit() effects { net } -> Int {
  res.text(200, "submitted");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let csrfCfg = csrf.fromPolicy();
  let router = csrf.withCsrf(router, csrfCfg);
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
            "--backend",
            "lasm",
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "20000",
        ])
        .env("SEC4_RT_CSRF_MODE", "off")
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
            panic!("run command LASM csrf middleware env-off test could not connect to server");
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
                "run command LASM csrf middleware env-off process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM csrf middleware env-off process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line when csrf mode is off:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nsubmitted"),
        "response should include expected submitted body when csrf mode is off:\n{response}"
    );
    assert!(
        !response.contains("\"code\":\"AUTH.CSRF_TOKEN_INVALID\""),
        "response should not include csrf invalid envelope when csrf mode is off:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_does_not_apply_unbound_middleware_to_route() {
    let project_dir = temp_dir("sec4-run-command-lasm-unbound-middleware");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmunboundmiddlewarecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn public() effects { net } -> Int {
  res.text(200, "public");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let authOnly = auth.withAuth(http.router(), auth.fromPolicy());
  let csrfOnly = csrf.withCsrf(http.router(), csrf.fromPolicy());
  http.get(router, "/public", public);
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
            "--backend",
            "lasm",
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
                        b"GET /public HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM unbound middleware test could not connect to server");
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
            panic!("run command LASM unbound middleware process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM unbound middleware process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line for unbound middleware route:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npublic"),
        "response should include expected public body:\n{response}"
    );
    assert!(
        !response.contains("\"code\":\"AUTH.UNAUTHORIZED\"")
            && !response.contains("\"code\":\"AUTH.CSRF_TOKEN_INVALID\""),
        "response should not include auth/csrf middleware failures for unrelated route:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_applies_auth_middleware_from_helper_call_side_effect() {
    let project_dir = temp_dir("sec4-run-command-lasm-helper-auth-middleware");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmhelperauthmiddlewarecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  res.text(200, "secure");
  0
}

fn attachAuth(router: Router) effects { net } -> Int {
  auth.withAuth(router, auth.fromPolicy());
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  attachAuth(router);
  http.get(router, "/secure", secure);
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
            "--backend",
            "lasm",
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
                        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM helper auth middleware test could not connect to server");
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
                "run command LASM helper auth middleware process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM helper auth middleware process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line when helper middleware enables auth:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\""),
        "response should include deterministic unauthorized envelope when helper middleware applies:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_applies_helper_middleware_through_route_register_function() {
    let project_dir = temp_dir("sec4-run-command-lasm-helper-middleware-route-register");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmhelpermiddlewarerouteregistercommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  res.text(200, "secure");
  0
}

fn attachAuth(router: Router) effects { net } -> Int {
  auth.withAuth(router, auth.fromPolicy());
  0
}

fn registerSecure(router: Router) effects { net } -> Int {
  http.get(router, "/secure", secure);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  attachAuth(router);
  registerSecure(router);
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
            "--backend",
            "lasm",
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
                        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM helper middleware route-register test could not connect to server");
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
                "run command LASM helper middleware route-register process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM helper middleware route-register process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line when helper middleware propagates into route helper registration:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\""),
        "response should include deterministic unauthorized envelope when helper middleware propagates into route helper registration:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_applies_helper_middleware_when_router_arg_is_not_first() {
    let project_dir = temp_dir("sec4-run-command-lasm-helper-middleware-router-second-arg");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmhelpermiddlewareroutersecondargcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  res.text(200, "secure");
  0
}

fn wrapRoute(mode: Int, router: Router) effects { net } -> Router {
  router
}

fn registerSecure(router: Router) effects { net } -> Int {
  http.get(wrapRoute(1, router), "/secure", secure);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  let secured = auth.withAuth(router, auth.fromPolicy());
  registerSecure(secured);
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
            "--backend",
            "lasm",
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
                        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!(
                "run command LASM helper middleware second-arg test could not connect to server"
            );
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
                "run command LASM helper middleware second-arg process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM helper middleware second-arg process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line when helper wrapper routes preserve auth middleware:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\""),
        "response should include deterministic unauthorized envelope when helper wrapper routes preserve auth middleware:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_applies_auth_middleware_wrapped_in_serve_call() {
    let project_dir = temp_dir("sec4-run-command-lasm-auth-middleware-serve-wrapper");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmauthmiddlewareservewrappercommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  res.text(200, "secure");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/secure", secure);
  let authCfg = auth.fromPolicy();
  http.serve(8080, auth.withAuth(router, authCfg));
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
            "--backend",
            "lasm",
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
                        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!(
                "run command LASM auth middleware serve-wrapper test could not connect to server"
            );
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
                "run command LASM auth middleware serve-wrapper process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM auth middleware serve-wrapper process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line when serve wrapper enables auth middleware:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\""),
        "response should include deterministic unauthorized envelope when serve wrapper enables auth middleware:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_applies_helper_side_effect_middleware_when_router_arg_is_not_first(
) {
    let project_dir = temp_dir("sec4-run-command-lasm-helper-side-effect-router-second-arg");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmhelpersideeffectroutersecondargcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn secure() effects { net } -> Int {
  res.text(200, "secure");
  0
}

fn applyAuth(mode: Int, router: Router) effects { net } -> Int {
  auth.withAuth(router, auth.fromPolicy());
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/secure", secure);
  applyAuth(1, router);
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
            "--backend",
            "lasm",
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
                        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!(
                "run command LASM helper side-effect middleware second-arg test could not connect to server"
            );
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
                "run command LASM helper side-effect middleware second-arg process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM helper side-effect middleware second-arg process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line when helper side-effect middleware uses non-first router arg:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\""),
        "response should include deterministic unauthorized envelope when helper side-effect middleware uses non-first router arg:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_uses_latest_duplicate_route_registration() {
    let project_dir = temp_dir("sec4-run-command-lasm-latest-duplicate-route");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmlatestduplicateroutecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn first() effects { net } -> Int {
  res.text(200, "first");
  0
}

fn second() effects { net } -> Int {
  res.text(200, "second");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/dup", first);
  http.get(router, "/dup", second);
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
            "--backend",
            "lasm",
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
                    .write_all(b"GET /dup HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
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
            panic!("run command LASM duplicate-route test could not connect to server");
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
            panic!("run command LASM duplicate-route process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM duplicate-route process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nsecond"),
        "response body should come from latest duplicate route registration:\n{response}"
    );
    assert!(
        !response.contains("\r\n\r\nfirst"),
        "response should not use the first duplicate route handler:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_uses_latest_response_write_in_handler() {
    let project_dir = temp_dir("sec4-run-command-lasm-latest-response-write");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmlatestresponsewritecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn flip() effects { net } -> Int {
  res.text(200, "first");
  res.text(201, "second");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/flip", flip);
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
            "--backend",
            "lasm",
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
                        b"GET /flip HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM latest-response-write test could not connect to server");
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
                "run command LASM latest-response-write process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM latest-response-write process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 201 Created"),
        "response should contain status from latest response write:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nsecond"),
        "response body should come from latest response write:\n{response}"
    );
    assert!(
        !response.contains("\r\n\r\nfirst"),
        "response should not include first response write body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_resolves_param_bound_response_helper_calls() {
    let project_dir = temp_dir("sec4-run-command-lasm-param-bound-response-helper");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmparamboundresponsehelpercommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn emit(status: Int, body: String) effects { net } -> Int {
  res.text(status, body);
  0
}

fn handle() effects { net } -> Int {
  emit(201, "from-helper");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/helper", handle);
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
            "--backend",
            "lasm",
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
                        b"GET /helper HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM param-bound response helper test could not connect to server");
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
                "run command LASM param-bound response helper process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM param-bound response helper process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 201 Created"),
        "response should contain helper-bound status code:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nfrom-helper"),
        "response body should contain helper-bound literal:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_resolves_param_bound_response_header_helper_calls() {
    let project_dir = temp_dir("sec4-run-command-lasm-param-bound-response-header-helper");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmparamboundresponseheaderhelpercommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn emitHeader(name: String, value: String) effects { net } -> Int {
  res.setHeader(headers.name(name), headers.value(value));
  0
}

fn handle() effects { net } -> Int {
  emitHeader("X-From-Helper", "active");
  res.text(200, "header-bound");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/header", handle);
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
            "--backend",
            "lasm",
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
                        b"GET /header HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM param-bound response header helper test could not connect");
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
                "run command LASM param-bound response header helper process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM param-bound response header helper process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("X-From-Helper: active"),
        "response should contain header materialized through helper parameters:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nheader-bound"),
        "response should contain expected body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_missing_http_version() {
    let project_dir = temp_dir("sec4-run-command-lasm-missing-http-version");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmmissinghttpversioncommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before missing-version request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(b"GET /health\r\nHost: localhost\r\nConnection: close\r\n\r\n")
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
            panic!("run command LASM missing-http-version test could not connect to server");
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
            panic!("run command LASM missing-http-version process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM missing-http-version process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic bad-request status:\n{response}"
    );
    assert!(
        response.contains("invalid request line: missing http version"),
        "response should include deterministic missing-version message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_505_for_unsupported_http_version() {
    let project_dir = temp_dir("sec4-run-command-lasm-unsupported-http-version");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmunsupportedhttpversioncommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before unsupported-version request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/2.0\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM unsupported-http-version test could not connect to server");
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
                "run command LASM unsupported-http-version process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM unsupported-http-version process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 505 HTTP Version Not Supported"),
        "response should contain deterministic unsupported-version status:\n{response}"
    );
    assert!(
        response.contains("unsupported http version: HTTP/2.0"),
        "response should include deterministic unsupported-version message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_431_when_request_line_exceeds_header_limit() {
    let project_dir = temp_dir("sec4-run-command-lasm-request-line-limit");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmrequestlinelimitcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[http]
max_header_bytes = 40
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
            "--backend",
            "lasm",
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

    let long_path = "a".repeat(40);
    let request = format!(
        "GET /{}/extra HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        long_path
    );

    let mut response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before request-line-limit request with status: {status}");
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
            panic!("run command LASM request-line-limit test could not connect to server");
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
            panic!("run command LASM request-line-limit process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM request-line-limit process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 431 Request Header Fields Too Large"),
        "response should contain 431 status line:\n{response}"
    );
    assert!(
        response.contains("request headers exceed configured limit (40 bytes)"),
        "response should include deterministic request-line limit message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_accepts_chunked_transfer_encoding() {
    let project_dir = temp_dir("sec4-run-command-lasm-transfer-encoding");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmtransferencodingcommand"
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
            "--backend",
            "lasm",
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
            panic!(
                "run command exited before chunked transfer-encoding request with status: {status}"
            );
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n4\r\npong\r\n0\r\n\r\n",
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
            panic!("run command LASM chunked transfer-encoding test could not connect to server");
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
            panic!("run command LASM chunked transfer-encoding process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM chunked transfer-encoding process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain deterministic chunked transfer-encoding status:\n{response}"
    );
    assert!(
        response.contains("\r\npong"),
        "response should include route body for chunked transfer-encoding request:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_417_for_expect_header() {
    let project_dir = temp_dir("sec4-run-command-lasm-expect-header");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmexpectheadercommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before expect-header request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /health HTTP/1.1\r\nHost: localhost\r\nExpect: 100-continue\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM expect-header test could not connect to server");
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
            panic!("run command LASM expect-header process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM expect-header process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 417 Expectation Failed"),
        "response should contain deterministic expect-header status:\n{response}"
    );
    assert!(
        response.contains("expect header is not supported"),
        "response should include deterministic expect-header message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_conflicting_content_length_headers() {
    let project_dir = temp_dir("sec4-run-command-lasm-conflicting-content-length");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmconflictingcontentlengthcommand"
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
            "--backend",
            "lasm",
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
            panic!(
                "run command exited before conflicting-content-length request with status: {status}"
            );
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /health HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nContent-Length: 4\r\nConnection: close\r\n\r\npong",
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
            panic!("run command LASM conflicting-content-length test could not connect to server");
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
                "run command LASM conflicting-content-length process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM conflicting-content-length process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic conflicting-content-length status:\n{response}"
    );
    assert!(
        response.contains("conflicting content-length headers"),
        "response should include deterministic conflicting-content-length message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_missing_host_header() {
    let project_dir = temp_dir("sec4-run-command-lasm-missing-host");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmmissinghostcommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before missing-host request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(b"GET /health HTTP/1.1\r\nConnection: close\r\n\r\n")
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
            panic!("run command LASM missing-host test could not connect to server");
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
            panic!("run command LASM missing-host process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM missing-host process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic missing-host status:\n{response}"
    );
    assert!(
        response.contains("missing host header"),
        "response should include deterministic missing-host message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_conflicting_host_headers() {
    let project_dir = temp_dir("sec4-run-command-lasm-conflicting-host");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmconflictinghostcommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before conflicting-host request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nHost: example.com\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM conflicting-host test could not connect to server");
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
            panic!("run command LASM conflicting-host process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM conflicting-host process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic conflicting-host status:\n{response}"
    );
    assert!(
        response.contains("conflicting host headers"),
        "response should include deterministic conflicting-host message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_merges_duplicate_request_headers_case_insensitively() {
    let project_dir = temp_dir("sec4-run-command-lasm-merge-duplicate-request-headers");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmmergeduplicaterequestheaderscommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let request_id = validate.nonEmpty(req.header("X-Request-Id"));
  res.text(200, request_id);
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
            "--backend",
            "lasm",
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
            panic!("run command exited before duplicate-header request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nX-Request-Id: req-1\r\nx-request-id: req-2\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM duplicate-header merge test could not connect to server");
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
                "run command LASM duplicate-header merge process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM duplicate-header merge process should exit successfully"
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
        response.contains("\r\n\r\nreq-1, req-2"),
        "response should merge duplicate request headers case-insensitively in arrival order:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_accepts_absolute_form_request_target() {
    let project_dir = temp_dir("sec4-run-command-lasm-absolute-form-target");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmabsoluteformtargetcommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before absolute-form request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET http://localhost/health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM absolute-form test could not connect to server");
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
            panic!("run command LASM absolute-form process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM absolute-form process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain successful absolute-form status:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npong"),
        "response should include expected body for absolute-form target:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_absolute_form_host_mismatch() {
    let project_dir = temp_dir("sec4-run-command-lasm-absolute-form-host-mismatch");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmabsoluteformhostmismatchcommand"
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
            "--backend",
            "lasm",
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
            panic!(
                "run command exited before absolute-form host-mismatch request with status: {status}"
            );
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET http://localhost/health HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM absolute-form host-mismatch test could not connect to server");
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
                "run command LASM absolute-form host-mismatch process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM absolute-form host-mismatch process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic absolute-form host-mismatch status:\n{response}"
    );
    assert!(
        response.contains("host header does not match request target authority"),
        "response should include deterministic absolute-form host-mismatch message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_accepts_absolute_form_default_port_equivalence() {
    let project_dir = temp_dir("sec4-run-command-lasm-absolute-form-default-port");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmabsoluteformdefaultportcommand"
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
            "--backend",
            "lasm",
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
            panic!(
                "run command exited before absolute-form default-port request with status: {status}"
            );
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET http://localhost:80/health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM absolute-form default-port test could not connect to server");
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
                "run command LASM absolute-form default-port process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM absolute-form default-port process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain successful absolute-form default-port status:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npong"),
        "response should include expected body for absolute-form default-port target:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_invalid_host_header_format() {
    let project_dir = temp_dir("sec4-run-command-lasm-invalid-host-header");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasminvalidhostheadercommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before invalid-host-header request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost,example.com\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM invalid-host-header test could not connect to server");
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
            panic!("run command LASM invalid-host-header process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM invalid-host-header process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic invalid-host-header status:\n{response}"
    );
    assert!(
        response.contains("invalid host header"),
        "response should include deterministic invalid-host-header message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_absolute_form_userinfo_authority() {
    let project_dir = temp_dir("sec4-run-command-lasm-absolute-form-userinfo-authority");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmabsoluteformuserinfoauthoritycommand"
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
            "--backend",
            "lasm",
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
            panic!(
                "run command exited before absolute-form-userinfo request with status: {status}"
            );
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET http://user@localhost/health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!(
                "run command LASM absolute-form-userinfo-authority test could not connect to server"
            );
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
                "run command LASM absolute-form-userinfo-authority process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM absolute-form-userinfo-authority process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic absolute-form-userinfo status:\n{response}"
    );
    assert!(
        response.contains("invalid request target: malformed absolute-form"),
        "response should include deterministic absolute-form-userinfo message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_invalid_request_target() {
    let project_dir = temp_dir("sec4-run-command-lasm-invalid-request-target");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasminvalidrequesttargetcommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before invalid-target request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM invalid-target test could not connect to server");
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
            panic!("run command LASM invalid-target process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM invalid-target process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic invalid-target status:\n{response}"
    );
    assert!(
        response.contains("invalid request target"),
        "response should include deterministic invalid-target message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_invalid_method_token() {
    let project_dir = temp_dir("sec4-run-command-lasm-invalid-method-token");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasminvalidmethodtokencommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before invalid-method request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"G@T /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM invalid-method test could not connect to server");
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
            panic!("run command LASM invalid-method process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM invalid-method process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic invalid-method status:\n{response}"
    );
    assert!(
        response.contains("invalid request line: invalid method token"),
        "response should include deterministic invalid-method message:\n{response}"
    );
    assert!(
        response.contains("Content-Type: application/json; charset=utf-8"),
        "parser failures should emit JSON envelope content-type:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"HTTP.BAD_REQUEST\"")
            && response.contains("\"kind\":\"validation\""),
        "parser failure response should contain deterministic envelope code/kind:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Origin:"),
        "parser failures should suppress default CORS headers:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_header_name_whitespace() {
    let project_dir = temp_dir("sec4-run-command-lasm-header-name-whitespace");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmheadernamewhitespacecommand"
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
            "--backend",
            "lasm",
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
            panic!(
                "run command exited before header-name-whitespace request with status: {status}"
            );
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost : localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM header-name-whitespace test could not connect to server");
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
                "run command LASM header-name-whitespace process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM header-name-whitespace process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic header-name-whitespace status:\n{response}"
    );
    assert!(
        response.contains("invalid header line: whitespace around header name"),
        "response should include deterministic header-name-whitespace message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_invalid_header_encoding() {
    let project_dir = temp_dir("sec4-run-command-lasm-invalid-header-encoding");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasminvalidheaderencodingcommand"
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
            "--backend",
            "lasm",
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
            panic!(
                "run command exited before invalid-header-encoding request with status: {status}"
            );
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: local\xffhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM invalid-header-encoding test could not connect to server");
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
                "run command LASM invalid-header-encoding process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM invalid-header-encoding process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic invalid-header-encoding status:\n{response}"
    );
    assert!(
        response.contains("invalid request encoding while reading header line"),
        "response should include deterministic invalid-header-encoding message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_invalid_header_value_character() {
    let project_dir = temp_dir("sec4-run-command-lasm-invalid-header-value-char");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasminvalidheadervaluecharcommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before invalid-header-value request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nX-Demo: ok\x07bad\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM invalid-header-value test could not connect to server");
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
            panic!("run command LASM invalid-header-value process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM invalid-header-value process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic invalid-header-value status:\n{response}"
    );
    assert!(
        response.contains("invalid header line: invalid header value character"),
        "response should include deterministic invalid-header-value message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_request_target_fragment() {
    let project_dir = temp_dir("sec4-run-command-lasm-request-target-fragment");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmrequesttargetfragmentcommand"
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
            "--backend",
            "lasm",
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
            panic!("run command exited before fragment-target request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health#frag HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM fragment-target test could not connect to server");
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
            panic!("run command LASM fragment-target process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM fragment-target process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic fragment-target status:\n{response}"
    );
    assert!(
        response.contains("invalid request target: fragment is not allowed"),
        "response should include deterministic fragment-target message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_408_when_request_read_times_out() {
    let project_dir = temp_dir("sec4-run-command-lasm-read-timeout");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmreadtimeoutcommand"
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
            "--backend",
            "lasm",
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "200",
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
            panic!("run command exited before timeout probe with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("timeout response should be readable");
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
            panic!("run command LASM read-timeout test could not connect to server");
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
            panic!("run command LASM read-timeout process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM read-timeout process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 408 Request Timeout"),
        "response should contain deterministic request-timeout status:\n{response}"
    );
    assert!(
        response.contains("request read timeout while reading request line"),
        "response should include deterministic request-timeout message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_respects_cors_and_security_policy_headers() {
    let project_dir = temp_dir("sec4-run-command-lasm-policy-headers");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmpolicyheaderscommand"
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
allowed_methods = ["GET", "POST"]
allowed_headers = ["content-type", "authorization"]
max_age_seconds = 123
allow_credentials = true
allow_private_network = true
require_vary_origin = true
exposed_headers = ["x-trace-id"]

[security_headers]
enabled = true
x_content_type_options = true
x_frame_options = "DENY"
referrer_policy = "no-referrer"

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
            "--backend",
            "lasm",
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
            panic!("run command LASM policy-header test could not connect to server");
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
            panic!("run command LASM policy-header process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM policy-header process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: https://frontend.example"),
        "response should include policy-derived allow-origin header:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Expose-Headers: x-trace-id"),
        "response should include policy-derived expose-headers value:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Credentials: true"),
        "response should include policy-derived allow-credentials value:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Methods: GET,POST"),
        "response should include policy-derived allow-methods value:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Headers: content-type,authorization"),
        "response should include policy-derived allow-headers value:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Max-Age: 123"),
        "response should include policy-derived max-age value:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Private-Network: true"),
        "response should include policy-derived private-network value:\n{response}"
    );
    assert!(
        response.contains("Vary: Origin"),
        "response should include policy-derived vary-origin value:\n{response}"
    );
    assert!(
        response.contains("X-Frame-Options: DENY"),
        "response should include policy-derived x-frame-options value:\n{response}"
    );
    assert!(
        response.contains("Referrer-Policy: no-referrer"),
        "response should include policy-derived referrer-policy value:\n{response}"
    );
    assert!(
        response.contains("Strict-Transport-Security: max-age=777"),
        "response should include policy-derived HSTS header:\n{response}"
    );
    assert!(
        response.contains(
            "Content-Security-Policy-Report-Only: default-src 'none'; frame-ancestors 'none'"
        ),
        "response should include policy-derived report-only CSP header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_allows_cors_private_network_preflight() {
    let project_dir = temp_dir("sec4-run-command-lasm-cors-preflight");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmcorspreflightcommand"
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
            "--backend",
            "lasm",
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
            panic!("run command LASM CORS preflight test could not connect to server");
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
            panic!("run command LASM CORS preflight process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM CORS preflight process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain deterministic 204 preflight status:\n{response}"
    );
    assert!(
        response.contains("Content-Length: 0"),
        "response should include zero body length for preflight:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: https://frontend.example"),
        "response should include policy-derived allow-origin header:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Private-Network: true"),
        "response should include policy-derived private-network header:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Methods: POST"),
        "response should include policy-derived allow-methods header:\n{response}"
    );
    assert!(
        !response.contains("route not found"),
        "preflight should not fall back to route-not-found response:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_selects_matching_origin_from_allowlist() {
    let project_dir = temp_dir("sec4-run-command-lasm-cors-origin-selection");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmcorsoriginselectioncommand"
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
allowed_origins = ["https://frontend-a.example", "https://frontend-b.example"]
allow_credentials = true
allowed_methods = ["GET", "POST"]
allowed_headers = ["x-auth-token"]
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
            "--backend",
            "lasm",
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
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nOrigin: https://frontend-b.example\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM CORS origin-selection test could not connect to server");
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
                "run command LASM CORS origin-selection process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM CORS origin-selection process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain deterministic 200 status:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: https://frontend-b.example"),
        "response should select matching origin from policy allowlist:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Origin: https://frontend-a.example"),
        "response should not fall back to first allowlist origin when request origin matches a later entry:\n{response}"
    );
    assert!(
        response.contains("Vary: Origin"),
        "response should keep vary-origin semantics with multi-origin allowlists:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_omits_allow_origin_for_disallowed_origin() {
    let project_dir = temp_dir("sec4-run-command-lasm-cors-origin-disallowed");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmcorsorigindisallowedcommand"
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
allowed_origins = ["https://frontend-a.example", "https://frontend-b.example"]
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
            "--backend",
            "lasm",
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
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nOrigin: https://unknown.example\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM CORS disallowed-origin test could not connect to server");
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
                "run command LASM CORS disallowed-origin process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM CORS disallowed-origin process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain deterministic 200 status:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Origin:"),
        "response should not emit allow-origin header for disallowed request origin:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Credentials:"),
        "response should not emit other CORS allow headers for disallowed request origin:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Methods:"),
        "response should not emit CORS method defaults for disallowed request origin:\n{response}"
    );
    assert!(
        !response.contains("Vary: Origin"),
        "response should omit vary-origin when cors headers are suppressed for disallowed origin:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_supports_bound_res_text_arguments() {
    let project_dir = temp_dir("sec4-run-command-lasm-bound-res-text");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmboundrestextcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let status = 200;
  let body = "pong";
  res.text(status, body);
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
            "--backend",
            "lasm",
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
            panic!("run command LASM bound res.text test could not connect to server");
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
            panic!("run command LASM bound res.text process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM bound res.text process should exit successfully"
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
        "response should include bound response body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_lasm_backend_materializes_req_placeholders_in_res_text() {
    let project_dir = temp_dir("sec4-run-command-lasm-req-placeholders");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmreqplaceholdercommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn userRoute() effects { net } -> Int {
  let id = validate.nonEmpty(req.pathParam("id"));
  res.text(200, id);
  0
}

fn headerRoute() effects { net } -> Int {
  let request_id = validate.nonEmpty(req.header("X-Request-Id"));
  res.text(200, request_id);
  0
}

fn queryRoute() effects { net } -> Int {
  let trace = validate.nonEmpty(req.query("trace"));
  res.text(200, trace);
  0
}

fn cookieRoute() effects { net } -> Int {
  let session = validate.nonEmpty(req.cookie("session"));
  res.setHeader(headers.name("X-Session-Echo"), headers.value(session));
  res.text(200, "cookie={{req.cookie:session}}");
  0
}

fn composeRoute() effects { net } -> Int {
  let echo_name = headers.name(validate.nonEmpty(req.query("header_name")));
  let trace_value = validate.headerValue(req.query("trace"));
  res.setHeader(echo_name, trace_value);
  res.setHeader(headers.name("X-Method-Echo"), headers.value(validate.nonEmpty(req.method())));
  res.setHeader(headers.name("X-Version-Echo"), headers.value(validate.nonEmpty(req.httpVersion())));
  res.setHeader(headers.name("X-Request-Id-Echo"), headers.value(validate.nonEmpty(req.header("X-Request-Id"))));
  res.text(200, "version={{req.httpVersion}};method={{req.method}};path={{req.path}};id={{req.pathParam:id}};trace={{req.query:trace}};requestId={{req.header:X-Request-Id}}");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/users/:id", userRoute);
  http.get(router, "/request-id", headerRoute);
  http.get(router, "/query", queryRoute);
  http.get(router, "/cookie", cookieRoute);
  http.get(router, "/compose/:id", composeRoute);
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
            "--backend",
            "lasm",
            "--port",
            port_value.as_str(),
            "--serve-timeout-ms",
            "5000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut user_response = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before /users request with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /users/42 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("/users request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("/users response should be readable");
                user_response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let user_response = match user_response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command LASM req placeholder test could not connect /users request");
        }
    };

    let mut header_response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /request-id HTTP/1.1\r\nHost: localhost\r\nX-Request-Id: req-42\r\nConnection: close\r\n\r\n",
                    )
                    .expect("/request-id request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("/request-id response should be readable");
                header_response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let header_response = match header_response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command LASM req placeholder test could not connect /request-id request");
        }
    };

    let mut query_response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /query?tra%63e=q%2B7+ok HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("/query request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("/query response should be readable");
                query_response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let query_response = match query_response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command LASM req placeholder test could not connect /query request");
        }
    };

    let mut query_duplicate_response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /query?trace=first&trace=second HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("/query duplicate-key request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("/query duplicate-key response should be readable");
                query_duplicate_response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let query_duplicate_response = match query_duplicate_response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "run command LASM req placeholder test could not connect /query duplicate-key request"
            );
        }
    };

    let mut cookie_response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /cookie HTTP/1.1\r\nHost: localhost\r\nCookie: mode=active; session=sess-42\r\nConnection: close\r\n\r\n",
                    )
                    .expect("/cookie request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("/cookie response should be readable");
                cookie_response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let cookie_response = match cookie_response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command LASM req placeholder test could not connect /cookie request");
        }
    };

    let mut cookie_case_variant_response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /cookie HTTP/1.1\r\nHost: localhost\r\nCookie: Session=sess-43; mode=active\r\nConnection: close\r\n\r\n",
                    )
                    .expect("/cookie case-variant request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("/cookie case-variant response should be readable");
                cookie_case_variant_response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let cookie_case_variant_response = match cookie_case_variant_response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command LASM req placeholder test could not connect /cookie case-variant request");
        }
    };

    let mut compose_response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /compose/user-7?trace=q%2B7+ok&header_name=X-Trace-Echo HTTP/1.1\r\nHost: localhost\r\nX-Request-Id: req-99\r\nConnection: close\r\n\r\n",
                    )
                    .expect("/compose request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("/compose response should be readable");
                compose_response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let compose_response = match compose_response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command LASM req placeholder test could not connect /compose request");
        }
    };

    let mut compose_invalid_name_response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /compose/user-8?trace=q%2B8+ok&header_name=X_Trace_Echo HTTP/1.1\r\nHost: localhost\r\nX-Request-Id: req-100\r\nConnection: close\r\n\r\n",
                    )
                    .expect("/compose invalid-name request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("/compose invalid-name response should be readable");
                compose_invalid_name_response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let compose_invalid_name_response = match compose_invalid_name_response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "run command LASM req placeholder test could not connect /compose invalid-name request"
            );
        }
    };

    let mut query_invalid_escape_response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /query?trace=bad%zz HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("/query invalid-escape request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("/query invalid-escape response should be readable");
                query_invalid_escape_response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let query_invalid_escape_response = match query_invalid_escape_response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "run command LASM req placeholder test could not connect /query invalid-escape request"
            );
        }
    };

    assert!(
        user_response.contains("HTTP/1.1 200 OK"),
        "/users response should contain 200 status line:\n{user_response}"
    );
    assert!(
        user_response.contains("\r\n\r\n42"),
        "/users response should materialize path param in body:\n{user_response}"
    );
    assert!(
        header_response.contains("HTTP/1.1 200 OK"),
        "/request-id response should contain 200 status line:\n{header_response}"
    );
    assert!(
        header_response.contains("\r\n\r\nreq-42"),
        "/request-id response should materialize request header in body:\n{header_response}"
    );
    assert!(
        query_response.contains("HTTP/1.1 200 OK"),
        "/query response should contain 200 status line:\n{query_response}"
    );
    assert!(
        query_response.contains("\r\n\r\nq+7 ok"),
        "/query response should materialize request query value in body:\n{query_response}"
    );
    assert!(
        query_duplicate_response.contains("HTTP/1.1 200 OK"),
        "/query duplicate-key response should contain 200 status line:\n{query_duplicate_response}"
    );
    assert!(
        query_duplicate_response.contains("\r\n\r\nfirst"),
        "/query duplicate-key response should keep first query value for duplicate keys:\n{query_duplicate_response}"
    );
    assert!(
        cookie_response.contains("HTTP/1.1 200 OK"),
        "/cookie response should contain 200 status line:\n{cookie_response}"
    );
    assert!(
        cookie_response.contains("X-Session-Echo: sess-42"),
        "/cookie response should materialize request cookie in response header:\n{cookie_response}"
    );
    assert!(
        cookie_response.contains("\r\n\r\ncookie=sess-42"),
        "/cookie response should materialize request cookie in response body:\n{cookie_response}"
    );
    assert!(
        cookie_case_variant_response.contains("HTTP/1.1 200 OK"),
        "/cookie case-variant response should contain 200 status line:\n{cookie_case_variant_response}"
    );
    assert!(
        cookie_case_variant_response.contains("X-Session-Echo: sess-43"),
        "/cookie case-variant response should match req.cookie lookups case-insensitively:\n{cookie_case_variant_response}"
    );
    assert!(
        cookie_case_variant_response.contains("\r\n\r\ncookie=sess-43"),
        "/cookie case-variant response should materialize request cookie case-insensitively in body:\n{cookie_case_variant_response}"
    );
    assert!(
        compose_response.contains("HTTP/1.1 200 OK"),
        "/compose response should contain 200 status line:\n{compose_response}"
    );
    assert!(
        compose_response.contains("X-Trace-Echo: q+7 ok"),
        "/compose response should materialize query-driven placeholder for response header:\n{compose_response}"
    );
    assert!(
        compose_response.contains("X-Method-Echo: GET"),
        "/compose response should materialize request method for response header:\n{compose_response}"
    );
    assert!(
        compose_response.contains("X-Version-Echo: HTTP/1.1"),
        "/compose response should materialize request http version for response header:\n{compose_response}"
    );
    assert!(
        compose_response.contains("X-Request-Id-Echo: req-99"),
        "/compose response should materialize request-header placeholder for response header:\n{compose_response}"
    );
    assert!(
        compose_response.contains(
            "\r\n\r\nversion=HTTP/1.1;method=GET;path=/compose/user-7;id=user-7;trace=q+7 ok;requestId=req-99",
        ),
        "/compose response should materialize version/method/path and request-derived placeholders in body:\n{compose_response}"
    );
    assert!(
        compose_invalid_name_response.contains("HTTP/1.1 200 OK"),
        "/compose invalid-name response should contain 200 status line:\n{compose_invalid_name_response}"
    );
    assert!(
        !compose_invalid_name_response.contains("X_Trace_Echo:"),
        "/compose invalid-name response should drop dynamic headers whose materialized names fail header-name grammar:\n{compose_invalid_name_response}"
    );
    assert!(
        compose_invalid_name_response.contains("X-Method-Echo: GET"),
        "/compose invalid-name response should keep valid request-method response header:\n{compose_invalid_name_response}"
    );
    assert!(
        compose_invalid_name_response.contains("X-Version-Echo: HTTP/1.1"),
        "/compose invalid-name response should keep valid request-version response header:\n{compose_invalid_name_response}"
    );
    assert!(
        compose_invalid_name_response.contains("X-Request-Id-Echo: req-100"),
        "/compose invalid-name response should keep valid static response headers:\n{compose_invalid_name_response}"
    );
    assert!(
        compose_invalid_name_response.contains(
            "\r\n\r\nversion=HTTP/1.1;method=GET;path=/compose/user-8;id=user-8;trace=q+8 ok;requestId=req-100",
        ),
        "/compose invalid-name response should keep version/method/path and dynamic body materialization intact:\n{compose_invalid_name_response}"
    );
    assert!(
        query_invalid_escape_response.contains("HTTP/1.1 200 OK"),
        "/query invalid-escape response should contain 200 status line:\n{query_invalid_escape_response}"
    );
    assert!(
        query_invalid_escape_response.contains("\r\n\r\nbad%zz"),
        "/query invalid-escape response should keep raw query value on invalid percent sequence:\n{query_invalid_escape_response}"
    );

    let _ = child.kill();
    let _ = child.wait();
    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_emits_set_cookie_from_add_cookie() {
    let project_dir = temp_dir("sec4-run-command-lasm-set-cookie");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmsetcookiecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let cookie_name = validate.nonEmpty(req.query("cookie_name"));
  let cookie_value = cookie.build(cookie_name, validate.nonEmpty(req.query("session")));
  let mode_cookie = cookie.build("mode", "active");
  res.addCookie(cookie_value);
  res.addCookie(mode_cookie);
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
            "--backend",
            "lasm",
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
                        b"GET /health?cookie_name=demo_session&session=demo%2Btoken HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM set-cookie test could not connect to server");
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
            panic!("run command LASM set-cookie process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM set-cookie process should exit successfully"
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
        response.contains("Set-Cookie: demo_session=demo+token"),
        "response should include query-derived set-cookie name/value header:\n{response}"
    );
    assert!(
        response.contains("Set-Cookie: mode=active"),
        "response should include additional set-cookie header when multiple cookies are added:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Expose-Headers: x-trace-id,x-showcase"),
        "response should include deterministic exposed-header list:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npong"),
        "response should include expected body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_supports_bound_route_and_handler_aliases() {
    let project_dir = temp_dir("sec4-run-command-lasm-bound-route-handler");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmboundroutehandlercommand"
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
  let route_path = "/health";
  let route_handler = health;
  http.get(router, route_path, route_handler);
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
            "--backend",
            "lasm",
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
            panic!("run command LASM bound route/handler test could not connect to server");
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
            panic!("run command LASM bound route/handler process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM bound route/handler process should exit successfully"
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
fn run_command_oneshot_lasm_backend_sets_json_content_type_for_res_ok() {
    let project_dir = temp_dir("sec4-run-command-lasm-json-content-type");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmjsoncontenttypecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn createUser() effects { net } -> Int {
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
            "--backend",
            "lasm",
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
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM json content-type test could not connect to server");
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
            panic!("run command LASM json content-type process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM json content-type process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 201 Created"),
        "response should contain extracted success status line:\n{response}"
    );
    assert!(
        response.contains("Content-Type: application/json; charset=utf-8"),
        "response should include deterministic JSON content-type:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\n{\"ok\":true,\"status\":201,\"schema\":\"CreateUserResponse\"}"),
        "response should include deterministic res.ok JSON envelope body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_merges_header_names_case_insensitively() {
    let project_dir = temp_dir("sec4-run-command-lasm-header-case-merge");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmheadercasemergecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.setHeader(headers.name("content-type"), headers.value("application/x-sec4-demo"));
  res.setHeader(headers.name("x-trace-id"), headers.value("client-supplied-trace"));
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
            "--backend",
            "lasm",
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
            panic!("run command LASM header-case merge test could not connect to server");
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
            panic!("run command LASM header-case merge process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM header-case merge process should exit successfully"
    );
    let response_lower = response.to_ascii_lowercase();
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response_lower.matches("\r\ncontent-type:").count() == 1,
        "response should include exactly one content-type header after case-insensitive merge:\n{response}"
    );
    assert!(
        response_lower.contains("\r\ncontent-type: application/x-sec4-demo"),
        "response should preserve user-provided content-type value without injecting fallback default:\n{response}"
    );
    assert!(
        !response_lower.contains("text/plain; charset=utf-8"),
        "response should not include fallback content-type when case-insensitive match exists:\n{response}"
    );
    assert!(
        response_lower.matches("\r\nx-trace-id:").count() == 1,
        "response should include exactly one x-trace-id header after trace injection:\n{response}"
    );
    assert!(
        response_lower.contains("\r\nx-trace-id: rt-1"),
        "runtime trace injection should overwrite user-provided x-trace-id value deterministically:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_overrides_user_content_length_with_actual_body_size() {
    let project_dir = temp_dir("sec4-run-command-lasm-content-length-override");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmcontentlengthoverridecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.setHeader(headers.name("content-length"), headers.value("999"));
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
            "--backend",
            "lasm",
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
            panic!("run command LASM content-length override test could not connect to server");
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
                "run command LASM content-length override process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM content-length override process should exit successfully"
    );
    let response_lower = response.to_ascii_lowercase();
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response_lower.matches("\r\ncontent-length:").count() == 1,
        "response should include exactly one content-length header:\n{response}"
    );
    assert!(
        response_lower.contains("\r\ncontent-length: 4"),
        "response should emit actual body length regardless of user-provided content-length:\n{response}"
    );
    assert!(
        !response_lower.contains("\r\ncontent-length: 999"),
        "response should not preserve invalid user-provided content-length:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_drops_invalid_dynamic_response_headers() {
    let project_dir = temp_dir("sec4-run-command-lasm-invalid-dynamic-headers");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasminvaliddynamicheaderscommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  let dynamic_name = headers.name(validate.nonEmpty(req.query("name")));
  let dynamic_value = headers.value(validate.nonEmpty(req.query("value")));
  res.setHeader(dynamic_name, dynamic_value);
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
            "--backend",
            "lasm",
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
                        b"GET /health?name=X-Bad%0D%0AInjected&value=ok%0D%0AInjected:%20yes HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM invalid dynamic headers test could not connect to server");
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
                "run command LASM invalid dynamic headers process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM invalid dynamic headers process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npong"),
        "response should include expected body:\n{response}"
    );
    assert!(
        !response.to_ascii_lowercase().contains("injected:"),
        "response should not emit header-injection artifacts from invalid dynamic header materialization:\n{response}"
    );
    assert!(
        !response.contains("\r\nX-Bad"),
        "response should drop invalid dynamic header name/value pairs:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_drops_empty_dynamic_response_header_values() {
    let project_dir = temp_dir("sec4-run-command-lasm-empty-dynamic-header-values");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmemptydynamicheadervaluescommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  res.setHeader(headers.name("X-Dynamic"), headers.value("{{req.query:value}}"));
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
            "--backend",
            "lasm",
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
                        b"GET /health?value= HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM empty dynamic header values test could not connect to server");
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
                "run command LASM empty dynamic header values process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM empty dynamic header values process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\npong"),
        "response should include expected body:\n{response}"
    );
    assert!(
        !response.to_ascii_lowercase().contains("\r\nx-dynamic:"),
        "response should drop dynamic response headers with empty values:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_invalid_json_payload() {
    let project_dir = temp_dir("sec4-run-command-lasm-invalid-json");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasminvalidjsoncommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn createUser() effects { net } -> Int {
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
            "--backend",
            "lasm",
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
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 1\r\nConnection: close\r\n\r\n{",
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
            panic!("run command LASM invalid-json test could not connect to server");
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
            panic!("run command LASM invalid-json process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM invalid-json process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain invalid-json status line:\n{response}"
    );
    assert!(
        response.contains("Content-Type: application/json; charset=utf-8"),
        "response should include deterministic JSON content-type:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"JSON.INVALID_SYNTAX\""),
        "response should include deterministic invalid-json error code:\n{response}"
    );
    assert!(
        response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic trace id in error envelope:\n{response}"
    );
    assert!(
        response.contains("\"timeMs\":"),
        "response should include deterministic error timestamp field:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_invalid_user_payload() {
    let project_dir = temp_dir("sec4-run-command-lasm-invalid-user-payload");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasminvaliduserpayloadcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn createUser() effects { net } -> Int {
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
            "--backend",
            "lasm",
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

    let payload = r#"{"id":"123e4567-e89b-42d3-a456-426614174000","email":"bad-email","age":30,"name":"alice","tags":["demo"],"address":{"zip":"12345"},"meta":{"flags":{"a":true,"b":false,"c":true}}}"#;
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
            panic!("run command LASM invalid-user-payload test could not connect to server");
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
            panic!("run command LASM invalid-user-payload process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM invalid-user-payload process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain invalid-user-payload status line:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"VALIDATION.INVALID\""),
        "response should include deterministic validation error code:\n{response}"
    );
    assert!(
        response.contains("email must be a valid email string"),
        "response should include deterministic validation message:\n{response}"
    );
    assert!(
        response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic trace id in validation envelope:\n{response}"
    );
    assert!(
        response.contains("\"timeMs\":"),
        "response should include deterministic validation timestamp field:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_400_for_non_json_content_type_payload() {
    let project_dir = temp_dir("sec4-run-command-lasm-non-json-content-type");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmnonjsoncontenttypecommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn createUser() effects { net } -> Int {
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
            "--backend",
            "lasm",
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

    let payload = r#"{"id":"123e4567-e89b-42d3-a456-426614174000"}"#;
    let request = format!(
        "POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
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
            panic!("run command LASM non-json content-type test could not connect to server");
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
                "run command LASM non-json content-type process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command LASM non-json content-type process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain non-json content-type status line:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"HTTP.BAD_REQUEST\""),
        "response should include deterministic bad-request code:\n{response}"
    );
    assert!(
        response.contains("content-type must be application/json"),
        "response should include deterministic content-type guidance:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_head_request_omits_response_body() {
    let project_dir = temp_dir("sec4-run-command-lasm-head-body");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmheadbodycommand"
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
            "--backend",
            "lasm",
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
                        b"HEAD /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM HEAD test could not connect to server");
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
            panic!("run command LASM HEAD process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM HEAD process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "HEAD response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "HEAD response should include deterministic trace header:\n{response}"
    );
    assert!(
        response.contains("Content-Length: 0"),
        "HEAD response should report zero response-body length:\n{response}"
    );
    assert!(
        !response.contains("\r\n\r\npong"),
        "HEAD response should omit response body bytes:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_405_with_allow_header_on_method_mismatch() {
    let project_dir = temp_dir("sec4-run-command-lasm-method-mismatch");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmmethodmismatchcommand"
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
            "--backend",
            "lasm",
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
                        b"POST /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM method-mismatch test could not connect to server");
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
            panic!("run command LASM method-mismatch process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM method-mismatch process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 405 Method Not Allowed"),
        "response should include deterministic 405 status line for method mismatch:\n{response}"
    );
    assert!(
        response.contains("Allow: GET, HEAD"),
        "response should include deterministic allow header:\n{response}"
    );
    assert!(
        response.contains("method not allowed"),
        "response should include deterministic method-mismatch response body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_enforces_body_limit_override() {
    let project_dir = temp_dir("sec4-run-command-lasm-body-limit");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmbodylimitcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn create() effects { net } -> Int {
  res.text(201, "created");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.post(router, "/users", create);
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
            "--backend",
            "lasm",
            "--oneshot",
            "--port",
            port_value.as_str(),
            "--max-body-bytes",
            "4",
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
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Length: 8\r\nConnection: close\r\n\r\nabcdefgh",
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
            panic!("run command LASM body-limit test could not connect to server");
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
            panic!("run command LASM body-limit process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM body-limit process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 413 Payload Too Large"),
        "response should contain deterministic body-limit status line:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );
    assert!(
        response.contains("request body exceeds configured limit (4 bytes)"),
        "response should include deterministic body-limit error body:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_returns_431_when_header_limit_is_exceeded() {
    let project_dir = temp_dir("sec4-run-command-lasm-header-limit");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmheaderlimitcommand"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("sec4.policy"),
        r#"[http]
max_header_bytes = 80
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
            "--backend",
            "lasm",
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
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nX-Long-Header: 1234567890123456789012345678901234567890\r\nConnection: close\r\n\r\n",
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
            panic!("run command LASM header-limit test could not connect to server");
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
            panic!("run command LASM header-limit process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command LASM header-limit process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 431 Request Header Fields Too Large"),
        "response should contain deterministic header-limit status:\n{response}"
    );
    assert!(
        response.contains("request headers exceed configured limit"),
        "response should include deterministic header-limit message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_lasm_backend_returns_503_when_concurrency_limit_is_reached() {
    let project_dir = temp_dir("sec4-run-command-lasm-concurrency-limit");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmconcurrencylimitcommand"
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
            "--backend",
            "lasm",
            "--port",
            port_value.as_str(),
            "--max-concurrency",
            "1",
            "--serve-timeout-ms",
            "5000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut held = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before opening held connection with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\n")
                    .expect("held partial request should be written");
                held = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _held_stream = match held {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("could not establish held connection for LASM concurrency-limit test");
        }
    };

    thread::sleep(Duration::from_millis(120));

    let mut queued = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => {
                queued = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _queued_stream = match queued {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("could not establish queued connection for LASM concurrency-limit test");
        }
    };

    thread::sleep(Duration::from_millis(120));

    let mut response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
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
            panic!("run command LASM concurrency-limit test could not connect overflow client");
        }
    };

    assert!(
        response.contains("HTTP/1.1 503 Service Unavailable"),
        "response should contain deterministic service unavailable status:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-"),
        "response should include deterministic trace header prefix:\n{response}"
    );
    assert!(
        response.contains("server busy: max concurrency reached"),
        "response should include deterministic concurrency-limit body:\n{response}"
    );

    let _ = child.kill();
    let _ = child.wait();
    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_lasm_backend_overflow_path_returns_parse_error_for_malformed_request() {
    let project_dir = temp_dir("sec4-run-command-lasm-overflow-parse-error");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmoverflowparseerrorcommand"
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
            "--backend",
            "lasm",
            "--port",
            port_value.as_str(),
            "--max-concurrency",
            "1",
            "--max-pending",
            "1",
            "--serve-timeout-ms",
            "5000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut held = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before opening held connection with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\n")
                    .expect("held partial request should be written");
                held = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _held_stream = match held {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("could not establish held connection for LASM overflow-parse-error test");
        }
    };

    thread::sleep(Duration::from_millis(120));

    let mut queued = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => {
                queued = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _queued_stream = match queued {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("could not establish queued connection for LASM overflow-parse-error test");
        }
    };

    thread::sleep(Duration::from_millis(120));

    let mut response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"G@T /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("overflow malformed request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("overflow response should be readable");
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
            panic!("run command LASM overflow-parse-error test could not connect overflow client");
        }
    };

    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "overflow parser response should contain deterministic bad-request status:\n{response}"
    );
    assert!(
        response.contains("\"code\":\"HTTP.BAD_REQUEST\"")
            && response.contains("\"kind\":\"validation\""),
        "overflow parser response should include deterministic code/kind:\n{response}"
    );
    assert!(
        response.contains("invalid request line: invalid method token"),
        "overflow parser response should include deterministic parser message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-"),
        "overflow parser response should include deterministic trace header:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Origin: "),
        "overflow parser response should suppress CORS defaults:\n{response}"
    );

    let _ = child.kill();
    let _ = child.wait();
    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_lasm_backend_supports_keep_alive_for_multiple_requests() {
    let project_dir = temp_dir("sec4-run-command-lasm-keep-alive");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmkeepalivecommand"
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
            "--backend",
            "lasm",
            "--port",
            port_value.as_str(),
            "--max-concurrency",
            "1",
            "--serve-timeout-ms",
            "5000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut stream = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before keep-alive connect with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(next) => {
                stream = Some(next);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let mut stream = match stream {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command LASM keep-alive test could not connect to server");
        }
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("keep-alive stream read timeout should be configurable");

    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\n\r\n")
        .expect("first keep-alive request should be written");
    let mut reader = BufReader::new(&mut stream);
    let first_response = read_http_response(&mut reader);
    assert!(
        first_response.contains("HTTP/1.1 200 OK"),
        "first response should be successful:\n{first_response}"
    );
    assert!(
        first_response.contains("Connection: keep-alive"),
        "first response should keep connection open:\n{first_response}"
    );
    assert!(
        first_response.ends_with("\r\n\r\npong"),
        "first response should include expected body:\n{first_response}"
    );

    reader
        .get_mut()
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .expect("second keep-alive request should be written");
    let second_response = read_http_response(&mut reader);
    assert!(
        second_response.contains("HTTP/1.1 200 OK"),
        "second response should be successful:\n{second_response}"
    );
    assert!(
        second_response.contains("Connection: close"),
        "second response should close the connection:\n{second_response}"
    );
    assert!(
        second_response.ends_with("\r\n\r\npong"),
        "second response should include expected body:\n{second_response}"
    );

    let _ = child.kill();
    let _ = child.wait();
    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_lasm_backend_supports_pipelined_requests_on_single_socket() {
    let project_dir = temp_dir("sec4-run-command-lasm-pipelined-requests");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmpipelinedrequestscommand"
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
            "--backend",
            "lasm",
            "--port",
            port_value.as_str(),
            "--max-concurrency",
            "1",
            "--serve-timeout-ms",
            "5000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut stream = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before pipelined connect with status: {status}");
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(next) => {
                stream = Some(next);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let mut stream = match stream {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("run command LASM pipelined test could not connect to server");
        }
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("pipelined stream read timeout should be configurable");

    stream
        .write_all(
            b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive, upgrade\r\n\r\nGET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        )
        .expect("pipelined requests should be written");
    let mut reader = BufReader::new(&mut stream);
    let first_response = read_http_response(&mut reader);
    let second_response = read_http_response(&mut reader);

    assert!(
        first_response.contains("HTTP/1.1 200 OK"),
        "first pipelined response should be successful:\n{first_response}"
    );
    assert!(
        first_response.contains("Connection: keep-alive"),
        "first pipelined response should keep connection open:\n{first_response}"
    );
    assert!(
        first_response.ends_with("\r\n\r\npong"),
        "first pipelined response should include expected body:\n{first_response}"
    );

    assert!(
        second_response.contains("HTTP/1.1 200 OK"),
        "second pipelined response should be successful:\n{second_response}"
    );
    assert!(
        second_response.contains("Connection: close"),
        "second pipelined response should close connection:\n{second_response}"
    );
    assert!(
        second_response.ends_with("\r\n\r\npong"),
        "second pipelined response should include expected body:\n{second_response}"
    );

    let _ = child.kill();
    let _ = child.wait();
    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_lasm_backend_overflow_head_omits_response_body() {
    let project_dir = temp_dir("sec4-run-command-lasm-overflow-head-bodyless");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmoverflowheadbodylesscommand"
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
            "--backend",
            "lasm",
            "--port",
            port_value.as_str(),
            "--max-concurrency",
            "1",
            "--max-pending",
            "1",
            "--serve-timeout-ms",
            "5000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut held = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before opening held connection with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\n")
                    .expect("held partial request should be written");
                held = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _held_stream = match held {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("could not establish held connection for LASM overflow HEAD test");
        }
    };

    thread::sleep(Duration::from_millis(120));

    let mut queued = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => {
                queued = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _queued_stream = match queued {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("could not establish queued connection for LASM overflow HEAD test");
        }
    };

    thread::sleep(Duration::from_millis(120));

    let mut response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"HEAD /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("overflow HEAD request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("overflow HEAD response should be readable");
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
            panic!("run command LASM overflow HEAD test could not connect overflow client");
        }
    };

    assert!(
        response.contains("HTTP/1.1 503 Service Unavailable"),
        "response should contain deterministic service unavailable status:\n{response}"
    );
    assert!(
        response.contains("Content-Type: application/json; charset=utf-8"),
        "response should keep deterministic JSON content-type on overload HEAD response:\n{response}"
    );
    assert!(
        response.contains("Content-Length: "),
        "response should include deterministic content length header:\n{response}"
    );
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("");
    assert!(
        body.is_empty(),
        "overflow HEAD response body must be empty:\n{response}"
    );

    let _ = child.kill();
    let _ = child.wait();
    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_lasm_backend_overflow_omits_cors_headers_for_disallowed_origin() {
    let project_dir = temp_dir("sec4-run-command-lasm-overflow-cors-origin-disallowed");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmoverflowcorsorigindisallowedcommand"
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
allowed_origins = ["https://frontend-allowed.example"]
allow_credentials = true
allowed_methods = ["GET"]
allowed_headers = ["x-auth-token"]
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
            "--backend",
            "lasm",
            "--port",
            port_value.as_str(),
            "--max-concurrency",
            "1",
            "--max-pending",
            "1",
            "--serve-timeout-ms",
            "5000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut held = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before opening held connection with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\n")
                    .expect("held partial request should be written");
                held = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _held_stream = match held {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("could not establish held connection for LASM overflow disallowed-origin test");
        }
    };

    thread::sleep(Duration::from_millis(120));

    let mut queued = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => {
                queued = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _queued_stream = match queued {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "could not establish queued connection for LASM overflow disallowed-origin test"
            );
        }
    };

    thread::sleep(Duration::from_millis(120));

    let mut response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nOrigin: https://frontend-disallowed.example\r\nConnection: close\r\n\r\n",
                    )
                    .expect("overflow request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("overflow response should be readable");
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
            panic!(
                "run command LASM overflow disallowed-origin test could not connect overflow client"
            );
        }
    };

    assert!(
        response.contains("HTTP/1.1 503 Service Unavailable"),
        "response should contain deterministic service unavailable status:\n{response}"
    );
    assert!(
        response.contains("server busy: max concurrency reached"),
        "response should include deterministic concurrency-limit body:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Origin:"),
        "overflow response should omit allow-origin for disallowed request origin:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Credentials:"),
        "overflow response should omit other cors allow headers for disallowed request origin:\n{response}"
    );
    assert!(
        !response.contains("Vary: Origin"),
        "overflow response should omit vary header when cors defaults are suppressed:\n{response}"
    );

    let _ = child.kill();
    let _ = child.wait();
    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_lasm_backend_honors_max_pending_override_before_overflow() {
    let project_dir = temp_dir("sec4-run-command-lasm-max-pending-override");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runlasmmaxpendingoverridecommand"
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
            "--backend",
            "lasm",
            "--port",
            port_value.as_str(),
            "--max-concurrency",
            "1",
            "--max-pending",
            "2",
            "--serve-timeout-ms",
            "5000",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sec4 run command should start");

    let mut held = None;
    for _ in 0..800 {
        if let Some(status) = child
            .try_wait()
            .expect("run command wait should succeed while connecting")
        {
            panic!("run command exited before opening held connection with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\n")
                    .expect("held partial request should be written");
                held = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _held_stream = match held {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("could not establish held connection for LASM max-pending override test");
        }
    };

    thread::sleep(Duration::from_millis(120));

    let mut queued_a = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => {
                queued_a = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _queued_a_stream = match queued_a {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "could not establish first queued connection for LASM max-pending override test"
            );
        }
    };

    let mut queued_b = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => {
                queued_b = Some(stream);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    let _queued_b_stream = match queued_b {
        Some(stream) => stream,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "could not establish second queued connection for LASM max-pending override test"
            );
        }
    };

    thread::sleep(Duration::from_millis(120));

    let mut response = None;
    for _ in 0..400 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
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
            panic!("run command LASM max-pending override test could not connect overflow client");
        }
    };

    assert!(
        response.contains("HTTP/1.1 503 Service Unavailable"),
        "response should contain deterministic service unavailable status after max-pending override queue is saturated:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-"),
        "response should include deterministic trace header prefix:\n{response}"
    );
    assert!(
        response.contains("server busy: max concurrency reached"),
        "response should include deterministic saturation body:\n{response}"
    );

    let _ = child.kill();
    let _ = child.wait();
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
fn run_command_oneshot_lasm_backend_allows_cors_preflight_with_wildcard_methods_and_headers() {
    if !clang_available() {
        eprintln!("skipping run-command cors wildcard preflight test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-cors-preflight-wildcards");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runcorspreflightwildcardscommand"
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
allowed_methods = ["*"]
allowed_headers = ["*"]
max_age_seconds = 60
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
            "--backend",
            "lasm",
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
                        b"OPTIONS /users HTTP/1.1\r\nHost: localhost\r\nOrigin: https://frontend.example\r\nAccess-Control-Request-Method: DELETE\r\nAccess-Control-Request-Headers: x-custom-token\r\nConnection: close\r\n\r\n",
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
            panic!("run command cors wildcard preflight test could not connect to server");
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
            panic!("run command cors wildcard preflight process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command cors wildcard preflight process should exit successfully"
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
        response.contains("Access-Control-Allow-Methods: *"),
        "response should preserve wildcard allowed-methods header:\n{response}"
    );
    assert!(
        response.contains("Access-Control-Allow-Headers: *"),
        "response should preserve wildcard allowed-headers header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_rejects_invalid_cors_preflight_without_origin() {
    if !clang_available() {
        eprintln!("skipping run-command invalid cors preflight test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-invalid-cors-preflight");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runinvalidcorspreflightcommand"
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
            "--backend",
            "lasm",
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
                        b"OPTIONS /users HTTP/1.1\r\nHost: localhost\r\nAccess-Control-Request-Method: POST\r\nAccess-Control-Request-Headers: x-auth-token\r\nConnection: close\r\n\r\n",
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
            panic!("run command invalid cors preflight test could not connect to server");
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
            panic!("run command invalid cors preflight process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command invalid cors preflight process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain deterministic 400 status line for invalid cors preflight:\n{response}"
    );
    assert!(
        response.contains("cors preflight missing origin"),
        "response should include deterministic invalid preflight message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace id header:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Origin:"),
        "invalid preflight response should not include allow-origin defaults:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Methods:"),
        "invalid preflight response should not include allow-method defaults:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Headers:"),
        "invalid preflight response should not include allow-header defaults:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn run_command_oneshot_lasm_backend_rejects_cors_preflight_method_not_allowed() {
    if !clang_available() {
        eprintln!(
            "skipping run-command cors preflight method-not-allowed test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-run-command-cors-preflight-method-not-allowed");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runcorspreflightmethodnotallowedcommand"
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
            "--backend",
            "lasm",
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
                        b"OPTIONS /users HTTP/1.1\r\nHost: localhost\r\nOrigin: https://frontend.example\r\nAccess-Control-Request-Method: DELETE\r\nAccess-Control-Request-Headers: x-auth-token\r\nConnection: close\r\n\r\n",
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
            panic!(
                "run command cors preflight method-not-allowed test could not connect to server"
            );
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
                "run command cors preflight method-not-allowed process did not exit in expected window"
            );
        }
    };

    assert!(
        status.success(),
        "run command cors preflight method-not-allowed process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 403 Forbidden"),
        "response should contain deterministic 403 status line for disallowed preflight method:\n{response}"
    );
    assert!(
        response.contains("cors preflight method not allowed"),
        "response should include deterministic disallowed-method message:\n{response}"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic trace id header:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Origin:"),
        "disallowed preflight response should not include allow-origin defaults:\n{response}"
    );
    assert!(
        !response.contains("Access-Control-Allow-Methods:"),
        "disallowed preflight response should not include allow-method defaults:\n{response}"
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

#[test]
fn run_command_rejects_zero_max_pending_override() {
    let project_dir = temp_dir("sec4-run-command-zero-max-pending");
    let project_path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();

    let output = run_cli(&["run", "--path", &project_path, "--max-pending", "0"]);
    assert!(
        !output.status.success(),
        "run command should fail for zero --max-pending override"
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "run command should exit with deterministic invalid-flag status"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("run failed: --max-pending must be >= 1"),
        "stderr should include deterministic max-pending validation message:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}
