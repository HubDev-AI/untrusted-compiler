use crate::lasm_db_runtime_postgres::{
    persist_lasm_postgres_record_append_batch_thread_local,
    persist_lasm_postgres_record_append_thread_local,
    persist_lasm_postgres_records_full_sync_thread_local, LasmPostgresThreadLocalConfig,
};
use crate::LasmDbRecord;
use crossbeam_channel::{bounded, SendError, Sender, TryRecvError, TrySendError};
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

const LASM_POSTGRES_PERSIST_WORKERS_ENV: &str = "SEC4_RT_LASM_DB_POSTGRES_PERSIST_WORKERS";
const LASM_POSTGRES_PERSIST_WORKERS_DEFAULT: usize = 4;
const LASM_POSTGRES_PERSIST_WORKERS_MIN: usize = 1;
const LASM_POSTGRES_PERSIST_WORKERS_MAX: usize = 128;
const LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_ENV: &str =
    "SEC4_RT_LASM_DB_POSTGRES_PERSIST_QUEUE_CAPACITY";
const LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_DEFAULT: usize = 8192;
const LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_MIN: usize = 256;
const LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_MAX: usize = 1_048_576;
const LASM_POSTGRES_PERSIST_BATCH_MAX_ENV: &str = "SEC4_RT_LASM_DB_POSTGRES_PERSIST_BATCH_MAX";
const LASM_POSTGRES_PERSIST_BATCH_MAX_DEFAULT: usize = 256;
const LASM_POSTGRES_PERSIST_BATCH_MAX_MIN: usize = 1;
const LASM_POSTGRES_PERSIST_BATCH_MAX_MAX: usize = 4096;
const LASM_POSTGRES_PERSIST_QUEUE_FULL_MODE_ENV: &str =
    "SEC4_RT_LASM_DB_POSTGRES_PERSIST_QUEUE_FULL_MODE";
const LASM_RUNTIME_MAX_CONCURRENCY_ENV: &str = "SEC4_RT_LASM_MAX_CONCURRENCY";
const LASM_POSTGRES_PERSIST_WORKERS_AUTO_DEFAULT_CAP: usize = 16;
const LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_AUTO_DEFAULT_CAP: usize = 131_072;

static LASM_POSTGRES_PERSIST_WORKERS_RESOLVED: OnceLock<usize> = OnceLock::new();
static LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_RESOLVED: OnceLock<usize> = OnceLock::new();
static LASM_POSTGRES_PERSIST_BATCH_MAX_RESOLVED: OnceLock<usize> = OnceLock::new();
static LASM_POSTGRES_PERSIST_QUEUE_FULL_MODE_RESOLVED: OnceLock<LasmPostgresPersistQueueFullMode> =
    OnceLock::new();
static LASM_POSTGRES_PERSIST_QUEUE: OnceLock<Sender<LasmPostgresPersistTask>> = OnceLock::new();
static LASM_POSTGRES_PERSIST_CONFIG_LOCKS: OnceLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> =
    OnceLock::new();
static LASM_POSTGRES_PERSIST_WORKERS_AVAILABLE: AtomicBool = AtomicBool::new(false);
static LASM_POSTGRES_PERSIST_QUEUE_BACKPRESSURE_TOTAL: AtomicUsize = AtomicUsize::new(0);
static LASM_POSTGRES_PERSIST_SYNC_FALLBACK_TOTAL: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy)]
enum LasmPostgresPersistQueueFullMode {
    Block,
    SyncFallback,
}

#[inline(always)]
fn lasm_postgres_persist_queue_full_mode_label(
    mode: LasmPostgresPersistQueueFullMode,
) -> &'static str {
    match mode {
        LasmPostgresPersistQueueFullMode::Block => "block",
        LasmPostgresPersistQueueFullMode::SyncFallback => "sync-fallback",
    }
}

#[inline(always)]
fn resolve_lasm_runtime_max_concurrency_hint() -> Option<usize> {
    let raw = env::var(LASM_RUNTIME_MAX_CONCURRENCY_ENV).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<usize>().ok().filter(|value| *value > 0)
}

#[inline(always)]
fn resolve_lasm_postgres_persist_workers_auto_default() -> usize {
    match resolve_lasm_runtime_max_concurrency_hint() {
        Some(hint) => {
            // Keep persist workers below request-worker fanout while still scaling
            // with runtime concurrency for DB-heavy workloads.
            let derived = hint.div_ceil(8);
            derived.clamp(
                LASM_POSTGRES_PERSIST_WORKERS_DEFAULT,
                LASM_POSTGRES_PERSIST_WORKERS_AUTO_DEFAULT_CAP,
            )
        }
        None => LASM_POSTGRES_PERSIST_WORKERS_DEFAULT,
    }
}

#[inline(always)]
fn resolve_lasm_postgres_persist_queue_capacity_auto_default() -> usize {
    match resolve_lasm_runtime_max_concurrency_hint() {
        Some(hint) => {
            // Provide enough queue headroom per concurrent worker without
            // unbounded growth on high-concurrency settings.
            let derived = hint.saturating_mul(64);
            derived.clamp(
                LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_DEFAULT,
                LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_AUTO_DEFAULT_CAP,
            )
        }
        None => LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_DEFAULT,
    }
}

#[inline(always)]
fn resolve_lasm_postgres_persist_workers() -> usize {
    *LASM_POSTGRES_PERSIST_WORKERS_RESOLVED.get_or_init(|| {
        let Ok(raw) = env::var(LASM_POSTGRES_PERSIST_WORKERS_ENV) else {
            return resolve_lasm_postgres_persist_workers_auto_default();
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return resolve_lasm_postgres_persist_workers_auto_default();
        }
        let Ok(parsed) = trimmed.parse::<usize>() else {
            return resolve_lasm_postgres_persist_workers_auto_default();
        };
        parsed.clamp(
            LASM_POSTGRES_PERSIST_WORKERS_MIN,
            LASM_POSTGRES_PERSIST_WORKERS_MAX,
        )
    })
}

#[inline(always)]
fn resolve_lasm_postgres_persist_queue_capacity() -> usize {
    *LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_RESOLVED.get_or_init(|| {
        let Ok(raw) = env::var(LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_ENV) else {
            return resolve_lasm_postgres_persist_queue_capacity_auto_default();
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return resolve_lasm_postgres_persist_queue_capacity_auto_default();
        }
        let Ok(parsed) = trimmed.parse::<usize>() else {
            return resolve_lasm_postgres_persist_queue_capacity_auto_default();
        };
        parsed.clamp(
            LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_MIN,
            LASM_POSTGRES_PERSIST_QUEUE_CAPACITY_MAX,
        )
    })
}

#[inline(always)]
fn resolve_lasm_postgres_persist_batch_max() -> usize {
    *LASM_POSTGRES_PERSIST_BATCH_MAX_RESOLVED.get_or_init(|| {
        let Ok(raw) = env::var(LASM_POSTGRES_PERSIST_BATCH_MAX_ENV) else {
            return LASM_POSTGRES_PERSIST_BATCH_MAX_DEFAULT;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return LASM_POSTGRES_PERSIST_BATCH_MAX_DEFAULT;
        }
        let Ok(parsed) = trimmed.parse::<usize>() else {
            return LASM_POSTGRES_PERSIST_BATCH_MAX_DEFAULT;
        };
        parsed.clamp(
            LASM_POSTGRES_PERSIST_BATCH_MAX_MIN,
            LASM_POSTGRES_PERSIST_BATCH_MAX_MAX,
        )
    })
}

#[inline(always)]
fn resolve_lasm_postgres_persist_queue_full_mode() -> LasmPostgresPersistQueueFullMode {
    *LASM_POSTGRES_PERSIST_QUEUE_FULL_MODE_RESOLVED.get_or_init(|| {
        let Ok(raw) = env::var(LASM_POSTGRES_PERSIST_QUEUE_FULL_MODE_ENV) else {
            return LasmPostgresPersistQueueFullMode::Block;
        };
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "sync-fallback" | "sync_fallback" | "sync" | "fallback" => {
                LasmPostgresPersistQueueFullMode::SyncFallback
            }
            _ => LasmPostgresPersistQueueFullMode::Block,
        }
    })
}

pub(crate) fn lasm_postgres_persist_workers_configured() -> usize {
    resolve_lasm_postgres_persist_workers()
}

pub(crate) fn lasm_postgres_persist_queue_capacity_configured() -> usize {
    resolve_lasm_postgres_persist_queue_capacity()
}

pub(crate) fn lasm_postgres_persist_batch_max_configured() -> usize {
    resolve_lasm_postgres_persist_batch_max()
}

pub(crate) fn lasm_postgres_persist_queue_full_mode() -> &'static str {
    lasm_postgres_persist_queue_full_mode_label(resolve_lasm_postgres_persist_queue_full_mode())
}

pub(crate) fn lasm_postgres_persist_workers_available() -> bool {
    LASM_POSTGRES_PERSIST_WORKERS_AVAILABLE.load(Ordering::Relaxed)
}

pub(crate) fn lasm_postgres_persist_queue_backpressure_total() -> usize {
    LASM_POSTGRES_PERSIST_QUEUE_BACKPRESSURE_TOTAL.load(Ordering::Relaxed)
}

pub(crate) fn lasm_postgres_persist_sync_fallback_total() -> usize {
    LASM_POSTGRES_PERSIST_SYNC_FALLBACK_TOTAL.load(Ordering::Relaxed)
}

pub(crate) fn lasm_postgres_persist_queue_depth() -> usize {
    LASM_POSTGRES_PERSIST_QUEUE
        .get()
        .map(Sender::len)
        .unwrap_or(0)
}

#[derive(Clone)]
struct LasmPostgresPersistTask {
    config: LasmPostgresThreadLocalConfig,
    record: LasmDbRecord,
    compaction_snapshot: Option<Vec<LasmDbRecord>>,
}

fn lasm_postgres_persist_config_key(config: &LasmPostgresThreadLocalConfig) -> String {
    let tls_mode = match config.tls_mode {
        crate::lasm_db_adapter_state::LasmDbPostgresTlsMode::Auto => "auto",
        crate::lasm_db_adapter_state::LasmDbPostgresTlsMode::Disable => "disable",
        crate::lasm_db_adapter_state::LasmDbPostgresTlsMode::Require => "require",
    };
    format!(
        "{tls_mode}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}",
        config.dsn, config.statement_timeout_ms, config.lock_timeout_ms, config.connect_timeout_ms
    )
}

fn lasm_postgres_persist_config_lock(key: &str) -> Arc<Mutex<()>> {
    let locks = LASM_POSTGRES_PERSIST_CONFIG_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = match locks.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!(
                "warning: LASM dynamic postgres persist config lock registry poisoned; continuing with recovered state"
            );
            poisoned.into_inner()
        }
    };
    guard
        .entry(key.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

fn run_lasm_postgres_persist_task_batch(tasks: Vec<LasmPostgresPersistTask>) {
    if tasks.is_empty() {
        return;
    }
    let mut grouped: BTreeMap<
        String,
        (
            LasmPostgresThreadLocalConfig,
            Vec<LasmDbRecord>,
            Option<Vec<LasmDbRecord>>,
        ),
    > = BTreeMap::new();
    for task in tasks {
        let LasmPostgresPersistTask {
            config,
            record,
            compaction_snapshot,
        } = task;
        let key = lasm_postgres_persist_config_key(&config);
        let entry = grouped
            .entry(key)
            .or_insert_with(|| (config.clone(), Vec::new(), None));
        entry.1.push(record);
        if compaction_snapshot.is_some() {
            entry.2 = compaction_snapshot;
        }
    }
    for (_, (config, records, compaction_snapshot)) in grouped {
        let config_key = lasm_postgres_persist_config_key(&config);
        let config_lock = lasm_postgres_persist_config_lock(config_key.as_str());
        let _config_guard = match config_lock.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!(
                    "warning: LASM dynamic postgres persist per-config lock poisoned; continuing with recovered state"
                );
                poisoned.into_inner()
            }
        };
        let mut append_records = records;
        let mut full_sync_failed = false;
        if let Some(snapshot) = compaction_snapshot.as_deref() {
            match persist_lasm_postgres_records_full_sync_thread_local(&config, snapshot) {
                Ok(()) => {
                    if let Some(snapshot_max_id) = snapshot.iter().map(|record| record.id).max() {
                        append_records.retain(|record| record.id > snapshot_max_id);
                    } else {
                        append_records.clear();
                    }
                }
                Err(message) => {
                    full_sync_failed = true;
                    eprintln!(
                        "warning: LASM dynamic postgres records compaction full sync failed: {message}"
                    );
                }
            }
        }
        if append_records.is_empty() {
            continue;
        }
        if append_records.len() == 1 {
            if let Err(message) =
                persist_lasm_postgres_record_append_thread_local(&config, &append_records[0])
            {
                eprintln!(
                    "warning: LASM dynamic postgres records append persistence failed: {message}"
                );
            }
            continue;
        }
        if let Err(message) = persist_lasm_postgres_record_append_batch_thread_local(
            &config,
            append_records.as_slice(),
        ) {
            let message = if full_sync_failed {
                format!("{message}; append fallback also failed after compaction sync failure")
            } else {
                message
            };
            eprintln!(
                "warning: LASM dynamic postgres records batch append persistence failed: {message}"
            );
        }
    }
}

fn lasm_postgres_persist_queue_sender() -> &'static Sender<LasmPostgresPersistTask> {
    LASM_POSTGRES_PERSIST_QUEUE.get_or_init(|| {
        let worker_target = resolve_lasm_postgres_persist_workers();
        let queue_capacity = resolve_lasm_postgres_persist_queue_capacity();
        let batch_max = resolve_lasm_postgres_persist_batch_max();
        let (sender, receiver) = bounded::<LasmPostgresPersistTask>(queue_capacity.max(1));
        let mut started_workers = 0usize;
        for worker_index in 0..worker_target.max(1) {
            let worker_receiver = receiver.clone();
            match thread::Builder::new()
                .name(format!("sec4-lasm-postgres-persist-{worker_index}"))
                .spawn(move || {
                    while let Ok(task) = worker_receiver.recv() {
                        let mut batch = Vec::with_capacity(batch_max);
                        batch.push(task);
                        for _ in 1..batch_max {
                            match worker_receiver.try_recv() {
                                Ok(next) => batch.push(next),
                                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => {
                                    break;
                                }
                            }
                        }
                        run_lasm_postgres_persist_task_batch(batch);
                    }
                }) {
                Ok(_) => {
                    started_workers = started_workers.saturating_add(1);
                }
                Err(err) => {
                    eprintln!(
                        "warning: LASM dynamic postgres persist worker failed to start: {err}"
                    );
                }
            }
        }
        LASM_POSTGRES_PERSIST_WORKERS_AVAILABLE.store(started_workers > 0, Ordering::Relaxed);
        if started_workers == 0 {
            eprintln!(
                "warning: LASM dynamic postgres persist workers unavailable; \
                 request threads will use synchronous persistence fallback"
            );
        } else if started_workers < worker_target {
            eprintln!(
                "warning: LASM dynamic postgres persist started {started_workers}/{worker_target} workers \
                 (queue capacity={queue_capacity})"
            );
        }
        sender
    })
}

pub(crate) fn persist_lasm_postgres_record_after_unlock(
    config: &LasmPostgresThreadLocalConfig,
    record: &LasmDbRecord,
    compaction_snapshot: Option<Vec<LasmDbRecord>>,
) {
    let sender = lasm_postgres_persist_queue_sender();
    if !LASM_POSTGRES_PERSIST_WORKERS_AVAILABLE.load(Ordering::Relaxed) {
        LASM_POSTGRES_PERSIST_SYNC_FALLBACK_TOTAL.fetch_add(1, Ordering::Relaxed);
        run_lasm_postgres_persist_task_batch(vec![LasmPostgresPersistTask {
            config: config.clone(),
            record: record.clone(),
            compaction_snapshot,
        }]);
        return;
    }
    let task = LasmPostgresPersistTask {
        config: config.clone(),
        record: record.clone(),
        compaction_snapshot,
    };
    match sender.try_send(task) {
        Ok(()) => {}
        Err(TrySendError::Full(full_task)) => {
            let backpressure_total =
                LASM_POSTGRES_PERSIST_QUEUE_BACKPRESSURE_TOTAL.fetch_add(1, Ordering::Relaxed) + 1;
            let full_mode = resolve_lasm_postgres_persist_queue_full_mode();
            if backpressure_total == 1 || backpressure_total.is_multiple_of(65_536) {
                match full_mode {
                    LasmPostgresPersistQueueFullMode::Block => eprintln!(
                        "warning: LASM dynamic postgres persist queue saturated; applying backpressure (total events={backpressure_total})"
                    ),
                    LasmPostgresPersistQueueFullMode::SyncFallback => eprintln!(
                        "warning: LASM dynamic postgres persist queue saturated; using synchronous fallback (total events={backpressure_total})"
                    ),
                }
            }
            match full_mode {
                LasmPostgresPersistQueueFullMode::Block => match sender.send(full_task) {
                    Ok(()) => {}
                    Err(SendError(disconnected_task)) => {
                        LASM_POSTGRES_PERSIST_WORKERS_AVAILABLE.store(false, Ordering::Relaxed);
                        LASM_POSTGRES_PERSIST_SYNC_FALLBACK_TOTAL.fetch_add(1, Ordering::Relaxed);
                        run_lasm_postgres_persist_task_batch(vec![disconnected_task]);
                    }
                },
                LasmPostgresPersistQueueFullMode::SyncFallback => {
                    LASM_POSTGRES_PERSIST_SYNC_FALLBACK_TOTAL.fetch_add(1, Ordering::Relaxed);
                    run_lasm_postgres_persist_task_batch(vec![full_task]);
                }
            }
        }
        Err(TrySendError::Disconnected(disconnected_task)) => {
            LASM_POSTGRES_PERSIST_WORKERS_AVAILABLE.store(false, Ordering::Relaxed);
            LASM_POSTGRES_PERSIST_SYNC_FALLBACK_TOTAL.fetch_add(1, Ordering::Relaxed);
            run_lasm_postgres_persist_task_batch(vec![disconnected_task]);
        }
    }
}
