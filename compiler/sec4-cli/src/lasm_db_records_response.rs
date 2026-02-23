use crate::lasm_db_adapter_state::lasm_db_postgres_tls_mode_label;
use crate::lasm_db_config::lasm_db_records_adapter_label;
use crate::{
    lasm_db_record_to_json, lasm_error_envelope, set_lasm_json_response, LasmDynamicResponseState,
    LasmRunRequest,
};
use std::{collections::BTreeSet, sync::Mutex};

const LASM_DB_RECORDS_LIMIT_MAX: usize = 1000;

pub(crate) fn apply_lasm_db_list_records_response_materialization(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    trace_id: &str,
) {
    let records_limit = if let Some(raw_limit) = request.query_params.get("limit") {
        let trimmed = raw_limit.trim();
        match trimmed.parse::<usize>() {
            Ok(value) if (1..=LASM_DB_RECORDS_LIMIT_MAX).contains(&value) => Some(value),
            _ => {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_LIMIT_INVALID",
                        "validation",
                        "db records limit must be an integer between 1 and 1000",
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
    let records_offset = if let Some(raw_offset) = request.query_params.get("offset") {
        let trimmed = raw_offset.trim();
        match trimmed.parse::<usize>() {
            Ok(value) => Some(value),
            _ => {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_FILTER_INVALID",
                        "validation",
                        "db records offset filter must be an integer >= 0",
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
    let records_ops_filter = if let Some(raw_ops) = request.query_params.get("ops") {
        let mut values = BTreeSet::new();
        for raw_entry in raw_ops.split(',') {
            let trimmed = raw_entry.trim();
            if trimmed.is_empty() || !matches!(trimmed, "exec" | "execTx" | "queryOne") {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_FILTER_INVALID",
                        "validation",
                        "db records ops filter must be comma-separated values from exec, execTx, queryOne",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
            values.insert(trimmed.to_string());
        }
        if values.is_empty() {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.RECORDS_FILTER_INVALID",
                    "validation",
                    "db records ops filter must be comma-separated values from exec, execTx, queryOne",
                    400,
                    trace_id,
                ),
            );
            return;
        }
        Some(values.into_iter().collect::<Vec<_>>())
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
    let records_params_contains_filter =
        if let Some(raw_params_contains) = request.query_params.get("paramsContains") {
            let trimmed = raw_params_contains.trim();
            if trimmed.is_empty() {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_FILTER_INVALID",
                        "validation",
                        "db records paramsContains filter must be a non-empty string",
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
    let records_created_from_ms_filter =
        if let Some(raw_created_from_ms) = request.query_params.get("createdFromMs") {
            let trimmed = raw_created_from_ms.trim();
            match trimmed.parse::<u64>() {
                Ok(value) => Some(value),
                _ => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.RECORDS_FILTER_INVALID",
                            "validation",
                            "db records createdFromMs filter must be an integer >= 0",
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
    let records_created_to_ms_filter =
        if let Some(raw_created_to_ms) = request.query_params.get("createdToMs") {
            let trimmed = raw_created_to_ms.trim();
            match trimmed.parse::<u64>() {
                Ok(value) => Some(value),
                _ => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.RECORDS_FILTER_INVALID",
                            "validation",
                            "db records createdToMs filter must be an integer >= 0",
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
    if let (Some(created_from_ms), Some(created_to_ms)) =
        (records_created_from_ms_filter, records_created_to_ms_filter)
    {
        if created_from_ms > created_to_ms {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.RECORDS_FILTER_INVALID",
                    "validation",
                    "db records createdFromMs filter must be <= createdToMs",
                    400,
                    trace_id,
                ),
            );
            return;
        }
    }
    let records_id_from_filter = if let Some(raw_id_from) = request.query_params.get("idFrom") {
        let trimmed = raw_id_from.trim();
        match trimmed.parse::<u64>() {
            Ok(value) if value >= 1 => Some(value),
            _ => {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_FILTER_INVALID",
                        "validation",
                        "db records idFrom filter must be an integer >= 1",
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
    let records_id_filter = if let Some(raw_id) = request.query_params.get("id") {
        let trimmed = raw_id.trim();
        match trimmed.parse::<u64>() {
            Ok(value) if value >= 1 => Some(value),
            _ => {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_FILTER_INVALID",
                        "validation",
                        "db records id filter must be an integer >= 1",
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
    let records_id_to_filter = if let Some(raw_id_to) = request.query_params.get("idTo") {
        let trimmed = raw_id_to.trim();
        match trimmed.parse::<u64>() {
            Ok(value) if value >= 1 => Some(value),
            _ => {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.RECORDS_FILTER_INVALID",
                        "validation",
                        "db records idTo filter must be an integer >= 1",
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
    if let (Some(id_from), Some(id_to)) = (records_id_from_filter, records_id_to_filter) {
        if id_from > id_to {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.RECORDS_FILTER_INVALID",
                    "validation",
                    "db records idFrom filter must be <= idTo",
                    400,
                    trace_id,
                ),
            );
            return;
        }
    }
    let records_order_filter = if let Some(raw_order) = request.query_params.get("order") {
        let normalized = raw_order.trim().to_ascii_lowercase();
        if normalized == "asc" || normalized == "desc" {
            normalized
        } else {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.RECORDS_FILTER_INVALID",
                    "validation",
                    "db records order filter must be one of asc or desc",
                    400,
                    trace_id,
                ),
            );
            return;
        }
    } else {
        "asc".to_string()
    };
    let include_records =
        if let Some(raw_include_records) = request.query_params.get("includeRecords") {
            let normalized = raw_include_records.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "true" | "1" => true,
                "false" | "0" => false,
                _ => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.RECORDS_FILTER_INVALID",
                            "validation",
                            "db records includeRecords filter must be one of true, false, 1, 0",
                            400,
                            trace_id,
                        ),
                    );
                    return;
                }
            }
        } else {
            true
        };
    let records_affected_rows_min_filter =
        if let Some(raw_affected_rows_min) = request.query_params.get("affectedRowsMin") {
            let trimmed = raw_affected_rows_min.trim();
            match trimmed.parse::<u64>() {
                Ok(value) => Some(value),
                _ => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.RECORDS_FILTER_INVALID",
                            "validation",
                            "db records affectedRowsMin filter must be an integer >= 0",
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
    let records_affected_rows_max_filter =
        if let Some(raw_affected_rows_max) = request.query_params.get("affectedRowsMax") {
            let trimmed = raw_affected_rows_max.trim();
            match trimmed.parse::<u64>() {
                Ok(value) => Some(value),
                _ => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.RECORDS_FILTER_INVALID",
                            "validation",
                            "db records affectedRowsMax filter must be an integer >= 0",
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
    if let (Some(affected_rows_min), Some(affected_rows_max)) = (
        records_affected_rows_min_filter,
        records_affected_rows_max_filter,
    ) {
        if affected_rows_min > affected_rows_max {
            set_lasm_json_response(
                response,
                400,
                &lasm_error_envelope(
                    "DB.RECORDS_FILTER_INVALID",
                    "validation",
                    "db records affectedRowsMin filter must be <= affectedRowsMax",
                    400,
                    trace_id,
                ),
            );
            return;
        }
    }
    let (
        records,
        records_count,
        records_total,
        records_has_more,
        records_next_offset,
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
        postgres_retryable_conflict_retry_max,
        sqlite_busy_timeout_ms,
        sqlite_lock_retry_max,
        sqlite_lock_retry_delay_ms,
        sqlite_journal_mode,
        sqlite_synchronous,
    ) = match dynamic_state.lock() {
        Ok(state) => {
            let mut filtered_indices = state
                .db_records
                .iter()
                .enumerate()
                .filter(|record| {
                    let (_, record) = record;
                    records_op_filter
                        .as_ref()
                        .map(|op| record.op == *op)
                        .unwrap_or(true)
                        && records_ops_filter
                            .as_ref()
                            .map(|ops| ops.iter().any(|op| record.op == *op))
                            .unwrap_or(true)
                        && records_db_filter.map(|db| record.db == db).unwrap_or(true)
                        && records_tx_filter.map(|tx| record.tx == tx).unwrap_or(true)
                        && records_template_contains_filter
                            .as_ref()
                            .map(|needle| record.template.contains(needle))
                            .unwrap_or(true)
                        && records_params_contains_filter
                            .as_ref()
                            .map(|needle| record.params.contains(needle))
                            .unwrap_or(true)
                        && records_created_from_ms_filter
                            .map(|from_ms| record.created_at_ms >= from_ms)
                            .unwrap_or(true)
                        && records_created_to_ms_filter
                            .map(|to_ms| record.created_at_ms <= to_ms)
                            .unwrap_or(true)
                        && records_id_from_filter
                            .map(|id_from| record.id >= id_from)
                            .unwrap_or(true)
                        && records_id_filter
                            .map(|id| record.id == id)
                            .unwrap_or(true)
                        && records_id_to_filter
                            .map(|id_to| record.id <= id_to)
                            .unwrap_or(true)
                        && records_affected_rows_min_filter
                            .map(|affected_rows_min| record.affected_rows >= affected_rows_min)
                            .unwrap_or(true)
                        && records_affected_rows_max_filter
                            .map(|affected_rows_max| record.affected_rows <= affected_rows_max)
                            .unwrap_or(true)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            let (records_exec_count, records_exec_tx_count, records_query_one_count) =
                filtered_indices.iter().fold(
                    (0usize, 0usize, 0usize),
                    |(exec_count, exec_tx_count, query_one_count), record_index| {
                        let record = &state.db_records[*record_index];
                        match record.op.as_str() {
                            "exec" => (exec_count + 1, exec_tx_count, query_one_count),
                            "execTx" => (exec_count, exec_tx_count + 1, query_one_count),
                            "queryOne" => (exec_count, exec_tx_count, query_one_count + 1),
                            _ => (exec_count, exec_tx_count, query_one_count),
                        }
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
            let affected_rows_filtered_total = filtered_indices
                .iter()
                .fold(0u64, |acc, record_index| {
                    acc.saturating_add(state.db_records[*record_index].affected_rows)
                });
            let affected_rows_global_total = state
                .db_records
                .iter()
                .fold(0u64, |acc, record| acc.saturating_add(record.affected_rows));
            if records_order_filter == "desc" {
                filtered_indices.reverse();
            }
            let total = filtered_indices.len();
            let offset = records_offset.unwrap_or(0);
            let (window_start, window_end) = if records_order_filter == "desc" {
                let start = offset.min(total);
                if let Some(limit) = records_limit {
                    let end = start.saturating_add(limit).min(total);
                    (start, end)
                } else {
                    (start, total)
                }
            } else if let Some(limit) = records_limit {
                let end = total.saturating_sub(offset);
                let start = end.saturating_sub(limit);
                (start, end)
            } else if offset > 0 {
                (offset.min(total), total)
            } else {
                (0, total)
            };
            let window_len = window_end.saturating_sub(window_start);
            let records = if include_records {
                filtered_indices[window_start..window_end]
                    .iter()
                    .map(|record_index| state.db_records[*record_index].clone())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let affected_rows_total = filtered_indices[window_start..window_end]
                .iter()
                .fold(0u64, |acc, record_index| {
                    acc.saturating_add(state.db_records[*record_index].affected_rows)
                });
            let records_has_more = if records_limit.is_some() {
                if records_order_filter == "desc" {
                    window_end < total
                } else {
                    total.saturating_sub(offset).saturating_sub(window_len) > 0
                }
            } else {
                false
            };
            let records_next_offset = if records_has_more {
                Some(offset.saturating_add(window_len))
            } else {
                None
            };
            (
                records,
                window_len,
                total,
                records_has_more,
                records_next_offset,
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
                state.db_postgres_retryable_conflict_retry_max,
                state.db_sqlite_busy_timeout_ms,
                state.db_sqlite_lock_retry_max,
                state.db_sqlite_lock_retry_delay_ms,
                state.db_sqlite_journal_mode.clone(),
                state.db_sqlite_synchronous.clone(),
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
            "count": records_count,
            "recordsTotal": records_total,
            "hasMore": records_has_more,
            "nextOffset": records_next_offset,
            "recordsGlobalTotal": records_global_total,
            "recordsCapacity": records_capacity,
            "recordsDroppedTotal": records_dropped_total,
            "affectedRowsTotal": affected_rows_total,
            "affectedRowsFilteredTotal": affected_rows_filtered_total,
            "affectedRowsGlobalTotal": affected_rows_global_total,
            "adapter": adapter,
            "filters": {
                "op": records_op_filter,
                "ops": records_ops_filter,
                "db": records_db_filter,
                "tx": records_tx_filter,
                "templateContains": records_template_contains_filter,
                "paramsContains": records_params_contains_filter,
                "createdFromMs": records_created_from_ms_filter,
                "createdToMs": records_created_to_ms_filter,
                "idFrom": records_id_from_filter,
                "id": records_id_filter,
                "idTo": records_id_to_filter,
                "limit": records_limit,
                "offset": records_offset,
                "order": records_order_filter,
                "includeRecords": include_records,
                "affectedRowsMin": records_affected_rows_min_filter,
                "affectedRowsMax": records_affected_rows_max_filter,
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
                "postgresRetryableConflictRetryMax": postgres_retryable_conflict_retry_max,
                "sqliteBusy": sqlite_busy_timeout_ms,
                "sqliteLockRetryMax": sqlite_lock_retry_max,
                "sqliteLockRetryDelayMs": sqlite_lock_retry_delay_ms,
                "sqliteJournalMode": sqlite_journal_mode,
                "sqliteSynchronous": sqlite_synchronous,
            },
            "records": if include_records {
                records.iter().map(lasm_db_record_to_json).collect::<Vec<_>>()
            } else {
                Vec::<serde_json::Value>::new()
            },
        }),
    );
}
