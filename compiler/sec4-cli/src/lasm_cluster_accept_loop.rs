use crossbeam_channel::Sender;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use crate::lasm_cluster_accept_dispatch::{
    flush_lasm_cluster_active_connection_increments, flush_lasm_cluster_dispatch_fallback_total,
    flush_lasm_cluster_dispatch_short_circuit_total, flush_lasm_cluster_saturation_counters,
    handle_lasm_cluster_accept_dispatch_error,
};
use crate::lasm_cluster_fallback_dispatch::{
    dispatch_lasm_cluster_relay_stream_fallback_dual_live,
    dispatch_lasm_cluster_relay_stream_fallback_multi,
    dispatch_lasm_cluster_relay_stream_fallback_single_live, LasmClusterRelayDispatchError,
};
use crate::lasm_cluster_relay_send::{
    attempt_lasm_cluster_relay_send, attempt_lasm_cluster_relay_send_single,
};
use crate::lasm_cluster_relay_topology::{
    lookup_lasm_cluster_next_live_sender_index, realign_lasm_cluster_dispatch_cursor_to_live,
    refresh_lasm_cluster_dual_live_sender_indices, refresh_lasm_cluster_live_sender_hints,
    refresh_lasm_cluster_next_live_sender_lookup, refresh_lasm_cluster_single_live_sender_index,
};
use crate::{
    LASM_CLUSTER_IDLE_SLEEP_MICROS, LASM_CLUSTER_IDLE_SPIN_THRESHOLD,
    LASM_CLUSTER_RELAY_SENDER_DEAD, LASM_CLUSTER_RELAY_SENDER_LIVE,
};

#[derive(Default)]
struct LasmClusterAcceptDispatchCounters {
    listener_saturation_pending_local: usize,
    listener_saturation_total_local: u64,
    listener_dispatch_fallback_total_local: u64,
    listener_dispatch_short_circuit_total_local: u64,
}

fn dispatch_lasm_cluster_relay_stream_fallback_with_live_hints(
    stream: TcpStream,
    relay_senders: &[Sender<TcpStream>],
    relay_sender_live: &mut [u8],
    relay_next_live_sender_lookup: &[usize],
    relay_has_next_live_sender_lookup_for_fallback: bool,
    relay_live_sender_count: &mut usize,
    relay_all_senders_live: &mut bool,
    next_dispatch_index: usize,
    saw_live_sender: bool,
    stream_dispatch_start: usize,
    relay_single_live_sender_index: &mut Option<usize>,
    relay_dual_live_sender_indices: &mut Option<(usize, usize)>,
) -> Result<(), LasmClusterRelayDispatchError> {
    if *relay_all_senders_live {
        return dispatch_lasm_cluster_relay_stream_fallback_multi(
            stream,
            relay_senders,
            relay_sender_live,
            relay_next_live_sender_lookup,
            relay_has_next_live_sender_lookup_for_fallback,
            relay_live_sender_count,
            relay_all_senders_live,
            next_dispatch_index,
            saw_live_sender,
        );
    }
    match *relay_live_sender_count {
        2 => {
            let dual_live_indices = if let Some(indices) = *relay_dual_live_sender_indices {
                Some(indices)
            } else {
                refresh_lasm_cluster_dual_live_sender_indices(
                    relay_sender_live,
                    *relay_live_sender_count,
                    relay_dual_live_sender_indices,
                );
                *relay_dual_live_sender_indices
            };
            if let Some((first_live, second_live)) = dual_live_indices {
                let alternate_live_index = if stream_dispatch_start == first_live {
                    second_live
                } else if stream_dispatch_start == second_live {
                    first_live
                } else {
                    first_live
                };
                dispatch_lasm_cluster_relay_stream_fallback_dual_live(
                    stream,
                    relay_senders,
                    relay_sender_live,
                    relay_live_sender_count,
                    relay_all_senders_live,
                    alternate_live_index,
                    saw_live_sender,
                )
            } else {
                dispatch_lasm_cluster_relay_stream_fallback_multi(
                    stream,
                    relay_senders,
                    relay_sender_live,
                    relay_next_live_sender_lookup,
                    relay_has_next_live_sender_lookup_for_fallback,
                    relay_live_sender_count,
                    relay_all_senders_live,
                    next_dispatch_index,
                    saw_live_sender,
                )
            }
        }
        1 => {
            let single_live_index = if let Some(index) = *relay_single_live_sender_index {
                Some(index)
            } else {
                refresh_lasm_cluster_single_live_sender_index(
                    relay_sender_live,
                    *relay_live_sender_count,
                    relay_single_live_sender_index,
                );
                *relay_single_live_sender_index
            };
            if let Some(single_live_index) = single_live_index {
                dispatch_lasm_cluster_relay_stream_fallback_single_live(
                    stream,
                    relay_senders,
                    relay_sender_live,
                    relay_live_sender_count,
                    relay_all_senders_live,
                    single_live_index,
                    saw_live_sender,
                )
            } else {
                dispatch_lasm_cluster_relay_stream_fallback_multi(
                    stream,
                    relay_senders,
                    relay_sender_live,
                    relay_next_live_sender_lookup,
                    relay_has_next_live_sender_lookup_for_fallback,
                    relay_live_sender_count,
                    relay_all_senders_live,
                    next_dispatch_index,
                    saw_live_sender,
                )
            }
        }
        _ => dispatch_lasm_cluster_relay_stream_fallback_multi(
            stream,
            relay_senders,
            relay_sender_live,
            relay_next_live_sender_lookup,
            relay_has_next_live_sender_lookup_for_fallback,
            relay_live_sender_count,
            relay_all_senders_live,
            next_dispatch_index,
            saw_live_sender,
        ),
    }
}

fn handle_lasm_cluster_accept_unavailable_stream(
    client_stream: TcpStream,
    active_connections: &AtomicUsize,
    relay_saturation_events: &AtomicUsize,
    relay_saturation_events_total: &AtomicU64,
    relay_dispatch_fallback_total: &AtomicU64,
    relay_dispatch_short_circuit_total: &AtomicU64,
    listener_enqueued_local: &mut usize,
    listener_dispatch_counters: &mut LasmClusterAcceptDispatchCounters,
) -> Result<(), String> {
    handle_lasm_cluster_accept_dispatch_error(
        LasmClusterRelayDispatchError::Unavailable(client_stream),
        active_connections,
        relay_saturation_events,
        relay_saturation_events_total,
        relay_dispatch_fallback_total,
        relay_dispatch_short_circuit_total,
        listener_enqueued_local,
        &mut listener_dispatch_counters.listener_saturation_pending_local,
        &mut listener_dispatch_counters.listener_saturation_total_local,
        &mut listener_dispatch_counters.listener_dispatch_fallback_total_local,
        &mut listener_dispatch_counters.listener_dispatch_short_circuit_total_local,
    )
}

fn flush_lasm_cluster_accept_dispatch_counters(
    relay_saturation_events: &AtomicUsize,
    relay_saturation_events_total: &AtomicU64,
    relay_dispatch_fallback_total: &AtomicU64,
    relay_dispatch_short_circuit_total: &AtomicU64,
    listener_dispatch_counters: &mut LasmClusterAcceptDispatchCounters,
) {
    if listener_dispatch_counters.listener_saturation_pending_local > 0
        || listener_dispatch_counters.listener_saturation_total_local > 0
    {
        flush_lasm_cluster_saturation_counters(
            relay_saturation_events,
            relay_saturation_events_total,
            &mut listener_dispatch_counters.listener_saturation_pending_local,
            &mut listener_dispatch_counters.listener_saturation_total_local,
        );
    }
    if listener_dispatch_counters.listener_dispatch_fallback_total_local > 0 {
        flush_lasm_cluster_dispatch_fallback_total(
            relay_dispatch_fallback_total,
            &mut listener_dispatch_counters.listener_dispatch_fallback_total_local,
        );
    }
    if listener_dispatch_counters.listener_dispatch_short_circuit_total_local > 0 {
        flush_lasm_cluster_dispatch_short_circuit_total(
            relay_dispatch_short_circuit_total,
            &mut listener_dispatch_counters.listener_dispatch_short_circuit_total_local,
        );
    }
}

pub(crate) fn run_lasm_cluster_accept_loop(
    listener: &TcpListener,
    relay_senders: &[Sender<TcpStream>],
    active_connections: &AtomicUsize,
    relay_saturation_events: &AtomicUsize,
    relay_saturation_events_total: &AtomicU64,
    relay_live_sender_count_observed: &AtomicUsize,
    initial_dispatch_cursor: usize,
    relay_dispatch_fallback_total: &AtomicU64,
    relay_dispatch_short_circuit_total: &AtomicU64,
    stop_flag: &AtomicBool,
    relay_accept_batch_max: usize,
) -> Result<(), String> {
    let mut listener_dispatch_counters = LasmClusterAcceptDispatchCounters::default();
    let mut listener_idle_spins = 0_u32;
    let listener_idle_sleep_duration = Duration::from_micros(LASM_CLUSTER_IDLE_SLEEP_MICROS);
    let relay_sender_count = relay_senders.len();
    if relay_sender_count == 0 {
        return Err("LASM cluster relay sender pool unavailable".to_string());
    }
    let relay_single_sender = if relay_sender_count == 1 {
        relay_senders.first()
    } else {
        None
    };
    let mut relay_dispatch_cursor = if relay_sender_count > 1 {
        initial_dispatch_cursor % relay_sender_count
    } else {
        0
    };
    let mut relay_sender_live = if relay_sender_count > 1 {
        vec![LASM_CLUSTER_RELAY_SENDER_LIVE; relay_sender_count]
    } else {
        Vec::new()
    };
    let mut relay_next_live_sender_lookup = if relay_sender_count > 2 {
        (0..relay_sender_count).collect::<Vec<usize>>()
    } else {
        Vec::new()
    };
    let relay_has_next_live_sender_lookup = !relay_next_live_sender_lookup.is_empty();
    let mut relay_live_sender_count = relay_sender_count;
    let mut relay_all_senders_live = relay_sender_count > 1;
    let mut relay_single_live_sender_index: Option<usize> = None;
    let mut relay_dual_live_sender_indices: Option<(usize, usize)> = None;

    loop {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }
        let mut listener_enqueued_local = 0_usize;
        let mut listener_accepted_in_batch = 0_usize;
        let mut listener_all_senders_saturated_in_batch = false;
        if let Some(relay_single_sender) = relay_single_sender {
            while listener_accepted_in_batch < relay_accept_batch_max {
                match listener.accept() {
                    Ok((client_stream, _)) => {
                        listener_accepted_in_batch += 1;
                        if let Err(dispatch_error) = attempt_lasm_cluster_relay_send_single(
                            client_stream,
                            relay_single_sender,
                        ) {
                            if let Err(message) = handle_lasm_cluster_accept_dispatch_error(
                                dispatch_error,
                                active_connections,
                                relay_saturation_events,
                                relay_saturation_events_total,
                                relay_dispatch_fallback_total,
                                relay_dispatch_short_circuit_total,
                                &mut listener_enqueued_local,
                                &mut listener_dispatch_counters.listener_saturation_pending_local,
                                &mut listener_dispatch_counters.listener_saturation_total_local,
                                &mut listener_dispatch_counters
                                    .listener_dispatch_fallback_total_local,
                                &mut listener_dispatch_counters
                                    .listener_dispatch_short_circuit_total_local,
                            ) {
                                return Err(message);
                            }
                        } else {
                            listener_enqueued_local += 1;
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(err) => {
                        flush_lasm_cluster_accept_dispatch_counters(
                            relay_saturation_events,
                            relay_saturation_events_total,
                            relay_dispatch_fallback_total,
                            relay_dispatch_short_circuit_total,
                            &mut listener_dispatch_counters,
                        );
                        return Err(format!("LASM cluster proxy accept error: {err}"));
                    }
                }
            }
        } else {
            while listener_accepted_in_batch < relay_accept_batch_max {
                match listener.accept() {
                    Ok((client_stream, _)) => {
                        listener_accepted_in_batch += 1;
                        if !relay_all_senders_live {
                            if relay_live_sender_count == 0 {
                                if let Err(message) = handle_lasm_cluster_accept_unavailable_stream(
                                    client_stream,
                                    active_connections,
                                    relay_saturation_events,
                                    relay_saturation_events_total,
                                    relay_dispatch_fallback_total,
                                    relay_dispatch_short_circuit_total,
                                    &mut listener_enqueued_local,
                                    &mut listener_dispatch_counters,
                                ) {
                                    return Err(message);
                                }
                                continue;
                            }
                            if relay_live_sender_count == 1 {
                                let single_live_index =
                                    if let Some(index) = relay_single_live_sender_index {
                                        index
                                    } else {
                                        refresh_lasm_cluster_single_live_sender_index(
                                            relay_sender_live.as_slice(),
                                            relay_live_sender_count,
                                            &mut relay_single_live_sender_index,
                                        );
                                        let Some(index) = relay_single_live_sender_index else {
                                            if let Err(message) =
                                                handle_lasm_cluster_accept_unavailable_stream(
                                                    client_stream,
                                                    active_connections,
                                                    relay_saturation_events,
                                                    relay_saturation_events_total,
                                                    relay_dispatch_fallback_total,
                                                    relay_dispatch_short_circuit_total,
                                                    &mut listener_enqueued_local,
                                                    &mut listener_dispatch_counters,
                                                )
                                            {
                                                return Err(message);
                                            }
                                            continue;
                                        };
                                        index
                                    };
                                relay_dispatch_cursor = single_live_index;
                            } else if relay_live_sender_count == 2 {
                                let (first_live, second_live) =
                                    if let Some(indices) = relay_dual_live_sender_indices {
                                        indices
                                    } else {
                                        refresh_lasm_cluster_dual_live_sender_indices(
                                            relay_sender_live.as_slice(),
                                            relay_live_sender_count,
                                            &mut relay_dual_live_sender_indices,
                                        );
                                        let Some(indices) = relay_dual_live_sender_indices else {
                                            if let Err(message) =
                                                handle_lasm_cluster_accept_unavailable_stream(
                                                    client_stream,
                                                    active_connections,
                                                    relay_saturation_events,
                                                    relay_saturation_events_total,
                                                    relay_dispatch_fallback_total,
                                                    relay_dispatch_short_circuit_total,
                                                    &mut listener_enqueued_local,
                                                    &mut listener_dispatch_counters,
                                                )
                                            {
                                                return Err(message);
                                            }
                                            continue;
                                        };
                                        indices
                                    };
                                if relay_dispatch_cursor != first_live
                                    && relay_dispatch_cursor != second_live
                                {
                                    relay_dispatch_cursor = first_live;
                                }
                            } else if relay_sender_live[relay_dispatch_cursor]
                                != LASM_CLUSTER_RELAY_SENDER_LIVE
                            {
                                if let Some(next_live_index) =
                                    lookup_lasm_cluster_next_live_sender_index(
                                        relay_sender_live.as_slice(),
                                        relay_next_live_sender_lookup.as_slice(),
                                        relay_dispatch_cursor,
                                    )
                                {
                                    relay_dispatch_cursor = next_live_index;
                                } else if !realign_lasm_cluster_dispatch_cursor_to_live(
                                    relay_sender_live.as_slice(),
                                    &mut relay_dispatch_cursor,
                                ) {
                                    if let Err(message) =
                                        handle_lasm_cluster_accept_unavailable_stream(
                                            client_stream,
                                            active_connections,
                                            relay_saturation_events,
                                            relay_saturation_events_total,
                                            relay_dispatch_fallback_total,
                                            relay_dispatch_short_circuit_total,
                                            &mut listener_enqueued_local,
                                            &mut listener_dispatch_counters,
                                        )
                                    {
                                        return Err(message);
                                    }
                                    continue;
                                }
                            }
                        }
                        let stream_dispatch_start = relay_dispatch_cursor;
                        let next_dispatch_wrapped =
                            if stream_dispatch_start + 1 == relay_sender_count {
                                0
                            } else {
                                stream_dispatch_start + 1
                            };
                        let next_dispatch_index = if !relay_all_senders_live {
                            if relay_live_sender_count == 1 {
                                stream_dispatch_start
                            } else if relay_live_sender_count == 2 {
                                if let Some((first_live, second_live)) =
                                    relay_dual_live_sender_indices
                                {
                                    if stream_dispatch_start == first_live {
                                        second_live
                                    } else {
                                        first_live
                                    }
                                } else {
                                    next_dispatch_wrapped
                                }
                            } else if relay_sender_live[next_dispatch_wrapped]
                                == LASM_CLUSTER_RELAY_SENDER_LIVE
                            {
                                next_dispatch_wrapped
                            } else if let Some(next_live_index) =
                                lookup_lasm_cluster_next_live_sender_index(
                                    relay_sender_live.as_slice(),
                                    relay_next_live_sender_lookup.as_slice(),
                                    next_dispatch_wrapped,
                                )
                            {
                                next_live_index
                            } else {
                                next_dispatch_wrapped
                            }
                        } else {
                            next_dispatch_wrapped
                        };
                        relay_dispatch_cursor = next_dispatch_index;

                        let live_count_before_primary_dispatch = relay_live_sender_count;
                        let mut saw_live_sender = false;
                        if let Err(stream) = attempt_lasm_cluster_relay_send(
                            client_stream,
                            relay_senders,
                            relay_sender_live.as_mut_slice(),
                            &mut relay_live_sender_count,
                            &mut relay_all_senders_live,
                            stream_dispatch_start,
                            &mut saw_live_sender,
                        ) {
                            if listener_all_senders_saturated_in_batch && saw_live_sender {
                                if let Err(message) = handle_lasm_cluster_accept_dispatch_error(
                                    LasmClusterRelayDispatchError::Saturated(stream),
                                    active_connections,
                                    relay_saturation_events,
                                    relay_saturation_events_total,
                                    relay_dispatch_fallback_total,
                                    relay_dispatch_short_circuit_total,
                                    &mut listener_enqueued_local,
                                    &mut listener_dispatch_counters
                                        .listener_saturation_pending_local,
                                    &mut listener_dispatch_counters.listener_saturation_total_local,
                                    &mut listener_dispatch_counters
                                        .listener_dispatch_fallback_total_local,
                                    &mut listener_dispatch_counters
                                        .listener_dispatch_short_circuit_total_local,
                                ) {
                                    return Err(message);
                                }
                                listener_dispatch_counters
                                    .listener_dispatch_short_circuit_total_local += 1;
                                continue;
                            }
                            listener_dispatch_counters.listener_dispatch_fallback_total_local += 1;
                            let live_count_before_fallback = relay_live_sender_count;
                            let primary_disconnected =
                                relay_live_sender_count != live_count_before_primary_dispatch;
                            if primary_disconnected {
                                relay_single_live_sender_index = None;
                                relay_dual_live_sender_indices = None;
                            }
                            let relay_has_next_live_sender_lookup_for_fallback =
                                relay_has_next_live_sender_lookup && relay_live_sender_count > 2;
                            if primary_disconnected
                                && relay_has_next_live_sender_lookup_for_fallback
                            {
                                refresh_lasm_cluster_next_live_sender_lookup(
                                    relay_sender_live.as_slice(),
                                    relay_next_live_sender_lookup.as_mut_slice(),
                                );
                            }
                            let live_count_after_primary_dispatch = relay_live_sender_count;
                            let dispatch_result =
                                dispatch_lasm_cluster_relay_stream_fallback_with_live_hints(
                                    stream,
                                    relay_senders,
                                    relay_sender_live.as_mut_slice(),
                                    relay_next_live_sender_lookup.as_slice(),
                                    relay_has_next_live_sender_lookup_for_fallback,
                                    &mut relay_live_sender_count,
                                    &mut relay_all_senders_live,
                                    next_dispatch_index,
                                    saw_live_sender,
                                    stream_dispatch_start,
                                    &mut relay_single_live_sender_index,
                                    &mut relay_dual_live_sender_indices,
                                );
                            if !relay_all_senders_live
                                && relay_live_sender_count > 1
                                && relay_sender_live[relay_dispatch_cursor]
                                    == LASM_CLUSTER_RELAY_SENDER_DEAD
                            {
                                let _ = realign_lasm_cluster_dispatch_cursor_to_live(
                                    relay_sender_live.as_slice(),
                                    &mut relay_dispatch_cursor,
                                );
                            }
                            if relay_live_sender_count != live_count_before_fallback {
                                refresh_lasm_cluster_live_sender_hints(
                                    relay_sender_live.as_slice(),
                                    relay_live_sender_count,
                                    &mut relay_single_live_sender_index,
                                    &mut relay_dual_live_sender_indices,
                                );
                                if relay_live_sender_count > 2
                                    && relay_has_next_live_sender_lookup
                                    && relay_live_sender_count != live_count_after_primary_dispatch
                                {
                                    refresh_lasm_cluster_next_live_sender_lookup(
                                        relay_sender_live.as_slice(),
                                        relay_next_live_sender_lookup.as_mut_slice(),
                                    );
                                }
                                relay_live_sender_count_observed
                                    .fetch_min(relay_live_sender_count, Ordering::Relaxed);
                            }
                            match dispatch_result {
                                Ok(()) => {
                                    listener_enqueued_local += 1;
                                    listener_all_senders_saturated_in_batch = false;
                                }
                                Err(dispatch_error) => {
                                    if matches!(
                                        &dispatch_error,
                                        LasmClusterRelayDispatchError::Saturated(_)
                                    ) {
                                        listener_all_senders_saturated_in_batch = true;
                                    }
                                    if let Err(message) = handle_lasm_cluster_accept_dispatch_error(
                                        dispatch_error,
                                        active_connections,
                                        relay_saturation_events,
                                        relay_saturation_events_total,
                                        relay_dispatch_fallback_total,
                                        relay_dispatch_short_circuit_total,
                                        &mut listener_enqueued_local,
                                        &mut listener_dispatch_counters
                                            .listener_saturation_pending_local,
                                        &mut listener_dispatch_counters
                                            .listener_saturation_total_local,
                                        &mut listener_dispatch_counters
                                            .listener_dispatch_fallback_total_local,
                                        &mut listener_dispatch_counters
                                            .listener_dispatch_short_circuit_total_local,
                                    ) {
                                        return Err(message);
                                    }
                                }
                            }
                        } else {
                            listener_enqueued_local += 1;
                            listener_all_senders_saturated_in_batch = false;
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(err) => {
                        flush_lasm_cluster_accept_dispatch_counters(
                            relay_saturation_events,
                            relay_saturation_events_total,
                            relay_dispatch_fallback_total,
                            relay_dispatch_short_circuit_total,
                            &mut listener_dispatch_counters,
                        );
                        return Err(format!("LASM cluster proxy accept error: {err}"));
                    }
                }
            }
        }
        if listener_accepted_in_batch == 0 {
            listener_idle_spins += 1;
            if listener_idle_spins < LASM_CLUSTER_IDLE_SPIN_THRESHOLD {
                std::thread::yield_now();
            } else {
                std::thread::sleep(listener_idle_sleep_duration);
                listener_idle_spins = 0;
            }
            continue;
        }
        listener_idle_spins = 0;

        if listener_enqueued_local > 0 {
            flush_lasm_cluster_active_connection_increments(
                active_connections,
                &mut listener_enqueued_local,
            );
        }
        if listener_dispatch_counters.listener_dispatch_fallback_total_local > 0 {
            flush_lasm_cluster_dispatch_fallback_total(
                relay_dispatch_fallback_total,
                &mut listener_dispatch_counters.listener_dispatch_fallback_total_local,
            );
        }
        if listener_dispatch_counters.listener_dispatch_short_circuit_total_local > 0 {
            flush_lasm_cluster_dispatch_short_circuit_total(
                relay_dispatch_short_circuit_total,
                &mut listener_dispatch_counters.listener_dispatch_short_circuit_total_local,
            );
        }
    }

    flush_lasm_cluster_accept_dispatch_counters(
        relay_saturation_events,
        relay_saturation_events_total,
        relay_dispatch_fallback_total,
        relay_dispatch_short_circuit_total,
        &mut listener_dispatch_counters,
    );
    Ok(())
}
