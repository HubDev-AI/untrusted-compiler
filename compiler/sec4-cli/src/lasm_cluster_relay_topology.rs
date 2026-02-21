use crate::{LASM_CLUSTER_RELAY_SENDER_DEAD, LASM_CLUSTER_RELAY_SENDER_LIVE};

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
    debug_assert_eq!(relay_sender_live.len(), relay_next_live_sender_lookup.len());
    if relay_sender_live.is_empty() {
        return None;
    }
    debug_assert!(start_index_wrapped < relay_sender_live.len());
    let cached_index = relay_next_live_sender_lookup[start_index_wrapped];
    if cached_index < relay_sender_live.len()
        && relay_sender_live[cached_index] == LASM_CLUSTER_RELAY_SENDER_LIVE
    {
        return Some(cached_index);
    }
    lasm_cluster_next_live_sender_index(relay_sender_live, start_index_wrapped)
}

#[inline(always)]
pub(crate) fn resolve_lasm_cluster_next_live_sender_index(
    relay_sender_live: &[u8],
    relay_next_live_sender_lookup: &[usize],
    start_index_wrapped: usize,
) -> Option<usize> {
    if relay_sender_live
        .get(start_index_wrapped)
        .copied()
        .unwrap_or(LASM_CLUSTER_RELAY_SENDER_DEAD)
        == LASM_CLUSTER_RELAY_SENDER_LIVE
    {
        return Some(start_index_wrapped);
    }
    if !relay_next_live_sender_lookup.is_empty() {
        debug_assert_eq!(relay_next_live_sender_lookup.len(), relay_sender_live.len());
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
