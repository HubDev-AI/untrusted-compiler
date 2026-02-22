use crate::lasm_db_runtime_common::{
    insert_lasm_bounded_cache_entry, lasm_dynamic_postgres_client_mut,
    lasm_dynamic_postgres_prepared_statement, reconnect_lasm_dynamic_postgres_client,
};
use crate::{has_lasm_sql_non_trailing_statement_separator, LasmDynamicResponseState};
use postgres::types::ToSql;
use postgres::{Client as PostgresClient, Statement as PostgresStatement};

pub(crate) enum LasmPostgresParam {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null(Option<String>),
}

fn parse_lasm_postgres_query_param_value(value: serde_json::Value) -> LasmPostgresParam {
    match value {
        serde_json::Value::String(inner) => LasmPostgresParam::Text(inner),
        serde_json::Value::Number(inner) => {
            if let Some(value) = inner.as_i64() {
                return LasmPostgresParam::Int(value);
            }
            if let Some(value) = inner.as_f64() {
                return LasmPostgresParam::Float(value);
            }
            LasmPostgresParam::Text(inner.to_string())
        }
        serde_json::Value::Bool(inner) => LasmPostgresParam::Bool(inner),
        serde_json::Value::Null => LasmPostgresParam::Null(None),
        other => LasmPostgresParam::Text(serde_json::to_string(&other).unwrap_or_default()),
    }
}

pub(crate) fn parse_lasm_postgres_query_params(value: &str) -> Vec<LasmPostgresParam> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return Vec::new();
    }
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return match parsed {
            serde_json::Value::Array(entries) => entries
                .into_iter()
                .map(parse_lasm_postgres_query_param_value)
                .collect(),
            serde_json::Value::Null => Vec::new(),
            other => vec![parse_lasm_postgres_query_param_value(other)],
        };
    }
    vec![LasmPostgresParam::Text(trimmed.to_string())]
}

fn lasm_postgres_query_param_refs(params: &[LasmPostgresParam]) -> Vec<&(dyn ToSql + Sync)> {
    params
        .iter()
        .map(|value| match value {
            LasmPostgresParam::Text(inner) => inner as &(dyn ToSql + Sync),
            LasmPostgresParam::Int(inner) => inner as &(dyn ToSql + Sync),
            LasmPostgresParam::Float(inner) => inner as &(dyn ToSql + Sync),
            LasmPostgresParam::Bool(inner) => inner as &(dyn ToSql + Sync),
            LasmPostgresParam::Null(inner) => inner as &(dyn ToSql + Sync),
        })
        .collect()
}

fn parse_lasm_postgres_dollar_quote_delimiter<'a>(
    query_template: &'a str,
    index: usize,
) -> Option<&'a str> {
    let bytes = query_template.as_bytes();
    if index >= bytes.len() || bytes[index] != b'$' {
        return None;
    }
    let mut cursor = index + 1;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if byte == b'$' {
            return Some(&query_template[index..=cursor]);
        }
        if !(byte == b'_' || byte.is_ascii_alphanumeric()) {
            return None;
        }
        cursor += 1;
    }
    None
}

fn max_lasm_postgres_placeholder_index(query_template: &str) -> usize {
    let bytes = query_template.as_bytes();
    let mut index = 0usize;
    let mut max_placeholder = 0usize;
    let mut in_single_quote = false;
    let mut active_dollar_quote: Option<String> = None;
    let mut in_line_comment = false;
    let mut block_comment_depth = 0usize;
    while index < bytes.len() {
        if in_line_comment {
            if bytes[index] == b'\n' {
                in_line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment_depth > 0 {
            if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
                block_comment_depth += 1;
                index += 2;
                continue;
            }
            if index + 1 < bytes.len() && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                block_comment_depth = block_comment_depth.saturating_sub(1);
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        if let Some(delimiter) = active_dollar_quote.as_ref() {
            if query_template[index..].starts_with(delimiter.as_str()) {
                index += delimiter.len();
                active_dollar_quote = None;
                continue;
            }
            index += 1;
            continue;
        }
        if bytes[index] == b'\'' {
            if in_single_quote {
                if index + 1 < bytes.len() && bytes[index + 1] == b'\'' {
                    index += 2;
                    continue;
                }
                in_single_quote = false;
                index += 1;
                continue;
            }
            in_single_quote = true;
            index += 1;
            continue;
        }
        if in_single_quote {
            index += 1;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'-' && bytes[index + 1] == b'-' {
            in_line_comment = true;
            index += 2;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            block_comment_depth = 1;
            index += 2;
            continue;
        }
        if bytes[index] == b'$' {
            if let Some(delimiter) =
                parse_lasm_postgres_dollar_quote_delimiter(query_template, index)
            {
                active_dollar_quote = Some(delimiter.to_string());
                index += delimiter.len();
                continue;
            }
            let mut cursor = index + 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            if cursor > index + 1 {
                if let Ok(value) = query_template[index + 1..cursor].parse::<usize>() {
                    max_placeholder = max_placeholder.max(value);
                }
            }
            index = cursor;
            continue;
        }
        index += 1;
    }
    max_placeholder
}

fn max_lasm_postgres_placeholder_index_cached(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
) -> usize {
    if let Some(value) = state.db_postgres_placeholder_max_cache.get(query_template) {
        return *value;
    }
    let value = max_lasm_postgres_placeholder_index(query_template);
    let evicted = insert_lasm_bounded_cache_entry(
        &mut state.db_postgres_placeholder_max_cache,
        &mut state.db_postgres_placeholder_max_cache_order,
        state.db_postgres_placeholder_cache_max,
        query_template.to_string(),
        value,
    );
    state.db_postgres_placeholder_cache_evictions_total = state
        .db_postgres_placeholder_cache_evictions_total
        .saturating_add(evicted);
    value
}

fn first_lasm_postgres_keyword(query_template: &str) -> Option<String> {
    let bytes = query_template.as_bytes();
    let mut index = 0usize;
    let mut in_line_comment = false;
    let mut block_comment_depth = 0usize;
    while index < bytes.len() {
        if in_line_comment {
            if bytes[index] == b'\n' {
                in_line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment_depth > 0 {
            if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
                block_comment_depth += 1;
                index += 2;
                continue;
            }
            if index + 1 < bytes.len() && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                block_comment_depth = block_comment_depth.saturating_sub(1);
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'-' && bytes[index + 1] == b'-' {
            in_line_comment = true;
            index += 2;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            block_comment_depth = 1;
            index += 2;
            continue;
        }
        break;
    }
    if index >= bytes.len() {
        return None;
    }
    let start = index;
    while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_') {
        index += 1;
    }
    if index <= start {
        return None;
    }
    Some(query_template[start..index].to_ascii_uppercase())
}

pub(crate) fn is_lasm_postgres_query_one_select_like(query_template: &str) -> bool {
    matches!(
        first_lasm_postgres_keyword(query_template).as_deref(),
        Some("SELECT" | "WITH" | "VALUES" | "TABLE")
    )
}

pub(crate) fn normalize_lasm_postgres_query_for_subquery(query_template: &str) -> String {
    let mut normalized = query_template.trim().to_string();
    while normalized.ends_with(';') {
        normalized.pop();
        normalized = normalized.trim_end().to_string();
    }
    normalized
}

pub(crate) fn run_lasm_postgres_exec(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
    params: &[LasmPostgresParam],
) -> Result<u64, String> {
    let required_params = max_lasm_postgres_placeholder_index_cached(state, query_template);
    if required_params > params.len() {
        return Err(format!(
            "postgres query requires at least {required_params} sql parameters but received {}",
            params.len()
        ));
    }
    let use_prepared = required_params > 0 || !params.is_empty();
    if use_prepared && has_lasm_sql_non_trailing_statement_separator(query_template) {
        return Err("postgres parameterized execution requires a single SQL statement".to_string());
    }
    let prepared_statement = if use_prepared {
        Some(lasm_dynamic_postgres_prepared_statement(
            state,
            query_template,
        )?)
    } else {
        None
    };
    let initial = if use_prepared {
        let param_refs = lasm_postgres_query_param_refs(params);
        let client = lasm_dynamic_postgres_client_mut(state)?;
        let statement = prepared_statement
            .as_ref()
            .expect("prepared statement should be available for prepared execution");
        client.execute(statement, param_refs.as_slice())
    } else {
        let client = lasm_dynamic_postgres_client_mut(state)?;
        client.batch_execute(query_template).map(|_| 0u64)
    };
    let affected_rows = match initial {
        Ok(count) => count,
        Err(err) if err.is_closed() => {
            reconnect_lasm_dynamic_postgres_client(state)?;
            if use_prepared {
                let retry_statement =
                    lasm_dynamic_postgres_prepared_statement(state, query_template)?;
                let param_refs = lasm_postgres_query_param_refs(params);
                let client = lasm_dynamic_postgres_client_mut(state)?;
                client
                    .execute(&retry_statement, param_refs.as_slice())
                    .map_err(|retry_err| {
                        format!("postgres execution failed after reconnect: {retry_err}")
                    })?
            } else {
                let client = lasm_dynamic_postgres_client_mut(state)?;
                client
                    .batch_execute(query_template)
                    .map(|_| 0u64)
                    .map_err(|retry_err| {
                        format!("postgres execution failed after reconnect: {retry_err}")
                    })?
            }
        }
        Err(err)
            if use_prepared
                && err
                    .to_string()
                    .contains("cannot insert multiple commands into a prepared statement") =>
        {
            return Err(
                "postgres parameterized execution requires a single SQL statement".to_string(),
            )
        }
        Err(err) => return Err(format!("postgres execution failed: {err}")),
    };
    Ok(affected_rows)
}

fn run_lasm_postgres_exec_tx_once(
    client: &mut PostgresClient,
    query_template: &str,
    params: &[LasmPostgresParam],
    prepared_statement: Option<&PostgresStatement>,
) -> Result<u64, postgres::Error> {
    let mut tx = client.transaction()?;
    let affected_rows = if let Some(statement) = prepared_statement {
        let param_refs = lasm_postgres_query_param_refs(params);
        tx.execute(statement, param_refs.as_slice())?
    } else {
        tx.batch_execute(query_template)?;
        0
    };
    tx.commit()?;
    Ok(affected_rows)
}

pub(crate) fn run_lasm_postgres_exec_tx(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
    params: &[LasmPostgresParam],
) -> Result<u64, String> {
    let required_params = max_lasm_postgres_placeholder_index_cached(state, query_template);
    if required_params > params.len() {
        return Err(format!(
            "postgres query requires at least {required_params} sql parameters but received {}",
            params.len()
        ));
    }
    let use_prepared = required_params > 0 || !params.is_empty();
    if use_prepared && has_lasm_sql_non_trailing_statement_separator(query_template) {
        return Err("postgres parameterized execution requires a single SQL statement".to_string());
    }
    let prepared_statement = if use_prepared {
        Some(lasm_dynamic_postgres_prepared_statement(
            state,
            query_template,
        )?)
    } else {
        None
    };
    let initial = {
        let client = lasm_dynamic_postgres_client_mut(state)?;
        run_lasm_postgres_exec_tx_once(client, query_template, params, prepared_statement.as_ref())
    };
    let affected_rows = match initial {
        Ok(count) => count,
        Err(err) if err.is_closed() => {
            reconnect_lasm_dynamic_postgres_client(state)?;
            let retry_statement = if use_prepared {
                Some(lasm_dynamic_postgres_prepared_statement(
                    state,
                    query_template,
                )?)
            } else {
                None
            };
            let client = lasm_dynamic_postgres_client_mut(state)?;
            run_lasm_postgres_exec_tx_once(client, query_template, params, retry_statement.as_ref())
                .map_err(|retry_err| {
                    format!("postgres transaction execution failed after reconnect: {retry_err}")
                })?
        }
        Err(err)
            if use_prepared
                && err
                    .to_string()
                    .contains("cannot insert multiple commands into a prepared statement") =>
        {
            return Err(
                "postgres parameterized execution requires a single SQL statement".to_string(),
            )
        }
        Err(err) => return Err(format!("postgres transaction execution failed: {err}")),
    };
    Ok(affected_rows)
}

pub(crate) fn run_lasm_postgres_query_one(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
    params: &[LasmPostgresParam],
) -> Result<Option<serde_json::Value>, String> {
    let normalized_query = normalize_lasm_postgres_query_for_subquery(query_template);
    if normalized_query.trim().is_empty() {
        return Err("postgres queryOne requires non-empty SQL statement".to_string());
    }
    if !is_lasm_postgres_query_one_select_like(normalized_query.as_str()) {
        return Err("postgres queryOne requires SELECT-style SQL statement".to_string());
    }
    if has_lasm_sql_non_trailing_statement_separator(normalized_query.as_str()) {
        return Err("postgres parameterized execution requires a single SQL statement".to_string());
    }
    let required_params =
        max_lasm_postgres_placeholder_index_cached(state, normalized_query.as_str());
    if required_params > params.len() {
        return Err(format!(
            "postgres query requires at least {required_params} sql parameters but received {}",
            params.len()
        ));
    }
    let wrapped_query = format!(
        "SELECT row_to_json(_sec4_row)::text AS __sec4_row \
         FROM ({}) AS _sec4_row LIMIT 1",
        normalized_query
    );
    let prepared_statement =
        lasm_dynamic_postgres_prepared_statement(state, wrapped_query.as_str())?;
    let execute_query = |client: &mut PostgresClient,
                         statement: &PostgresStatement|
     -> Result<Option<postgres::Row>, postgres::Error> {
        let param_refs = lasm_postgres_query_param_refs(params);
        client.query_opt(statement, param_refs.as_slice())
    };
    let row = {
        let initial = {
            let client = lasm_dynamic_postgres_client_mut(state)?;
            execute_query(client, &prepared_statement)
        };
        match initial {
            Ok(row) => row,
            Err(err) if err.is_closed() => {
                reconnect_lasm_dynamic_postgres_client(state)?;
                let retry_statement =
                    lasm_dynamic_postgres_prepared_statement(state, wrapped_query.as_str())?;
                let client = lasm_dynamic_postgres_client_mut(state)?;
                execute_query(client, &retry_statement).map_err(|retry_err| {
                    format!("postgres queryOne execution failed after reconnect: {retry_err}")
                })?
            }
            Err(err)
                if err
                    .to_string()
                    .contains("cannot insert multiple commands into a prepared statement") =>
            {
                return Err(
                    "postgres parameterized execution requires a single SQL statement".to_string(),
                )
            }
            Err(err) => return Err(format!("postgres queryOne execution failed: {err}")),
        }
    };
    let Some(row) = row else {
        return Ok(None);
    };
    let row_payload: Option<String> = row
        .try_get(0)
        .map_err(|err| format!("postgres queryOne row materialization failed: {err}"))?;
    let row_payload = row_payload.unwrap_or_else(|| "null".to_string());
    let row_json = serde_json::from_str::<serde_json::Value>(row_payload.as_str())
        .map_err(|err| format!("postgres queryOne row json decode failed: {err}"))?;
    Ok(Some(row_json))
}

#[cfg(test)]
mod tests {
    use super::max_lasm_postgres_placeholder_index_cached;
    use crate::LasmDynamicResponseState;

    #[test]
    fn placeholder_cache_eviction_counter_increments_when_capacity_is_hit() {
        let mut state = LasmDynamicResponseState {
            db_postgres_placeholder_cache_max: 1,
            ..Default::default()
        };
        let first = max_lasm_postgres_placeholder_index_cached(&mut state, "SELECT $1");
        assert_eq!(first, 1);
        assert_eq!(state.db_postgres_placeholder_cache_evictions_total, 0);
        assert_eq!(state.db_postgres_placeholder_max_cache.len(), 1);

        let second = max_lasm_postgres_placeholder_index_cached(&mut state, "SELECT $2");
        assert_eq!(second, 2);
        assert_eq!(state.db_postgres_placeholder_cache_evictions_total, 1);
        assert_eq!(state.db_postgres_placeholder_max_cache.len(), 1);
    }
}
