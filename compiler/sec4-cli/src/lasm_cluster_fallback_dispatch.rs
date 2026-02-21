use crossbeam_channel::Sender;
use std::net::TcpStream;

use crate::lasm_cluster_relay_send::attempt_lasm_cluster_relay_send;
use crate::lasm_cluster_relay_topology::{
    lasm_cluster_next_index_wrapped, lasm_cluster_next_live_sender_index,
    lookup_lasm_cluster_next_live_sender_index, resolve_lasm_cluster_next_live_sender_index,
};
use crate::{LasmClusterRelayDispatchError, LASM_CLUSTER_RELAY_SENDER_DEAD, LASM_CLUSTER_RELAY_SENDER_LIVE};

#[inline(always)]
fn lasm_cluster_fallback_terminal_dispatch_error(
    client_stream: TcpStream,
    saw_live_sender: bool,
    relay_live_sender_count: usize,
) -> Result<(), LasmClusterRelayDispatchError> {
    if saw_live_sender && relay_live_sender_count > 0 {
        Err(LasmClusterRelayDispatchError::Saturated(client_stream))
    } else {
        Err(LasmClusterRelayDispatchError::Unavailable(client_stream))
    }
}

#[inline(always)]
fn lasm_cluster_forward_distance_wrapped(start: usize, end: usize, count: usize) -> usize {
    debug_assert!(count > 0);
    debug_assert!(start < count);
    debug_assert!(end < count);
    if end >= start {
        end - start
    } else {
        count - start + end
    }
}

#[inline(always)]
fn advance_lasm_cluster_fallback_scan_index(
    relay_sender_live: &[u8],
    relay_next_live_sender_lookup: &[usize],
    current_index: usize,
) -> (usize, usize) {
    let sender_count = relay_sender_live.len();
    debug_assert!(sender_count > 0);
    debug_assert!(current_index < sender_count);

    let next_scan_start = lasm_cluster_next_index_wrapped(current_index, sender_count);
    let next_scan_index = if relay_sender_live[next_scan_start] == LASM_CLUSTER_RELAY_SENDER_LIVE {
        next_scan_start
    } else if !relay_next_live_sender_lookup.is_empty() {
        debug_assert_eq!(relay_next_live_sender_lookup.len(), relay_sender_live.len());
        if let Some(next_live_index) = lookup_lasm_cluster_next_live_sender_index(
            relay_sender_live,
            relay_next_live_sender_lookup,
            next_scan_start,
        ) {
            next_live_index
        } else {
            next_scan_start
        }
    } else if let Some(next_live_index) =
        lasm_cluster_next_live_sender_index(relay_sender_live, next_scan_start)
    {
        next_live_index
    } else {
        next_scan_start
    };
    let advanced_slots =
        lasm_cluster_forward_distance_wrapped(current_index, next_scan_index, sender_count).max(1);
    (next_scan_index, advanced_slots)
}

#[inline(always)]
pub(crate) fn dispatch_lasm_cluster_relay_stream_fallback_dual_live(
    client_stream: TcpStream,
    relay_senders: &[Sender<TcpStream>],
    relay_sender_live: &mut [u8],
    relay_live_sender_count: &mut usize,
    relay_all_senders_live: &mut bool,
    alternate_live_index: usize,
    saw_live_sender: bool,
) -> Result<(), LasmClusterRelayDispatchError> {
    debug_assert!(alternate_live_index < relay_senders.len());
    if relay_sender_live
        .get(alternate_live_index)
        .copied()
        .unwrap_or(LASM_CLUSTER_RELAY_SENDER_DEAD)
        != LASM_CLUSTER_RELAY_SENDER_LIVE
    {
        return lasm_cluster_fallback_terminal_dispatch_error(
            client_stream,
            saw_live_sender,
            *relay_live_sender_count,
        );
    }
    let mut saw_live_sender_dynamic = saw_live_sender;
    match attempt_lasm_cluster_relay_send(
        client_stream,
        relay_senders,
        relay_sender_live,
        relay_live_sender_count,
        relay_all_senders_live,
        alternate_live_index,
        &mut saw_live_sender_dynamic,
    ) {
        Ok(()) => Ok(()),
        Err(stream) => lasm_cluster_fallback_terminal_dispatch_error(
            stream,
            saw_live_sender_dynamic,
            *relay_live_sender_count,
        ),
    }
}

#[inline(always)]
pub(crate) fn dispatch_lasm_cluster_relay_stream_fallback_single_live(
    client_stream: TcpStream,
    relay_senders: &[Sender<TcpStream>],
    relay_sender_live: &mut [u8],
    relay_live_sender_count: &mut usize,
    relay_all_senders_live: &mut bool,
    single_live_index: usize,
    saw_live_sender: bool,
) -> Result<(), LasmClusterRelayDispatchError> {
    debug_assert!(single_live_index < relay_senders.len());
    if relay_sender_live
        .get(single_live_index)
        .copied()
        .unwrap_or(LASM_CLUSTER_RELAY_SENDER_DEAD)
        != LASM_CLUSTER_RELAY_SENDER_LIVE
    {
        return lasm_cluster_fallback_terminal_dispatch_error(
            client_stream,
            saw_live_sender,
            *relay_live_sender_count,
        );
    }
    let mut saw_live_sender_dynamic = saw_live_sender;
    match attempt_lasm_cluster_relay_send(
        client_stream,
        relay_senders,
        relay_sender_live,
        relay_live_sender_count,
        relay_all_senders_live,
        single_live_index,
        &mut saw_live_sender_dynamic,
    ) {
        Ok(()) => Ok(()),
        Err(stream) => lasm_cluster_fallback_terminal_dispatch_error(
            stream,
            saw_live_sender_dynamic,
            *relay_live_sender_count,
        ),
    }
}

#[inline(always)]
pub(crate) fn dispatch_lasm_cluster_relay_stream_fallback_multi(
    mut client_stream: TcpStream,
    relay_senders: &[Sender<TcpStream>],
    relay_sender_live: &mut [u8],
    relay_next_live_sender_lookup: &[usize],
    relay_live_sender_count: &mut usize,
    relay_all_senders_live: &mut bool,
    start_index_wrapped: usize,
    mut saw_live_sender: bool,
) -> Result<(), LasmClusterRelayDispatchError> {
    let sender_count = relay_senders.len();
    debug_assert!(sender_count > 1);
    debug_assert_eq!(relay_sender_live.len(), sender_count);
    debug_assert!(start_index_wrapped < sender_count);
    if *relay_live_sender_count == 0 {
        return Err(LasmClusterRelayDispatchError::Unavailable(client_stream));
    }
    if sender_count == 2 {
        return dispatch_lasm_cluster_relay_stream_fallback_single_live(
            client_stream,
            relay_senders,
            relay_sender_live,
            relay_live_sender_count,
            relay_all_senders_live,
            start_index_wrapped,
            saw_live_sender,
        );
    }
    let mut scan_index = start_index_wrapped;
    let scan_slot_limit = sender_count.saturating_sub(1);
    if *relay_all_senders_live {
        for _ in 0..scan_slot_limit {
            if let Err(next_stream) = attempt_lasm_cluster_relay_send(
                client_stream,
                relay_senders,
                relay_sender_live,
                relay_live_sender_count,
                relay_all_senders_live,
                scan_index,
                &mut saw_live_sender,
            ) {
                client_stream = next_stream;
            } else {
                return Ok(());
            }
            scan_index = lasm_cluster_next_index_wrapped(scan_index, sender_count);
        }
        return lasm_cluster_fallback_terminal_dispatch_error(
            client_stream,
            saw_live_sender,
            *relay_live_sender_count,
        );
    }
    let scan_live_target = relay_live_sender_count.saturating_sub(1);
    if scan_live_target <= 1 {
        if let Some(live_index) = resolve_lasm_cluster_next_live_sender_index(
            relay_sender_live,
            relay_next_live_sender_lookup,
            start_index_wrapped,
        ) {
            return dispatch_lasm_cluster_relay_stream_fallback_single_live(
                client_stream,
                relay_senders,
                relay_sender_live,
                relay_live_sender_count,
                relay_all_senders_live,
                live_index,
                saw_live_sender,
            );
        }
        return lasm_cluster_fallback_terminal_dispatch_error(
            client_stream,
            saw_live_sender,
            *relay_live_sender_count,
        );
    }
    if scan_live_target == 2 {
        if let Some(first_live_index) = resolve_lasm_cluster_next_live_sender_index(
            relay_sender_live,
            relay_next_live_sender_lookup,
            start_index_wrapped,
        ) {
            if let Err(next_stream) = attempt_lasm_cluster_relay_send(
                client_stream,
                relay_senders,
                relay_sender_live,
                relay_live_sender_count,
                relay_all_senders_live,
                first_live_index,
                &mut saw_live_sender,
            ) {
                client_stream = next_stream;
            } else {
                return Ok(());
            }
            if *relay_live_sender_count == 0 {
                return lasm_cluster_fallback_terminal_dispatch_error(
                    client_stream,
                    saw_live_sender,
                    *relay_live_sender_count,
                );
            }
            let second_start_index = lasm_cluster_next_index_wrapped(first_live_index, sender_count);
            let second_live_index = if relay_sender_live[second_start_index]
                == LASM_CLUSTER_RELAY_SENDER_LIVE
            {
                Some(second_start_index)
            } else {
                resolve_lasm_cluster_next_live_sender_index(
                    relay_sender_live,
                    relay_next_live_sender_lookup,
                    second_start_index,
                )
            };
            if let Some(second_live_index) = second_live_index {
                return dispatch_lasm_cluster_relay_stream_fallback_single_live(
                    client_stream,
                    relay_senders,
                    relay_sender_live,
                    relay_live_sender_count,
                    relay_all_senders_live,
                    second_live_index,
                    saw_live_sender,
                );
            }
        }
        return lasm_cluster_fallback_terminal_dispatch_error(
            client_stream,
            saw_live_sender,
            *relay_live_sender_count,
        );
    }
    let mut scan_live_target_dynamic = scan_live_target;
    let mut scanned_slots = 0usize;
    let mut scanned_live = 0usize;
    while scanned_slots < scan_slot_limit && scanned_live < scan_live_target_dynamic {
        if relay_sender_live[scan_index] == LASM_CLUSTER_RELAY_SENDER_DEAD {
            if scanned_slots.saturating_add(1) >= scan_slot_limit {
                break;
            }
            let (next_scan_index, advanced_slots) = advance_lasm_cluster_fallback_scan_index(
                relay_sender_live,
                relay_next_live_sender_lookup,
                scan_index,
            );
            scan_index = next_scan_index;
            scanned_slots = scanned_slots.saturating_add(advanced_slots);
            continue;
        }
        scanned_live += 1;
        let live_sender_count_before_attempt = *relay_live_sender_count;
        if let Err(next_stream) = attempt_lasm_cluster_relay_send(
            client_stream,
            relay_senders,
            relay_sender_live,
            relay_live_sender_count,
            relay_all_senders_live,
            scan_index,
            &mut saw_live_sender,
        ) {
            if *relay_live_sender_count != live_sender_count_before_attempt {
                scan_live_target_dynamic = relay_live_sender_count.saturating_sub(1);
            }
            client_stream = next_stream;
        } else {
            return Ok(());
        }
        if *relay_live_sender_count == 0 {
            return Err(LasmClusterRelayDispatchError::Unavailable(client_stream));
        }
        if scanned_live >= scan_live_target_dynamic {
            break;
        }
        if scanned_slots.saturating_add(1) >= scan_slot_limit {
            break;
        }
        let (next_scan_index, advanced_slots) = advance_lasm_cluster_fallback_scan_index(
            relay_sender_live,
            relay_next_live_sender_lookup,
            scan_index,
        );
        scan_index = next_scan_index;
        scanned_slots = scanned_slots.saturating_add(advanced_slots);
    }

    lasm_cluster_fallback_terminal_dispatch_error(
        client_stream,
        saw_live_sender,
        *relay_live_sender_count,
    )
}
