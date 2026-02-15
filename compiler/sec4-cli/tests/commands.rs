use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn cli_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_sec4"))
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(cli_bin())
        .args(args)
        .output()
        .expect("sec4 CLI should run")
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
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("entry should be written");
}

#[test]
fn test_command_success_path() {
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
        "fn alpha() -> Int {\n  1\n}\n",
    )
    .expect("alpha test should be written");
    fs::write(
        project_dir.join("tests/nested/beta.ut"),
        "fn beta() -> Int {\n  2\n}\n",
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
        stdout.contains("test summary: passed=2, failed=0"),
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
        project_dir.join("tests/good.ut"),
        "fn good() -> Int {\n  1\n}\n",
    )
    .expect("good test should be written");
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
        stdout.contains("test summary: passed=1, failed=1"),
        "test summary should report one failing test:\n{stdout}"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("tests/bad.ut"),
        "diagnostics should reference the invalid test file:\n{stderr}"
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
