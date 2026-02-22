use crossbeam_channel::Sender;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use crate::lasm_cluster_accept_loop::run_lasm_cluster_accept_loop;

pub(crate) struct LasmClusterAcceptWorkersConfig<'a> {
    pub(crate) listener: &'a TcpListener,
    pub(crate) relay_accept_worker_count: usize,
    pub(crate) relay_senders: &'a Arc<Vec<Sender<TcpStream>>>,
    pub(crate) active_connections: &'a Arc<AtomicUsize>,
    pub(crate) relay_saturation_events: &'a Arc<AtomicUsize>,
    pub(crate) relay_saturation_events_total: &'a Arc<AtomicU64>,
    pub(crate) relay_dispatch_fallback_total: &'a Arc<AtomicU64>,
    pub(crate) relay_dispatch_saturation_short_circuit_total: &'a Arc<AtomicU64>,
    pub(crate) relay_live_sender_count: &'a Arc<AtomicUsize>,
    pub(crate) stop_flag: &'a Arc<AtomicBool>,
    pub(crate) relay_accept_batch_max: usize,
}

pub(crate) fn run_lasm_cluster_accept_workers(
    config: LasmClusterAcceptWorkersConfig<'_>,
) -> Result<(), String> {
    let LasmClusterAcceptWorkersConfig {
        listener,
        relay_accept_worker_count,
        relay_senders,
        active_connections,
        relay_saturation_events,
        relay_saturation_events_total,
        relay_dispatch_fallback_total,
        relay_dispatch_saturation_short_circuit_total,
        relay_live_sender_count,
        stop_flag,
        relay_accept_batch_max,
    } = config;

    let accept_error_reported = Arc::new(AtomicBool::new(false));
    let mut accept_handles: Vec<std::thread::JoinHandle<()>> =
        Vec::with_capacity(relay_accept_worker_count.saturating_sub(1));
    for accept_worker_index in 1..relay_accept_worker_count {
        let accept_listener = match listener.try_clone() {
            Ok(listener) => listener,
            Err(err) => {
                stop_flag.store(true, Ordering::Relaxed);
                for handle in accept_handles {
                    let _ = handle.join();
                }
                return Err(format!("could not clone LASM cluster listener: {err}"));
            }
        };
        let accept_relay_senders = Arc::clone(relay_senders);
        let accept_active_connections = Arc::clone(active_connections);
        let accept_saturation_events = Arc::clone(relay_saturation_events);
        let accept_saturation_events_total = Arc::clone(relay_saturation_events_total);
        let accept_dispatch_fallback_total = Arc::clone(relay_dispatch_fallback_total);
        let accept_dispatch_short_circuit_total =
            Arc::clone(relay_dispatch_saturation_short_circuit_total);
        let accept_relay_live_sender_count = Arc::clone(relay_live_sender_count);
        let accept_stop_flag = Arc::clone(stop_flag);
        let accept_error_reported = Arc::clone(&accept_error_reported);
        accept_handles.push(std::thread::spawn(move || {
            if let Err(message) = run_lasm_cluster_accept_loop(
                &accept_listener,
                accept_relay_senders.as_slice(),
                accept_active_connections.as_ref(),
                accept_saturation_events.as_ref(),
                accept_saturation_events_total.as_ref(),
                accept_relay_live_sender_count.as_ref(),
                accept_worker_index,
                accept_dispatch_fallback_total.as_ref(),
                accept_dispatch_short_circuit_total.as_ref(),
                accept_stop_flag.as_ref(),
                relay_accept_batch_max,
            ) {
                if !accept_error_reported.swap(true, Ordering::Relaxed) {
                    eprintln!("run failed: {message}");
                }
                accept_stop_flag.store(true, Ordering::Relaxed);
            }
        }));
    }

    if let Err(message) = run_lasm_cluster_accept_loop(
        listener,
        relay_senders.as_slice(),
        active_connections.as_ref(),
        relay_saturation_events.as_ref(),
        relay_saturation_events_total.as_ref(),
        relay_live_sender_count.as_ref(),
        0,
        relay_dispatch_fallback_total.as_ref(),
        relay_dispatch_saturation_short_circuit_total.as_ref(),
        stop_flag.as_ref(),
        relay_accept_batch_max,
    ) {
        if !accept_error_reported.swap(true, Ordering::Relaxed) {
            eprintln!("run failed: {message}");
        }
    }

    stop_flag.store(true, Ordering::Relaxed);
    for handle in accept_handles {
        let _ = handle.join();
    }
    Ok(())
}
