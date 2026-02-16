use serde_json::Value;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

fn openssl_cli_available() -> bool {
    Command::new("openssl")
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn find_available_tcp_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("ephemeral tcp bind should work");
    listener
        .local_addr()
        .expect("listener local address should resolve")
        .port()
}

fn spawn_one_shot_http_server(body: &str) -> (u16, thread::JoinHandle<()>) {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("oneshot server local address should resolve")
        .port();
    let payload = body.as_bytes().to_vec();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        let response_head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            payload.len()
        );
        stream
            .write_all(response_head.as_bytes())
            .expect("oneshot server response headers should write");
        if !payload.is_empty() {
            stream
                .write_all(&payload)
                .expect("oneshot server response body should write");
        }
        stream
            .flush()
            .expect("oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_chunked_server(body: &str) -> (u16, thread::JoinHandle<()>) {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("chunked oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("chunked oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("chunked oneshot server local address should resolve")
        .port();
    let payload = body.as_bytes().to_vec();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("chunked oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("chunked oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("chunked oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
            )
            .expect("chunked oneshot server response headers should write");

        let mut cursor = 0usize;
        while cursor < payload.len() {
            let chunk_len = (payload.len() - cursor).min(6);
            let chunk_head = format!("{chunk_len:X}\r\n");
            stream
                .write_all(chunk_head.as_bytes())
                .expect("chunked oneshot server chunk length should write");
            stream
                .write_all(&payload[cursor..cursor + chunk_len])
                .expect("chunked oneshot server chunk body should write");
            stream
                .write_all(b"\r\n")
                .expect("chunked oneshot server chunk terminator should write");
            cursor += chunk_len;
        }

        stream
            .write_all(b"0\r\n\r\n")
            .expect("chunked oneshot server final chunk should write");
        stream
            .flush()
            .expect("chunked oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_malformed_chunked_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("malformed chunked oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("malformed chunked oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("malformed chunked oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("malformed chunked oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("malformed chunked oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("malformed chunked oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
            )
            .expect("malformed chunked oneshot server response headers should write");
        // Invalid frame: declares 5 bytes but provides 3.
        stream
            .write_all(b"5\r\nabc\r\n0\r\n\r\n")
            .expect("malformed chunked oneshot server payload should write");
        stream
            .flush()
            .expect("malformed chunked oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_chunked_with_trailers_server(body: &str) -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("chunked trailers oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("chunked trailers oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("chunked trailers oneshot server local address should resolve")
        .port();
    let payload = body.as_bytes().to_vec();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("chunked trailers oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("chunked trailers oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("chunked trailers oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
            )
            .expect("chunked trailers oneshot server response headers should write");

        let mut cursor = 0usize;
        let mut chunk_index = 0usize;
        while cursor < payload.len() {
            let chunk_len = (payload.len() - cursor).min(5);
            let chunk_head = if chunk_index == 0 {
                format!("{chunk_len:X};ext=1\r\n")
            } else {
                format!("{chunk_len:X}\r\n")
            };
            stream
                .write_all(chunk_head.as_bytes())
                .expect("chunked trailers oneshot server chunk length should write");
            stream
                .write_all(&payload[cursor..cursor + chunk_len])
                .expect("chunked trailers oneshot server chunk body should write");
            stream
                .write_all(b"\r\n")
                .expect("chunked trailers oneshot server chunk terminator should write");
            cursor += chunk_len;
            chunk_index += 1;
        }

        stream
            .write_all(b"0\r\nX-Test: one\r\nY-Trace: abc\r\n\r\n")
            .expect("chunked trailers oneshot server final trailer should write");
        stream
            .flush()
            .expect("chunked trailers oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_chunked_missing_trailer_terminator_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("chunked missing terminator oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("chunked missing terminator oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("chunked missing terminator oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!(
                            "chunked missing terminator oneshot server timed out waiting for client"
                        );
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("chunked missing terminator oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("chunked missing terminator server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
            )
            .expect("chunked missing terminator response headers should write");
        // Missing final CRLF that should terminate trailers.
        stream
            .write_all(b"4\r\ntest\r\n0\r\nX-Trailer: missing-end\r\n")
            .expect("chunked missing terminator payload should write");
        stream
            .flush()
            .expect("chunked missing terminator response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_large_headers_server() -> (u16, thread::JoinHandle<()>) {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("large-headers oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("large-headers oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("large-headers oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("large-headers oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("large-headers oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("large-headers oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        let oversized_header = "x".repeat(8500);
        let response_head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Large: {oversized_header}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream
            .write_all(response_head.as_bytes())
            .expect("large-headers oneshot server response should write");
        stream
            .flush()
            .expect("large-headers oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_conflicting_framing_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("conflicting framing oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("conflicting framing oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("conflicting framing oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("conflicting framing oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("conflicting framing oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("conflicting framing oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked\r\nContent-Length: 4\r\nConnection: close\r\n\r\n4\r\ntest\r\n0\r\n\r\n",
            )
            .expect("conflicting framing oneshot server response should write");
        stream
            .flush()
            .expect("conflicting framing oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_status_line_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("invalid status-line oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("invalid status-line oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("invalid status-line oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("invalid status-line oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("invalid status-line oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("invalid status-line oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1X 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\nConnection: close\r\n\r\ntest",
            )
            .expect("invalid status-line oneshot server response should write");
        stream
            .flush()
            .expect("invalid status-line oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_transfer_encoding_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("invalid transfer-encoding oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("invalid transfer-encoding oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("invalid transfer-encoding oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!(
                            "invalid transfer-encoding oneshot server timed out waiting for client"
                        );
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => {
                    panic!("invalid transfer-encoding oneshot server accept failed: {err}")
                }
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("invalid transfer-encoding oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: chunked, gzip\r\nConnection: close\r\n\r\n4\r\ntest\r\n0\r\n\r\n",
            )
            .expect("invalid transfer-encoding oneshot server response should write");
        stream
            .flush()
            .expect("invalid transfer-encoding oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_unsupported_transfer_encoding_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("unsupported transfer-encoding oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("unsupported transfer-encoding oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("unsupported transfer-encoding oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!(
                            "unsupported transfer-encoding oneshot server timed out waiting for client"
                        );
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => {
                    panic!("unsupported transfer-encoding oneshot server accept failed: {err}")
                }
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("unsupported transfer-encoding oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTransfer-Encoding: gzip\r\nConnection: close\r\n\r\ntest",
            )
            .expect("unsupported transfer-encoding oneshot server response should write");
        stream
            .flush()
            .expect("unsupported transfer-encoding oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_obs_fold_header_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("obs-fold header oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("obs-fold header oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("obs-fold header oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("obs-fold header oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("obs-fold header oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("obs-fold header oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Test: one\r\n\tcontinued\r\nContent-Length: 4\r\nConnection: close\r\n\r\ntest",
            )
            .expect("obs-fold header oneshot server response should write");
        stream
            .flush()
            .expect("obs-fold header oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_header_whitespace_before_colon_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("header whitespace oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("header whitespace oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("header whitespace oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("header whitespace oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("header whitespace oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("header whitespace oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Trace : abc\r\nContent-Length: 4\r\nConnection: close\r\n\r\ntest",
            )
            .expect("header whitespace oneshot server response should write");
        stream
            .flush()
            .expect("header whitespace oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_header_section_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("invalid header-section oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("invalid header-section oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("invalid header-section oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("invalid header-section oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("invalid header-section oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("invalid header-section oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        // Missing terminal CRLF CRLF for header section.
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4")
            .expect("invalid header-section oneshot server response should write");
        stream
            .flush()
            .expect("invalid header-section oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_header_control_char_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("header control-char oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("header control-char oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("header control-char oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("header control-char oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("header control-char oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("header control-char oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        let mut response = Vec::new();
        response.extend_from_slice(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Bad: value");
        response.push(0x01);
        response.extend_from_slice(b"\r\nContent-Length: 4\r\nConnection: close\r\n\r\ntest");
        stream
            .write_all(&response)
            .expect("header control-char oneshot server response should write");
        stream
            .flush()
            .expect("header control-char oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_content_type_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("invalid content-type oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("invalid content-type oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("invalid content-type oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("invalid content-type oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("invalid content-type oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("invalid content-type oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: textplain\r\nContent-Length: 4\r\nConnection: close\r\n\r\ntest",
            )
            .expect("invalid content-type oneshot server response should write");
        stream
            .flush()
            .expect("invalid content-type oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_unsupported_version_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("unsupported version oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("unsupported version oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("unsupported version oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("unsupported version oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("unsupported version oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("unsupported version oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/2.0 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4\r\nConnection: close\r\n\r\ntest",
            )
            .expect("unsupported version oneshot server response should write");
        stream
            .flush()
            .expect("unsupported version oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_header_line_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("invalid header-line oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("invalid header-line oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("invalid header-line oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("invalid header-line oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("invalid header-line oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("invalid header-line oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        // Missing colon in header line -> invalid parser input.
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Bad-Header value\r\nContent-Length: 4\r\nConnection: close\r\n\r\ntest",
            )
            .expect("invalid header-line oneshot server response should write");
        stream
            .flush()
            .expect("invalid header-line oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_retry_after_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("invalid retry-after oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("invalid retry-after oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("invalid retry-after oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("invalid retry-after oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("invalid retry-after oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("invalid retry-after oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: text/plain\r\nRetry-After: soon\r\nContent-Length: 4\r\nConnection: close\r\n\r\ntest",
            )
            .expect("invalid retry-after oneshot server response should write");
        stream
            .flush()
            .expect("invalid retry-after oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_conflicting_location_headers_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("conflicting location oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("conflicting location oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("conflicting location oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("conflicting location oneshot server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("conflicting location oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("conflicting location oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: /hop-a\r\nLocation: /hop-b\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("conflicting location oneshot server response should write");
        stream
            .flush()
            .expect("conflicting location oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_duplicate_content_length_equal_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("duplicate content-length equal oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("duplicate content-length equal oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("duplicate content-length equal oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("duplicate content-length equal server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => {
                    panic!("duplicate content-length equal oneshot server accept failed: {err}")
                }
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("duplicate content-length equal server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4, 4\r\nConnection: close\r\n\r\ntest",
            )
            .expect("duplicate content-length equal server response should write");
        stream
            .flush()
            .expect("duplicate content-length equal server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_duplicate_content_length_conflict_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("duplicate content-length conflict oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("duplicate content-length conflict oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("duplicate content-length conflict oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!(
                            "duplicate content-length conflict server timed out waiting for client"
                        );
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => {
                    panic!("duplicate content-length conflict oneshot server accept failed: {err}")
                }
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("duplicate content-length conflict server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 4, 5\r\nConnection: close\r\n\r\ntest",
            )
            .expect("duplicate content-length conflict server response should write");
        stream
            .flush()
            .expect("duplicate content-length conflict server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_server_with_response_delay(
    body: &str,
    delay: Duration,
) -> (u16, thread::JoinHandle<()>) {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("oneshot server local address should resolve")
        .port();
    let payload = body.as_bytes().to_vec();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("oneshot delayed server timed out waiting for client");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("oneshot delayed server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("oneshot delayed server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        thread::sleep(delay);

        let response_head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            payload.len()
        );
        let _ = stream.write_all(response_head.as_bytes());
        if !payload.is_empty() {
            let _ = stream.write_all(&payload);
        }
        let _ = stream.flush();
    });
    (port, handle)
}

fn spawn_one_shot_http_redirect_chain_server(
    expected_requests: usize,
) -> (u16, thread::JoinHandle<()>) {
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut served = 0_usize;
        while served < expected_requests {
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            panic!("redirect oneshot server timed out waiting for request");
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("redirect oneshot server accept failed: {err}"),
                }
            };

            let mut buffer = [0_u8; 1024];
            let mut request = Vec::new();
            loop {
                let bytes = stream
                    .read(&mut buffer)
                    .expect("redirect oneshot server request read should succeed");
                if bytes == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..bytes]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096
                {
                    break;
                }
            }

            let request_text = String::from_utf8_lossy(&request);
            let path = request_text
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");

            let (status_line, location, body) = match path {
                "/internal-start" => ("302 Found", Some("/internal-hop"), "redirect-start"),
                "/internal-hop" => ("302 Found", Some("/internal-final"), "redirect-hop"),
                "/internal-final" => ("200 OK", None, "internal-redirect-body"),
                _ => ("404 Not Found", None, "not-found"),
            };

            let payload = body.as_bytes();
            let mut response_head = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n",
                payload.len()
            );
            if let Some(location_value) = location {
                response_head.push_str(&format!("Location: {location_value}\r\n"));
            }
            response_head.push_str("\r\n");

            stream
                .write_all(response_head.as_bytes())
                .expect("redirect oneshot server response headers should write");
            if !payload.is_empty() {
                stream
                    .write_all(payload)
                    .expect("redirect oneshot server response body should write");
            }
            stream
                .flush()
                .expect("redirect oneshot server response flush should succeed");
            served += 1;
        }
    });
    (port, handle)
}

fn spawn_one_shot_http_monotonic_redirect_server(
    expected_requests: usize,
) -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("monotonic redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("monotonic redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("monotonic redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut served = 0_usize;
        while served < expected_requests {
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            panic!("monotonic redirect oneshot server timed out waiting for request");
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("monotonic redirect oneshot server accept failed: {err}"),
                }
            };

            let mut buffer = [0_u8; 1024];
            let mut request = Vec::new();
            loop {
                let bytes = stream
                    .read(&mut buffer)
                    .expect("monotonic redirect oneshot server request read should succeed");
                if bytes == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..bytes]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096
                {
                    break;
                }
            }

            let next_hop = served + 1;
            let response_head = format!(
                "HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: /hop/{next_hop}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream
                .write_all(response_head.as_bytes())
                .expect("monotonic redirect oneshot server response should write");
            stream
                .flush()
                .expect("monotonic redirect oneshot server response flush should succeed");
            served += 1;
        }
    });
    (port, handle)
}

fn spawn_one_shot_http_relative_redirect_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("relative redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("relative redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("relative redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut served = 0_usize;
        while served < 2 {
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            panic!("relative redirect oneshot server timed out waiting for request");
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("relative redirect oneshot server accept failed: {err}"),
                }
            };

            let mut buffer = [0_u8; 1024];
            let mut request = Vec::new();
            loop {
                let bytes = stream
                    .read(&mut buffer)
                    .expect("relative redirect oneshot server request read should succeed");
                if bytes == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..bytes]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096
                {
                    break;
                }
            }

            let request_text = String::from_utf8_lossy(&request);
            let path = request_text
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");

            let (status_line, location, body) = match path {
                "/svc/start" => ("302 Found", Some("../internal-final"), "redirect-rel-start"),
                "/internal-final" => ("200 OK", None, "internal-relative-body"),
                _ => ("404 Not Found", None, "not-found"),
            };

            let payload = body.as_bytes();
            let mut response_head = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n",
                payload.len()
            );
            if let Some(location_value) = location {
                response_head.push_str(&format!("Location: {location_value}\r\n"));
            }
            response_head.push_str("\r\n");

            stream
                .write_all(response_head.as_bytes())
                .expect("relative redirect oneshot server response headers should write");
            if !payload.is_empty() {
                stream
                    .write_all(payload)
                    .expect("relative redirect oneshot server response body should write");
            }
            stream
                .flush()
                .expect("relative redirect oneshot server response flush should succeed");
            served += 1;
        }
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_relative_redirect_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("invalid relative redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("invalid relative redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("invalid relative redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!(
                            "invalid relative redirect oneshot server timed out waiting for request"
                        );
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("invalid relative redirect oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("invalid relative redirect oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: ../../../../escape\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("invalid relative redirect oneshot server response should write");
        stream
            .flush()
            .expect("invalid relative redirect oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_fragment_redirect_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("fragment redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("fragment redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("fragment redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("fragment redirect oneshot server timed out waiting for request");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("fragment redirect oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("fragment redirect oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: /internal-final#frag\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("fragment redirect oneshot server response should write");
        stream
            .flush()
            .expect("fragment redirect oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_target_char_redirect_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("target-char redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("target-char redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("target-char redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("target-char redirect oneshot server timed out waiting for request");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("target-char redirect oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("target-char redirect oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: /internal final\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("target-char redirect oneshot server response should write");
        stream
            .flush()
            .expect("target-char redirect oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_query_percent_redirect_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("query-percent redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("query-percent redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("query-percent redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("query-percent redirect oneshot server timed out waiting for request");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("query-percent redirect oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("query-percent redirect oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: /internal-final?token=%zz\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("query-percent redirect oneshot server response should write");
        stream
            .flush()
            .expect("query-percent redirect oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_query_separator_redirect_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("query-separator redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("query-separator redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("query-separator redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("query-separator redirect oneshot server timed out waiting for request");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("query-separator redirect oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("query-separator redirect oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: /internal-final?ok=1?bad=2\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("query-separator redirect oneshot server response should write");
        stream
            .flush()
            .expect("query-separator redirect oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_scope_invalid_redirect_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("scope-invalid redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("scope-invalid redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("scope-invalid redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("scope-invalid redirect oneshot server timed out waiting for request");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("scope-invalid redirect oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("scope-invalid redirect oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: http://example.com/outside\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("scope-invalid redirect oneshot server response should write");
        stream
            .flush()
            .expect("scope-invalid redirect oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_missing_location_redirect_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("missing-location redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("missing-location redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("missing-location redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!(
                            "missing-location redirect oneshot server timed out waiting for request"
                        );
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("missing-location redirect oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("missing-location redirect oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("missing-location redirect oneshot server response should write");
        stream
            .flush()
            .expect("missing-location redirect oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_scheme_redirect_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("invalid-scheme redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("invalid-scheme redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("invalid-scheme redirect oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("invalid-scheme redirect oneshot server timed out waiting for request");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("invalid-scheme redirect oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("invalid-scheme redirect oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        stream
            .write_all(
                b"HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: ftp://example.com/outside\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .expect("invalid-scheme redirect oneshot server response should write");
        stream
            .flush()
            .expect("invalid-scheme redirect oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_redirect_cycle_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("redirect-cycle oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("redirect-cycle oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("redirect-cycle oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut served = 0_usize;
        while served < 2 {
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            panic!("redirect-cycle oneshot server timed out waiting for request");
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("redirect-cycle oneshot server accept failed: {err}"),
                }
            };

            let mut buffer = [0_u8; 1024];
            let mut request = Vec::new();
            loop {
                let bytes = stream
                    .read(&mut buffer)
                    .expect("redirect-cycle oneshot server request read should succeed");
                if bytes == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..bytes]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096
                {
                    break;
                }
            }

            let request_text = String::from_utf8_lossy(&request);
            let path = request_text
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");

            let location = match path {
                "/cycle/start" => "/cycle/loop",
                "/cycle/loop" => "/cycle/start",
                _ => "/cycle/start",
            };

            let response_head = format!(
                "HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream
                .write_all(response_head.as_bytes())
                .expect("redirect-cycle oneshot server response should write");
            stream
                .flush()
                .expect("redirect-cycle oneshot server response flush should succeed");
            served += 1;
        }
    });
    (port, handle)
}

fn spawn_one_shot_http_absolute_redirect_upper_host_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("absolute redirect upper-host oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("absolute redirect upper-host oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("absolute redirect upper-host oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut served = 0_usize;
        while served < 2 {
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            panic!(
                                "absolute redirect upper-host oneshot server timed out waiting for request"
                            );
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("absolute redirect upper-host oneshot server accept failed: {err}"),
                }
            };

            let mut buffer = [0_u8; 1024];
            let mut request = Vec::new();
            loop {
                let bytes = stream
                    .read(&mut buffer)
                    .expect("absolute redirect upper-host oneshot server request read should succeed");
                if bytes == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..bytes]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096
                {
                    break;
                }
            }

            let request_text = String::from_utf8_lossy(&request);
            let path = request_text
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");

            let (status_line, location, body) = match path {
                "/abs/start" => (
                    "302 Found",
                    Some(format!("http://LOCALHOST:{port}/internal-final")),
                    "redirect-abs-start".to_string(),
                ),
                "/internal-final" => ("200 OK", None, "internal-absolute-body".to_string()),
                _ => ("404 Not Found", None, "not-found".to_string()),
            };

            let payload = body.as_bytes();
            let mut response_head = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n",
                payload.len()
            );
            if let Some(location_value) = location {
                response_head.push_str(&format!("Location: {location_value}\r\n"));
            }
            response_head.push_str("\r\n");

            stream
                .write_all(response_head.as_bytes())
                .expect("absolute redirect upper-host oneshot server response headers should write");
            if !payload.is_empty() {
                stream
                    .write_all(payload)
                    .expect("absolute redirect upper-host oneshot server response body should write");
            }
            stream
                .flush()
                .expect("absolute redirect upper-host oneshot server response flush should succeed");
            served += 1;
        }
    });
    (port, handle)
}

fn spawn_one_shot_http_invalid_redirect_host_server() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("invalid redirect-host oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("invalid redirect-host oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("invalid redirect-host oneshot server local address should resolve")
        .port();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!(
                            "invalid redirect-host oneshot server timed out waiting for request"
                        );
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("invalid redirect-host oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("invalid redirect-host oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        let location_value = format!("http://bad_host:{port}/internal-final");
        let response_head = format!(
            "HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: {location_value}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream
            .write_all(response_head.as_bytes())
            .expect("invalid redirect-host oneshot server response should write");
        stream
            .flush()
            .expect("invalid redirect-host oneshot server response flush should succeed");
    });
    (port, handle)
}

fn spawn_one_shot_http_ipv6_malformed_redirect_server(
    location: &str,
) -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .expect("malformed ipv6 redirect oneshot server bind should work");
    listener
        .set_nonblocking(true)
        .expect("malformed ipv6 redirect oneshot server nonblocking setup should work");
    let port = listener
        .local_addr()
        .expect("malformed ipv6 redirect oneshot server local address should resolve")
        .port();
    let location_value = location.to_string();
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("malformed ipv6 redirect oneshot server timed out waiting for request");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("malformed ipv6 redirect oneshot server accept failed: {err}"),
            }
        };

        let mut buffer = [0_u8; 1024];
        let mut request = Vec::new();
        loop {
            let bytes = stream
                .read(&mut buffer)
                .expect("malformed ipv6 redirect oneshot server request read should succeed");
            if bytes == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..bytes]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") || request.len() >= 4096 {
                break;
            }
        }

        let response_head = format!(
            "HTTP/1.1 302 Found\r\nContent-Type: text/plain\r\nLocation: {location_value}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream
            .write_all(response_head.as_bytes())
            .expect("malformed ipv6 redirect oneshot server response should write");
        stream
            .flush()
            .expect("malformed ipv6 redirect oneshot server response flush should succeed");
    });
    (port, handle)
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

fn write_capture_file_with_multiple_db_fs_dependencies(
    path: &PathBuf,
    policy_hash: &str,
    compiler_hash: &str,
    runtime_hash: &str,
    db_requests: &[(&str, Option<&str>)],
    fs_requests: &[(&str, &str)],
) {
    let db_entries = db_requests
        .iter()
        .map(|(query_template_id, params_sha256)| {
            let mut db_request = serde_json::Map::new();
            db_request.insert(
                "queryTemplateId".to_string(),
                Value::String((*query_template_id).to_string()),
            );
            if let Some(params_sha256) = params_sha256 {
                db_request.insert(
                    "paramsSha256".to_string(),
                    Value::String((*params_sha256).to_string()),
                );
            }
            serde_json::json!({ "request": Value::Object(db_request) })
        })
        .collect::<Vec<_>>();
    let fs_entries = fs_requests
        .iter()
        .map(|(op, path_sha256)| {
            serde_json::json!({
                "request": {
                    "op": op,
                    "pathSha256": path_sha256
                }
            })
        })
        .collect::<Vec<_>>();

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
            "db": db_entries,
            "fs": fs_entries
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
fn replay_check_fails_when_capture_has_no_url_derivation_fields() {
    let dir = temp_dir("sec4-replay-missing-url");
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
        "replay check should fail when request signature URL cannot be derived"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("capture.request.url must be present or capture.request.scheme/host"),
        "stderr should include url-derivation failure reason:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_capture_method_is_not_uppercase() {
    let dir = temp_dir("sec4-replay-invalid-method-shape");
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
            "method":"get",
            "scheme":"https",
            "host":"example.com",
            "path":"/ping",
            "headers":{},
            "body":{"encoding":"base64","bytes":"e30=","sha256":"body_sha256","truncated":false}
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
        "replay check should fail when request.method is not uppercase"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("capture.request.method must be a non-empty uppercase string"),
        "stderr should include request.method contract failure:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_base64_capture_body_lacks_sha256() {
    let dir = temp_dir("sec4-replay-missing-base64-sha256");
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
            "body":{"encoding":"base64","bytes":"e30=","truncated":false}
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
        "replay check should fail when base64 body sha256 is missing"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("capture.request.body.sha256 must be present for encoding=base64"),
        "stderr should include base64 body sha256 contract failure:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_base64_capture_body_is_invalid() {
    let dir = temp_dir("sec4-replay-invalid-base64-body");
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
            "body":{"encoding":"base64","bytes":"%%%invalid%%%","sha256":"body_sha256","truncated":false}
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
        "replay check should fail when base64 body bytes are invalid"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("capture.request.body.bytes must be valid base64 for encoding=base64"),
        "stderr should include invalid base64 contract failure:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_none_capture_body_includes_bytes() {
    let dir = temp_dir("sec4-replay-none-body-includes-bytes");
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
            "body":{"encoding":"none","bytes":"e30=","sha256":"body_sha256","truncated":false}
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
        "replay check should fail when none body includes bytes"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("capture.request.body.bytes must be absent for encoding=none"),
        "stderr should include none-body bytes contract failure:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_capture_query_is_not_a_string() {
    let dir = temp_dir("sec4-replay-invalid-query-type");
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
            "query":5,
            "headers":{},
            "body":{"encoding":"base64","bytes":"e30=","sha256":"body_sha256","truncated":false}
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
        "replay check should fail when request.query is not a string"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("capture.request.query must be a string when present"),
        "stderr should include request.query contract failure:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_capture_url_is_not_a_string() {
    let dir = temp_dir("sec4-replay-invalid-url-type");
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
            "url":5,
            "scheme":"https",
            "host":"example.com",
            "path":"/ping",
            "headers":{},
            "body":{"encoding":"base64","bytes":"e30=","sha256":"body_sha256","truncated":false}
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
        "replay check should fail when request.url is not a string"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("capture.request.url must be a non-empty string when present"),
        "stderr should include request.url contract failure:\n{stderr}"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_fails_when_capture_route_is_not_a_string() {
    let dir = temp_dir("sec4-replay-invalid-route-type");
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
            "route":5,
            "headers":{},
            "body":{"encoding":"base64","bytes":"e30=","sha256":"body_sha256","truncated":false}
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
        "replay check should fail when request.route is not a string"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        stderr.contains("capture.request.route must be a non-empty string when present"),
        "stderr should include request.route contract failure:\n{stderr}"
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
    assert!(
        stdout.contains("replay mock executed stubs: net=1 db=0 fs=0"),
        "stdout should include deterministic executed-stub count summary:\n{stdout}"
    );
    assert!(
        stdout.contains(
            "replay mock execution traces: net=net:0(GET|https://example.com/ping|empty) db=- fs=-"
        ),
        "stdout should include deterministic executed-stub trace summary:\n{stdout}"
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
        stdout.contains(
            "replay mock dependency traces: db=db:0(users.by_id|abc123) fs=fs:0(read|p1)"
        ),
        "stdout should include deterministic dependency trace line:\n{stdout}"
    );
    assert!(
        stdout.contains("replay mock executed stubs: net=1 db=1 fs=1"),
        "stdout should include deterministic executed-stub count summary:\n{stdout}"
    );
    assert!(
        stdout.contains("replay mock execution traces: net=net:0(GET|https://example.com/ping|empty) db=db:0(users.by_id|abc123) fs=fs:0(read|p1)"),
        "stdout should include deterministic executed-stub trace summary:\n{stdout}"
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
            .and_then(|entry| entry.get("index"))
            .and_then(Value::as_u64)
            .expect("mockDependencyTraces.db[0].index should be present"),
        0
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
            .and_then(|entry| entry.get("index"))
            .and_then(Value::as_u64)
            .expect("mockDependencyTraces.fs[0].index should be present"),
        0
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
    assert_eq!(
        parsed
            .get("mockExecutionCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("net"))
            .and_then(Value::as_u64)
            .expect("mockExecutionCounts.net should be present"),
        1
    );
    assert_eq!(
        parsed
            .get("mockExecutionCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("db"))
            .and_then(Value::as_u64)
            .expect("mockExecutionCounts.db should be present"),
        1
    );
    assert_eq!(
        parsed
            .get("mockExecutionCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("fs"))
            .and_then(Value::as_u64)
            .expect("mockExecutionCounts.fs should be present"),
        1
    );
    assert_eq!(
        parsed
            .get("mockExecutionTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("net"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("traceId"))
            .and_then(Value::as_str)
            .expect("mockExecutionTraces.net[0].traceId should be present"),
        "net:0"
    );
    assert_eq!(
        parsed
            .get("mockExecutionTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("net"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("mockExecutionTraces.net[0].signature should be present"),
        "GET|https://example.com/ping|empty"
    );
    assert_eq!(
        parsed
            .get("mockExecutionTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("db"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("mockExecutionTraces.db[0].signature should be present"),
        "users.by_id|abc123"
    );
    assert_eq!(
        parsed
            .get("mockExecutionTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("fs"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("mockExecutionTraces.fs[0].signature should be present"),
        "read|p1"
    );

    fs::remove_dir_all(&dir).expect("temp project cleanup should succeed");
}

#[test]
fn replay_check_mock_mode_json_reports_dependency_trace_order_for_multiple_entries() {
    let dir = temp_dir("sec4-replay-mock-json-dependency-trace-order");
    let capture = dir.join("capture.json");
    let stubs = dir.join("stubs.json");
    write_capture_file_with_multiple_db_fs_dependencies(
        &capture,
        "pol_A",
        "cpl_A",
        "rt_A",
        &[("users.by_id", Some("abc123")), ("users.search", None)],
        &[("read", "p1"), ("write", "p2")],
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
        "replay json mode should pass with multiple dependency entries"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let parsed: Value =
        serde_json::from_str(&stdout).expect("replay --format json should output parseable json");

    let db_trace_entries = parsed
        .get("mockDependencyTraces")
        .and_then(Value::as_object)
        .and_then(|traces| traces.get("db"))
        .and_then(Value::as_array)
        .expect("mockDependencyTraces.db should be present");
    assert_eq!(db_trace_entries.len(), 2, "expected two db trace entries");
    assert_eq!(
        db_trace_entries
            .first()
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("index"))
            .and_then(Value::as_u64)
            .expect("first db trace index should be present"),
        0
    );
    assert_eq!(
        db_trace_entries
            .first()
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("traceId"))
            .and_then(Value::as_str)
            .expect("first db trace id should be present"),
        "db:0"
    );
    assert_eq!(
        db_trace_entries
            .first()
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("first db trace signature should be present"),
        "users.by_id|abc123"
    );
    assert_eq!(
        db_trace_entries
            .get(1)
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("index"))
            .and_then(Value::as_u64)
            .expect("second db trace index should be present"),
        1
    );
    assert_eq!(
        db_trace_entries
            .get(1)
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("traceId"))
            .and_then(Value::as_str)
            .expect("second db trace id should be present"),
        "db:1"
    );
    assert_eq!(
        db_trace_entries
            .get(1)
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("second db trace signature should be present"),
        "users.search|-"
    );

    let fs_trace_entries = parsed
        .get("mockDependencyTraces")
        .and_then(Value::as_object)
        .and_then(|traces| traces.get("fs"))
        .and_then(Value::as_array)
        .expect("mockDependencyTraces.fs should be present");
    assert_eq!(fs_trace_entries.len(), 2, "expected two fs trace entries");
    assert_eq!(
        fs_trace_entries
            .first()
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("index"))
            .and_then(Value::as_u64)
            .expect("first fs trace index should be present"),
        0
    );
    assert_eq!(
        fs_trace_entries
            .first()
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("traceId"))
            .and_then(Value::as_str)
            .expect("first fs trace id should be present"),
        "fs:0"
    );
    assert_eq!(
        fs_trace_entries
            .first()
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("first fs trace signature should be present"),
        "read|p1"
    );
    assert_eq!(
        fs_trace_entries
            .get(1)
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("index"))
            .and_then(Value::as_u64)
            .expect("second fs trace index should be present"),
        1
    );
    assert_eq!(
        fs_trace_entries
            .get(1)
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("traceId"))
            .and_then(Value::as_str)
            .expect("second fs trace id should be present"),
        "fs:1"
    );
    assert_eq!(
        fs_trace_entries
            .get(1)
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("second fs trace signature should be present"),
        "write|p2"
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
    assert_eq!(
        parsed
            .get("mockExecutionCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("net"))
            .and_then(Value::as_u64)
            .expect("mockExecutionCounts.net should be present"),
        1
    );
    assert_eq!(
        parsed
            .get("mockExecutionCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("db"))
            .and_then(Value::as_u64)
            .expect("mockExecutionCounts.db should be present"),
        0
    );
    assert_eq!(
        parsed
            .get("mockExecutionCounts")
            .and_then(Value::as_object)
            .and_then(|counts| counts.get("fs"))
            .and_then(Value::as_u64)
            .expect("mockExecutionCounts.fs should be present"),
        0
    );
    assert_eq!(
        parsed
            .get("mockExecutionTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("net"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("traceId"))
            .and_then(Value::as_str)
            .expect("mockExecutionTraces.net[0].traceId should be present"),
        "net:0"
    );
    assert_eq!(
        parsed
            .get("mockExecutionTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("net"))
            .and_then(Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("signature"))
            .and_then(Value::as_str)
            .expect("mockExecutionTraces.net[0].signature should be present"),
        "GET|https://example.com/ping|empty"
    );
    assert!(
        parsed
            .get("mockExecutionTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("db"))
            .and_then(Value::as_array)
            .is_some_and(|entries| entries.is_empty()),
        "mockExecutionTraces.db should be present and empty when capture has no dependencies"
    );
    assert!(
        parsed
            .get("mockExecutionTraces")
            .and_then(Value::as_object)
            .and_then(|traces| traces.get("fs"))
            .and_then(Value::as_array)
            .is_some_and(|entries| entries.is_empty()),
        "mockExecutionTraces.fs should be present and empty when capture has no dependencies"
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
fn build_emit_c_bin_compiles_with_openssl_tls_backend_when_toolchain_available() {
    if !clang_available() {
        eprintln!("skipping c-bin openssl tls-backend test: clang not available");
        return;
    }
    if !clang_with_openssl_available() {
        eprintln!(
            "skipping c-bin openssl tls-backend test: OpenSSL headers/libs not available to clang"
        );
        return;
    }

    let hello_path = workspace_root().join("examples/hello");
    let hello = hello_path
        .to_str()
        .expect("example path should be valid utf-8");

    let output = run_cli(&[
        "build",
        "--path",
        hello,
        "--emit",
        "c-bin",
        "--tls-backend",
        "openssl",
    ]);

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    assert!(
        output.status.success(),
        "expected success status for openssl tls backend build, stderr={stderr}"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("compiled binary:"),
        "build output should include compiled binary location for openssl tls backend"
    );
    assert!(
        !stderr.contains("TLS backend linkage failed"),
        "openssl build should not emit linkage failure diagnostic when toolchain is present"
    );

    let binary_path = hello_path.join("build").join("hello");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for openssl tls backend build"
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
        r#"fn main() effects { time.now } -> Int {
  time.now();
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
  log.warn(log.event("warn"));
  log.error(log.event("error"));
  log.emit(log.event("emit"));
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
    assert!(generated_c.contains("sec4_rt_log_info(sec4_rt_log_event(\"event\"))"));
    assert!(generated_c.contains("sec4_rt_log_warn(sec4_rt_log_event(\"warn\"))"));
    assert!(generated_c.contains("sec4_rt_log_error(sec4_rt_log_event(\"error\"))"));
    assert!(generated_c.contains("sec4_rt_log_any(sec4_rt_log_event(\"emit\"))"));

    let binary_path = project_dir.join("build").join("logdemo");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "compiled binary should exit successfully"
    );
    let stderr = String::from_utf8(run.stderr).expect("stderr should be valid utf-8");
    assert!(
        stderr.contains("\"event\":\"event\""),
        "runtime log output should include event name:\n{stderr}"
    );
    assert!(
        stderr.contains("\"level\":\"info\""),
        "runtime log output should include info level:\n{stderr}"
    );
    assert!(
        stderr.contains("\"event\":\"warn\"") && stderr.contains("\"level\":\"warn\""),
        "runtime log output should include warn level event:\n{stderr}"
    );
    assert!(
        stderr.contains("\"event\":\"error\"") && stderr.contains("\"level\":\"error\""),
        "runtime log output should include error level event:\n{stderr}"
    );
    assert!(
        stderr.contains("\"traceId\":\"rt-0\""),
        "runtime log output should include deterministic trace id outside HTTP request context:\n{stderr}"
    );
    assert!(
        stderr.contains("\"timeMs\":"),
        "runtime log output should include timeMs field:\n{stderr}"
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
        r#"fn main() effects { log } -> Int {
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
fn c_bin_runtime_log_builders_emit_structured_json_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping runtime structured log harness test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-structured-log-harness");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-structured-log");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"

int main(void) {
  int64_t event = sec4_rt_log_event("user.created");
  int64_t count = sec4_rt_log_i64(1);
  int64_t with_count = sec4_rt_log_with_attr(event, "count", count);
  int64_t redacted = sec4_rt_log_attr_redacted("token");
  int64_t with_attr = sec4_rt_log_with_attr(with_count, "token", redacted);
  int64_t with_http = sec4_rt_log_with_http(with_attr, "POST", "/users", 201, 12);
  int64_t err = sec4_rt_err_internal("boom");
  err = sec4_rt_err_with_path(err, "$.email");
  err = sec4_rt_err_with_detail(err, "validator", sec4_rt_log_str("validate.email"));
  int64_t with_error = sec4_rt_log_with_error(with_http, err);
  sec4_rt_log_info(with_error);
  sec4_rt_log_warn(with_error);
  sec4_rt_log_error(with_error);
  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime structured-log harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("runtime structured-log harness should run");
    assert!(
        run.status.success(),
        "runtime structured-log harness should exit successfully"
    );

    let stderr = String::from_utf8(run.stderr).expect("stderr should be valid utf-8");
    assert!(
        stderr.contains("\"event\":\"user.created\""),
        "structured log output should include event name:\n{stderr}"
    );
    assert!(
        stderr.contains("\"attrs\":{\"count\":1,\"token\":{\"redacted\":\"token\"}}"),
        "structured log output should include attrs block:\n{stderr}"
    );
    assert!(
        stderr.contains("\"http\":{\"method\":\"POST\",\"path\":\"/users\",\"status\":201,\"latencyMs\":12}"),
        "structured log output should include http metadata:\n{stderr}"
    );
    assert!(
        stderr.contains("\"error\":{\"code\":\"INTERNAL.ERROR\"")
            && stderr.contains("\"kind\":\"internal\"")
            && stderr.contains("\"message\":\"boom\"")
            && stderr.contains("\"status\":500")
            && stderr.contains("\"path\":\"$.email\"")
            && stderr.contains("\"details\":[{\"key\":\"validator\",\"value\":\"validate.email\"}]"),
        "structured log output should include structured error attachment:\n{stderr}"
    );
    assert!(
        stderr.contains("\"traceId\":\"rt-0\""),
        "structured log output should include default trace id outside request context:\n{stderr}"
    );
    assert!(
        stderr.contains("\"level\":\"info\"")
            && stderr.contains("\"level\":\"warn\"")
            && stderr.contains("\"level\":\"error\""),
        "structured log output should include info/warn/error levels:\n{stderr}"
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
        project_dir.join("sec4.policy"),
        r#"[fs]
enabled = true
"#,
    )
    .expect("policy should be written");
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
  let built = sql.q("SELECT 1 LIMIT 1", 2);
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
    assert!(generated_c.contains("sec4_rt_sql_q(\"SELECT 1 LIMIT 1\", 2)"));
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
        project_dir.join("sec4.policy"),
        r#"[net.internal]
enabled = true

[fs]
enabled = true
"#,
    )
    .expect("policy should be written");
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
fn c_bin_runtime_gate_handles_are_non_stub_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime gate handle test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-runtime-gate-handles");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runtimegatehandles"
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

[fs]
enabled = true
"#,
    )
    .expect("policy should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        r#"fn runtimeHandles() effects { net } -> Int {
  let rawA = req.query("user-a@example.com");
  let rawB = req.query("user-b@example.com");

  let emailA = validate.email(rawA);
  let emailB = validate.email(rawB);
  if emailA == emailB { return 11; };

  let headerA = validate.headerValue(rawA);
  let headerB = validate.headerValue(rawB);
  if headerA == headerB { return 12; };

  let rawPublicA = req.query("https://public-a.example/path");
  let rawPublicB = req.query("https://public-b.example/path");
  let publicA = url.public(rawPublicA);
  let publicB = url.public(rawPublicB);
  if publicA == publicB { return 13; };

  let rawInternalA = req.query("http://127.0.0.1/service-a");
  let rawInternalB = req.query("http://10.0.0.4/service-b");
  let internalA = url.internal(rawInternalA);
  let internalB = url.internal(rawInternalB);
  if internalA == internalB { return 14; };

  let rawBadPublicA = req.query("http://127.0.0.1/private-a");
  let rawBadPublicB = req.query("http://10.0.0.2/private-b");
  let badPublicA = url.public(rawBadPublicA);
  let badPublicB = url.public(rawBadPublicB);
  if badPublicA != badPublicB { return 19; };

  let baseA = path.base("/tmp/a");
  let baseB = path.base("/tmp/b");
  if baseA == baseB { return 15; };

  let safeA = path.under(baseA, rawA);
  let safeB = path.under(baseA, rawB);
  if safeA == safeB { return 16; };

  let nameA = headers.name("X-A");
  let nameB = headers.name("X-B");
  if nameA == nameB { return 17; };

  let valueA = headers.value("one");
  let valueB = headers.value("two");
  if valueA == valueB { return 18; };

  0
}

fn main() effects { net } -> Int {
  runtimeHandles()
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
        "c-bin build should succeed for runtime gate handle fixture"
    );

    let binary_path = project_dir.join("build").join("runtimegatehandles");
    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime gate handle fixture should exit successfully"
    );
}

#[test]
fn c_bin_runtime_path_and_header_guards_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime path/header guard test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-path-header-guards");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-path-header-guards");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"

int main(void) {
  if (sec4_rt_headers_name("X-Test") == 0) { return 11; }
  if (sec4_rt_headers_name("") != 0) { return 12; }
  if (sec4_rt_headers_value("ok") == 0) { return 13; }
  if (sec4_rt_headers_value("") != 0) { return 14; }
  if (sec4_rt_path_base("/tmp/base") == 0) { return 15; }
  if (sec4_rt_path_base("tmp/base") != 0) { return 16; }
  if (sec4_rt_path_base("/tmp/../base") != 0) { return 17; }
  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime path/header guard harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_validators_and_sanitizer_enforce_checks_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime validator/sanitizer test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-validator-sanitize-checks");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-validator-sanitize-checks");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  int64_t ok_email = sec4_rt_validate_email(sec4_rt_req_query("user@example.com"));
  if (ok_email == 0) { return 11; }

  sec4_rt_reset_response();
  if (sec4_rt_validate_email(sec4_rt_req_query("not-an-email")) != 0) { return 12; }
  if (!g_sec4_rt_response.active) { return 13; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"VALIDATE.EMAIL_INVALID\"") == NULL) { return 14; }

  sec4_rt_reset_response();
  if (sec4_rt_validate_email(INT64_C(444444)) != 0) { return 15; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"VALIDATE.EMAIL_INVALID\"") == NULL) { return 16; }

  int64_t ok_uuid = sec4_rt_validate_uuid(sec4_rt_req_query("550e8400-e29b-41d4-a716-446655440000"));
  if (ok_uuid == 0) { return 17; }

  sec4_rt_reset_response();
  if (sec4_rt_validate_uuid(sec4_rt_req_query("550e8400e29b41d4a716446655440000")) != 0) { return 18; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"VALIDATE.UUID_INVALID\"") == NULL) { return 19; }

  int64_t ok_int64 = sec4_rt_validate_int64(sec4_rt_req_query("-9223372036854775808"));
  if (ok_int64 == 0) { return 20; }

  sec4_rt_reset_response();
  if (sec4_rt_validate_int64(sec4_rt_req_query("9223372036854775808")) != 0) { return 21; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"VALIDATE.INT64_INVALID\"") == NULL) { return 22; }

  int64_t ok_non_empty = sec4_rt_validate_non_empty(sec4_rt_req_query("x"));
  if (ok_non_empty == 0) { return 23; }

  int64_t empty_value = sec4_rt_track_sized_value("", 0, UINT64_C(0xEEEEE));
  if (empty_value == 0) { return 24; }
  sec4_rt_reset_response();
  if (sec4_rt_validate_non_empty(empty_value) != 0) { return 25; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"VALIDATE.NON_EMPTY_INVALID\"") == NULL) { return 26; }

  int64_t sanitized = sec4_rt_sanitize_html(sec4_rt_req_query("<a&\"'>"));
  if (sanitized == 0) { return 27; }
  const char *sanitized_value = sec4_rt_lookup_tracked_value(sanitized);
  if (sanitized_value == NULL) { return 28; }
  if (strcmp(sanitized_value, "&lt;a&amp;&quot;&#39;&gt;") != 0) { return 29; }

  sec4_rt_reset_response();
  if (sec4_rt_sanitize_html(INT64_C(999999)) != 0) { return 30; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"SANITIZE.HTML_INVALID\"") == NULL) { return 31; }

  sec4_rt_reset_response();
  if (setenv("SEC4_RT_JSON_MAX_BYTES", "16", 1) != 0) { return 32; }
  int64_t decode_size_raw = sec4_rt_track_string_value("{\"blob\":\"12345678901234567890\"}", UINT64_C(0xFA001));
  if (decode_size_raw == 0) { return 33; }
  if (sec4_rt_json_decode(INT64_C(1), INT64_C(2), decode_size_raw) != 0) { return 34; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"JSON.SIZE_LIMIT\"") == NULL) { return 35; }

  sec4_rt_reset_response();
  if (setenv("SEC4_RT_JSON_MAX_DEPTH", "2", 1) != 0) { return 36; }
  if (setenv("SEC4_RT_JSON_MAX_BYTES", "256", 1) != 0) { return 45; }
  int64_t decode_depth_raw = sec4_rt_track_string_value("{\"a\":{\"b\":{\"c\":1}}}", UINT64_C(0xFA002));
  if (decode_depth_raw == 0) { return 37; }
  if (sec4_rt_json_decode(INT64_C(1), INT64_C(2), decode_depth_raw) != 0) { return 38; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"JSON.DEPTH_LIMIT\"") == NULL) { return 39; }

  sec4_rt_reset_response();
  if (setenv("SEC4_RT_JSON_MAX_BYTES", "256", 1) != 0) { return 40; }
  if (setenv("SEC4_RT_JSON_MAX_DEPTH", "6", 1) != 0) { return 41; }
  int64_t decode_ok_raw = sec4_rt_track_string_value("{\"a\":{\"b\":1}}", UINT64_C(0xFA003));
  if (decode_ok_raw == 0) { return 42; }
  int64_t decode_ok = sec4_rt_json_decode(INT64_C(1), INT64_C(2), decode_ok_raw);
  if (decode_ok == 0) { return 43; }
  const char *decode_ok_value = sec4_rt_lookup_tracked_value(decode_ok);
  if (decode_ok_value == NULL || strcmp(decode_ok_value, "{\"a\":{\"b\":1}}") != 0) { return 44; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime validator/sanitizer harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime validator/sanitizer harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_path_under_enforces_base_containment_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime path-under containment test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-path-under-containment");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-path-under-containment");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  int64_t base = sec4_rt_path_base("/tmp/safe//");
  if (base == 0) { return 11; }
  const char *base_value = sec4_rt_lookup_tracked_value(base);
  if (base_value == NULL || strcmp(base_value, "/tmp/safe") != 0) { return 12; }

  int64_t safe_rel = sec4_rt_path_under(base, sec4_rt_req_query("docs/readme.txt"));
  if (safe_rel == 0) { return 13; }
  const char *safe_rel_value = sec4_rt_lookup_tracked_value(safe_rel);
  if (safe_rel_value == NULL || strcmp(safe_rel_value, "/tmp/safe/docs/readme.txt") != 0) { return 14; }

  int64_t safe_abs = sec4_rt_path_under(base, sec4_rt_req_query("/tmp/safe/data/file.txt"));
  if (safe_abs == 0) { return 15; }

  sec4_rt_reset_response();
  if (sec4_rt_path_under(base, sec4_rt_req_query("../etc/passwd")) != 0) { return 16; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"PATH.UNDER_PATH_INVALID\"") == NULL) { return 17; }

  sec4_rt_reset_response();
  if (sec4_rt_path_under(base, sec4_rt_req_query("/etc/passwd")) != 0) { return 18; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"PATH.UNDER_OUT_OF_BASE\"") == NULL) { return 19; }

  sec4_rt_reset_response();
  if (sec4_rt_path_under(INT64_C(1234567), sec4_rt_req_query("ok.txt")) != 0) { return 20; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"PATH.UNDER_INVALID\"") == NULL) { return 21; }

  sec4_rt_reset_response();
  if (sec4_rt_path_base("tmp/base") != 0) { return 22; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"PATH.BASE_INVALID\"") == NULL) { return 23; }

  sec4_rt_reset_response();
  if (sec4_rt_path_base("/tmp/../escape") != 0) { return 24; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"PATH.BASE_INVALID\"") == NULL) { return 25; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime path-under containment harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime path-under containment harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_url_guards_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime url guard test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-url-guards");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-url-guards");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"

int main(void) {
  int64_t public_a = sec4_rt_url_public(sec4_rt_req_query("https://public-a.example/path"));
  int64_t public_b = sec4_rt_url_public(sec4_rt_req_query("https://public-b.example/path"));
  if (public_a == 0 || public_b == 0) { return 11; }
  if (public_a == public_b) { return 12; }

  int64_t blocked_public_a = sec4_rt_url_public(sec4_rt_req_query("http://127.0.0.1/private-a"));
  int64_t blocked_public_b = sec4_rt_url_public(sec4_rt_req_query("http://10.0.0.2/private-b"));
  if (blocked_public_a != 0 || blocked_public_b != 0) { return 13; }

  int64_t internal_a = sec4_rt_url_internal(sec4_rt_req_query("http://127.0.0.1/service-a"));
  int64_t internal_b = sec4_rt_url_internal(sec4_rt_req_query("http://10.0.0.4/service-b"));
  if (internal_a == 0 || internal_b == 0) { return 14; }
  if (internal_a == internal_b) { return 15; }

  int64_t blocked_internal = sec4_rt_url_internal(sec4_rt_req_query("https://public.example/path"));
  if (blocked_internal != 0) { return 16; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime url harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime url guard harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_url_ipv6_literal_diagnostics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime url ipv6-literal diagnostics test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-url-ipv6-literal-diagnostics");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-url-ipv6-literal-diagnostics");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES", "http,https", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_PORTS", "", 1);
  setenv("SEC4_RT_NET_SSRF_RESOLVE_DNS", "0", 1);
  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS", "fd00::/8", 1);

  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://[fd00::1/path")) != 0) { return 11; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_IPV6_BRACKET_MISSING\"") == NULL) { return 12; }

  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://[]/path")) != 0) { return 13; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_IPV6_EMPTY_LITERAL\"") == NULL) { return 14; }

  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://[zzzz::1]/path")) != 0) { return 15; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_IPV6_LITERAL_INVALID\"") == NULL) { return 16; }

  sec4_rt_reset_response();
  if (sec4_rt_url_internal(sec4_rt_req_query("http://[fd00::1/path")) != 0) { return 17; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_IPV6_BRACKET_MISSING\"") == NULL) { return 18; }

  sec4_rt_reset_response();
  if (sec4_rt_url_internal(sec4_rt_req_query("http://[]/path")) != 0) { return 19; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_IPV6_EMPTY_LITERAL\"") == NULL) { return 20; }

  sec4_rt_reset_response();
  if (sec4_rt_url_internal(sec4_rt_req_query("http://[zzzz::1]/path")) != 0) { return 21; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_IPV6_LITERAL_INVALID\"") == NULL) { return 22; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime url ipv6-literal diagnostics harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime url ipv6-literal diagnostics harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_url_public_respects_env_policy_lists_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime url policy list test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-url-policy-lists");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-url-policy-lists");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES", "https", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_PORTS", "", 1);

  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES", "ftp", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("https://public.example/path")) != 0) { return 31; }
  if (!g_sec4_rt_response.active) { return 32; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_POLICY_ALLOWED_SCHEMES_INVALID\"") == NULL) { return 33; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES\"") == NULL) { return 34; }
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES", "https", 1);

  setenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS", "bad domain", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("https://public.example/path")) != 0) { return 35; }
  if (!g_sec4_rt_response.active) { return 36; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_POLICY_BLOCKED_DOMAINS_INVALID\"") == NULL) { return 37; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS\"") == NULL) { return 38; }
  setenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS", "", 1);

  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS", "bad..domain", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("https://public.example/path")) != 0) { return 39; }
  if (!g_sec4_rt_response.active) { return 40; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_POLICY_ALLOWED_DOMAINS_INVALID\"") == NULL) { return 41; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS\"") == NULL) { return 42; }
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS", "", 1);

  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_PORTS", "443,abc", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("https://public.example/path")) != 0) { return 43; }
  if (!g_sec4_rt_response.active) { return 44; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_POLICY_ALLOWED_PORTS_INVALID\"") == NULL) { return 45; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_PUBLIC_ALLOWED_PORTS\"") == NULL) { return 46; }
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_PORTS", "", 1);

  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://public.example/path")) != 0) { return 11; }
  if (!g_sec4_rt_response.active) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_INVALID\"") == NULL) { return 13; }

  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES", "https,http", 1);
  setenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS", "blocked.example", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("https://blocked.example/path")) != 0) { return 14; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_INVALID\"") == NULL) { return 15; }

  setenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS", "allowed.example", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("https://other.example/path")) != 0) { return 16; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_INVALID\"") == NULL) { return 17; }

  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("https://allowed.example/path")) == 0) { return 18; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime url policy-list harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime url policy-list harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_url_public_resolve_dns_policy_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime url resolve-dns policy-invalid test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-url-resolve-dns-policy-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-url-resolve-dns-policy-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES", "http,https", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_PORTS", "", 1);
  setenv("SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES", "1", 1);
  setenv("SEC4_RT_NET_SSRF_BLOCK_LOOPBACK", "1", 1);
  setenv("SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL", "1", 1);
  setenv("SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS", "1", 1);
  setenv("SEC4_RT_NET_SSRF_RESOLVE_DNS", "invalid", 1);

  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://public.example/path")) != 0) { return 11; }
  if (!g_sec4_rt_response.active) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.SSRF_POLICY_RESOLVE_DNS_INVALID\"") == NULL) { return 13; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_SSRF_RESOLVE_DNS\"") == NULL) { return 14; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime url resolve-dns policy-invalid harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime url resolve-dns policy-invalid harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_url_public_dns_resolution_toggle_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime url dns toggle test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-url-dns-toggle");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-url-dns-toggle");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  struct addrinfo hints;
  memset(&hints, 0, sizeof(hints));
  hints.ai_family = AF_UNSPEC;
  hints.ai_socktype = SOCK_STREAM;
  struct addrinfo *addresses = NULL;
  int resolve_rc = getaddrinfo("localhost.", "80", &hints, &addresses);
  if (resolve_rc != 0 || addresses == NULL) {
    if (addresses != NULL) {
      freeaddrinfo(addresses);
    }
    return 0;
  }
  freeaddrinfo(addresses);

  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES", "http,https", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_PORTS", "", 1);

  setenv("SEC4_RT_NET_SSRF_RESOLVE_DNS", "1", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://localhost./dns-check")) != 0) { return 11; }
  if (!g_sec4_rt_response.active) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_INVALID\"") == NULL) { return 13; }

  setenv("SEC4_RT_NET_SSRF_RESOLVE_DNS", "0", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://localhost./dns-check")) == 0) { return 14; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime url dns-toggle harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime url dns-toggle harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_url_public_respects_ssrf_block_toggles_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime url ssrf block-toggle test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-url-ssrf-block-toggles");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-url-ssrf-block-toggles");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES", "http,https", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_PORTS", "", 1);
  setenv("SEC4_RT_NET_SSRF_RESOLVE_DNS", "0", 1);

  setenv("SEC4_RT_NET_SSRF_BLOCK_LOOPBACK", "maybe", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://public.example/policy")) != 0) { return 9; }
  if (!g_sec4_rt_response.active) { return 10; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.SSRF_POLICY_BLOCK_LOOPBACK_INVALID\"") == NULL) { return 19; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_SSRF_BLOCK_LOOPBACK\"") == NULL) { return 20; }
  setenv("SEC4_RT_NET_SSRF_BLOCK_LOOPBACK", "1", 1);

  setenv("SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES", "bad", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://public.example/policy")) != 0) { return 21; }
  if (!g_sec4_rt_response.active) { return 22; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.SSRF_POLICY_BLOCK_PRIVATE_RANGES_INVALID\"") == NULL) { return 23; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES\"") == NULL) { return 24; }
  setenv("SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES", "1", 1);

  setenv("SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL", "bad", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://public.example/policy")) != 0) { return 25; }
  if (!g_sec4_rt_response.active) { return 26; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.SSRF_POLICY_BLOCK_LINK_LOCAL_INVALID\"") == NULL) { return 27; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL\"") == NULL) { return 28; }
  setenv("SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL", "1", 1);

  setenv("SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS", "bad", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://public.example/policy")) != 0) { return 29; }
  if (!g_sec4_rt_response.active) { return 30; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.SSRF_POLICY_BLOCK_METADATA_IPS_INVALID\"") == NULL) { return 31; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS\"") == NULL) { return 32; }
  setenv("SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS", "1", 1);

  setenv("SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES", "1", 1);
  setenv("SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL", "1", 1);
  setenv("SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS", "1", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://127.0.0.1/private")) != 0) { return 11; }
  if (!g_sec4_rt_response.active) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_INVALID\"") == NULL) { return 13; }

  setenv("SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES", "0", 1);
  setenv("SEC4_RT_NET_SSRF_BLOCK_LOOPBACK", "0", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://127.0.0.1/private")) == 0) { return 14; }

  setenv("SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL", "0", 1);
  setenv("SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS", "1", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://169.254.169.254/meta")) != 0) { return 15; }
  if (!g_sec4_rt_response.active) { return 16; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_INVALID\"") == NULL) { return 17; }

  setenv("SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS", "0", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://169.254.169.254/meta")) == 0) { return 18; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime url ssrf block-toggle harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime url ssrf block-toggle harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_url_public_enforces_allowed_ports_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime url public port policy test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-url-public-port-policy");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-url-public-port-policy");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES", "https,http", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_PUBLIC_ALLOWED_PORTS", "443, 8080", 1);

  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("https://public.example:8443/path")) != 0) { return 11; }
  if (!g_sec4_rt_response.active) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_INVALID\"") == NULL) { return 13; }

  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://public.example/path")) != 0) { return 14; }
  if (!g_sec4_rt_response.active) { return 15; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_INVALID\"") == NULL) { return 16; }

  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("https://public.example/path")) == 0) { return 17; }

  sec4_rt_reset_response();
  if (sec4_rt_url_public(sec4_rt_req_query("http://public.example:8080/path")) == 0) { return 18; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime url public port-policy harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime url public port-policy harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_url_internal_respects_allowed_cidrs_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal cidr test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-cidr-policy");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-cidr-policy");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_DOMAINS", "internal.service", 1);
  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS", "127.0.0.0/8", 1);

  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_DOMAINS", "bad domain", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_internal(sec4_rt_req_query("http://127.0.0.1/service")) != 0) { return 21; }
  if (!g_sec4_rt_response.active) { return 22; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_POLICY_ALLOWED_DOMAINS_INVALID\"") == NULL) { return 23; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_INTERNAL_ALLOWED_DOMAINS\"") == NULL) { return 24; }
  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_DOMAINS", "internal.service", 1);

  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS", "127.0.0.0/x", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_internal(sec4_rt_req_query("http://127.0.0.1/service")) != 0) { return 25; }
  if (!g_sec4_rt_response.active) { return 26; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_POLICY_ALLOWED_CIDRS_INVALID\"") == NULL) { return 27; }
  if (strstr(g_sec4_rt_response.body, "\"policyKey\",\"value\":\"SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS\"") == NULL) { return 28; }
  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS", "127.0.0.0/8", 1);

  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_DOMAINS", "", 1);
  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS", "fd00::/8", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_internal(sec4_rt_req_query("http://[fd00::1]/service")) == 0) { return 31; }

  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS", "fd00:1::/64", 1);
  sec4_rt_reset_response();
  if (sec4_rt_url_internal(sec4_rt_req_query("http://[fd00:2::1]/service")) != 0) { return 32; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_INVALID\"") == NULL) { return 33; }
  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_DOMAINS", "internal.service", 1);
  setenv("SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS", "127.0.0.0/8", 1);

  sec4_rt_reset_response();
  if (sec4_rt_url_internal(sec4_rt_req_query("http://127.0.0.1/service")) == 0) { return 11; }

  sec4_rt_reset_response();
  if (sec4_rt_url_internal(sec4_rt_req_query("http://10.0.0.5/service")) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_INVALID\"") == NULL) { return 13; }

  sec4_rt_reset_response();
  if (sec4_rt_url_internal(sec4_rt_req_query("http://internal.service/path")) != 0) { return 14; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_INVALID\"") == NULL) { return 15; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime internal cidr harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal cidr harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_outbound_url_parser_supports_ipv6_literals_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime outbound url ipv6 parser test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-outbound-url-ipv6-parser");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-outbound-url-ipv6-parser");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  char host[SEC4_RT_MAX_OUTBOUND_HTTP_HOST_BYTES];
  uint16_t port = 0;
  const char *target_start = NULL;
  size_t target_len = 0;
  bool is_http = false;
  bool is_https = false;

  if (!sec4_rt_parse_outbound_http_url(
          "http://[fd00::1]:8080/health?x=1",
          host,
          sizeof(host),
          &port,
          &target_start,
          &target_len,
          &is_http,
          &is_https,
          NULL)) { return 11; }
  if (strcmp(host, "fd00::1") != 0) { return 12; }
  if (port != 8080) { return 13; }
  if (!is_http || is_https) { return 14; }
  if (target_start == NULL || target_len != strlen("/health?x=1")) { return 15; }
  if (strncmp(target_start, "/health?x=1", target_len) != 0) { return 16; }

  if (sec4_rt_parse_outbound_http_url(
          "http://[fd00::1/health",
          host,
          sizeof(host),
          &port,
          &target_start,
          &target_len,
          &is_http,
          &is_https,
          NULL)) { return 17; }

  if (sec4_rt_parse_outbound_http_url(
          "http://[]/health",
          host,
          sizeof(host),
          &port,
          &target_start,
          &target_len,
          &is_http,
          &is_https,
          NULL)) { return 18; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime outbound url ipv6 parser harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime outbound url ipv6 parser harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_outbound_request_parser_ipv6_diagnostics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime outbound request parser ipv6 diagnostics test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-outbound-request-parser-ipv6-diagnostics");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-outbound-request-parser-ipv6-diagnostics");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  sec4_rt_reset_response();
  if (sec4_rt_outbound_http_get_handle("http://[fd00::1/path", UINT64_C(0x2A01), SEC4_RT_NET_SCOPE_INTERNAL) != 0) { return 11; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_IPV6_BRACKET_MISSING\"") == NULL) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"ipv6\"}]") == NULL) { return 13; }

  sec4_rt_reset_response();
  if (sec4_rt_outbound_http_get_handle("http://[]/path", UINT64_C(0x2A02), SEC4_RT_NET_SCOPE_INTERNAL) != 0) { return 14; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_IPV6_EMPTY_LITERAL\"") == NULL) { return 15; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"ipv6\"}]") == NULL) { return 16; }

  sec4_rt_reset_response();
  if (sec4_rt_outbound_http_get_handle("http://[zzzz::1]/path", UINT64_C(0x2A03), SEC4_RT_NET_SCOPE_INTERNAL) != 0) { return 17; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_IPV6_LITERAL_INVALID\"") == NULL) { return 18; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"ipv6\"}]") == NULL) { return 19; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime outbound request parser ipv6 diagnostics harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime outbound request parser ipv6 diagnostics harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime outbound request parser fallback diagnostics test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-outbound-request-parser-fallback-diagnostics");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-outbound-request-parser-fallback-diagnostics");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  sec4_rt_reset_response();
  if (sec4_rt_outbound_http_get_handle("127.0.0.1/path", UINT64_C(0x2B01), SEC4_RT_NET_SCOPE_INTERNAL) != 0) { return 11; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_SCHEME_MISSING\"") == NULL) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"scheme\"}]") == NULL) { return 13; }

  sec4_rt_reset_response();
  if (sec4_rt_outbound_http_get_handle("ftp://127.0.0.1/path", UINT64_C(0x2B02), SEC4_RT_NET_SCOPE_INTERNAL) != 0) { return 14; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_SCHEME_INVALID\"") == NULL) { return 15; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"scheme\"}]") == NULL) { return 16; }

  sec4_rt_reset_response();
  if (sec4_rt_outbound_http_get_handle("http://bad host/path", UINT64_C(0x2B03), SEC4_RT_NET_SCOPE_INTERNAL) != 0) { return 17; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_HOST_INVALID\"") == NULL) { return 18; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"host\"}]") == NULL) { return 19; }

  sec4_rt_reset_response();
  if (sec4_rt_outbound_http_get_handle("http://127.0.0.1:abc/path", UINT64_C(0x2B04), SEC4_RT_NET_SCOPE_INTERNAL) != 0) { return 20; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_PORT_INVALID\"") == NULL) { return 21; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"port\"}]") == NULL) { return 22; }

  sec4_rt_reset_response();
  if (sec4_rt_outbound_http_get_handle("http://127.0.0.1/path#frag", UINT64_C(0x2B05), SEC4_RT_NET_SCOPE_INTERNAL) != 0) { return 23; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_TARGET_INVALID\"") == NULL) { return 24; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"target\"}]") == NULL) { return 25; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime outbound request parser fallback diagnostics harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime outbound request parser fallback diagnostics harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_outbound_request_parser_unknown_fallback_is_deterministic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime outbound request parser unknown fallback test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-outbound-request-parser-unknown-fallback");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-outbound-request-parser-unknown-fallback");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  sec4_rt_reset_response();
  if (!sec4_rt_store_request_parse_error_from_outbound_parse_status(SEC4_RT_OUTBOUND_URL_PARSE_INVALID)) { return 11; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_PARSE_INVALID\"") == NULL) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"unknown\"}]") == NULL) { return 13; }
  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime outbound request parser unknown fallback harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime outbound request parser unknown fallback harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_redirect_request_parser_unknown_fallback_is_deterministic_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect request parser unknown fallback test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-redirect-request-parser-unknown-fallback");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-redirect-request-parser-unknown-fallback");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  if (sec4_rt_redirect_status_from_outbound_parse_status(SEC4_RT_OUTBOUND_URL_PARSE_INVALID)
      != SEC4_RT_REDIRECT_RESOLVE_REQUEST_PARSE_INVALID) { return 11; }
  sec4_rt_reset_response();
  if (!sec4_rt_store_request_parse_error_from_redirect_resolve_status(
          SEC4_RT_REDIRECT_RESOLVE_REQUEST_PARSE_INVALID)) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_PARSE_INVALID\"") == NULL) { return 13; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"unknown\"}]") == NULL) { return 14; }
  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime redirect request parser unknown fallback harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect request parser unknown fallback harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_request_parser_diagnostics_use_unified_details_order_when_clang_available() {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime request parser unified details-order test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-request-parser-unified-details-order");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-request-parser-unified-details-order");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

static int assert_diag(
    sec4_rt_outbound_url_parse_status status,
    const char *expected_code,
    const char *expected_component
) {
  sec4_rt_reset_response();
  if (!sec4_rt_store_request_parse_error_from_outbound_parse_status(status)) { return 1; }
  if (strstr(g_sec4_rt_response.body, expected_code) == NULL) { return 2; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"phase\",\"value\":\"parse\"},{\"key\":\"component\",\"value\":\"") == NULL) { return 3; }
  if (strstr(g_sec4_rt_response.body, expected_component) == NULL) { return 4; }
  return 0;
}

int main(void) {
  if (assert_diag(SEC4_RT_OUTBOUND_URL_PARSE_INVALID, "\"code\":\"NET.REQUEST_PARSE_INVALID\"", "\"component\",\"value\":\"unknown\"") != 0) { return 11; }
  if (assert_diag(SEC4_RT_OUTBOUND_URL_PARSE_SCHEME_INVALID, "\"code\":\"NET.REQUEST_SCHEME_INVALID\"", "\"component\",\"value\":\"scheme\"") != 0) { return 12; }
  if (assert_diag(SEC4_RT_OUTBOUND_URL_PARSE_HOST_INVALID, "\"code\":\"NET.REQUEST_HOST_INVALID\"", "\"component\",\"value\":\"host\"") != 0) { return 13; }
  if (assert_diag(SEC4_RT_OUTBOUND_URL_PARSE_PORT_INVALID, "\"code\":\"NET.REQUEST_PORT_INVALID\"", "\"component\",\"value\":\"port\"") != 0) { return 14; }
  if (assert_diag(SEC4_RT_OUTBOUND_URL_PARSE_TARGET_INVALID, "\"code\":\"NET.REQUEST_TARGET_INVALID\"", "\"component\",\"value\":\"target\"") != 0) { return 15; }
  if (assert_diag(SEC4_RT_OUTBOUND_URL_PARSE_IPV6_LITERAL_INVALID, "\"code\":\"NET.REQUEST_IPV6_LITERAL_INVALID\"", "\"component\",\"value\":\"ipv6\"") != 0) { return 16; }
  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime request parser unified details-order harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime request parser unified details-order harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_db_fs_net_intrinsics_produce_non_stub_handles_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime db/fs/net handle test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-db-fs-net-handles");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-db-fs-net-handles");
    let db_base = project_dir.join("db-base");
    let fs_base = project_dir.join("fs-base");
    fs::create_dir_all(&db_base).expect("db base should be created");
    fs::create_dir_all(&fs_base).expect("fs base should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_a_raw = getenv("SEC4_RT_TEST_INTERNAL_URL_A");
  const char *internal_url_b_raw = getenv("SEC4_RT_TEST_INTERNAL_URL_B");
  if (internal_url_a_raw == NULL || internal_url_b_raw == NULL) { return 10; }

  int64_t query_a = sec4_rt_sql_q("SELECT 1", 11);
  int64_t query_b = sec4_rt_sql_q("SELECT 2", 11);
  if (query_a == 0 || query_b == 0) { return 11; }
  if (query_a == query_b) { return 12; }
  if (sec4_rt_sql_q("", 11) != 0) { return 13; }

  int64_t tx = sec4_rt_db_tx(101);
  if (tx == 0) { return 14; }
  if (sec4_rt_db_tx(0) != 0) { return 15; }

  int64_t exec_a = sec4_rt_db_exec(101, query_a);
  int64_t exec_b = sec4_rt_db_exec(101, query_b);
  if (exec_a == 0 || exec_b == 0) { return 16; }
  if (exec_a == exec_b) { return 17; }
  if (sec4_rt_db_exec(0, query_a) != 0) { return 18; }

  int64_t exec_tx = sec4_rt_db_exec_tx(tx, query_a);
  if (exec_tx == 0) { return 19; }
  if (sec4_rt_db_exec_tx(0, query_a) != 0) { return 20; }

  int64_t row_a = sec4_rt_db_query_one(101, query_a, 501);
  int64_t row_b = sec4_rt_db_query_one(101, query_a, 502);
  if (row_a == 0 || row_b == 0) { return 21; }
  if (row_a == row_b) { return 22; }
  if (sec4_rt_db_query_one(0, query_a, 501) != 0) { return 23; }

  int64_t fs_path_a = sec4_rt_req_query("db-fs-net/a.txt");
  int64_t fs_path_b = sec4_rt_req_query("db-fs-net/b.txt");
  int64_t fs_value_a = sec4_rt_req_query("alpha");
  int64_t fs_value_b = sec4_rt_req_query("beta");
  if (fs_path_a == 0 || fs_path_b == 0 || fs_value_a == 0 || fs_value_b == 0) { return 24; }

  int64_t fs_write_a = sec4_rt_fs_write(7, fs_path_a, fs_value_a);
  int64_t fs_write_b = sec4_rt_fs_write(7, fs_path_b, fs_value_b);
  if (fs_write_a == 0 || fs_write_b == 0) { return 25; }
  if (fs_write_a == fs_write_b) { return 26; }

  int64_t fs_read_a = sec4_rt_fs_read(7, fs_path_a);
  int64_t fs_read_b = sec4_rt_fs_read(7, fs_path_b);
  if (fs_read_a == 0 || fs_read_b == 0) { return 27; }
  if (fs_read_a == fs_read_b) { return 28; }
  if (sec4_rt_fs_read(0, fs_path_a) != 0) { return 29; }
  if (sec4_rt_fs_read(7, 2001) != 0) { return 30; }

  int64_t internal_url_a = sec4_rt_req_query(internal_url_a_raw);
  int64_t internal_url_b = sec4_rt_req_query(internal_url_b_raw);
  int64_t net_a = sec4_rt_http_get_internal(3, internal_url_a);
  int64_t net_b = sec4_rt_http_get_internal(3, internal_url_b);
  if (net_a == 0 || net_b == 0) { return 31; }
  if (net_a == net_b) { return 32; }
  if (sec4_rt_http_get_internal(0, internal_url_a) != 0) { return 33; }
  if (sec4_rt_http_get_internal(3, 2001) != 0) { return 34; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime db/fs/net harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let (internal_port_a, server_a) = spawn_one_shot_http_server("net-body-a");
    let (internal_port_b, server_b) = spawn_one_shot_http_server("net-body-b");
    let internal_url_a = format!("http://127.0.0.1:{internal_port_a}/db-fs-net-a");
    let internal_url_b = format!("http://127.0.0.1:{internal_port_b}/db-fs-net-b");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_DB_BASE", &db_base)
        .env("SEC4_RT_FS_BASE", &fs_base)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL_A", &internal_url_a)
        .env("SEC4_RT_TEST_INTERNAL_URL_B", &internal_url_b)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime db/fs/net harness should exit successfully"
    );
    server_a
        .join()
        .expect("first runtime net test server should exit cleanly");
    server_b
        .join()
        .expect("second runtime net test server should exit cleanly");

    assert_eq!(
        fs::read_to_string(fs_base.join("db-fs-net").join("a.txt"))
            .expect("first fs file should be readable"),
        "alpha"
    );
    assert_eq!(
        fs::read_to_string(fs_base.join("db-fs-net").join("b.txt"))
            .expect("second fs file should be readable"),
        "beta"
    );
}

#[test]
fn c_bin_runtime_db_exec_query_one_roundtrip_returns_tracked_body_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime db roundtrip test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-db-roundtrip");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-db-roundtrip");
    let db_base = project_dir.join("db-base");
    fs::create_dir_all(&db_base).expect("db base should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"

int main(void) {
  int64_t query = sec4_rt_sql_q("SELECT roundtrip", 41);
  if (query == 0) { return 10; }

  int64_t exec_handle = sec4_rt_db_exec(9001, query);
  if (exec_handle == 0) { return 11; }

  int64_t row = sec4_rt_db_query_one(9001, query, 7001);
  if (row == 0) { return 12; }

  int64_t redacted = sec4_rt_secret_redact(row);
  if (redacted == 0) { return 13; }

  int64_t row_again = sec4_rt_db_query_one(9001, query, 7001);
  if (row_again == 0) { return 14; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime db roundtrip harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .env("SEC4_RT_DB_BASE", &db_base)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime db roundtrip harness should exit successfully"
    );

    let records_path = db_base.join("records.log");
    let records = fs::read_to_string(&records_path).expect("db records log should be readable");
    assert!(
        records.contains("v1|9001|"),
        "db records log should include deterministic db/query prefix:\n{records}"
    );
}

#[test]
fn c_bin_runtime_db_query_one_missing_record_returns_error_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime db missing-record test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-db-missing-record");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-db-missing-record");
    let db_base = project_dir.join("db-base");
    fs::create_dir_all(&db_base).expect("db base should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"

int main(void) {
  int64_t query = sec4_rt_sql_q("SELECT missing", 77);
  if (query == 0) { return 10; }

  if (sec4_rt_db_query_one(404, query, 7001) != 0) { return 11; }
  if (sec4_rt_db_query_one(404, query, 7001) != 0) { return 12; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime db missing-record harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .env("SEC4_RT_DB_BASE", &db_base)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime db missing-record harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_fs_write_read_roundtrip_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime fs roundtrip test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-fs-roundtrip");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-fs-roundtrip");
    let fs_base = project_dir.join("fs-base");
    fs::create_dir_all(&fs_base).expect("fs base should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"

int main(void) {
  int64_t path = sec4_rt_req_query("nested/roundtrip.txt");
  int64_t mirror = sec4_rt_req_query("nested/mirror.txt");
  int64_t value = sec4_rt_req_query("hello-runtime-fs");
  if (path == 0 || mirror == 0 || value == 0) { return 11; }

  if (sec4_rt_fs_write(7, path, value) == 0) { return 12; }
  int64_t read_handle = sec4_rt_fs_read(7, path);
  if (read_handle == 0) { return 13; }
  if (sec4_rt_fs_write(7, mirror, read_handle) == 0) { return 14; }
  int64_t mirror_handle = sec4_rt_fs_read(7, mirror);
  if (mirror_handle == 0) { return 15; }
  if (mirror_handle != read_handle) { return 16; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime fs roundtrip harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_FS_BASE", &fs_base)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime fs roundtrip harness should exit successfully"
    );

    assert_eq!(
        fs::read_to_string(fs_base.join("nested").join("roundtrip.txt"))
            .expect("roundtrip file should be readable"),
        "hello-runtime-fs"
    );
    assert_eq!(
        fs::read_to_string(fs_base.join("nested").join("mirror.txt"))
            .expect("mirror file should be readable"),
        "hello-runtime-fs"
    );
}

#[test]
fn c_bin_runtime_fs_traversal_and_out_of_base_are_denied_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime fs deny test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-fs-deny");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-fs-deny");
    let fs_base = project_dir.join("fs-base");
    fs::create_dir_all(&fs_base).expect("fs base should be created");
    let outside_path = project_dir.join("outside-target.txt");
    let outside_path_rendered = outside_path.to_string_lossy();

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    let harness_source = format!(
        r#"#include "sec4_runtime.h"

int main(void) {{
  int64_t traversal = sec4_rt_req_query("../escape.txt");
  int64_t outside = sec4_rt_req_query("{outside_path}");
  int64_t safe = sec4_rt_req_query("safe/inside.txt");
  int64_t value = sec4_rt_req_query("deny-check");
  if (traversal == 0 || outside == 0 || safe == 0 || value == 0) {{ return 11; }}

  if (sec4_rt_fs_write(7, safe, value) == 0) {{ return 12; }}
  if (sec4_rt_fs_read(7, safe) == 0) {{ return 13; }}

  if (sec4_rt_fs_write(7, traversal, value) != 0) {{ return 14; }}
  if (sec4_rt_fs_read(7, traversal) != 0) {{ return 15; }}
  if (sec4_rt_fs_write(7, outside, value) != 0) {{ return 16; }}
  if (sec4_rt_fs_read(7, outside) != 0) {{ return 17; }}

  return 0;
}}
"#,
        outside_path = outside_path_rendered
    );
    fs::write(&harness_path, harness_source).expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime fs deny harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_FS_BASE", &fs_base)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime fs deny harness should exit successfully"
    );

    assert_eq!(
        fs::read_to_string(fs_base.join("safe").join("inside.txt"))
            .expect("safe file should be readable"),
        "deny-check"
    );
    assert!(
        !outside_path.exists(),
        "outside file should not be written outside fs base"
    );
    assert!(
        !project_dir.join("escape.txt").exists(),
        "traversal path should not materialize outside fs base"
    );
}

#[test]
fn c_bin_runtime_internal_net_denied_by_default_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net deny test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-deny");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-deny");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"

int main(void) {
  int64_t internal_url = sec4_rt_req_query("http://127.0.0.1/internal-service");
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime internal-net deny harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net deny harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_internal_get_roundtrip_succeeds_with_env_override_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net roundtrip test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-roundtrip");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-roundtrip");
    let fs_base = project_dir.join("fs-base");
    fs::create_dir_all(&fs_base).expect("fs base should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  int64_t body = sec4_rt_http_get_internal(1, internal_url);
  if (internal_url == 0 || body == 0) { return 11; }
  if (sec4_rt_http_get_internal(0, internal_url) != 0) { return 12; }

  int64_t output_path = sec4_rt_req_query("internal-net/body.txt");
  if (output_path == 0) { return 13; }
  if (sec4_rt_fs_write(5, output_path, body) == 0) { return 14; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime internal-net roundtrip harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let (internal_port, server_handle) = spawn_one_shot_http_server("internal-roundtrip-body");
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-service");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_FS_BASE", &fs_base)
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net roundtrip harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime internal-net roundtrip server should exit cleanly");

    assert_eq!(
        fs::read_to_string(fs_base.join("internal-net").join("body.txt"))
            .expect("internal-net response body should be written"),
        "internal-roundtrip-body"
    );
}

#[test]
fn c_bin_runtime_internal_get_ipv6_loopback_roundtrip_when_supported_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net ipv6 roundtrip test: clang not available");
        return;
    }

    let listener = match TcpListener::bind("[::1]:0") {
        Ok(listener) => listener,
        Err(_) => {
            eprintln!(
                "skipping c-bin runtime internal-net ipv6 roundtrip test: ipv6 loopback unavailable"
            );
            return;
        }
    };
    let internal_port = listener
        .local_addr()
        .expect("ipv6 listener local addr should be available")
        .port();
    let server_handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request_buf = [0u8; 1024];
            let _ = stream.read(&mut request_buf);
            let response =
                "HTTP/1.1 200 OK\r\nContent-Length: 16\r\nConnection: close\r\n\r\ninternal-v6-body";
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    let project_dir = temp_dir("sec4-runtime-c-internal-net-ipv6-roundtrip");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-ipv6-roundtrip");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }
  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  int64_t body = sec4_rt_http_get_internal(1, internal_url);
  if (body == 0) { return 12; }
  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime internal-net ipv6 roundtrip harness should compile successfully"
    );

    let internal_url = format!("http://[::1]:{internal_port}/internal-v6");
    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net ipv6 roundtrip harness should exit successfully"
    );

    server_handle
        .join()
        .expect("runtime internal-net ipv6 roundtrip server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_ipv6_url_gate_diagnostics_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net ipv6 request diagnostics test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-ipv6-request-diagnostics");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-ipv6-request-diagnostics");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  setenv("SEC4_RT_ALLOW_INTERNAL_NET", "1", 1);

  sec4_rt_reset_response();
  if (sec4_rt_http_get_internal(1, sec4_rt_req_query("http://[fd00::1/path")) != 0) { return 11; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_IPV6_BRACKET_MISSING\"") == NULL) { return 12; }

  sec4_rt_reset_response();
  if (sec4_rt_http_get_internal(1, sec4_rt_req_query("http://[]/path")) != 0) { return 13; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_IPV6_EMPTY_LITERAL\"") == NULL) { return 14; }

  sec4_rt_reset_response();
  if (sec4_rt_http_get_internal(1, sec4_rt_req_query("http://[zzzz::1]/path")) != 0) { return 15; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_IPV6_LITERAL_INVALID\"") == NULL) { return 16; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime internal-net ipv6 request diagnostics harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net ipv6 request diagnostics harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_internal_get_chunked_body_is_decoded_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net chunked test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-chunked");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-chunked");
    let fs_base = project_dir.join("fs-base");
    fs::create_dir_all(&fs_base).expect("fs base should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  int64_t body = sec4_rt_http_get_internal(1, internal_url);
  if (internal_url == 0 || body == 0) { return 11; }

  int64_t output_path = sec4_rt_req_query("internal-net/chunked-body.txt");
  if (output_path == 0) { return 12; }
  if (sec4_rt_fs_write(5, output_path, body) == 0) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime chunked harness");
    assert!(
        output.status.success(),
        "runtime internal-net chunked harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let (internal_port, server_handle) =
        spawn_one_shot_http_chunked_server("internal-chunked-body");
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-chunked");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_FS_BASE", &fs_base)
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net chunked harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime internal-net chunked server should exit cleanly");

    assert_eq!(
        fs::read_to_string(fs_base.join("internal-net").join("chunked-body.txt"))
            .expect("internal-net chunked response body should be written"),
        "internal-chunked-body"
    );
}

#[test]
fn c_bin_runtime_internal_get_malformed_chunked_returns_chunk_invalid_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime malformed chunked test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-malformed-chunked");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-malformed-chunked");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.CHUNK_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for malformed chunked harness");
    assert!(
        output.status.success(),
        "runtime malformed chunked harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let (internal_port, server_handle) = spawn_one_shot_http_malformed_chunked_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-chunked-malformed");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime malformed chunked harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime malformed chunked server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_chunked_trailers_roundtrip_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime chunked trailer roundtrip test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-chunked-trailers");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-chunked-trailers");
    let fs_base = project_dir.join("fs-base");
    fs::create_dir_all(&fs_base).expect("fs base should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  int64_t body = sec4_rt_http_get_internal(1, internal_url);
  if (internal_url == 0 || body == 0) { return 11; }

  int64_t output_path = sec4_rt_req_query("internal-net/chunked-trailers-body.txt");
  if (output_path == 0) { return 12; }
  if (sec4_rt_fs_write(5, output_path, body) == 0) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for chunked trailers harness");
    assert!(
        output.status.success(),
        "runtime chunked trailers harness should compile successfully"
    );

    let (internal_port, server_handle) =
        spawn_one_shot_http_chunked_with_trailers_server("internal-trailer-body");
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-chunked-trailers");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_FS_BASE", &fs_base)
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime chunked trailers harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime chunked trailers server should exit cleanly");

    assert_eq!(
        fs::read_to_string(fs_base.join("internal-net").join("chunked-trailers-body.txt"))
            .expect("chunked trailers decoded response should be written"),
        "internal-trailer-body"
    );
}

#[test]
fn c_bin_runtime_internal_get_chunked_missing_trailer_terminator_returns_chunk_invalid_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime chunked trailer terminator test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-chunked-missing-trailer-end");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-chunked-missing-trailer-end");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.CHUNK_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for chunked missing terminator harness");
    assert!(
        output.status.success(),
        "runtime chunked missing terminator harness should compile successfully"
    );

    let (internal_port, server_handle) =
        spawn_one_shot_http_chunked_missing_trailer_terminator_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-chunked-bad-trailers");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime chunked missing terminator harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime chunked missing terminator server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_oversized_headers_returns_deterministic_code_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime oversized-header test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-oversized-headers");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-oversized-headers");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.RESPONSE_HEADERS_TOO_LARGE\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for oversized-header harness");
    assert!(
        output.status.success(),
        "runtime oversized-header harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_large_headers_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-large-headers");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime oversized-header harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime oversized-header server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_conflicting_framing_headers_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime conflicting-framing test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-conflicting-framing");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-conflicting-framing");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.RESPONSE_FRAMING_CONFLICT\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for conflicting framing harness");
    assert!(
        output.status.success(),
        "runtime conflicting framing harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_conflicting_framing_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-framing-conflict");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime conflicting framing harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime conflicting framing server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_invalid_status_line_returns_deterministic_code_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime invalid status-line test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-invalid-status-line");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-invalid-status-line");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.STATUS_LINE_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for invalid status-line harness");
    assert!(
        output.status.success(),
        "runtime invalid status-line harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_status_line_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-invalid-status-line");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime invalid status-line harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime invalid status-line server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_invalid_transfer_encoding_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime invalid transfer-encoding test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-invalid-transfer-encoding");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-invalid-transfer-encoding");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.TRANSFER_ENCODING_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for invalid transfer-encoding harness");
    assert!(
        output.status.success(),
        "runtime invalid transfer-encoding harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_transfer_encoding_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-invalid-transfer");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime invalid transfer-encoding harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime invalid transfer-encoding server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_unsupported_transfer_encoding_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime unsupported transfer-encoding test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-unsupported-transfer-encoding");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-unsupported-transfer-encoding");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.TRANSFER_ENCODING_UNSUPPORTED\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for unsupported transfer-encoding harness");
    assert!(
        output.status.success(),
        "runtime unsupported transfer-encoding harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_unsupported_transfer_encoding_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-unsupported-transfer");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime unsupported transfer-encoding harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime unsupported transfer-encoding server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_obs_fold_header_returns_deterministic_code_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime obs-fold header test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-obs-fold-header");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-obs-fold-header");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.HEADER_OBS_FOLD_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for obs-fold header harness");
    assert!(
        output.status.success(),
        "runtime obs-fold header harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_obs_fold_header_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-obs-fold-header");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime obs-fold header harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime obs-fold header server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_header_whitespace_before_colon_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime header-whitespace test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-header-whitespace");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-header-whitespace");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.HEADER_WHITESPACE_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for header whitespace harness");
    assert!(
        output.status.success(),
        "runtime header whitespace harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_header_whitespace_before_colon_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-header-whitespace");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime header whitespace harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime header whitespace server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_invalid_header_section_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime invalid header-section test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-invalid-header-section");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-invalid-header-section");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.HEADER_SECTION_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for invalid header-section harness");
    assert!(
        output.status.success(),
        "runtime invalid header-section harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_header_section_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-invalid-header-section");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime invalid header-section harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime invalid header-section server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_header_value_control_char_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime header control-char test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-header-control-char");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-header-control-char");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.HEADER_VALUE_CONTROL_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for header control-char harness");
    assert!(
        output.status.success(),
        "runtime header control-char harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_header_control_char_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-header-control-char");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime header control-char harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime header control-char server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_invalid_content_type_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime invalid content-type test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-invalid-content-type");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-invalid-content-type");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.CONTENT_TYPE_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for invalid content-type harness");
    assert!(
        output.status.success(),
        "runtime invalid content-type harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_content_type_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-invalid-content-type");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime invalid content-type harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime invalid content-type server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_unsupported_version_returns_deterministic_code_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime unsupported-version test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-unsupported-version");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-unsupported-version");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.RESPONSE_VERSION_UNSUPPORTED\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for unsupported-version harness");
    assert!(
        output.status.success(),
        "runtime unsupported-version harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_unsupported_version_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-unsupported-version");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime unsupported-version harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime unsupported-version server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_invalid_header_line_returns_deterministic_code_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime invalid header-line test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-invalid-header-line");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-invalid-header-line");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.HEADER_LINE_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for invalid header-line harness");
    assert!(
        output.status.success(),
        "runtime invalid header-line harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_header_line_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-invalid-header-line");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime invalid header-line harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime invalid header-line server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_invalid_retry_after_returns_deterministic_code_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime invalid retry-after test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-invalid-retry-after");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-invalid-retry-after");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.RETRY_AFTER_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for invalid retry-after harness");
    assert!(
        output.status.success(),
        "runtime invalid retry-after harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_retry_after_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-invalid-retry-after");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime invalid retry-after harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime invalid retry-after server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_conflicting_location_headers_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime conflicting location-header test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-conflicting-location");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-conflicting-location");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_LOCATION_CONFLICT\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for conflicting location-header harness");
    assert!(
        output.status.success(),
        "runtime conflicting location-header harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_conflicting_location_headers_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-conflicting-location");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime conflicting location-header harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime conflicting location-header server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_duplicate_content_length_equal_is_accepted_when_clang_available() {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime duplicate content-length equal test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-duplicate-content-length-equal");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-duplicate-content-length-equal");
    let fs_base = project_dir.join("fs-base");
    fs::create_dir_all(&fs_base).expect("fs base dir should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  int64_t body = sec4_rt_http_get_internal(1, internal_url);
  if (body == 0) { return 12; }

  int64_t output_path = sec4_rt_req_query("internal-net/content-length-equal.txt");
  if (output_path == 0) { return 13; }
  if (sec4_rt_fs_write(1, output_path, body) == 0) { return 14; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for duplicate content-length equal harness");
    assert!(
        output.status.success(),
        "runtime duplicate content-length equal harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_duplicate_content_length_equal_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-content-length-equal");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_FS_BASE", &fs_base)
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime duplicate content-length equal harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime duplicate content-length equal server should exit cleanly");
    assert_eq!(
        fs::read_to_string(fs_base.join("internal-net").join("content-length-equal.txt"))
            .expect("duplicate content-length equal body should be written"),
        "test"
    );
}

#[test]
fn c_bin_runtime_internal_get_duplicate_content_length_conflict_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime duplicate content-length conflict test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-duplicate-content-length-conflict");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-duplicate-content-length-conflict");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.CONTENT_LENGTH_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for duplicate content-length conflict harness");
    assert!(
        output.status.success(),
        "runtime duplicate content-length conflict harness should compile successfully"
    );

    let (internal_port, server_handle) =
        spawn_one_shot_http_duplicate_content_length_conflict_server();
    let internal_url =
        format!("http://127.0.0.1:{internal_port}/internal-content-length-conflict");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime duplicate content-length conflict harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime duplicate content-length conflict server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_relative_redirect_is_normalized_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime relative redirect normalize test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-relative-redirect");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-relative-redirect");
    let fs_base = project_dir.join("fs-base");
    fs::create_dir_all(&fs_base).expect("fs base should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  int64_t body = sec4_rt_http_get_internal(1, internal_url);
  if (body == 0) { return 12; }

  int64_t output_path = sec4_rt_req_query("internal-net/redirect-relative-body.txt");
  if (output_path == 0) { return 13; }
  if (sec4_rt_fs_write(5, output_path, body) == 0) { return 14; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for relative redirect normalize harness");
    assert!(
        output.status.success(),
        "runtime relative redirect normalize harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_relative_redirect_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/svc/start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_FS_BASE", &fs_base)
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime relative redirect normalize harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime relative redirect normalize server should exit cleanly");
    assert_eq!(
        fs::read_to_string(fs_base.join("internal-net").join("redirect-relative-body.txt"))
            .expect("relative redirect response body should be written"),
        "internal-relative-body"
    );
}

#[test]
fn c_bin_runtime_internal_get_relative_redirect_invalid_target_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime invalid relative redirect test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-relative-redirect-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-relative-redirect-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_TARGET_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for invalid relative redirect harness");
    assert!(
        output.status.success(),
        "runtime invalid relative redirect harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_relative_redirect_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/svc/start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime invalid relative redirect harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime invalid relative redirect server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_redirect_fragment_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect fragment invalid test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-fragment-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-fragment-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_FRAGMENT_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect fragment invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect fragment invalid harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_fragment_redirect_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect fragment invalid harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime redirect fragment invalid server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_redirect_target_char_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect target-char invalid test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-target-char-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-target-char-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_TARGET_CHAR_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect target-char invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect target-char invalid harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_target_char_redirect_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect target-char invalid harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime redirect target-char invalid server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_redirect_query_percent_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect query-percent invalid test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-query-percent-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-query-percent-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_QUERY_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect query-percent invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect query-percent invalid harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_query_percent_redirect_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect query-percent invalid harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime redirect query-percent invalid server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_redirect_query_separator_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime redirect query-separator invalid test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-query-separator-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-query-separator-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_QUERY_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect query-separator invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect query-separator invalid harness should compile successfully"
    );

    let (internal_port, server_handle) =
        spawn_one_shot_http_invalid_query_separator_redirect_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect query-separator invalid harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime redirect query-separator invalid server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_redirect_scope_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect scope-invalid test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-scope-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-scope-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_SCOPE_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect scope-invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect scope-invalid harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_scope_invalid_redirect_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect scope-invalid harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime redirect scope-invalid server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_redirect_scope_revalidation_disabled_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime redirect scope-revalidation-disabled test: clang not available"
        );
        return;
    }

    let project_dir =
        temp_dir("sec4-runtime-c-internal-net-redirect-scope-revalidation-disabled");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-scope-revalidation-disabled");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_SCOPE_REVALIDATION_DISABLED\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect scope-revalidation-disabled harness");
    assert!(
        output.status.success(),
        "runtime redirect scope-revalidation-disabled harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_scope_invalid_redirect_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_NET_SSRF_REVALIDATE_REDIRECTS", "0")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect scope-revalidation-disabled harness should exit successfully"
    );
    server_handle.join().expect(
        "runtime redirect scope-revalidation-disabled server should exit cleanly",
    );
}

#[test]
fn c_bin_runtime_internal_get_redirect_location_missing_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime redirect missing-location test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-location-missing");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-location-missing");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_LOCATION_MISSING\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect missing-location harness");
    assert!(
        output.status.success(),
        "runtime redirect missing-location harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_missing_location_redirect_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect missing-location harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime redirect missing-location server should exit cleanly");
}

#[test]
fn c_bin_runtime_redirect_resolver_rejects_https_to_http_downgrade_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect downgrade resolver test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-redirect-downgrade-resolver");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-redirect-downgrade-resolver");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"

int main(void) {
  char resolved[SEC4_RT_MAX_OUTBOUND_HTTP_URL_BYTES];
  sec4_rt_redirect_resolve_status status = sec4_rt_resolve_redirect_url(
      "https://secure.example/start",
      "http://secure.example/downgrade",
      resolved,
      sizeof(resolved),
      false);
  if (status != SEC4_RT_REDIRECT_RESOLVE_DOWNGRADE_INVALID) { return 10; }
  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect downgrade resolver harness");
    assert!(
        output.status.success(),
        "runtime redirect downgrade resolver harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect downgrade resolver harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_redirect_resolver_allows_https_to_http_downgrade_when_enabled_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect downgrade-allow resolver test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-redirect-downgrade-allow-resolver");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-redirect-downgrade-allow-resolver");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  char resolved[SEC4_RT_MAX_OUTBOUND_HTTP_URL_BYTES];
  sec4_rt_redirect_resolve_status status = sec4_rt_resolve_redirect_url(
      "https://secure.example/start",
      "http://secure.example/downgrade",
      resolved,
      sizeof(resolved),
      true);
  if (status != SEC4_RT_REDIRECT_RESOLVE_OK) { return 10; }
  if (strcmp(resolved, "http://secure.example/downgrade") != 0) { return 11; }
  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect downgrade-allow resolver harness");
    assert!(
        output.status.success(),
        "runtime redirect downgrade-allow resolver harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect downgrade-allow resolver harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_internal_get_redirect_scheme_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect invalid-scheme test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-scheme-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-scheme-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_SCHEME_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect invalid-scheme harness");
    assert!(
        output.status.success(),
        "runtime redirect invalid-scheme harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_scheme_redirect_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect invalid-scheme harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime redirect invalid-scheme server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available() {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime redirect ipv6 request parser diagnostics test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-ipv6-request-diagnostics");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-ipv6-request-diagnostics");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }
  const char *expected_code = getenv("SEC4_RT_EXPECTED_CODE");
  if (expected_code == NULL || expected_code[0] == '\0') { return 11; }
  const char *expected_component = getenv("SEC4_RT_EXPECTED_COMPONENT");
  if (expected_component == NULL || expected_component[0] == '\0') { return 12; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 13; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 14; }
  if (strstr(g_sec4_rt_response.body, expected_code) == NULL) { return 15; }
  if (strstr(g_sec4_rt_response.body, "\"key\":\"phase\",\"value\":\"parse\"") == NULL) { return 16; }
  if (strstr(g_sec4_rt_response.body, expected_component) == NULL) { return 17; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect ipv6 request diagnostics harness");
    assert!(
        output.status.success(),
        "runtime redirect ipv6 request diagnostics harness should compile successfully"
    );

    let cases = [
        (
            "http://[fd00::1/path",
            "NET.REQUEST_IPV6_BRACKET_MISSING",
            "\"key\":\"component\",\"value\":\"ipv6\"",
        ),
        (
            "http://[]/path",
            "NET.REQUEST_IPV6_EMPTY_LITERAL",
            "\"key\":\"component\",\"value\":\"ipv6\"",
        ),
        (
            "http://[zzzz::1]/path",
            "NET.REQUEST_IPV6_LITERAL_INVALID",
            "\"key\":\"component\",\"value\":\"ipv6\"",
        ),
        (
            "http://bad host/path",
            "NET.REQUEST_HOST_INVALID",
            "\"key\":\"component\",\"value\":\"host\"",
        ),
        (
            "http://127.0.0.1:abc/path",
            "NET.REQUEST_PORT_INVALID",
            "\"key\":\"component\",\"value\":\"port\"",
        ),
        (
            "http://127.0.0.1/path#frag",
            "NET.REQUEST_TARGET_INVALID",
            "\"key\":\"component\",\"value\":\"target\"",
        ),
    ];
    for (redirect_location, expected_code, expected_component) in cases {
        let (internal_port, server_handle) =
            spawn_one_shot_http_ipv6_malformed_redirect_server(redirect_location);
        let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

        let run = Command::new(&binary_path)
            .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
            .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
            .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
            .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
            .env("SEC4_RT_EXPECTED_CODE", expected_code)
            .env("SEC4_RT_EXPECTED_COMPONENT", expected_component)
            .output()
            .expect("compiled binary should run");
        assert!(
            run.status.success(),
            "runtime redirect ipv6 request diagnostics harness should exit successfully for {expected_code}"
        );
        server_handle
            .join()
            .expect("runtime redirect ipv6 request diagnostics server should exit cleanly");
    }
}

#[test]
fn c_bin_runtime_public_get_redirect_request_parser_diagnostics_match_internal_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime public redirect request parser diagnostics parity test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-public-net-redirect-request-diagnostics");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-public-net-redirect-request-diagnostics");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *public_url_raw = getenv("SEC4_RT_TEST_PUBLIC_URL");
  if (public_url_raw == NULL) { return 10; }
  const char *expected_code = getenv("SEC4_RT_EXPECTED_CODE");
  if (expected_code == NULL || expected_code[0] == '\0') { return 11; }
  const char *expected_component = getenv("SEC4_RT_EXPECTED_COMPONENT");
  if (expected_component == NULL || expected_component[0] == '\0') { return 12; }

  int64_t public_url = sec4_rt_req_query(public_url_raw);
  if (public_url == 0) { return 13; }
  if (sec4_rt_http_get(1, public_url) != 0) { return 14; }
  if (strstr(g_sec4_rt_response.body, expected_code) == NULL) { return 15; }
  if (strstr(g_sec4_rt_response.body, "\"key\":\"phase\",\"value\":\"parse\"") == NULL) { return 16; }
  if (strstr(g_sec4_rt_response.body, expected_component) == NULL) { return 17; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for public redirect request diagnostics parity harness");
    assert!(
        output.status.success(),
        "runtime public redirect request diagnostics parity harness should compile successfully"
    );

    let cases = [
        (
            "http://[fd00::1/path",
            "NET.REQUEST_IPV6_BRACKET_MISSING",
            "\"key\":\"component\",\"value\":\"ipv6\"",
        ),
        (
            "http://bad host/path",
            "NET.REQUEST_HOST_INVALID",
            "\"key\":\"component\",\"value\":\"host\"",
        ),
        (
            "http://127.0.0.1:abc/path",
            "NET.REQUEST_PORT_INVALID",
            "\"key\":\"component\",\"value\":\"port\"",
        ),
        (
            "http://127.0.0.1/path#frag",
            "NET.REQUEST_TARGET_INVALID",
            "\"key\":\"component\",\"value\":\"target\"",
        ),
    ];
    for (redirect_location, expected_code, expected_component) in cases {
        let (public_port, server_handle) =
            spawn_one_shot_http_ipv6_malformed_redirect_server(redirect_location);
        let public_url = format!("http://127.0.0.1:{public_port}/internal-start");

        let run = Command::new(&binary_path)
            .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
            .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
            .env("SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES", "0")
            .env("SEC4_RT_NET_SSRF_BLOCK_LOOPBACK", "0")
            .env("SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL", "0")
            .env("SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS", "0")
            .env("SEC4_RT_NET_SSRF_RESOLVE_DNS", "0")
            .env("SEC4_RT_TEST_PUBLIC_URL", &public_url)
            .env("SEC4_RT_EXPECTED_CODE", expected_code)
            .env("SEC4_RT_EXPECTED_COMPONENT", expected_component)
            .output()
            .expect("compiled binary should run");
        assert!(
            run.status.success(),
            "runtime public redirect request diagnostics parity harness should exit successfully for {expected_code}"
        );
        server_handle
            .join()
            .expect("runtime public redirect request diagnostics parity server should exit cleanly");
    }
}

#[test]
fn c_bin_runtime_internal_get_redirect_allow_redirects_policy_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect policy-invalid test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-policy-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-policy-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  int64_t internal_url = sec4_rt_req_query("http://127.0.0.1:1/internal-start");
  if (internal_url == 0) { return 10; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 11; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_POLICY_ALLOW_REDIRECTS_INVALID\"") == NULL) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"policyKey\",\"value\":\"SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS\"}]") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect policy-invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect policy-invalid harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "maybe")
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect policy-invalid harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_internal_get_redirect_max_redirects_policy_range_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime redirect max-redirects policy-invalid test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-max-policy-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-max-policy-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  int64_t internal_url = sec4_rt_req_query("http://127.0.0.1:1/internal-start");
  if (internal_url == 0) { return 10; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 11; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_POLICY_MAX_REDIRECTS_RANGE_INVALID\"") == NULL) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"policyKey\",\"value\":\"SEC4_RT_NET_PUBLIC_MAX_REDIRECTS\"}]") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect max-redirects policy-invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect max-redirects policy-invalid harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "9007199254740999")
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect max-redirects policy-invalid harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_internal_get_redirect_max_redirects_policy_format_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime redirect max-redirects format-invalid test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-max-format-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-max-format-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  int64_t internal_url = sec4_rt_req_query("http://127.0.0.1:1/internal-start");
  if (internal_url == 0) { return 10; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 11; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_POLICY_MAX_REDIRECTS_FORMAT_INVALID\"") == NULL) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"policyKey\",\"value\":\"SEC4_RT_NET_PUBLIC_MAX_REDIRECTS\"}]") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect max-redirects format-invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect max-redirects format-invalid harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "abc")
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect max-redirects format-invalid harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_internal_get_redirect_revalidate_policy_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime redirect revalidate policy-invalid test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-revalidate-policy-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-revalidate-policy-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  int64_t internal_url = sec4_rt_req_query("http://127.0.0.1:1/internal-start");
  if (internal_url == 0) { return 10; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 11; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_POLICY_REVALIDATE_REDIRECTS_INVALID\"") == NULL) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"policyKey\",\"value\":\"SEC4_RT_NET_SSRF_REVALIDATE_REDIRECTS\"}]") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect revalidate policy-invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect revalidate policy-invalid harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_SSRF_REVALIDATE_REDIRECTS", "maybe")
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect revalidate policy-invalid harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_internal_get_redirect_allow_downgrade_policy_invalid_returns_deterministic_code_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime redirect allow-downgrade policy-invalid test: clang not available"
        );
        return;
    }

    let project_dir =
        temp_dir("sec4-runtime-c-internal-net-redirect-allow-downgrade-policy-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-allow-downgrade-policy-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  int64_t internal_url = sec4_rt_req_query("http://127.0.0.1:1/internal-start");
  if (internal_url == 0) { return 10; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 11; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_POLICY_ALLOW_DOWNGRADE_INVALID\"") == NULL) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"details\":[{\"key\":\"policyKey\",\"value\":\"SEC4_RT_NET_ALLOW_HTTPS_DOWNGRADE\"}]") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect allow-downgrade policy-invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect allow-downgrade policy-invalid harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_ALLOW_HTTPS_DOWNGRADE", "maybe")
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect allow-downgrade policy-invalid harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_internal_get_redirect_cycle_returns_deterministic_code_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect-cycle test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-cycle");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-cycle");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_CYCLE_DETECTED\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect-cycle harness");
    assert!(
        output.status.success(),
        "runtime redirect-cycle harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_redirect_cycle_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/cycle/start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "5")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect-cycle harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime redirect-cycle server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_absolute_redirect_upper_host_succeeds_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime absolute redirect upper-host test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-absolute-redirect-upper-host");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-absolute-redirect-upper-host");
    let fs_base = project_dir.join("fs-base");
    fs::create_dir_all(&fs_base).expect("fs base should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  int64_t body = sec4_rt_http_get_internal(1, internal_url);
  if (body == 0) { return 12; }

  int64_t output_path = sec4_rt_req_query("internal-net/redirect-absolute-body.txt");
  if (output_path == 0) { return 13; }
  if (sec4_rt_fs_write(5, output_path, body) == 0) { return 14; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for absolute redirect upper-host harness");
    assert!(
        output.status.success(),
        "runtime absolute redirect upper-host harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_absolute_redirect_upper_host_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/abs/start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_FS_BASE", &fs_base)
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime absolute redirect upper-host harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime absolute redirect upper-host server should exit cleanly");
    assert_eq!(
        fs::read_to_string(fs_base.join("internal-net").join("redirect-absolute-body.txt"))
            .expect("absolute redirect response body should be written"),
        "internal-absolute-body"
    );
}

#[test]
fn c_bin_runtime_internal_get_redirect_host_invalid_returns_deterministic_code_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime redirect host invalid test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-host-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-host-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_HOST_INVALID\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for redirect host-invalid harness");
    assert!(
        output.status.success(),
        "runtime redirect host-invalid harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_invalid_redirect_host_server();
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "3")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime redirect host-invalid harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime redirect host-invalid server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_redirect_denied_by_default_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net redirect deny test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-denied");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-denied");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_FORBIDDEN\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime redirect-deny harness");
    assert!(
        output.status.success(),
        "runtime internal-net redirect-deny harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let (internal_port, server_handle) = spawn_one_shot_http_redirect_chain_server(1);
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net redirect-deny harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime internal-net redirect-deny server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_redirect_allowed_with_env_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net redirect allow test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-allowed");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-allowed");
    let fs_base = project_dir.join("fs-base");
    fs::create_dir_all(&fs_base).expect("fs base should be created");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }

  int64_t body = sec4_rt_http_get_internal(1, internal_url);
  if (body == 0) { return 12; }

  int64_t output_path = sec4_rt_req_query("internal-net/redirect-body.txt");
  if (output_path == 0) { return 13; }
  if (sec4_rt_fs_write(5, output_path, body) == 0) { return 14; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime redirect-allow harness");
    assert!(
        output.status.success(),
        "runtime internal-net redirect-allow harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let (internal_port, server_handle) = spawn_one_shot_http_redirect_chain_server(3);
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "2")
        .env("SEC4_RT_FS_BASE", &fs_base)
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net redirect-allow harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime internal-net redirect-allow server should exit cleanly");

    assert_eq!(
        fs::read_to_string(fs_base.join("internal-net").join("redirect-body.txt"))
            .expect("redirect response body should be written"),
        "internal-redirect-body"
    );
}

#[test]
fn c_bin_runtime_internal_get_redirect_limit_exceeded_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net redirect limit test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-limit");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-limit");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_LIMIT\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime redirect-limit harness");
    assert!(
        output.status.success(),
        "runtime internal-net redirect-limit harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let (internal_port, server_handle) = spawn_one_shot_http_redirect_chain_server(2);
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net redirect-limit harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime internal-net redirect-limit server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_redirect_cap_limit_returns_deterministic_code_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net redirect cap-limit test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-redirect-cap-limit");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-redirect-cap-limit");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REDIRECT_CAP_LIMIT\"") == NULL) { return 13; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime redirect cap-limit harness");
    assert!(
        output.status.success(),
        "runtime internal-net redirect cap-limit harness should compile successfully"
    );

    let (internal_port, server_handle) = spawn_one_shot_http_monotonic_redirect_server(65);
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-start");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS", "1")
        .env("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", "200")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net redirect cap-limit harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime internal-net redirect cap-limit server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_https_self_signed_fails_tls_verify_without_insecure_override_when_openssl_backend_available()
{
    if !clang_with_openssl_available() {
        eprintln!(
            "skipping c-bin runtime internal-net https tls-verify failure test: clang/OpenSSL headers/libs unavailable"
        );
        return;
    }
    if !openssl_cli_available() {
        eprintln!(
            "skipping c-bin runtime internal-net https tls-verify failure test: openssl CLI unavailable"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-https-tls-verify-failure");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-https-tls-verify-failure");
    let cert_path = project_dir.join("localhost-cert.pem");
    let key_path = project_dir.join("localhost-key.pem");
    let runtime_port = find_available_tcp_port();

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        format!(
            r#"#include "sec4_runtime.h"
#include <stdlib.h>

static int64_t probe(void) {{
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) {{
    (void) sec4_rt_err_internal("missing internal tls url");
    return 0;
  }}

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) {{
    (void) sec4_rt_err_internal("invalid internal tls url");
    return 0;
  }}

  int64_t body = sec4_rt_http_get_internal(1, internal_url);
  if (body == 0) {{
    return 0;
  }}

  (void) sec4_rt_res_text(200, "https-ok");
  return 0;
}}

int main(void) {{
  int64_t router = sec4_rt_http_router();
  if (router == 0) {{ return 1; }}
  if (sec4_rt_http_route_get(router, "/probe", probe) != 0) {{ return 2; }}
  return sec4_rt_http_serve({runtime_port}, router) == 0 ? 0 : 3;
}}
"#
        ),
    )
    .expect("harness source should be written");

    let compile_output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-DSEC4_RT_ENABLE_OPENSSL_TLS")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .arg("-lssl")
        .arg("-lcrypto")
        .output()
        .expect("clang should execute for runtime https tls-verify-failure harness");
    let compile_stderr = String::from_utf8_lossy(&compile_output.stderr);
    assert!(
        compile_output.status.success(),
        "runtime internal-net https tls-verify-failure harness should compile successfully, stderr={compile_stderr}"
    );

    let cert_output = Command::new("openssl")
        .arg("req")
        .arg("-x509")
        .arg("-newkey")
        .arg("rsa:2048")
        .arg("-keyout")
        .arg(&key_path)
        .arg("-out")
        .arg(&cert_path)
        .arg("-sha256")
        .arg("-days")
        .arg("1")
        .arg("-nodes")
        .arg("-subj")
        .arg("/CN=127.0.0.1")
        .output()
        .expect("openssl req should execute for runtime https tls-verify-failure harness");
    let cert_stderr = String::from_utf8_lossy(&cert_output.stderr);
    assert!(
        cert_output.status.success(),
        "openssl req should create cert/key for runtime https tls-verify-failure harness, stderr={cert_stderr}"
    );

    let upstream_tls_port = find_available_tcp_port();
    let mut tls_server = Command::new("openssl")
        .arg("s_server")
        .arg("-accept")
        .arg(upstream_tls_port.to_string())
        .arg("-cert")
        .arg(&cert_path)
        .arg("-key")
        .arg(&key_path)
        .arg("-www")
        .arg("-quiet")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("openssl s_server should start for runtime https tls-verify-failure harness");

    let mut upstream_ready = false;
    for _ in 0..200 {
        if let Some(status) = tls_server
            .try_wait()
            .expect("openssl s_server wait should succeed while waiting for readiness")
        {
            panic!("openssl s_server exited before readiness with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", upstream_tls_port)) {
            Ok(_) => {
                upstream_ready = true;
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    if !upstream_ready {
        let _ = tls_server.kill();
        let _ = tls_server.wait();
        panic!("openssl s_server did not become ready in expected window");
    }

    let internal_url = format!("https://127.0.0.1:{upstream_tls_port}/internal-https");
    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("runtime https tls-verify-failure harness should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "runtime https tls-verify-failure harness exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", runtime_port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /probe HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = tls_server.kill();
            let _ = tls_server.wait();
            panic!("runtime https tls-verify-failure harness could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            let _ = tls_server.kill();
            let _ = tls_server.wait();
            panic!("runtime https tls-verify-failure harness did not exit in expected window");
        }
    };

    if tls_server
        .try_wait()
        .expect("openssl s_server wait should succeed during teardown")
        .is_none()
    {
        let _ = tls_server.kill();
        let _ = tls_server.wait();
    }

    assert!(
        status.success(),
        "runtime https tls-verify-failure harness should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 500 Internal Server Error"),
        "response should contain 500 status line when tls verification fails"
    );
    assert!(
        response.contains("\"code\":\"NET.TLS_VERIFY_FAILED\"")
            && response.contains("\"message\":\"outbound tls certificate verification failed\"")
            && response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic NET.TLS_VERIFY_FAILED payload"
    );
}

#[test]
fn c_bin_runtime_internal_https_roundtrip_succeeds_with_openssl_backend_when_available() {
    if !clang_with_openssl_available() {
        eprintln!(
            "skipping c-bin runtime internal-net https roundtrip test: clang/OpenSSL headers/libs unavailable"
        );
        return;
    }
    if !openssl_cli_available() {
        eprintln!("skipping c-bin runtime internal-net https roundtrip test: openssl CLI unavailable");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-https-roundtrip");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-https-roundtrip");
    let cert_path = project_dir.join("localhost-cert.pem");
    let key_path = project_dir.join("localhost-key.pem");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }

  int64_t body = sec4_rt_http_get_internal(1, internal_url);
  if (body == 0) { return 12; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let compile_output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-DSEC4_RT_ENABLE_OPENSSL_TLS")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .arg("-lssl")
        .arg("-lcrypto")
        .output()
        .expect("clang should execute for runtime https harness");
    let compile_stderr = String::from_utf8_lossy(&compile_output.stderr);
    assert!(
        compile_output.status.success(),
        "runtime internal-net https harness should compile successfully, stderr={compile_stderr}"
    );

    let cert_output = Command::new("openssl")
        .arg("req")
        .arg("-x509")
        .arg("-newkey")
        .arg("rsa:2048")
        .arg("-keyout")
        .arg(&key_path)
        .arg("-out")
        .arg(&cert_path)
        .arg("-sha256")
        .arg("-days")
        .arg("1")
        .arg("-nodes")
        .arg("-subj")
        .arg("/CN=127.0.0.1")
        .output()
        .expect("openssl req should execute for runtime https harness");
    let cert_stderr = String::from_utf8_lossy(&cert_output.stderr);
    assert!(
        cert_output.status.success(),
        "openssl req should create cert/key for runtime https harness, stderr={cert_stderr}"
    );

    let port = find_available_tcp_port();
    let mut tls_server = Command::new("openssl")
        .arg("s_server")
        .arg("-accept")
        .arg(port.to_string())
        .arg("-cert")
        .arg(&cert_path)
        .arg("-key")
        .arg(&key_path)
        .arg("-www")
        .arg("-quiet")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("openssl s_server should start for runtime https harness");

    let mut ready = false;
    for _ in 0..200 {
        if let Some(status) = tls_server
            .try_wait()
            .expect("openssl s_server wait should succeed while waiting for readiness")
        {
            panic!("openssl s_server exited before readiness with status: {status}");
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(_) => {
                ready = true;
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(25)),
        }
    }
    if !ready {
        let _ = tls_server.kill();
        let _ = tls_server.wait();
        panic!("openssl s_server did not become ready in expected window");
    }

    let internal_url = format!("https://127.0.0.1:{port}/internal-https");
    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_TLS_ALLOW_INSECURE", "1")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled runtime https harness should run");

    if tls_server
        .try_wait()
        .expect("openssl s_server wait should succeed during teardown")
        .is_none()
    {
        let _ = tls_server.kill();
        let _ = tls_server.wait();
    }

    assert!(
        run.status.success(),
        "runtime internal-net https harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_internal_get_timeout_returns_failure_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net timeout test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-timeout");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-timeout");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime internal-net timeout harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let (internal_port, server_handle) = spawn_one_shot_http_server_with_response_delay(
        "internal-timeout-body",
        Duration::from_millis(250),
    );
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-timeout");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_TIMEOUT_MS", "50")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net timeout harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime internal-net timeout server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_body_limit_returns_failure_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net body-limit test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-body-limit");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-body-limit");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"
#include <stdlib.h>

int main(void) {
  const char *internal_url_raw = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (internal_url_raw == NULL) { return 10; }

  int64_t internal_url = sec4_rt_req_query(internal_url_raw);
  if (internal_url == 0) { return 11; }
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 12; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime internal-net body-limit harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let oversized_body = "x".repeat(512);
    let (internal_port, server_handle) = spawn_one_shot_http_server(&oversized_body);
    let internal_url = format!("http://127.0.0.1:{internal_port}/internal-body-limit");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .env("SEC4_RT_NET_MAX_BODY_BYTES", "64")
        .env("SEC4_RT_TEST_INTERNAL_URL", &internal_url)
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net body-limit harness should exit successfully"
    );
    server_handle
        .join()
        .expect("runtime internal-net body-limit server should exit cleanly");
}

#[test]
fn c_bin_runtime_internal_get_rejects_invalid_url_or_untracked_handles_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping c-bin runtime internal-net invalid-url test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-net-invalid");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-net-invalid");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.h"

int main(void) {
  int64_t invalid_scheme = sec4_rt_req_query("ftp://127.0.0.1/not-http");
  int64_t unsupported_tls = sec4_rt_req_query("https://127.0.0.1/notls");
  if (invalid_scheme == 0 || unsupported_tls == 0) { return 11; }

  if (sec4_rt_http_get(1, invalid_scheme) != 0) { return 12; }
  if (sec4_rt_http_get_internal(1, invalid_scheme) != 0) { return 13; }
  if (sec4_rt_http_get_internal(1, unsupported_tls) != 0) { return 14; }
  if (sec4_rt_http_get(1, 2999) != 0) { return 15; }
  if (sec4_rt_http_get_internal(1, 3999) != 0) { return 16; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime internal-net invalid-url harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime internal-net invalid-url harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_direct_wrapper_malformed_target_parser_diagnostics_match_between_public_and_internal_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime direct wrapper malformed-target parity test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-direct-wrapper-target-parser-parity");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-direct-wrapper-target-parser-parity");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

static int assert_target_invalid_for_sink(bool internal_sink, const char *raw_url, int base) {
  memset(&g_sec4_rt_response, 0, sizeof(g_sec4_rt_response));
  int64_t handle = sec4_rt_req_query(raw_url);
  if (handle == 0) { return base + 1; }
  if (internal_sink) {
    if (sec4_rt_http_get_internal(1, handle) != 0) { return base + 2; }
  } else {
    if (sec4_rt_http_get(1, handle) != 0) { return base + 2; }
  }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.REQUEST_TARGET_INVALID\"") == NULL) { return base + 3; }
  if (strstr(g_sec4_rt_response.body, "\"key\":\"phase\",\"value\":\"parse\"") == NULL) { return base + 4; }
  if (strstr(g_sec4_rt_response.body, "\"key\":\"component\",\"value\":\"target\"") == NULL) { return base + 5; }
  return 0;
}

int main(void) {
  const char *raw_url = getenv("SEC4_RT_TEST_MALFORMED_URL");
  if (raw_url == NULL || raw_url[0] == '\0') { return 10; }

  int public_result = assert_target_invalid_for_sink(false, raw_url, 10);
  if (public_result != 0) { return public_result; }

  int internal_result = assert_target_invalid_for_sink(true, raw_url, 20);
  if (internal_result != 0) { return internal_result; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for direct wrapper malformed-target parity harness");
    assert!(
        output.status.success(),
        "runtime direct wrapper malformed-target parity harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let malformed_cases = [
        "http://127.0.0.1/path#frag",
        "http://127.0.0.1/path\r\nx-test: 1",
        "http://127.0.0.1/?ok=1#tail",
    ];

    for malformed_url in malformed_cases {
        let run = Command::new(&binary_path)
            .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
            .env("SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES", "0")
            .env("SEC4_RT_NET_SSRF_BLOCK_LOOPBACK", "0")
            .env("SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL", "0")
            .env("SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS", "0")
            .env("SEC4_RT_NET_SSRF_RESOLVE_DNS", "0")
            .env("SEC4_RT_TEST_MALFORMED_URL", malformed_url)
            .output()
            .expect("compiled binary should run");
        assert!(
            run.status.success(),
            "runtime direct wrapper malformed-target parity harness should exit successfully for malformed url {malformed_url:?}"
        );
    }
}

#[test]
fn c_bin_runtime_wrapper_invalid_url_preparser_envelope_parity_between_public_and_internal_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime wrapper invalid-url pre-parser parity test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-wrapper-invalid-url-preparser-parity");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-wrapper-invalid-url-preparser-parity");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

static int assert_wrapper_invalid(int internal_sink, const char *raw_url, int base) {
  memset(&g_sec4_rt_response, 0, sizeof(g_sec4_rt_response));
  int64_t handle = sec4_rt_req_query(raw_url);
  if (handle == 0) { return base + 1; }

  if (internal_sink) {
    if (sec4_rt_http_get_internal(1, handle) != 0) { return base + 2; }
    if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_INVALID\"") == NULL) { return base + 3; }
    if (strstr(g_sec4_rt_response.body, "httpClient.getInternal requires InternalUrl input") == NULL) { return base + 4; }
  } else {
    if (sec4_rt_http_get(1, handle) != 0) { return base + 2; }
    if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_INVALID\"") == NULL) { return base + 3; }
    if (strstr(g_sec4_rt_response.body, "httpClient.get requires PublicUrl input") == NULL) { return base + 4; }
  }

  if (strstr(g_sec4_rt_response.body, "\"kind\":\"validation\"") == NULL) { return base + 5; }
  if (strstr(g_sec4_rt_response.body, "NET.REQUEST_") != NULL) { return base + 6; }
  if (strstr(g_sec4_rt_response.body, "\"key\":\"phase\",\"value\":\"parse\"") != NULL) { return base + 7; }

  return 0;
}

int main(void) {
  const char *raw_url = getenv("SEC4_RT_TEST_MALFORMED_URL");
  if (raw_url == NULL || raw_url[0] == '\0') { return 10; }

  int public_result = assert_wrapper_invalid(0, raw_url, 10);
  if (public_result != 0) { return public_result; }

  int internal_result = assert_wrapper_invalid(1, raw_url, 20);
  if (internal_result != 0) { return internal_result; }

  memset(&g_sec4_rt_response, 0, sizeof(g_sec4_rt_response));
  if (sec4_rt_http_get(1, 2999) != 0) { return 30; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_PUBLIC_INVALID\"") == NULL) { return 31; }
  if (strstr(g_sec4_rt_response.body, "NET.REQUEST_") != NULL) { return 32; }

  memset(&g_sec4_rt_response, 0, sizeof(g_sec4_rt_response));
  if (sec4_rt_http_get_internal(1, 3999) != 0) { return 40; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.URL_INTERNAL_INVALID\"") == NULL) { return 41; }
  if (strstr(g_sec4_rt_response.body, "NET.REQUEST_") != NULL) { return 42; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for wrapper invalid-url pre-parser parity harness");
    assert!(
        output.status.success(),
        "runtime wrapper invalid-url pre-parser parity harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let malformed_cases = [
        "ftp://example.com/not-http",
        "example.com/no-scheme",
        "http:///missing-host",
        "http://user@example.com/path",
    ];

    for malformed_url in malformed_cases {
        let run = Command::new(&binary_path)
            .env("SEC4_RT_ALLOW_INTERNAL_NET", "1")
            .env("SEC4_RT_TEST_MALFORMED_URL", malformed_url)
            .output()
            .expect("compiled binary should run");
        assert!(
            run.status.success(),
            "runtime wrapper invalid-url pre-parser parity harness should exit successfully for malformed url {malformed_url:?}"
        );
    }
}

#[test]
fn c_bin_runtime_wrapper_invalid_handle_diagnostics_parity_between_public_and_internal_when_clang_available(
) {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime wrapper invalid-handle parity test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-wrapper-invalid-handle-parity");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-wrapper-invalid-handle-parity");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

static int assert_public_invalid_handles(int64_t valid_url, int base) {
  memset(&g_sec4_rt_response, 0, sizeof(g_sec4_rt_response));
  if (sec4_rt_http_get(0, valid_url) != 0) { return base + 1; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.GET_INVALID\"") == NULL) { return base + 2; }
  if (strstr(g_sec4_rt_response.body, "\"kind\":\"validation\"") == NULL) { return base + 3; }
  if (strstr(g_sec4_rt_response.body, "requires net capability and url handles") == NULL) { return base + 4; }
  if (strstr(g_sec4_rt_response.body, "NET.INTERNAL_DENIED") != NULL) { return base + 5; }

  memset(&g_sec4_rt_response, 0, sizeof(g_sec4_rt_response));
  if (sec4_rt_http_get(1, 0) != 0) { return base + 6; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.GET_INVALID\"") == NULL) { return base + 7; }
  if (strstr(g_sec4_rt_response.body, "\"kind\":\"validation\"") == NULL) { return base + 8; }
  if (strstr(g_sec4_rt_response.body, "NET.INTERNAL_DENIED") != NULL) { return base + 9; }
  return 0;
}

static int assert_internal_invalid_handles(int64_t valid_url, int base) {
  memset(&g_sec4_rt_response, 0, sizeof(g_sec4_rt_response));
  if (sec4_rt_http_get_internal(0, valid_url) != 0) { return base + 1; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.GET_INTERNAL_INVALID\"") == NULL) { return base + 2; }
  if (strstr(g_sec4_rt_response.body, "\"kind\":\"validation\"") == NULL) { return base + 3; }
  if (strstr(g_sec4_rt_response.body, "requires net capability and url handles") == NULL) { return base + 4; }
  if (strstr(g_sec4_rt_response.body, "NET.INTERNAL_DENIED") != NULL) { return base + 5; }

  memset(&g_sec4_rt_response, 0, sizeof(g_sec4_rt_response));
  if (sec4_rt_http_get_internal(1, 0) != 0) { return base + 6; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.GET_INTERNAL_INVALID\"") == NULL) { return base + 7; }
  if (strstr(g_sec4_rt_response.body, "\"kind\":\"validation\"") == NULL) { return base + 8; }
  if (strstr(g_sec4_rt_response.body, "NET.INTERNAL_DENIED") != NULL) { return base + 9; }
  return 0;
}

int main(void) {
  int64_t valid_url = sec4_rt_req_query("http://127.0.0.1/test");
  if (valid_url == 0) { return 10; }

  int public_result = assert_public_invalid_handles(valid_url, 10);
  if (public_result != 0) { return public_result; }

  int internal_result = assert_internal_invalid_handles(valid_url, 30);
  if (internal_result != 0) { return internal_result; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for wrapper invalid-handle parity harness");
    assert!(
        output.status.success(),
        "runtime wrapper invalid-handle parity harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "0")
        .output()
        .expect("compiled binary should run");
    assert!(
        run.status.success(),
        "runtime wrapper invalid-handle parity harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_internal_policy_denial_precedence_for_valid_internal_urls_when_clang_available() {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime internal-policy denial precedence test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-policy-denial-precedence");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-policy-denial-precedence");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

static int assert_internal_denied_for_valid_url(int64_t url_handle, int base) {
  memset(&g_sec4_rt_response, 0, sizeof(g_sec4_rt_response));
  if (sec4_rt_http_get_internal(1, url_handle) != 0) { return base + 1; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"NET.INTERNAL_DENIED\"") == NULL) { return base + 2; }
  if (strstr(g_sec4_rt_response.body, "\"kind\":\"authorization\"") == NULL) { return base + 3; }
  if (strstr(g_sec4_rt_response.body, "internal network access denied by runtime policy") == NULL) { return base + 4; }
  if (strstr(g_sec4_rt_response.body, "NET.URL_INTERNAL_INVALID") != NULL) { return base + 5; }
  if (strstr(g_sec4_rt_response.body, "NET.REQUEST_") != NULL) { return base + 6; }
  if (strstr(g_sec4_rt_response.body, "NET.GET_INTERNAL_INVALID") != NULL) { return base + 7; }
  return 0;
}

int main(void) {
  int64_t url_loopback_ip = sec4_rt_req_query("http://127.0.0.1/internal-a");
  int64_t url_localhost = sec4_rt_req_query("http://localhost/internal-b");
  if (url_loopback_ip == 0 || url_localhost == 0) { return 10; }

  int first = assert_internal_denied_for_valid_url(url_loopback_ip, 10);
  if (first != 0) { return first; }

  int second = assert_internal_denied_for_valid_url(url_localhost, 20);
  if (second != 0) { return second; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for internal-policy denial precedence harness");
    assert!(
        output.status.success(),
        "runtime internal-policy denial precedence harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let run_default_deny = Command::new(&binary_path)
        .env_remove("SEC4_RT_ALLOW_INTERNAL_NET")
        .output()
        .expect("compiled binary should run with default deny");
    assert!(
        run_default_deny.status.success(),
        "runtime internal-policy denial precedence harness should exit successfully with default deny"
    );

    let run_explicit_deny = Command::new(&binary_path)
        .env("SEC4_RT_ALLOW_INTERNAL_NET", "0")
        .output()
        .expect("compiled binary should run with explicit deny");
    assert!(
        run_explicit_deny.status.success(),
        "runtime internal-policy denial precedence harness should exit successfully with explicit deny"
    );
}

#[test]
fn c_bin_runtime_internal_policy_allow_truthy_tokens_bypass_denial_when_clang_available() {
    if !clang_available() {
        eprintln!(
            "skipping c-bin runtime internal-policy allow-token truthy test: clang not available"
        );
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-internal-policy-allow-truthy");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-internal-policy-allow-truthy");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <stdlib.h>
#include <string.h>

int main(void) {
  const char *raw_url = getenv("SEC4_RT_TEST_INTERNAL_URL");
  if (raw_url == NULL || raw_url[0] == '\0') { return 10; }
  const char *expected_code = getenv("SEC4_RT_EXPECTED_CODE");
  if (expected_code == NULL || expected_code[0] == '\0') { return 11; }
  const char *expected_kind = getenv("SEC4_RT_EXPECTED_KIND");
  if (expected_kind == NULL || expected_kind[0] == '\0') { return 12; }

  int64_t internal_url = sec4_rt_req_query(raw_url);
  if (internal_url == 0) { return 13; }

  memset(&g_sec4_rt_response, 0, sizeof(g_sec4_rt_response));
  if (sec4_rt_http_get_internal(1, internal_url) != 0) { return 14; }
  if (strstr(g_sec4_rt_response.body, expected_code) == NULL) { return 15; }
  if (strstr(g_sec4_rt_response.body, expected_kind) == NULL) { return 16; }
  if (strstr(g_sec4_rt_response.body, "NET.INTERNAL_DENIED") != NULL) { return 17; }
  if (strstr(g_sec4_rt_response.body, "NET.GET_INTERNAL_INVALID") != NULL) { return 18; }

  return 0;
}
"#,
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for internal-policy allow truthy harness");
    assert!(
        output.status.success(),
        "runtime internal-policy allow truthy harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let allow_tokens = ["1", "true", "yes", "on", "allow", "TRUE", "YeS", "On"];
    let outcome_cases = [
        (
            "http://127.0.0.1/internal#frag",
            "\"code\":\"NET.REQUEST_TARGET_INVALID\"",
            "\"kind\":\"validation\"",
        ),
        (
            "https://127.0.0.1/internal",
            "\"code\":\"NET.TLS_UNSUPPORTED\"",
            "\"kind\":\"runtime\"",
        ),
    ];

    for allow_token in allow_tokens {
        for (test_url, expected_code, expected_kind) in outcome_cases {
            let run = Command::new(&binary_path)
                .env("SEC4_RT_ALLOW_INTERNAL_NET", allow_token)
                .env("SEC4_RT_TEST_INTERNAL_URL", test_url)
                .env("SEC4_RT_EXPECTED_CODE", expected_code)
                .env("SEC4_RT_EXPECTED_KIND", expected_kind)
                .output()
                .expect("compiled binary should run");
            assert!(
                run.status.success(),
                "runtime internal-policy allow truthy harness should exit successfully for token {allow_token:?}, url {test_url:?}"
            );
        }
    }
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

    let output = run_cli(&[
        "run",
        "--path",
        hello,
        "--oneshot",
        "--serve-timeout-ms",
        "100",
    ]);
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
fn c_bin_http_runtime_serves_health_route_in_oneshot_mode_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpruntimee2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpruntimee2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!("http runtime e2e binary exited before request with status: {status}");
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
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic runtime trace header"
    );
    assert!(
        response.contains("\r\n\r\nok"),
        "response should include text body from res.text"
    );
}

#[test]
fn c_bin_http_runtime_err_internal_sets_error_response_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime err.internal e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-http-runtime-err-internal-harness");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("http-runtime-err-internal");
    let port = find_available_tcp_port();

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        format!(
            r#"#include "sec4_runtime.h"

static int64_t boom(void) {{
  (void) sec4_rt_err_internal("boom");
  return 0;
}}

int main(void) {{
  int64_t router = sec4_rt_http_router();
  if (router == 0) {{ return 1; }}
  if (sec4_rt_http_route_get(router, "/boom", boom) != 0) {{ return 2; }}
  return sec4_rt_http_serve({port}, router) == 0 ? 0 : 3;
}}
"#
        ),
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime err.internal harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime err.internal harness should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime err.internal harness exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(b"GET /boom HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime err.internal harness could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime err.internal harness did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime err.internal harness should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 500 Internal Server Error"),
        "response should contain 500 status line"
    );
    assert!(
        response.contains("\"code\":\"INTERNAL.ERROR\"")
            && response.contains("\"message\":\"boom\"")
            && response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic INTERNAL.ERROR envelope payload"
    );
    let body = response
        .split("\r\n\r\n")
        .nth(1)
        .expect("http response should include body");
    let parsed: serde_json::Value =
        serde_json::from_str(body).expect("error response body should be valid json");
    let time_ms = parsed
        .get("error")
        .and_then(|error| error.get("timeMs"))
        .and_then(serde_json::Value::as_i64)
        .expect("error.timeMs should be present");
    assert!(time_ms > 0, "error.timeMs should be non-zero: {body}");
}

#[test]
fn c_bin_http_runtime_err_rate_limit_sets_limit_field_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime err.rateLimit e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-http-runtime-err-rate-limit-harness");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("http-runtime-err-rate-limit");
    let port = find_available_tcp_port();

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        format!(
            r#"#include "sec4_runtime.h"

static int64_t limited(void) {{
  (void) sec4_rt_err_rate_limit("LIMIT.RATE", "rate limited", 7);
  return 0;
}}

int main(void) {{
  int64_t router = sec4_rt_http_router();
  if (router == 0) {{ return 1; }}
  if (sec4_rt_http_route_get(router, "/limited", limited) != 0) {{ return 2; }}
  return sec4_rt_http_serve({port}, router) == 0 ? 0 : 3;
}}
"#
        ),
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime err.rateLimit harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime err.rateLimit harness should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime err.rateLimit harness exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /limited HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime err.rateLimit harness could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime err.rateLimit harness did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime err.rateLimit harness should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 429 Too Many Requests"),
        "response should contain 429 status line"
    );
    assert!(
        response.contains("\"code\":\"LIMIT.RATE\"")
            && response.contains("\"message\":\"rate limited\"")
            && response.contains("\"limit\":{\"name\":\"limit\",\"value\":7,\"max\":7}"),
        "response should include deterministic rate-limit envelope with limit object:\n{response}"
    );
}

#[test]
fn c_bin_http_runtime_err_with_helpers_enrich_error_response_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime err.with* e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-http-runtime-err-enrich-harness");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("http-runtime-err-enrich");
    let port = find_available_tcp_port();

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        format!(
            r#"#include "sec4_runtime.h"

static int64_t validate(void) {{
  int64_t base = sec4_rt_err_validation("VALIDATE.EMAIL_INVALID", "Invalid email.");
  int64_t with_path = sec4_rt_err_with_path(base, "$.email");
  int64_t validator = sec4_rt_log_str("validate.email");
  int64_t with_detail = sec4_rt_err_with_detail(with_path, "validator", validator);
  int64_t expected = sec4_rt_log_str("Email");
  int64_t with_detail2 = sec4_rt_err_with_detail(with_detail, "expected", expected);
  int64_t with_limit = sec4_rt_err_with_limit(with_detail2, "maxJsonDepth", 33, 32);
  int64_t with_dependency = sec4_rt_err_with_dependency(with_limit, "postgres", "query", 1);
  (void) sec4_rt_err_with_cause(with_dependency, 123);
  return 0;
}}

int main(void) {{
  int64_t router = sec4_rt_http_router();
  if (router == 0) {{ return 1; }}
  if (sec4_rt_http_route_get(router, "/validate", validate) != 0) {{ return 2; }}
  return sec4_rt_http_serve({port}, router) == 0 ? 0 : 3;
}}
"#
        ),
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime err.with* harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime err.with* harness should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime err.with* harness exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /validate HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime err.with* harness could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime err.with* harness did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime err.with* harness should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain 400 status line"
    );
    assert!(
        response.contains("\"code\":\"VALIDATE.EMAIL_INVALID\"")
            && response.contains("\"message\":\"Invalid email.\"")
            && response.contains("\"path\":\"$.email\""),
        "response should include validation error and path fields:\n{response}"
    );
    assert!(
        response.contains("\"details\":[")
            && response.contains("{\"key\":\"validator\",\"value\":\"validate.email\"}")
            && response.contains("{\"key\":\"expected\",\"value\":\"Email\"}"),
        "response should include accumulated details field entries:\n{response}"
    );
    assert!(
        response.contains("\"limit\":{\"name\":\"maxJsonDepth\",\"value\":33,\"max\":32}"),
        "response should include limit field:\n{response}"
    );
    assert!(
        response
            .contains("\"dependency\":{\"service\":\"postgres\",\"operation\":\"query\",\"retryable\":true}"),
        "response should include dependency field:\n{response}"
    );
    assert!(
        response.contains("\"cause\":{\"handle\":123}"),
        "response should include cause field:\n{response}"
    );
}

#[test]
fn c_bin_http_runtime_auth_require_rejects_without_authorization_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime auth.require e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-http-runtime-auth-require-harness");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("http-runtime-auth-require");
    let port = find_available_tcp_port();

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        format!(
            r#"#include "sec4_runtime.h"

static int64_t secure(void) {{
  (void) sec4_rt_auth_require(1);
  return 0;
}}

int main(void) {{
  int64_t router = sec4_rt_http_router();
  if (router == 0) {{ return 1; }}
  if (sec4_rt_http_route_get(router, "/secure", secure) != 0) {{ return 2; }}
  return sec4_rt_http_serve({port}, router) == 0 ? 0 : 3;
}}
"#
        ),
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime auth.require harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime auth.require harness should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime auth.require harness exited before request with status: {status}"
            );
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
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime auth.require harness could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime auth.require harness did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime auth.require harness should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\"")
            && response.contains("\"message\":\"Authorization header missing or invalid\"")
            && response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic auth error envelope payload"
    );
}

#[test]
fn c_bin_http_runtime_time_now_nonzero_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime time.now e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-http-runtime-time-now-harness");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("http-runtime-time-now");
    let port = find_available_tcp_port();

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        format!(
            r#"#include "sec4_runtime.h"

static int64_t now_route(void) {{
  if (sec4_rt_time_now() == 0) {{
    return 0;
  }}
  sec4_rt_res_text(200, "ok");
  return 0;
}}

int main(void) {{
  int64_t router = sec4_rt_http_router();
  if (router == 0) {{ return 1; }}
  if (sec4_rt_http_route_get(router, "/now", now_route) != 0) {{ return 2; }}
  return sec4_rt_http_serve({port}, router) == 0 ? 0 : 3;
}}
"#
        ),
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime time.now harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime time.now harness should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime time.now harness exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(b"GET /now HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime time.now harness could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime time.now harness did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime time.now harness should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line when time.now is non-zero"
    );
    assert!(
        response.contains("\r\n\r\nok"),
        "response should include text body from successful time.now path"
    );
}

#[test]
fn c_bin_http_runtime_applies_custom_header_and_cookie_when_set() {
    if !clang_available() {
        eprintln!("skipping http runtime custom header/cookie e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-header-cookie-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpruntimeheadercookiee2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  let name = headers.name("X-Test");
  let value = headers.value("ok");
  let sessionCookie = cookie.build("session", "token");
  res.setHeader(name, value);
  res.addCookie(sessionCookie);
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP runtime custom header/cookie fixture"
    );

    let binary_path = project_dir.join("build").join("httpruntimeheadercookiee2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP runtime custom header/cookie fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime custom header/cookie e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime custom header/cookie binary exited before request with status: {status}"
            );
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
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime custom header/cookie e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime custom header/cookie e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime custom header/cookie e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line"
    );
    assert!(
        response.contains("X-Test: ok"),
        "response should include custom header"
    );
    assert!(
        response.contains("Set-Cookie: session=token"),
        "response should include cookie header"
    );
}

#[test]
fn c_bin_http_runtime_matches_parameterized_route_in_oneshot_mode_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime parameterized-route e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-param-route-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpruntimeparamroutee2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn userById() effects {{ net }} -> Int {{
  let pathId = req.pathParam("id");
  let missingQuery = req.query("id");
  if pathId == missingQuery {{
    res.text(500, "bad");
    return 0;
  }};
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/users/:id", userById);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP parameterized-route runtime fixture"
    );

    let binary_path = project_dir.join("build").join("httpruntimeparamroutee2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP parameterized-route runtime fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http parameterized-route runtime binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!("http parameterized-route binary exited before request with status: {status}");
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /users/123 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http parameterized-route e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http parameterized-route binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http parameterized-route binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line for parameterized route"
    );
    assert!(
        response.contains("\r\n\r\nok"),
        "response should include text body from parameterized route handler"
    );
}

#[test]
fn c_bin_http_runtime_applies_security_headers_on_success_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime security-headers e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-security-headers-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpruntimesecurityheaderse2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  let headersCfg = sec.defaultHeaders();
  let withHeaders = sec.withSecurityHeaders(router, headersCfg);
  http.serve({}, withHeaders);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP security-headers runtime e2e fixture"
    );

    let binary_path = project_dir
        .join("build")
        .join("httpruntimesecurityheaderse2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP security-headers runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime security-headers e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime security-headers e2e binary exited before request with status: {status}"
            );
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
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime security-headers e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime security-headers e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime security-headers e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line"
    );
    assert!(
        response.contains("X-Content-Type-Options: nosniff"),
        "response should include security header nosniff"
    );
    assert!(
        response.contains("X-Frame-Options: DENY"),
        "response should include security header frame options"
    );
    assert!(
        response.contains("Referrer-Policy: strict-origin-when-cross-origin"),
        "response should include security header referrer policy"
    );
}

#[test]
fn c_bin_http_runtime_applies_security_headers_on_not_found_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime security-headers 404 e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-security-headers-404-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpruntimesecurityheaders404e2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  let headersCfg = sec.defaultHeaders();
  let withHeaders = sec.withSecurityHeaders(router, headersCfg);
  http.serve({}, withHeaders);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP security-headers 404 runtime e2e fixture"
    );

    let binary_path = project_dir
        .join("build")
        .join("httpruntimesecurityheaders404e2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP security-headers 404 runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime security-headers 404 e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime security-headers 404 e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /missing HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime security-headers 404 e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime security-headers 404 e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime security-headers 404 e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 404 Not Found"),
        "response should contain 404 status line"
    );
    assert!(
        response.contains("X-Content-Type-Options: nosniff"),
        "404 response should include security header nosniff"
    );
    assert!(
        response.contains("X-Frame-Options: DENY"),
        "404 response should include security header frame options"
    );
    assert!(
        response.contains("Referrer-Policy: strict-origin-when-cross-origin"),
        "404 response should include security header referrer policy"
    );
}

#[test]
fn c_bin_http_runtime_applies_security_headers_on_auth_reject_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime security-headers auth reject e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-security-headers-auth-reject-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpruntimesecurityheadersauthrejecte2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  let headersCfg = sec.defaultHeaders();
  let withHeaders = sec.withSecurityHeaders(router, headersCfg);
  let authCfg = auth.fromPolicy();
  let withAuth = auth.withAuth(withHeaders, authCfg);
  http.serve({}, withAuth);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP security-headers auth reject runtime e2e fixture"
    );

    let binary_path = project_dir
        .join("build")
        .join("httpruntimesecurityheadersauthrejecte2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP security-headers auth reject runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime security-headers auth reject e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime security-headers auth reject e2e binary exited before request with status: {status}"
            );
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
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime security-headers auth reject e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime security-headers auth reject e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime security-headers auth reject e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line"
    );
    assert!(
        response.contains("X-Content-Type-Options: nosniff"),
        "401 response should include security header nosniff"
    );
    assert!(
        response.contains("X-Frame-Options: DENY"),
        "401 response should include security header frame options"
    );
    assert!(
        response.contains("Referrer-Policy: strict-origin-when-cross-origin"),
        "401 response should include security header referrer policy"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\""),
        "401 response should include deterministic auth error code"
    );
}

#[test]
fn c_bin_http_runtime_applies_security_headers_on_csrf_reject_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime security-headers csrf reject e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-security-headers-csrf-reject-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpruntimesecurityheaderscsrfrejecte2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  let headersCfg = sec.defaultHeaders();
  let withHeaders = sec.withSecurityHeaders(router, headersCfg);
  let csrfCfg = csrf.fromPolicy();
  let withCsrf = csrf.withCsrf(withHeaders, csrfCfg);
  http.serve({}, withCsrf);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP security-headers csrf reject runtime e2e fixture"
    );

    let binary_path = project_dir
        .join("build")
        .join("httpruntimesecurityheaderscsrfrejecte2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP security-headers csrf reject runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime security-headers csrf reject e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime security-headers csrf reject e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime security-headers csrf reject e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime security-headers csrf reject e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime security-headers csrf reject e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 403 Forbidden"),
        "response should contain 403 status line"
    );
    assert!(
        response.contains("X-Content-Type-Options: nosniff"),
        "403 response should include security header nosniff"
    );
    assert!(
        response.contains("X-Frame-Options: DENY"),
        "403 response should include security header frame options"
    );
    assert!(
        response.contains("Referrer-Policy: strict-origin-when-cross-origin"),
        "403 response should include security header referrer policy"
    );
    assert!(
        response.contains("\"code\":\"AUTH.CSRF_TOKEN_INVALID\""),
        "403 response should include deterministic csrf error code"
    );
}

#[test]
fn c_bin_http_runtime_applies_security_headers_on_405_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime security-headers 405 e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-security-headers-405-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpruntimesecurityheaders405e2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  let headersCfg = sec.defaultHeaders();
  let withHeaders = sec.withSecurityHeaders(router, headersCfg);
  http.serve({}, withHeaders);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP security-headers 405 runtime e2e fixture"
    );

    let binary_path = project_dir
        .join("build")
        .join("httpruntimesecurityheaders405e2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP security-headers 405 runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime security-headers 405 e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime security-headers 405 e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /users HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime security-headers 405 e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime security-headers 405 e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime security-headers 405 e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 405 Method Not Allowed"),
        "response should contain 405 status line"
    );
    assert!(
        response.contains("Allow: POST"),
        "response should include Allow header on 405 response"
    );
    assert!(
        response.contains("X-Content-Type-Options: nosniff"),
        "405 response should include security header nosniff"
    );
    assert!(
        response.contains("X-Frame-Options: DENY"),
        "405 response should include security header frame options"
    );
    assert!(
        response.contains("Referrer-Policy: strict-origin-when-cross-origin"),
        "405 response should include security header referrer policy"
    );
}

#[test]
fn c_bin_http_runtime_applies_security_headers_on_cors_preflight_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime security-headers preflight e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-security-headers-preflight-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpruntimesecurityheaderspreflighte2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  res.text(201, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  let corsCfg = cors.fromPolicy();
  let withCors = cors.withCors(router, corsCfg);
  let headersCfg = sec.defaultHeaders();
  let withHeaders = sec.withSecurityHeaders(withCors, headersCfg);
  http.serve({}, withHeaders);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP security-headers preflight runtime e2e fixture"
    );

    let binary_path = project_dir
        .join("build")
        .join("httpruntimesecurityheaderspreflighte2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP security-headers preflight runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime security-headers preflight e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime security-headers preflight e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"OPTIONS /users HTTP/1.1\r\nHost: localhost\r\nOrigin: https://app.example.com\r\nAccess-Control-Request-Method: POST\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime security-headers preflight e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime security-headers preflight e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime security-headers preflight e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 preflight status line"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: *"),
        "preflight response should include cors allow-origin header"
    );
    assert!(
        response.contains("X-Content-Type-Options: nosniff"),
        "preflight response should include security header nosniff"
    );
    assert!(
        response.contains("X-Frame-Options: DENY"),
        "preflight response should include security header frame options"
    );
    assert!(
        response.contains("Referrer-Policy: strict-origin-when-cross-origin"),
        "preflight response should include security header referrer policy"
    );
}

#[test]
fn c_bin_http_runtime_rejects_request_without_auth_header_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime auth reject e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-auth-reject-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpauthrejecte2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  let authCfg = auth.fromPolicy();
  let withAuth = auth.withAuth(router, authCfg);
  http.serve({}, withAuth);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP auth reject runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpauthrejecte2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP auth reject runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime auth reject e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime auth reject e2e binary exited before request with status: {status}"
            );
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
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime auth reject e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime auth reject e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime auth reject e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line for missing auth header"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\"")
            && response.contains("\"message\":\"Authorization header missing or invalid\""),
        "response should include deterministic auth error envelope payload"
    );
}

#[test]
fn c_bin_http_runtime_applies_cors_origin_header_on_auth_reject_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime auth/cors reject e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-auth-cors-reject-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpauthcorsrejecte2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  let corsCfg = cors.fromPolicy();
  let withCors = cors.withCors(router, corsCfg);
  let authCfg = auth.fromPolicy();
  let withAuth = auth.withAuth(withCors, authCfg);
  http.serve({}, withAuth);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP auth/cors reject runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpauthcorsrejecte2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP auth/cors reject runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime auth/cors reject e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime auth/cors reject e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nOrigin: https://app.example.com\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime auth/cors reject e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime auth/cors reject e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime auth/cors reject e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "response should contain 401 status line for missing auth header"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: *"),
        "response should include cors allow-origin header on auth rejection response"
    );
    assert!(
        response.contains("\"code\":\"AUTH.UNAUTHORIZED\"")
            && response.contains("\"message\":\"Authorization header missing or invalid\""),
        "response should include deterministic auth error envelope payload"
    );
}

#[test]
fn c_bin_http_runtime_allows_request_with_auth_header_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime auth allow e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-auth-allow-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpauthallowe2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  let authCfg = auth.fromPolicy();
  let withAuth = auth.withAuth(router, authCfg);
  http.serve({}, withAuth);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP auth allow runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpauthallowe2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP auth allow runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime auth allow e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime auth allow e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer token123\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime auth allow e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime auth allow e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime auth allow e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line when auth header is valid"
    );
    assert!(
        response.contains("\r\n\r\nok"),
        "response should include route body when auth header is valid"
    );
}

#[test]
fn c_bin_http_runtime_allows_request_with_session_cookie_when_cookie_auth_mode_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime cookie-auth allow e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-cookie-auth-allow-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcookieauthallowe2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  let authCfg = auth.fromPolicy();
  let withAuth = auth.withAuth(router, authCfg);
  http.serve({}, withAuth);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP cookie-auth allow runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcookieauthallowe2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP cookie-auth allow runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_AUTH_MODE", "cookie")
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime cookie-auth allow e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime cookie-auth allow e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /health HTTP/1.1\r\nHost: localhost\r\nCookie: session=session123\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime cookie-auth allow e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime cookie-auth allow e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime cookie-auth allow e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line when session cookie is valid"
    );
    assert!(
        response.contains("\r\n\r\nok"),
        "response should include route body when session cookie is valid"
    );
}

#[test]
fn c_bin_http_runtime_auth_require_role_rejects_cookie_without_required_role_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime auth.requireRole cookie e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-http-runtime-auth-require-role-cookie-harness");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("http-runtime-auth-require-role-cookie");
    let port = find_available_tcp_port();

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        format!(
            r#"#include "sec4_runtime.h"

static int64_t secure(void) {{
  (void) sec4_rt_auth_require_role(1, "admin");
  return 0;
}}

int main(void) {{
  int64_t router = sec4_rt_http_router();
  if (router == 0) {{ return 1; }}
  if (sec4_rt_http_route_get(router, "/secure", secure) != 0) {{ return 2; }}
  return sec4_rt_http_serve({port}, router) == 0 ? 0 : 3;
}}
"#
        ),
    )
    .expect("harness source should be written");

    let output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime harness");
    assert!(
        output.status.success(),
        "runtime auth.requireRole cookie harness should compile successfully"
    );

    assert!(binary_path.exists(), "compiled binary should exist");

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_AUTH_MODE", "cookie")
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("runtime auth.requireRole cookie harness should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "runtime auth.requireRole cookie harness exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /secure HTTP/1.1\r\nHost: localhost\r\nCookie: session=session123\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("runtime auth.requireRole cookie harness could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("runtime auth.requireRole cookie harness did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "runtime auth.requireRole cookie harness should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 403 Forbidden"),
        "response should contain 403 status line when role is missing in cookie mode"
    );
    assert!(
        response.contains("\"code\":\"AUTH.FORBIDDEN\"")
            && response.contains("\"message\":\"Authenticated cookie principal missing required role\""),
        "response should include deterministic cookie-role forbidden error envelope payload"
    );
}

#[test]
fn c_bin_http_runtime_returns_405_on_method_mismatch_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime 405 e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-405-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpruntime405e2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP 405 runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpruntime405e2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP 405 runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime 405 e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!("http runtime 405 e2e binary exited before request with status: {status}");
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /users HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime 405 e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime 405 e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime 405 e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 405 Method Not Allowed"),
        "response should contain 405 status line for method mismatch"
    );
    assert!(
        response.contains("X-Trace-Id: rt-1"),
        "response should include deterministic runtime trace header"
    );
    assert!(
        response.contains("Allow: POST"),
        "response should include Allow header for method mismatch route"
    );
    assert!(
        response.contains("\r\n\r\nmethod not allowed"),
        "response should include method-not-allowed body"
    );
}

#[test]
fn c_bin_http_runtime_applies_cors_origin_header_on_not_found_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime cors 404 e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-cors-404-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcors404e2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  let corsCfg = cors.fromPolicy();
  let withCors = cors.withCors(router, corsCfg);
  http.serve({}, withCors);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP cors 404 runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcors404e2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP cors 404 runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime cors 404 e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!("http runtime cors 404 e2e binary exited before request with status: {status}");
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /missing HTTP/1.1\r\nHost: localhost\r\nOrigin: https://app.example.com\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime cors 404 e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime cors 404 e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime cors 404 e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 404 Not Found"),
        "response should contain 404 status line"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: *"),
        "response should include cors allow-origin header on 404 response"
    );
}

#[test]
fn c_bin_http_runtime_applies_cors_origin_header_on_405_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime cors 405 e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-cors-405-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcors405e2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  let corsCfg = cors.fromPolicy();
  let withCors = cors.withCors(router, corsCfg);
  http.serve({}, withCors);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP cors 405 runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcors405e2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP cors 405 runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime cors 405 e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!("http runtime cors 405 e2e binary exited before request with status: {status}");
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /users HTTP/1.1\r\nHost: localhost\r\nOrigin: https://app.example.com\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime cors 405 e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime cors 405 e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime cors 405 e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 405 Method Not Allowed"),
        "response should contain 405 status line"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: *"),
        "response should include cors allow-origin header on 405 response"
    );
    assert!(
        response.contains("Allow: POST"),
        "response should include Allow header on 405 response"
    );
}

#[test]
fn c_bin_http_runtime_handles_cors_preflight_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime cors preflight e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-cors-preflight-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcorspreflighte2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  res.text(201, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  let corsCfg = cors.fromPolicy();
  let withCors = cors.withCors(router, corsCfg);
  http.serve({}, withCors);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP cors preflight runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcorspreflighte2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP cors preflight runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime cors preflight e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime cors preflight e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"OPTIONS /users HTTP/1.1\r\nHost: localhost\r\nOrigin: https://app.example.com\r\nAccess-Control-Request-Method: POST\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime cors preflight e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime cors preflight e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime cors preflight e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 status line for CORS preflight request"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: *"),
        "response should include cors allow-origin header"
    );
    assert!(
        response.contains("Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS"),
        "response should include cors allow-methods header"
    );
    assert!(
        response.contains("Access-Control-Allow-Headers: content-type, authorization"),
        "response should include cors allow-headers header"
    );
}

#[test]
fn c_bin_http_runtime_allows_cors_preflight_with_auth_and_csrf_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime cors preflight auth/csrf e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-cors-preflight-auth-csrf-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcorspreflightauthcsrfe2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  res.text(201, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  let corsCfg = cors.fromPolicy();
  let withCors = cors.withCors(router, corsCfg);
  let csrfCfg = csrf.fromPolicy();
  let withCsrf = csrf.withCsrf(withCors, csrfCfg);
  let authCfg = auth.fromPolicy();
  let withAuth = auth.withAuth(withCsrf, authCfg);
  http.serve({}, withAuth);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP cors preflight auth/csrf runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcorspreflightauthcsrfe2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP cors preflight auth/csrf runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime cors preflight auth/csrf e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime cors preflight auth/csrf e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"OPTIONS /users HTTP/1.1\r\nHost: localhost\r\nOrigin: https://app.example.com\r\nAccess-Control-Request-Method: POST\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime cors preflight auth/csrf e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime cors preflight auth/csrf e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime cors preflight auth/csrf e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 204 No Content"),
        "response should contain 204 status line for CORS preflight request"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: *"),
        "response should include cors allow-origin header"
    );
    assert!(
        response.contains("Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS"),
        "response should include cors allow-methods header"
    );
    assert!(
        response.contains("Access-Control-Allow-Headers: content-type, authorization"),
        "response should include cors allow-headers header"
    );
}

#[test]
fn c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime cors origin e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-cors-origin-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcorsorigine2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn health() effects {{ net }} -> Int {{
  res.text(200, "ok");
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.get(router, "/health", health);
  let corsCfg = cors.fromPolicy();
  let withCors = cors.withCors(router, corsCfg);
  http.serve({}, withCors);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP cors origin runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcorsorigine2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP cors origin runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime cors origin e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime cors origin e2e binary exited before request with status: {status}"
            );
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
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime cors origin e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime cors origin e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime cors origin e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: *"),
        "response should include cors allow-origin header on success response"
    );
}

#[test]
fn c_bin_http_runtime_rejects_post_without_csrf_tokens_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime csrf reject e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-csrf-reject-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcsrfrejecte2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  let csrfCfg = csrf.fromPolicy();
  let withCsrf = csrf.withCsrf(router, csrfCfg);
  http.serve({}, withCsrf);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP csrf reject runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcsrfrejecte2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP csrf reject runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime csrf reject e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime csrf reject e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime csrf reject e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime csrf reject e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime csrf reject e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 403 Forbidden"),
        "response should contain 403 status line for missing csrf tokens"
    );
    assert!(
        response.contains("\"code\":\"AUTH.CSRF_TOKEN_INVALID\"")
            && response.contains("\"message\":\"CSRF token missing or invalid\""),
        "response should include deterministic csrf error envelope payload"
    );
}

#[test]
fn c_bin_http_runtime_applies_cors_origin_header_on_csrf_reject_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime csrf/cors reject e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-csrf-cors-reject-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcsrfcorsrejecte2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  let corsCfg = cors.fromPolicy();
  let withCors = cors.withCors(router, corsCfg);
  let csrfCfg = csrf.fromPolicy();
  let withCsrf = csrf.withCsrf(withCors, csrfCfg);
  http.serve({}, withCsrf);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP csrf/cors reject runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcsrfcorsrejecte2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP csrf/cors reject runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime csrf/cors reject e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime csrf/cors reject e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nOrigin: https://app.example.com\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime csrf/cors reject e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime csrf/cors reject e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime csrf/cors reject e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 403 Forbidden"),
        "response should contain 403 status line for missing csrf tokens"
    );
    assert!(
        response.contains("Access-Control-Allow-Origin: *"),
        "response should include cors allow-origin header on csrf rejection response"
    );
    assert!(
        response.contains("\"code\":\"AUTH.CSRF_TOKEN_INVALID\"")
            && response.contains("\"message\":\"CSRF token missing or invalid\""),
        "response should include deterministic csrf error envelope payload"
    );
}

#[test]
fn c_bin_http_runtime_allows_post_with_matching_csrf_tokens_when_enabled() {
    if !clang_available() {
        eprintln!("skipping http runtime csrf allow e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-csrf-allow-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcsrfallowe2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  let csrfCfg = csrf.fromPolicy();
  let withCsrf = csrf.withCsrf(router, csrfCfg);
  http.serve({}, withCsrf);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP csrf allow runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcsrfallowe2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP csrf allow runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime csrf allow e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime csrf allow e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nX-CSRF-Token: token123\r\nCookie: csrf=token123\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime csrf allow e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime csrf allow e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime csrf allow e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 201 Created"),
        "response should contain 201 status line when csrf tokens match"
    );
    assert!(
        response.contains("\"ok\":true")
            && response.contains("\"status\":201")
            && response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic std-success envelope payload"
    );
}

#[test]
fn c_bin_http_runtime_serves_users_post_with_json_response_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime post/json e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-post-json-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httppostjsone2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP POST/JSON runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httppostjsone2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP POST/JSON runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime post/json e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!("http runtime post/json e2e binary exited before request with status: {status}");
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime post/json e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime post/json e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime post/json e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 201 Created"),
        "response should contain 201 status line for res.ok"
    );
    assert!(
        response.contains("Content-Type: application/json; charset=utf-8"),
        "response should include JSON content-type"
    );
    assert!(
        response.contains("\"ok\":true")
            && response.contains("\"status\":201")
            && response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic std-success envelope body"
    );
    let body = response
        .split("\r\n\r\n")
        .nth(1)
        .expect("http response should include body");
    let parsed: serde_json::Value =
        serde_json::from_str(body).expect("success response body should be valid json");
    let time_ms = parsed
        .get("timeMs")
        .and_then(serde_json::Value::as_i64)
        .expect("success timeMs should be present");
    assert!(time_ms > 0, "success timeMs should be non-zero: {body}");
}

#[test]
fn c_bin_http_runtime_res_ok_honors_custom_status_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime custom-status e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-custom-status-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcustomstatuse2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(202, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP custom-status runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcustomstatuse2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP custom-status runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime custom-status e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime custom-status e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime custom-status e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime custom-status e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime custom-status e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 202 "),
        "response should contain custom 202 status line for res.ok"
    );
    assert!(
        response.contains("Content-Type: application/json; charset=utf-8"),
        "response should include JSON content-type"
    );
    assert!(
        response.contains("\"ok\":true")
            && response.contains("\"status\":202")
            && response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic std-success envelope body"
    );
}

#[test]
fn c_bin_http_runtime_res_ok_meta_includes_meta_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime res.okMeta e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-ok-meta-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpokmetae2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.okMeta(201, "CreateUserResponse", 1, 2);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP okMeta runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpokmetae2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP okMeta runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime okMeta e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!("http runtime okMeta e2e binary exited before request with status: {status}");
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime okMeta e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime okMeta e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime okMeta e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 201 Created"),
        "response should contain 201 status line for res.okMeta"
    );
    assert!(
        response.contains("Content-Type: application/json; charset=utf-8"),
        "response should include JSON content-type"
    );
    assert!(
        response.contains("\"ok\":true")
            && response.contains("\"status\":201")
            && response.contains("\"traceId\":\"rt-1\"")
            && response.contains("\"data\":1")
            && response.contains("\"meta\":2"),
        "response should include deterministic std-success envelope with meta"
    );
}

#[test]
fn c_bin_http_runtime_req_json_rejects_invalid_body_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime invalid-json e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-invalid-json-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpinvalidjsone2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP invalid-json runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpinvalidjsone2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP invalid-json runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime invalid-json e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime invalid-json e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 8\r\nConnection: close\r\n\r\nnot-json",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime invalid-json e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime invalid-json e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime invalid-json e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 400 Bad Request"),
        "response should contain 400 status line for invalid req.json body"
    );
    assert!(
        response.contains("\"code\":\"JSON.INVALID_BODY\"")
            && response.contains("\"message\":\"invalid json body\"")
            && response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic std-error invalid-json payload"
    );
}

#[test]
fn c_bin_http_runtime_req_json_rejects_non_json_content_type_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime content-type e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-content-type-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpcontenttypee2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP content-type runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpcontenttypee2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP content-type runtime e2e fixture"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime content-type e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime content-type e2e binary exited before request with status: {status}"
            );
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime content-type e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime content-type e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime content-type e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 415 Unsupported Media Type"),
        "response should contain 415 status line for non-json content-type"
    );
    assert!(
        response.contains("\"code\":\"HTTP.CONTENT_TYPE_INVALID\"")
            && response.contains("\"message\":\"content-type must be application/json\""),
        "response should include deterministic std-error content-type gate payload"
    );
}

#[test]
fn c_bin_http_runtime_req_json_rejects_oversized_body_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime oversized-body e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-oversized-body-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpoverbodye2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP oversized-body runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpoverbodye2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP oversized-body runtime e2e fixture"
    );

    let oversized_body = format!("{{\"blob\":\"{}\"}}", "a".repeat(5000));
    let request = format!(
        "POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        oversized_body.len(),
        oversized_body
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime oversized-body e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime oversized-body e2e binary exited before request with status: {status}"
            );
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
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime oversized-body e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime oversized-body e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime oversized-body e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 413 Payload Too Large"),
        "response should contain 413 status line for oversized req.json body"
    );
    assert!(
        response.contains("\"code\":\"LIMIT.BODY_BYTES\"")
            && response.contains("\"message\":\"request body exceeds runtime limit\""),
        "response should include deterministic std-error oversized-body payload"
    );
}

#[test]
fn c_bin_http_runtime_req_json_honors_env_body_limit_when_set() {
    if !clang_available() {
        eprintln!("skipping http runtime env-body-limit e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-c-bin-http-runtime-env-body-limit-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "httpenvbodylimite2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");

    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let build_output = run_cli(&["build", "--path", path, "--emit", "c-bin"]);
    assert!(
        build_output.status.success(),
        "c-bin build should succeed for HTTP env-body-limit runtime e2e fixture"
    );

    let binary_path = project_dir.join("build").join("httpenvbodylimite2e");
    assert!(
        binary_path.exists(),
        "compiled binary should exist for HTTP env-body-limit runtime e2e fixture"
    );

    let constrained_body = "{\"payload\":\"12345678901234567890\"}";
    let request = format!(
        "POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        constrained_body.len(),
        constrained_body
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .env("SEC4_RT_HTTP_MAX_BODY_BYTES", "16")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("http runtime env-body-limit e2e binary should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!(
                "http runtime env-body-limit e2e binary exited before request with status: {status}"
            );
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
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("http runtime env-body-limit e2e test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("http runtime env-body-limit e2e binary did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "http runtime env-body-limit e2e binary should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 413 Payload Too Large"),
        "response should contain 413 status line when env body limit is exceeded"
    );
    assert!(
        response.contains("\"code\":\"LIMIT.BODY_BYTES\"")
            && response.contains("\"message\":\"request body exceeds runtime limit\""),
        "response should include deterministic body-limit error payload"
    );
}

#[test]
fn c_bin_http_runtime_req_json_rejects_json_size_limit_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime json-size-limit harness test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-json-size-limit");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-json-size-limit");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  const char *json = "{\"blob\":\"12345678901234567890\"}";
  if (setenv("SEC4_RT_JSON_MAX_BYTES", "16", 1) != 0) { return 11; }

  sec4_rt_reset_request();
  sec4_rt_reset_response();
  g_sec4_rt_request.has_request = true;
  g_sec4_rt_request.has_content_type = true;
  g_sec4_rt_request.content_type_is_json = true;

  size_t json_len = strlen(json);
  memcpy(g_sec4_rt_request.body, json, json_len + 1);
  g_sec4_rt_request.body_len = json_len;

  if (sec4_rt_req_json(INT64_C(1)) == 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"JSON.SIZE_LIMIT\"") == NULL) { return 13; }
  if (strstr(g_sec4_rt_response.body, "\"message\":\"json payload exceeds runtime size limit\"") == NULL) { return 14; }
  return 0;
}
"#,
    )
    .expect("runtime harness source should be written");

    let compile_output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime json-size-limit harness");
    assert!(
        compile_output.status.success(),
        "runtime json-size-limit harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("runtime json-size-limit harness should run");
    assert!(
        run.status.success(),
        "runtime json-size-limit harness should exit successfully"
    );
}

#[test]
fn c_bin_http_runtime_req_json_rejects_json_depth_limit_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime json-depth-limit harness test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-json-depth-limit");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-json-depth-limit");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

int main(void) {
  const char *deep_json = "{\"a\":{\"b\":{\"c\":1}}}";
  const char *bounded_json = "{\"a\":{\"b\":1}}";

  if (setenv("SEC4_RT_JSON_MAX_DEPTH", "2", 1) != 0) { return 11; }
  if (setenv("SEC4_RT_JSON_MAX_BYTES", "64", 1) != 0) { return 12; }

  sec4_rt_reset_request();
  sec4_rt_reset_response();
  g_sec4_rt_request.has_request = true;
  g_sec4_rt_request.has_content_type = true;
  g_sec4_rt_request.content_type_is_json = true;

  size_t deep_len = strlen(deep_json);
  memcpy(g_sec4_rt_request.body, deep_json, deep_len + 1);
  g_sec4_rt_request.body_len = deep_len;

  if (sec4_rt_req_json(INT64_C(1)) == 0) { return 13; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"JSON.DEPTH_LIMIT\"") == NULL) { return 14; }
  if (strstr(g_sec4_rt_response.body, "\"message\":\"json payload exceeds runtime depth limit\"") == NULL) { return 15; }

  if (setenv("SEC4_RT_JSON_MAX_DEPTH", "4", 1) != 0) { return 16; }
  sec4_rt_reset_request();
  sec4_rt_reset_response();
  g_sec4_rt_request.has_request = true;
  g_sec4_rt_request.has_content_type = true;
  g_sec4_rt_request.content_type_is_json = true;

  size_t bounded_len = strlen(bounded_json);
  memcpy(g_sec4_rt_request.body, bounded_json, bounded_len + 1);
  g_sec4_rt_request.body_len = bounded_len;

  if (sec4_rt_req_json(INT64_C(1)) != 0) { return 17; }
  if (!g_sec4_rt_request.json_valid) { return 18; }
  return 0;
}
"#,
    )
    .expect("runtime harness source should be written");

    let compile_output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime json-depth-limit harness");
    assert!(
        compile_output.status.success(),
        "runtime json-depth-limit harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("runtime json-depth-limit harness should run");
    assert!(
        run.status.success(),
        "runtime json-depth-limit harness should exit successfully"
    );
}

#[test]
fn c_bin_runtime_json_decode_enforces_gate_and_schema_alignment_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping runtime json.decode gate/schema harness test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-c-json-gate-schema");
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("runtime-json-gate-schema");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        r#"#include "sec4_runtime.c"
#include <string.h>

static void set_request_body(const char *body) {
  size_t body_len = strlen(body);
  sec4_rt_reset_request();
  sec4_rt_reset_response();
  g_sec4_rt_request.has_request = true;
  g_sec4_rt_request.has_content_type = true;
  g_sec4_rt_request.content_type_is_json = true;
  memcpy(g_sec4_rt_request.body, body, body_len + 1);
  g_sec4_rt_request.body_len = body_len;
}

int main(void) {
  const char *body = "{\"email\":\"user@example.com\"}";

  set_request_body(body);
  int64_t schema_req = sec4_rt_track_string_value("CreateUserRequest", UINT64_C(0xFAB01));
  int64_t schema_alt = sec4_rt_track_string_value("CreateUserAlt", UINT64_C(0xFAB02));
  int64_t raw = sec4_rt_track_string_value(body, UINT64_C(0xFAB03));
  if (schema_req == 0 || schema_alt == 0 || raw == 0) { return 11; }

  if (sec4_rt_json_decode(INT64_C(1), schema_req, raw) != 0) { return 12; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"JSON.GATE_REQUIRED\"") == NULL) { return 13; }

  sec4_rt_reset_response();
  if (sec4_rt_req_json(INT64_C(0)) == 0) { return 14; }
  sec4_rt_reset_response();
  if (sec4_rt_json_decode(INT64_C(1), schema_req, raw) != 0) { return 15; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"JSON.GATE_FAILED\"") == NULL) { return 16; }

  set_request_body(body);
  schema_req = sec4_rt_track_string_value("CreateUserRequest", UINT64_C(0xFAB11));
  schema_alt = sec4_rt_track_string_value("CreateUserAlt", UINT64_C(0xFAB12));
  raw = sec4_rt_track_string_value(body, UINT64_C(0xFAB13));
  if (schema_req == 0 || schema_alt == 0 || raw == 0) { return 17; }
  if (sec4_rt_req_json(schema_req) != 0) { return 18; }

  sec4_rt_reset_response();
  if (sec4_rt_json_decode(INT64_C(1), schema_alt, raw) != 0) { return 19; }
  if (strstr(g_sec4_rt_response.body, "\"code\":\"JSON.SCHEMA_MISMATCH\"") == NULL) { return 20; }

  sec4_rt_reset_response();
  int64_t decoded = sec4_rt_json_decode(INT64_C(1), schema_req, raw);
  if (decoded == 0) { return 21; }
  const char *decoded_value = sec4_rt_lookup_tracked_value(decoded);
  if (decoded_value == NULL || strcmp(decoded_value, body) != 0) { return 22; }

  return 0;
}
"#,
    )
    .expect("runtime harness source should be written");

    let compile_output = Command::new("clang")
        .arg(&harness_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime json gate/schema harness");
    assert!(
        compile_output.status.success(),
        "runtime json gate/schema harness should compile successfully"
    );

    let run = Command::new(&binary_path)
        .output()
        .expect("runtime json gate/schema harness should run");
    assert!(
        run.status.success(),
        "runtime json gate/schema harness should exit successfully"
    );
}

#[test]
fn c_bin_http_runtime_crypto_ct_eq_with_redacted_query_values_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime crypto.ctEq redaction e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-http-cteq-redacted-e2e");
    let port = find_available_tcp_port();
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("http-cteq-redacted-e2e");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        format!(
            r#"#include "sec4_runtime.h"

static int64_t compare_handler(void) {{
  int64_t left = sec4_rt_req_query("left");
  int64_t right = sec4_rt_req_query("right");
  int64_t left_redacted = sec4_rt_secret_redact(left);
  int64_t right_redacted = sec4_rt_secret_redact(right);
  if (left_redacted == 0 || right_redacted == 0) {{
    sec4_rt_res_text(500, "redact-failed");
    return 0;
  }}
  if (sec4_rt_crypto_ct_eq(left_redacted, right_redacted)) {{
    sec4_rt_res_text(200, "equal");
  }} else {{
    sec4_rt_res_text(401, "not-equal");
  }}
  return 0;
}}

int main(void) {{
  int64_t router = sec4_rt_http_router();
  if (router == 0) {{
    return 11;
  }}
  if (sec4_rt_http_route_get(router, "/compare", compare_handler) != 0) {{
    return 12;
  }}
  return (int) sec4_rt_http_serve({}, router);
}}
"#,
            port
        ),
    )
    .expect("runtime harness source should be written");

    let compile_output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime crypto.ctEq harness");
    assert!(
        compile_output.status.success(),
        "runtime crypto.ctEq harness should compile successfully"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("runtime crypto.ctEq harness should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!("runtime crypto.ctEq harness exited before request with status: {status}");
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /compare?left=token123&right=token123 HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("runtime crypto.ctEq harness test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("runtime crypto.ctEq harness did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "runtime crypto.ctEq harness should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line for equal redacted query values"
    );
    assert!(
        response.contains("equal"),
        "response should include equal body when redacted query values match"
    );
}

#[test]
fn c_bin_http_runtime_secret_reveal_denied_by_default_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime secret reveal deny e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-http-secret-reveal-deny-e2e");
    let port = find_available_tcp_port();
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("http-secret-reveal-deny-e2e");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        format!(
            r#"#include "sec4_runtime.h"

static int64_t reveal_handler(void) {{
  int64_t secret = sec4_rt_secret_get(1, "SEC4_RT_TEST_SECRET_VALUE");
  if (secret == 0) {{
    sec4_rt_res_text(500, "missing-secret");
    return 0;
  }}
  int64_t revealed = sec4_rt_secret_reveal(1, secret);
  if (revealed == 0) {{
    sec4_rt_res_text(403, "reveal-denied");
    return 0;
  }}
  sec4_rt_res_text(200, "reveal-allowed");
  return 0;
}}

int main(void) {{
  int64_t router = sec4_rt_http_router();
  if (router == 0) {{
    return 11;
  }}
  if (sec4_rt_http_route_get(router, "/reveal", reveal_handler) != 0) {{
    return 12;
  }}
  return (int) sec4_rt_http_serve({}, router);
}}
"#,
            port
        ),
    )
    .expect("runtime harness source should be written");

    let compile_output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime secret reveal deny harness");
    assert!(
        compile_output.status.success(),
        "runtime secret reveal deny harness should compile successfully"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .env("SEC4_RT_TEST_SECRET_VALUE", "runtime-secret")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("runtime secret reveal deny harness should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!("runtime secret reveal deny harness exited before request with status: {status}");
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /reveal HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("runtime secret reveal deny harness test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("runtime secret reveal deny harness did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "runtime secret reveal deny harness should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 403 Forbidden"),
        "response should contain 403 status line when secret reveal is denied by default"
    );
    assert!(
        response.contains("reveal-denied"),
        "response should include reveal-denied body when secret reveal is denied"
    );
}

#[test]
fn c_bin_http_runtime_secret_reveal_allowed_with_env_override_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping http runtime secret reveal allow e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-runtime-http-secret-reveal-allow-e2e");
    let port = find_available_tcp_port();
    let harness_path = project_dir.join("harness.c");
    let binary_path = project_dir.join("http-secret-reveal-allow-e2e");

    let runtime_c_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("runtime")
        .join("c");
    let runtime_source = runtime_c_dir.join("sec4_runtime.c");
    let runtime_include = runtime_c_dir;

    fs::write(
        &harness_path,
        format!(
            r#"#include "sec4_runtime.h"

static int64_t reveal_handler(void) {{
  int64_t secret = sec4_rt_secret_get(1, "SEC4_RT_TEST_SECRET_VALUE");
  if (secret == 0) {{
    sec4_rt_res_text(500, "missing-secret");
    return 0;
  }}
  int64_t revealed = sec4_rt_secret_reveal(1, secret);
  if (revealed == 0) {{
    sec4_rt_res_text(403, "reveal-denied");
    return 0;
  }}
  sec4_rt_res_text(200, "reveal-allowed");
  return 0;
}}

int main(void) {{
  int64_t router = sec4_rt_http_router();
  if (router == 0) {{
    return 11;
  }}
  if (sec4_rt_http_route_get(router, "/reveal", reveal_handler) != 0) {{
    return 12;
  }}
  return (int) sec4_rt_http_serve({}, router);
}}
"#,
            port
        ),
    )
    .expect("runtime harness source should be written");

    let compile_output = Command::new("clang")
        .arg(&harness_path)
        .arg(&runtime_source)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&runtime_include)
        .arg("-o")
        .arg(&binary_path)
        .output()
        .expect("clang should execute for runtime secret reveal allow harness");
    assert!(
        compile_output.status.success(),
        "runtime secret reveal allow harness should compile successfully"
    );

    let mut child = Command::new(&binary_path)
        .env("SEC4_RT_HTTP_SERVE_MODE", "oneshot")
        .env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", "8000")
        .env("SEC4_RT_TEST_SECRET_VALUE", "runtime-secret")
        .env("SEC4_RT_ALLOW_SECRET_REVEAL", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("runtime secret reveal allow harness should start");

    let mut response = None;
    for _ in 0..240 {
        if let Some(status) = child
            .try_wait()
            .expect("child wait should succeed while connecting")
        {
            panic!("runtime secret reveal allow harness exited before request with status: {status}");
        }
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"GET /reveal HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                    )
                    .expect("request should be written");
                let mut body = String::new();
                stream
                    .read_to_string(&mut body)
                    .expect("response should be readable");
                response = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(40)),
        }
    }

    let response = match response {
        Some(response) => response,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            panic!("runtime secret reveal allow harness test could not connect to server");
        }
    };

    let mut status = None;
    for _ in 0..200 {
        match child.try_wait().expect("child wait should succeed") {
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
            panic!("runtime secret reveal allow harness did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "runtime secret reveal allow harness should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "response should contain 200 status line when secret reveal override is enabled"
    );
    assert!(
        response.contains("reveal-allowed"),
        "response should include reveal-allowed body when secret reveal override is enabled"
    );
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

    let output = run_cli(&[
        "run",
        "--path",
        hello_api,
        "--oneshot",
        "--serve-timeout-ms",
        "150",
    ]);
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
fn run_command_help_lists_runtime_bridge_flags() {
    let output = run_cli(&["run", "--help"]);
    assert!(
        output.status.success(),
        "run --help should exit successfully"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(
        stdout.contains("--oneshot"),
        "run --help should list --oneshot runtime bridge flag"
    );
    assert!(
        stdout.contains("--port <PORT>"),
        "run --help should list --port runtime bridge flag"
    );
    assert!(
        stdout.contains("--max-body-bytes <MAX_BODY_BYTES>"),
        "run --help should list --max-body-bytes runtime bridge flag"
    );
    assert!(
        stdout.contains("--serve-timeout-ms <SERVE_TIMEOUT_MS>"),
        "run --help should list --serve-timeout-ms runtime bridge flag"
    );
}

#[test]
fn run_command_serves_http_route_in_oneshot_mode_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping run-command http runtime e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-http-runtime-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runhttpruntimee2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");

    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--serve-timeout-ms",
            "30000",
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
            if !status.success() {
                panic!("run command exited before request with status: {status}");
            }
            break;
        }

        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(mut stream) => {
                stream
                    .write_all(
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
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
            panic!("run command http runtime e2e test could not connect to server");
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
            panic!("run command http runtime e2e process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command http runtime e2e process should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 201 Created"),
        "response should contain 201 status line for req.json + res.ok route"
    );
    assert!(
        response.contains("Content-Type: application/json; charset=utf-8"),
        "response should include JSON content-type"
    );
    assert!(
        response.contains("\"ok\":true")
            && response.contains("\"status\":201")
            && response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic std-success envelope body"
    );
}

#[test]
fn run_command_port_flag_overrides_http_serve_port_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping run-command port-override e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-port-override-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runportoverridee2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
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
            "--port",
            port_value.as_str(),
            "--oneshot",
            "--serve-timeout-ms",
            "30000",
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
                        b"POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
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
            panic!("run command port-override e2e test could not connect to server");
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
            panic!("run command port-override e2e process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command port-override e2e process should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 201 Created"),
        "response should contain 201 status line on overridden runtime port"
    );
    assert!(
        response.contains("\"ok\":true")
            && response.contains("\"status\":201")
            && response.contains("\"traceId\":\"rt-1\""),
        "response should include deterministic std-success envelope body"
    );
}

#[test]
fn run_command_enforces_max_body_bytes_flag_when_clang_available() {
    if !clang_available() {
        eprintln!("skipping run-command max-body-bytes e2e test: clang not available");
        return;
    }

    let project_dir = temp_dir("sec4-run-command-max-body-bytes-e2e");
    let port = find_available_tcp_port();
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "runmaxbodybytese2e"
version = "0.1.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        format!(
            r#"fn createUser() effects {{ net }} -> Int {{
  req.json("CreateUserRequest");
  res.ok(201, "CreateUserResponse", 1);
  0
}}

fn main() effects {{ net }} -> Int {{
  let router = http.router();
  http.post(router, "/users", createUser);
  http.serve({}, router);
  0
}}
"#,
            port
        ),
    )
    .expect("source should be written");

    let path = project_dir
        .to_str()
        .expect("project path should be valid utf-8");
    let constrained_body = "{\"payload\":\"12345678901234567890\"}";
    let request = format!(
        "POST /users HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        constrained_body.len(),
        constrained_body
    );

    let mut child = Command::new(cli_bin())
        .args([
            "run",
            "--path",
            path,
            "--oneshot",
            "--max-body-bytes",
            "16",
            "--serve-timeout-ms",
            "30000",
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
            panic!("run command max-body-bytes e2e test could not connect to server");
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
            panic!("run command max-body-bytes e2e process did not exit in expected window");
        }
    };

    assert!(
        status.success(),
        "run command max-body-bytes e2e process should exit successfully in oneshot mode"
    );
    assert!(
        response.contains("HTTP/1.1 413 Payload Too Large"),
        "response should contain 413 status line when --max-body-bytes is exceeded"
    );
    assert!(
        response.contains("\"code\":\"LIMIT.BODY_BYTES\"")
            && response.contains("\"message\":\"request body exceeds runtime limit\""),
        "response should include deterministic body-limit error payload"
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
