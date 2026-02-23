use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub(crate) struct LasmClusterStatusSnapshot {
    pub(crate) listen_port: u16,
    pub(crate) min_instances: usize,
    pub(crate) max_instances: usize,
    pub(crate) worker_count: usize,
    pub(crate) relay_worker_count: usize,
    pub(crate) relay_queue_capacity: usize,
    pub(crate) relay_queue_shard_capacity: usize,
    pub(crate) relay_buffer_bytes: usize,
    pub(crate) relay_buffer_pool_max: usize,
    pub(crate) relay_buffer_pool_prewarm: usize,
    pub(crate) worker_ports: Arc<Vec<u16>>,
    pub(crate) active_connections: usize,
    pub(crate) active_connections_per_worker: f64,
    pub(crate) relay_saturation_events_pending: usize,
    pub(crate) relay_saturation_events_total: u64,
    pub(crate) relay_saturation_events_per_sec: f64,
    pub(crate) relay_accept_batch_max: usize,
    pub(crate) relay_pump_batch_max: usize,
    pub(crate) relay_selection_reservation_min_chunk: usize,
    pub(crate) relay_idle_spin_threshold: u32,
    pub(crate) relay_idle_sleep_micros: u64,
    pub(crate) relay_accept_workers: usize,
    pub(crate) relay_backend_connect_timeout_ms: u64,
    pub(crate) relay_backend_connect_cooldown_ms: u64,
    pub(crate) db_adapter: Option<String>,
    pub(crate) db_postgres_tls_mode: Option<String>,
    pub(crate) db_max_tx_handles: Option<u64>,
    pub(crate) db_records_max: Option<u64>,
    pub(crate) db_postgres_statement_cache_max: Option<u64>,
    pub(crate) db_postgres_placeholder_cache_max: Option<u64>,
    pub(crate) db_postgres_statement_timeout_ms: Option<u64>,
    pub(crate) db_postgres_lock_timeout_ms: Option<u64>,
    pub(crate) db_postgres_connect_timeout_ms: Option<u64>,
    pub(crate) db_sqlite_busy_timeout_ms: Option<u64>,
    pub(crate) db_sqlite_journal_mode: Option<String>,
    pub(crate) db_sqlite_synchronous: Option<String>,
    pub(crate) db_postgres_retryable_conflict_retry_max: Option<u64>,
    pub(crate) db_sqlite_lock_retry_max: Option<u64>,
    pub(crate) db_sqlite_lock_retry_delay_ms: Option<u64>,
    pub(crate) relay_dispatch_fallback_total: u64,
    pub(crate) relay_dispatch_fallback_per_sec: f64,
    pub(crate) relay_dispatch_saturation_short_circuit_total: u64,
    pub(crate) relay_dispatch_saturation_short_circuit_per_sec: f64,
    pub(crate) relay_live_sender_count: usize,
    pub(crate) relay_queue_depth: usize,
    pub(crate) relay_queue_max_depth: usize,
    pub(crate) relay_pump_connections: usize,
    pub(crate) relay_buffer_pool_entries: usize,
    pub(crate) reusable_ports_count: usize,
    pub(crate) autoscale_desired_instances: usize,
    pub(crate) autoscale_last_saturation_events: usize,
    pub(crate) autoscale_last_dynamic_boost_step: usize,
    pub(crate) autoscale_scale_up_cooldown_remaining_ms: u64,
    pub(crate) autoscale_scale_down_cooldown_remaining_ms: u64,
}

impl PartialEq for LasmClusterStatusSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.listen_port == other.listen_port
            && self.min_instances == other.min_instances
            && self.max_instances == other.max_instances
            && self.worker_count == other.worker_count
            && self.relay_worker_count == other.relay_worker_count
            && self.relay_queue_capacity == other.relay_queue_capacity
            && self.relay_queue_shard_capacity == other.relay_queue_shard_capacity
            && self.relay_buffer_bytes == other.relay_buffer_bytes
            && self.relay_buffer_pool_max == other.relay_buffer_pool_max
            && self.relay_buffer_pool_prewarm == other.relay_buffer_pool_prewarm
            && (Arc::ptr_eq(&self.worker_ports, &other.worker_ports)
                || self.worker_ports.as_slice() == other.worker_ports.as_slice())
            && self.active_connections == other.active_connections
            && self.active_connections_per_worker == other.active_connections_per_worker
            && self.relay_saturation_events_pending == other.relay_saturation_events_pending
            && self.relay_saturation_events_total == other.relay_saturation_events_total
            && self.relay_saturation_events_per_sec == other.relay_saturation_events_per_sec
            && self.relay_accept_batch_max == other.relay_accept_batch_max
            && self.relay_pump_batch_max == other.relay_pump_batch_max
            && self.relay_selection_reservation_min_chunk
                == other.relay_selection_reservation_min_chunk
            && self.relay_idle_spin_threshold == other.relay_idle_spin_threshold
            && self.relay_idle_sleep_micros == other.relay_idle_sleep_micros
            && self.relay_accept_workers == other.relay_accept_workers
            && self.relay_backend_connect_timeout_ms == other.relay_backend_connect_timeout_ms
            && self.relay_backend_connect_cooldown_ms == other.relay_backend_connect_cooldown_ms
            && self.db_adapter == other.db_adapter
            && self.db_postgres_tls_mode == other.db_postgres_tls_mode
            && self.db_max_tx_handles == other.db_max_tx_handles
            && self.db_records_max == other.db_records_max
            && self.db_postgres_statement_cache_max == other.db_postgres_statement_cache_max
            && self.db_postgres_placeholder_cache_max == other.db_postgres_placeholder_cache_max
            && self.db_postgres_statement_timeout_ms == other.db_postgres_statement_timeout_ms
            && self.db_postgres_lock_timeout_ms == other.db_postgres_lock_timeout_ms
            && self.db_postgres_connect_timeout_ms == other.db_postgres_connect_timeout_ms
            && self.db_sqlite_busy_timeout_ms == other.db_sqlite_busy_timeout_ms
            && self.db_sqlite_journal_mode == other.db_sqlite_journal_mode
            && self.db_sqlite_synchronous == other.db_sqlite_synchronous
            && self.db_postgres_retryable_conflict_retry_max
                == other.db_postgres_retryable_conflict_retry_max
            && self.db_sqlite_lock_retry_max == other.db_sqlite_lock_retry_max
            && self.db_sqlite_lock_retry_delay_ms == other.db_sqlite_lock_retry_delay_ms
            && self.relay_dispatch_fallback_total == other.relay_dispatch_fallback_total
            && self.relay_dispatch_fallback_per_sec == other.relay_dispatch_fallback_per_sec
            && self.relay_dispatch_saturation_short_circuit_total
                == other.relay_dispatch_saturation_short_circuit_total
            && self.relay_dispatch_saturation_short_circuit_per_sec
                == other.relay_dispatch_saturation_short_circuit_per_sec
            && self.relay_live_sender_count == other.relay_live_sender_count
            && self.relay_queue_depth == other.relay_queue_depth
            && self.relay_queue_max_depth == other.relay_queue_max_depth
            && self.relay_pump_connections == other.relay_pump_connections
            && self.relay_buffer_pool_entries == other.relay_buffer_pool_entries
            && self.reusable_ports_count == other.reusable_ports_count
            && self.autoscale_desired_instances == other.autoscale_desired_instances
            && self.autoscale_last_saturation_events == other.autoscale_last_saturation_events
            && self.autoscale_last_dynamic_boost_step == other.autoscale_last_dynamic_boost_step
            && self.autoscale_scale_up_cooldown_remaining_ms
                == other.autoscale_scale_up_cooldown_remaining_ms
            && self.autoscale_scale_down_cooldown_remaining_ms
                == other.autoscale_scale_down_cooldown_remaining_ms
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LasmClusterStatusPayload<'a> {
    mode: &'static str,
    updated_at_ms: u64,
    listen_port: u16,
    min_instances: usize,
    max_instances: usize,
    worker_count: usize,
    relay_worker_count: usize,
    relay_queue_capacity: usize,
    relay_queue_shard_capacity: usize,
    relay_buffer_bytes: usize,
    relay_buffer_pool_max: usize,
    relay_buffer_pool_prewarm: usize,
    worker_ports: &'a [u16],
    active_connections: usize,
    active_connections_per_worker: f64,
    relay_saturation_events_pending: usize,
    relay_saturation_events_total: u64,
    relay_saturation_events_per_sec: f64,
    relay_accept_batch_max: usize,
    relay_pump_batch_max: usize,
    relay_selection_reservation_min_chunk: usize,
    relay_idle_spin_threshold: u32,
    relay_idle_sleep_micros: u64,
    relay_accept_workers: usize,
    relay_backend_connect_timeout_ms: u64,
    relay_backend_connect_cooldown_ms: u64,
    db_adapter: Option<&'a str>,
    db_postgres_tls_mode: Option<&'a str>,
    db_max_tx_handles: Option<u64>,
    db_records_max: Option<u64>,
    db_postgres_statement_cache_max: Option<u64>,
    db_postgres_placeholder_cache_max: Option<u64>,
    db_postgres_statement_timeout_ms: Option<u64>,
    db_postgres_lock_timeout_ms: Option<u64>,
    db_postgres_connect_timeout_ms: Option<u64>,
    db_sqlite_busy_timeout_ms: Option<u64>,
    db_sqlite_journal_mode: Option<&'a str>,
    db_sqlite_synchronous: Option<&'a str>,
    db_postgres_retryable_conflict_retry_max: Option<u64>,
    db_sqlite_lock_retry_max: Option<u64>,
    db_sqlite_lock_retry_delay_ms: Option<u64>,
    relay_dispatch_fallback_total: u64,
    relay_dispatch_fallback_per_sec: f64,
    relay_dispatch_saturation_short_circuit_total: u64,
    relay_dispatch_saturation_short_circuit_per_sec: f64,
    relay_live_sender_count: usize,
    relay_queue_depth: usize,
    relay_queue_max_depth: usize,
    relay_pump_connections: usize,
    relay_buffer_pool_entries: usize,
    reusable_ports_count: usize,
    autoscale_desired_instances: usize,
    autoscale_last_saturation_events: usize,
    autoscale_last_dynamic_boost_step: usize,
    autoscale_scale_up_cooldown_remaining_ms: u64,
    autoscale_scale_down_cooldown_remaining_ms: u64,
}

fn lasm_cluster_status_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

pub(crate) fn write_lasm_cluster_status_json(
    path: &Path,
    tmp_path: &Path,
    snapshot: LasmClusterStatusSnapshot,
    last_snapshot: &mut Option<LasmClusterStatusSnapshot>,
    status_parent_ready: &mut bool,
) -> Result<(), String> {
    if last_snapshot
        .as_ref()
        .map(|previous| previous == &snapshot)
        .unwrap_or(false)
    {
        return Ok(());
    }

    let ensure_status_parent_dir = |ready: &mut bool| -> Result<(), String> {
        if *ready {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|err| {
                    format!(
                        "could not create cluster status json parent directory {}: {err}",
                        parent.display()
                    )
                })?;
            }
        }
        *ready = true;
        Ok(())
    };
    ensure_status_parent_dir(status_parent_ready)?;

    let payload = LasmClusterStatusPayload {
        mode: "lasm-cluster",
        updated_at_ms: lasm_cluster_status_now_ms(),
        listen_port: snapshot.listen_port,
        min_instances: snapshot.min_instances,
        max_instances: snapshot.max_instances,
        worker_count: snapshot.worker_count,
        relay_worker_count: snapshot.relay_worker_count,
        relay_queue_capacity: snapshot.relay_queue_capacity,
        relay_queue_shard_capacity: snapshot.relay_queue_shard_capacity,
        relay_buffer_bytes: snapshot.relay_buffer_bytes,
        relay_buffer_pool_max: snapshot.relay_buffer_pool_max,
        relay_buffer_pool_prewarm: snapshot.relay_buffer_pool_prewarm,
        worker_ports: snapshot.worker_ports.as_slice(),
        active_connections: snapshot.active_connections,
        active_connections_per_worker: snapshot.active_connections_per_worker,
        relay_saturation_events_pending: snapshot.relay_saturation_events_pending,
        relay_saturation_events_total: snapshot.relay_saturation_events_total,
        relay_saturation_events_per_sec: snapshot.relay_saturation_events_per_sec,
        relay_accept_batch_max: snapshot.relay_accept_batch_max,
        relay_pump_batch_max: snapshot.relay_pump_batch_max,
        relay_selection_reservation_min_chunk: snapshot.relay_selection_reservation_min_chunk,
        relay_idle_spin_threshold: snapshot.relay_idle_spin_threshold,
        relay_idle_sleep_micros: snapshot.relay_idle_sleep_micros,
        relay_accept_workers: snapshot.relay_accept_workers,
        relay_backend_connect_timeout_ms: snapshot.relay_backend_connect_timeout_ms,
        relay_backend_connect_cooldown_ms: snapshot.relay_backend_connect_cooldown_ms,
        db_adapter: snapshot.db_adapter.as_deref(),
        db_postgres_tls_mode: snapshot.db_postgres_tls_mode.as_deref(),
        db_max_tx_handles: snapshot.db_max_tx_handles,
        db_records_max: snapshot.db_records_max,
        db_postgres_statement_cache_max: snapshot.db_postgres_statement_cache_max,
        db_postgres_placeholder_cache_max: snapshot.db_postgres_placeholder_cache_max,
        db_postgres_statement_timeout_ms: snapshot.db_postgres_statement_timeout_ms,
        db_postgres_lock_timeout_ms: snapshot.db_postgres_lock_timeout_ms,
        db_postgres_connect_timeout_ms: snapshot.db_postgres_connect_timeout_ms,
        db_sqlite_busy_timeout_ms: snapshot.db_sqlite_busy_timeout_ms,
        db_sqlite_journal_mode: snapshot.db_sqlite_journal_mode.as_deref(),
        db_sqlite_synchronous: snapshot.db_sqlite_synchronous.as_deref(),
        db_postgres_retryable_conflict_retry_max: snapshot.db_postgres_retryable_conflict_retry_max,
        db_sqlite_lock_retry_max: snapshot.db_sqlite_lock_retry_max,
        db_sqlite_lock_retry_delay_ms: snapshot.db_sqlite_lock_retry_delay_ms,
        relay_dispatch_fallback_total: snapshot.relay_dispatch_fallback_total,
        relay_dispatch_fallback_per_sec: snapshot.relay_dispatch_fallback_per_sec,
        relay_dispatch_saturation_short_circuit_total: snapshot
            .relay_dispatch_saturation_short_circuit_total,
        relay_dispatch_saturation_short_circuit_per_sec: snapshot
            .relay_dispatch_saturation_short_circuit_per_sec,
        relay_live_sender_count: snapshot.relay_live_sender_count,
        relay_queue_depth: snapshot.relay_queue_depth,
        relay_queue_max_depth: snapshot.relay_queue_max_depth,
        relay_pump_connections: snapshot.relay_pump_connections,
        relay_buffer_pool_entries: snapshot.relay_buffer_pool_entries,
        reusable_ports_count: snapshot.reusable_ports_count,
        autoscale_desired_instances: snapshot.autoscale_desired_instances,
        autoscale_last_saturation_events: snapshot.autoscale_last_saturation_events,
        autoscale_last_dynamic_boost_step: snapshot.autoscale_last_dynamic_boost_step,
        autoscale_scale_up_cooldown_remaining_ms: snapshot.autoscale_scale_up_cooldown_remaining_ms,
        autoscale_scale_down_cooldown_remaining_ms: snapshot
            .autoscale_scale_down_cooldown_remaining_ms,
    };
    let tmp_file = match fs::File::create(tmp_path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            *status_parent_ready = false;
            ensure_status_parent_dir(status_parent_ready)?;
            fs::File::create(tmp_path).map_err(|retry_err| {
                format!(
                    "could not create cluster status json temporary file {}: {retry_err}",
                    tmp_path.display()
                )
            })?
        }
        Err(err) => {
            return Err(format!(
                "could not create cluster status json temporary file {}: {err}",
                tmp_path.display()
            ));
        }
    };
    let mut tmp_writer = BufWriter::new(tmp_file);
    serde_json::to_writer(&mut tmp_writer, &payload)
        .map_err(|err| format!("could not encode cluster status json payload: {err}"))?;
    tmp_writer.flush().map_err(|err| {
        format!(
            "could not flush cluster status json temporary file {}: {err}",
            tmp_path.display()
        )
    })?;
    fs::rename(tmp_path, path).map_err(|err| {
        format!(
            "could not move cluster status json temporary file {} to {}: {err}",
            tmp_path.display(),
            path.display()
        )
    })?;
    *last_snapshot = Some(snapshot);
    Ok(())
}
