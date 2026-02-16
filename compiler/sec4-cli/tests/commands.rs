use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::Duration;
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
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("entry should be written");
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
        stdout.contains(
            "test summary: static_passed=2, static_failed=0, runtime_passed=2, runtime_failed=0"
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
        stdout.contains(
            "test summary: static_passed=0, static_failed=1, runtime_passed=0, runtime_failed=0"
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
        stdout.contains(
            "test summary: static_passed=2, static_failed=0, runtime_passed=1, runtime_failed=1"
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
            panic!(
                "run command dns-resolution policy process did not exit in expected window"
            );
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
