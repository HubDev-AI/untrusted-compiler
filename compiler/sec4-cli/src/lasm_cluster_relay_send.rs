use crossbeam_channel::{Sender, TrySendError};
use std::net::TcpStream;

use crate::lasm_cluster_fallback_dispatch::LasmClusterRelayDispatchError;
use crate::LASM_CLUSTER_RELAY_SENDER_DEAD;

#[inline(always)]
pub(crate) fn attempt_lasm_cluster_relay_send(
    client_stream: TcpStream,
    relay_senders: &[Sender<TcpStream>],
    relay_sender_live: &mut [u8],
    relay_live_sender_count: &mut usize,
    relay_all_senders_live: &mut bool,
    relay_index: usize,
    saw_live_sender: &mut bool,
) -> Result<(), TcpStream> {
    debug_assert!(relay_index < relay_senders.len());
    match relay_senders[relay_index].try_send(client_stream) {
        Ok(()) => Ok(()),
        Err(TrySendError::Full(next_stream)) => {
            *saw_live_sender = true;
            Err(next_stream)
        }
        Err(TrySendError::Disconnected(next_stream)) => {
            if relay_sender_live[relay_index] != LASM_CLUSTER_RELAY_SENDER_DEAD {
                relay_sender_live[relay_index] = LASM_CLUSTER_RELAY_SENDER_DEAD;
                if *relay_live_sender_count > 0 {
                    *relay_live_sender_count -= 1;
                } else {
                    debug_assert_eq!(*relay_live_sender_count, 0);
                }
                *relay_all_senders_live = false;
            }
            Err(next_stream)
        }
    }
}

#[inline(always)]
pub(crate) fn attempt_lasm_cluster_relay_send_single(
    client_stream: TcpStream,
    relay_sender: &Sender<TcpStream>,
) -> Result<(), LasmClusterRelayDispatchError> {
    match relay_sender.try_send(client_stream) {
        Ok(()) => Ok(()),
        Err(TrySendError::Full(stream)) => Err(LasmClusterRelayDispatchError::Saturated(stream)),
        Err(TrySendError::Disconnected(stream)) => {
            Err(LasmClusterRelayDispatchError::Unavailable(stream))
        }
    }
}
