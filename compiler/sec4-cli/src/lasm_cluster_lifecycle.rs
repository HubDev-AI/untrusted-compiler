use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::{
    push_optional_db_adapter_run_arg, push_optional_db_postgres_tls_mode_run_arg,
    LasmClusterConfig, LasmClusterState, LasmClusterWorker,
};

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
        "--db-sqlite-busy-timeout-ms",
        config.db_sqlite_busy_timeout_ms,
    );
    if let Some(dsn) = config.db_postgres_dsn.as_deref() {
        cmd.env("SEC4_RT_LASM_DB_POSTGRES_DSN", dsn);
    }

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

pub(crate) fn prune_dead_lasm_cluster_workers(state: &mut LasmClusterState) {
    let mut kept = Vec::with_capacity(state.workers.len());
    for mut worker in state.workers.drain(..) {
        match worker.child.try_wait() {
            Ok(Some(status)) => {
                eprintln!(
                    "warning: LASM cluster worker on port {} exited: {}",
                    worker.port, status
                );
            }
            Ok(None) => kept.push(worker),
            Err(err) => {
                eprintln!(
                    "warning: could not inspect LASM cluster worker on port {}: {}",
                    worker.port, err
                );
            }
        }
    }
    state.workers = kept;
}

pub(crate) fn reserve_lasm_cluster_min_worker_ports(
    state: &mut LasmClusterState,
    min_instances: usize,
) -> Vec<u16> {
    let missing = min_instances.saturating_sub(state.workers.len());
    let mut worker_ports = Vec::with_capacity(missing);
    for _ in 0..missing {
        let worker_port = state.next_port;
        state.next_port = state.next_port.saturating_add(1);
        worker_ports.push(worker_port);
    }
    worker_ports
}

pub(crate) fn stop_lasm_cluster_workers(state: &mut LasmClusterState) {
    for worker in &mut state.workers {
        let _ = worker.child.kill();
        let _ = worker.child.wait();
    }
    state.workers.clear();
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
        prune_dead_lasm_cluster_workers(&mut state);
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
