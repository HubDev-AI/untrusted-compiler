use crossbeam_channel::Sender;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use crate::lasm_cluster_lifecycle::stop_lasm_cluster_workers;
use crate::LasmClusterState;

pub(crate) struct LasmClusterShutdownSummary {
    pub(crate) relay_worker_panics: usize,
    pub(crate) autoscale_panicked: bool,
    pub(crate) status_writer_panicked: bool,
    pub(crate) state_lock_poisoned: bool,
}

impl LasmClusterShutdownSummary {
    pub(crate) fn has_thread_panics(&self) -> bool {
        self.relay_worker_panics > 0 || self.autoscale_panicked || self.status_writer_panicked
    }

    pub(crate) fn has_failures(&self) -> bool {
        self.has_thread_panics() || self.state_lock_poisoned
    }

    pub(crate) fn failure_message(&self) -> String {
        format!(
            "LASM cluster shutdown observed failure(s): relay_worker_panics={}, autoscale_panicked={}, status_writer_panicked={}, state_lock_poisoned={}",
            self.relay_worker_panics,
            self.autoscale_panicked,
            self.status_writer_panicked,
            self.state_lock_poisoned
        )
    }
}

pub(crate) fn finalize_lasm_cluster_runtime(
    stop_flag: &Arc<AtomicBool>,
    relay_senders: Arc<Vec<Sender<TcpStream>>>,
    relay_handles: Vec<std::thread::JoinHandle<()>>,
    autoscale_handle: std::thread::JoinHandle<()>,
    status_writer_handle: Option<std::thread::JoinHandle<()>>,
    shared_state: &Arc<RwLock<LasmClusterState>>,
) -> LasmClusterShutdownSummary {
    stop_flag.store(true, Ordering::Relaxed);
    drop(relay_senders);
    let mut relay_worker_panics = 0_usize;
    for handle in relay_handles {
        if handle.join().is_err() {
            relay_worker_panics += 1;
        }
    }
    let autoscale_panicked = autoscale_handle.join().is_err();
    let status_writer_panicked = status_writer_handle
        .map(|handle| handle.join().is_err())
        .unwrap_or(false);
    let state_lock_poisoned = match shared_state.write() {
        Ok(mut state) => {
            stop_lasm_cluster_workers(&mut state);
            false
        }
        Err(_) => true,
    };
    LasmClusterShutdownSummary {
        relay_worker_panics,
        autoscale_panicked,
        status_writer_panicked,
        state_lock_poisoned,
    }
}
