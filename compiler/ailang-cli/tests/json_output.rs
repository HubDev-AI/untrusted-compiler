use serde_json::Value;
use std::fs;
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

fn clang_available() -> bool {
    Command::new("clang")
        .arg("--version")
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

fn list_json_files(path: &PathBuf) -> Vec<PathBuf> {
    let mut files = fs::read_dir(path)
        .expect("directory should be readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|entry| {
            entry
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    files.sort();
    files
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

#[test]
fn sec_audit_history_dir_writes_reports_and_autoloads_baseline() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");
    let history_dir = temp_dir("ailang-audit-history");
    let history = history_dir
        .to_str()
        .expect("history path should be valid utf-8");

    let first = run_cli(&[
        "sec",
        "audit",
        "--path",
        hello,
        "--format",
        "json",
        "--history-dir",
        history,
    ]);
    assert!(first.status.success(), "first history run should succeed");
    let first_stdout = String::from_utf8(first.stdout).expect("stdout should be utf-8");
    let first_report: Value =
        serde_json::from_str(&first_stdout).expect("stdout should be parseable json");
    assert!(
        first_report.get("trend").is_none(),
        "first history run should not emit trend without prior baseline"
    );
    assert_eq!(list_json_files(&history_dir).len(), 1);

    let second = run_cli(&[
        "sec",
        "audit",
        "--path",
        hello,
        "--format",
        "json",
        "--history-dir",
        history,
    ]);
    assert!(second.status.success(), "second history run should succeed");
    let second_stdout = String::from_utf8(second.stdout).expect("stdout should be utf-8");
    let second_report: Value =
        serde_json::from_str(&second_stdout).expect("stdout should be parseable json");
    assert!(
        second_report.get("trend").is_some(),
        "second history run should emit trend from latest history baseline"
    );
    assert_eq!(list_json_files(&history_dir).len(), 2);

    let second_stderr = String::from_utf8(second.stderr).expect("stderr should be utf-8");
    assert!(
        second_stderr.contains("baseline report:"),
        "history run should report baseline source"
    );
    assert!(
        second_stderr.contains("history report:"),
        "history run should report written history path"
    );
    assert!(
        second_stderr.contains("security map:"),
        "history run should still emit security map path on stderr in JSON mode"
    );

    fs::remove_dir_all(&history_dir).expect("temp history dir cleanup should succeed");
}

#[test]
fn build_emit_mir_prints_textual_mir() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");

    let output = run_cli(&["build", "--path", hello, "--emit", "mir"]);
    assert!(output.status.success(), "expected success status");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("build succeeded"),
        "build output should include success line"
    );
    assert!(
        stdout.contains("fn main() -> Int"),
        "MIR output should include function signature"
    );
    assert!(
        stdout.contains("bb0:"),
        "MIR output should include basic block"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.trim().is_empty(),
        "stderr should stay empty for successful build --emit mir"
    );
}

#[test]
fn build_emit_c_prints_generated_c_source() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");

    let output = run_cli(&["build", "--path", hello, "--emit", "c"]);
    assert!(output.status.success(), "expected success status");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("build succeeded"),
        "build output should include success line"
    );
    assert!(
        stdout.contains("#include <stdint.h>"),
        "C output should include standard integer header"
    );
    assert!(
        stdout.contains("int main(void)"),
        "C output should include generated main signature"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.trim().is_empty(),
        "stderr should stay empty for successful build --emit c"
    );
}

#[test]
fn build_emit_c_bin_compiles_binary_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin integration test: clang not available");
        return;
    }

    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");

    let output = run_cli(&["build", "--path", hello, "--emit", "c-bin"]);
    assert!(output.status.success(), "expected success status");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("compiled binary:"),
        "build output should include compiled binary location"
    );

    let binary_path = hello_path.join("build").join("hello");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "compiled binary should exit successfully"
    );
}

#[test]
fn build_emit_mir_json_writes_only_json_on_stdout() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");

    let output = run_cli(&["build", "--path", hello, "--emit", "mir-json"]);
    assert!(output.status.success(), "expected success status");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("stdout should be machine-parseable JSON");
    assert!(
        parsed
            .get("functions")
            .and_then(|value| value.as_array())
            .is_some_and(|functions| !functions.is_empty()),
        "MIR JSON output should include at least one function"
    );
    assert!(!stdout.contains("build succeeded"));
    assert!(!stdout.contains("wrote lockfile stub"));

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.trim().is_empty(),
        "stderr should stay empty for successful build --emit mir-json"
    );
}
