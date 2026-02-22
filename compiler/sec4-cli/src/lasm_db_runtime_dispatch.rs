use crate::lasm_db_adapter_state::{
    persist_lasm_dynamic_db_record_append, persist_lasm_dynamic_db_records_full_sync,
};
use crate::lasm_db_runtime_common::{
    allocate_lasm_db_tx_handle, classify_lasm_db_runtime_error, is_lasm_valid_db_cap_handle,
    normalize_lasm_db_params, parse_lasm_positive_i64,
};
use crate::lasm_db_runtime_postgres::{
    parse_lasm_postgres_query_template_and_params, run_lasm_postgres_exec,
    run_lasm_postgres_exec_tx, run_lasm_postgres_query_one,
};
use crate::lasm_db_runtime_sqlite::{
    parse_lasm_sqlite_query_params, run_lasm_sqlite_exec, run_lasm_sqlite_exec_tx,
    run_lasm_sqlite_query_one,
};
use crate::{
    append_lasm_dynamic_db_record, lasm_db_record_to_json, lasm_error_envelope, lasm_now_ms,
    set_lasm_json_response, LasmDbRecord, LasmDbRecordsAdapter, LasmDynamicResponseState,
    LasmRunRequest, LASM_INTERNAL_DB_HANDLE_HEADER, LASM_INTERNAL_DB_OP_HEADER,
    LASM_INTERNAL_DB_PARAMS_HEADER, LASM_INTERNAL_DB_ROW_SCHEMA_HEADER,
    LASM_INTERNAL_DB_TEMPLATE_HEADER, LASM_INTERNAL_DB_TX_DB_HEADER, LASM_INTERNAL_DB_TX_HEADER,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

fn persist_lasm_db_record_with_capacity_guard(
    state: &mut LasmDynamicResponseState,
    record: &LasmDbRecord,
) {
    let history_overflowed = append_lasm_dynamic_db_record(state, record.clone());
    if let Err(message) = persist_lasm_dynamic_db_record_append(state, record) {
        eprintln!("warning: LASM dynamic records store persistence failed: {message}");
    }
    if history_overflowed {
        if let Err(message) = persist_lasm_dynamic_db_records_full_sync(state) {
            eprintln!(
                "warning: LASM dynamic records store compaction sync failed after \
                 in-memory overflow: {message}"
            );
        }
    }
}

pub(crate) fn apply_lasm_internal_db_operation_materialization(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_records_adapter: LasmDbRecordsAdapter,
    trace_id: &str,
) -> bool {
    let Some(raw_operation) = take_lasm_internal_header_value(response, LASM_INTERNAL_DB_OP_HEADER)
    else {
        return false;
    };
    let operation = materialize_lasm_internal_header_value(raw_operation, request, path_params);
    let operation = operation.trim();
    match operation {
        "exec" => {
            let template = materialize_lasm_internal_header_value(
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TEMPLATE_HEADER)
                    .unwrap_or_default(),
                request,
                path_params,
            );
            if template.trim().is_empty() {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.SQL_TEMPLATE_INVALID",
                        "validation",
                        "sql.q query template is required",
                        400,
                        trace_id,
                    ),
                );
                return true;
            }
            let params = materialize_lasm_internal_header_value(
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_PARAMS_HEADER)
                    .unwrap_or_else(|| "0".to_string()),
                request,
                path_params,
            );
            let db_raw = materialize_lasm_internal_header_value(
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_HANDLE_HEADER)
                    .unwrap_or_else(|| "1".to_string()),
                request,
                path_params,
            );
            let db = match parse_lasm_positive_i64(db_raw.trim()) {
                Some(value) => value,
                None => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.EXEC_INVALID",
                            "validation",
                            "db.exec requires db capability and query handle",
                            400,
                            trace_id,
                        ),
                    );
                    return true;
                }
            };
            if !is_lasm_valid_db_cap_handle(db) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.EXEC_INVALID",
                        "validation",
                        "db.exec requires db capability and query handle",
                        400,
                        trace_id,
                    ),
                );
                return true;
            }
            let template = template.trim().to_string();
            let params = normalize_lasm_db_params(params.as_str());
            let postgres_preparsed = if db_records_adapter == LasmDbRecordsAdapter::Postgres {
                Some(parse_lasm_postgres_query_template_and_params(
                    template.as_str(),
                    params.as_str(),
                ))
            } else {
                None
            };
            let sqlite_params = if db_records_adapter == LasmDbRecordsAdapter::Sqlite {
                Some(parse_lasm_sqlite_query_params(params.as_str()))
            } else {
                None
            };
            let (record, affected_rows) = match dynamic_state.lock() {
                Ok(mut state) => {
                    debug_assert_eq!(state.db_records_adapter, db_records_adapter);
                    let mut affected_rows = 0u64;
                    if db_records_adapter == LasmDbRecordsAdapter::Postgres {
                        let (postgres_template, postgres_params) = match postgres_preparsed
                            .as_ref()
                            .expect("postgres preparse should exist for postgres adapter path")
                        {
                            Ok((rewritten_template, parsed_params)) => {
                                (rewritten_template.as_str(), parsed_params.as_slice())
                            }
                            Err(message) => {
                                set_lasm_json_response(
                                    response,
                                    400,
                                    &lasm_error_envelope(
                                        "DB.EXEC_INVALID",
                                        "validation",
                                        message.as_str(),
                                        400,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                        };
                        let postgres_affected_rows = match run_lasm_postgres_exec(
                            &mut state,
                            postgres_template,
                            postgres_params,
                        ) {
                            Ok(value) => value,
                            Err(message) => {
                                let (status, code, kind) =
                                    classify_lasm_db_runtime_error("exec", message.as_str());
                                set_lasm_json_response(
                                    response,
                                    status,
                                    &lasm_error_envelope(
                                        code,
                                        kind,
                                        message.as_str(),
                                        status,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                        };
                        affected_rows = postgres_affected_rows;
                    } else if db_records_adapter == LasmDbRecordsAdapter::Sqlite {
                        let sqlite_affected_rows = match run_lasm_sqlite_exec(
                            &mut state,
                            template.as_str(),
                            sqlite_params
                                .as_ref()
                                .expect("sqlite params should exist for sqlite adapter path"),
                        ) {
                            Ok(value) => value,
                            Err(message) => {
                                let (status, code, kind) =
                                    classify_lasm_db_runtime_error("exec", message.as_str());
                                set_lasm_json_response(
                                    response,
                                    status,
                                    &lasm_error_envelope(
                                        code,
                                        kind,
                                        message.as_str(),
                                        status,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                        };
                        affected_rows = sqlite_affected_rows;
                    }
                    let record = LasmDbRecord {
                        id: state.next_db_record_id,
                        op: "exec".to_string(),
                        db,
                        template: template.clone(),
                        params: params.clone(),
                        tx: 0,
                        affected_rows,
                        created_at_ms: lasm_now_ms(),
                    };
                    state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                    persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                    (record, affected_rows)
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
                    return true;
                }
            };
            set_lasm_json_response(
                response,
                200,
                &serde_json::json!({
                    "ok": true,
                    "recordId": record.id,
                    "db": record.db,
                    "op": record.op,
                    "template": record.template,
                    "params": record.params,
                    "tx": record.tx,
                    "affectedRows": affected_rows,
                }),
            );
            true
        }
        "execTx" => {
            let template = materialize_lasm_internal_header_value(
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TEMPLATE_HEADER)
                    .unwrap_or_default(),
                request,
                path_params,
            );
            if template.trim().is_empty() {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.SQL_TEMPLATE_INVALID",
                        "validation",
                        "sql.q query template is required",
                        400,
                        trace_id,
                    ),
                );
                return true;
            }
            let params = materialize_lasm_internal_header_value(
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_PARAMS_HEADER)
                    .unwrap_or_else(|| "0".to_string()),
                request,
                path_params,
            );
            let tx_db_source =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_DB_HEADER).map(
                    |value| materialize_lasm_internal_header_value(value, request, path_params),
                );
            let tx_handle_raw =
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TX_HEADER).map(
                    |value| materialize_lasm_internal_header_value(value, request, path_params),
                );
            let template = template.trim().to_string();
            let params = normalize_lasm_db_params(params.as_str());
            enum ExecTxSource {
                AllocateFromDb(i64),
                ExistingTx(i64),
            }
            let tx_source = if let Some(db_raw) = tx_db_source {
                let Some(db_value) = parse_lasm_positive_i64(db_raw.trim()) else {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.EXEC_TX_INVALID",
                            "validation",
                            "db.execTx requires transaction and query handles",
                            400,
                            trace_id,
                        ),
                    );
                    return true;
                };
                ExecTxSource::AllocateFromDb(db_value)
            } else {
                let Some(tx_raw) = tx_handle_raw.as_ref() else {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.EXEC_TX_INVALID",
                            "validation",
                            "db.execTx requires transaction and query handles",
                            400,
                            trace_id,
                        ),
                    );
                    return true;
                };
                let Some(tx_value) = parse_lasm_positive_i64(tx_raw.trim()) else {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.EXEC_TX_INVALID",
                            "validation",
                            "db.execTx requires transaction and query handles",
                            400,
                            trace_id,
                        ),
                    );
                    return true;
                };
                ExecTxSource::ExistingTx(tx_value)
            };
            if let ExecTxSource::AllocateFromDb(db_value) = &tx_source {
                if !is_lasm_valid_db_cap_handle(*db_value) {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.EXEC_TX_INVALID",
                            "validation",
                            "db.execTx requires db.tx(dbCap) with valid db capability handle",
                            400,
                            trace_id,
                        ),
                    );
                    return true;
                }
            }
            let postgres_preparsed = if db_records_adapter == LasmDbRecordsAdapter::Postgres {
                Some(parse_lasm_postgres_query_template_and_params(
                    template.as_str(),
                    params.as_str(),
                ))
            } else {
                None
            };
            let sqlite_params = if db_records_adapter == LasmDbRecordsAdapter::Sqlite {
                Some(parse_lasm_sqlite_query_params(params.as_str()))
            } else {
                None
            };

            let (record, affected_rows) = match dynamic_state.lock() {
                Ok(mut state) => {
                    debug_assert_eq!(state.db_records_adapter, db_records_adapter);
                    let (db, tx, allocated_tx_handle) = match tx_source {
                        ExecTxSource::AllocateFromDb(db_value) => {
                            let Some(tx_value) = allocate_lasm_db_tx_handle(&mut state, db_value)
                            else {
                                set_lasm_json_response(
                                    response,
                                    500,
                                    &lasm_error_envelope(
                                        "DB.TX_INTERNAL",
                                        "internal",
                                        "db.tx runtime failure",
                                        500,
                                        trace_id,
                                    ),
                                );
                                return true;
                            };
                            (db_value, tx_value, Some(tx_value))
                        }
                        ExecTxSource::ExistingTx(tx_value) => {
                            let Some(db_value) = state.db_tx_handles.get(&tx_value).copied() else {
                                set_lasm_json_response(
                                    response,
                                    400,
                                    &lasm_error_envelope(
                                        "DB.EXEC_TX_HANDLE_INVALID",
                                        "validation",
                                        "db.execTx transaction handle must come from db.tx",
                                        400,
                                        trace_id,
                                    ),
                                );
                                return true;
                            };
                            (db_value, tx_value, None)
                        }
                    };
                    let mut affected_rows = 0u64;
                    if db_records_adapter == LasmDbRecordsAdapter::Postgres {
                        let (postgres_template, postgres_params) = match postgres_preparsed
                            .as_ref()
                            .expect("postgres preparse should exist for postgres adapter path")
                        {
                            Ok((rewritten_template, parsed_params)) => {
                                (rewritten_template.as_str(), parsed_params.as_slice())
                            }
                            Err(message) => {
                                if let Some(tx_handle) = allocated_tx_handle {
                                    state.db_tx_handles.remove(&tx_handle);
                                }
                                set_lasm_json_response(
                                    response,
                                    400,
                                    &lasm_error_envelope(
                                        "DB.EXEC_TX_INVALID",
                                        "validation",
                                        message.as_str(),
                                        400,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                        };
                        let postgres_affected_rows = match run_lasm_postgres_exec_tx(
                            &mut state,
                            postgres_template,
                            postgres_params,
                        ) {
                            Ok(value) => value,
                            Err(message) => {
                                if let Some(tx_handle) = allocated_tx_handle {
                                    state.db_tx_handles.remove(&tx_handle);
                                }
                                let (status, code, kind) =
                                    classify_lasm_db_runtime_error("execTx", message.as_str());
                                set_lasm_json_response(
                                    response,
                                    status,
                                    &lasm_error_envelope(
                                        code,
                                        kind,
                                        message.as_str(),
                                        status,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                        };
                        affected_rows = postgres_affected_rows;
                    } else if db_records_adapter == LasmDbRecordsAdapter::Sqlite {
                        let sqlite_affected_rows = match run_lasm_sqlite_exec_tx(
                            &mut state,
                            template.as_str(),
                            sqlite_params
                                .as_ref()
                                .expect("sqlite params should exist for sqlite adapter path"),
                        ) {
                            Ok(value) => value,
                            Err(message) => {
                                if let Some(tx_handle) = allocated_tx_handle {
                                    state.db_tx_handles.remove(&tx_handle);
                                }
                                let (status, code, kind) =
                                    classify_lasm_db_runtime_error("execTx", message.as_str());
                                set_lasm_json_response(
                                    response,
                                    status,
                                    &lasm_error_envelope(
                                        code,
                                        kind,
                                        message.as_str(),
                                        status,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                        };
                        affected_rows = sqlite_affected_rows;
                    }
                    if let Some(tx_handle) = allocated_tx_handle {
                        state.db_tx_handles.remove(&tx_handle);
                    }
                    let record = LasmDbRecord {
                        id: state.next_db_record_id,
                        op: "execTx".to_string(),
                        db,
                        template: template.clone(),
                        params: params.clone(),
                        tx,
                        affected_rows,
                        created_at_ms: lasm_now_ms(),
                    };
                    state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                    persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                    (record, affected_rows)
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
                    return true;
                }
            };
            set_lasm_json_response(
                response,
                200,
                &serde_json::json!({
                    "ok": true,
                    "recordId": record.id,
                    "db": record.db,
                    "op": record.op,
                    "template": record.template,
                    "params": record.params,
                    "tx": record.tx,
                    "affectedRows": affected_rows,
                }),
            );
            true
        }
        "queryOne" => {
            let template = materialize_lasm_internal_header_value(
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_TEMPLATE_HEADER)
                    .unwrap_or_default(),
                request,
                path_params,
            );
            if template.trim().is_empty() {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.SQL_TEMPLATE_INVALID",
                        "validation",
                        "sql.q query template is required",
                        400,
                        trace_id,
                    ),
                );
                return true;
            }
            let params = materialize_lasm_internal_header_value(
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_PARAMS_HEADER)
                    .unwrap_or_else(|| "0".to_string()),
                request,
                path_params,
            );
            let db_raw = materialize_lasm_internal_header_value(
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_HANDLE_HEADER)
                    .unwrap_or_else(|| "1".to_string()),
                request,
                path_params,
            );
            let db = match parse_lasm_positive_i64(db_raw.trim()) {
                Some(value) => value,
                None => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.QUERY_ONE_INVALID",
                            "validation",
                            "db.queryOne requires db capability, query, and row schema handles",
                            400,
                            trace_id,
                        ),
                    );
                    return true;
                }
            };
            if !is_lasm_valid_db_cap_handle(db) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "DB.QUERY_ONE_INVALID",
                        "validation",
                        "db.queryOne requires db capability, query, and row schema handles",
                        400,
                        trace_id,
                    ),
                );
                return true;
            }
            let row_schema_raw = materialize_lasm_internal_header_value(
                take_lasm_internal_header_value(response, LASM_INTERNAL_DB_ROW_SCHEMA_HEADER)
                    .unwrap_or_else(|| "1".to_string()),
                request,
                path_params,
            );
            let row_schema = match parse_lasm_positive_i64(row_schema_raw.trim()) {
                Some(value) => value,
                None => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "DB.QUERY_ONE_INVALID",
                            "validation",
                            "db.queryOne requires db capability, query, and row schema handles",
                            400,
                            trace_id,
                        ),
                    );
                    return true;
                }
            };
            let template = template.trim().to_string();
            let params = normalize_lasm_db_params(params.as_str());
            let postgres_preparsed = if db_records_adapter == LasmDbRecordsAdapter::Postgres {
                Some(parse_lasm_postgres_query_template_and_params(
                    template.as_str(),
                    params.as_str(),
                ))
            } else {
                None
            };
            let sqlite_params = if db_records_adapter == LasmDbRecordsAdapter::Sqlite {
                Some(parse_lasm_sqlite_query_params(params.as_str()))
            } else {
                None
            };
            let matched_record = match dynamic_state.lock() {
                Ok(mut state) => {
                    debug_assert_eq!(state.db_records_adapter, db_records_adapter);
                    if db_records_adapter == LasmDbRecordsAdapter::Postgres {
                        let (postgres_template, postgres_params) = match postgres_preparsed
                            .as_ref()
                            .expect("postgres preparse should exist for postgres adapter path")
                        {
                            Ok((rewritten_template, parsed_params)) => {
                                (rewritten_template.as_str(), parsed_params.as_slice())
                            }
                            Err(message) => {
                                set_lasm_json_response(
                                    response,
                                    400,
                                    &lasm_error_envelope(
                                        "DB.QUERY_ONE_INVALID",
                                        "validation",
                                        message.as_str(),
                                        400,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                        };
                        let row_object = match run_lasm_postgres_query_one(
                            &mut state,
                            postgres_template,
                            postgres_params,
                        ) {
                            Ok(Some(value)) => value,
                            Ok(None) => {
                                set_lasm_json_response(
                                    response,
                                    404,
                                    &lasm_error_envelope(
                                        "DB.QUERY_ONE_NOT_FOUND",
                                        "missing_dependency",
                                        "db.queryOne row not found",
                                        404,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                            Err(message) => {
                                let (status, code, kind) =
                                    classify_lasm_db_runtime_error("queryOne", message.as_str());
                                set_lasm_json_response(
                                    response,
                                    status,
                                    &lasm_error_envelope(
                                        code,
                                        kind,
                                        message.as_str(),
                                        status,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                        };
                        let row =
                            serde_json::to_string(&row_object).unwrap_or_else(|_| "{}".to_string());
                        let record = LasmDbRecord {
                            id: state.next_db_record_id,
                            op: "queryOne".to_string(),
                            db,
                            template: template.clone(),
                            params: params.clone(),
                            tx: 0,
                            affected_rows: 1,
                            created_at_ms: lasm_now_ms(),
                        };
                        state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                        persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                        set_lasm_json_response(
                            response,
                            200,
                            &serde_json::json!({
                                "ok": true,
                                "recordId": record.id,
                                "rowSchema": row_schema,
                                "row": row,
                                "rowObject": row_object,
                                "record": lasm_db_record_to_json(&record),
                            }),
                        );
                        return true;
                    }
                    if db_records_adapter == LasmDbRecordsAdapter::Sqlite {
                        let row_object = match run_lasm_sqlite_query_one(
                            &mut state,
                            template.as_str(),
                            sqlite_params
                                .as_ref()
                                .expect("sqlite params should exist for sqlite adapter path"),
                        ) {
                            Ok(Some(value)) => value,
                            Ok(None) => {
                                set_lasm_json_response(
                                    response,
                                    404,
                                    &lasm_error_envelope(
                                        "DB.QUERY_ONE_NOT_FOUND",
                                        "missing_dependency",
                                        "db.queryOne row not found",
                                        404,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                            Err(message) => {
                                let (status, code, kind) =
                                    classify_lasm_db_runtime_error("queryOne", message.as_str());
                                set_lasm_json_response(
                                    response,
                                    status,
                                    &lasm_error_envelope(
                                        code,
                                        kind,
                                        message.as_str(),
                                        status,
                                        trace_id,
                                    ),
                                );
                                return true;
                            }
                        };
                        let row =
                            serde_json::to_string(&row_object).unwrap_or_else(|_| "{}".to_string());
                        let record = LasmDbRecord {
                            id: state.next_db_record_id,
                            op: "queryOne".to_string(),
                            db,
                            template: template.clone(),
                            params: params.clone(),
                            tx: 0,
                            affected_rows: 1,
                            created_at_ms: lasm_now_ms(),
                        };
                        state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                        persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                        set_lasm_json_response(
                            response,
                            200,
                            &serde_json::json!({
                                "ok": true,
                                "recordId": record.id,
                                "rowSchema": row_schema,
                                "row": row,
                                "rowObject": row_object,
                                "record": lasm_db_record_to_json(&record),
                            }),
                        );
                        return true;
                    }
                    let signature =
                        crate::lasm_db_record_signature_key(db, template.as_str(), params.as_str());
                    if !state.db_record_signatures.contains_key(signature.as_str()) {
                        None
                    } else {
                        let matched_source_record = state
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
                            });
                        if let Some(matched_source_record) = matched_source_record {
                            let record = LasmDbRecord {
                                id: state.next_db_record_id,
                                op: "queryOne".to_string(),
                                db,
                                template: template.clone(),
                                params: params.clone(),
                                tx: 0,
                                affected_rows: 1,
                                created_at_ms: lasm_now_ms(),
                            };
                            state.next_db_record_id = state.next_db_record_id.saturating_add(1);
                            persist_lasm_db_record_with_capacity_guard(&mut state, &record);
                            Some((record, matched_source_record))
                        } else {
                            None
                        }
                    }
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
                    return true;
                }
            };
            let Some((record, matched_source_record)) = matched_record else {
                set_lasm_json_response(
                    response,
                    404,
                    &lasm_error_envelope(
                        "DB.QUERY_ONE_NOT_FOUND",
                        "missing_dependency",
                        "db.queryOne record not found",
                        404,
                        trace_id,
                    ),
                );
                return true;
            };
            let row_object = serde_json::json!({
                "op": matched_source_record.op,
                "db": matched_source_record.db,
                "template": matched_source_record.template,
                "params": matched_source_record.params,
                "tx": matched_source_record.tx,
                "affected_rows": matched_source_record.affected_rows,
                "rowSchema": row_schema,
            });
            let row = serde_json::to_string(&row_object).unwrap_or_else(|_| "{}".to_string());
            set_lasm_json_response(
                response,
                200,
                &serde_json::json!({
                    "ok": true,
                    "recordId": record.id,
                    "rowSchema": row_schema,
                    "row": row,
                    "rowObject": row_object,
                    "record": lasm_db_record_to_json(&record),
                }),
            );
            true
        }
        _ => false,
    }
}

fn take_lasm_internal_header_value(
    response: &mut sec4_core::HttpResponse,
    header_name: &str,
) -> Option<String> {
    let key = crate::find_lasm_header_key_case_insensitive(&response.headers, header_name)?;
    response.headers.remove(&key)
}

fn materialize_lasm_internal_header_value(
    value: String,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
) -> String {
    if crate::contains_lasm_request_placeholder_tokens(value.as_str()) {
        crate::materialize_lasm_request_placeholders(value.as_str(), request, path_params)
    } else {
        value
    }
}
