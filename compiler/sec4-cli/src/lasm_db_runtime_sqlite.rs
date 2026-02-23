use base64::Engine;
use rusqlite::{
    types::{Value as SqliteValue, ValueRef as SqliteValueRef},
    Connection, ToSql,
};
use std::collections::BTreeMap;

use crate::lasm_db_adapter_state::connect_lasm_dynamic_db_records_sqlite;
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

#[derive(Clone, Debug)]
pub(crate) enum LasmSqliteQueryParams {
    Positional(Vec<SqliteValue>),
    Named(Vec<(String, SqliteValue)>),
}

impl LasmSqliteQueryParams {
    fn is_empty(&self) -> bool {
        match self {
            Self::Positional(values) => values.is_empty(),
            Self::Named(values) => values.is_empty(),
        }
    }
}

fn parse_lasm_sqlite_positional_object_params(
    entries: &serde_json::Map<String, serde_json::Value>,
) -> Option<Vec<SqliteValue>> {
    let mut indexed = Vec::with_capacity(entries.len());
    let mut max_index = 0usize;
    for (key, value) in entries {
        let index = parse_lasm_sqlite_positional_param_index(key.as_str())?;
        if index == 0 {
            return None;
        }
        max_index = max_index.max(index);
        indexed.push((index, parse_lasm_sqlite_query_param_value(value.clone())));
    }
    let mut params = vec![SqliteValue::Null; max_index];
    for (index, value) in indexed {
        params[index - 1] = value;
    }
    Some(params)
}

fn parse_lasm_sqlite_named_object_params(
    entries: &serde_json::Map<String, serde_json::Value>,
) -> Option<Vec<(String, SqliteValue)>> {
    let mut named = BTreeMap::new();
    for (key, value) in entries {
        let key = parse_lasm_sqlite_named_param_key(key.as_str())?;
        named.insert(key, parse_lasm_sqlite_query_param_value(value.clone()));
    }
    Some(named.into_iter().collect())
}

fn parse_lasm_sqlite_named_param_key(key: &str) -> Option<String> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return None;
    }
    let raw = trimmed
        .strip_prefix(':')
        .or_else(|| trimmed.strip_prefix('@'))
        .or_else(|| trimmed.strip_prefix('$'))
        .unwrap_or(trimmed);
    if raw.is_empty() {
        return None;
    }
    if !raw
        .chars()
        .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    {
        return None;
    }
    Some(format!(":{raw}"))
}

fn parse_lasm_sqlite_positional_param_index(key: &str) -> Option<usize> {
    let trimmed = key.trim();
    let digits = trimmed
        .strip_prefix('$')
        .or_else(|| trimmed.strip_prefix('?'))
        .unwrap_or(trimmed);
    let index = digits.parse::<usize>().ok()?;
    if index == 0 {
        return None;
    }
    Some(index)
}

pub(crate) fn parse_lasm_sqlite_query_params(value: &str) -> LasmSqliteQueryParams {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return LasmSqliteQueryParams::Positional(Vec::new());
    }
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return match parsed {
            serde_json::Value::Array(entries) => LasmSqliteQueryParams::Positional(
                entries
                    .into_iter()
                    .map(parse_lasm_sqlite_query_param_value)
                    .collect(),
            ),
            serde_json::Value::Object(entries) => {
                if let Some(params) = parse_lasm_sqlite_positional_object_params(&entries) {
                    LasmSqliteQueryParams::Positional(params)
                } else if let Some(named) = parse_lasm_sqlite_named_object_params(&entries) {
                    LasmSqliteQueryParams::Named(named)
                } else {
                    LasmSqliteQueryParams::Positional(vec![parse_lasm_sqlite_query_param_value(
                        serde_json::Value::Object(entries),
                    )])
                }
            }
            serde_json::Value::Null => LasmSqliteQueryParams::Positional(Vec::new()),
            other => {
                LasmSqliteQueryParams::Positional(vec![parse_lasm_sqlite_query_param_value(other)])
            }
        };
    }
    LasmSqliteQueryParams::Positional(vec![SqliteValue::Text(trimmed.to_string())])
}

fn lasm_dynamic_sqlite_runtime_connection_mut(
    state: &mut LasmDynamicResponseState,
) -> Result<&mut Connection, String> {
    if state.db_records_sqlite_connection.is_none() {
        let path = state
            .db_records_sqlite_store_path
            .as_ref()
            .ok_or_else(|| "sqlite records store path unavailable".to_string())?;
        let connection = connect_lasm_dynamic_db_records_sqlite(
            path.as_path(),
            state.db_sqlite_busy_timeout_ms.max(1),
        )?;
        state.db_records_sqlite_connection = Some(connection);
    }
    state
        .db_records_sqlite_connection
        .as_mut()
        .ok_or_else(|| "sqlite records store connection unavailable".to_string())
}

fn lasm_sqlite_runtime_error_is_no_retry(message: &str) -> bool {
    message.contains("database is locked") || message.contains("bad parameter")
}

fn run_lasm_sqlite_with_connection_retry<T, F>(
    state: &mut LasmDynamicResponseState,
    error_prefix: &str,
    mut run: F,
) -> Result<T, String>
where
    F: FnMut(&mut Connection) -> Result<T, String>,
{
    let first = {
        let connection = lasm_dynamic_sqlite_runtime_connection_mut(state)?;
        run(connection)
    };
    match first {
        Ok(value) => Ok(value),
        Err(message) if lasm_sqlite_runtime_error_is_no_retry(message.as_str()) => Err(message),
        Err(message) => {
            state.db_records_sqlite_connection = None;
            let connection = lasm_dynamic_sqlite_runtime_connection_mut(state)?;
            run(connection).map_err(|retry_error| {
                format!(
                    "{error_prefix} failed: {message}; retry after reconnect failed: {retry_error}"
                )
            })
        }
    }
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

fn lasm_sqlite_named_param_refs(params: &[(String, SqliteValue)]) -> Vec<(&str, &dyn ToSql)> {
    params
        .iter()
        .map(|(name, value)| (name.as_str(), value as &dyn ToSql))
        .collect()
}

pub(crate) fn run_lasm_sqlite_exec(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
    sqlite_params: &LasmSqliteQueryParams,
) -> Result<u64, String> {
    run_lasm_sqlite_with_connection_retry(state, "sqlite execution", |connection| {
        let tx = connection
            .transaction()
            .map_err(|err| format!("sqlite execution transaction start failed: {err}"))?;
        if sqlite_params.is_empty() && has_lasm_sql_non_trailing_statement_separator(query_template) {
            let before_changes = tx.total_changes();
            tx.execute_batch(query_template)
                .map_err(|err| format!("sqlite execution failed: {err}"))?;
            let affected_rows = tx.total_changes().saturating_sub(before_changes);
            tx.commit()
                .map_err(|err| format!("sqlite execution transaction commit failed: {err}"))?;
            return Ok(affected_rows);
        }
        let mut statement = tx
            .prepare_cached(query_template)
            .map_err(|err| format!("sqlite execution prepare failed: {err}"))?;
        let parameter_count = statement.parameter_count();
        if let LasmSqliteQueryParams::Positional(values) = sqlite_params {
            validate_lasm_sqlite_parameter_arity(parameter_count, values.len())?;
        }
        let use_params = parameter_count > 0 && !sqlite_params.is_empty();
        if use_params && has_lasm_sql_non_trailing_statement_separator(query_template) {
            return Err(
                "sqlite parameterized execution requires a single SQL statement".to_string(),
            );
        }
        let execute_result = if use_params {
            match sqlite_params {
                LasmSqliteQueryParams::Positional(values) => {
                    statement.execute(rusqlite::params_from_iter(values.iter()))
                }
                LasmSqliteQueryParams::Named(values) => {
                    let named_refs = lasm_sqlite_named_param_refs(values.as_slice());
                    statement.execute(named_refs.as_slice())
                }
            }
        } else {
            statement.execute([])
        };
        let affected_rows = match execute_result {
            Ok(count) => count as u64,
            Err(rusqlite::Error::ExecuteReturnedResults) => {
                let mut rows = if use_params {
                    match sqlite_params {
                        LasmSqliteQueryParams::Positional(values) => statement
                            .query(rusqlite::params_from_iter(values.iter()))
                            .map_err(|err| format!("sqlite execution query failed: {err}"))?,
                        LasmSqliteQueryParams::Named(values) => {
                            let named_refs = lasm_sqlite_named_param_refs(values.as_slice());
                            statement
                                .query(named_refs.as_slice())
                                .map_err(|err| format!("sqlite execution query failed: {err}"))?
                        }
                    }
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
    })
}

pub(crate) fn run_lasm_sqlite_exec_tx(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
    sqlite_params: &LasmSqliteQueryParams,
) -> Result<u64, String> {
    run_lasm_sqlite_exec(state, query_template, sqlite_params)
}

pub(crate) fn run_lasm_sqlite_query_one(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
    sqlite_params: &LasmSqliteQueryParams,
) -> Result<Option<serde_json::Value>, String> {
    let normalized_query = normalize_lasm_postgres_query_for_subquery(query_template);
    if normalized_query.trim().is_empty() {
        return Err("sqlite queryOne requires non-empty SQL statement".to_string());
    }
    if !is_lasm_postgres_query_one_select_like(normalized_query.as_str()) {
        return Err("sqlite queryOne requires SELECT-style SQL statement".to_string());
    }
    run_lasm_sqlite_with_connection_retry(state, "sqlite queryOne", |connection| {
        let mut statement = connection
            .prepare_cached(normalized_query.as_str())
            .map_err(|err| format!("sqlite queryOne prepare failed: {err}"))?;
        let parameter_count = statement.parameter_count();
        if let LasmSqliteQueryParams::Positional(values) = sqlite_params {
            validate_lasm_sqlite_parameter_arity(parameter_count, values.len())?;
        }
        let use_params = parameter_count > 0 && !sqlite_params.is_empty();
        if use_params && has_lasm_sql_non_trailing_statement_separator(normalized_query.as_str()) {
            return Err(
                "sqlite parameterized execution requires a single SQL statement".to_string(),
            );
        }
        let mut rows = if use_params {
            match sqlite_params {
                LasmSqliteQueryParams::Positional(values) => statement
                    .query(rusqlite::params_from_iter(values.iter()))
                    .map_err(|err| format!("sqlite queryOne execution failed: {err}"))?,
                LasmSqliteQueryParams::Named(values) => {
                    let named_refs = lasm_sqlite_named_param_refs(values.as_slice());
                    statement
                        .query(named_refs.as_slice())
                        .map_err(|err| format!("sqlite queryOne execution failed: {err}"))?
                }
            }
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
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_lasm_sqlite_query_params, LasmSqliteQueryParams};
    use rusqlite::types::Value as SqliteValue;

    #[test]
    fn positional_object_params_expand_with_null_fill() {
        let params = parse_lasm_sqlite_query_params("{\"2\":5}");
        let LasmSqliteQueryParams::Positional(values) = params else {
            panic!("expected positional params");
        };
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], SqliteValue::Null);
        assert_eq!(values[1], SqliteValue::Integer(5));
    }

    #[test]
    fn non_numeric_object_params_fall_back_to_single_text_param() {
        let params = parse_lasm_sqlite_query_params("{\"user\":\"alice\"}");
        let LasmSqliteQueryParams::Named(values) = params else {
            panic!("expected named params");
        };
        assert_eq!(values.len(), 1);
        assert_eq!(
            values[0],
            (":user".to_string(), SqliteValue::Text("alice".to_string()))
        );
    }

    #[test]
    fn positional_object_params_accept_placeholder_prefixed_keys() {
        let params = parse_lasm_sqlite_query_params("{\"?3\":7}");
        let LasmSqliteQueryParams::Positional(values) = params else {
            panic!("expected positional params");
        };
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], SqliteValue::Null);
        assert_eq!(values[1], SqliteValue::Null);
        assert_eq!(values[2], SqliteValue::Integer(7));
    }

    #[test]
    fn named_object_params_accept_prefixed_and_plain_names() {
        let params = parse_lasm_sqlite_query_params("{\"name\":\"alice\",\"@role\":\"admin\"}");
        let LasmSqliteQueryParams::Named(values) = params else {
            panic!("expected named params");
        };
        assert_eq!(
            values,
            vec![
                (":name".to_string(), SqliteValue::Text("alice".to_string())),
                (":role".to_string(), SqliteValue::Text("admin".to_string()))
            ]
        );
    }

    #[test]
    fn invalid_named_object_params_fall_back_to_single_text_param() {
        let params = parse_lasm_sqlite_query_params("{\"user-name\":\"alice\"}");
        let LasmSqliteQueryParams::Positional(values) = params else {
            panic!("expected fallback positional params");
        };
        assert_eq!(values.len(), 1);
        assert_eq!(
            values[0],
            SqliteValue::Text("{\"user-name\":\"alice\"}".to_string())
        );
    }
}
