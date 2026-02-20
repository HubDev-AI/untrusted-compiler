use base64::Engine;
use rusqlite::{
    types::{Value as SqliteValue, ValueRef as SqliteValueRef},
    Connection,
};
use std::fs;

use crate::lasm_db_adapter_state::ensure_lasm_dynamic_db_records_sqlite_schema;
use crate::lasm_db_runtime_postgres::{
    is_lasm_postgres_query_one_select_like, normalize_lasm_postgres_query_for_subquery,
};
use crate::{has_lasm_sql_non_trailing_statement_separator, LasmDynamicResponseState};

fn parse_lasm_sqlite_query_param_value(value: serde_json::Value) -> SqliteValue {
    match value {
        serde_json::Value::Null => SqliteValue::Null,
        serde_json::Value::Bool(inner) => SqliteValue::Integer(if inner { 1 } else { 0 }),
        serde_json::Value::Number(inner) => {
            if let Some(value) = inner.as_i64() {
                SqliteValue::Integer(value)
            } else if let Some(value) = inner.as_f64() {
                SqliteValue::Real(value)
            } else {
                SqliteValue::Text(inner.to_string())
            }
        }
        serde_json::Value::String(inner) => SqliteValue::Text(inner),
        other => SqliteValue::Text(serde_json::to_string(&other).unwrap_or_default()),
    }
}

fn parse_lasm_sqlite_query_params(value: &str) -> Vec<SqliteValue> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return Vec::new();
    }
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return match parsed {
            serde_json::Value::Array(entries) => entries
                .into_iter()
                .map(parse_lasm_sqlite_query_param_value)
                .collect(),
            serde_json::Value::Null => Vec::new(),
            other => vec![parse_lasm_sqlite_query_param_value(other)],
        };
    }
    vec![SqliteValue::Text(trimmed.to_string())]
}

fn lasm_dynamic_sqlite_runtime_connection(
    state: &LasmDynamicResponseState,
) -> Result<Connection, String> {
    let path = state
        .db_records_sqlite_store_path
        .as_ref()
        .ok_or_else(|| "sqlite records store path unavailable".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "could not create LASM dynamic sqlite runtime store directory `{}`: {err}",
                parent.display()
            )
        })?;
    }
    let connection = Connection::open(path).map_err(|err| {
        format!(
            "could not open LASM dynamic sqlite runtime store `{}`: {err}",
            path.display()
        )
    })?;
    ensure_lasm_dynamic_db_records_sqlite_schema(&connection).map_err(|err| {
        format!(
            "could not initialize LASM dynamic sqlite runtime schema `{}`: {err}",
            path.display()
        )
    })?;
    Ok(connection)
}

fn sqlite_value_ref_to_json(value: SqliteValueRef<'_>) -> serde_json::Value {
    match value {
        SqliteValueRef::Null => serde_json::Value::Null,
        SqliteValueRef::Integer(inner) => serde_json::Value::Number(inner.into()),
        SqliteValueRef::Real(inner) => serde_json::Number::from_f64(inner)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        SqliteValueRef::Text(inner) => {
            serde_json::Value::String(String::from_utf8_lossy(inner).to_string())
        }
        SqliteValueRef::Blob(inner) => {
            serde_json::Value::String(base64::engine::general_purpose::STANDARD.encode(inner))
        }
    }
}

fn validate_lasm_sqlite_parameter_arity(
    parameter_count: usize,
    provided_count: usize,
) -> Result<(), String> {
    if provided_count >= parameter_count {
        return Ok(());
    }
    Err(format!(
        "sqlite query requires at least {parameter_count} sql parameters but received {provided_count}"
    ))
}

pub(crate) fn run_lasm_sqlite_exec(
    state: &LasmDynamicResponseState,
    query_template: &str,
    params: &str,
) -> Result<u64, String> {
    let mut connection = lasm_dynamic_sqlite_runtime_connection(state)?;
    let sqlite_params = parse_lasm_sqlite_query_params(params);
    let tx = connection
        .transaction()
        .map_err(|err| format!("sqlite execution transaction start failed: {err}"))?;
    let mut statement = tx
        .prepare(query_template)
        .map_err(|err| format!("sqlite execution prepare failed: {err}"))?;
    let parameter_count = statement.parameter_count();
    validate_lasm_sqlite_parameter_arity(parameter_count, sqlite_params.len())?;
    let use_params = parameter_count > 0 && !sqlite_params.is_empty();
    if use_params && has_lasm_sql_non_trailing_statement_separator(query_template) {
        return Err("sqlite parameterized execution requires a single SQL statement".to_string());
    }
    let execute_result = if use_params {
        statement.execute(rusqlite::params_from_iter(sqlite_params.iter()))
    } else {
        statement.execute([])
    };
    let affected_rows = match execute_result {
        Ok(count) => count as u64,
        Err(rusqlite::Error::ExecuteReturnedResults) => {
            let mut rows = if use_params {
                statement
                    .query(rusqlite::params_from_iter(sqlite_params.iter()))
                    .map_err(|err| format!("sqlite execution query failed: {err}"))?
            } else {
                statement
                    .query([])
                    .map_err(|err| format!("sqlite execution query failed: {err}"))?
            };
            let mut row_count = 0u64;
            while rows
                .next()
                .map_err(|err| format!("sqlite execution row drain failed: {err}"))?
                .is_some()
            {
                row_count = row_count.saturating_add(1);
            }
            row_count
        }
        Err(err) => return Err(format!("sqlite execution failed: {err}")),
    };
    drop(statement);
    tx.commit()
        .map_err(|err| format!("sqlite execution transaction commit failed: {err}"))?;
    Ok(affected_rows)
}

pub(crate) fn run_lasm_sqlite_exec_tx(
    state: &LasmDynamicResponseState,
    query_template: &str,
    params: &str,
) -> Result<u64, String> {
    run_lasm_sqlite_exec(state, query_template, params)
}

pub(crate) fn run_lasm_sqlite_query_one(
    state: &LasmDynamicResponseState,
    query_template: &str,
    params: &str,
) -> Result<Option<serde_json::Value>, String> {
    let normalized_query = normalize_lasm_postgres_query_for_subquery(query_template);
    if normalized_query.trim().is_empty() {
        return Err("sqlite queryOne requires non-empty SQL statement".to_string());
    }
    if !is_lasm_postgres_query_one_select_like(normalized_query.as_str()) {
        return Err("sqlite queryOne requires SELECT-style SQL statement".to_string());
    }
    let connection = lasm_dynamic_sqlite_runtime_connection(state)?;
    let sqlite_params = parse_lasm_sqlite_query_params(params);
    let mut statement = connection
        .prepare(normalized_query.as_str())
        .map_err(|err| format!("sqlite queryOne prepare failed: {err}"))?;
    let parameter_count = statement.parameter_count();
    validate_lasm_sqlite_parameter_arity(parameter_count, sqlite_params.len())?;
    let use_params = parameter_count > 0 && !sqlite_params.is_empty();
    if use_params && has_lasm_sql_non_trailing_statement_separator(normalized_query.as_str()) {
        return Err("sqlite parameterized execution requires a single SQL statement".to_string());
    }
    let mut rows = if use_params {
        statement
            .query(rusqlite::params_from_iter(sqlite_params.iter()))
            .map_err(|err| format!("sqlite queryOne execution failed: {err}"))?
    } else {
        statement
            .query([])
            .map_err(|err| format!("sqlite queryOne execution failed: {err}"))?
    };
    let Some(row) = rows
        .next()
        .map_err(|err| format!("sqlite queryOne row fetch failed: {err}"))?
    else {
        return Ok(None);
    };
    let row_ref = row.as_ref();
    let mut object = serde_json::Map::new();
    for index in 0..row_ref.column_count() {
        let name = row_ref.column_name(index).unwrap_or("").to_string();
        let value = row
            .get_ref(index)
            .map(sqlite_value_ref_to_json)
            .map_err(|err| format!("sqlite queryOne row decode failed: {err}"))?;
        object.insert(name, value);
    }
    Ok(Some(serde_json::Value::Object(object)))
}
