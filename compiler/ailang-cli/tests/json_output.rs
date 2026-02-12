use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf()
}

fn cli_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ailang"))
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(cli_bin())
        .args(args)
        .output()
        .expect("ailang CLI should run")
}

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    format!("{nanos}")
}

#[test]
fn check_diagnostics_json_success_writes_only_json_on_stdout() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");

    let output = run_cli(&["check", "--path", hello, "--emit", "diagnostics-json"]);
    assert!(output.status.success(), "expected success status");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("stdout should be machine-parseable JSON");
    assert_eq!(parsed, Value::Array(Vec::new()));
    assert!(!stdout.contains("check succeeded"));

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.trim().is_empty(),
        "stderr should stay empty in JSON mode"
    );
}

#[test]
fn check_diagnostics_json_failure_writes_only_json_on_stdout() {
    let missing = workspace_root().join(format!("missing-project-{}", unique_suffix()));
    let missing_path = missing
        .to_str()
        .expect("missing path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "check",
        "--path",
        &missing_path,
        "--emit",
        "diagnostics-json",
    ]);
    assert!(!output.status.success(), "expected failure status");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("stdout should be machine-parseable JSON");
    let diagnostics = parsed
        .as_array()
        .expect("diagnostics-json output should be an array");
    assert!(!diagnostics.is_empty(), "expected at least one diagnostic");
    assert!(!stdout.contains("check succeeded"));

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.trim().is_empty(),
        "stderr should stay empty in JSON mode"
    );
}

#[test]
fn sec_audit_json_keeps_stdout_parseable_json() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");

    let output = run_cli(&["sec", "audit", "--path", hello, "--format", "json"]);
    assert!(output.status.success(), "expected success status");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("stdout should be machine-parseable JSON");
    assert!(
        parsed.get("policy").is_some(),
        "audit report must include policy"
    );
    assert!(
        parsed.get("summary").is_some(),
        "audit report must include summary"
    );
    assert!(!stdout.contains("security map:"));

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("security map:"),
        "security map location should be emitted via stderr"
    );
}
