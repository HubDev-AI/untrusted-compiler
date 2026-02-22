use std::io::Write;
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crate::lasm_cluster_fallback_dispatch::LasmClusterRelayDispatchError;

pub(crate) const LASM_CLUSTER_SATURATION_COUNTER_FLUSH_BATCH: usize = 8;

const LASM_CLUSTER_UNAVAILABLE_NO_HEALTHY_WORKERS_RESPONSE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: 54\r\nConnection: close\r\n\r\n{\"ok\":false,\"status\":503,\"error\":\"no healthy workers\"}";
const LASM_CLUSTER_UNAVAILABLE_WORKER_UNAVAILABLE_RESPONSE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: 54\r\nConnection: close\r\n\r\n{\"ok\":false,\"status\":503,\"error\":\"worker unavailable\"}";
const LASM_CLUSTER_UNAVAILABLE_RELAY_SATURATED_RESPONSE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: 59\r\nConnection: close\r\n\r\n{\"ok\":false,\"status\":503,\"error\":\"cluster relay saturated\"}";
const LASM_CLUSTER_UNAVAILABLE_RELAY_UNAVAILABLE_RESPONSE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: 61\r\nConnection: close\r\n\r\n{\"ok\":false,\"status\":503,\"error\":\"cluster relay unavailable\"}";

pub(crate) enum LasmClusterUnavailableReason {
    NoHealthyWorkers,
    WorkerUnavailable,
}

#[inline(always)]
fn lasm_cluster_unavailable_response(reason: LasmClusterUnavailableReason) -> &'static [u8] {
    match reason {
        LasmClusterUnavailableReason::NoHealthyWorkers => {
            LASM_CLUSTER_UNAVAILABLE_NO_HEALTHY_WORKERS_RESPONSE
        }
        LasmClusterUnavailableReason::WorkerUnavailable => {
            LASM_CLUSTER_UNAVAILABLE_WORKER_UNAVAILABLE_RESPONSE
        }
    }
}

pub(crate) fn write_lasm_cluster_unavailable_response(
    client: &mut TcpStream,
    reason: LasmClusterUnavailableReason,
) -> Result<(), String> {
    let response = lasm_cluster_unavailable_response(reason);
    client
        .write_all(response)
        .map_err(|err| format!("could not write LASM cluster overload response: {err}"))
}

#[inline(always)]
pub(crate) fn flush_lasm_cluster_saturation_counters(
    pending_counter: &AtomicUsize,
    total_counter: &AtomicU64,
    pending_local: &mut usize,
    total_local: &mut u64,
) {
    if *pending_local > 0 {
        pending_counter.fetch_add(*pending_local, Ordering::Relaxed);
        *pending_local = 0;
    }
    if *total_local > 0 {
        total_counter.fetch_add(*total_local, Ordering::Relaxed);
        *total_local = 0;
    }
}

#[inline(always)]
pub(crate) fn flush_lasm_cluster_dispatch_fallback_total(
    counter: &AtomicU64,
    total_local: &mut u64,
) {
    if *total_local > 0 {
        counter.fetch_add(*total_local, Ordering::Relaxed);
        *total_local = 0;
    }
}

#[inline(always)]
pub(crate) fn flush_lasm_cluster_dispatch_short_circuit_total(
    counter: &AtomicU64,
    total_local: &mut u64,
) {
    if *total_local > 0 {
        counter.fetch_add(*total_local, Ordering::Relaxed);
        *total_local = 0;
    }
}

#[inline(always)]
pub(crate) fn flush_lasm_cluster_active_connection_increments(
    active_counter: &AtomicUsize,
    increments_local: &mut usize,
) {
    if *increments_local > 0 {
        active_counter.fetch_add(*increments_local, Ordering::Relaxed);
        *increments_local = 0;
    }
}

#[inline(always)]
pub(crate) fn flush_lasm_cluster_active_connection_decrements(
    active_counter: &AtomicUsize,
    decrements_local: &mut usize,
) {
    if *decrements_local > 0 {
        active_counter.fetch_sub(*decrements_local, Ordering::Relaxed);
        *decrements_local = 0;
    }
}

#[inline(always)]
pub(crate) fn handle_lasm_cluster_accept_dispatch_error(
    dispatch_error: LasmClusterRelayDispatchError,
    active_connections: &AtomicUsize,
    relay_saturation_events: &AtomicUsize,
    relay_saturation_events_total: &AtomicU64,
    relay_dispatch_fallback_total: &AtomicU64,
    relay_dispatch_short_circuit_total: &AtomicU64,
    listener_enqueued_local: &mut usize,
    listener_saturation_pending_local: &mut usize,
    listener_saturation_total_local: &mut u64,
    listener_dispatch_fallback_total_local: &mut u64,
    listener_dispatch_short_circuit_total_local: &mut u64,
) -> Result<(), String> {
    match dispatch_error {
        LasmClusterRelayDispatchError::Saturated(mut stream) => {
            *listener_saturation_pending_local += 1;
            *listener_saturation_total_local += 1;
            let _ = stream.write_all(LASM_CLUSTER_UNAVAILABLE_RELAY_SATURATED_RESPONSE);
            if *listener_saturation_pending_local >= LASM_CLUSTER_SATURATION_COUNTER_FLUSH_BATCH {
                flush_lasm_cluster_saturation_counters(
                    relay_saturation_events,
                    relay_saturation_events_total,
                    listener_saturation_pending_local,
                    listener_saturation_total_local,
                );
            }
            Ok(())
        }
        LasmClusterRelayDispatchError::Unavailable(mut stream) => {
            let _ = stream.write_all(LASM_CLUSTER_UNAVAILABLE_RELAY_UNAVAILABLE_RESPONSE);
            flush_lasm_cluster_active_connection_increments(
                active_connections,
                listener_enqueued_local,
            );
            flush_lasm_cluster_saturation_counters(
                relay_saturation_events,
                relay_saturation_events_total,
                listener_saturation_pending_local,
                listener_saturation_total_local,
            );
            flush_lasm_cluster_dispatch_fallback_total(
                relay_dispatch_fallback_total,
                listener_dispatch_fallback_total_local,
            );
            flush_lasm_cluster_dispatch_short_circuit_total(
                relay_dispatch_short_circuit_total,
                listener_dispatch_short_circuit_total_local,
            );
            Err("LASM cluster relay worker pool disconnected unexpectedly".to_string())
        }
    }
}
