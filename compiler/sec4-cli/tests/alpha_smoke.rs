use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

fn find_available_tcp_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("ephemeral tcp bind should work");
    listener
        .local_addr()
        .expect("listener local address should resolve")
        .port()
}

#[test]
fn alpha_smoke_full_flow_init_check_build_run_oneshot() {
    if !clang_available() {
        eprintln!("skipping alpha smoke full-flow test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-alpha-smoke");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let init_output = run_cli(&[
        "init",
        "--path",
        &project_path,
        "--name",
        "alpha_smoke_demo",
    ]);
    assert!(
        init_output.status.success(),
        "init should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init_output.stdout),
        String::from_utf8_lossy(&init_output.stderr)
    );

    let check_output = run_cli(&["check", "--path", &project_path]);
    assert!(
        check_output.status.success(),
        "check should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check_output.stdout),
        String::from_utf8_lossy(&check_output.stderr)
    );

    let build_output = run_cli(&["build", "--path", &project_path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "build --emit c-bin should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build_output.stdout),
        String::from_utf8_lossy(&build_output.stderr)
    );

    let port = find_available_tcp_port();
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
            panic!("alpha smoke test could not connect to oneshot server");
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
            panic!("alpha smoke oneshot process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run --oneshot process should exit successfully"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line:\n{response}"
    );
    assert!(
        response.contains("\r\n\r\nhello from sec4"),
        "response should include expected generated route body:\n{response}"
    );
    assert!(
        response.contains("\r\nX-Trace-Id:"),
        "response should include X-Trace-Id header:\n{response}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn alpha_smoke_check_fails_with_diagnostics_for_broken_source() {
    let project_dir = temp_dir("sec4-alpha-smoke-negative");
    let project_path = project_dir
        .to_str()
        .expect("project path should be valid utf-8")
        .to_string();

    let init_output = run_cli(&[
        "init",
        "--path",
        &project_path,
        "--name",
        "alpha_smoke_demo",
    ]);
    assert!(
        init_output.status.success(),
        "init should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init_output.stdout),
        String::from_utf8_lossy(&init_output.stderr)
    );

    fs::write(
        project_dir.join("src/main.ut"),
        "fn main( -> Int {\n  0\n}\n",
    )
    .expect("broken source should be written");

    let check_output = run_cli(&["check", "--path", &project_path]);
    assert!(
        !check_output.status.success(),
        "check should fail for broken source\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check_output.stdout),
        String::from_utf8_lossy(&check_output.stderr)
    );

    let stderr = String::from_utf8(check_output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("P2004"),
        "broken-source diagnostics should include parser code P2004:\n{stderr}"
    );
    assert!(
        stderr.contains("expected parameter name"),
        "broken-source diagnostics should include parser explanation:\n{stderr}"
    );
    assert!(
        stderr.contains("src/main.ut"),
        "broken-source diagnostics should include source path:\n{stderr}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}
