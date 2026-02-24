use base64::Engine;
use rusqlite::{
    types::{Value as SqliteValue, ValueRef as SqliteValueRef},
    Connection, ToSql,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

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
) -> Vec<SqliteValue> {
    let mut indexed = Vec::with_capacity(entries.len());
    let mut max_index = 0usize;
    for (key, value) in entries {
        let index = parse_lasm_sqlite_positional_param_index(key.as_str())
            .expect("positional object keys should be validated before parsing");
        max_index = max_index.max(index);
        indexed.push((index, parse_lasm_sqlite_query_param_value(value.clone())));
    }
    let mut params = vec![SqliteValue::Null; max_index];
    for (index, value) in indexed {
        params[index - 1] = value;
    }
    params
}

fn parse_lasm_sqlite_named_object_params(
    entries: &serde_json::Map<String, serde_json::Value>,
) -> Result<Vec<(String, SqliteValue)>, String> {
    let mut named = BTreeMap::new();
    for (key, value) in entries {
        let key = parse_lasm_sqlite_named_param_key(key.as_str())
            .expect("named object keys should be validated before parsing");
        let value = parse_lasm_sqlite_query_param_value(value.clone());
        if named.insert(key.clone(), value).is_some() {
            return Err(format!(
                "sqlite params object contains duplicate normalized key `{}`",
                key.trim_start_matches(':'),
            ));
        }
    }
    Ok(named.into_iter().collect())
}

fn normalize_lasm_sqlite_named_param_raw(key: &str) -> Option<String> {
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
    if raw.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    if !raw
        .chars()
        .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
    {
        return None;
    }
    Some(raw.to_string())
}

fn parse_lasm_sqlite_named_param_key(key: &str) -> Option<String> {
    let raw = normalize_lasm_sqlite_named_param_raw(key)?;
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

enum LasmSqliteParamsObjectKeyStyle {
    Positional,
    Named,
}

fn classify_lasm_sqlite_params_object_keys(
    entries: &serde_json::Map<String, serde_json::Value>,
) -> Result<LasmSqliteParamsObjectKeyStyle, String> {
    let mut positional_all = true;
    let mut named_all = true;
    for key in entries.keys() {
        positional_all &= parse_lasm_sqlite_positional_param_index(key.as_str()).is_some();
        named_all &= parse_lasm_sqlite_named_param_key(key.as_str()).is_some();
    }
    if positional_all {
        return Ok(LasmSqliteParamsObjectKeyStyle::Positional);
    }
    if named_all {
        return Ok(LasmSqliteParamsObjectKeyStyle::Named);
    }
    Err(
        "sqlite params object keys must be all positional ($1/?1/1) or all named (:name/@name/$name/name)"
            .to_string(),
    )
}

pub(crate) fn parse_lasm_sqlite_query_params(value: &str) -> Result<LasmSqliteQueryParams, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return Ok(LasmSqliteQueryParams::Positional(Vec::new()));
    }
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return parse_lasm_sqlite_query_params_value(&parsed);
    }
    Ok(LasmSqliteQueryParams::Positional(vec![SqliteValue::Text(
        trimmed.to_string(),
    )]))
}

pub(crate) fn parse_lasm_sqlite_query_params_value(
    parsed: &serde_json::Value,
) -> Result<LasmSqliteQueryParams, String> {
    match parsed {
        serde_json::Value::Array(entries) => Ok(LasmSqliteQueryParams::Positional(
            entries
                .iter()
                .cloned()
                .map(parse_lasm_sqlite_query_param_value)
                .collect(),
        )),
        serde_json::Value::Object(entries) => {
            match classify_lasm_sqlite_params_object_keys(entries)? {
                LasmSqliteParamsObjectKeyStyle::Positional => {
                    Ok(LasmSqliteQueryParams::Positional(
                        parse_lasm_sqlite_positional_object_params(entries),
                    ))
                }
                LasmSqliteParamsObjectKeyStyle::Named => Ok(LasmSqliteQueryParams::Named(
                    parse_lasm_sqlite_named_object_params(entries)?,
                )),
            }
        }
        serde_json::Value::Null => Ok(LasmSqliteQueryParams::Positional(Vec::new())),
        other => Ok(LasmSqliteQueryParams::Positional(vec![
            parse_lasm_sqlite_query_param_value(other.clone()),
        ])),
    }
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
            state.db_sqlite_journal_mode.as_str(),
            state.db_sqlite_synchronous.as_str(),
        )?;
        state.db_records_sqlite_connection = Some(connection);
    }
    state
        .db_records_sqlite_connection
        .as_mut()
        .ok_or_else(|| "sqlite records store connection unavailable".to_string())
}

fn is_lasm_sqlite_runtime_lock_error(message: &str) -> bool {
    message.contains("database is locked")
}

fn is_lasm_sqlite_runtime_non_retryable_param_error(message: &str) -> bool {
    message.contains("bad parameter")
}

fn format_lasm_sqlite_runtime_error(context: &str, err: &rusqlite::Error) -> String {
    match err {
        rusqlite::Error::SqliteFailure(inner, _) => format!(
            "{context}: {err}; sqlite_code={:?}; sqlite_extended_code={}",
            inner.code, inner.extended_code
        ),
        _ => format!("{context}: {err}"),
    }
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
        Err(message) if is_lasm_sqlite_runtime_lock_error(message.as_str()) => {
            let mut latest_message = message;
            for _ in 0..state.db_sqlite_lock_retry_max {
                state.db_sqlite_lock_retry_attempts_total =
                    state.db_sqlite_lock_retry_attempts_total.saturating_add(1);
                if state.db_sqlite_lock_retry_delay_ms > 0 {
                    std::thread::sleep(Duration::from_millis(state.db_sqlite_lock_retry_delay_ms));
                }
                let retry = {
                    let connection = lasm_dynamic_sqlite_runtime_connection_mut(state)?;
                    run(connection)
                };
                match retry {
                    Ok(value) => {
                        state.db_sqlite_lock_retry_success_total =
                            state.db_sqlite_lock_retry_success_total.saturating_add(1);
                        return Ok(value);
                    }
                    Err(message) if is_lasm_sqlite_runtime_lock_error(message.as_str()) => {
                        latest_message = message;
                    }
                    Err(message) => return Err(message),
                }
            }
            Err(latest_message)
        }
        Err(message) if is_lasm_sqlite_runtime_non_retryable_param_error(message.as_str()) => {
            Err(message)
        }
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
    if parameter_count == 0 {
        return Ok(());
    }
    if provided_count < parameter_count {
        return Err(format!(
            "sqlite query requires at least {parameter_count} sql parameters but received {provided_count}"
        ));
    }
    if provided_count > parameter_count {
        return Err(format!(
            "sqlite query expects exactly {parameter_count} sql parameters but received {provided_count}"
        ));
    }
    Ok(())
}

fn resolve_lasm_sqlite_named_param_bindings<'a>(
    statement: &rusqlite::Statement<'_>,
    values: &'a [(String, SqliteValue)],
) -> Result<Vec<(String, &'a SqliteValue)>, String> {
    let mut provided = BTreeMap::new();
    for (name, value) in values {
        let raw = normalize_lasm_sqlite_named_param_raw(name.as_str())
            .ok_or_else(|| format!("sqlite query requires valid named parameter key `{name}`"))?;
        provided.insert(raw, value);
    }

    let mut used = BTreeSet::new();
    let mut bindings = Vec::new();
    for parameter_index in 1..=statement.parameter_count() {
        let Some(parameter_name) = statement.parameter_name(parameter_index) else {
            return Err(
                "sqlite named parameterized execution requires SQL placeholders to be named (:name, @name, or $name)"
                    .to_string(),
            );
        };
        let Some(raw) = normalize_lasm_sqlite_named_param_raw(parameter_name) else {
            return Err(format!(
                "sqlite named parameterized execution requires valid named placeholder `{parameter_name}`"
            ));
        };
        let Some(value) = provided.get(raw.as_str()) else {
            return Err(format!(
                "sqlite query requires named parameter `{raw}` in params object"
            ));
        };
        used.insert(raw);
        bindings.push((parameter_name.to_string(), *value));
    }
    for raw in provided.keys() {
        if !used.contains(raw) {
            return Err(format!(
                "sqlite query parameter `{raw}` is not present in SQL statement"
            ));
        }
    }
    Ok(bindings)
}

fn lasm_sqlite_named_param_refs<'a>(
    params: &'a [(String, &'a SqliteValue)],
) -> Vec<(&'a str, &'a dyn ToSql)> {
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
        let tx = connection.transaction().map_err(|err| {
            format_lasm_sqlite_runtime_error("sqlite execution transaction start failed", &err)
        })?;
        if sqlite_params.is_empty() && has_lasm_sql_non_trailing_statement_separator(query_template)
        {
            let before_changes = tx.total_changes();
            tx.execute_batch(query_template)
                .map_err(|err| format_lasm_sqlite_runtime_error("sqlite execution failed", &err))?;
            let affected_rows = tx.total_changes().saturating_sub(before_changes);
            tx.commit().map_err(|err| {
                format_lasm_sqlite_runtime_error("sqlite execution transaction commit failed", &err)
            })?;
            return Ok(affected_rows);
        }
        let mut statement = tx.prepare_cached(query_template).map_err(|err| {
            format_lasm_sqlite_runtime_error("sqlite execution prepare failed", &err)
        })?;
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
                    let named_bindings =
                        resolve_lasm_sqlite_named_param_bindings(&statement, values.as_slice())?;
                    let named_refs = lasm_sqlite_named_param_refs(named_bindings.as_slice());
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
                            .map_err(|err| {
                                format_lasm_sqlite_runtime_error(
                                    "sqlite execution query failed",
                                    &err,
                                )
                            })?,
                        LasmSqliteQueryParams::Named(values) => {
                            let named_bindings = resolve_lasm_sqlite_named_param_bindings(
                                &statement,
                                values.as_slice(),
                            )?;
                            let named_refs =
                                lasm_sqlite_named_param_refs(named_bindings.as_slice());
                            statement.query(named_refs.as_slice()).map_err(|err| {
                                format_lasm_sqlite_runtime_error(
                                    "sqlite execution query failed",
                                    &err,
                                )
                            })?
                        }
                    }
                } else {
                    statement.query([]).map_err(|err| {
                        format_lasm_sqlite_runtime_error("sqlite execution query failed", &err)
                    })?
                };
                let mut row_count = 0u64;
                while rows
                    .next()
                    .map_err(|err| {
                        format_lasm_sqlite_runtime_error("sqlite execution row drain failed", &err)
                    })?
                    .is_some()
                {
                    row_count = row_count.saturating_add(1);
                }
                row_count
            }
            Err(err) => {
                return Err(format_lasm_sqlite_runtime_error(
                    "sqlite execution failed",
                    &err,
                ))
            }
        };
        drop(statement);
        tx.commit().map_err(|err| {
            format_lasm_sqlite_runtime_error("sqlite execution transaction commit failed", &err)
        })?;
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
        return Err(
            "sqlite queryOne requires row-returning SQL statement (SELECT/WITH/VALUES/TABLE or DML ... RETURNING)"
                .to_string(),
        );
    }
    run_lasm_sqlite_with_connection_retry(state, "sqlite queryOne", |connection| {
        let mut statement = connection
            .prepare_cached(normalized_query.as_str())
            .map_err(|err| {
                format_lasm_sqlite_runtime_error("sqlite queryOne prepare failed", &err)
            })?;
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
                    .map_err(|err| {
                        format_lasm_sqlite_runtime_error("sqlite queryOne execution failed", &err)
                    })?,
                LasmSqliteQueryParams::Named(values) => {
                    let named_bindings =
                        resolve_lasm_sqlite_named_param_bindings(&statement, values.as_slice())?;
                    let named_refs = lasm_sqlite_named_param_refs(named_bindings.as_slice());
                    statement.query(named_refs.as_slice()).map_err(|err| {
                        format_lasm_sqlite_runtime_error("sqlite queryOne execution failed", &err)
                    })?
                }
            }
        } else {
            statement.query([]).map_err(|err| {
                format_lasm_sqlite_runtime_error("sqlite queryOne execution failed", &err)
            })?
        };
        let Some(row) = rows.next().map_err(|err| {
            format_lasm_sqlite_runtime_error("sqlite queryOne row fetch failed", &err)
        })?
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
                .map_err(|err| {
                    format_lasm_sqlite_runtime_error("sqlite queryOne row decode failed", &err)
                })?;
            object.insert(name, value);
        }
        Ok(Some(serde_json::Value::Object(object)))
    })
}

#[cfg(test)]
mod tests {
    use super::{
        parse_lasm_sqlite_query_params, resolve_lasm_sqlite_named_param_bindings,
        validate_lasm_sqlite_parameter_arity, LasmSqliteQueryParams,
    };
    use rusqlite::types::Value as SqliteValue;
    use rusqlite::Connection;

    #[test]
    fn positional_object_params_expand_with_null_fill() {
        let params =
            parse_lasm_sqlite_query_params("{\"2\":5}").expect("positional params should parse");
        let LasmSqliteQueryParams::Positional(values) = params else {
            panic!("expected positional params");
        };
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], SqliteValue::Null);
        assert_eq!(values[1], SqliteValue::Integer(5));
    }

    #[test]
    fn non_numeric_object_params_fall_back_to_single_text_param() {
        let params = parse_lasm_sqlite_query_params("{\"user\":\"alice\"}")
            .expect("named params should parse");
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
        let params =
            parse_lasm_sqlite_query_params("{\"?3\":7}").expect("positional params should parse");
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
        let params = parse_lasm_sqlite_query_params("{\"name\":\"alice\",\"@role\":\"admin\"}")
            .expect("named params should parse");
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
    fn invalid_named_object_params_return_validation_error() {
        let error = parse_lasm_sqlite_query_params("{\"user-name\":\"alice\"}")
            .expect_err("invalid keys should fail");
        assert!(error.contains("sqlite params object keys must be all positional"));
    }

    #[test]
    fn mixed_object_params_return_validation_error() {
        let error = parse_lasm_sqlite_query_params("{\"1\":\"alice\",\"name\":\"bob\"}")
            .expect_err("mixed positional and named keys should fail");
        assert!(error.contains("sqlite params object keys must be all positional"));
    }

    #[test]
    fn named_object_params_reject_duplicate_normalized_keys() {
        let error = parse_lasm_sqlite_query_params("{\":name\":\"alice\",\"name\":\"bob\"}")
            .expect_err("duplicate normalized named keys should fail");
        assert!(error.contains("sqlite params object contains duplicate normalized key `name`"));
    }

    #[test]
    fn named_bindings_match_statement_prefix_variants() {
        let params = parse_lasm_sqlite_query_params("{\"name\":\"alice\",\"role\":\"admin\"}")
            .expect("named params should parse");
        let LasmSqliteQueryParams::Named(values) = params else {
            panic!("expected named params");
        };
        let connection = Connection::open_in_memory().expect("sqlite in-memory connection");
        let statement = connection
            .prepare("SELECT :name, @role")
            .expect("statement should prepare");
        let bindings = resolve_lasm_sqlite_named_param_bindings(&statement, values.as_slice())
            .expect("bindings should resolve");
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].0, ":name");
        assert_eq!(bindings[1].0, "@role");
    }

    #[test]
    fn named_bindings_fail_when_required_param_is_missing() {
        let params = parse_lasm_sqlite_query_params("{\"name\":\"alice\"}")
            .expect("named params should parse");
        let LasmSqliteQueryParams::Named(values) = params else {
            panic!("expected named params");
        };
        let connection = Connection::open_in_memory().expect("sqlite in-memory connection");
        let statement = connection
            .prepare("SELECT :name, :role")
            .expect("statement should prepare");
        let error = resolve_lasm_sqlite_named_param_bindings(&statement, values.as_slice())
            .expect_err("missing named param should fail");
        assert!(error.contains("requires named parameter `role` in params object"));
    }

    #[test]
    fn named_bindings_fail_when_extra_param_is_provided() {
        let params = parse_lasm_sqlite_query_params("{\"name\":\"alice\",\"role\":\"admin\"}")
            .expect("named params should parse");
        let LasmSqliteQueryParams::Named(values) = params else {
            panic!("expected named params");
        };
        let connection = Connection::open_in_memory().expect("sqlite in-memory connection");
        let statement = connection
            .prepare("SELECT :name")
            .expect("statement should prepare");
        let error = resolve_lasm_sqlite_named_param_bindings(&statement, values.as_slice())
            .expect_err("extra named param should fail");
        assert!(error.contains("query parameter `role` is not present in SQL statement"));
    }

    #[test]
    fn positional_arity_validation_rejects_extra_params() {
        let error =
            validate_lasm_sqlite_parameter_arity(1, 2).expect_err("extra params should fail");
        assert!(error.contains("expects exactly 1 sql parameters but received 2"));
    }

    #[test]
    fn positional_arity_validation_allows_extra_params_when_sql_has_no_placeholders() {
        validate_lasm_sqlite_parameter_arity(0, 2)
            .expect("zero-placeholder sql should ignore extra params for compatibility");
    }
}
