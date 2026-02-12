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
    assert!(
        hello_path.join("build").join("ailang_runtime.h").exists(),
        "runtime header should exist"
    );
    assert!(
        hello_path.join("build").join("ailang_runtime.c").exists(),
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

    let project_dir = temp_dir("ailang-c-bin-flow");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "flowdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("int64_t choose(bool flag);"));
    assert!(generated_c.contains("if (flag) goto"));
    assert!(generated_c.contains("return ailang_rt_identity_i64(choose(true));"));

    let binary_path = project_dir.join("build").join("flowdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_time_now_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-time-now");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "timedemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_time_now()"));

    let binary_path = project_dir.join("build").join("timedemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_log_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin log intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-log");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "logdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn main() effects { log } -> Int {
  log.info(1);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_log_any(1)"));

    let binary_path = project_dir.join("build").join("logdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_log_builder_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin log builder integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-log-builders");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "logbuildersdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn main() -> Int {
  let event = log.event(1);
  let field = log.field(1, 2);
  let obj = log.obj(1);
  let text = log.str(1);
  let num = log.i64(1);
  let flag = log.bool(1);
  let secret = log.redacted(1);
  let attrSecret = log.attrRedacted(1);
  let withAttr = log.withAttr(event, 1, attrSecret);
  let withHttp = log.withHttp(withAttr, 1, 2, 200, 42);
  let withError = log.withError(withHttp, 1);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_log_event(1)"));
    assert!(generated_c.contains("ailang_rt_log_field(1, 2)"));
    assert!(generated_c.contains("ailang_rt_log_obj(1)"));
    assert!(generated_c.contains("ailang_rt_log_str(1)"));
    assert!(generated_c.contains("ailang_rt_log_i64(1)"));
    assert!(generated_c.contains("ailang_rt_log_bool(1)"));
    assert!(generated_c.contains("ailang_rt_log_redacted(1)"));
    assert!(generated_c.contains("ailang_rt_log_attr_redacted(1)"));
    assert!(generated_c.contains("ailang_rt_log_with_attr(event, 1, attrSecret)"));
    assert!(generated_c.contains("ailang_rt_log_with_http(withAttr, 1, 2, 200, 42)"));
    assert!(generated_c.contains("ailang_rt_log_with_error(withHttp, 1)"));

    let binary_path = project_dir.join("build").join("logbuildersdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_req_res_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin req/res intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-req-res");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "reqresdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn decode(schema: Schema<Int>) effects { net } -> Int {
  req.body(1, 2);
  req.query(1, 2);
  req.pathParam(1, 2);
  req.header(1, 2);
  req.json(schema);
  0
}

fn encode(schema: Schema<Int>) effects { net } -> Int {
  res.json(schema, 1);
  res.ok(201, schema, 1);
  res.okMeta(201, schema, 1, 2);
  res.html(1);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_req_body(1, 2)"));
    assert!(generated_c.contains("ailang_rt_req_query(1, 2)"));
    assert!(generated_c.contains("ailang_rt_req_path_param(1, 2)"));
    assert!(generated_c.contains("ailang_rt_req_header(1, 2)"));
    assert!(generated_c.contains("ailang_rt_req_json(schema)"));
    assert!(generated_c.contains("ailang_rt_res_json(schema, 1)"));
    assert!(generated_c.contains("ailang_rt_res_ok(201, schema, 1)"));
    assert!(generated_c.contains("ailang_rt_res_ok_meta(201, schema, 1, 2)"));
    assert!(generated_c.contains("ailang_rt_res_html(1)"));
    assert!(generated_c.contains("ailang_rt_res_text(200, \"ok\")"));

    let binary_path = project_dir.join("build").join("reqresdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_json_helper_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin json helper integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-json-helpers");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "jsonhelpersdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn main() -> Int {
  json.decode(1, 2, 3);
  json.encode(2, 3);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_json_decode(1, 2, 3)"));
    assert!(generated_c.contains("ailang_rt_json_encode(2, 3)"));

    let binary_path = project_dir.join("build").join("jsonhelpersdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_header_cookie_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin header/cookie intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-header-cookie");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "headercookiedemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn configure() effects { net } -> Int {
  let cookie = cookie.build(1, 2);
  res.setHeader(1, 2);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_cookie_build(1, 2)"));
    assert!(generated_c.contains("ailang_rt_set_header(1, 2)"));
    assert!(generated_c.contains("ailang_rt_set_cookie(cookie)"));

    let binary_path = project_dir.join("build").join("headercookiedemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_db_fs_net_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin db/fs/net intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-db-fs-net");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "dbfsnetdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn ioOps(
  db: DbCap,
  tx: TxCap,
  fs: FsCap,
  net: NetCap,
  query: SqlQuery,
  path: PathSafe,
  url: PublicUrl
) effects { db.write, db.read, db.tx, fs.read, fs.write, net } -> Int {
  let built = sql.q(1, 2);
  db.tx(db);
  db.execTx(tx, built);
  db.exec(db, built);
  db.queryOne(db, built, 1);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_sql_q(1, 2)"));
    assert!(generated_c.contains("ailang_rt_db_tx(db)"));
    assert!(generated_c.contains("ailang_rt_db_exec_tx(tx, built)"));
    assert!(generated_c.contains("ailang_rt_db_exec(db, built)"));
    assert!(generated_c.contains("ailang_rt_db_query_one(db, built, 1)"));
    assert!(generated_c.contains("ailang_rt_fs_read(fs, path)"));
    assert!(generated_c.contains("ailang_rt_fs_write(fs, path, 1)"));
    assert!(generated_c.contains("ailang_rt_http_get(net, url)"));

    let binary_path = project_dir.join("build").join("dbfsnetdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_secret_read_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin secret intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-secret-read");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "secretreaddemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn readSecret(sec: SecretsCap) effects { secrets.read } -> Int {
  secrets.get(sec, 1);
  secrets.redact(1);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_secret_get(sec, 1)"));
    assert!(generated_c.contains("ailang_rt_secret_redact(1)"));

    let binary_path = project_dir.join("build").join("secretreaddemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_gate_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin gate intrinsic integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-gates");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "gatesdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
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
  path.base(1);
  headers.name(1);
  headers.value(1);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_validate_header_value(input)"));
    assert!(generated_c.contains("ailang_rt_validate_email(input)"));
    assert!(generated_c.contains("ailang_rt_validate_uuid(input)"));
    assert!(generated_c.contains("ailang_rt_validate_int64(input)"));
    assert!(generated_c.contains("ailang_rt_validate_non_empty(input)"));
    assert!(generated_c.contains("ailang_rt_sanitize_html(input)"));
    assert!(generated_c.contains("ailang_rt_url_public(input)"));
    assert!(generated_c.contains("ailang_rt_url_internal(input)"));
    assert!(generated_c.contains("ailang_rt_path_under(base, input)"));
    assert!(generated_c.contains("ailang_rt_path_base(1)"));
    assert!(generated_c.contains("ailang_rt_headers_name(1)"));
    assert!(generated_c.contains("ailang_rt_headers_value(1)"));

    let binary_path = project_dir.join("build").join("gatesdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_http_router_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin http router integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-http-router");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "httprouterdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_http_router()"));
    assert!(generated_c.contains("ailang_rt_http_route_get(router, \"/health\", health)"));
    assert!(generated_c.contains("ailang_rt_http_route_post(router, \"/users\", createUser)"));
    assert!(generated_c.contains("ailang_rt_http_serve(1, router)"));

    let binary_path = project_dir.join("build").join("httprouterdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_security_middleware_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin security middleware integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-security-middleware");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "securitymiddlewaredemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_http_router()"));
    assert!(generated_c.contains("ailang_rt_with_security_headers(router, headers)"));
    assert!(generated_c.contains("ailang_rt_with_cors(withHeaders, corsCfg)"));
    assert!(generated_c.contains("ailang_rt_with_csrf(withCors, csrfCfg)"));
    assert!(generated_c.contains("ailang_rt_with_auth(withCsrf, authCfg)"));

    let binary_path = project_dir.join("build").join("securitymiddlewaredemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_policy_config_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin policy config integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-policy-config");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "policyconfigdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn main() -> Int {
  sec.defaultHeaders();
  let csp = sec.csp();
  sec.cspAdd(csp, 1, 2);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_sec_default_headers()"));
    assert!(generated_c.contains("ailang_rt_sec_csp()"));
    assert!(generated_c.contains("ailang_rt_sec_csp_add(csp, 1, 2)"));
    assert!(generated_c.contains("ailang_rt_cors_from_policy()"));
    assert!(generated_c.contains("ailang_rt_csrf_from_policy()"));
    assert!(generated_c.contains("ailang_rt_auth_from_policy()"));

    let binary_path = project_dir.join("build").join("policyconfigdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_auth_requirement_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin auth requirement integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-auth-require");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "authrequiredemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn main() -> Int {
  auth.require(1);
  auth.requireRole(1, 2);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_auth_require(1)"));
    assert!(generated_c.contains("ailang_rt_auth_require_role(1, 2)"));

    let binary_path = project_dir.join("build").join("authrequiredemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_error_builder_intrinsics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin error builder integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-error-builders");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "errorbuildersdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn main() -> Int {
  let base = err.validation(1, 2);
  err.auth(1, 2, 401);
  err.notFound(1, 2);
  err.conflict(1, 2);
  err.rateLimit(1, 2, 3);
  let internal = err.internal(1);
  err.withPath(base, 1);
  err.withDetail(base, 1, 2);
  err.withLimit(base, 1, 2, 3);
  err.withDependency(base, 1, 2, 3);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_err_validation(1, 2)"));
    assert!(generated_c.contains("ailang_rt_err_auth(1, 2, 401)"));
    assert!(generated_c.contains("ailang_rt_err_not_found(1, 2)"));
    assert!(generated_c.contains("ailang_rt_err_conflict(1, 2)"));
    assert!(generated_c.contains("ailang_rt_err_rate_limit(1, 2, 3)"));
    assert!(generated_c.contains("ailang_rt_err_internal(1)"));
    assert!(generated_c.contains("ailang_rt_err_with_path(base, 1)"));
    assert!(generated_c.contains("ailang_rt_err_with_detail(base, 1, 2)"));
    assert!(generated_c.contains("ailang_rt_err_with_limit(base, 1, 2, 3)"));
    assert!(generated_c.contains("ailang_rt_err_with_dependency(base, 1, 2, 3)"));
    assert!(generated_c.contains("ailang_rt_err_with_cause(base, internal)"));

    let binary_path = project_dir.join("build").join("errorbuildersdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_cors_origin_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin cors.origin integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-cors-origin");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "corsorigindemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
        r#"fn main() effects { net } -> Int {
  let origin = req.header(1, 2);
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_req_header(1, 2)"));
    assert!(generated_c.contains("ailang_rt_cors_origin(origin)"));

    let binary_path = project_dir.join("build").join("corsorigindemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_handles_csrf_issue_token_intrinsic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin csrf.issueToken integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-csrf-issue-token");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "csrfissuetokendemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
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

    let generated_c =
        fs::read_to_string(project_dir.join("build").join("generated.c")).expect("read generated C");
    assert!(generated_c.contains("ailang_rt_csrf_issue_token(1)"));

    let binary_path = project_dir.join("build").join("csrfissuetokendemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(run.status.success(), "compiled binary should exit successfully");
}

#[test]
fn build_emit_c_bin_accepts_http_surface_types_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin http surface type integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-http-surface-types");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "httpsurfacetypesdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
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
        eprintln!("skipping c-bin security config surface type integration test: clang not available");
        return;
    }

    let project_dir = temp_dir("ailang-c-bin-security-config-surface-types");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("ailang.toml"),
        r#"[package]
name = "securityconfigsurfacetypesdemo"
version = "0.1.0"

[build]
entry = "src/main.ai"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ai"),
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

    let binary_path = project_dir.join("build").join("securityconfigsurfacetypesdemo");
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
    assert!(generated_c.contains("ailang_rt_http_router()"));
    assert!(generated_c.contains("ailang_rt_http_route_get(router, \"/health\", health)"));
    assert!(generated_c.contains("ailang_rt_http_route_post(router, \"/users\", createUser)"));
    assert!(generated_c.contains("ailang_rt_req_json(\"CreateUserRequest\")"));
    assert!(generated_c.contains("ailang_rt_res_ok(201, \"CreateUserResponse\", 1)"));
    assert!(generated_c.contains("ailang_rt_res_text(200, \"ok\")"));

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
