#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-sec4-run-runtime-flag-contract.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

cli_path="${tmp_dir}/main.rs"

cat > "${cli_path}" <<'RS'
enum Commands {
    Run {
        path: String,
        port: Option<u16>,
        oneshot: bool,
        max_body_bytes: Option<u64>,
        serve_timeout_ms: Option<u64>,
    },
}

fn dispatch(path: String, port: Option<u16>, oneshot: bool, max_body_bytes: Option<u64>, serve_timeout_ms: Option<u64>) {
    let _ = match Commands::Run { path, port, oneshot, max_body_bytes, serve_timeout_ms } {
        Commands::Run { path, port, oneshot, max_body_bytes, serve_timeout_ms } => cmd_run(&path, port, oneshot, max_body_bytes, serve_timeout_ms),
    };
}

fn cmd_run(path: &String, port: Option<u16>, oneshot: bool, max_body_bytes: Option<u64>, serve_timeout_ms: Option<u64>) -> i32 {
    let mut cmd = Command::new(path);
    if let Some(port) = port {
        cmd.env("SEC4_RT_HTTP_PORT", port.to_string());
    }
    if oneshot {
        cmd.env("SEC4_RT_HTTP_SERVE_MODE", "oneshot");
    }
    if let Some(bytes) = max_body_bytes {
        cmd.env("SEC4_RT_HTTP_MAX_BODY_BYTES", bytes.to_string());
    }
    if let Some(timeout_ms) = serve_timeout_ms {
        cmd.env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", timeout_ms.to_string());
    }
    0
}
RS

"${contract_script}" --cli "${cli_path}" >/dev/null

cat > "${cli_path}" <<'RS'
enum Commands {
    Run {
        path: String,
        port: Option<u16>,
        oneshot: bool,
        max_body_bytes: Option<u64>,
        serve_timeout_ms: Option<u64>,
    },
}

fn dispatch(path: String, port: Option<u16>, oneshot: bool, max_body_bytes: Option<u64>, serve_timeout_ms: Option<u64>) {
    let _ = match Commands::Run { path, port, oneshot, max_body_bytes, serve_timeout_ms } {
        Commands::Run { path, port, oneshot, max_body_bytes, serve_timeout_ms } => cmd_run(&path, oneshot, max_body_bytes, serve_timeout_ms),
    };
}

fn cmd_run(path: &String, oneshot: bool, max_body_bytes: Option<u64>, serve_timeout_ms: Option<u64>) -> i32 {
    let mut cmd = Command::new(path);
    if oneshot {
        cmd.env("SEC4_RT_HTTP_SERVE_MODE", "oneshot");
    }
    if let Some(bytes) = max_body_bytes {
        cmd.env("SEC4_RT_HTTP_MAX_BODY_BYTES", bytes.to_string());
    }
    if let Some(timeout_ms) = serve_timeout_ms {
        cmd.env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", timeout_ms.to_string());
    }
    0
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected run runtime-flag contract failure when dispatch omits port forwarding" >&2
  exit 1
fi

cat > "${cli_path}" <<'RS'
enum Commands {
    Run {
        path: String,
        port: Option<u16>,
        oneshot: bool,
        max_body_bytes: Option<u64>,
        serve_timeout_ms: Option<u64>,
    },
}

fn dispatch(path: String, port: Option<u16>, oneshot: bool, max_body_bytes: Option<u64>, serve_timeout_ms: Option<u64>) {
    let _ = match Commands::Run { path, port, oneshot, max_body_bytes, serve_timeout_ms } {
        Commands::Run { path, port, oneshot, max_body_bytes, serve_timeout_ms } => cmd_run(&path, port, oneshot, max_body_bytes, serve_timeout_ms),
    };
}

fn cmd_run(path: &String, port: Option<u16>, oneshot: bool, max_body_bytes: Option<u64>, serve_timeout_ms: Option<u64>) -> i32 {
    let mut cmd = Command::new(path);
    if let Some(port) = port {
        cmd.env("SEC4_RT_HTTP_PORT", port.to_string());
    }
    if oneshot {
        cmd.env("SEC4_RT_HTTP_SERVE_MODE", "oneshot");
    }
    if let Some(bytes) = max_body_bytes {
        cmd.env("SEC4_RT_HTTP_MAX_BODY_BYTES", bytes.to_string());
    }
    if let Some(timeout_ms) = serve_timeout_ms {
        cmd.env("SEC4_RT_HTTP_TIMEOUT_MS", timeout_ms.to_string());
    }
    0
}
RS

if "${contract_script}" --cli "${cli_path}" >/dev/null 2>&1; then
  echo "expected run runtime-flag contract failure when timeout env bridge drifts" >&2
  exit 1
fi

echo "sec4 run runtime-flag contract guard test passed"
