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
}

impl LasmClusterShutdownSummary {
    pub(crate) fn has_thread_panics(&self) -> bool {
        self.relay_worker_panics > 0 || self.autoscale_panicked || self.status_writer_panicked
    }

    pub(crate) fn panic_message(&self) -> String {
        format!(
            "LASM cluster shutdown observed thread panic(s): relay_workers={}, autoscale_panicked={}, status_writer_panicked={}",
            self.relay_worker_panics, self.autoscale_panicked, self.status_writer_panicked
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
    if let Ok(mut state) = shared_state.write() {
        stop_lasm_cluster_workers(&mut state);
    }
    LasmClusterShutdownSummary {
        relay_worker_panics,
        autoscale_panicked,
        status_writer_panicked,
    }
}
