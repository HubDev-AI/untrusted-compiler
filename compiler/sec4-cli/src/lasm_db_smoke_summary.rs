use crate::lasm_db_adapter_state::lasm_db_postgres_tls_mode_label;
use crate::lasm_db_config::lasm_db_records_adapter_label;
use crate::lasm_db_runtime_dispatch;
use crate::lasm_db_runtime_postgres;
use crate::lasm_db_runtime_postgres_persist;
use crate::lasm_dynamic_state::LasmDynamicResponseState;
use serde_json::Value;

#[derive(Debug, Clone)]
pub(crate) struct LasmSmokeDbSummary {
    pub(crate) adapter_label: String,
    pub(crate) records_max: usize,
    pub(crate) tx_max_handles: usize,
    pub(crate) op_sequence_max: usize,
    pub(crate) store_path: Option<String>,
    pub(crate) sqlite_store_path: Option<String>,
    pub(crate) postgres_dsn_configured: bool,
    pub(crate) postgres_tls_mode: String,
    pub(crate) postgres_statement_timeout_ms: u64,
    pub(crate) postgres_lock_timeout_ms: u64,
    pub(crate) postgres_connect_timeout_ms: u64,
    pub(crate) postgres_retryable_conflict_retry_max: usize,
    pub(crate) postgres_statement_cache_max: usize,
    pub(crate) postgres_placeholder_cache_max: usize,
    pub(crate) sqlite_busy_timeout_ms: u64,
    pub(crate) sqlite_lock_retry_max: usize,
    pub(crate) sqlite_lock_retry_delay_ms: u64,
    pub(crate) sqlite_journal_mode: String,
    pub(crate) sqlite_synchronous: String,
    pub(crate) postgres_shared_client_pool_keys: usize,
    pub(crate) postgres_shared_client_pool_idle_total: usize,
    pub(crate) postgres_shared_client_pool_active_keys: usize,
    pub(crate) postgres_shared_client_pool_active_total: usize,
    pub(crate) postgres_shared_client_max_idle_per_key: usize,
    pub(crate) postgres_shared_client_max_total_idle: usize,
    pub(crate) postgres_shared_client_max_active_per_key: usize,
    pub(crate) postgres_shared_client_max_active_total: usize,
    pub(crate) postgres_persist_workers: usize,
    pub(crate) postgres_persist_queue_capacity: usize,
    pub(crate) postgres_persist_batch_max: usize,
    pub(crate) postgres_persist_queue_full_mode: String,
    pub(crate) postgres_persist_workers_available: bool,
    pub(crate) postgres_persist_queue_depth: usize,
    pub(crate) postgres_persist_queue_backpressure_total: usize,
    pub(crate) postgres_persist_sync_fallback_total: usize,
}

pub(crate) fn build_lasm_smoke_db_summary(state: &LasmDynamicResponseState) -> LasmSmokeDbSummary {
    LasmSmokeDbSummary {
        adapter_label: lasm_db_records_adapter_label(state.db_records_adapter).to_string(),
        records_max: state.db_records_max,
        tx_max_handles: state.db_tx_max_handles,
        op_sequence_max: lasm_db_runtime_dispatch::lasm_db_op_sequence_max_limit(),
        store_path: state
            .db_records_store_path
            .as_ref()
            .map(|path| path.display().to_string()),
        sqlite_store_path: state
            .db_records_sqlite_store_path
            .as_ref()
            .map(|path| path.display().to_string()),
        postgres_dsn_configured: state.db_records_postgres_dsn.is_some(),
        postgres_tls_mode: lasm_db_postgres_tls_mode_label(state.db_postgres_tls_mode).to_string(),
        postgres_statement_timeout_ms: state.db_postgres_statement_timeout_ms,
        postgres_lock_timeout_ms: state.db_postgres_lock_timeout_ms,
        postgres_connect_timeout_ms: state.db_postgres_connect_timeout_ms,
        postgres_retryable_conflict_retry_max: state.db_postgres_retryable_conflict_retry_max,
        postgres_statement_cache_max: state.db_postgres_statement_cache_max,
        postgres_placeholder_cache_max: state.db_postgres_placeholder_cache_max,
        sqlite_busy_timeout_ms: state.db_sqlite_busy_timeout_ms,
        sqlite_lock_retry_max: state.db_sqlite_lock_retry_max,
        sqlite_lock_retry_delay_ms: state.db_sqlite_lock_retry_delay_ms,
        sqlite_journal_mode: state.db_sqlite_journal_mode.clone(),
        sqlite_synchronous: state.db_sqlite_synchronous.clone(),
        postgres_shared_client_pool_keys:
            lasm_db_runtime_postgres::lasm_postgres_shared_client_pool_key_count(),
        postgres_shared_client_pool_idle_total:
            lasm_db_runtime_postgres::lasm_postgres_shared_client_pool_idle_total(),
        postgres_shared_client_pool_active_keys:
            lasm_db_runtime_postgres::lasm_postgres_shared_client_pool_active_key_count(),
        postgres_shared_client_pool_active_total:
            lasm_db_runtime_postgres::lasm_postgres_shared_client_pool_active_total(),
        postgres_shared_client_max_idle_per_key:
            lasm_db_runtime_postgres::lasm_postgres_shared_client_max_idle_per_key(),
        postgres_shared_client_max_total_idle:
            lasm_db_runtime_postgres::lasm_postgres_shared_client_max_total_idle(),
        postgres_shared_client_max_active_per_key:
            lasm_db_runtime_postgres::lasm_postgres_shared_client_max_active_per_key(),
        postgres_shared_client_max_active_total:
            lasm_db_runtime_postgres::lasm_postgres_shared_client_max_active_total(),
        postgres_persist_workers:
            lasm_db_runtime_postgres_persist::lasm_postgres_persist_workers_configured(),
        postgres_persist_queue_capacity:
            lasm_db_runtime_postgres_persist::lasm_postgres_persist_queue_capacity_configured(),
        postgres_persist_batch_max:
            lasm_db_runtime_postgres_persist::lasm_postgres_persist_batch_max_configured(),
        postgres_persist_queue_full_mode:
            lasm_db_runtime_postgres_persist::lasm_postgres_persist_queue_full_mode().to_string(),
        postgres_persist_workers_available:
            lasm_db_runtime_postgres_persist::lasm_postgres_persist_workers_available(),
        postgres_persist_queue_depth:
            lasm_db_runtime_postgres_persist::lasm_postgres_persist_queue_depth(),
        postgres_persist_queue_backpressure_total:
            lasm_db_runtime_postgres_persist::lasm_postgres_persist_queue_backpressure_total(),
        postgres_persist_sync_fallback_total:
            lasm_db_runtime_postgres_persist::lasm_postgres_persist_sync_fallback_total(),
    }
}

impl LasmSmokeDbSummary {
    pub(crate) fn as_text_store_path(&self) -> &str {
        self.store_path.as_deref().unwrap_or("-")
    }

    pub(crate) fn as_text_sqlite_store_path(&self) -> &str {
        self.sqlite_store_path.as_deref().unwrap_or("-")
    }

    pub(crate) fn into_json_payload(self) -> Value {
        serde_json::json!({
            "adapter": self.adapter_label,
            "recordsMax": self.records_max,
            "txMaxHandles": self.tx_max_handles,
            "opSequenceMax": self.op_sequence_max,
            "storePath": self.store_path,
            "sqliteStorePath": self.sqlite_store_path,
            "postgres": {
                "dsnConfigured": self.postgres_dsn_configured,
                "tlsMode": self.postgres_tls_mode,
                "statementTimeoutMs": self.postgres_statement_timeout_ms,
                "lockTimeoutMs": self.postgres_lock_timeout_ms,
                "connectTimeoutMs": self.postgres_connect_timeout_ms,
                "retryableConflictRetryMax": self.postgres_retryable_conflict_retry_max,
                "statementCacheMax": self.postgres_statement_cache_max,
                "placeholderCacheMax": self.postgres_placeholder_cache_max,
                "sharedClient": {
                    "poolKeys": self.postgres_shared_client_pool_keys,
                    "poolIdleTotal": self.postgres_shared_client_pool_idle_total,
                    "poolActiveKeys": self.postgres_shared_client_pool_active_keys,
                    "poolActiveTotal": self.postgres_shared_client_pool_active_total,
                    "maxIdlePerKey": self.postgres_shared_client_max_idle_per_key,
                    "maxTotalIdle": self.postgres_shared_client_max_total_idle,
                    "maxActivePerKey": self.postgres_shared_client_max_active_per_key,
                    "maxActiveTotal": self.postgres_shared_client_max_active_total
                },
                "persist": {
                    "workers": self.postgres_persist_workers,
                    "queueCapacity": self.postgres_persist_queue_capacity,
                    "batchMax": self.postgres_persist_batch_max,
                    "queueFullMode": self.postgres_persist_queue_full_mode,
                    "workersAvailable": self.postgres_persist_workers_available,
                    "queueDepth": self.postgres_persist_queue_depth,
                    "queueBackpressureTotal": self.postgres_persist_queue_backpressure_total,
                    "syncFallbackTotal": self.postgres_persist_sync_fallback_total
                }
            },
            "sqlite": {
                "busyTimeoutMs": self.sqlite_busy_timeout_ms,
                "lockRetryMax": self.sqlite_lock_retry_max,
                "lockRetryDelayMs": self.sqlite_lock_retry_delay_ms,
                "journalMode": self.sqlite_journal_mode,
                "synchronous": self.sqlite_synchronous
            }
        })
    }
}
