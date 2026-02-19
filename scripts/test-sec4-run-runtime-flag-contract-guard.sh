#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
contract_script="${root_dir}/scripts/test-sec4-run-runtime-flag-contract.sh"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

pass_cli="${tmp_dir}/main-pass.rs"

cat > "${pass_cli}" <<'RS'
#[derive(Clone, Copy, PartialEq, Eq)]
enum RunBackend {
    C,
    Lasm,
}

#[derive(Clone, Copy)]
enum RunDbAdapter {
    RecordsLog,
    Sqlite,
}

enum Commands {
    Run {
        path: String,
        port: Option<u16>,
        oneshot: bool,
        max_body_bytes: Option<u64>,
        serve_timeout_ms: Option<u64>,
        db_adapter: Option<RunDbAdapter>,
        autoscale_saturation_boost_step: usize,
    },
}

struct Command;

impl Command {
    fn new(_path: &String) -> Self {
        Self
    }

    fn env<T>(&mut self, _key: &str, _value: T) {}

    fn arg<T>(&mut self, _value: T) -> &mut Self {
        self
    }
}

struct LasmClusterConfig {
    db_adapter: Option<RunDbAdapter>,
}

fn dispatch(
    path: String,
    port: Option<u16>,
    oneshot: bool,
    max_body_bytes: Option<u64>,
    serve_timeout_ms: Option<u64>,
    db_adapter: Option<RunDbAdapter>,
    autoscale_saturation_boost_step: usize,
) {
    let _ = match Commands::Run {
        path,
        port,
        oneshot,
        max_body_bytes,
        serve_timeout_ms,
        db_adapter,
        autoscale_saturation_boost_step,
    } {
        Commands::Run {
            path,
            port,
            oneshot,
            max_body_bytes,
            serve_timeout_ms,
            db_adapter,
            autoscale_saturation_boost_step,
        } => cmd_run(
            &path,
            port,
            oneshot,
            max_body_bytes,
            serve_timeout_ms,
            db_adapter,
            autoscale_saturation_boost_step,
            RunBackend::Lasm,
        ),
    };
}

fn run_db_adapter_arg_value(adapter: RunDbAdapter) -> &'static str {
    match adapter {
        RunDbAdapter::RecordsLog => "records-log",
        RunDbAdapter::Sqlite => "sqlite",
    }
}

fn run_db_adapter_to_lasm_db_records_adapter(adapter: RunDbAdapter) -> RunDbAdapter {
    adapter
}

fn push_optional_db_adapter_run_arg(cmd: &mut Command, value: Option<RunDbAdapter>) {
    if let Some(value) = value {
        cmd.arg("--db-adapter").arg(run_db_adapter_arg_value(value));
    }
}

fn spawn_lasm_cluster_worker(config: &LasmClusterConfig) {
    let mut cmd = Command::new(&"worker".to_string());
    push_optional_db_adapter_run_arg(&mut cmd, config.db_adapter);
}

fn cmd_run_lasm_backend(db_adapter: Option<RunDbAdapter>) {
    let _mapped = db_adapter.map(run_db_adapter_to_lasm_db_records_adapter);
}

fn cmd_run(
    path: &String,
    port: Option<u16>,
    oneshot: bool,
    max_body_bytes: Option<u64>,
    serve_timeout_ms: Option<u64>,
    db_adapter: Option<RunDbAdapter>,
    autoscale_saturation_boost_step: usize,
    backend: RunBackend,
) -> i32 {
    let mut cmd = Command::new(path);
    if backend != RunBackend::Lasm && db_adapter.is_some() {
        eprintln!("run failed: --db-adapter is only supported with --backend lasm");
    }
    if backend != RunBackend::Lasm && autoscale_saturation_boost_step != 4 {
        eprintln!("run failed: --autoscale-saturation-boost-step is only supported with --backend lasm");
    }
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
    cmd_run_lasm_backend(db_adapter);
    0
}
RS

"${contract_script}" --cli "${pass_cli}" >/dev/null

guard_cli="${tmp_dir}/main-guard-message.rs"
cp "${pass_cli}" "${guard_cli}"
missing_guard_label='run db-adapter lasm-only guard'
perl -0pi -e 's/run failed: --db-adapter is only supported with --backend lasm/run failed: --db-base is only supported with --backend lasm/' "${guard_cli}"

if "${contract_script}" --cli "${guard_cli}" >"${tmp_dir}/guard-message.log" 2>&1; then
  echo "expected run runtime-flag contract failure when db-adapter lasm-only guard drifts" >&2
  exit 1
fi

if ! rg -Fq "missing run-flag contract pattern (${missing_guard_label})" "${tmp_dir}/guard-message.log"; then
  echo "expected missing-pattern diagnostic for ${missing_guard_label}" >&2
  exit 1
fi

saturation_guard_cli="${tmp_dir}/main-saturation-guard.rs"
cp "${pass_cli}" "${saturation_guard_cli}"
missing_saturation_guard_label='run autoscale saturation-boost-step lasm-only guard'
perl -0pi -e 's/run failed: --autoscale-saturation-boost-step is only supported with --backend lasm/run failed: --autoscale-scale-up-step is only supported with --backend lasm/' "${saturation_guard_cli}"

if "${contract_script}" --cli "${saturation_guard_cli}" >"${tmp_dir}/saturation-guard.log" 2>&1; then
  echo "expected run runtime-flag contract failure when autoscale saturation-boost-step lasm-only guard drifts" >&2
  exit 1
fi

if ! rg -Fq "missing run-flag contract pattern (${missing_saturation_guard_label})" "${tmp_dir}/saturation-guard.log"; then
  echo "expected missing-pattern diagnostic for ${missing_saturation_guard_label}" >&2
  exit 1
fi

cluster_cli="${tmp_dir}/main-cluster-bridge.rs"
cp "${pass_cli}" "${cluster_cli}"
missing_cluster_label='run db-adapter cluster bridge'
perl -0pi -e 's/push_optional_db_adapter_run_arg\(&mut cmd, config\.db_adapter\);/push_optional_db_adapter_run_arg\(&mut cmd, None);/' "${cluster_cli}"

if "${contract_script}" --cli "${cluster_cli}" >"${tmp_dir}/cluster-bridge.log" 2>&1; then
  echo "expected run runtime-flag contract failure when db-adapter cluster bridge drifts" >&2
  exit 1
fi

if ! rg -Fq "missing run-flag contract pattern (${missing_cluster_label})" "${tmp_dir}/cluster-bridge.log"; then
  echo "expected missing-pattern diagnostic for ${missing_cluster_label}" >&2
  exit 1
fi

timeout_cli="${tmp_dir}/main-timeout.rs"
cp "${pass_cli}" "${timeout_cli}"
perl -0pi -e 's/SEC4_RT_HTTP_SERVE_TIMEOUT_MS/SEC4_RT_HTTP_TIMEOUT_MS/' "${timeout_cli}"

if "${contract_script}" --cli "${timeout_cli}" >/dev/null 2>&1; then
  echo "expected run runtime-flag contract failure when timeout env bridge drifts" >&2
  exit 1
fi

echo "sec4 run runtime-flag contract guard test passed"
