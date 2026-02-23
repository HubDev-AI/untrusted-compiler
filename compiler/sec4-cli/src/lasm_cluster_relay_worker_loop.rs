use arc_swap::ArcSwap;
use crossbeam_channel::{Receiver, TryRecvError};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::lasm_cluster_accept_dispatch::{
    flush_lasm_cluster_active_connection_decrements, flush_lasm_cluster_saturation_counters,
    write_lasm_cluster_no_healthy_workers_response, write_lasm_cluster_worker_unavailable_response,
};
use crate::lasm_cluster_backend_selection::{
    rebuild_lasm_cluster_backend_selection_lookup, rebuild_lasm_cluster_worker_backend_addrs,
    remap_lasm_cluster_relay_port_state_by_index, LASM_CLUSTER_SELECTION_LOOKUP_NONE,
};
use crate::lasm_cluster_relay_pump::{LasmClusterRelayPump, LasmClusterRelayPumpStep};
use crate::lasm_cluster_runtime_config::{
    resolve_lasm_cluster_idle_sleep_micros, resolve_lasm_cluster_idle_spin_threshold,
};
use crate::{LASM_CLUSTER_RELAY_WARNING_THROTTLE_MS, LASM_CLUSTER_UNHEALTHY_PRUNE_INTERVAL_MS};

fn initialize_lasm_cluster_relay_connection(
    client: TcpStream,
    upstream: TcpStream,
    relay_buffer_bytes: usize,
    relay_io_burst_max: usize,
    relay_idle_backoff_max: usize,
    relay_buffer_pool: &mut Vec<(Vec<u8>, Vec<u8>)>,
    relay_connections: &mut Vec<LasmClusterRelayPump>,
    pump_warning_next_allowed: &mut Option<Instant>,
    relay_warning_throttle_duration: Duration,
    active_connection_decrements_local: &mut usize,
) {
    let _ = client.set_nodelay(true);
    let _ = upstream.set_nodelay(true);
    let relay_result =
        if let Some((client_to_upstream, upstream_to_client)) = relay_buffer_pool.pop() {
            LasmClusterRelayPump::new_with_buffers(
                client,
                upstream,
                client_to_upstream,
                upstream_to_client,
                relay_buffer_bytes,
                relay_io_burst_max,
                relay_idle_backoff_max,
            )
        } else {
            LasmClusterRelayPump::new(
                client,
                upstream,
                relay_buffer_bytes,
                relay_io_burst_max,
                relay_idle_backoff_max,
            )
        };
    match relay_result {
        Ok(relay) => relay_connections.push(relay),
        Err(message) => {
            let relay_init_now = Instant::now();
            let warning_allowed = match *pump_warning_next_allowed {
                Some(next_allowed_at) => relay_init_now >= next_allowed_at,
                None => true,
            };
            if warning_allowed {
                eprintln!("warning: LASM cluster relay init failed: {message}");
                *pump_warning_next_allowed = Some(relay_init_now + relay_warning_throttle_duration);
            }
            *active_connection_decrements_local += 1;
        }
    }
}

fn release_lasm_cluster_relay_connection(
    relay_connections: &mut Vec<LasmClusterRelayPump>,
    relay_index: usize,
    relay_buffer_pool: &mut Vec<(Vec<u8>, Vec<u8>)>,
    relay_buffer_pool_max: usize,
    active_connection_decrements_local: &mut usize,
) {
    let relay = relay_connections.swap_remove(relay_index);
    if relay_buffer_pool.len() < relay_buffer_pool_max {
        relay_buffer_pool.push(relay.into_buffers());
    }
    *active_connection_decrements_local += 1;
}

fn emit_lasm_cluster_relay_pump_warning_if_allowed(
    pump_warning_next_allowed: &mut Option<Instant>,
    relay_warning_throttle_duration: Duration,
    message: &str,
) {
    let now = Instant::now();
    let warning_allowed = match *pump_warning_next_allowed {
        Some(next_allowed_at) => now >= next_allowed_at,
        None => true,
    };
    if warning_allowed {
        eprintln!("warning: LASM cluster relay pump failed: {message}");
        *pump_warning_next_allowed = Some(now + relay_warning_throttle_duration);
    }
}

fn advance_lasm_cluster_relay_pump_cursor(
    relay_pump_cursor: &mut usize,
    relay_len_before_step: usize,
) {
    *relay_pump_cursor += 1;
    if *relay_pump_cursor == relay_len_before_step {
        *relay_pump_cursor = 0;
    }
}

fn normalize_lasm_cluster_relay_pump_cursor(
    relay_pump_cursor: &mut usize,
    relay_connections_len: usize,
) -> bool {
    if relay_connections_len == 0 {
        *relay_pump_cursor = 0;
        return true;
    }
    if *relay_pump_cursor >= relay_connections_len {
        *relay_pump_cursor = 0;
    }
    false
}

struct LasmClusterRelayPumpDispatchOutcome {
    progressed: bool,
    removed: bool,
    consumed_pump_budget: bool,
}

struct LasmClusterRelayPumpModeResolution {
    full_scan_pump_mode: bool,
    initial_cursor: usize,
    pump_budget: usize,
    scan_budget: usize,
}

fn resolve_lasm_cluster_relay_pump_mode(
    relay_count: usize,
    relay_pump_batch_max: usize,
    relay_pump_cursor: usize,
) -> LasmClusterRelayPumpModeResolution {
    if relay_count == 0 {
        return LasmClusterRelayPumpModeResolution {
            full_scan_pump_mode: true,
            initial_cursor: 0,
            pump_budget: 0,
            scan_budget: 0,
        };
    }
    let full_scan_pump_mode = relay_count <= relay_pump_batch_max;
    let initial_cursor = if full_scan_pump_mode {
        0
    } else {
        let mut cursor = relay_pump_cursor;
        let _ = normalize_lasm_cluster_relay_pump_cursor(&mut cursor, relay_count);
        cursor
    };
    let pump_budget = if full_scan_pump_mode {
        relay_count
    } else {
        relay_pump_batch_max.min(relay_count)
    };
    let scan_budget = if full_scan_pump_mode {
        relay_count
    } else {
        relay_count.min(relay_pump_batch_max.saturating_mul(4).max(1))
    };
    LasmClusterRelayPumpModeResolution {
        full_scan_pump_mode,
        initial_cursor,
        pump_budget,
        scan_budget,
    }
}

fn pump_lasm_cluster_relay_connection_once(
    relay_connections: &mut Vec<LasmClusterRelayPump>,
    relay_index: usize,
    relay_buffer_pool: &mut Vec<(Vec<u8>, Vec<u8>)>,
    relay_buffer_pool_max: usize,
    pump_warning_next_allowed: &mut Option<Instant>,
    relay_warning_throttle_duration: Duration,
    active_connection_decrements_local: &mut usize,
) -> LasmClusterRelayPumpDispatchOutcome {
    match relay_connections[relay_index].pump_once() {
        Ok(LasmClusterRelayPumpStep::Progressed) => LasmClusterRelayPumpDispatchOutcome {
            progressed: true,
            removed: false,
            consumed_pump_budget: true,
        },
        Ok(LasmClusterRelayPumpStep::Idle) => LasmClusterRelayPumpDispatchOutcome {
            progressed: false,
            removed: false,
            consumed_pump_budget: true,
        },
        Ok(LasmClusterRelayPumpStep::BackoffDeferred) => LasmClusterRelayPumpDispatchOutcome {
            progressed: false,
            removed: false,
            consumed_pump_budget: false,
        },
        Ok(LasmClusterRelayPumpStep::Complete) => {
            release_lasm_cluster_relay_connection(
                relay_connections,
                relay_index,
                relay_buffer_pool,
                relay_buffer_pool_max,
                active_connection_decrements_local,
            );
            LasmClusterRelayPumpDispatchOutcome {
                progressed: true,
                removed: true,
                consumed_pump_budget: true,
            }
        }
        Err(err) => {
            emit_lasm_cluster_relay_pump_warning_if_allowed(
                pump_warning_next_allowed,
                relay_warning_throttle_duration,
                &err,
            );
            release_lasm_cluster_relay_connection(
                relay_connections,
                relay_index,
                relay_buffer_pool,
                relay_buffer_pool_max,
                active_connection_decrements_local,
            );
            LasmClusterRelayPumpDispatchOutcome {
                progressed: true,
                removed: true,
                consumed_pump_budget: true,
            }
        }
    }
}

fn pump_lasm_cluster_relay_connections(
    relay_connections: &mut Vec<LasmClusterRelayPump>,
    relay_pump_batch_max: usize,
    relay_pump_cursor: &mut usize,
    relay_buffer_pool: &mut Vec<(Vec<u8>, Vec<u8>)>,
    relay_buffer_pool_max: usize,
    pump_warning_next_allowed: &mut Option<Instant>,
    relay_warning_throttle_duration: Duration,
    active_connection_decrements_local: &mut usize,
) -> bool {
    let mut progressed = false;
    let mut relay_count = relay_connections.len();
    let pump_mode =
        resolve_lasm_cluster_relay_pump_mode(relay_count, relay_pump_batch_max, *relay_pump_cursor);
    let mut relay_scan_cursor = pump_mode.initial_cursor;
    let mut pump_budget = pump_mode.pump_budget;
    let mut scan_budget = pump_mode.scan_budget;
    while pump_budget > 0 && scan_budget > 0 {
        let relay_len_before_step = relay_count;
        let pump_outcome = pump_lasm_cluster_relay_connection_once(
            relay_connections,
            relay_scan_cursor,
            relay_buffer_pool,
            relay_buffer_pool_max,
            pump_warning_next_allowed,
            relay_warning_throttle_duration,
            active_connection_decrements_local,
        );
        progressed |= pump_outcome.progressed;
        if pump_outcome.consumed_pump_budget {
            pump_budget -= 1;
        }
        scan_budget -= 1;
        if pump_outcome.removed {
            relay_count -= 1;
            if normalize_lasm_cluster_relay_pump_cursor(&mut relay_scan_cursor, relay_count) {
                break;
            }
            continue;
        }
        advance_lasm_cluster_relay_pump_cursor(&mut relay_scan_cursor, relay_len_before_step);
    }
    if pump_mode.full_scan_pump_mode {
        *relay_pump_cursor = 0;
    } else {
        *relay_pump_cursor = relay_scan_cursor;
    }
    progressed
}

fn recompute_lasm_cluster_relay_selection_state(
    worker_port_count: usize,
    unhealthy_ports_until_by_index: &[Option<Instant>],
    unhealthy_port_count: usize,
    selection_lookup: &mut Vec<usize>,
) -> (bool, bool, usize, usize) {
    if worker_port_count == 0 {
        selection_lookup.clear();
        return (false, false, 0, LASM_CLUSTER_SELECTION_LOOKUP_NONE);
    }
    if unhealthy_port_count == 0 {
        selection_lookup.clear();
        return (
            true,
            true,
            worker_port_count,
            LASM_CLUSTER_SELECTION_LOOKUP_NONE,
        );
    }
    if worker_port_count == 1 {
        selection_lookup.clear();
        return (false, true, 0, LASM_CLUSTER_SELECTION_LOOKUP_NONE);
    }
    if unhealthy_port_count + 1 == worker_port_count {
        selection_lookup.clear();
        if let Some(single_healthy_index) = unhealthy_ports_until_by_index
            .iter()
            .take(worker_port_count)
            .position(|entry| entry.is_none())
        {
            return (true, false, 1, single_healthy_index);
        }
        return (false, false, 0, LASM_CLUSTER_SELECTION_LOOKUP_NONE);
    }

    let (has_healthy_backends, lookup_is_identity, lookup_cycle_span) =
        rebuild_lasm_cluster_backend_selection_lookup(
            worker_port_count,
            unhealthy_ports_until_by_index,
            unhealthy_port_count,
            selection_lookup,
        );
    (
        has_healthy_backends,
        lookup_is_identity,
        lookup_cycle_span,
        LASM_CLUSTER_SELECTION_LOOKUP_NONE,
    )
}

fn mark_lasm_cluster_relay_backend_connect_failure(
    backend_addr: std::net::SocketAddr,
    backend_index: usize,
    connect_error: &std::io::Error,
    now: Instant,
    relay_backend_connect_cooldown: Duration,
    unhealthy_ports_until_by_index: &mut [Option<Instant>],
    unhealthy_port_count: &mut usize,
    selection_lookup_dirty: &mut bool,
    unhealthy_prune_next_at: &mut Option<Instant>,
    unhealthy_prune_interval: Duration,
    connect_warning_next_allowed_by_index: &mut [Option<Instant>],
    relay_warning_throttle_duration: Duration,
) {
    let unhealthy_until = now + relay_backend_connect_cooldown;
    let unhealthy_entry = &mut unhealthy_ports_until_by_index[backend_index];
    let should_mark_unhealthy = match *unhealthy_entry {
        Some(existing_until) => existing_until <= now,
        None => true,
    };
    if should_mark_unhealthy {
        *unhealthy_port_count += 1;
        *selection_lookup_dirty = true;
    }
    *unhealthy_entry = Some(unhealthy_until);
    if unhealthy_prune_next_at.is_none() && *unhealthy_port_count > 0 {
        *unhealthy_prune_next_at = Some(now + unhealthy_prune_interval);
    }

    let warning_next_allowed_entry = &mut connect_warning_next_allowed_by_index[backend_index];
    let warning_allowed = match *warning_next_allowed_entry {
        Some(next_allowed_at) => now >= next_allowed_at,
        None => true,
    };
    if warning_allowed {
        eprintln!(
            "warning: LASM cluster worker {} connect failed: {}",
            backend_addr.port(),
            connect_error
        );
        *warning_next_allowed_entry = Some(now + relay_warning_throttle_duration);
    }
}

fn try_lasm_cluster_relay_fallback_connect(
    fallback_backend_addr: std::net::SocketAddr,
    fallback_backend_index: usize,
    relay_backend_connect_timeout: Duration,
    fallback_client: &mut Option<TcpStream>,
    relay_buffer_bytes: usize,
    relay_io_burst_max: usize,
    relay_idle_backoff_max: usize,
    relay_buffer_pool: &mut Vec<(Vec<u8>, Vec<u8>)>,
    relay_connections: &mut Vec<LasmClusterRelayPump>,
    pump_warning_next_allowed: &mut Option<Instant>,
    relay_warning_throttle_duration: Duration,
    active_connection_decrements_local: &mut usize,
    relay_backend_connect_cooldown: Duration,
    unhealthy_ports_until_by_index: &mut [Option<Instant>],
    unhealthy_port_count: &mut usize,
    selection_lookup_dirty: &mut bool,
    unhealthy_prune_next_at: &mut Option<Instant>,
    unhealthy_prune_interval: Duration,
    connect_warning_next_allowed_by_index: &mut [Option<Instant>],
) -> bool {
    match TcpStream::connect_timeout(&fallback_backend_addr, relay_backend_connect_timeout) {
        Ok(upstream) => {
            let client_for_fallback = fallback_client
                .take()
                .expect("relay fallback keeps client stream until fallback connect succeeds");
            initialize_lasm_cluster_relay_connection(
                client_for_fallback,
                upstream,
                relay_buffer_bytes,
                relay_io_burst_max,
                relay_idle_backoff_max,
                relay_buffer_pool,
                relay_connections,
                pump_warning_next_allowed,
                relay_warning_throttle_duration,
                active_connection_decrements_local,
            );
            true
        }
        Err(fallback_err) => {
            let fallback_now = Instant::now();
            mark_lasm_cluster_relay_backend_connect_failure(
                fallback_backend_addr,
                fallback_backend_index,
                &fallback_err,
                fallback_now,
                relay_backend_connect_cooldown,
                unhealthy_ports_until_by_index,
                unhealthy_port_count,
                selection_lookup_dirty,
                unhealthy_prune_next_at,
                unhealthy_prune_interval,
                connect_warning_next_allowed_by_index,
                relay_warning_throttle_duration,
            );
            false
        }
    }
}

fn resolve_lasm_cluster_fallback_lookup_start_cursor(
    selection_lookup: &[usize],
    selection_lookup_cycle_span: usize,
    selected_backend_index: usize,
    selected_worker_port_count: usize,
) -> usize {
    debug_assert!(selection_lookup_cycle_span > 0);
    debug_assert_eq!(selection_lookup.len(), selection_lookup_cycle_span);
    let failed_backend_next_index = selected_backend_index + 1;
    let search_index = if failed_backend_next_index == selected_worker_port_count {
        0
    } else {
        failed_backend_next_index
    };
    match selection_lookup.binary_search(&search_index) {
        Ok(index) | Err(index) => {
            if index == selection_lookup_cycle_span {
                0
            } else {
                index
            }
        }
    }
}

fn rebuild_lasm_cluster_fallback_lookup_start_cursor_by_backend(
    selection_lookup: &[usize],
    selection_lookup_cycle_span: usize,
    selected_worker_port_count: usize,
    fallback_lookup_start_cursor_by_backend: &mut Vec<usize>,
) {
    fallback_lookup_start_cursor_by_backend.clear();
    if selection_lookup_cycle_span == 0 || selected_worker_port_count == 0 {
        return;
    }
    fallback_lookup_start_cursor_by_backend.resize(selected_worker_port_count, 0);
    for selected_backend_index in 0..selected_worker_port_count {
        fallback_lookup_start_cursor_by_backend[selected_backend_index] =
            resolve_lasm_cluster_fallback_lookup_start_cursor(
                selection_lookup,
                selection_lookup_cycle_span,
                selected_backend_index,
                selected_worker_port_count,
            );
    }
}

fn load_lasm_cluster_worker_ports_snapshot_if_changed(
    relay_worker_ports: &Arc<ArcSwap<Vec<u16>>>,
    selected_worker_ports_snapshot: &Arc<Vec<u16>>,
) -> Option<Arc<Vec<u16>>> {
    let snapshot = relay_worker_ports.load();
    if Arc::ptr_eq(selected_worker_ports_snapshot, &snapshot) {
        None
    } else {
        Some(Arc::clone(&snapshot))
    }
}

fn prewarm_lasm_cluster_relay_buffer_pool(
    relay_buffer_pool: &mut Vec<(Vec<u8>, Vec<u8>)>,
    relay_buffer_pool_prewarm: usize,
    relay_buffer_bytes: usize,
) {
    if relay_buffer_pool_prewarm == 0 {
        return;
    }
    relay_buffer_pool.reserve(relay_buffer_pool_prewarm);
    for _ in 0..relay_buffer_pool_prewarm {
        relay_buffer_pool.push((
            vec![0_u8; relay_buffer_bytes],
            vec![0_u8; relay_buffer_bytes],
        ));
    }
}

fn apply_lasm_cluster_relay_worker_hot_path_count_delta(
    counter: &AtomicUsize,
    previous: usize,
    next: usize,
) {
    if next > previous {
        counter.fetch_add(next - previous, Ordering::Relaxed);
    } else if previous > next {
        counter.fetch_sub(previous - next, Ordering::Relaxed);
    }
}

pub(crate) fn spawn_lasm_cluster_relay_worker_loop(
    relay_receiver: Receiver<TcpStream>,
    relay_active: Arc<AtomicUsize>,
    relay_selection_counter: Arc<AtomicUsize>,
    relay_worker_ports: Arc<ArcSwap<Vec<u16>>>,
    relay_saturation_events: Arc<AtomicUsize>,
    relay_saturation_events_total: Arc<AtomicU64>,
    relay_backend_connect_timeout: Duration,
    relay_backend_connect_cooldown: Duration,
    relay_accept_batch_max: usize,
    relay_pump_batch_max: usize,
    relay_selection_reservation_min_chunk: usize,
    relay_buffer_bytes: usize,
    relay_io_burst_max: usize,
    relay_idle_backoff_max: usize,
    relay_buffer_pool_max: usize,
    relay_buffer_pool_prewarm: usize,
    relay_pump_connections_total: Arc<AtomicUsize>,
    relay_buffer_pool_entries_total: Arc<AtomicUsize>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut relay_connections: Vec<LasmClusterRelayPump> =
            Vec::with_capacity(relay_accept_batch_max.max(1));
        let mut relay_buffer_pool: Vec<(Vec<u8>, Vec<u8>)> =
            Vec::with_capacity(relay_buffer_pool_max);
        prewarm_lasm_cluster_relay_buffer_pool(
            &mut relay_buffer_pool,
            relay_buffer_pool_prewarm.min(relay_buffer_pool_max),
            relay_buffer_bytes,
        );
        let mut last_published_relay_connections = 0usize;
        let mut last_published_relay_buffer_pool_entries = relay_buffer_pool.len();
        if last_published_relay_buffer_pool_entries > 0 {
            relay_buffer_pool_entries_total
                .fetch_add(last_published_relay_buffer_pool_entries, Ordering::Relaxed);
        }
        let mut unhealthy_ports_until_by_index: Vec<Option<Instant>> = Vec::new();
        let mut connect_warning_next_allowed_by_index: Vec<Option<Instant>> = Vec::new();
        let mut unhealthy_port_count = 0_usize;
        let mut pump_warning_next_allowed: Option<Instant> = None;
        let mut receiver_closed = false;
        let mut idle_spins = 0_u32;
        let relay_idle_spin_threshold = resolve_lasm_cluster_idle_spin_threshold();
        let relay_idle_sleep_duration =
            Duration::from_micros(resolve_lasm_cluster_idle_sleep_micros());
        let mut saturation_events_pending_local = 0_usize;
        let mut saturation_events_total_local = 0_u64;
        let mut active_connection_decrements_local = 0_usize;
        let mut relay_selection_reservation_len = 0_usize;
        let mut relay_selection_reservation_offset = 0_usize;
        let mut relay_selection_reservation_next_index = 0_usize;
        let mut relay_selection_reservation_span = 0_usize;
        let relay_selection_reservation_chunk =
            relay_accept_batch_max.max(relay_selection_reservation_min_chunk);
        let mut relay_pump_cursor = 0_usize;
        let mut unhealthy_prune_next_at: Option<Instant> = None;
        let relay_warning_throttle_duration =
            Duration::from_millis(LASM_CLUSTER_RELAY_WARNING_THROTTLE_MS);
        let unhealthy_prune_interval =
            Duration::from_millis(LASM_CLUSTER_UNHEALTHY_PRUNE_INTERVAL_MS);
        let mut selected_worker_ports_snapshot = Arc::clone(&relay_worker_ports.load());
        let mut selected_worker_port_count = selected_worker_ports_snapshot.len();
        let mut selected_worker_backend_addrs: Vec<std::net::SocketAddr> =
            Vec::with_capacity(selected_worker_port_count);
        rebuild_lasm_cluster_worker_backend_addrs(
            selected_worker_ports_snapshot.as_ref(),
            &mut selected_worker_backend_addrs,
        );
        unhealthy_ports_until_by_index.resize(selected_worker_port_count, None);
        connect_warning_next_allowed_by_index.resize(selected_worker_port_count, None);
        let mut selection_lookup: Vec<usize> = Vec::new();
        let mut selection_has_healthy_backends = false;
        let mut selection_lookup_is_identity = false;
        let mut selection_lookup_cycle_span = 0_usize;
        let mut selection_single_healthy_index = LASM_CLUSTER_SELECTION_LOOKUP_NONE;
        let mut fallback_lookup_start_cursor_by_backend: Vec<usize> = Vec::new();
        let mut selection_lookup_dirty = true;

        loop {
            let mut accepted = false;
            let mut accepted_in_batch = 0_usize;
            let mut worker_ports_snapshot: Option<Arc<Vec<u16>>> = None;
            let mut worker_ports_snapshot_loaded = false;
            if unhealthy_port_count > 0 {
                let now = Instant::now();
                let should_prune = match unhealthy_prune_next_at {
                    Some(next_at) => now >= next_at,
                    None => true,
                };
                if should_prune {
                    if let Some(snapshot) = load_lasm_cluster_worker_ports_snapshot_if_changed(
                        &relay_worker_ports,
                        &selected_worker_ports_snapshot,
                    ) {
                        let previous_ports_snapshot =
                            std::mem::replace(&mut selected_worker_ports_snapshot, snapshot);
                        let previous_unhealthy_ports_until_by_index =
                            std::mem::take(&mut unhealthy_ports_until_by_index);
                        let previous_connect_warning_next_allowed_by_index =
                            std::mem::take(&mut connect_warning_next_allowed_by_index);
                        selected_worker_port_count = selected_worker_ports_snapshot.len();
                        rebuild_lasm_cluster_worker_backend_addrs(
                            selected_worker_ports_snapshot.as_ref(),
                            &mut selected_worker_backend_addrs,
                        );
                        unhealthy_port_count = remap_lasm_cluster_relay_port_state_by_index(
                            previous_ports_snapshot.as_ref(),
                            selected_worker_ports_snapshot.as_ref(),
                            previous_unhealthy_ports_until_by_index.as_slice(),
                            previous_connect_warning_next_allowed_by_index.as_slice(),
                            now,
                            &mut unhealthy_ports_until_by_index,
                            &mut connect_warning_next_allowed_by_index,
                        );
                        selection_lookup_dirty = true;
                    }
                    let mut unhealthy_port_count_after_prune = 0_usize;
                    for entry in &mut unhealthy_ports_until_by_index {
                        if let Some(until) = *entry {
                            if until <= now {
                                *entry = None;
                            } else {
                                unhealthy_port_count_after_prune += 1;
                            }
                        }
                    }
                    if unhealthy_port_count_after_prune != unhealthy_port_count {
                        selection_lookup_dirty = true;
                    }
                    unhealthy_port_count = unhealthy_port_count_after_prune;
                    unhealthy_prune_next_at = if unhealthy_port_count == 0 {
                        None
                    } else {
                        Some(now + unhealthy_prune_interval)
                    };
                    worker_ports_snapshot_loaded = true;
                }
            }
            loop {
                if accepted_in_batch >= relay_accept_batch_max {
                    break;
                }
                let incoming = match relay_receiver.try_recv() {
                    Ok(stream) => Some(stream),
                    Err(TryRecvError::Empty) => None,
                    Err(TryRecvError::Disconnected) => {
                        receiver_closed = true;
                        None
                    }
                };
                let Some(mut client) = incoming else {
                    break;
                };
                accepted = true;
                accepted_in_batch += 1;

                if !worker_ports_snapshot_loaded || selection_lookup_dirty {
                    if !worker_ports_snapshot_loaded {
                        if let Some(snapshot) = load_lasm_cluster_worker_ports_snapshot_if_changed(
                            &relay_worker_ports,
                            &selected_worker_ports_snapshot,
                        ) {
                            worker_ports_snapshot = Some(snapshot);
                        }
                        worker_ports_snapshot_loaded = true;
                    }
                    if let Some(worker_ports_snapshot_ref) = worker_ports_snapshot.as_ref() {
                        if !Arc::ptr_eq(&selected_worker_ports_snapshot, worker_ports_snapshot_ref)
                        {
                            let now = Instant::now();
                            let next_ports_snapshot = Arc::clone(worker_ports_snapshot_ref);
                            let previous_ports_snapshot = std::mem::replace(
                                &mut selected_worker_ports_snapshot,
                                next_ports_snapshot,
                            );
                            let previous_unhealthy_ports_until_by_index =
                                std::mem::take(&mut unhealthy_ports_until_by_index);
                            let previous_connect_warning_next_allowed_by_index =
                                std::mem::take(&mut connect_warning_next_allowed_by_index);
                            selected_worker_port_count = selected_worker_ports_snapshot.len();
                            rebuild_lasm_cluster_worker_backend_addrs(
                                selected_worker_ports_snapshot.as_ref(),
                                &mut selected_worker_backend_addrs,
                            );
                            unhealthy_port_count = remap_lasm_cluster_relay_port_state_by_index(
                                previous_ports_snapshot.as_ref(),
                                selected_worker_ports_snapshot.as_ref(),
                                previous_unhealthy_ports_until_by_index.as_slice(),
                                previous_connect_warning_next_allowed_by_index.as_slice(),
                                now,
                                &mut unhealthy_ports_until_by_index,
                                &mut connect_warning_next_allowed_by_index,
                            );
                            unhealthy_prune_next_at = if unhealthy_port_count == 0 {
                                None
                            } else {
                                Some(now + unhealthy_prune_interval)
                            };
                            selection_lookup_dirty = true;
                        }
                    }
                    let worker_port_count = selected_worker_port_count;
                    if worker_port_count <= 1 {
                        selection_lookup.clear();
                        fallback_lookup_start_cursor_by_backend.clear();
                        if worker_port_count == 1 && unhealthy_port_count == 0 {
                            selection_has_healthy_backends = true;
                            selection_lookup_is_identity = true;
                            selection_lookup_cycle_span = 1;
                            selection_single_healthy_index = 0;
                        } else {
                            selection_has_healthy_backends = false;
                            selection_lookup_is_identity = false;
                            selection_lookup_cycle_span = 0;
                            selection_single_healthy_index = LASM_CLUSTER_SELECTION_LOOKUP_NONE;
                        }
                        selection_lookup_dirty = false;
                    } else if selection_lookup_dirty
                        || (!selection_lookup_is_identity
                            && selection_lookup.len() != selection_lookup_cycle_span)
                    {
                        (
                            selection_has_healthy_backends,
                            selection_lookup_is_identity,
                            selection_lookup_cycle_span,
                            selection_single_healthy_index,
                        ) = recompute_lasm_cluster_relay_selection_state(
                            worker_port_count,
                            unhealthy_ports_until_by_index.as_slice(),
                            unhealthy_port_count,
                            &mut selection_lookup,
                        );
                        if selection_lookup_is_identity || selection_lookup_cycle_span == 0 {
                            fallback_lookup_start_cursor_by_backend.clear();
                        } else {
                            rebuild_lasm_cluster_fallback_lookup_start_cursor_by_backend(
                                selection_lookup.as_slice(),
                                selection_lookup_cycle_span,
                                worker_port_count,
                                &mut fallback_lookup_start_cursor_by_backend,
                            );
                        }
                        selection_lookup_dirty = false;
                    }
                }
                let worker_port_count = selected_worker_port_count;
                let selected_backend_index = if worker_port_count == 0 {
                    LASM_CLUSTER_SELECTION_LOOKUP_NONE
                } else if worker_port_count == 1 {
                    if unhealthy_port_count == 0 {
                        0
                    } else {
                        LASM_CLUSTER_SELECTION_LOOKUP_NONE
                    }
                } else if !selection_has_healthy_backends {
                    LASM_CLUSTER_SELECTION_LOOKUP_NONE
                } else if selection_single_healthy_index != LASM_CLUSTER_SELECTION_LOOKUP_NONE {
                    selection_single_healthy_index
                } else {
                    let selection_span = if selection_lookup_is_identity {
                        worker_port_count
                    } else {
                        selection_lookup_cycle_span
                    };
                    if selection_span == 0 {
                        LASM_CLUSTER_SELECTION_LOOKUP_NONE
                    } else {
                        if relay_selection_reservation_offset >= relay_selection_reservation_len
                            || relay_selection_reservation_span != selection_span
                        {
                            let relay_selection_reservation_base = relay_selection_counter
                                .fetch_add(relay_selection_reservation_chunk, Ordering::Relaxed);
                            relay_selection_reservation_len = relay_selection_reservation_chunk;
                            relay_selection_reservation_offset = 0;
                            relay_selection_reservation_span = selection_span;
                            relay_selection_reservation_next_index =
                                relay_selection_reservation_base % selection_span;
                        }
                        let start_index = relay_selection_reservation_next_index;
                        relay_selection_reservation_offset += 1;
                        relay_selection_reservation_next_index += 1;
                        if relay_selection_reservation_next_index == selection_span {
                            relay_selection_reservation_next_index = 0;
                        }
                        if selection_lookup_is_identity {
                            start_index
                        } else {
                            debug_assert_eq!(selection_lookup.len(), selection_span);
                            selection_lookup[start_index]
                        }
                    }
                };

                if selected_backend_index == LASM_CLUSTER_SELECTION_LOOKUP_NONE {
                    saturation_events_pending_local += 1;
                    saturation_events_total_local += 1;
                    let _ = write_lasm_cluster_no_healthy_workers_response(&mut client);
                    active_connection_decrements_local += 1;
                    continue;
                }

                let backend_addr = selected_worker_backend_addrs[selected_backend_index];
                match TcpStream::connect_timeout(&backend_addr, relay_backend_connect_timeout) {
                    Ok(upstream) => {
                        initialize_lasm_cluster_relay_connection(
                            client,
                            upstream,
                            relay_buffer_bytes,
                            relay_io_burst_max,
                            relay_idle_backoff_max,
                            &mut relay_buffer_pool,
                            &mut relay_connections,
                            &mut pump_warning_next_allowed,
                            relay_warning_throttle_duration,
                            &mut active_connection_decrements_local,
                        );
                    }
                    Err(err) => {
                        let now = Instant::now();
                        mark_lasm_cluster_relay_backend_connect_failure(
                            backend_addr,
                            selected_backend_index,
                            &err,
                            now,
                            relay_backend_connect_cooldown,
                            unhealthy_ports_until_by_index.as_mut_slice(),
                            &mut unhealthy_port_count,
                            &mut selection_lookup_dirty,
                            &mut unhealthy_prune_next_at,
                            unhealthy_prune_interval,
                            connect_warning_next_allowed_by_index.as_mut_slice(),
                            relay_warning_throttle_duration,
                        );
                        let mut fallback_connected = false;
                        let mut fallback_client = Some(client);
                        if unhealthy_port_count < selected_worker_port_count {
                            (
                                selection_has_healthy_backends,
                                selection_lookup_is_identity,
                                selection_lookup_cycle_span,
                                selection_single_healthy_index,
                            ) = recompute_lasm_cluster_relay_selection_state(
                                selected_worker_port_count,
                                unhealthy_ports_until_by_index.as_slice(),
                                unhealthy_port_count,
                                &mut selection_lookup,
                            );
                            if selection_lookup_is_identity || selection_lookup_cycle_span == 0 {
                                fallback_lookup_start_cursor_by_backend.clear();
                            } else {
                                rebuild_lasm_cluster_fallback_lookup_start_cursor_by_backend(
                                    selection_lookup.as_slice(),
                                    selection_lookup_cycle_span,
                                    selected_worker_port_count,
                                    &mut fallback_lookup_start_cursor_by_backend,
                                );
                            }
                            selection_lookup_dirty = false;

                            if selection_single_healthy_index != LASM_CLUSTER_SELECTION_LOOKUP_NONE
                            {
                                let fallback_backend_index = selection_single_healthy_index;
                                if fallback_backend_index != selected_backend_index {
                                    let fallback_backend_addr =
                                        selected_worker_backend_addrs[fallback_backend_index];
                                    fallback_connected = try_lasm_cluster_relay_fallback_connect(
                                        fallback_backend_addr,
                                        fallback_backend_index,
                                        relay_backend_connect_timeout,
                                        &mut fallback_client,
                                        relay_buffer_bytes,
                                        relay_io_burst_max,
                                        relay_idle_backoff_max,
                                        &mut relay_buffer_pool,
                                        &mut relay_connections,
                                        &mut pump_warning_next_allowed,
                                        relay_warning_throttle_duration,
                                        &mut active_connection_decrements_local,
                                        relay_backend_connect_cooldown,
                                        unhealthy_ports_until_by_index.as_mut_slice(),
                                        &mut unhealthy_port_count,
                                        &mut selection_lookup_dirty,
                                        &mut unhealthy_prune_next_at,
                                        unhealthy_prune_interval,
                                        connect_warning_next_allowed_by_index.as_mut_slice(),
                                    );
                                }
                            } else if selection_has_healthy_backends {
                                if selection_lookup_is_identity {
                                    let mut fallback_backend_index = selected_backend_index + 1;
                                    if fallback_backend_index == selected_worker_port_count {
                                        fallback_backend_index = 0;
                                    }
                                    let mut remaining_fallback_scan =
                                        selected_worker_port_count.saturating_sub(1);
                                    while remaining_fallback_scan > 0 {
                                        let fallback_backend_addr =
                                            selected_worker_backend_addrs[fallback_backend_index];
                                        fallback_connected =
                                            try_lasm_cluster_relay_fallback_connect(
                                                fallback_backend_addr,
                                                fallback_backend_index,
                                                relay_backend_connect_timeout,
                                                &mut fallback_client,
                                                relay_buffer_bytes,
                                                relay_io_burst_max,
                                                relay_idle_backoff_max,
                                                &mut relay_buffer_pool,
                                                &mut relay_connections,
                                                &mut pump_warning_next_allowed,
                                                relay_warning_throttle_duration,
                                                &mut active_connection_decrements_local,
                                                relay_backend_connect_cooldown,
                                                unhealthy_ports_until_by_index.as_mut_slice(),
                                                &mut unhealthy_port_count,
                                                &mut selection_lookup_dirty,
                                                &mut unhealthy_prune_next_at,
                                                unhealthy_prune_interval,
                                                connect_warning_next_allowed_by_index
                                                    .as_mut_slice(),
                                            );
                                        if fallback_connected
                                            || unhealthy_port_count >= selected_worker_port_count
                                        {
                                            break;
                                        }
                                        fallback_backend_index += 1;
                                        if fallback_backend_index == selected_worker_port_count {
                                            fallback_backend_index = 0;
                                        }
                                        remaining_fallback_scan -= 1;
                                    }
                                } else {
                                    let mut fallback_lookup_cursor =
                                        fallback_lookup_start_cursor_by_backend
                                            .get(selected_backend_index)
                                            .copied()
                                            .unwrap_or_else(|| {
                                                resolve_lasm_cluster_fallback_lookup_start_cursor(
                                                    selection_lookup.as_slice(),
                                                    selection_lookup_cycle_span,
                                                    selected_backend_index,
                                                    selected_worker_port_count,
                                                )
                                            });
                                    let mut remaining_fallback_scan = selection_lookup_cycle_span;
                                    while remaining_fallback_scan > 0 {
                                        let fallback_backend_index =
                                            selection_lookup[fallback_lookup_cursor];
                                        if fallback_backend_index == selected_backend_index {
                                            fallback_lookup_cursor += 1;
                                            if fallback_lookup_cursor == selection_lookup_cycle_span
                                            {
                                                fallback_lookup_cursor = 0;
                                            }
                                            remaining_fallback_scan -= 1;
                                            continue;
                                        }
                                        let fallback_backend_addr =
                                            selected_worker_backend_addrs[fallback_backend_index];
                                        fallback_connected =
                                            try_lasm_cluster_relay_fallback_connect(
                                                fallback_backend_addr,
                                                fallback_backend_index,
                                                relay_backend_connect_timeout,
                                                &mut fallback_client,
                                                relay_buffer_bytes,
                                                relay_io_burst_max,
                                                relay_idle_backoff_max,
                                                &mut relay_buffer_pool,
                                                &mut relay_connections,
                                                &mut pump_warning_next_allowed,
                                                relay_warning_throttle_duration,
                                                &mut active_connection_decrements_local,
                                                relay_backend_connect_cooldown,
                                                unhealthy_ports_until_by_index.as_mut_slice(),
                                                &mut unhealthy_port_count,
                                                &mut selection_lookup_dirty,
                                                &mut unhealthy_prune_next_at,
                                                unhealthy_prune_interval,
                                                connect_warning_next_allowed_by_index
                                                    .as_mut_slice(),
                                            );
                                        if fallback_connected
                                            || unhealthy_port_count >= selected_worker_port_count
                                        {
                                            break;
                                        }
                                        fallback_lookup_cursor += 1;
                                        if fallback_lookup_cursor == selection_lookup_cycle_span {
                                            fallback_lookup_cursor = 0;
                                        }
                                        remaining_fallback_scan -= 1;
                                    }
                                }
                            }
                        }
                        if fallback_connected {
                            continue;
                        }
                        let mut client = match fallback_client {
                            Some(client) => client,
                            None => continue,
                        };
                        saturation_events_pending_local += 1;
                        saturation_events_total_local += 1;
                        let _ = write_lasm_cluster_worker_unavailable_response(&mut client);
                        active_connection_decrements_local += 1;
                    }
                }
            }

            let progressed = pump_lasm_cluster_relay_connections(
                &mut relay_connections,
                relay_pump_batch_max,
                &mut relay_pump_cursor,
                &mut relay_buffer_pool,
                relay_buffer_pool_max,
                &mut pump_warning_next_allowed,
                relay_warning_throttle_duration,
                &mut active_connection_decrements_local,
            );

            if saturation_events_pending_local > 0 || saturation_events_total_local > 0 {
                flush_lasm_cluster_saturation_counters(
                    &relay_saturation_events,
                    &relay_saturation_events_total,
                    &mut saturation_events_pending_local,
                    &mut saturation_events_total_local,
                );
            }
            if active_connection_decrements_local > 0 {
                flush_lasm_cluster_active_connection_decrements(
                    &relay_active,
                    &mut active_connection_decrements_local,
                );
            }
            let relay_connections_len = relay_connections.len();
            if relay_connections_len != last_published_relay_connections {
                apply_lasm_cluster_relay_worker_hot_path_count_delta(
                    &relay_pump_connections_total,
                    last_published_relay_connections,
                    relay_connections_len,
                );
                last_published_relay_connections = relay_connections_len;
            }
            let relay_buffer_pool_entries = relay_buffer_pool.len();
            if relay_buffer_pool_entries != last_published_relay_buffer_pool_entries {
                apply_lasm_cluster_relay_worker_hot_path_count_delta(
                    &relay_buffer_pool_entries_total,
                    last_published_relay_buffer_pool_entries,
                    relay_buffer_pool_entries,
                );
                last_published_relay_buffer_pool_entries = relay_buffer_pool_entries;
            }

            if receiver_closed && relay_connections.is_empty() {
                break;
            }

            if accepted || progressed {
                idle_spins = 0;
                continue;
            }

            idle_spins += 1;
            if idle_spins < relay_idle_spin_threshold {
                std::thread::yield_now();
            } else {
                std::thread::sleep(relay_idle_sleep_duration);
                idle_spins = 0;
            }
        }
        if last_published_relay_connections > 0 {
            relay_pump_connections_total
                .fetch_sub(last_published_relay_connections, Ordering::Relaxed);
        }
        if last_published_relay_buffer_pool_entries > 0 {
            relay_buffer_pool_entries_total
                .fetch_sub(last_published_relay_buffer_pool_entries, Ordering::Relaxed);
        }
    })
}
