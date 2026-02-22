use arc_swap::ArcSwap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::lasm_cluster_runtime_config::{
    resolve_lasm_cluster_idle_sleep_micros, resolve_lasm_cluster_idle_spin_threshold,
};
use crate::lasm_cluster_status_json::{write_lasm_cluster_status_json, LasmClusterStatusSnapshot};
use crate::LasmClusterConfig;

pub(crate) struct LasmClusterStatusWriterConfig {
    pub(crate) status_path: Option<PathBuf>,
    pub(crate) stop_flag: Arc<AtomicBool>,
    pub(crate) shared_config: Arc<LasmClusterConfig>,
    pub(crate) active_connections: Arc<AtomicUsize>,
    pub(crate) relay_saturation_events: Arc<AtomicUsize>,
    pub(crate) relay_saturation_events_total: Arc<AtomicU64>,
    pub(crate) worker_ports_snapshot: Arc<ArcSwap<Vec<u16>>>,
    pub(crate) relay_worker_count: usize,
    pub(crate) relay_queue_capacity: usize,
    pub(crate) relay_queue_shard_capacity: usize,
    pub(crate) relay_accept_worker_count: usize,
    pub(crate) relay_dispatch_fallback_total: Arc<AtomicU64>,
    pub(crate) relay_dispatch_saturation_short_circuit_total: Arc<AtomicU64>,
    pub(crate) relay_live_sender_count: Arc<AtomicUsize>,
    pub(crate) autoscale_last_desired_instances: Arc<AtomicUsize>,
    pub(crate) autoscale_last_saturation_events: Arc<AtomicUsize>,
    pub(crate) autoscale_last_dynamic_boost_step: Arc<AtomicUsize>,
    pub(crate) autoscale_scale_up_cooldown_remaining_ms: Arc<AtomicU64>,
    pub(crate) autoscale_scale_down_cooldown_remaining_ms: Arc<AtomicU64>,
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
        worker_ports_snapshot,
        relay_worker_count,
        relay_queue_capacity,
        relay_queue_shard_capacity,
        relay_accept_worker_count,
        relay_dispatch_fallback_total,
        relay_dispatch_saturation_short_circuit_total,
        relay_live_sender_count,
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
            let worker_ports = worker_ports_snapshot.load_full();
            let worker_count = worker_ports.len();
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
                worker_ports,
                active_connections,
                active_connections_per_worker,
                relay_saturation_events_pending: relay_saturation_events.load(Ordering::Relaxed),
                relay_saturation_events_total: saturation_total,
                relay_saturation_events_per_sec: saturation_per_sec,
                relay_accept_batch_max: shared_config.cluster_relay_accept_batch_max,
                relay_pump_batch_max: shared_config.cluster_relay_pump_batch_max,
                relay_selection_reservation_min_chunk: shared_config
                    .cluster_selection_reservation_min_chunk,
                relay_idle_spin_threshold,
                relay_idle_sleep_micros,
                relay_accept_workers: relay_accept_worker_count,
                relay_backend_connect_timeout_ms: shared_config.cluster_backend_connect_timeout_ms,
                relay_backend_connect_cooldown_ms: shared_config
                    .cluster_backend_connect_cooldown_ms,
                relay_dispatch_fallback_total: dispatch_fallback_total,
                relay_dispatch_fallback_per_sec: dispatch_fallback_per_sec,
                relay_dispatch_saturation_short_circuit_total:
                    dispatch_saturation_short_circuit_total,
                relay_dispatch_saturation_short_circuit_per_sec:
                    dispatch_saturation_short_circuit_per_sec,
                relay_live_sender_count: relay_live_sender_count.load(Ordering::Relaxed),
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
