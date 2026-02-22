use crate::LASM_CLUSTER_RELAY_SENDER_LIVE;

#[inline(always)]
pub(crate) fn lasm_cluster_next_index_wrapped(index: usize, count: usize) -> usize {
    debug_assert!(count > 0);
    if index + 1 == count {
        0
    } else {
        index + 1
    }
}

#[inline(always)]
pub(crate) fn lasm_cluster_next_live_sender_index(
    relay_sender_live: &[u8],
    start_index_wrapped: usize,
) -> Option<usize> {
    let sender_count = relay_sender_live.len();
    debug_assert!(sender_count > 0);
    debug_assert!(start_index_wrapped < sender_count);
    let mut scan_index = start_index_wrapped;
    for _ in 0..sender_count {
        if relay_sender_live[scan_index] == LASM_CLUSTER_RELAY_SENDER_LIVE {
            return Some(scan_index);
        }
        scan_index = lasm_cluster_next_index_wrapped(scan_index, sender_count);
    }
    None
}

#[inline(always)]
pub(crate) fn lookup_lasm_cluster_next_live_sender_index(
    relay_sender_live: &[u8],
    relay_next_live_sender_lookup: &[usize],
    start_index_wrapped: usize,
) -> Option<usize> {
    let sender_count = relay_sender_live.len();
    debug_assert!(sender_count > 0);
    debug_assert_eq!(relay_sender_live.len(), relay_next_live_sender_lookup.len());
    debug_assert!(start_index_wrapped < sender_count);
    let cached_index = relay_next_live_sender_lookup[start_index_wrapped];
    debug_assert!(cached_index < sender_count);
    if relay_sender_live[cached_index] == LASM_CLUSTER_RELAY_SENDER_LIVE {
        return Some(cached_index);
    }
    lasm_cluster_next_live_sender_index(relay_sender_live, start_index_wrapped)
}

#[inline(always)]
pub(crate) fn resolve_lasm_cluster_next_live_sender_index_with_lookup_state(
    relay_sender_live: &[u8],
    relay_next_live_sender_lookup: &[usize],
    relay_has_next_live_sender_lookup: bool,
    start_index_wrapped: usize,
) -> Option<usize> {
    let sender_count = relay_sender_live.len();
    debug_assert!(sender_count > 0);
    debug_assert!(start_index_wrapped < sender_count);
    if relay_sender_live[start_index_wrapped] == LASM_CLUSTER_RELAY_SENDER_LIVE {
        return Some(start_index_wrapped);
    }
    if relay_has_next_live_sender_lookup {
        debug_assert_eq!(relay_next_live_sender_lookup.len(), sender_count);
        return lookup_lasm_cluster_next_live_sender_index(
            relay_sender_live,
            relay_next_live_sender_lookup,
            start_index_wrapped,
        );
    }
    lasm_cluster_next_live_sender_index(relay_sender_live, start_index_wrapped)
}

#[inline(always)]
pub(crate) fn realign_lasm_cluster_dispatch_cursor_to_live(
    relay_sender_live: &[u8],
    relay_dispatch_cursor: &mut usize,
) -> bool {
    debug_assert!(!relay_sender_live.is_empty());
    if relay_sender_live[*relay_dispatch_cursor] == LASM_CLUSTER_RELAY_SENDER_LIVE {
        return true;
    }
    match lasm_cluster_next_live_sender_index(relay_sender_live, *relay_dispatch_cursor) {
        Some(index) => {
            *relay_dispatch_cursor = index;
            true
        }
        None => false,
    }
}

#[inline(always)]
pub(crate) fn refresh_lasm_cluster_next_live_sender_lookup(
    relay_sender_live: &[u8],
    relay_next_live_sender_lookup: &mut [usize],
) {
    debug_assert_eq!(relay_sender_live.len(), relay_next_live_sender_lookup.len());
    let sender_count = relay_sender_live.len();
    if sender_count == 0 {
        return;
    }
    let Some(first_live_index) = relay_sender_live
        .iter()
        .position(|value| *value == LASM_CLUSTER_RELAY_SENDER_LIVE)
    else {
        for (index, slot) in relay_next_live_sender_lookup.iter_mut().enumerate() {
            *slot = index;
        }
        return;
    };

    let mut next_live_index = first_live_index;
    for reverse_offset in 0..sender_count {
        let index = if reverse_offset == 0 {
            first_live_index
        } else {
            first_live_index + sender_count - reverse_offset
        };
        let wrapped_index = if index >= sender_count {
            index - sender_count
        } else {
            index
        };
        if relay_sender_live[wrapped_index] == LASM_CLUSTER_RELAY_SENDER_LIVE {
            next_live_index = wrapped_index;
        }
        relay_next_live_sender_lookup[wrapped_index] = next_live_index;
    }
}

#[inline(always)]
pub(crate) fn refresh_lasm_cluster_single_live_sender_index(
    relay_sender_live: &[u8],
    relay_live_sender_count: usize,
    relay_single_live_sender_index: &mut Option<usize>,
) {
    if relay_live_sender_count != 1 {
        *relay_single_live_sender_index = None;
        return;
    }
    if let Some(index) = *relay_single_live_sender_index {
        if relay_sender_live[index] == LASM_CLUSTER_RELAY_SENDER_LIVE {
            return;
        }
    }
    *relay_single_live_sender_index = lasm_cluster_next_live_sender_index(relay_sender_live, 0);
}

#[inline(always)]
pub(crate) fn refresh_lasm_cluster_dual_live_sender_indices(
    relay_sender_live: &[u8],
    relay_live_sender_count: usize,
    relay_dual_live_sender_indices: &mut Option<(usize, usize)>,
) {
    if relay_live_sender_count != 2 {
        *relay_dual_live_sender_indices = None;
        return;
    }
    if let Some((first, second)) = *relay_dual_live_sender_indices {
        if relay_sender_live[first] == LASM_CLUSTER_RELAY_SENDER_LIVE
            && relay_sender_live[second] == LASM_CLUSTER_RELAY_SENDER_LIVE
        {
            return;
        }
    }
    let mut first_live = None;
    let mut second_live = None;
    for (index, value) in relay_sender_live.iter().enumerate() {
        if *value != LASM_CLUSTER_RELAY_SENDER_LIVE {
            continue;
        }
        if first_live.is_none() {
            first_live = Some(index);
            continue;
        }
        second_live = Some(index);
        break;
    }
    *relay_dual_live_sender_indices = match (first_live, second_live) {
        (Some(first), Some(second)) => Some((first, second)),
        _ => None,
    };
}

#[inline(always)]
pub(crate) fn refresh_lasm_cluster_live_sender_hints(
    relay_sender_live: &[u8],
    relay_live_sender_count: usize,
    relay_single_live_sender_index: &mut Option<usize>,
    relay_dual_live_sender_indices: &mut Option<(usize, usize)>,
) {
    if relay_live_sender_count == 1 {
        let mut single_live_index = if let Some(index) = *relay_single_live_sender_index {
            if relay_sender_live[index] == LASM_CLUSTER_RELAY_SENDER_LIVE {
                Some(index)
            } else {
                None
            }
        } else {
            None
        };
        if single_live_index.is_none() {
            if let Some((first, second)) = *relay_dual_live_sender_indices {
                if relay_sender_live[first] == LASM_CLUSTER_RELAY_SENDER_LIVE {
                    single_live_index = Some(first);
                } else if relay_sender_live[second] == LASM_CLUSTER_RELAY_SENDER_LIVE {
                    single_live_index = Some(second);
                }
            }
        }
        if single_live_index.is_none() {
            single_live_index = lasm_cluster_next_live_sender_index(relay_sender_live, 0);
        }
        *relay_single_live_sender_index = single_live_index;
        *relay_dual_live_sender_indices = None;
        return;
    }
    *relay_single_live_sender_index = None;
    if relay_live_sender_count == 2 {
        refresh_lasm_cluster_dual_live_sender_indices(
            relay_sender_live,
            relay_live_sender_count,
            relay_dual_live_sender_indices,
        );
        return;
    }
    *relay_dual_live_sender_indices = None;
}
