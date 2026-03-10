use arc_swap::ArcSwap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::lasm_cluster_relay_pump::{
    default_lasm_cluster_relay_buffer_bytes, default_lasm_cluster_relay_idle_backoff_max,
    default_lasm_cluster_relay_io_burst_max,
};
use crate::{
    LasmClusterConfig, LasmClusterState, LASM_CLUSTER_IDLE_SLEEP_MICROS,
    LASM_CLUSTER_IDLE_SPIN_THRESHOLD, LASM_CLUSTER_RELAY_PUMP_BATCH_MAX,
    LASM_CLUSTER_RELAY_PUMP_BATCH_MIN, LASM_CLUSTER_RELAY_PUMP_BATCH_MULTIPLIER,
    LASM_CLUSTER_SELECTION_RESERVATION_MIN_CHUNK,
};

pub(crate) fn lasm_cluster_maintenance_interval_ms(config: &LasmClusterConfig) -> u64 {
    config.autoscale_check_ms.clamp(100, 500)
}

pub(crate) fn lasm_cluster_backend_connect_timeout(config: &LasmClusterConfig) -> Duration {
    Duration::from_millis(config.cluster_backend_connect_timeout_ms.max(25))
}

pub(crate) fn lasm_cluster_backend_connect_cooldown(config: &LasmClusterConfig) -> Duration {
    Duration::from_millis(config.cluster_backend_connect_cooldown_ms.max(25))
}

fn resolve_lasm_cluster_env_u64(name: &str, default_value: u64, min: u64, max: u64) -> u64 {
    let Ok(raw) = std::env::var(name) else {
        return default_value;
    };
    let value = raw.trim();
    if value.is_empty() {
        return default_value;
    }
    value
        .parse::<u64>()
        .ok()
        .map(|parsed| parsed.clamp(min, max))
        .unwrap_or(default_value)
}

pub(crate) fn resolve_lasm_cluster_idle_spin_threshold() -> u32 {
    let value = resolve_lasm_cluster_env_u64(
        "SEC4_RT_LASM_CLUSTER_IDLE_SPIN_THRESHOLD",
        u64::from(LASM_CLUSTER_IDLE_SPIN_THRESHOLD),
        1,
        4096,
    );
    u32::try_from(value).unwrap_or(LASM_CLUSTER_IDLE_SPIN_THRESHOLD)
}

pub(crate) fn resolve_lasm_cluster_idle_sleep_micros() -> u64 {
    resolve_lasm_cluster_env_u64(
        "SEC4_RT_LASM_CLUSTER_IDLE_SLEEP_MICROS",
        LASM_CLUSTER_IDLE_SLEEP_MICROS,
        1,
        50_000,
    )
}

pub(crate) fn resolve_lasm_cluster_relay_buffer_bytes() -> usize {
    let default_value = default_lasm_cluster_relay_buffer_bytes();
    let value = resolve_lasm_cluster_env_u64(
        "SEC4_RT_LASM_CLUSTER_RELAY_BUFFER_BYTES",
        default_value as u64,
        1024,
        1024 * 1024,
    );
    usize::try_from(value).unwrap_or(default_value)
}

pub(crate) fn resolve_lasm_cluster_relay_buffer_pool_max(relay_accept_batch_max: usize) -> usize {
    let default_value = relay_accept_batch_max
        .saturating_mul(4)
        .max(64)
        .clamp(16, 65_536);
    let value = resolve_lasm_cluster_env_u64(
        "SEC4_RT_LASM_CLUSTER_RELAY_BUFFER_POOL_MAX",
        default_value as u64,
        16,
        65_536,
    );
    usize::try_from(value).unwrap_or(default_value)
}

pub(crate) fn resolve_lasm_cluster_relay_buffer_pool_prewarm(
    relay_buffer_pool_max: usize,
) -> usize {
    let default_value = 0usize;
    let value = resolve_lasm_cluster_env_u64(
        "SEC4_RT_LASM_CLUSTER_RELAY_BUFFER_POOL_PREWARM",
        default_value as u64,
        0,
        relay_buffer_pool_max.min(65_536) as u64,
    );
    usize::try_from(value)
        .unwrap_or(default_value)
        .min(relay_buffer_pool_max)
}

pub(crate) fn resolve_lasm_cluster_relay_io_burst_max() -> usize {
    let default_value = default_lasm_cluster_relay_io_burst_max();
    let value = resolve_lasm_cluster_env_u64(
        "SEC4_RT_LASM_CLUSTER_RELAY_IO_BURST_MAX",
        default_value as u64,
        1,
        64,
    );
    usize::try_from(value).unwrap_or(default_value)
}

pub(crate) fn resolve_lasm_cluster_relay_idle_backoff_max() -> usize {
    let default_value = default_lasm_cluster_relay_idle_backoff_max();
    let value = resolve_lasm_cluster_env_u64(
        "SEC4_RT_LASM_CLUSTER_RELAY_IDLE_BACKOFF_MAX",
        default_value as u64,
        0,
        32,
    );
    usize::try_from(value).unwrap_or(default_value)
}

pub(crate) fn resolve_lasm_cluster_relay_pump_scan_multiplier() -> usize {
    let default_value = 4usize;
    let value = resolve_lasm_cluster_env_u64(
        "SEC4_RT_LASM_CLUSTER_RELAY_PUMP_SCAN_MULTIPLIER",
        default_value as u64,
        1,
        16,
    );
    usize::try_from(value).unwrap_or(default_value)
}

pub(crate) fn resolve_lasm_cluster_fallback_connect_max_attempts() -> usize {
    let default_value = 4usize;
    let value = resolve_lasm_cluster_env_u64(
        "SEC4_RT_LASM_CLUSTER_FALLBACK_CONNECT_MAX_ATTEMPTS",
        default_value as u64,
        1,
        256,
    );
    usize::try_from(value).unwrap_or(default_value)
}

pub(crate) fn refresh_lasm_cluster_worker_ports_snapshot_if_changed(
    state: &LasmClusterState,
    snapshot: &Arc<ArcSwap<Vec<u16>>>,
    last_published_ports: &mut Vec<u16>,
) -> bool {
    if state.workers.len() == last_published_ports.len()
        && state
            .workers
            .iter()
            .zip(last_published_ports.iter())
            .all(|(worker, port)| worker.port == *port)
    {
        return false;
    }

    last_published_ports.clear();
    last_published_ports.extend(state.workers.iter().map(|worker| worker.port));
    if last_published_ports
        .windows(2)
        .any(|window| window[0] > window[1])
    {
        last_published_ports.sort_unstable();
    }
    snapshot.store(Arc::new(last_published_ports.clone()));
    true
}

pub(crate) fn lasm_cluster_remaining_cooldown_ms(
    now: Instant,
    last_at: Option<Instant>,
    cooldown_ms: u64,
) -> u64 {
    let Some(last_at) = last_at else {
        return 0;
    };
    let elapsed_ms = now
        .duration_since(last_at)
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;
    cooldown_ms.saturating_sub(elapsed_ms)
}

pub(crate) fn desired_lasm_cluster_instances(
    active_connections: usize,
    min_instances: usize,
    max_instances: usize,
    target_connections_per_instance: usize,
) -> usize {
    let target_connections_per_instance = target_connections_per_instance.max(1);
    let needed = if active_connections == 0 {
        min_instances
    } else {
        ((active_connections - 1) / target_connections_per_instance) + 1
    };
    needed.clamp(min_instances, max_instances)
}

pub(crate) fn lasm_cluster_proxy_worker_count(config: &LasmClusterConfig) -> usize {
    if let Some(value) = config.cluster_relay_workers {
        return value;
    }
    let host_parallelism = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(4)
        .max(1);
    let instance_hint = config.max_instances.max(config.min_instances).max(1);
    let mut relay_hint = if instance_hint <= 1 {
        1
    } else {
        // Keep auto relay sizing conservative in multi-instance mode:
        // short capacity probes show lower relay-thread counts reduce contention
        // versus ceil(sqrt(...)) defaults under current cluster topology.
        //
        // For 4+ instance proxy topology we keep a floor of 3 relay workers to
        // reduce transient relay saturation on mixed write-heavy workloads.
        let base_hint = ((instance_hint as f64).sqrt() as usize).max(2);
        if instance_hint >= 4 {
            base_hint.max(3)
        } else {
            base_hint
        }
    };
    relay_hint = relay_hint.min(host_parallelism);
    relay_hint.clamp(1, 16)
}

pub(crate) fn lasm_cluster_proxy_queue_capacity(
    config: &LasmClusterConfig,
    worker_count: usize,
) -> usize {
    if let Some(value) = config.cluster_relay_queue {
        return value;
    }
    config
        .target_connections_per_instance
        .max(1)
        .saturating_mul(config.max_instances.max(1))
        .max(worker_count.saturating_mul(2))
        .min(65_536)
}

pub(crate) fn lasm_cluster_accept_worker_count(
    config: &LasmClusterConfig,
    relay_worker_count: usize,
) -> usize {
    if let Some(value) = config.cluster_accept_workers {
        return value;
    }
    let default_value = relay_worker_count.max(1).min(4);
    let Ok(raw) = std::env::var("SEC4_RT_LASM_CLUSTER_ACCEPT_WORKERS") else {
        return default_value;
    };
    let value = raw.trim();
    if value.is_empty() {
        return default_value;
    }
    value
        .parse::<usize>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .map(|parsed| parsed.clamp(1, relay_worker_count.max(1).min(16)))
        .unwrap_or(default_value)
}

pub(crate) fn lasm_cluster_relay_accept_batch_max(config: &LasmClusterConfig) -> usize {
    config.cluster_relay_accept_batch_max.max(1)
}

pub(crate) fn lasm_cluster_relay_pump_batch_max(config: &LasmClusterConfig) -> usize {
    config.cluster_relay_pump_batch_max.max(1)
}

pub(crate) fn lasm_cluster_selection_reservation_min_chunk(config: &LasmClusterConfig) -> usize {
    config.cluster_selection_reservation_min_chunk.max(1)
}

pub(crate) fn resolve_lasm_cluster_relay_accept_batch_max(
    explicit_override: Option<usize>,
) -> usize {
    let default_value = 64_usize;
    if let Some(value) = explicit_override {
        return value.clamp(1, 4_096);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_CLUSTER_RELAY_ACCEPT_BATCH_MAX") else {
        return default_value;
    };
    let value = raw.trim();
    if value.is_empty() {
        return default_value;
    }
    value
        .parse::<usize>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .map(|parsed| parsed.clamp(1, 4_096))
        .unwrap_or(default_value)
}

pub(crate) fn resolve_lasm_cluster_relay_pump_batch_max(
    explicit_override: Option<usize>,
    relay_accept_batch_max: usize,
) -> usize {
    let default_value = relay_accept_batch_max
        .saturating_mul(LASM_CLUSTER_RELAY_PUMP_BATCH_MULTIPLIER)
        .clamp(
            LASM_CLUSTER_RELAY_PUMP_BATCH_MIN,
            LASM_CLUSTER_RELAY_PUMP_BATCH_MAX,
        );
    if let Some(value) = explicit_override {
        return value.clamp(1, LASM_CLUSTER_RELAY_PUMP_BATCH_MAX);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_CLUSTER_RELAY_PUMP_BATCH_MAX") else {
        return default_value;
    };
    let value = raw.trim();
    if value.is_empty() {
        return default_value;
    }
    value
        .parse::<usize>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .map(|parsed| parsed.clamp(1, LASM_CLUSTER_RELAY_PUMP_BATCH_MAX))
        .unwrap_or(default_value)
}

pub(crate) fn resolve_lasm_cluster_selection_reservation_min_chunk(
    relay_accept_batch_max: usize,
) -> usize {
    let default_value = LASM_CLUSTER_SELECTION_RESERVATION_MIN_CHUNK.max(1);
    let Ok(raw) = std::env::var("SEC4_RT_LASM_CLUSTER_SELECTION_RESERVATION_MIN_CHUNK") else {
        return default_value;
    };
    let value = raw.trim();
    if value.is_empty() {
        return default_value;
    }
    value
        .parse::<usize>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .map(|parsed| parsed.clamp(1, 65_536))
        .unwrap_or(default_value)
        .max(relay_accept_batch_max.max(1))
}
