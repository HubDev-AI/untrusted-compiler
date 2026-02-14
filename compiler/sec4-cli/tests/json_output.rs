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

fn write_minimal_project(project_dir: &PathBuf, policy_source: &str) {
    fs::create_dir_all(project_dir.join("src")).expect("src dir should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"gate-contract\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("sec4.policy"), policy_source).expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("source should be written");
}

fn write_capture_file(path: &PathBuf, policy_hash: &str, compiler_hash: &str, runtime_hash: &str) {
    let payload = serde_json::json!({
        "version": "0.1",
        "captureId": "cap_01",
        "traceId": "tr_01",
        "timeMs": 1760000000000_i64,
        "policyHash": policy_hash,
        "compilerHash": compiler_hash,
        "runtimeHash": runtime_hash,
        "request": {
            "method": "GET",
            "scheme": "https",
            "host": "example.com",
            "path": "/ping",
            "headers": {},
            "body": {
                "encoding": "none",
                "sha256": "empty",
                "truncated": false
            }
        },
        "determinism": {
            "seed": 1_i64,
            "time": {"mode": "frozen", "nowMs": 1760000000000_i64},
            "uuid": {"mode": "seeded"},
            "budget": {
                "maxBodyBytes": 1_i64,
                "maxJsonBytes": 1_i64,
                "maxJsonDepth": 1_i64,
                "deadlineMs": 1_i64
            }
        },
        "redaction": {"headers": [], "jsonPaths": []}
    });

    let rendered =
        serde_json::to_string_pretty(&payload).expect("capture payload should serialize to json");
    fs::write(path, rendered).expect("capture file should be written");
}

fn write_capture_file_with_db_fs_dependencies(
    path: &PathBuf,
    policy_hash: &str,
    compiler_hash: &str,
    runtime_hash: &str,
    db_query_template_id: &str,
    db_params_sha256: Option<&str>,
    fs_op: &str,
    fs_path_sha256: &str,
) {
    let mut db_request = serde_json::Map::new();
    db_request.insert(
        "queryTemplateId".to_string(),
        Value::String(db_query_template_id.to_string()),
    );
    if let Some(params_sha256) = db_params_sha256 {
        db_request.insert(
            "paramsSha256".to_string(),
            Value::String(params_sha256.to_string()),
        );
    }

    let payload = serde_json::json!({
        "version": "0.1",
        "captureId": "cap_01",
        "traceId": "tr_01",
        "timeMs": 1760000000000_i64,
        "policyHash": policy_hash,
        "compilerHash": compiler_hash,
        "runtimeHash": runtime_hash,
        "request": {
            "method": "GET",
            "scheme": "https",
            "host": "example.com",
            "path": "/ping",
            "headers": {},
            "body": {
                "encoding": "none",
                "sha256": "empty",
                "truncated": false
            }
        },
        "determinism": {
            "seed": 1_i64,
            "time": {"mode": "frozen", "nowMs": 1760000000000_i64},
            "uuid": {"mode": "seeded"},
            "budget": {
                "maxBodyBytes": 1_i64,
                "maxJsonBytes": 1_i64,
                "maxJsonDepth": 1_i64,
                "deadlineMs": 1_i64
            }
        },
        "redaction": {"headers": [], "jsonPaths": []},
        "dependencies": {
            "db": [
                {
                    "request": Value::Object(db_request)
                }
            ],
            "fs": [
                {
                    "request": {
                        "op": fs_op,
                        "pathSha256": fs_path_sha256
                    }
                }
            ]
        }
    });

    let rendered =
        serde_json::to_string_pretty(&payload).expect("capture payload should serialize to json");
    fs::write(path, rendered).expect("capture file should be written");
}

fn write_stub_registry_file(path: &PathBuf) {
    let payload = serde_json::json!({
        "version": "0.1",
        "stubs": {
            "net": [
                {
                    "request": {
                        "method": "GET",
                        "url": "https://example.com/ping",
                        "bodySha256": "empty"
                    },
                    "response": {
                        "status": 200,
                        "bodyBase64": "eyJvayI6dHJ1ZX0=",
                        "truncated": false
                    }
                }
            ],
            "db": [],
            "fs": []
        },
        "redaction": {
            "headers": ["authorization", "cookie", "set-cookie"],
            "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
        }
    });

    let rendered =
        serde_json::to_string_pretty(&payload).expect("stub payload should serialize to json");
    fs::write(path, rendered).expect("stub file should be written");
}

fn write_stub_registry_with_db_fs_file(path: &PathBuf) {
    let payload = serde_json::json!({
        "version": "0.1",
        "stubs": {
            "net": [
                {
                    "request": {
                        "method": "GET",
                        "url": "https://example.com/ping",
                        "bodySha256": "empty"
                    },
                    "response": {
                        "status": 200,
                        "bodyBase64": "eyJvayI6dHJ1ZX0=",
                        "truncated": false
                    }
                }
            ],
            "db": [
                {
                    "request": {
                        "queryTemplateId": "users.by_id",
                        "paramsSha256": "abc123"
                    },
                    "response": {
                        "rowCount": 1,
                        "truncated": false
                    }
                },
                {
                    "request": {
                        "queryTemplateId": "users.by_id"
                    },
                    "response": {
                        "rowCount": 0,
                        "truncated": false
                    }
                },
                {
                    "request": {
                        "queryTemplateId": "users.search"
                    },
                    "response": {
                        "rowCount": 5,
                        "truncated": true
                    }
                }
            ],
            "fs": [
                {
                    "request": {
                        "op": "read",
                        "pathSha256": "p1"
                    },
                    "response": {
                        "ok": true,
                        "bytes": 64,
                        "truncated": false
                    }
                },
                {
                    "request": {
                        "op": "write",
                        "pathSha256": "p2"
                    },
                    "response": {
                        "ok": true,
                        "bytes": 32,
                        "truncated": false
                    }
                },
                {
                    "request": {
                        "op": "delete",
                        "pathSha256": "p3"
                    },
                    "response": {
                        "ok": false,
                        "truncated": false
                    }
                }
            ]
        },
        "redaction": {
            "headers": ["authorization", "cookie", "set-cookie"],
            "jsonPaths": ["$.password", "$.token", "$.secret", "$.apiKey"]
        }
    });

    let rendered = serde_json::to_string_pretty(&payload)
        .expect("stub payload with db/fs should serialize to json");
    fs::write(path, rendered).expect("stub file should be written");
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
fn check_failure_stderr_includes_source_snippet_and_tags() {
    let project_dir = temp_dir("sec4-diagnostic-snippet");
    fs::create_dir_all(project_dir.join("src")).expect("src dir should be created");

    fs::write(
        project_dir.join("sec4.toml"),
        "[package]\nname = \"diag-snippet\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  let name = req.query(12);\n  0\n}\n",
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["check", "--path", &path]);
    assert!(!output.status.success(), "check should fail");

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("tags: security, schema"),
        "stderr should include diagnostic tags:\n{stderr}"
    );
    assert!(
        stderr.contains("2 |   let name = req.query(12)"),
        "stderr should include source snippet line:\n{stderr}"
    );
    assert!(
        stderr.contains("|                        ^"),
        "stderr should include source marker:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn explain_known_security_code_prints_targeted_guidance() {
    let output = run_cli(&["explain", "E1002"]);
    assert!(
        output.status.success(),
        "explain should succeed for known code"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("E1002 - Untrusted Input Reached Typed Sink"),
        "stdout should include exact mapped explain topic:\n{stdout}"
    );
    assert!(
        stdout.contains("sec4 check --emit diagnostics-json"),
        "stdout should include follow-up command guidance:\n{stdout}"
    );
    assert!(
        stdout.contains("docs/book/56-security-diagnostics-taxonomy.md"),
        "stdout should include docs pointer for the code family:\n{stdout}"
    );
}

#[test]
fn explain_exact_capability_code_uses_specific_mapping() {
    let output = run_cli(&["explain", "E2003"]);
    assert!(
        output.status.success(),
        "explain should succeed for exact mapped capability code"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("E2003 - Missing Required Capability"),
        "stdout should include exact mapped topic:\n{stdout}"
    );
    assert!(
        stdout.contains("required capability token in scope"),
        "stdout should include exact mapped summary:\n{stdout}"
    );
    assert!(
        stdout.contains("docs/book/54-v0-stdlib-security-surface.md"),
        "stdout should include exact mapped docs pointer:\n{stdout}"
    );
}

#[test]
fn explain_exact_effect_code_uses_specific_mapping() {
    let output = run_cli(&["explain", "E2001"]);
    assert!(
        output.status.success(),
        "explain should succeed for exact mapped effect code"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("E2001 - Missing Effect Declaration"),
        "stdout should include exact mapped topic:\n{stdout}"
    );
    assert!(
        stdout.contains("uses one or more effects that are not declared"),
        "stdout should include exact mapped summary:\n{stdout}"
    );
    assert!(
        stdout.contains("docs/book/55-v0-typing-effects-security-rules.md"),
        "stdout should include exact mapped docs pointer:\n{stdout}"
    );
}

#[test]
fn explain_exact_schema_call_contract_code_uses_specific_mapping() {
    let output = run_cli(&["explain", "E4001"]);
    assert!(
        output.status.success(),
        "explain should succeed for exact mapped schema contract code"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("E4001 - Invalid Typed API Call Contract"),
        "stdout should include exact mapped topic:\n{stdout}"
    );
    assert!(
        stdout.contains("invalid arity or argument typing"),
        "stdout should include exact mapped summary:\n{stdout}"
    );
    assert!(
        stdout.contains("docs/book/54-v0-stdlib-security-surface.md"),
        "stdout should include exact mapped docs pointer:\n{stdout}"
    );
}

#[test]
fn explain_unknown_code_prints_generic_guidance() {
    let output = run_cli(&["explain", "Z9999"]);
    assert!(
        output.status.success(),
        "explain should stay successful for unknown code"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("Z9999 - Unknown Diagnostic Family"),
        "stdout should identify unknown family:\n{stdout}"
    );
    assert!(
        stdout.contains("docs/05-sec4-master-roadmap.md"),
        "stdout should include generic docs fallback:\n{stdout}"
    );
}

#[test]
fn explain_policy_allow_expired_code_prints_targeted_guidance() {
    let output = run_cli(&["explain", "ALLOW_EXPIRED"]);
    assert!(
        output.status.success(),
        "explain should succeed for policy finding code"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("ALLOW_EXPIRED - Expired Policy Allowlist Exception"),
        "stdout should include exact mapped policy finding topic:\n{stdout}"
    );
    assert!(
        stdout.contains("docs/book/66-deterministic-severity-mapping-for-sec-audit.md"),
        "stdout should include policy finding docs pointer:\n{stdout}"
    );
}

#[test]
fn explain_json_mode_writes_parseable_payload() {
    let output = run_cli(&["explain", "E2001", "--format", "json"]);
    assert!(output.status.success(), "explain json mode should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("explain --format json should output parseable json");

    assert_eq!(
        parsed
            .get("code")
            .and_then(Value::as_str)
            .expect("code should be present"),
        "E2001"
    );
    assert_eq!(
        parsed
            .get("topic")
            .and_then(Value::as_str)
            .expect("topic should be present"),
        "Missing Effect Declaration"
    );
    assert!(
        parsed
            .get("likelyActions")
            .and_then(Value::as_array)
            .map(|values| !values.is_empty())
            .unwrap_or(false),
        "likelyActions should be present and non-empty"
    );
    assert_eq!(
        parsed
            .get("docsPath")
            .and_then(Value::as_str)
            .expect("docsPath should be present"),
        "docs/book/55-v0-typing-effects-security-rules.md"
    );
}

#[test]
fn replay_check_passes_when_capture_hashes_match() {
    let dir = temp_dir("sec4-replay-match");
    let capture = dir.join("capture.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(output.status.success(), "replay check should pass");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("replay capture compatibility check passed"),
        "stdout should confirm replay compatibility pass:\n{stdout}"
    );
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.trim().is_empty(),
        "stderr should be empty:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_on_policy_mismatch_without_allow_flag() {
    let dir = temp_dir("sec4-replay-policy-mismatch");
    let capture = dir.join("capture.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--policy-hash",
        "pol_B",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail without allow-policy-mismatch"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("policyHash mismatch"),
        "stderr should include policy mismatch reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_allows_policy_mismatch_with_allow_flag() {
    let dir = temp_dir("sec4-replay-policy-allow");
    let capture = dir.join("capture.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--policy-hash",
        "pol_B",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
        "--allow-policy-mismatch",
    ]);
    assert!(
        output.status.success(),
        "replay check should pass with allow flag"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("policyHash mismatch allowed"),
        "stderr should include policy mismatch warning:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_still_fails_on_compiler_mismatch_with_allow_flag() {
    let dir = temp_dir("sec4-replay-compiler-mismatch");
    let capture = dir.join("capture.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_B",
        "--runtime-hash",
        "rt_A",
        "--allow-policy-mismatch",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail on compiler mismatch even with allow flag"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("compilerHash mismatch"),
        "stderr should include compiler mismatch reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_with_stub_registry_passes() {
    let dir = temp_dir("sec4-replay-stub-pass");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    write_stub_registry_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        output.status.success(),
        "replay check should pass with valid stub registry"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("replay capture compatibility check passed"),
        "stdout should confirm replay compatibility pass:\n{stdout}"
    );
    assert!(
        stdout.contains("replay stubs loaded: net=1 db=0 fs=0"),
        "stdout should include replay stub inventory summary:\n{stdout}"
    );
    assert!(
        stdout.contains("replay stub details: dbEntries=0 dbTemplates=0 fsEntries=0 fsOps(read=0,write=0,other=0)"),
        "stdout should include replay stub details summary:\n{stdout}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_stub_registry_contract_is_invalid() {
    let dir = temp_dir("sec4-replay-stub-invalid");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    fs::write(
        &stubs,
        r#"{
          "version":"0.1",
          "stubs":{"net":[]}
        }"#,
    )
    .expect("invalid stub payload should be written");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail for invalid stub registry"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("stub registry contract invalid"),
        "stderr should include stub registry contract reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_db_stub_contract_is_invalid() {
    let dir = temp_dir("sec4-replay-stub-db-invalid");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    fs::write(
        &stubs,
        r#"{
          "version":"0.1",
          "stubs":{
            "net":[
              {
                "request":{"method":"GET","url":"https://example.com/ping","bodySha256":"empty"},
                "response":{"status":200,"bodyBase64":"eyJvayI6dHJ1ZX0=","truncated":false}
              }
            ],
            "db":[
              {
                "request":{"paramsSha256":"abc123"},
                "response":{"rowCount":1,"truncated":false}
              }
            ],
            "fs":[]
          },
          "redaction":{"headers":["authorization","cookie","set-cookie"],"jsonPaths":["$.password","$.token","$.secret","$.apiKey"]}
        }"#,
    )
    .expect("stub payload should be written");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail for invalid db stub registry shape"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("request.queryTemplateId"),
        "stderr should include db stub contract reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_fs_stub_contract_is_invalid() {
    let dir = temp_dir("sec4-replay-stub-fs-invalid");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    fs::write(
        &stubs,
        r#"{
          "version":"0.1",
          "stubs":{
            "net":[
              {
                "request":{"method":"GET","url":"https://example.com/ping","bodySha256":"empty"},
                "response":{"status":200,"bodyBase64":"eyJvayI6dHJ1ZX0=","truncated":false}
              }
            ],
            "db":[],
            "fs":[
              {
                "request":{"op":"read"},
                "response":{"ok":true,"truncated":false}
              }
            ]
          },
          "redaction":{"headers":["authorization","cookie","set-cookie"],"jsonPaths":["$.password","$.token","$.secret","$.apiKey"]}
        }"#,
    )
    .expect("stub payload should be written");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail for invalid fs stub registry shape"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("request.pathSha256"),
        "stderr should include fs stub contract reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_db_stub_request_signatures_are_duplicate() {
    let dir = temp_dir("sec4-replay-stub-db-duplicate");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    fs::write(
        &stubs,
        r#"{
          "version":"0.1",
          "stubs":{
            "net":[
              {
                "request":{"method":"GET","url":"https://example.com/ping","bodySha256":"empty"},
                "response":{"status":200,"bodyBase64":"eyJvayI6dHJ1ZX0=","truncated":false}
              }
            ],
            "db":[
              {
                "request":{"queryTemplateId":"users.by_id","paramsSha256":"x"},
                "response":{"rowCount":1,"truncated":false}
              },
              {
                "request":{"queryTemplateId":"users.by_id","paramsSha256":"x"},
                "response":{"rowCount":2,"truncated":false}
              }
            ],
            "fs":[]
          },
          "redaction":{"headers":["authorization","cookie","set-cookie"],"jsonPaths":["$.password","$.token","$.secret","$.apiKey"]}
        }"#,
    )
    .expect("stub payload should be written");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail for duplicate db request signatures"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("stubs.db has duplicate request signature"),
        "stderr should include duplicate db signature reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_fs_stub_request_signatures_are_duplicate() {
    let dir = temp_dir("sec4-replay-stub-fs-duplicate");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    fs::write(
        &stubs,
        r#"{
          "version":"0.1",
          "stubs":{
            "net":[
              {
                "request":{"method":"GET","url":"https://example.com/ping","bodySha256":"empty"},
                "response":{"status":200,"bodyBase64":"eyJvayI6dHJ1ZX0=","truncated":false}
              }
            ],
            "db":[],
            "fs":[
              {
                "request":{"op":"read","pathSha256":"p"},
                "response":{"ok":true,"truncated":false}
              },
              {
                "request":{"op":"READ","pathSha256":"p"},
                "response":{"ok":false,"truncated":false}
              }
            ]
          },
          "redaction":{"headers":["authorization","cookie","set-cookie"],"jsonPaths":["$.password","$.token","$.secret","$.apiKey"]}
        }"#,
    )
    .expect("stub payload should be written");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail for duplicate fs request signatures"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("stubs.fs has duplicate request signature"),
        "stderr should include duplicate fs signature reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_stub_registry_redaction_headers_are_incomplete() {
    let dir = temp_dir("sec4-replay-stub-redaction-headers");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    fs::write(
        &stubs,
        r#"{
          "version":"0.1",
          "stubs":{
            "net":[
              {
                "request":{"method":"GET","url":"https://example.com/ping","bodySha256":"empty"},
                "response":{"status":200,"bodyBase64":"eyJvayI6dHJ1ZX0=","truncated":false}
              }
            ]
          },
          "redaction":{"headers":["authorization","cookie"],"jsonPaths":["$.password","$.token","$.secret","$.apiKey"]}
        }"#,
    )
    .expect("stub payload should be written");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail when required redaction headers are missing"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("must include required header 'set-cookie'"),
        "stderr should include missing required redaction header reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_stub_registry_redaction_json_paths_are_incomplete() {
    let dir = temp_dir("sec4-replay-stub-redaction-json-paths");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    fs::write(
        &stubs,
        r#"{
          "version":"0.1",
          "stubs":{
            "net":[
              {
                "request":{"method":"GET","url":"https://example.com/ping","bodySha256":"empty"},
                "response":{"status":200,"bodyBase64":"eyJvayI6dHJ1ZX0=","truncated":false}
              }
            ]
          },
          "redaction":{"headers":["authorization","cookie","set-cookie"],"jsonPaths":["$.password","$.token"]}
        }"#,
    )
    .expect("stub payload should be written");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail when required redaction json paths are missing"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("must include required path '$.secret'"),
        "stderr should include missing required redaction json path reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_on_duplicate_stub_request_signatures() {
    let dir = temp_dir("sec4-replay-stub-duplicate");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    fs::write(
        &stubs,
        r#"{
          "version":"0.1",
          "stubs":{
            "net":[
              {
                "request":{"method":"GET","url":"https://example.com/ping","bodySha256":"empty"},
                "response":{"status":200,"bodyBase64":"eyJvayI6dHJ1ZX0=","truncated":false}
              },
              {
                "request":{"method":"GET","url":"https://example.com/ping","bodySha256":"empty"},
                "response":{"status":200,"bodySha256":"abc","truncated":false}
              }
            ]
          },
          "redaction":{"headers":["authorization","cookie","set-cookie"],"jsonPaths":["$.password","$.token","$.secret","$.apiKey"]}
        }"#,
    )
    .expect("duplicate stub payload should be written");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail on duplicate stub signatures"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("duplicate request signature"),
        "stderr should include duplicate signature reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_requires_stub_registry() {
    let dir = temp_dir("sec4-replay-mock-requires-stubs");
    let capture = dir.join("capture.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--effects",
        "mock",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail in mock mode without stubs"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("mock effects mode requires --stubs"),
        "stderr should include mock-mode stub requirement:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_fails_when_capture_signature_has_no_matching_stub() {
    let dir = temp_dir("sec4-replay-mock-stub-missing");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    fs::write(
        &stubs,
        r#"{
          "version":"0.1",
          "stubs":{
            "net":[
              {
                "request":{"method":"GET","url":"https://example.com/missing","bodySha256":"empty"},
                "response":{"status":200,"bodyBase64":"eyJvayI6dHJ1ZX0=","truncated":false}
              }
            ],
            "db":[],
            "fs":[]
          },
          "redaction":{"headers":["authorization","cookie","set-cookie"],"jsonPaths":["$.password","$.token","$.secret","$.apiKey"]}
        }"#,
    )
    .expect("stub payload should be written");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail when no stub matches capture signature in mock mode"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("REPLAY.STUB_MISSING"),
        "stderr should include deterministic replay stub-miss code:\n{stderr}"
    );
    assert!(
        stderr.contains("GET|https://example.com/ping|empty"),
        "stderr should include missing request signature:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_fails_when_capture_has_no_url_derivation_fields() {
    let dir = temp_dir("sec4-replay-mock-missing-url");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    fs::write(
        &capture,
        r#"{
          "version":"0.1",
          "captureId":"cap_01",
          "traceId":"tr_01",
          "timeMs":1760000000000,
          "policyHash":"pol_A",
          "compilerHash":"cpl_A",
          "runtimeHash":"rt_A",
          "request":{
            "method":"GET",
            "path":"/ping",
            "headers":{},
            "body":{"encoding":"none","sha256":"empty","truncated":false}
          },
          "determinism":{
            "seed":1,
            "time":{"mode":"frozen","nowMs":1760000000000},
            "uuid":{"mode":"seeded"},
            "budget":{"maxBodyBytes":1,"maxJsonBytes":1,"maxJsonDepth":1,"deadlineMs":1}
          },
          "redaction":{"headers":[],"jsonPaths":[]}
        }"#,
    )
    .expect("capture payload should be written");
    write_stub_registry_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail when mock signature URL cannot be derived"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("capture.request.url must be present or capture.request.scheme/host"),
        "stderr should include url-derivation failure reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_fails_when_capture_db_dependency_signature_has_no_matching_stub() {
    let dir = temp_dir("sec4-replay-mock-db-dependency-stub-missing");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file_with_db_fs_dependencies(
        &capture,
        "pol_A",
        "cpl_A",
        "rt_A",
        "users.missing",
        Some("abc123"),
        "read",
        "p1",
    );
    write_stub_registry_with_db_fs_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail in mock mode when db dependency signature is missing"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("REPLAY.DB_STUB_MISSING"),
        "stderr should include deterministic db-stub miss code:\n{stderr}"
    );
    assert!(
        stderr.contains("users.missing|abc123"),
        "stderr should include missing db dependency signature:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_fails_when_capture_fs_dependency_signature_has_no_matching_stub() {
    let dir = temp_dir("sec4-replay-mock-fs-dependency-stub-missing");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file_with_db_fs_dependencies(
        &capture,
        "pol_A",
        "cpl_A",
        "rt_A",
        "users.by_id",
        Some("abc123"),
        "read",
        "p-missing",
    );
    write_stub_registry_with_db_fs_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail in mock mode when fs dependency signature is missing"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("REPLAY.FS_STUB_MISSING"),
        "stderr should include deterministic fs-stub miss code:\n{stderr}"
    );
    assert!(
        stderr.contains("read|p-missing"),
        "stderr should include missing fs dependency signature:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_fails_when_capture_db_dependency_signatures_are_duplicate() {
    let dir = temp_dir("sec4-replay-mock-db-dependency-duplicate");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    fs::write(
        &capture,
        r#"{
          "version":"0.1",
          "captureId":"cap_01",
          "traceId":"tr_01",
          "timeMs":1760000000000,
          "policyHash":"pol_A",
          "compilerHash":"cpl_A",
          "runtimeHash":"rt_A",
          "request":{
            "method":"GET",
            "scheme":"https",
            "host":"example.com",
            "path":"/ping",
            "headers":{},
            "body":{"encoding":"none","sha256":"empty","truncated":false}
          },
          "determinism":{
            "seed":1,
            "time":{"mode":"frozen","nowMs":1760000000000},
            "uuid":{"mode":"seeded"},
            "budget":{"maxBodyBytes":1,"maxJsonBytes":1,"maxJsonDepth":1,"deadlineMs":1}
          },
          "redaction":{"headers":[],"jsonPaths":[]},
          "dependencies":{
            "db":[
              {"request":{"queryTemplateId":"users.by_id","paramsSha256":"abc123"}},
              {"request":{"queryTemplateId":"users.by_id","paramsSha256":"abc123"}}
            ],
            "fs":[
              {"request":{"op":"read","pathSha256":"p1"}}
            ]
          }
        }"#,
    )
    .expect("capture payload should be written");
    write_stub_registry_with_db_fs_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail in mock mode when capture db dependency signatures are duplicate"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("REPLAY.CAPTURE_DB_DEPENDENCY_DUPLICATE"),
        "stderr should include deterministic duplicate-db-dependency code:\n{stderr}"
    );
    assert!(
        stderr.contains("users.by_id|abc123"),
        "stderr should include duplicate db dependency signature:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_fails_when_capture_fs_dependency_signatures_are_duplicate() {
    let dir = temp_dir("sec4-replay-mock-fs-dependency-duplicate");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    fs::write(
        &capture,
        r#"{
          "version":"0.1",
          "captureId":"cap_01",
          "traceId":"tr_01",
          "timeMs":1760000000000,
          "policyHash":"pol_A",
          "compilerHash":"cpl_A",
          "runtimeHash":"rt_A",
          "request":{
            "method":"GET",
            "scheme":"https",
            "host":"example.com",
            "path":"/ping",
            "headers":{},
            "body":{"encoding":"none","sha256":"empty","truncated":false}
          },
          "determinism":{
            "seed":1,
            "time":{"mode":"frozen","nowMs":1760000000000},
            "uuid":{"mode":"seeded"},
            "budget":{"maxBodyBytes":1,"maxJsonBytes":1,"maxJsonDepth":1,"deadlineMs":1}
          },
          "redaction":{"headers":[],"jsonPaths":[]},
          "dependencies":{
            "db":[
              {"request":{"queryTemplateId":"users.by_id","paramsSha256":"abc123"}}
            ],
            "fs":[
              {"request":{"op":"read","pathSha256":"p1"}},
              {"request":{"op":"READ","pathSha256":"p1"}}
            ]
          }
        }"#,
    )
    .expect("capture payload should be written");
    write_stub_registry_with_db_fs_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail in mock mode when capture fs dependency signatures are duplicate"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("REPLAY.CAPTURE_FS_DEPENDENCY_DUPLICATE"),
        "stderr should include deterministic duplicate-fs-dependency code:\n{stderr}"
    );
    assert!(
        stderr.contains("read|p1"),
        "stderr should include duplicate fs dependency signature:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_deny_mode_fails_when_capture_db_dependency_signatures_are_duplicate() {
    let dir = temp_dir("sec4-replay-deny-db-dependency-duplicate");
    let capture = dir.join("capture.json");
    fs::write(
        &capture,
        r#"{
          "version":"0.1",
          "captureId":"cap_01",
          "traceId":"tr_01",
          "timeMs":1760000000000,
          "policyHash":"pol_A",
          "compilerHash":"cpl_A",
          "runtimeHash":"rt_A",
          "request":{
            "method":"GET",
            "scheme":"https",
            "host":"example.com",
            "path":"/ping",
            "headers":{},
            "body":{"encoding":"none","sha256":"empty","truncated":false}
          },
          "determinism":{
            "seed":1,
            "time":{"mode":"frozen","nowMs":1760000000000},
            "uuid":{"mode":"seeded"},
            "budget":{"maxBodyBytes":1,"maxJsonBytes":1,"maxJsonDepth":1,"deadlineMs":1}
          },
          "redaction":{"headers":[],"jsonPaths":[]},
          "dependencies":{
            "db":[
              {"request":{"queryTemplateId":"users.by_id","paramsSha256":"abc123"}},
              {"request":{"queryTemplateId":"users.by_id","paramsSha256":"abc123"}}
            ]
          }
        }"#,
    )
    .expect("capture payload should be written");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        !output.status.success(),
        "replay check should fail in deny mode when capture db dependency signatures are duplicate"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("REPLAY.CAPTURE_DB_DEPENDENCY_DUPLICATE"),
        "stderr should include deterministic duplicate-db-dependency code:\n{stderr}"
    );
    assert!(
        stderr.contains("users.by_id|abc123"),
        "stderr should include duplicate db dependency signature:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_text_reports_matched_stub_response_summary() {
    let dir = temp_dir("sec4-replay-mock-text-summary");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    write_stub_registry_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        output.status.success(),
        "replay check should pass in mock mode when a matching stub exists"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("replay mock stub matched: GET|https://example.com/ping|empty"),
        "stdout should include matched mock request signature:\n{stdout}"
    );
    assert!(
        stdout.contains("replay mock stub response: status=200 truncated=false bodyKind=base64"),
        "stdout should include matched mock response summary:\n{stdout}"
    );
    assert!(
        stdout.contains("replay mock dependency signatures: db=- fs=-"),
        "stdout should include deterministic empty dependency signature summary:\n{stdout}"
    );
    assert!(
        stdout.contains("replay mock dependency stub summaries: db=- fs=-"),
        "stdout should include deterministic empty dependency stub-summary line:\n{stdout}"
    );
    assert!(
        stdout.contains("replay mock dependency traces: db=- fs=-"),
        "stdout should include deterministic empty dependency trace line:\n{stdout}"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.trim().is_empty(),
        "stderr should be empty for successful mock-mode replay:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_text_reports_dependency_stub_summaries() {
    let dir = temp_dir("sec4-replay-mock-text-dependency-stub-summaries");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file_with_db_fs_dependencies(
        &capture,
        "pol_A",
        "cpl_A",
        "rt_A",
        "users.by_id",
        Some("abc123"),
        "read",
        "p1",
    );
    write_stub_registry_with_db_fs_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        output.status.success(),
        "replay text mode should pass when all dependency signatures are present in stubs"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains(
            "replay mock dependency stub summaries: db=users.by_id|abc123(rowCount=1,truncated=false) fs=read|p1(ok=true,truncated=false,bytes=64)"
        ),
        "stdout should include deterministic dependency stub summary line:\n{stdout}"
    );
    assert!(
        stdout.contains("replay mock dependency traces: db=db:0(users.by_id|abc123) fs=fs:0(read|p1)"),
        "stdout should include deterministic dependency trace line:\n{stdout}"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.trim().is_empty(),
        "stderr should be empty:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_json_reports_dependency_match_counts() {
    let dir = temp_dir("sec4-replay-mock-json-dependency-match-counts");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file_with_db_fs_dependencies(
        &capture,
        "pol_A",
        "cpl_A",
        "rt_A",
        "users.by_id",
        Some("abc123"),
        "read",
        "p1",
    );
    write_stub_registry_with_db_fs_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--format",
        "json",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        output.status.success(),
        "replay json mode should pass when all dependency signatures are present in stubs"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("replay --format json should output parseable json");
    assert_eq!(
        parsed
            .get("mockDependencyMatches")
            .and_then(Value::as_object)
            .and_then(|matches| matches.get("db"))
            .and_then(Value::as_u64)
            .expect("mockDependencyMatches.db should be present"),
        1
    );
    assert_eq!(
        parsed
            .get("mockDependencyMatches")
            .and_then(Value::as_object)
            .and_then(|matches| matches.get("fs"))
            .and_then(Value::as_u64)
            .expect("mockDependencyMatches.fs should be present"),
        1
    );
    assert_eq!(
        parsed
            .get("mockDependencySignatures")
            .and_then(Value::as_object)
            .and_then(|signatures| signatures.get("db"))
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_str)
            .expect("mockDependencySignatures.db[0] should be present"),
        "users.by_id|abc123"
    );
    assert_eq!(
        parsed
            .get("mockDependencySignatures")
            .and_then(Value::as_object)
            .and_then(|signatures| signatures.get("fs"))
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(Value::as_str)
            .expect("mockDependencySignatures.fs[0] should be present"),
        "read|p1"
    );
    assert_eq!(
        parsed
            .get("mockDependencyStubSummaries")
            .and_then(Value::as_object)
            .and_then(|summaries| summaries.get("db"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("mockDependencyStubSummaries.db[0].signature should be present"),
        "users.by_id|abc123"
    );
    assert_eq!(
        parsed
            .get("mockDependencyStubSummaries")
            .and_then(Value::as_object)
            .and_then(|summaries| summaries.get("db"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("rowCount"))
            .and_then(Value::as_i64)
            .expect("mockDependencyStubSummaries.db[0].rowCount should be present"),
        1
    );
    assert_eq!(
        parsed
            .get("mockDependencyStubSummaries")
            .and_then(Value::as_object)
            .and_then(|summaries| summaries.get("db"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("truncated"))
            .and_then(Value::as_bool)
            .expect("mockDependencyStubSummaries.db[0].truncated should be present"),
        false
    );
    assert_eq!(
        parsed
            .get("mockDependencyStubSummaries")
            .and_then(Value::as_object)
            .and_then(|summaries| summaries.get("fs"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("mockDependencyStubSummaries.fs[0].signature should be present"),
        "read|p1"
    );
    assert_eq!(
        parsed
            .get("mockDependencyStubSummaries")
            .and_then(Value::as_object)
            .and_then(|summaries| summaries.get("fs"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("ok"))
            .and_then(Value::as_bool)
            .expect("mockDependencyStubSummaries.fs[0].ok should be present"),
        true
    );
    assert_eq!(
        parsed
            .get("mockDependencyStubSummaries")
            .and_then(Value::as_object)
            .and_then(|summaries| summaries.get("fs"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("truncated"))
            .and_then(Value::as_bool)
            .expect("mockDependencyStubSummaries.fs[0].truncated should be present"),
        false
    );
    assert_eq!(
        parsed
            .get("mockDependencyStubSummaries")
            .and_then(Value::as_object)
            .and_then(|summaries| summaries.get("fs"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("bytes"))
            .and_then(Value::as_i64)
            .expect("mockDependencyStubSummaries.fs[0].bytes should be present"),
        64
    );
    assert_eq!(
        parsed
            .get("mockDependencyTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("db"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("traceId"))
            .and_then(Value::as_str)
            .expect("mockDependencyTraces.db[0].traceId should be present"),
        "db:0"
    );
    assert_eq!(
        parsed
            .get("mockDependencyTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("db"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("mockDependencyTraces.db[0].signature should be present"),
        "users.by_id|abc123"
    );
    assert_eq!(
        parsed
            .get("mockDependencyTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("fs"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("traceId"))
            .and_then(Value::as_str)
            .expect("mockDependencyTraces.fs[0].traceId should be present"),
        "fs:0"
    );
    assert_eq!(
        parsed
            .get("mockDependencyTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("fs"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("mockDependencyTraces.fs[0].signature should be present"),
        "read|p1"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_json_reports_db_fs_stub_details() {
    let dir = temp_dir("sec4-replay-mock-json-dbfs-details");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    write_stub_registry_with_db_fs_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();

    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--format",
        "json",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        output.status.success(),
        "replay json mode should pass for valid db/fs stub details"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("replay --format json should output parseable json");

    assert_eq!(
        parsed
            .get("stubCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("db"))
            .and_then(Value::as_u64)
            .expect("stubCounts.db should be present"),
        3
    );
    assert_eq!(
        parsed
            .get("stubCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("fs"))
            .and_then(Value::as_u64)
            .expect("stubCounts.fs should be present"),
        3
    );
    assert_eq!(
        parsed
            .get("stubDetails")
            .and_then(Value::as_object)
            .and_then(|details| details.get("db"))
            .and_then(Value::as_object)
            .and_then(|db| db.get("entries"))
            .and_then(Value::as_u64)
            .expect("stubDetails.db.entries should be present"),
        3
    );
    assert_eq!(
        parsed
            .get("stubDetails")
            .and_then(Value::as_object)
            .and_then(|details| details.get("db"))
            .and_then(Value::as_object)
            .and_then(|db| db.get("uniqueQueryTemplateIds"))
            .and_then(Value::as_u64)
            .expect("stubDetails.db.uniqueQueryTemplateIds should be present"),
        2
    );
    assert_eq!(
        parsed
            .get("stubDetails")
            .and_then(Value::as_object)
            .and_then(|details| details.get("fs"))
            .and_then(Value::as_object)
            .and_then(|fs| fs.get("readOps"))
            .and_then(Value::as_u64)
            .expect("stubDetails.fs.readOps should be present"),
        1
    );
    assert_eq!(
        parsed
            .get("stubDetails")
            .and_then(Value::as_object)
            .and_then(|details| details.get("fs"))
            .and_then(Value::as_object)
            .and_then(|fs| fs.get("writeOps"))
            .and_then(Value::as_u64)
            .expect("stubDetails.fs.writeOps should be present"),
        1
    );
    assert_eq!(
        parsed
            .get("stubDetails")
            .and_then(Value::as_object)
            .and_then(|details| details.get("fs"))
            .and_then(Value::as_object)
            .and_then(|fs| fs.get("otherOps"))
            .and_then(Value::as_u64)
            .expect("stubDetails.fs.otherOps should be present"),
        1
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_allow_mode_emits_warning() {
    let dir = temp_dir("sec4-replay-allow-warning");
    let capture = dir.join("capture.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--effects",
        "allow",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(
        output.status.success(),
        "replay check should pass in allow mode for contract checks"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("replay effects mode is allow"),
        "stderr should include allow-mode warning:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_json_mode_writes_parseable_payload() {
    let dir = temp_dir("sec4-replay-json-mode");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file(&capture, "pol_A", "cpl_A", "rt_A");
    write_stub_registry_file(&stubs);

    let capture_path = capture
        .to_str()
        .expect("capture path should be valid utf-8")
        .to_string();
    let stubs_path = stubs
        .to_str()
        .expect("stubs path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "replay",
        "--capture",
        &capture_path,
        "--stubs",
        &stubs_path,
        "--effects",
        "mock",
        "--format",
        "json",
        "--policy-hash",
        "pol_A",
        "--compiler-hash",
        "cpl_A",
        "--runtime-hash",
        "rt_A",
    ]);
    assert!(output.status.success(), "replay json mode should pass");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("replay --format json should output parseable json");
    assert_eq!(
        parsed
            .get("ok")
            .and_then(Value::as_bool)
            .expect("ok should be present"),
        true
    );
    assert_eq!(
        parsed
            .get("effectsMode")
            .and_then(Value::as_str)
            .expect("effectsMode should be present"),
        "mock"
    );
    assert_eq!(
        parsed
            .get("policyHashMatched")
            .and_then(Value::as_bool)
            .expect("policyHashMatched should be present"),
        true
    );
    assert!(
        parsed
            .get("warnings")
            .and_then(Value::as_array)
            .is_some_and(|warnings| warnings.is_empty()),
        "warnings should be present and empty for clean mock run"
    );
    assert_eq!(
        parsed
            .get("stubCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("net"))
            .and_then(Value::as_u64)
            .expect("stubCounts.net should be present"),
        1
    );
    assert_eq!(
        parsed
            .get("stubCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("db"))
            .and_then(Value::as_u64)
            .expect("stubCounts.db should be present"),
        0
    );
    assert_eq!(
        parsed
            .get("stubCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("fs"))
            .and_then(Value::as_u64)
            .expect("stubCounts.fs should be present"),
        0
    );
    assert_eq!(
        parsed
            .get("stubDetails")
            .and_then(Value::as_object)
            .and_then(|details| details.get("db"))
            .and_then(Value::as_object)
            .and_then(|db| db.get("entries"))
            .and_then(Value::as_u64)
            .expect("stubDetails.db.entries should be present"),
        0
    );
    assert_eq!(
        parsed
            .get("stubDetails")
            .and_then(Value::as_object)
            .and_then(|details| details.get("fs"))
            .and_then(Value::as_object)
            .and_then(|fs| fs.get("entries"))
            .and_then(Value::as_u64)
            .expect("stubDetails.fs.entries should be present"),
        0
    );
    assert_eq!(
        parsed
            .get("mockRequestSignature")
            .and_then(Value::as_str)
            .expect("mockRequestSignature should be present"),
        "GET|https://example.com/ping|empty"
    );
    assert_eq!(
        parsed
            .get("mockMatchedStub")
            .and_then(Value::as_object)
            .and_then(|stub| stub.get("status"))
            .and_then(Value::as_i64)
            .expect("mockMatchedStub.status should be present"),
        200
    );
    assert_eq!(
        parsed
            .get("mockMatchedStub")
            .and_then(Value::as_object)
            .and_then(|stub| stub.get("truncated"))
            .and_then(Value::as_bool)
            .expect("mockMatchedStub.truncated should be present"),
        false
    );
    assert_eq!(
        parsed
            .get("mockMatchedStub")
            .and_then(Value::as_object)
            .and_then(|stub| stub.get("bodyKind"))
            .and_then(Value::as_str)
            .expect("mockMatchedStub.bodyKind should be present"),
        "base64"
    );
    assert_eq!(
        parsed
            .get("mockDependencyMatches")
            .and_then(Value::as_object)
            .and_then(|matches| matches.get("db"))
            .and_then(Value::as_u64)
            .expect("mockDependencyMatches.db should be present"),
        0
    );
    assert!(
        parsed
            .get("mockDependencySignatures")
            .and_then(Value::as_object)
            .and_then(|signatures| signatures.get("db"))
            .and_then(Value::as_array)
            .is_some_and(|items| items.is_empty()),
        "mockDependencySignatures.db should be present and empty when capture has no dependencies"
    );
    assert!(
        parsed
            .get("mockDependencySignatures")
            .and_then(Value::as_object)
            .and_then(|signatures| signatures.get("fs"))
            .and_then(Value::as_array)
            .is_some_and(|items| items.is_empty()),
        "mockDependencySignatures.fs should be present and empty when capture has no dependencies"
    );
    assert!(
        parsed
            .get("mockDependencyStubSummaries")
            .and_then(Value::as_object)
            .and_then(|summaries| summaries.get("db"))
            .and_then(Value::as_array)
            .is_some_and(|entries| entries.is_empty()),
        "mockDependencyStubSummaries.db should be present and empty when capture has no dependencies"
    );
    assert!(
        parsed
            .get("mockDependencyStubSummaries")
            .and_then(Value::as_object)
            .and_then(|summaries| summaries.get("fs"))
            .and_then(Value::as_array)
            .is_some_and(|entries| entries.is_empty()),
        "mockDependencyStubSummaries.fs should be present and empty when capture has no dependencies"
    );
    assert!(
        parsed
            .get("mockDependencyTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("db"))
            .and_then(Value::as_array)
            .is_some_and(|entries| entries.is_empty()),
        "mockDependencyTraces.db should be present and empty when capture has no dependencies"
    );
    assert!(
        parsed
            .get("mockDependencyTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("fs"))
            .and_then(Value::as_array)
            .is_some_and(|entries| entries.is_empty()),
        "mockDependencyTraces.fs should be present and empty when capture has no dependencies"
    );
    assert_eq!(
        parsed
            .get("mockDependencyMatches")
            .and_then(Value::as_object)
            .and_then(|matches| matches.get("fs"))
            .and_then(Value::as_u64)
            .expect("mockDependencyMatches.fs should be present"),
        0
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.trim().is_empty(),
        "stderr should be empty:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn explain_cors_wildcard_finding_prints_targeted_guidance() {
    let output = run_cli(&["explain", "CORS_CREDENTIALS_WITH_WILDCARD"]);
    assert!(
        output.status.success(),
        "explain should succeed for cors wildcard finding id"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("CORS_CREDENTIALS_WITH_WILDCARD - CORS Credentials With Wildcard Origin"),
        "stdout should include mapped finding topic:\n{stdout}"
    );
    assert!(
        stdout.contains("docs/book/66-deterministic-severity-mapping-for-sec-audit.md"),
        "stdout should include mapped finding docs pointer:\n{stdout}"
    );
}

#[test]
fn explain_allow_count_high_finding_in_json_mode_is_parseable() {
    let output = run_cli(&["explain", "ALLOW_COUNT_HIGH", "--format", "json"]);
    assert!(
        output.status.success(),
        "explain json mode should succeed for finding id"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value = serde_json::from_str(&stdout).expect("output should be parseable json");

    assert_eq!(
        parsed
            .get("code")
            .and_then(Value::as_str)
            .expect("code should be present"),
        "ALLOW_COUNT_HIGH"
    );
    assert_eq!(
        parsed
            .get("topic")
            .and_then(Value::as_str)
            .expect("topic should be present"),
        "Allowlist Exception Count High"
    );
}

#[test]
fn explain_all_current_audit_finding_ids_in_json_mode_have_exact_mappings() {
    let codes = [
        "CORS_CREDENTIALS_WITH_WILDCARD",
        "CORS_ANY_ORIGIN",
        "CORS_REFLECT_ORIGIN_ENABLED",
        "CORS_VARY_ORIGIN_MISSING",
        "CSP_DISABLED",
        "CSP_REPORT_ONLY",
        "HSTS_DISABLED_IN_PROD",
        "REFERRER_POLICY_WEAK",
        "NOSNIFF_DISABLED",
        "XFO_DISABLED",
        "CSRF_REQUIRED_BUT_DISABLED",
        "CSRF_PROTECTED_METHODS_INCOMPLETE",
        "COOKIE_CROSS_SITE_WITHOUT_CORS_CREDS",
        "COOKIE_CROSS_SITE_WITH_WILDCARD_ORIGIN",
        "COOKIE_SAMESITE_NONE_WITHOUT_SECURE",
        "INTERNAL_NET_ENABLED_NO_ALLOWLIST",
        "INTERNAL_NET_CALL_ALLOWLISTED",
        "PUBLIC_REDIRECTS_ENABLED_WITHOUT_REVALIDATION",
        "DNS_RESOLUTION_DISABLED",
        "PUBLIC_EGRESS_NO_DOMAIN_POLICY",
        "FS_ENABLED_NO_BASE_ALLOWLIST",
        "SYMLINK_POLICY_WEAK",
        "CAPTURE_REDACTION_INCOMPLETE",
        "CAPTURE_ALL_IN_PROD",
        "REPLAY_EFFECTS_ALLOW",
        "LOG_STRUCTURED_ONLY_DISABLED",
        "LOG_REMOTE_IP_ENABLED",
        "LOG_USER_AGENT_ENABLED",
        "SQL_RAW_ALLOWED_BY_POLICY",
        "SQL_LIMIT_RULE_DISABLED",
        "SQL_SELECT_WITHOUT_LIMIT",
        "SECRETS_REVEAL_USED",
        "SECRETS_REVEAL_ALLOWLISTED",
        "ALLOW_EXPIRED",
        "ALLOW_EXPIRING_SOON",
        "ALLOW_EXPIRY_WINDOW_ROLLUP",
        "ALLOW_COUNT_HIGH",
    ];

    for code in codes {
        let output = run_cli(&["explain", code, "--format", "json"]);
        assert!(
            output.status.success(),
            "explain json mode should succeed for finding id {code}"
        );

        let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
        let parsed: Value = serde_json::from_str(&stdout).expect("output should be parseable json");

        assert_eq!(
            parsed
                .get("code")
                .and_then(Value::as_str)
                .expect("code should be present"),
            code
        );
        assert_ne!(
            parsed
                .get("topic")
                .and_then(Value::as_str)
                .expect("topic should be present"),
            "Unknown Diagnostic Family",
            "finding id should have an exact explain mapping: {code}"
        );
        assert_ne!(
            parsed
                .get("docsPath")
                .and_then(Value::as_str)
                .expect("docsPath should be present"),
            "docs/05-sec4-master-roadmap.md",
            "finding id should not route to generic roadmap fallback: {code}"
        );
    }
}

#[test]
fn sec_audit_json_keeps_stdout_parseable_json() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");

    let output = run_cli(&["audit", "--path", hello, "--format", "json"]);
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
fn sec_gate_defaults_to_risk_high_threshold() {
    let project_dir = temp_dir("sec4-gate-default-threshold");
    write_minimal_project(
        &project_dir,
        r#"[security_headers.csp]
enabled = false
report_only = false
"#,
    );

    let path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["gate", "--path", &path]);
    assert!(
        !output.status.success(),
        "gate should fail by default when HIGH findings are present"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("security audit failed: findings at or above threshold HIGH"),
        "gate should report default HIGH threshold failure:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn sec_gate_json_mode_allows_high_when_threshold_is_critical() {
    let project_dir = temp_dir("sec4-gate-custom-threshold");
    write_minimal_project(
        &project_dir,
        r#"[security_headers.csp]
enabled = false
report_only = false
"#,
    );

    let path = project_dir
        .to_str()
        .expect("temp project path should be valid utf-8")
        .to_string();
    let output = run_cli(&[
        "gate",
        "--path",
        &path,
        "--format",
        "json",
        "--fail-on",
        "risk>=CRITICAL",
    ]);
    assert!(
        output.status.success(),
        "gate should pass when only HIGH findings are present and threshold is CRITICAL"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("gate --format json should emit parseable JSON");
    let has_csp_disabled = parsed
        .get("findings")
        .and_then(Value::as_array)
        .map(|findings| {
            findings
                .iter()
                .any(|finding| finding.get("id").and_then(Value::as_str) == Some("CSP_DISABLED"))
        })
        .unwrap_or(false);
    assert!(
        has_csp_disabled,
        "expected CSP_DISABLED finding in gate JSON output"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("security map:"),
        "gate json mode should keep auxiliary lines on stderr"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn sec_audit_history_dir_writes_reports_and_autoloads_baseline() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");
    let history_dir = temp_dir("sec4-audit-history");
    let history = history_dir
        .to_str()
        .expect("history path should be valid utf-8");

    let first = run_cli(&[
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
fn sec_audit_history_window_summary_is_emitted_on_stderr_in_json_mode() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");
    let history_dir = temp_dir("sec4-audit-history-window");
    let history = history_dir
        .to_str()
        .expect("history path should be valid utf-8");

    let first = run_cli(&[
        "audit",
        "--path",
        hello,
        "--format",
        "json",
        "--history-dir",
        history,
    ]);
    assert!(first.status.success(), "first history run should succeed");

    let second = run_cli(&[
        "audit",
        "--path",
        hello,
        "--format",
        "json",
        "--history-dir",
        history,
        "--history-window",
        "2",
    ]);
    assert!(second.status.success(), "second history run should succeed");

    let second_stdout = String::from_utf8(second.stdout).expect("stdout should be utf-8");
    let second_report: Value =
        serde_json::from_str(&second_stdout).expect("stdout should be parseable json");
    assert!(
        second_report.get("summary").is_some(),
        "audit report should still be emitted on stdout"
    );
    let history_window = second_report
        .get("historyWindow")
        .expect("history-window run should include historyWindow in report");
    assert_eq!(
        history_window
            .get("window")
            .and_then(Value::as_i64)
            .expect("historyWindow.window should be present"),
        2
    );
    assert_eq!(
        history_window
            .get("reports")
            .and_then(Value::as_i64)
            .expect("historyWindow.reports should be present"),
        2
    );

    let second_stderr = String::from_utf8(second.stderr).expect("stderr should be utf-8");
    assert!(
        second_stderr.contains("history window summary:"),
        "history-window run should include summary line"
    );
    assert!(
        second_stderr.contains("\"window\":2"),
        "history-window summary should include requested window size"
    );
    assert!(
        second_stderr.contains("\"reports\":2"),
        "history-window summary should include number of sampled reports"
    );
    assert!(
        second_stderr.contains("\"severityRollup\""),
        "history-window summary should include severity rollups"
    );
    assert!(
        second_stderr.contains("\"severityLatestDelta\""),
        "history-window summary should include latest-vs-oldest severity deltas"
    );

    fs::remove_dir_all(&history_dir).expect("temp history dir cleanup should succeed");
}

#[test]
fn sec_audit_history_window_summary_is_emitted_on_stdout_in_text_mode() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");
    let history_dir = temp_dir("sec4-audit-history-window-text");
    let history = history_dir
        .to_str()
        .expect("history path should be valid utf-8");

    let first = run_cli(&["audit", "--path", hello, "--history-dir", history]);
    assert!(first.status.success(), "first history run should succeed");

    let second = run_cli(&[
        "audit",
        "--path",
        hello,
        "--history-dir",
        history,
        "--history-window",
        "2",
    ]);
    assert!(second.status.success(), "second history run should succeed");

    let second_stdout = String::from_utf8(second.stdout).expect("stdout should be utf-8");
    assert!(
        second_stdout.contains("history window summary:"),
        "text-mode history-window run should include summary on stdout"
    );

    let second_stderr = String::from_utf8(second.stderr).expect("stderr should be utf-8");
    assert!(
        second_stderr.trim().is_empty(),
        "text-mode history-window run should keep stderr empty"
    );

    fs::remove_dir_all(&history_dir).expect("temp history dir cleanup should succeed");
}

#[test]
fn sec_audit_history_window_requires_history_dir() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");

    let output = run_cli(&["audit", "--path", hello, "--history-window", "2"]);
    assert!(
        !output.status.success(),
        "history-window without history-dir should fail"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("--history-window requires --history-dir"),
        "expected explicit usage error for missing history-dir"
    );
}

#[test]
fn sec_audit_history_window_zero_is_rejected() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");
    let history_dir = temp_dir("sec4-audit-history-zero-window");
    let history = history_dir
        .to_str()
        .expect("history path should be valid utf-8");

    let output = run_cli(&[
        "audit",
        "--path",
        hello,
        "--history-dir",
        history,
        "--history-window",
        "0",
    ]);
    assert!(
        !output.status.success(),
        "history-window=0 should fail with explicit usage error"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("--history-window must be >= 1"),
        "expected explicit error for zero history-window"
    );

    fs::remove_dir_all(&history_dir).expect("temp history dir cleanup should succeed");
}

#[test]
fn sec_audit_write_history_summary_requires_history_window() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");
    let summary_path = temp_dir("sec4-history-summary-missing-window").join("summary.json");
    let summary = summary_path
        .to_str()
        .expect("summary path should be valid utf-8");

    let output = run_cli(&["audit", "--path", hello, "--write-history-summary", summary]);
    assert!(
        !output.status.success(),
        "write-history-summary without history-window should fail"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("--write-history-summary requires --history-window"),
        "expected explicit usage error for missing history-window"
    );
}

#[test]
fn sec_audit_history_window_summary_can_be_written_to_file() {
    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");
    let history_dir = temp_dir("sec4-audit-history-window-write");
    let history = history_dir
        .to_str()
        .expect("history path should be valid utf-8");
    let summary_path = history_dir.join("summary").join("window.json");
    let summary = summary_path
        .to_str()
        .expect("summary path should be valid utf-8");

    let first = run_cli(&[
        "audit",
        "--path",
        hello,
        "--format",
        "json",
        "--history-dir",
        history,
    ]);
    assert!(first.status.success(), "first history run should succeed");

    let second = run_cli(&[
        "audit",
        "--path",
        hello,
        "--format",
        "json",
        "--history-dir",
        history,
        "--history-window",
        "2",
        "--write-history-summary",
        summary,
    ]);
    assert!(second.status.success(), "second history run should succeed");
    assert!(
        summary_path.exists(),
        "history summary file should be written when requested"
    );

    let summary_raw =
        fs::read_to_string(&summary_path).expect("history summary file should be readable");
    let summary_json: Value =
        serde_json::from_str(&summary_raw).expect("history summary should be parseable json");
    assert_eq!(
        summary_json
            .get("window")
            .and_then(Value::as_i64)
            .expect("window should be present"),
        2
    );
    assert_eq!(
        summary_json
            .get("reports")
            .and_then(Value::as_i64)
            .expect("reports should be present"),
        2
    );
    assert!(
        summary_json.get("severityRollup").is_some(),
        "history summary should include severity rollups"
    );
    assert!(
        summary_json.get("severityLatestDelta").is_some(),
        "history summary should include severity latest delta"
    );
    assert!(
        summary_json.get("oldestTimeMs").is_some() && summary_json.get("latestTimeMs").is_some(),
        "history summary should include oldest/latest report timestamps"
    );
    assert!(
        summary_json.get("oldestPolicyHash").is_some()
            && summary_json.get("latestPolicyHash").is_some(),
        "history summary should include oldest/latest policy hashes"
    );

    let second_stderr = String::from_utf8(second.stderr).expect("stderr should be utf-8");
    assert!(
        second_stderr.contains("history summary:"),
        "history-summary write should report output path"
    );

    fs::remove_dir_all(&history_dir).expect("temp history dir cleanup should succeed");
}

#[test]
fn build_locked_fails_when_lockfile_missing() {
    let project_dir = temp_dir("sec4-build-locked-missing-lock");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "locked_missing"
version = "0.1.0"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("source should be written");

    let project = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let output = run_cli(&["build", "--path", &project, "--locked"]);
    assert!(!output.status.success(), "locked build should fail");

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("[M0202]"),
        "stderr should contain lockfile-missing code:\n{stderr}"
    );
    assert!(
        stderr.contains("sec4.lock is required in --locked mode"),
        "stderr should explain missing lockfile:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn build_locked_succeeds_with_matching_lockfile() {
    let project_dir = temp_dir("sec4-build-locked-valid-lock");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "locked_valid"
version = "0.1.0"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("source should be written");

    let project = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let initial = run_cli(&["build", "--path", &project]);
    assert!(initial.status.success(), "initial build should succeed");

    let locked = run_cli(&["build", "--path", &project, "--locked"]);
    assert!(locked.status.success(), "locked build should succeed");
    let stdout = String::from_utf8(locked.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("verified lockfile:"),
        "locked build should report lockfile verification:\n{stdout}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn build_locked_fails_when_lockfile_is_stale() {
    let project_dir = temp_dir("sec4-build-locked-stale-lock");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    let manifest_path = project_dir.join("sec4.toml");
    fs::write(
        &manifest_path,
        r#"[package]
name = "locked_stale"
version = "0.1.0"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("source should be written");

    let project = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let initial = run_cli(&["build", "--path", &project]);
    assert!(initial.status.success(), "initial build should succeed");

    fs::write(
        &manifest_path,
        r#"[package]
name = "locked_stale"
version = "0.2.0"
"#,
    )
    .expect("manifest update should be written");

    let locked = run_cli(&["build", "--path", &project, "--locked"]);
    assert!(
        !locked.status.success(),
        "locked build should fail on stale lock"
    );
    let stderr = String::from_utf8(locked.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("[M0203]"),
        "stale lock should emit M0203:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn build_writes_deterministic_build_metadata_file() {
    let project_dir = temp_dir("sec4-build-metadata");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "metadata_demo"
version = "0.1.0"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("source should be written");

    let project = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let first = run_cli(&["build", "--path", &project]);
    assert!(first.status.success(), "first build should succeed");
    let first_stdout = String::from_utf8(first.stdout).expect("stdout should be utf-8");
    assert!(
        first_stdout.contains("wrote build metadata:"),
        "build output should include metadata path:\n{first_stdout}"
    );

    let metadata_path = project_dir.join("build").join("build_metadata.json");
    assert!(metadata_path.exists(), "build metadata file should exist");
    let first_metadata =
        fs::read_to_string(&metadata_path).expect("first metadata should be readable");

    let second = run_cli(&["build", "--path", &project]);
    assert!(second.status.success(), "second build should succeed");
    let second_metadata =
        fs::read_to_string(&metadata_path).expect("second metadata should be readable");

    assert_eq!(
        first_metadata, second_metadata,
        "build metadata must be deterministic for identical inputs"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn build_with_sbom_writes_deterministic_sbom_file() {
    let project_dir = temp_dir("sec4-build-sbom");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "sbom_demo"
version = "0.1.0"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("source should be written");

    let project = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();
    let first = run_cli(&["build", "--path", &project, "--sbom"]);
    assert!(first.status.success(), "first build --sbom should succeed");
    let first_stdout = String::from_utf8(first.stdout).expect("stdout should be utf-8");
    assert!(
        first_stdout.contains("wrote sbom:"),
        "build --sbom output should include sbom path:\n{first_stdout}"
    );

    let sbom_path = project_dir.join("build").join("sbom.json");
    assert!(sbom_path.exists(), "sbom file should exist");
    let first_sbom = fs::read_to_string(&sbom_path).expect("first sbom should be readable");

    let second = run_cli(&["build", "--path", &project, "--sbom"]);
    assert!(
        second.status.success(),
        "second build --sbom should succeed"
    );
    let second_sbom = fs::read_to_string(&sbom_path).expect("second sbom should be readable");
    assert_eq!(
        first_sbom, second_sbom,
        "sbom output should be deterministic for identical inputs"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
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
    assert!(
        hello_path.join("build").join("sec4_runtime.h").exists(),
        "runtime header should exist"
    );
    assert!(
        hello_path.join("build").join("sec4_runtime.c").exists(),
        "runtime source should exist"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "compiled binary should exit successfully"
    );
}

#[test]
fn build_emit_c_bin_handles_calls_and_control_flow_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin control-flow integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-flow");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "flowdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn choose(flag: Bool) -> Int {
  if flag {
    0
  } else {
    1
  }
}

fn main() -> Int {
  choose(true)
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for control-flow + call project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("int64_t choose(bool flag);"));
    assert!(generated_c.contains("if (flag) goto"));
    assert!(generated_c.contains("return sec4_rt_identity_i64(choose(true));"));

    let binary_path = project_dir.join("build").join("flowdemo");
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
fn build_emit_c_bin_handles_time_now_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-time-now");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "timedemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { time.now } -> Int64 {
  time.now()
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for time.now intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_time_now()"));

    let binary_path = project_dir.join("build").join("timedemo");
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
fn build_emit_c_bin_handles_log_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin log intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-log");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "logdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { log } -> Int {
  log.info(log.event("event"));
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for log intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_log_any(sec4_rt_log_event(\"event\"))"));

    let binary_path = project_dir.join("build").join("logdemo");
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
fn build_emit_c_bin_handles_log_builder_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin log builder integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-log-builders");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "logbuildersdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() -> Int {
  let event = log.event("user.created");
  let num = log.i64(1);
  let field = log.field("count", num);
  let obj = log.obj(field);
  let text = log.str("ok");
  let flag = log.bool(true);
  let secret = log.redacted("secret");
  let attrSecret = log.attrRedacted("token");
  let withAttr = log.withAttr(event, "token", attrSecret);
  let withHttp = log.withHttp(withAttr, "POST", "/users", 200, 42);
  let error = err.internal("boom");
  let withError = log.withError(withHttp, error);
  event;
  field;
  obj;
  text;
  num;
  flag;
  secret;
  attrSecret;
  withAttr;
  withHttp;
  error;
  withError;
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for log builder intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_log_event(\"user.created\")"));
    assert!(generated_c.contains("sec4_rt_log_field(\"count\", num)"));
    assert!(generated_c.contains("sec4_rt_log_obj(field)"));
    assert!(generated_c.contains("sec4_rt_log_str(\"ok\")"));
    assert!(generated_c.contains("sec4_rt_log_i64(1)"));
    assert!(generated_c.contains("sec4_rt_log_bool(true)"));
    assert!(generated_c.contains("sec4_rt_log_redacted(\"secret\")"));
    assert!(generated_c.contains("sec4_rt_log_attr_redacted(\"token\")"));
    assert!(generated_c.contains("sec4_rt_log_with_attr(event, \"token\", attrSecret)"));
    assert!(generated_c.contains("sec4_rt_log_with_http(withAttr, \"POST\", \"/users\", 200, 42)"));
    assert!(generated_c.contains("sec4_rt_err_internal(\"boom\")"));
    assert!(generated_c.contains("sec4_rt_log_with_error(withHttp, error)"));

    let binary_path = project_dir.join("build").join("logbuildersdemo");
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
fn build_emit_c_bin_handles_req_res_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin req/res intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-req-res");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "reqresdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn decode(ctx: Ctx, req: Request, schema: Schema<Int>) effects { net } -> Int {
  req.body(ctx, req);
  req.query("q");
  req.pathParam("id");
  req.header("authorization");
  req.json(schema);
  0
}

fn encode(schema: Schema<Int>) effects { net } -> Int {
  res.json(schema, 1);
  res.ok(201, schema, 1);
  res.okMeta(201, schema, 1, 2);
  let raw = req.query("html");
  let safe = sanitize.html(raw);
  res.html(safe);
  res.text(200, "ok");
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for req/res intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_req_body(ctx, req)"));
    assert!(generated_c.contains("sec4_rt_req_query(\"q\")"));
    assert!(generated_c.contains("sec4_rt_req_path_param(\"id\")"));
    assert!(generated_c.contains("sec4_rt_req_header(\"authorization\")"));
    assert!(generated_c.contains("sec4_rt_req_json(schema)"));
    assert!(generated_c.contains("sec4_rt_res_json(schema, 1)"));
    assert!(generated_c.contains("sec4_rt_res_ok(201, schema, 1)"));
    assert!(generated_c.contains("sec4_rt_res_ok_meta(201, schema, 1, 2)"));
    assert!(generated_c.contains("sec4_rt_req_query(\"html\")"));
    assert!(generated_c.contains("sec4_rt_sanitize_html(raw)"));
    assert!(generated_c.contains("sec4_rt_res_html(safe)"));
    assert!(generated_c.contains("sec4_rt_res_text(200, \"ok\")"));

    let binary_path = project_dir.join("build").join("reqresdemo");
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
fn build_emit_c_bin_handles_json_helper_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin json helper integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-json-helpers");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "jsonhelpersdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn useJson(ctx: Ctx, schema: Schema<Int>, raw: Untrusted<Bytes>) -> Int {
  json.decode(ctx, schema, raw);
  json.encode(schema, 3);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for json helper intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_json_decode(ctx, schema, raw)"));
    assert!(generated_c.contains("sec4_rt_json_encode(schema, 3)"));

    let binary_path = project_dir.join("build").join("jsonhelpersdemo");
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
fn build_emit_c_bin_handles_header_cookie_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin header/cookie intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-header-cookie");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "headercookiedemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn configure() effects { net } -> Int {
  let name = headers.name("X-Test");
  let value = headers.value("ok");
  let cookie = cookie.build("session", "token");
  res.setHeader(name, value);
  res.addCookie(cookie);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for header/cookie intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_headers_name(\"X-Test\")"));
    assert!(generated_c.contains("sec4_rt_headers_value(\"ok\")"));
    assert!(generated_c.contains("sec4_rt_cookie_build(\"session\", \"token\")"));
    assert!(generated_c.contains("sec4_rt_set_header(name, value)"));
    assert!(generated_c.contains("sec4_rt_set_cookie(cookie)"));

    let binary_path = project_dir.join("build").join("headercookiedemo");
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
fn build_emit_c_bin_handles_db_fs_net_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin db/fs/net intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-db-fs-net");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "dbfsnetdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn ioOps(
  db: DbCap,
  tx: TxCap,
  fs: FsCap,
  net: NetCap,
  query: SqlQuery,
  row: Schema<Int>,
  path: PathSafe,
  url: PublicUrl
) effects { db.write, db.read, db.tx, fs.read, fs.write, net } -> Int {
  let built = sql.q("SELECT 1", 2);
  db.tx(db);
  db.execTx(tx, built);
  db.exec(db, built);
  db.queryOne(db, built, row);
  query;
  fs.read(fs, path);
  fs.write(fs, path, 1);
  httpClient.get(net, url);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for db/fs/net intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_sql_q(\"SELECT 1\", 2)"));
    assert!(generated_c.contains("sec4_rt_db_tx(db)"));
    assert!(generated_c.contains("sec4_rt_db_exec_tx(tx, built)"));
    assert!(generated_c.contains("sec4_rt_db_exec(db, built)"));
    assert!(generated_c.contains("sec4_rt_db_query_one(db, built, row)"));
    assert!(generated_c.contains("sec4_rt_fs_read(fs, path)"));
    assert!(generated_c.contains("sec4_rt_fs_write(fs, path, 1)"));
    assert!(generated_c.contains("sec4_rt_http_get(net, url)"));

    let binary_path = project_dir.join("build").join("dbfsnetdemo");
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
fn build_emit_c_bin_handles_secret_read_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin secret intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-secret-read");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "secretreaddemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn readSecret(sec: SecretsCap, token: Secret<String>) effects { secrets.read } -> Int {
  secrets.get(sec, "TOKEN");
  secrets.redact(token);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for secret-read intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_secret_get(sec, \"TOKEN\")"));
    assert!(generated_c.contains("sec4_rt_secret_redact(token)"));

    let binary_path = project_dir.join("build").join("secretreaddemo");
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
fn build_emit_c_bin_handles_crypto_ct_eq_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin crypto intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-crypto-cteq");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "cryptocteqdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn compare(a: Secret<String>, b: Secret<String>) -> Bool {
  crypto.ctEq(a, b)
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for crypto.ctEq intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_crypto_ct_eq(a, b)"));

    let binary_path = project_dir.join("build").join("cryptocteqdemo");
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
fn build_emit_c_bin_handles_gate_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin gate intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-gates");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "gatesdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn gates(input: Untrusted<String>, base: PathSafe) effects { net } -> Int {
  validate.headerValue(input);
  validate.email(input);
  validate.uuid(input);
  validate.int64(input);
  validate.nonEmpty(input);
  sanitize.html(input);
  url.public(input);
  url.internal(input);
  path.under(base, input);
  path.base("/tmp/base");
  headers.name("X-Test");
  headers.value("ok");
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for gate intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_validate_header_value(input)"));
    assert!(generated_c.contains("sec4_rt_validate_email(input)"));
    assert!(generated_c.contains("sec4_rt_validate_uuid(input)"));
    assert!(generated_c.contains("sec4_rt_validate_int64(input)"));
    assert!(generated_c.contains("sec4_rt_validate_non_empty(input)"));
    assert!(generated_c.contains("sec4_rt_sanitize_html(input)"));
    assert!(generated_c.contains("sec4_rt_url_public(input)"));
    assert!(generated_c.contains("sec4_rt_url_internal(input)"));
    assert!(generated_c.contains("sec4_rt_path_under(base, input)"));
    assert!(generated_c.contains("sec4_rt_path_base(\"/tmp/base\")"));
    assert!(generated_c.contains("sec4_rt_headers_name(\"X-Test\")"));
    assert!(generated_c.contains("sec4_rt_headers_value(\"ok\")"));

    let binary_path = project_dir.join("build").join("gatesdemo");
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
fn build_emit_c_bin_handles_http_router_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin http router integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-router");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httprouterdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  0
}

fn createUser() effects { net } -> Int {
  0
}

fn buildRouter() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", health);
  http.post(router, "/users", createUser);
  http.serve(1, router);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for http router intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_http_router()"));
    assert!(generated_c.contains("sec4_rt_http_route_get(router, \"/health\", health)"));
    assert!(generated_c.contains("sec4_rt_http_route_post(router, \"/users\", createUser)"));
    assert!(generated_c.contains("sec4_rt_http_serve(1, router)"));

    let binary_path = project_dir.join("build").join("httprouterdemo");
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
fn build_emit_c_bin_handles_security_middleware_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin security middleware integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-security-middleware");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "securitymiddlewaredemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() -> Int {
  let headers = sec.defaultHeaders();
  let corsCfg = cors.fromPolicy();
  let csrfCfg = csrf.fromPolicy();
  let authCfg = auth.fromPolicy();
  let router = http.router();
  let withHeaders = sec.withSecurityHeaders(router, headers);
  let withCors = cors.withCors(withHeaders, corsCfg);
  let withCsrf = csrf.withCsrf(withCors, csrfCfg);
  auth.withAuth(withCsrf, authCfg);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for security middleware intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_http_router()"));
    assert!(generated_c.contains("sec4_rt_with_security_headers(router, headers)"));
    assert!(generated_c.contains("sec4_rt_with_cors(withHeaders, corsCfg)"));
    assert!(generated_c.contains("sec4_rt_with_csrf(withCors, csrfCfg)"));
    assert!(generated_c.contains("sec4_rt_with_auth(withCsrf, authCfg)"));

    let binary_path = project_dir.join("build").join("securitymiddlewaredemo");
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
fn build_emit_c_bin_handles_policy_config_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin policy config integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-policy-config");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "policyconfigdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() -> Int {
  sec.defaultHeaders();
  let csp = sec.csp();
  sec.cspAdd(csp, "default-src", "'self'");
  cors.fromPolicy();
  csrf.fromPolicy();
  auth.fromPolicy();
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for policy-config intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_sec_default_headers()"));
    assert!(generated_c.contains("sec4_rt_sec_csp()"));
    assert!(generated_c.contains("sec4_rt_sec_csp_add(csp, \"default-src\", \"'self'\")"));
    assert!(generated_c.contains("sec4_rt_cors_from_policy()"));
    assert!(generated_c.contains("sec4_rt_csrf_from_policy()"));
    assert!(generated_c.contains("sec4_rt_auth_from_policy()"));

    let binary_path = project_dir.join("build").join("policyconfigdemo");
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
fn build_emit_c_bin_handles_auth_requirement_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin auth requirement integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-auth-require");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "authrequiredemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn enforceAuth(ctx: Ctx) -> Int {
  auth.require(ctx);
  auth.requireRole(ctx, "admin");
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for auth requirement intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_auth_require(ctx)"));
    assert!(generated_c.contains("sec4_rt_auth_require_role(ctx, \"admin\")"));

    let binary_path = project_dir.join("build").join("authrequiredemo");
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
fn build_emit_c_bin_handles_error_builder_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin error builder integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-error-builders");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "errorbuildersdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() -> Int {
  let base = err.validation("VALIDATION.BAD_REQUEST", "invalid input");
  err.auth("AUTH.FORBIDDEN", "forbidden", 401);
  err.notFound("RESOURCE.NOT_FOUND", "missing");
  err.conflict("RESOURCE.CONFLICT", "conflict");
  err.rateLimit("LIMIT.RATE", "rate limited", 3);
  let internal = err.internal("internal");
  err.withPath(base, "$.field");
  err.withDetail(base, "field", 2);
  err.withLimit(base, "limit", 2, 3);
  err.withDependency(base, "postgres", "query", true);
  err.withCause(base, internal);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for error builder intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c
        .contains("sec4_rt_err_validation(\"VALIDATION.BAD_REQUEST\", \"invalid input\")"));
    assert!(generated_c.contains("sec4_rt_err_auth(\"AUTH.FORBIDDEN\", \"forbidden\", 401)"));
    assert!(generated_c.contains("sec4_rt_err_not_found(\"RESOURCE.NOT_FOUND\", \"missing\")"));
    assert!(generated_c.contains("sec4_rt_err_conflict(\"RESOURCE.CONFLICT\", \"conflict\")"));
    assert!(generated_c.contains("sec4_rt_err_rate_limit(\"LIMIT.RATE\", \"rate limited\", 3)"));
    assert!(generated_c.contains("sec4_rt_err_internal(\"internal\")"));
    assert!(generated_c.contains("sec4_rt_err_with_path(base, \"$.field\")"));
    assert!(generated_c.contains("sec4_rt_err_with_detail(base, \"field\", 2)"));
    assert!(generated_c.contains("sec4_rt_err_with_limit(base, \"limit\", 2, 3)"));
    assert!(
        generated_c.contains("sec4_rt_err_with_dependency(base, \"postgres\", \"query\", true)")
    );
    assert!(generated_c.contains("sec4_rt_err_with_cause(base, internal)"));

    let binary_path = project_dir.join("build").join("errorbuildersdemo");
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
fn build_emit_c_bin_handles_cors_origin_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin cors.origin integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-cors-origin");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "corsorigindemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { net } -> Int {
  let origin = req.header("origin");
  cors.origin(origin);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for cors.origin intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_req_header(\"origin\")"));
    assert!(generated_c.contains("sec4_rt_cors_origin(origin)"));

    let binary_path = project_dir.join("build").join("corsorigindemo");
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
fn build_emit_c_bin_handles_csrf_issue_token_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin csrf.issueToken integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-csrf-issue-token");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "csrfissuetokendemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn main() effects { net } -> Int {
  csrf.issueToken(1);
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should succeed for csrf.issueToken intrinsic project"
    );

    let generated_c = fs::read_to_string(project_dir.join("build").join("generated.c"))
        .expect("read generated C");
    assert!(generated_c.contains("sec4_rt_csrf_issue_token(1)"));

    let binary_path = project_dir.join("build").join("csrfissuetokendemo");
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
fn build_emit_c_bin_accepts_http_surface_types_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin http surface type integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-surface-types");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpsurfacetypesdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn health() effects { net } -> Int {
  0
}

fn createUser() effects { net } -> Int {
  0
}

fn wireRoutes(router: Router, request: Request, response: Response) effects { net } -> Int {
  request;
  response;
  http.get(router, "/health", health);
  http.post(router, "/users", createUser);
  http.serve(1, router);
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should accept http surface type names"
    );

    let binary_path = project_dir.join("build").join("httpsurfacetypesdemo");
    assert!(binary_path.exists(), "compiled binary should exist");
}

#[test]
fn build_emit_c_bin_accepts_security_config_surface_types_when_clang_available() {
    if !clang_available() {
        eprintln!(
            "skipping c-bin security config surface type integration test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-security-config-surface-types");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "securityconfigsurfacetypesdemo"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn configShapes(
  corsCfg: CorsConfig,
  headersCfg: SecurityHeadersConfig,
  csrfCfg: CsrfConfig,
  authCfg: AuthConfig,
  cspCfg: CspConfig,
  cspPolicy: CspPolicy,
  principal: Principal,
  caps: Caps,
  origin: Origin,
  originPattern: OriginPattern,
  origins: CorsOrigins
) -> Int {
  0
}

fn main() -> Int {
  0
}
"#,
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "c-bin build should accept security config surface type names"
    );

    let binary_path = project_dir
        .join("build")
        .join("securityconfigsurfacetypesdemo");
    assert!(binary_path.exists(), "compiled binary should exist");
}

#[test]
fn run_command_executes_compiled_binary_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping run integration test: clang not available");
        return;
    }

    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");

    let output = run_cli(&["run", "--path", hello]);
    assert!(output.status.success(), "run command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("compiled binary:"),
        "run command should compile through the c-bin pipeline"
    );
}

#[test]
fn build_emit_c_bin_compiles_hello_api_example_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping hello-api c-bin integration test: clang not available");
        return;
    }

    let hello_api_path = workspace_root().join("examples/hello-api");
    let hello_api = hello_api_path
        .to_str()
        .expect("hello-api path should be valid utf-8");

    let output = run_cli(&["build", "--path", hello_api, "--emit", "c-bin"]);
    assert!(
        output.status.success(),
        "hello-api should compile through c-bin pipeline"
    );

    let generated_c = fs::read_to_string(hello_api_path.join("build").join("generated.c"))
        .expect("generated C should exist for hello-api");
    assert!(generated_c.contains("sec4_rt_http_router()"));
    assert!(generated_c.contains("sec4_rt_http_route_get(router, \"/health\", health)"));
    assert!(generated_c.contains("sec4_rt_http_route_post(router, \"/users\", createUser)"));
    assert!(generated_c.contains("sec4_rt_req_json(\"CreateUserRequest\")"));
    assert!(generated_c.contains("sec4_rt_res_ok(201, \"CreateUserResponse\", 1)"));
    assert!(generated_c.contains("sec4_rt_res_text(200, \"ok\")"));

    let binary_path = hello_api_path.join("build").join("hello-api");
    assert!(binary_path.exists(), "hello-api binary should exist");
}

#[test]
fn run_command_executes_hello_api_example_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping hello-api run integration test: clang not available");
        return;
    }

    let hello_api_path = workspace_root().join("examples/hello-api");
    let hello_api = hello_api_path
        .to_str()
        .expect("hello-api path should be valid utf-8");

    let output = run_cli(&["run", "--path", hello_api]);
    assert!(
        output.status.success(),
        "run command should succeed for hello-api example"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("compiled binary:"),
        "run command should compile hello-api through c-bin pipeline"
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
