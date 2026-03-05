use crate::{LasmDbRecord, LasmDynamicResponseState};

pub(crate) fn find_lasm_records_log_latest_match(
    state: &LasmDynamicResponseState,
    db: i64,
    template: &str,
    params: &str,
) -> Option<LasmDbRecord> {
    let signature = crate::lasm_db_record_signature_key(db, template, params);
    if !state.db_record_signatures.contains_key(signature.as_str()) {
        return None;
    }
    state
        .db_latest_record_by_signature
        .get(signature.as_str())
        .cloned()
        .or_else(|| {
            state
                .db_records
                .iter()
                .rev()
                .find(|candidate| {
                    crate::lasm_db_record_signature_key(
                        candidate.db,
                        &candidate.template,
                        &candidate.params,
                    ) == signature
                })
                .cloned()
        })
}

pub(crate) fn build_lasm_records_log_query_one_row_object(
    record: &LasmDbRecord,
    row_schema: i64,
) -> serde_json::Value {
    serde_json::json!({
        "op": record.op,
        "db": record.db,
        "template": record.template,
        "params": record.params,
        "tx": record.tx,
        "affected_rows": record.affected_rows,
        "rowSchema": row_schema,
    })
}
