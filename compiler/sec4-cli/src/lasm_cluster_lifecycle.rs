use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::lasm_db_config::{
    LASM_DB_POSTGRES_DSN_FILE_KEYS, LASM_DB_POSTGRES_DSN_KEYS, LASM_DB_POSTGRES_RUNTIME_ENV_KEYS,
};
use crate::{
    push_optional_db_adapter_run_arg, push_optional_db_postgres_persist_queue_full_mode_run_arg,
    push_optional_db_postgres_tls_mode_run_arg, LasmClusterConfig, LasmClusterState,
    LasmClusterWorker,
};

fn forward_env_if_set(cmd: &mut Command, env_key: &str) {
    if let Ok(value) = std::env::var(env_key) {
        let value = value.trim();
        if !value.is_empty() {
            cmd.env(env_key, value);
        }
    }
}

fn forward_db_alias_env_vars_to_worker(cmd: &mut Command) {
    LASM_DB_POSTGRES_DSN_KEYS
        .iter()
        .for_each(|env_key| forward_env_if_set(cmd, env_key));
    LASM_DB_POSTGRES_DSN_FILE_KEYS
        .iter()
        .for_each(|env_key| forward_env_if_set(cmd, env_key));
    LASM_DB_POSTGRES_RUNTIME_ENV_KEYS
        .iter()
        .for_each(|env_key| forward_env_if_set(cmd, env_key));
}

pub(crate) fn compute_lasm_cluster_base_port(
    listen_port: u16,
    max_instances: usize,
) -> Result<u16, String> {
    let base_port = u32::from(listen_port) + 100;
    let needed_span = u32::try_from(max_instances.saturating_sub(1))
        .map_err(|_| "cluster instance count exceeds supported port span".to_string())?;
    let last_port = base_port + needed_span;
    if last_port > u32::from(u16::MAX) {
        return Err(
            "cannot allocate worker ports: choose a lower --port for cluster mode".to_string(),
        );
    }
    Ok(u16::try_from(base_port).expect("base port range prevalidated"))
}

fn push_optional_u64_run_arg(cmd: &mut Command, flag: &str, value: Option<u64>) {
    if let Some(value) = value {
        cmd.arg(flag).arg(value.to_string());
    }
}

fn push_optional_path_run_arg(cmd: &mut Command, flag: &str, value: Option<&Path>) {
    if let Some(value) = value {
        cmd.arg(flag).arg(value);
    }
}

fn push_optional_string_run_arg(cmd: &mut Command, flag: &str, value: Option<&str>) {
    if let Some(value) = value {
        cmd.arg(flag).arg(value);
    }
}

fn spawn_lasm_cluster_worker(
    config: &LasmClusterConfig,
    worker_port: u16,
) -> Result<Child, String> {
    let current_exe = std::env::current_exe().map_err(|err| {
        format!("could not resolve current sec4 executable for cluster mode: {err}")
    })?;
    let mut cmd = Command::new(current_exe);
    cmd.arg("run")
        .arg("--path")
        .arg(&config.path)
        .arg("--backend")
        .arg("lasm")
        .arg("--port")
        .arg(worker_port.to_string())
        .arg("--instances")
        .arg("1");
    if config.reuse_port_workers {
        cmd.arg("--reuse-port");
    }

    push_optional_u64_run_arg(&mut cmd, "--max-header-bytes", config.max_header_bytes);
    push_optional_u64_run_arg(&mut cmd, "--max-body-bytes", config.max_body_bytes);
    push_optional_u64_run_arg(&mut cmd, "--max-concurrency", config.max_concurrency);
    push_optional_u64_run_arg(&mut cmd, "--max-pending", config.max_pending);
    push_optional_u64_run_arg(&mut cmd, "--serve-timeout-ms", config.serve_timeout_ms);
    push_optional_u64_run_arg(
        &mut cmd,
        "--overflow-probe-timeout-ms",
        config.overflow_probe_timeout_ms,
    );
    push_optional_u64_run_arg(&mut cmd, "--max-runtime-steps", config.max_runtime_steps);
    push_optional_u64_run_arg(
        &mut cmd,
        "--max-keep-alive-requests",
        config.max_keep_alive_requests,
    );
    push_optional_path_run_arg(&mut cmd, "--db-base", config.db_base.as_deref());
    push_optional_db_adapter_run_arg(&mut cmd, config.db_adapter);
    push_optional_db_postgres_tls_mode_run_arg(&mut cmd, config.db_postgres_tls_mode);
    push_optional_u64_run_arg(&mut cmd, "--db-max-tx-handles", config.db_max_tx_handles);
    push_optional_u64_run_arg(&mut cmd, "--db-records-max", config.db_records_max);
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-query-one-row-max-bytes",
        config.db_query_one_row_max_bytes,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-query-one-row-max-columns",
        config.db_query_one_row_max_columns,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-sql-template-max-bytes",
        config.db_sql_template_max_bytes,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-params-max-bytes",
        config.db_params_max_bytes,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-params-max-entries",
        config.db_params_max_entries,
    );
    push_optional_u64_run_arg(&mut cmd, "--db-op-sequence-max", config.db_op_sequence_max);
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-statement-cache-max",
        config.db_postgres_statement_cache_max,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-placeholder-cache-max",
        config.db_postgres_placeholder_cache_max,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-statement-timeout-ms",
        config.db_postgres_statement_timeout_ms,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-lock-timeout-ms",
        config.db_postgres_lock_timeout_ms,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-connect-timeout-ms",
        config.db_postgres_connect_timeout_ms,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-shared-client-max-idle-per-key",
        config.db_postgres_shared_client_max_idle_per_key,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-shared-client-max-total-idle",
        config.db_postgres_shared_client_max_total_idle,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-persist-workers",
        config.db_postgres_persist_workers,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-persist-queue-capacity",
        config.db_postgres_persist_queue_capacity,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-persist-batch-max",
        config.db_postgres_persist_batch_max,
    );
    push_optional_db_postgres_persist_queue_full_mode_run_arg(
        &mut cmd,
        config.db_postgres_persist_queue_full_mode,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-sqlite-busy-timeout-ms",
        config.db_sqlite_busy_timeout_ms,
    );
    push_optional_string_run_arg(
        &mut cmd,
        "--db-sqlite-journal-mode",
        config.db_sqlite_journal_mode.as_deref(),
    );
    push_optional_string_run_arg(
        &mut cmd,
        "--db-sqlite-synchronous",
        config.db_sqlite_synchronous.as_deref(),
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-postgres-retryable-conflict-retry-max",
        config.db_postgres_retryable_conflict_retry_max,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-sqlite-lock-retry-max",
        config.db_sqlite_lock_retry_max,
    );
    push_optional_u64_run_arg(
        &mut cmd,
        "--db-sqlite-lock-retry-delay-ms",
        config.db_sqlite_lock_retry_delay_ms,
    );
    if let Some(dsn) = config
        .db_postgres_dsn
        .as_deref()
        .map(|dsn| dsn.trim())
        .filter(|dsn| !dsn.is_empty())
        .map(|dsn| dsn.to_string())
        .or_else(|| {
            std::env::var("SEC4_DB_ALPHA_DB_POSTGRES_DSN")
                .ok()
                .map(|dsn| dsn.trim().to_string())
                .filter(|dsn| !dsn.is_empty())
        })
        .or_else(|| {
            std::env::var("SEC4_RT_LASM_DB_POSTGRES_DSN")
                .ok()
                .map(|dsn| dsn.trim().to_string())
                .filter(|dsn| !dsn.is_empty())
        })
    {
        cmd.env("SEC4_DB_ALPHA_DB_POSTGRES_DSN", &dsn);
        cmd.env("SEC4_RT_LASM_DB_POSTGRES_DSN", &dsn);
    }

    forward_db_alias_env_vars_to_worker(&mut cmd);

    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    cmd.spawn()
        .map_err(|err| format!("could not spawn LASM cluster worker on port {worker_port}: {err}"))
}

fn wait_for_lasm_cluster_worker_ready(
    child: &mut Child,
    worker_port: u16,
    timeout_ms: u64,
) -> Result<(), String> {
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms.max(200));
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!(
                    "LASM cluster worker on port {worker_port} exited early with status {status}"
                ));
            }
            Ok(None) => {}
            Err(err) => {
                return Err(format!(
                    "could not check LASM cluster worker status on port {worker_port}: {err}"
                ));
            }
        }

        match TcpStream::connect(("127.0.0.1", worker_port)) {
            Ok(stream) => {
                let _ = stream.shutdown(Shutdown::Both);
                return Ok(());
            }
            Err(_) => {
                if start.elapsed() >= timeout {
                    return Err(format!(
                        "LASM cluster worker on port {worker_port} did not become ready within {} ms",
                        timeout.as_millis()
                    ));
                }
                std::thread::sleep(Duration::from_millis(40));
            }
        }
    }
}

fn wait_for_lasm_cluster_worker_alive(
    child: &mut Child,
    worker_port: u16,
    timeout_ms: u64,
) -> Result<(), String> {
    let grace_ms = timeout_ms.clamp(200, 2000);
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!(
                    "LASM cluster worker on port {worker_port} exited early with status {status}"
                ));
            }
            Ok(None) => {
                if start.elapsed() >= Duration::from_millis(grace_ms) {
                    return Ok(());
                }
            }
            Err(err) => {
                return Err(format!(
                    "could not check LASM cluster worker status on port {worker_port}: {err}"
                ));
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

pub(crate) fn spawn_and_wait_lasm_cluster_worker(
    config: &LasmClusterConfig,
    worker_port: u16,
) -> Result<LasmClusterWorker, String> {
    let mut child = spawn_lasm_cluster_worker(config, worker_port)?;
    let wait_result = if config.reuse_port_workers {
        wait_for_lasm_cluster_worker_alive(&mut child, worker_port, config.worker_ready_timeout_ms)
    } else {
        wait_for_lasm_cluster_worker_ready(&mut child, worker_port, config.worker_ready_timeout_ms)
    };
    if let Err(err) = wait_result {
        let _ = child.kill();
        let _ = child.wait();
        return Err(err);
    }
    Ok(LasmClusterWorker {
        port: worker_port,
        child,
    })
}

pub(crate) fn prune_dead_lasm_cluster_workers(state: &mut LasmClusterState) -> bool {
    let mut workers_changed = false;
    let mut worker_index = 0usize;
    while worker_index < state.workers.len() {
        let worker_port = state.workers[worker_index].port;
        match state.workers[worker_index].child.try_wait() {
            Ok(Some(status)) => {
                eprintln!(
                    "warning: LASM cluster worker on port {} exited: {}",
                    worker_port, status
                );
                state.workers.swap_remove(worker_index);
                state.reusable_ports.push(worker_port);
                workers_changed = true;
            }
            Ok(None) => {
                worker_index += 1;
            }
            Err(err) => {
                eprintln!(
                    "warning: could not inspect LASM cluster worker on port {}: {}",
                    worker_port, err
                );
                state.workers.swap_remove(worker_index);
                state.reusable_ports.push(worker_port);
                workers_changed = true;
            }
        }
    }
    workers_changed
}

pub(crate) fn reserve_lasm_cluster_min_worker_ports(
    state: &mut LasmClusterState,
    min_instances: usize,
    worker_ports_out: &mut Vec<u16>,
) {
    let missing = min_instances.saturating_sub(state.workers.len());
    reserve_lasm_cluster_worker_ports(state, missing, worker_ports_out);
}

pub(crate) fn reserve_lasm_cluster_worker_ports(
    state: &mut LasmClusterState,
    reserve_count: usize,
    worker_ports_out: &mut Vec<u16>,
) {
    worker_ports_out.reserve(reserve_count);
    let reusable_count = reserve_count.min(state.reusable_ports.len());
    if reusable_count > 0 {
        let reusable_start = state.reusable_ports.len() - reusable_count;
        worker_ports_out.extend_from_slice(&state.reusable_ports[reusable_start..]);
        state.reusable_ports.truncate(reusable_start);
    }
    let fresh_count = reserve_count.saturating_sub(reusable_count);
    for _ in 0..fresh_count {
        let worker_port = state.next_port;
        state.next_port = state.next_port.saturating_add(1);
        worker_ports_out.push(worker_port);
    }
}

pub(crate) fn stop_lasm_cluster_workers(state: &mut LasmClusterState) {
    for worker in &mut state.workers {
        let _ = worker.child.kill();
        let _ = worker.child.wait();
    }
    state.workers.clear();
    state.reusable_ports.clear();
}

pub(crate) fn bind_lasm_listener(
    listen_port: u16,
    reuse_port: bool,
) -> Result<TcpListener, String> {
    if !reuse_port {
        return TcpListener::bind(("127.0.0.1", listen_port)).map_err(|err| {
            format!("could not bind LASM backend listener on 127.0.0.1:{listen_port}: {err}")
        });
    }

    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
        .map_err(|err| format!("could not create LASM reuse-port socket: {err}"))?;
    socket.set_reuse_address(true).map_err(|err| {
        format!("could not set LASM reuse-port socket option SO_REUSEADDR: {err}")
    })?;
    #[cfg(unix)]
    socket.set_reuse_port(true).map_err(|err| {
        format!("could not set LASM reuse-port socket option SO_REUSEPORT: {err}")
    })?;

    let addr = std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, listen_port);
    socket.bind(&addr.into()).map_err(|err| {
        format!("could not bind LASM reuse-port listener on 127.0.0.1:{listen_port}: {err}")
    })?;
    socket.listen(1024).map_err(|err| {
        format!("could not listen on LASM reuse-port listener 127.0.0.1:{listen_port}: {err}")
    })?;

    Ok(socket.into())
}

pub(crate) fn cmd_run_lasm_reuseport_cluster(config: LasmClusterConfig) -> Result<(), i32> {
    let mut state = LasmClusterState {
        workers: Vec::new(),
        next_port: config.listen_port,
        reusable_ports: Vec::new(),
    };
    for _ in 0..config.min_instances {
        match spawn_and_wait_lasm_cluster_worker(&config, config.listen_port) {
            Ok(worker) => state.workers.push(worker),
            Err(message) => {
                stop_lasm_cluster_workers(&mut state);
                eprintln!("run failed: {message}");
                return Err(2);
            }
        }
    }
    if state.workers.is_empty() {
        eprintln!("run failed: could not bootstrap LASM reuse-port workers");
        return Err(2);
    }

    loop {
        std::thread::sleep(Duration::from_millis(config.autoscale_check_ms.max(200)));
        let _ = prune_dead_lasm_cluster_workers(&mut state);
        while state.workers.len() < config.min_instances {
            match spawn_and_wait_lasm_cluster_worker(&config, config.listen_port) {
                Ok(worker) => state.workers.push(worker),
                Err(message) => {
                    eprintln!("warning: LASM reuse-port worker recovery failed: {message}");
                    break;
                }
            }
        }
    }
}
