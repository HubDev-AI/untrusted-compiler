use crate::lasm_db_adapter_state::lasm_db_postgres_tls_mode_label;
use crate::lasm_db_config::lasm_db_records_adapter_label;
use crate::{
    lasm_db_record_to_json, lasm_error_envelope, set_lasm_json_response, LasmDynamicResponseState,
    LasmRunRequest,
};
use std::sync::Mutex;

pub(crate) fn apply_lasm_db_list_records_response_materialization(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    trace_id: &str,
) {
    let records_limit = if let Some(raw_limit) = request.query_params.get("limit") {
        let trimmed = raw_limit.trim();
        match trimmed.parse::<usize>() {
            Ok(value) if value >= 1 => Some(value),
            _ => {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_LIMIT_INVALID",
                        "validation",
                        "db records limit must be an integer >= 1",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
        }
    } else {
        None
    };
    let records_op_filter = if let Some(raw_op) = request.query_params.get("op") {
        let trimmed = raw_op.trim();
        if trimmed.is_empty() {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.RECORDS_FILTER_INVALID",
                    "validation",
                    "db records op filter must be a non-empty string",
                    400,
                    trace_id,
                ),
            );
            return;
        }
        if !matches!(trimmed, "exec" | "execTx" | "queryOne") {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.RECORDS_FILTER_INVALID",
                    "validation",
                    "db records op filter must be one of exec, execTx, queryOne",
                    400,
                    trace_id,
                ),
            );
            return;
        }
        Some(trimmed.to_string())
    } else {
        None
    };
    let records_db_filter = if let Some(raw_db) = request.query_params.get("db") {
        let trimmed = raw_db.trim();
        match trimmed.parse::<i64>() {
            Ok(value) if value >= 1 => Some(value),
            _ => {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_FILTER_INVALID",
                        "validation",
                        "db records db filter must be an integer >= 1",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
        }
    } else {
        None
    };
    let records_tx_filter = if let Some(raw_tx) = request.query_params.get("tx") {
        let trimmed = raw_tx.trim();
        match trimmed.parse::<i64>() {
            Ok(value) if value >= 0 => Some(value),
            _ => {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_FILTER_INVALID",
                        "validation",
                        "db records tx filter must be an integer >= 0",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
        }
    } else {
        None
    };
    let records_template_contains_filter =
        if let Some(raw_template_contains) = request.query_params.get("templateContains") {
            let trimmed = raw_template_contains.trim();
            if trimmed.is_empty() {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_FILTER_INVALID",
                        "validation",
                        "db records templateContains filter must be a non-empty string",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
            Some(trimmed.to_string())
        } else {
            None
        };
    let (
        records,
        records_total,
        records_global_total,
        records_capacity,
        records_dropped_total,
        affected_rows_total,
        affected_rows_filtered_total,
        affected_rows_global_total,
        records_exec_count,
        records_exec_tx_count,
        records_query_one_count,
        records_exec_global_count,
        records_exec_tx_global_count,
        records_query_one_global_count,
        adapter,
        tx_handle_count,
        tx_handle_capacity,
        postgres_statement_cache_count,
        postgres_statement_cache_capacity,
        postgres_statement_cache_evictions_total,
        postgres_placeholder_cache_count,
        postgres_placeholder_cache_capacity,
        postgres_placeholder_cache_evictions_total,
        postgres_statement_timeout_ms,
        postgres_lock_timeout_ms,
        postgres_connect_timeout_ms,
        postgres_tls_mode,
        sqlite_busy_timeout_ms,
    ) = match dynamic_state.lock() {
        Ok(state) => {
            let records_filtered = state
                .db_records
                .iter()
                .filter(|record| {
                    records_op_filter
                        .as_ref()
                        .map(|op| record.op == *op)
                        .unwrap_or(true)
                        && records_db_filter.map(|db| record.db == db).unwrap_or(true)
                        && records_tx_filter.map(|tx| record.tx == tx).unwrap_or(true)
                        && records_template_contains_filter
                            .as_ref()
                            .map(|needle| record.template.contains(needle))
                            .unwrap_or(true)
                })
                .cloned()
                .collect::<Vec<_>>();
            let (records_exec_count, records_exec_tx_count, records_query_one_count) =
                records_filtered.iter().fold(
                    (0usize, 0usize, 0usize),
                    |(exec_count, exec_tx_count, query_one_count), record| match record.op.as_str()
                    {
                        "exec" => (exec_count + 1, exec_tx_count, query_one_count),
                        "execTx" => (exec_count, exec_tx_count + 1, query_one_count),
                        "queryOne" => (exec_count, exec_tx_count, query_one_count + 1),
                        _ => (exec_count, exec_tx_count, query_one_count),
                    },
                );
            let (
                records_exec_global_count,
                records_exec_tx_global_count,
                records_query_one_global_count,
            ) = state.db_records.iter().fold(
                (0usize, 0usize, 0usize),
                |(exec_count, exec_tx_count, query_one_count), record| match record.op.as_str() {
                    "exec" => (exec_count + 1, exec_tx_count, query_one_count),
                    "execTx" => (exec_count, exec_tx_count + 1, query_one_count),
                    "queryOne" => (exec_count, exec_tx_count, query_one_count + 1),
                    _ => (exec_count, exec_tx_count, query_one_count),
                },
            );
            let affected_rows_filtered_total = records_filtered
                .iter()
                .fold(0u64, |acc, record| acc.saturating_add(record.affected_rows));
            let affected_rows_global_total = state
                .db_records
                .iter()
                .fold(0u64, |acc, record| acc.saturating_add(record.affected_rows));
            let total = records_filtered.len();
            let records = if let Some(limit) = records_limit {
                records_filtered
                    .iter()
                    .skip(total.saturating_sub(limit))
                    .cloned()
                    .collect::<Vec<_>>()
            } else {
                records_filtered
            };
            let affected_rows_total = records
                .iter()
                .fold(0u64, |acc, record| acc.saturating_add(record.affected_rows));
            (
                records,
                total,
                state.db_records.len(),
                state.db_records_max,
                state.db_records_dropped_total,
                affected_rows_total,
                affected_rows_filtered_total,
                affected_rows_global_total,
                records_exec_count,
                records_exec_tx_count,
                records_query_one_count,
                records_exec_global_count,
                records_exec_tx_global_count,
                records_query_one_global_count,
                lasm_db_records_adapter_label(state.db_records_adapter),
                state.db_tx_handles.len(),
                state.db_tx_max_handles,
                state.db_records_postgres_statement_cache.len(),
                state.db_postgres_statement_cache_max,
                state.db_postgres_statement_cache_evictions_total,
                state.db_postgres_placeholder_max_cache.len(),
                state.db_postgres_placeholder_cache_max,
                state.db_postgres_placeholder_cache_evictions_total,
                state.db_postgres_statement_timeout_ms,
                state.db_postgres_lock_timeout_ms,
                state.db_postgres_connect_timeout_ms,
                lasm_db_postgres_tls_mode_label(state.db_postgres_tls_mode),
                state.db_sqlite_busy_timeout_ms,
            )
        }
        Err(_) => {
            set_lasm_json_response(
                response,
                500,
                &lasm_error_envelope(
                    "HTTP.INTERNAL",
                    "internal",
                    "dynamic response state unavailable",
                    500,
                    trace_id,
                ),
            );
            return;
        }
    };
    set_lasm_json_response(
        response,
        200,
        &serde_json::json!({
            "ok": true,
            "count": records.len(),
            "recordsTotal": records_total,
            "recordsGlobalTotal": records_global_total,
            "recordsCapacity": records_capacity,
            "recordsDroppedTotal": records_dropped_total,
            "affectedRowsTotal": affected_rows_total,
            "affectedRowsFilteredTotal": affected_rows_filtered_total,
            "affectedRowsGlobalTotal": affected_rows_global_total,
            "adapter": adapter,
            "filters": {
                "op": records_op_filter,
                "db": records_db_filter,
                "tx": records_tx_filter,
                "templateContains": records_template_contains_filter,
            },
            "opCounts": {
                "exec": records_exec_count,
                "execTx": records_exec_tx_count,
                "queryOne": records_query_one_count,
            },
            "opCountsGlobal": {
                "exec": records_exec_global_count,
                "execTx": records_exec_tx_global_count,
                "queryOne": records_query_one_global_count,
            },
            "txHandleCount": tx_handle_count,
            "txHandleCapacity": tx_handle_capacity,
            "dbCache": {
                "postgresStatementCount": postgres_statement_cache_count,
                "postgresStatementCapacity": postgres_statement_cache_capacity,
                "postgresStatementEvictedTotal": postgres_statement_cache_evictions_total,
                "postgresPlaceholderCount": postgres_placeholder_cache_count,
                "postgresPlaceholderCapacity": postgres_placeholder_cache_capacity,
                "postgresPlaceholderEvictedTotal": postgres_placeholder_cache_evictions_total,
            },
            "dbTimeoutsMs": {
                "postgresStatement": postgres_statement_timeout_ms,
                "postgresLock": postgres_lock_timeout_ms,
                "postgresConnect": postgres_connect_timeout_ms,
                "postgresTlsMode": postgres_tls_mode,
                "sqliteBusy": sqlite_busy_timeout_ms,
            },
            "records": records.iter().map(lasm_db_record_to_json).collect::<Vec<_>>(),
        }),
    );
}
