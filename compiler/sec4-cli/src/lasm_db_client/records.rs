use super::adapter::{persist_lasm_db_record_append, persist_lasm_db_records_full_sync};
use crate::append_lasm_dynamic_db_record;
use crate::lasm_dynamic_state::compose_lasm_db_record_id;
use crate::{LasmDbRecord, LasmDynamicResponseState};
use std::time::{SystemTime, UNIX_EPOCH};

const LASM_DB_RECORDS_COMPACTION_SYNC_DROPS_INTERVAL: u64 = 1024;

fn lasm_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

pub(crate) fn persist_lasm_db_record_with_capacity_guard(
    state: &mut LasmDynamicResponseState,
    record: &LasmDbRecord,
) {
    let dropped_before = state.db_records_dropped_total;
    let history_overflowed = append_lasm_dynamic_db_record(state, record.clone());
    if let Err(message) = persist_lasm_db_record_append(state, record) {
        eprintln!("warning: LASM dynamic records store persistence failed: {message}");
    }
    if history_overflowed
        && state.db_records_dropped_total != dropped_before
        && state
            .db_records_dropped_total
            .is_multiple_of(LASM_DB_RECORDS_COMPACTION_SYNC_DROPS_INTERVAL)
    {
        if let Err(message) = persist_lasm_db_records_full_sync(state) {
            eprintln!(
                "warning: LASM dynamic records store compaction sync failed after \
                 in-memory overflow: {message}"
            );
        }
    }
}

pub(crate) fn allocate_lasm_db_runtime_record(
    state: &mut LasmDynamicResponseState,
    op: &str,
    db: i64,
    template: &str,
    params: &str,
    tx: i64,
    affected_rows: u64,
) -> LasmDbRecord {
    let record = LasmDbRecord {
        id: compose_lasm_db_record_id(state.db_records_adapter, state.next_db_record_id),
        op: op.to_string(),
        db,
        template: template.to_string(),
        params: params.to_string(),
        tx,
        affected_rows,
        created_at_ms: lasm_now_ms(),
    };
    state.next_db_record_id = state.next_db_record_id.saturating_add(1);
    record
}

pub(crate) fn append_lasm_db_record_in_memory_with_compaction_snapshot(
    state: &mut LasmDynamicResponseState,
    record: LasmDbRecord,
) -> (LasmDbRecord, Option<Vec<LasmDbRecord>>) {
    let dropped_before = state.db_records_dropped_total;
    let history_overflowed = append_lasm_dynamic_db_record(state, record.clone());
    let should_full_sync = history_overflowed
        && state.db_records_dropped_total != dropped_before
        && state
            .db_records_dropped_total
            .is_multiple_of(LASM_DB_RECORDS_COMPACTION_SYNC_DROPS_INTERVAL);
    let compaction_snapshot = if should_full_sync {
        Some(state.db_records.clone())
    } else {
        None
    };
    (record, compaction_snapshot)
}
