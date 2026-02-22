use crossbeam_channel::Sender;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use crate::lasm_cluster_lifecycle::stop_lasm_cluster_workers;
use crate::LasmClusterState;

pub(crate) fn finalize_lasm_cluster_runtime(
    stop_flag: &Arc<AtomicBool>,
    relay_senders: Arc<Vec<Sender<TcpStream>>>,
    relay_handles: Vec<std::thread::JoinHandle<()>>,
    autoscale_handle: std::thread::JoinHandle<()>,
    status_writer_handle: Option<std::thread::JoinHandle<()>>,
    shared_state: &Arc<RwLock<LasmClusterState>>,
) {
    stop_flag.store(true, Ordering::Relaxed);
    drop(relay_senders);
    for handle in relay_handles {
        let _ = handle.join();
    }
    let _ = autoscale_handle.join();
    if let Some(handle) = status_writer_handle {
        let _ = handle.join();
    }
    if let Ok(mut state) = shared_state.write() {
        stop_lasm_cluster_workers(&mut state);
    }
}
