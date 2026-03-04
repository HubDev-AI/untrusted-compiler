use arc_swap::ArcSwap;
use crossbeam_channel::Sender;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::lasm_cluster_runtime_config::{
    resolve_lasm_cluster_fallback_connect_max_attempts, resolve_lasm_cluster_idle_sleep_micros,
    resolve_lasm_cluster_idle_spin_threshold,
};
use crate::lasm_cluster_status_json::{write_lasm_cluster_status_json, LasmClusterStatusSnapshot};
use crate::lasm_db_config::{
    lasm_db_records_adapter_label, resolve_lasm_dynamic_db_postgres_dsn,
    resolve_lasm_dynamic_db_records_adapter, resolve_lasm_dynamic_db_tx_max_handles,
};
use crate::lasm_db_runtime_dispatch::{
    lasm_db_op_sequence_max_limit, lasm_db_params_max_bytes_limit,
    lasm_db_params_max_entries_limit, lasm_db_query_one_row_max_bytes_limit,
    lasm_db_query_one_row_max_columns_limit, lasm_db_sql_template_max_bytes_limit,
};
use crate::lasm_db_runtime_postgres::{
    lasm_postgres_shared_client_pool_idle_total, lasm_postgres_shared_client_pool_key_count,
};
use crate::lasm_db_runtime_postgres_persist::{
    lasm_postgres_persist_queue_backpressure_total, lasm_postgres_persist_queue_depth,
    lasm_postgres_persist_sync_fallback_total, lasm_postgres_persist_workers_available,
};
use crate::{
    LasmClusterConfig, LasmDbRecordsAdapter, RunDbAdapter, RunDbPostgresPersistQueueFullMode,
    RunDbPostgresTlsMode,
};

pub(crate) struct LasmClusterStatusWriterConfig {
    pub(crate) status_path: Option<PathBuf>,
    pub(crate) stop_flag: Arc<AtomicBool>,
    pub(crate) shared_config: Arc<LasmClusterConfig>,
    pub(crate) active_connections: Arc<AtomicUsize>,
    pub(crate) relay_saturation_events: Arc<AtomicUsize>,
    pub(crate) relay_saturation_events_total: Arc<AtomicU64>,
    pub(crate) relay_senders: Arc<Vec<Sender<TcpStream>>>,
    pub(crate) worker_ports_snapshot: Arc<ArcSwap<Vec<u16>>>,
    pub(crate) worker_ports_generation: Arc<AtomicU64>,
    pub(crate) relay_worker_count: usize,
    pub(crate) relay_queue_capacity: usize,
    pub(crate) relay_queue_shard_capacity: usize,
    pub(crate) relay_buffer_bytes: usize,
    pub(crate) relay_io_burst_max: usize,
    pub(crate) relay_idle_backoff_max: usize,
    pub(crate) relay_buffer_pool_max: usize,
    pub(crate) relay_buffer_pool_prewarm: usize,
    pub(crate) relay_pump_scan_multiplier: usize,
    pub(crate) relay_accept_worker_count: usize,
    pub(crate) relay_dispatch_fallback_total: Arc<AtomicU64>,
    pub(crate) relay_dispatch_saturation_short_circuit_total: Arc<AtomicU64>,
    pub(crate) relay_live_sender_count: Arc<AtomicUsize>,
    pub(crate) relay_pump_connections_total: Arc<AtomicUsize>,
    pub(crate) relay_buffer_pool_entries_total: Arc<AtomicUsize>,
    pub(crate) reusable_ports_count: Arc<AtomicUsize>,
    pub(crate) autoscale_last_desired_instances: Arc<AtomicUsize>,
    pub(crate) autoscale_last_saturation_events: Arc<AtomicUsize>,
    pub(crate) autoscale_last_dynamic_boost_step: Arc<AtomicUsize>,
    pub(crate) autoscale_scale_up_cooldown_remaining_ms: Arc<AtomicU64>,
    pub(crate) autoscale_scale_down_cooldown_remaining_ms: Arc<AtomicU64>,
}

fn lasm_cluster_status_db_postgres_tls_mode_label(
    mode: Option<RunDbPostgresTlsMode>,
) -> Option<&'static str> {
    mode.map(|value| match value {
        RunDbPostgresTlsMode::Auto => "auto",
        RunDbPostgresTlsMode::Disable => "disable",
        RunDbPostgresTlsMode::Require => "require",
    })
}

fn lasm_cluster_status_db_postgres_persist_queue_full_mode_label(
    mode: Option<RunDbPostgresPersistQueueFullMode>,
) -> Option<&'static str> {
    mode.map(|value| match value {
        RunDbPostgresPersistQueueFullMode::Block => "block",
        RunDbPostgresPersistQueueFullMode::SyncFallback => "sync-fallback",
    })
}

fn lasm_cluster_status_db_records_adapter(
    adapter: Option<RunDbAdapter>,
) -> Option<LasmDbRecordsAdapter> {
    match adapter {
        Some(RunDbAdapter::RecordsLog) => Some(LasmDbRecordsAdapter::RecordsLog),
        Some(RunDbAdapter::Sqlite) => Some(LasmDbRecordsAdapter::Sqlite),
        Some(RunDbAdapter::Postgres) => Some(LasmDbRecordsAdapter::Postgres),
        None => None,
    }
}

fn lasm_cluster_status_effective_u64_limit(
    explicit: Option<u64>,
    resolved_default: usize,
) -> Option<u64> {
    explicit.or_else(|| u64::try_from(resolved_default).ok())
}

fn lasm_cluster_status_effective_db_tx_max_handles(explicit: Option<u64>) -> Option<u64> {
    let explicit_usize = explicit.and_then(|value| usize::try_from(value).ok());
    resolve_lasm_dynamic_db_tx_max_handles(explicit_usize)
        .ok()
        .and_then(|value| u64::try_from(value).ok())
}

pub(crate) fn spawn_lasm_cluster_status_writer(
    config: LasmClusterStatusWriterConfig,
) -> Option<std::thread::JoinHandle<()>> {
    let LasmClusterStatusWriterConfig {
        status_path,
        stop_flag,
        shared_config,
        active_connections,
        relay_saturation_events,
        relay_saturation_events_total,
        relay_senders,
        worker_ports_snapshot,
        worker_ports_generation,
        relay_worker_count,
        relay_queue_capacity,
        relay_queue_shard_capacity,
        relay_buffer_bytes,
        relay_io_burst_max,
        relay_idle_backoff_max,
        relay_buffer_pool_max,
        relay_buffer_pool_prewarm,
        relay_pump_scan_multiplier,
        relay_accept_worker_count,
        relay_dispatch_fallback_total,
        relay_dispatch_saturation_short_circuit_total,
        relay_live_sender_count,
        relay_pump_connections_total,
        relay_buffer_pool_entries_total,
        reusable_ports_count,
        autoscale_last_desired_instances,
        autoscale_last_saturation_events,
        autoscale_last_dynamic_boost_step,
        autoscale_scale_up_cooldown_remaining_ms,
        autoscale_scale_down_cooldown_remaining_ms,
    } = config;

    let status_path = status_path?;
    let status_interval_ms = shared_config.autoscale_check_ms.clamp(100, 1000);
    let status_interval_duration = Duration::from_millis(status_interval_ms);
    let status_tmp_path = status_path.with_extension(format!(
        "{}.tmp",
        status_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("json")
    ));

    Some(std::thread::spawn(move || {
        let relay_idle_spin_threshold = resolve_lasm_cluster_idle_spin_threshold();
        let relay_idle_sleep_micros = resolve_lasm_cluster_idle_sleep_micros();
        let mut last_saturation_total = relay_saturation_events_total.load(Ordering::Relaxed);
        let mut last_dispatch_fallback_total =
            relay_dispatch_fallback_total.load(Ordering::Relaxed);
        let mut last_dispatch_saturation_short_circuit_total =
            relay_dispatch_saturation_short_circuit_total.load(Ordering::Relaxed);
        let mut last_saturation_sample_at = Instant::now();
        let mut last_status_snapshot: Option<LasmClusterStatusSnapshot> = None;
        let mut status_parent_ready = false;
        let mut last_status_write_error: Option<String> = None;
        let mut status_write_warning_next_allowed_at: Option<Instant> = None;
        let status_write_warning_throttle_duration = Duration::from_millis(1000);
        let mut selected_worker_ports_snapshot = Arc::clone(&worker_ports_snapshot.load());
        let mut selected_worker_ports_generation = worker_ports_generation.load(Ordering::Relaxed);
        let mut selected_worker_port_count = selected_worker_ports_snapshot.len();
        let relay_fallback_connect_max_attempts =
            resolve_lasm_cluster_fallback_connect_max_attempts();
        let effective_db_max_tx_handles =
            lasm_cluster_status_effective_db_tx_max_handles(shared_config.db_max_tx_handles);
        let effective_db_query_one_row_max_bytes = lasm_cluster_status_effective_u64_limit(
            shared_config.db_query_one_row_max_bytes,
            lasm_db_query_one_row_max_bytes_limit(),
        );
        let effective_db_query_one_row_max_columns = lasm_cluster_status_effective_u64_limit(
            shared_config.db_query_one_row_max_columns,
            lasm_db_query_one_row_max_columns_limit(),
        );
        let effective_db_sql_template_max_bytes = lasm_cluster_status_effective_u64_limit(
            shared_config.db_sql_template_max_bytes,
            lasm_db_sql_template_max_bytes_limit(),
        );
        let effective_db_params_max_bytes = lasm_cluster_status_effective_u64_limit(
            shared_config.db_params_max_bytes,
            lasm_db_params_max_bytes_limit(),
        );
        let effective_db_params_max_entries = lasm_cluster_status_effective_u64_limit(
            shared_config.db_params_max_entries,
            lasm_db_params_max_entries_limit(),
        );
        let effective_db_op_sequence_max = lasm_cluster_status_effective_u64_limit(
            shared_config.db_op_sequence_max,
            lasm_db_op_sequence_max_limit(),
        );
        let db_records_adapter = resolve_lasm_dynamic_db_records_adapter(
            lasm_cluster_status_db_records_adapter(shared_config.db_adapter),
        );
        let effective_db_adapter_label = lasm_db_records_adapter_label(db_records_adapter);
        let db_postgres_dsn_configured = if db_records_adapter == LasmDbRecordsAdapter::Postgres {
            Some(
                resolve_lasm_dynamic_db_postgres_dsn(
                    db_records_adapter,
                    shared_config.db_postgres_dsn.as_deref(),
                    Some(shared_config.path.as_path()),
                )
                .map(|value| value.is_some())
                .unwrap_or(false),
            )
        } else {
            None
        };
        loop {
            let sample_now = Instant::now();
            let saturation_total = relay_saturation_events_total.load(Ordering::Relaxed);
            let saturation_delta = saturation_total.saturating_sub(last_saturation_total);
            let dispatch_fallback_total = relay_dispatch_fallback_total.load(Ordering::Relaxed);
            let dispatch_fallback_delta =
                dispatch_fallback_total.saturating_sub(last_dispatch_fallback_total);
            let dispatch_saturation_short_circuit_total =
                relay_dispatch_saturation_short_circuit_total.load(Ordering::Relaxed);
            let dispatch_saturation_short_circuit_delta = dispatch_saturation_short_circuit_total
                .saturating_sub(last_dispatch_saturation_short_circuit_total);
            let elapsed_secs = sample_now
                .duration_since(last_saturation_sample_at)
                .as_secs_f64()
                .max(0.001);
            let saturation_per_sec = (saturation_delta as f64) / elapsed_secs;
            let dispatch_fallback_per_sec = (dispatch_fallback_delta as f64) / elapsed_secs;
            let dispatch_saturation_short_circuit_per_sec =
                (dispatch_saturation_short_circuit_delta as f64) / elapsed_secs;
            let mut relay_queue_depth = 0usize;
            let mut relay_queue_max_depth = 0usize;
            for sender in relay_senders.iter() {
                let depth = sender.len();
                relay_queue_depth = relay_queue_depth.saturating_add(depth);
                relay_queue_max_depth = relay_queue_max_depth.max(depth);
            }
            let observed_worker_ports_generation = worker_ports_generation.load(Ordering::Relaxed);
            if observed_worker_ports_generation != selected_worker_ports_generation {
                selected_worker_ports_generation = observed_worker_ports_generation;
                if let Some(next_snapshot) = {
                    let snapshot = worker_ports_snapshot.load();
                    if Arc::ptr_eq(&selected_worker_ports_snapshot, &snapshot) {
                        None
                    } else {
                        Some(Arc::clone(&snapshot))
                    }
                } {
                    selected_worker_port_count = next_snapshot.len();
                    selected_worker_ports_snapshot = next_snapshot;
                }
            }
            let worker_count = selected_worker_port_count;
            let active_connections = active_connections.load(Ordering::Relaxed);
            let active_connections_per_worker = if worker_count == 0 {
                0.0
            } else {
                (active_connections as f64) / (worker_count as f64)
            };
            let status_snapshot = LasmClusterStatusSnapshot {
                listen_port: shared_config.listen_port,
                min_instances: shared_config.min_instances,
                max_instances: shared_config.max_instances,
                worker_count,
                relay_worker_count,
                relay_queue_capacity,
                relay_queue_shard_capacity,
                relay_buffer_bytes,
                relay_io_burst_max,
                relay_idle_backoff_max,
                relay_buffer_pool_max,
                relay_buffer_pool_prewarm,
                worker_ports: Arc::clone(&selected_worker_ports_snapshot),
                active_connections,
                active_connections_per_worker,
                relay_saturation_events_pending: relay_saturation_events.load(Ordering::Relaxed),
                relay_saturation_events_total: saturation_total,
                relay_saturation_events_per_sec: saturation_per_sec,
                relay_accept_batch_max: shared_config.cluster_relay_accept_batch_max,
                relay_pump_batch_max: shared_config.cluster_relay_pump_batch_max,
                relay_pump_scan_multiplier,
                relay_selection_reservation_min_chunk: shared_config
                    .cluster_selection_reservation_min_chunk,
                relay_idle_spin_threshold,
                relay_idle_sleep_micros,
                relay_accept_workers: relay_accept_worker_count,
                relay_backend_connect_timeout_ms: shared_config.cluster_backend_connect_timeout_ms,
                relay_backend_connect_cooldown_ms: shared_config
                    .cluster_backend_connect_cooldown_ms,
                relay_fallback_connect_max_attempts,
                db_adapter: Some(effective_db_adapter_label.to_string()),
                db_postgres_dsn_configured,
                db_postgres_tls_mode: lasm_cluster_status_db_postgres_tls_mode_label(
                    shared_config.db_postgres_tls_mode,
                )
                .map(str::to_string),
                db_max_tx_handles: effective_db_max_tx_handles,
                db_records_max: shared_config.db_records_max,
                db_query_one_row_max_bytes: effective_db_query_one_row_max_bytes,
                db_query_one_row_max_columns: effective_db_query_one_row_max_columns,
                db_sql_template_max_bytes: effective_db_sql_template_max_bytes,
                db_params_max_bytes: effective_db_params_max_bytes,
                db_params_max_entries: effective_db_params_max_entries,
                db_op_sequence_max: effective_db_op_sequence_max,
                db_postgres_statement_cache_max: shared_config.db_postgres_statement_cache_max,
                db_postgres_placeholder_cache_max: shared_config.db_postgres_placeholder_cache_max,
                db_postgres_statement_timeout_ms: shared_config.db_postgres_statement_timeout_ms,
                db_postgres_lock_timeout_ms: shared_config.db_postgres_lock_timeout_ms,
                db_postgres_connect_timeout_ms: shared_config.db_postgres_connect_timeout_ms,
                db_postgres_shared_client_max_idle_per_key: shared_config
                    .db_postgres_shared_client_max_idle_per_key,
                db_postgres_shared_client_max_total_idle: shared_config
                    .db_postgres_shared_client_max_total_idle,
                db_postgres_persist_workers: shared_config.db_postgres_persist_workers,
                db_postgres_persist_queue_capacity: shared_config
                    .db_postgres_persist_queue_capacity,
                db_postgres_persist_batch_max: shared_config.db_postgres_persist_batch_max,
                db_postgres_persist_queue_full_mode:
                    lasm_cluster_status_db_postgres_persist_queue_full_mode_label(
                        shared_config.db_postgres_persist_queue_full_mode,
                    )
                    .map(str::to_string),
                db_postgres_shared_client_pool_keys: lasm_postgres_shared_client_pool_key_count(),
                db_postgres_shared_client_pool_idle_total:
                    lasm_postgres_shared_client_pool_idle_total(),
                db_postgres_persist_workers_available: lasm_postgres_persist_workers_available(),
                db_postgres_persist_queue_depth: lasm_postgres_persist_queue_depth(),
                db_postgres_persist_queue_backpressure_total:
                    lasm_postgres_persist_queue_backpressure_total(),
                db_postgres_persist_sync_fallback_total: lasm_postgres_persist_sync_fallback_total(
                ),
                db_sqlite_busy_timeout_ms: shared_config.db_sqlite_busy_timeout_ms,
                db_sqlite_journal_mode: shared_config.db_sqlite_journal_mode.clone(),
                db_sqlite_synchronous: shared_config.db_sqlite_synchronous.clone(),
                db_postgres_retryable_conflict_retry_max: shared_config
                    .db_postgres_retryable_conflict_retry_max,
                db_sqlite_lock_retry_max: shared_config.db_sqlite_lock_retry_max,
                db_sqlite_lock_retry_delay_ms: shared_config.db_sqlite_lock_retry_delay_ms,
                relay_dispatch_fallback_total: dispatch_fallback_total,
                relay_dispatch_fallback_per_sec: dispatch_fallback_per_sec,
                relay_dispatch_saturation_short_circuit_total:
                    dispatch_saturation_short_circuit_total,
                relay_dispatch_saturation_short_circuit_per_sec:
                    dispatch_saturation_short_circuit_per_sec,
                relay_live_sender_count: relay_live_sender_count.load(Ordering::Relaxed),
                relay_queue_depth,
                relay_queue_max_depth,
                relay_pump_connections: relay_pump_connections_total.load(Ordering::Relaxed),
                relay_buffer_pool_entries: relay_buffer_pool_entries_total.load(Ordering::Relaxed),
                reusable_ports_count: reusable_ports_count.load(Ordering::Relaxed),
                autoscale_desired_instances: autoscale_last_desired_instances
                    .load(Ordering::Relaxed),
                autoscale_last_saturation_events: autoscale_last_saturation_events
                    .load(Ordering::Relaxed),
                autoscale_last_dynamic_boost_step: autoscale_last_dynamic_boost_step
                    .load(Ordering::Relaxed),
                autoscale_scale_up_cooldown_remaining_ms: autoscale_scale_up_cooldown_remaining_ms
                    .load(Ordering::Relaxed),
                autoscale_scale_down_cooldown_remaining_ms:
                    autoscale_scale_down_cooldown_remaining_ms.load(Ordering::Relaxed),
            };
            if let Err(err) = write_lasm_cluster_status_json(
                status_path.as_path(),
                status_tmp_path.as_path(),
                status_snapshot,
                &mut last_status_snapshot,
                &mut status_parent_ready,
            ) {
                let error_message = err;
                let warning_allowed = match status_write_warning_next_allowed_at {
                    Some(next_allowed_at) => sample_now >= next_allowed_at,
                    None => true,
                };
                let should_log = warning_allowed
                    || last_status_write_error
                        .as_deref()
                        .map(|last| last != error_message)
                        .unwrap_or(true);
                if should_log {
                    eprintln!("warning: LASM cluster status json write failed: {error_message}");
                    status_write_warning_next_allowed_at =
                        Some(sample_now + status_write_warning_throttle_duration);
                    last_status_write_error = Some(error_message.to_string());
                }
            } else {
                last_status_write_error = None;
                status_write_warning_next_allowed_at = None;
            }
            last_saturation_total = saturation_total;
            last_dispatch_fallback_total = dispatch_fallback_total;
            last_dispatch_saturation_short_circuit_total = dispatch_saturation_short_circuit_total;
            last_saturation_sample_at = sample_now;

            if stop_flag.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(status_interval_duration);
        }
    }))
}
