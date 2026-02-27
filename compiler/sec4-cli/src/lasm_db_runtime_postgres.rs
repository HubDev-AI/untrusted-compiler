use crate::lasm_db_adapter_state::{
    connect_lasm_dynamic_db_records_postgres, ensure_lasm_dynamic_db_records_postgres_schema,
    LasmDbPostgresTlsMode,
};
use crate::lasm_db_runtime_common::{
    insert_lasm_bounded_cache_entry, lasm_dynamic_postgres_client_mut,
    lasm_dynamic_postgres_prepared_statement, reconnect_lasm_dynamic_postgres_client,
    touch_lasm_bounded_cache_entry,
};
use crate::{
    has_lasm_sql_non_trailing_statement_separator, LasmDbRecord, LasmDynamicResponseState,
    LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE,
};
use postgres::types::ToSql;
use postgres::{Client as PostgresClient, GenericClient, Statement as PostgresStatement};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

#[derive(Clone)]
pub(crate) enum LasmPostgresParam {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null(Option<String>),
}

#[derive(Clone)]
pub(crate) struct LasmPostgresThreadLocalConfig {
    pub(crate) dsn: String,
    pub(crate) tls_mode: LasmDbPostgresTlsMode,
    pub(crate) statement_timeout_ms: u64,
    pub(crate) lock_timeout_ms: u64,
    pub(crate) connect_timeout_ms: u64,
    pub(crate) retryable_conflict_retry_max: usize,
}

const LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_ENV: &str =
    "SEC4_RT_LASM_DB_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY";
const LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_DEFAULT: usize = 16;
const LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_MIN: usize = 1;
const LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_MAX: usize = 256;
const LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_ENV: &str =
    "SEC4_RT_LASM_DB_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE";
const LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_DEFAULT: usize = 128;
const LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_MIN: usize = 1;
const LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_MAX: usize = 4096;
static LASM_POSTGRES_SHARED_CLIENTS: OnceLock<Mutex<HashMap<String, Vec<PostgresClient>>>> =
    OnceLock::new();
static LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_RESOLVED: OnceLock<usize> = OnceLock::new();
static LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_RESOLVED: OnceLock<usize> = OnceLock::new();

fn resolve_lasm_postgres_shared_client_max_idle_per_key() -> usize {
    *LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_RESOLVED.get_or_init(|| {
        let Ok(raw) = std::env::var(LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_ENV) else {
            return LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_DEFAULT;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_DEFAULT;
        }
        let Ok(parsed) = trimmed.parse::<usize>() else {
            return LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_DEFAULT;
        };
        parsed.clamp(
            LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_MIN,
            LASM_POSTGRES_SHARED_CLIENT_MAX_IDLE_PER_KEY_MAX,
        )
    })
}

fn resolve_lasm_postgres_shared_client_max_total_idle() -> usize {
    *LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_RESOLVED.get_or_init(|| {
        let Ok(raw) = std::env::var(LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_ENV) else {
            return LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_DEFAULT;
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_DEFAULT;
        }
        let Ok(parsed) = trimmed.parse::<usize>() else {
            return LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_DEFAULT;
        };
        parsed.clamp(
            LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_MIN,
            LASM_POSTGRES_SHARED_CLIENT_MAX_TOTAL_IDLE_MAX,
        )
    })
}

pub(crate) fn lasm_postgres_shared_client_max_idle_per_key() -> usize {
    resolve_lasm_postgres_shared_client_max_idle_per_key()
}

pub(crate) fn lasm_postgres_shared_client_max_total_idle() -> usize {
    resolve_lasm_postgres_shared_client_max_total_idle()
}

pub(crate) fn lasm_postgres_shared_client_pool_key_count() -> usize {
    let pool = LASM_POSTGRES_SHARED_CLIENTS.get_or_init(|| Mutex::new(HashMap::new()));
    match pool.lock() {
        Ok(guard) => guard.len(),
        Err(_) => 0,
    }
}

pub(crate) fn lasm_postgres_shared_client_pool_idle_total() -> usize {
    let pool = LASM_POSTGRES_SHARED_CLIENTS.get_or_init(|| Mutex::new(HashMap::new()));
    match pool.lock() {
        Ok(guard) => guard.values().map(Vec::len).sum(),
        Err(_) => 0,
    }
}

fn lasm_postgres_shared_pool_idle_total_locked(
    pool: &HashMap<String, Vec<PostgresClient>>,
) -> usize {
    pool.values().map(Vec::len).sum()
}

fn parse_lasm_postgres_query_param_value(value: serde_json::Value) -> LasmPostgresParam {
    match value {
        serde_json::Value::String(inner) => LasmPostgresParam::Text(inner),
        serde_json::Value::Number(inner) => {
            if let Some(value) = inner.as_i64() {
                return LasmPostgresParam::Int(value);
            }
            if let Some(value) = inner.as_u64() {
                if value <= i64::MAX as u64 {
                    return LasmPostgresParam::Int(value as i64);
                }
                return LasmPostgresParam::Text(inner.to_string());
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

fn parse_lasm_postgres_positional_object_params(
    entries: &serde_json::Map<String, serde_json::Value>,
) -> Vec<LasmPostgresParam> {
    let mut indexed = Vec::with_capacity(entries.len());
    let mut max_index = 0usize;
    for (key, value) in entries {
        let index = parse_lasm_postgres_positional_param_index(key.as_str())
            .expect("positional object keys should be validated before parsing");
        max_index = max_index.max(index);
        indexed.push((index, parse_lasm_postgres_query_param_value(value.clone())));
    }
    let mut params = Vec::with_capacity(max_index);
    for _ in 0..max_index {
        params.push(LasmPostgresParam::Null(None));
    }
    for (index, value) in indexed {
        params[index - 1] = value;
    }
    params
}

fn parse_lasm_postgres_positional_param_index(key: &str) -> Option<usize> {
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

fn is_lasm_postgres_named_param_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn normalize_lasm_postgres_named_param_key(key: &str) -> Option<&str> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = trimmed
        .strip_prefix(':')
        .or_else(|| trimmed.strip_prefix('@'))
        .or_else(|| {
            let candidate = trimmed.strip_prefix('$')?;
            if candidate.chars().all(|ch| ch.is_ascii_digit()) {
                return None;
            }
            Some(candidate)
        })
        .unwrap_or(trimmed)
        .trim();
    if is_lasm_postgres_named_param_identifier(normalized) {
        Some(normalized)
    } else {
        None
    }
}

fn parse_lasm_postgres_named_object_params(
    entries: &serde_json::Map<String, serde_json::Value>,
) -> Result<HashMap<String, LasmPostgresParam>, String> {
    let mut named = HashMap::with_capacity(entries.len());
    for (key, value) in entries {
        let normalized = normalize_lasm_postgres_named_param_key(key.as_str())
            .expect("named object keys should be validated before parsing");
        let normalized = normalized.to_string();
        let value = parse_lasm_postgres_query_param_value(value.clone());
        if named.insert(normalized.clone(), value).is_some() {
            return Err(format!(
                "postgres params object contains duplicate normalized key `{normalized}`"
            ));
        }
    }
    Ok(named)
}

enum LasmPostgresParamsObjectKeyStyle {
    Positional,
    Named,
}

fn classify_lasm_postgres_params_object_keys(
    entries: &serde_json::Map<String, serde_json::Value>,
) -> Result<LasmPostgresParamsObjectKeyStyle, String> {
    let mut positional_all = true;
    let mut named_all = true;
    for key in entries.keys() {
        positional_all &= parse_lasm_postgres_positional_param_index(key.as_str()).is_some();
        named_all &= normalize_lasm_postgres_named_param_key(key.as_str()).is_some();
    }
    if positional_all {
        return Ok(LasmPostgresParamsObjectKeyStyle::Positional);
    }
    if named_all {
        return Ok(LasmPostgresParamsObjectKeyStyle::Named);
    }
    Err(
        "postgres params object keys must be all positional ($1/?1/1) or all named (:name/@name/$name/name)"
            .to_string(),
    )
}

fn rewrite_lasm_postgres_named_query_template(
    query_template: &str,
    named_params: &HashMap<String, LasmPostgresParam>,
) -> Result<Option<(String, Vec<LasmPostgresParam>)>, String> {
    let bytes = query_template.as_bytes();
    let mut index = 0usize;
    let mut rewritten = Vec::with_capacity(bytes.len() + 16);
    let mut in_single_quote = false;
    let mut in_line_comment = false;
    let mut block_comment_depth = 0usize;
    let mut active_dollar_quote: Option<String> = None;
    let mut named_placeholder_indices: HashMap<String, usize> = HashMap::new();
    let mut ordered_params = Vec::new();
    let mut saw_named_placeholder = false;
    let mut saw_positional_placeholder = false;

    while index < bytes.len() {
        if in_line_comment {
            rewritten.push(bytes[index]);
            if bytes[index] == b'\n' {
                in_line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment_depth > 0 {
            if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
                block_comment_depth += 1;
                rewritten.extend_from_slice(&bytes[index..index + 2]);
                index += 2;
                continue;
            }
            if index + 1 < bytes.len() && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                block_comment_depth = block_comment_depth.saturating_sub(1);
                rewritten.extend_from_slice(&bytes[index..index + 2]);
                index += 2;
                continue;
            }
            rewritten.push(bytes[index]);
            index += 1;
            continue;
        }
        if let Some(delimiter) = active_dollar_quote.as_ref() {
            if query_template[index..].starts_with(delimiter.as_str()) {
                rewritten.extend_from_slice(delimiter.as_bytes());
                index += delimiter.len();
                active_dollar_quote = None;
                continue;
            }
            rewritten.push(bytes[index]);
            index += 1;
            continue;
        }
        if bytes[index] == b'\'' {
            if in_single_quote {
                if index + 1 < bytes.len() && bytes[index + 1] == b'\'' {
                    rewritten.extend_from_slice(&bytes[index..index + 2]);
                    index += 2;
                    continue;
                }
                in_single_quote = false;
                rewritten.push(bytes[index]);
                index += 1;
                continue;
            }
            in_single_quote = true;
            rewritten.push(bytes[index]);
            index += 1;
            continue;
        }
        if in_single_quote {
            rewritten.push(bytes[index]);
            index += 1;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'-' && bytes[index + 1] == b'-' {
            in_line_comment = true;
            rewritten.extend_from_slice(&bytes[index..index + 2]);
            index += 2;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            block_comment_depth = 1;
            rewritten.extend_from_slice(&bytes[index..index + 2]);
            index += 2;
            continue;
        }
        if bytes[index] == b'$' {
            if let Some(delimiter) =
                parse_lasm_postgres_dollar_quote_delimiter(query_template, index)
            {
                active_dollar_quote = Some(delimiter.to_string());
                rewritten.extend_from_slice(delimiter.as_bytes());
                index += delimiter.len();
                continue;
            }
        }

        let marker = bytes[index];
        if marker != b':' && marker != b'@' && marker != b'$' {
            rewritten.push(marker);
            index += 1;
            continue;
        }
        if marker == b':' {
            if (index > 0 && bytes[index - 1] == b':')
                || (index + 1 < bytes.len() && bytes[index + 1] == b':')
            {
                rewritten.push(marker);
                index += 1;
                continue;
            }
        }
        if marker == b'$' && index + 1 < bytes.len() && bytes[index + 1].is_ascii_digit() {
            saw_positional_placeholder = true;
            rewritten.push(marker);
            index += 1;
            continue;
        }
        if index + 1 >= bytes.len() {
            rewritten.push(marker);
            index += 1;
            continue;
        }
        let next = bytes[index + 1];
        if !(next == b'_' || next.is_ascii_alphabetic()) {
            rewritten.push(marker);
            index += 1;
            continue;
        }
        let mut cursor = index + 2;
        while cursor < bytes.len() {
            let byte = bytes[cursor];
            if !(byte == b'_' || byte.is_ascii_alphanumeric()) {
                break;
            }
            cursor += 1;
        }
        let name = &query_template[index + 1..cursor];
        let Some(param_value) = named_params.get(name) else {
            return Err(format!(
                "postgres named parameter '{name}' is missing from params object"
            ));
        };
        saw_named_placeholder = true;
        let placeholder_index = if let Some(existing) = named_placeholder_indices.get(name) {
            *existing
        } else {
            ordered_params.push(param_value.clone());
            let next_index = ordered_params.len();
            named_placeholder_indices.insert(name.to_string(), next_index);
            next_index
        };
        rewritten.extend_from_slice(format!("${placeholder_index}").as_bytes());
        index = cursor;
    }

    if !saw_named_placeholder {
        return Ok(None);
    }
    if saw_positional_placeholder {
        return Err(
            "postgres named parameterized execution does not support mixing named and positional SQL placeholders"
                .to_string(),
        );
    }
    let mut unused_params: Vec<&str> = named_params
        .keys()
        .filter_map(|name| {
            if named_placeholder_indices.contains_key(name.as_str()) {
                None
            } else {
                Some(name.as_str())
            }
        })
        .collect();
    unused_params.sort_unstable();
    if let Some(unused_name) = unused_params.first() {
        return Err(format!(
            "postgres query parameter `{unused_name}` is not present in SQL statement"
        ));
    }
    let rewritten_query = String::from_utf8(rewritten)
        .map_err(|_| "postgres named parameter rewrite produced invalid UTF-8".to_string())?;
    Ok(Some((rewritten_query, ordered_params)))
}

pub(crate) fn parse_lasm_postgres_query_template_and_params(
    query_template: &str,
    value: &str,
) -> Result<(String, Vec<LasmPostgresParam>), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return Ok((query_template.to_string(), Vec::new()));
    }
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return parse_lasm_postgres_query_template_and_params_value(query_template, &parsed);
    }
    Ok((
        query_template.to_string(),
        vec![LasmPostgresParam::Text(trimmed.to_string())],
    ))
}

pub(crate) fn parse_lasm_postgres_query_template_and_params_value(
    query_template: &str,
    parsed: &serde_json::Value,
) -> Result<(String, Vec<LasmPostgresParam>), String> {
    match parsed {
        serde_json::Value::Array(entries) => Ok((
            query_template.to_string(),
            entries
                .iter()
                .cloned()
                .map(parse_lasm_postgres_query_param_value)
                .collect(),
        )),
        serde_json::Value::Object(entries) => {
            match classify_lasm_postgres_params_object_keys(entries)? {
                LasmPostgresParamsObjectKeyStyle::Positional => Ok((
                    query_template.to_string(),
                    parse_lasm_postgres_positional_object_params(entries),
                )),
                LasmPostgresParamsObjectKeyStyle::Named => {
                    let named = parse_lasm_postgres_named_object_params(entries)?;
                    if let Some((rewritten_template, params)) =
                        rewrite_lasm_postgres_named_query_template(query_template, &named)?
                    {
                        Ok((rewritten_template, params))
                    } else {
                        Err(
                        "postgres named parameterized execution requires SQL placeholders to be named (:name, @name, or $name)"
                            .to_string(),
                    )
                    }
                }
            }
        }
        serde_json::Value::Null => Ok((query_template.to_string(), Vec::new())),
        other => Ok((
            query_template.to_string(),
            vec![parse_lasm_postgres_query_param_value(other.clone())],
        )),
    }
}

#[cfg(test)]
pub(crate) fn parse_lasm_postgres_query_params(
    value: &str,
) -> Result<Vec<LasmPostgresParam>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "0" {
        return Ok(Vec::new());
    }
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return match parsed {
            serde_json::Value::Array(entries) => Ok(entries
                .into_iter()
                .map(parse_lasm_postgres_query_param_value)
                .collect()),
            serde_json::Value::Object(entries) => match classify_lasm_postgres_params_object_keys(
                &entries,
            )? {
                LasmPostgresParamsObjectKeyStyle::Positional => {
                    Ok(parse_lasm_postgres_positional_object_params(&entries))
                }
                LasmPostgresParamsObjectKeyStyle::Named => Err(
                    "postgres named params object requires SQL template context for placeholder rewrite"
                        .to_string(),
                ),
            },
            serde_json::Value::Null => Ok(Vec::new()),
            other => Ok(vec![parse_lasm_postgres_query_param_value(other)]),
        };
    }
    Ok(vec![LasmPostgresParam::Text(trimmed.to_string())])
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

#[allow(dead_code)]
fn max_lasm_postgres_placeholder_index_cached(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
) -> usize {
    if let Some(value) = state
        .db_postgres_placeholder_max_cache
        .get(query_template)
        .copied()
    {
        touch_lasm_bounded_cache_entry(
            &mut state.db_postgres_placeholder_max_cache_order,
            query_template,
        );
        return value;
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

fn has_lasm_postgres_keyword(query_template: &str, keyword: &str) -> bool {
    let bytes = query_template.as_bytes();
    let keyword_upper = keyword.to_ascii_uppercase();
    let mut index = 0usize;
    let mut in_single_quote = false;
    let mut in_line_comment = false;
    let mut block_comment_depth = 0usize;
    let mut active_dollar_quote: Option<String> = None;

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
        }
        if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            if query_template[start..index].to_ascii_uppercase() == keyword_upper {
                return true;
            }
            continue;
        }
        index += 1;
    }
    false
}

pub(crate) fn is_lasm_postgres_query_one_select_like(query_template: &str) -> bool {
    match first_lasm_postgres_keyword(query_template).as_deref() {
        Some("SELECT" | "WITH" | "VALUES" | "TABLE") => true,
        Some("INSERT" | "UPDATE" | "DELETE" | "MERGE") => {
            has_lasm_postgres_keyword(query_template, "RETURNING")
        }
        _ => false,
    }
}

pub(crate) fn normalize_lasm_postgres_query_for_subquery(query_template: &str) -> String {
    let mut normalized = query_template.trim().to_string();
    while normalized.ends_with(';') {
        normalized.pop();
        normalized = normalized.trim_end().to_string();
    }
    normalized
}

fn is_lasm_postgres_execute_rows_error(err: &postgres::Error) -> bool {
    let message = err.to_string().to_ascii_lowercase();
    message.contains("query returned rows") || message.contains("execute returned rows")
}

fn is_lasm_postgres_retryable_tx_sqlstate(code: Option<&str>) -> bool {
    matches!(code, Some("40001") | Some("40P01"))
}

fn is_lasm_postgres_retryable_tx_error(err: &postgres::Error) -> bool {
    is_lasm_postgres_retryable_tx_sqlstate(err.code().map(|code| code.code()))
}

fn is_lasm_postgres_reconnectable_sqlstate(code: Option<&str>) -> bool {
    let Some(value) = code else {
        return false;
    };
    let normalized = value.to_ascii_uppercase();
    matches!(normalized.as_str(), "57P01" | "57P02" | "57P03") || normalized.starts_with("08")
}

fn is_lasm_postgres_reconnectable_error_message(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    normalized.contains("connection closed")
        || normalized.contains("connection reset by peer")
        || normalized.contains("server closed the connection unexpectedly")
        || normalized.contains("broken pipe")
        || normalized.contains("terminating connection due to administrator command")
        || normalized.contains("could not connect to server")
}

fn is_lasm_postgres_reconnectable_error(err: &postgres::Error) -> bool {
    err.is_closed()
        || is_lasm_postgres_reconnectable_sqlstate(err.code().map(|code| code.code()))
        || is_lasm_postgres_reconnectable_error_message(err.to_string().as_str())
}

#[allow(dead_code)]
fn is_lasm_postgres_stale_prepared_statement_sqlstate(code: Option<&str>) -> bool {
    matches!(code, Some("26000"))
}

#[allow(dead_code)]
fn is_lasm_postgres_stale_prepared_statement_message(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    (normalized.contains("prepared statement") && normalized.contains("does not exist"))
        || normalized.contains("cached plan must not change result type")
}

#[allow(dead_code)]
fn is_lasm_postgres_stale_prepared_statement_error(err: &postgres::Error) -> bool {
    if is_lasm_postgres_stale_prepared_statement_sqlstate(err.code().map(|code| code.code())) {
        return true;
    }
    is_lasm_postgres_stale_prepared_statement_message(err.to_string().as_str())
}

#[allow(dead_code)]
fn evict_lasm_postgres_prepared_statement(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
) {
    state.db_postgres_stale_plan_reprepare_total = state
        .db_postgres_stale_plan_reprepare_total
        .saturating_add(1);
    state
        .db_records_postgres_statement_cache
        .remove(query_template);
    if let Some(index) = state
        .db_records_postgres_statement_cache_order
        .iter()
        .position(|entry| entry == query_template)
    {
        state
            .db_records_postgres_statement_cache_order
            .remove(index);
    }
}

fn format_lasm_postgres_runtime_error(context: &str, err: &postgres::Error) -> String {
    if let Some(sqlstate) = err.code().map(|code| code.code()) {
        return format!("{context}: {err}; sqlstate={sqlstate}");
    }
    format!("{context}: {err}")
}

#[allow(dead_code)]
fn extract_lasm_postgres_runtime_sqlstate(message: &str) -> Option<String> {
    let normalized = message.to_ascii_lowercase();
    let marker = "sqlstate=";
    let start = normalized.find(marker)? + marker.len();
    let tail = &normalized[start..];
    let end = tail
        .find(|ch: char| !ch.is_ascii_alphanumeric())
        .unwrap_or(tail.len());
    let code = &tail[..end];
    if code.len() == 5 && code.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        Some(code.to_string())
    } else {
        None
    }
}

#[allow(dead_code)]
fn is_lasm_postgres_reconnectable_prepare_error(message: &str) -> bool {
    if let Some(sqlstate) = extract_lasm_postgres_runtime_sqlstate(message) {
        return is_lasm_postgres_reconnectable_sqlstate(Some(sqlstate.as_str()));
    }
    is_lasm_postgres_reconnectable_error_message(message)
}

#[allow(dead_code)]
fn is_lasm_postgres_stale_prepare_error(message: &str) -> bool {
    if let Some(sqlstate) = extract_lasm_postgres_runtime_sqlstate(message) {
        if is_lasm_postgres_stale_prepared_statement_sqlstate(Some(sqlstate.as_str())) {
            return true;
        }
    }
    is_lasm_postgres_stale_prepared_statement_message(message)
}

#[allow(dead_code)]
fn prepare_lasm_postgres_statement_with_reconnect(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
) -> Result<PostgresStatement, String> {
    let mut stale_refresh_attempted = false;
    let mut reconnect_attempted = false;
    let mut previous_message: Option<String> = None;

    loop {
        match lasm_dynamic_postgres_prepared_statement(state, query_template) {
            Ok(statement) => return Ok(statement),
            Err(message)
                if is_lasm_postgres_stale_prepare_error(message.as_str())
                    && !stale_refresh_attempted =>
            {
                stale_refresh_attempted = true;
                previous_message = Some(message);
                evict_lasm_postgres_prepared_statement(state, query_template);
            }
            Err(message)
                if is_lasm_postgres_reconnectable_prepare_error(message.as_str())
                    && !reconnect_attempted =>
            {
                reconnect_attempted = true;
                previous_message = Some(message);
                reconnect_lasm_dynamic_postgres_client(state)?;
            }
            Err(message) => {
                if let Some(previous) = previous_message {
                    return Err(format!(
                        "{previous}; prepare recovery retries failed: {message}"
                    ));
                }
                return Err(message);
            }
        }
    }
}

fn lasm_postgres_retry_backoff_ms(attempt_index: usize) -> u64 {
    (attempt_index as u64).saturating_add(1).saturating_mul(2)
}

fn run_lasm_postgres_unprepared_exec_with_count(
    client: &mut impl GenericClient,
    query_template: &str,
) -> Result<u64, postgres::Error> {
    if has_lasm_sql_non_trailing_statement_separator(query_template) {
        client.batch_execute(query_template)?;
        return Ok(0);
    }
    match client.execute(query_template, &[]) {
        Ok(count) => Ok(count),
        Err(err) if is_lasm_postgres_execute_rows_error(&err) => {
            let rows = client.query(query_template, &[])?;
            Ok(rows.len() as u64)
        }
        Err(err) => Err(err),
    }
}

#[allow(dead_code)]
fn run_lasm_postgres_prepared_exec_with_count(
    client: &mut impl GenericClient,
    statement: &PostgresStatement,
    params: &[LasmPostgresParam],
) -> Result<u64, postgres::Error> {
    let param_refs = lasm_postgres_query_param_refs(params);
    match client.execute(statement, param_refs.as_slice()) {
        Ok(count) => Ok(count),
        Err(err) if is_lasm_postgres_execute_rows_error(&err) => {
            let rows = client.query(statement, param_refs.as_slice())?;
            Ok(rows.len() as u64)
        }
        Err(err) => Err(err),
    }
}

fn validate_lasm_postgres_parameter_arity(
    required_params: usize,
    provided_count: usize,
) -> Result<(), String> {
    if provided_count < required_params {
        return Err(format!(
            "postgres query requires at least {required_params} sql parameters but received {provided_count}"
        ));
    }
    if provided_count > required_params {
        return Err(format!(
            "postgres query expects exactly {required_params} sql parameters but received {provided_count}"
        ));
    }
    Ok(())
}

#[allow(dead_code)]
pub(crate) fn run_lasm_postgres_exec(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
    params: &[LasmPostgresParam],
) -> Result<u64, String> {
    let required_params = max_lasm_postgres_placeholder_index_cached(state, query_template);
    validate_lasm_postgres_parameter_arity(required_params, params.len())?;
    let use_prepared = required_params > 0;
    if use_prepared && has_lasm_sql_non_trailing_statement_separator(query_template) {
        return Err("postgres parameterized execution requires a single SQL statement".to_string());
    }
    let prepared_statement = if use_prepared {
        Some(prepare_lasm_postgres_statement_with_reconnect(
            state,
            query_template,
        )?)
    } else {
        None
    };
    let initial = if use_prepared {
        let client = lasm_dynamic_postgres_client_mut(state)?;
        let statement = prepared_statement
            .as_ref()
            .expect("prepared statement should be available for prepared execution");
        run_lasm_postgres_prepared_exec_with_count(client, statement, params)
    } else {
        let client = lasm_dynamic_postgres_client_mut(state)?;
        run_lasm_postgres_unprepared_exec_with_count(client, query_template)
    };
    let affected_rows = match initial {
        Ok(count) => count,
        Err(err) if is_lasm_postgres_reconnectable_error(&err) => {
            reconnect_lasm_dynamic_postgres_client(state)?;
            if use_prepared {
                let retry_statement =
                    prepare_lasm_postgres_statement_with_reconnect(state, query_template)?;
                let client = lasm_dynamic_postgres_client_mut(state)?;
                run_lasm_postgres_prepared_exec_with_count(client, &retry_statement, params)
                    .map_err(|retry_err| {
                        format_lasm_postgres_runtime_error(
                            "postgres execution failed after reconnect",
                            &retry_err,
                        )
                    })?
            } else {
                let client = lasm_dynamic_postgres_client_mut(state)?;
                run_lasm_postgres_unprepared_exec_with_count(client, query_template).map_err(
                    |retry_err| {
                        format_lasm_postgres_runtime_error(
                            "postgres execution failed after reconnect",
                            &retry_err,
                        )
                    },
                )?
            }
        }
        Err(err) if use_prepared && is_lasm_postgres_stale_prepared_statement_error(&err) => {
            evict_lasm_postgres_prepared_statement(state, query_template);
            let retry_statement =
                prepare_lasm_postgres_statement_with_reconnect(state, query_template)?;
            let client = lasm_dynamic_postgres_client_mut(state)?;
            run_lasm_postgres_prepared_exec_with_count(client, &retry_statement, params).map_err(
                |retry_err| {
                    format_lasm_postgres_runtime_error(
                        "postgres execution failed after stale prepared statement refresh",
                        &retry_err,
                    )
                },
            )?
        }
        Err(err) if is_lasm_postgres_retryable_tx_error(&err) => {
            let _ = err;
            let mut recovered = None;
            for attempt_index in 0..state.db_postgres_retryable_conflict_retry_max {
                state.db_postgres_retryable_conflict_retry_attempts_total = state
                    .db_postgres_retryable_conflict_retry_attempts_total
                    .saturating_add(1);
                let backoff_ms = lasm_postgres_retry_backoff_ms(attempt_index);
                if backoff_ms > 0 {
                    std::thread::sleep(Duration::from_millis(backoff_ms));
                }
                let retry_result = if use_prepared {
                    let retry_statement =
                        prepare_lasm_postgres_statement_with_reconnect(state, query_template)?;
                    let client = lasm_dynamic_postgres_client_mut(state)?;
                    run_lasm_postgres_prepared_exec_with_count(client, &retry_statement, params)
                } else {
                    let client = lasm_dynamic_postgres_client_mut(state)?;
                    run_lasm_postgres_unprepared_exec_with_count(client, query_template)
                };
                match retry_result {
                    Ok(count) => {
                        state.db_postgres_retryable_conflict_retry_success_total = state
                            .db_postgres_retryable_conflict_retry_success_total
                            .saturating_add(1);
                        recovered = Some(count);
                        break;
                    }
                    Err(err) if is_lasm_postgres_retryable_tx_error(&err) => {
                        let _ = err;
                    }
                    Err(err)
                        if use_prepared
                            && is_lasm_postgres_stale_prepared_statement_error(&err) =>
                    {
                        evict_lasm_postgres_prepared_statement(state, query_template);
                    }
                    Err(err) if is_lasm_postgres_reconnectable_error(&err) => {
                        reconnect_lasm_dynamic_postgres_client(state)?;
                    }
                    Err(err) => {
                        return Err(format_lasm_postgres_runtime_error(
                            "postgres execution failed after retryable conflict retry",
                            &err,
                        ));
                    }
                }
            }
            if let Some(count) = recovered {
                count
            } else {
                reconnect_lasm_dynamic_postgres_client(state)?;
                state.db_postgres_retryable_conflict_retry_attempts_total = state
                    .db_postgres_retryable_conflict_retry_attempts_total
                    .saturating_add(1);
                let final_retry = if use_prepared {
                    let retry_statement =
                        prepare_lasm_postgres_statement_with_reconnect(state, query_template)?;
                    let client = lasm_dynamic_postgres_client_mut(state)?;
                    run_lasm_postgres_prepared_exec_with_count(client, &retry_statement, params)
                } else {
                    let client = lasm_dynamic_postgres_client_mut(state)?;
                    run_lasm_postgres_unprepared_exec_with_count(client, query_template)
                };
                match final_retry {
                    Ok(count) => {
                        state.db_postgres_retryable_conflict_retry_success_total = state
                            .db_postgres_retryable_conflict_retry_success_total
                            .saturating_add(1);
                        count
                    }
                    Err(err)
                        if use_prepared
                            && is_lasm_postgres_stale_prepared_statement_error(&err) =>
                    {
                        evict_lasm_postgres_prepared_statement(state, query_template);
                        let refresh_statement =
                            prepare_lasm_postgres_statement_with_reconnect(state, query_template)?;
                        let client = lasm_dynamic_postgres_client_mut(state)?;
                        let refresh = run_lasm_postgres_prepared_exec_with_count(
                            client,
                            &refresh_statement,
                            params,
                        )
                        .map_err(|refresh_err| {
                            format_lasm_postgres_runtime_error(
                                "postgres execution failed after retryable conflict retries reconnect stale prepared statement refresh",
                                &refresh_err,
                            )
                        })?;
                        state.db_postgres_retryable_conflict_retry_success_total = state
                            .db_postgres_retryable_conflict_retry_success_total
                            .saturating_add(1);
                        refresh
                    }
                    Err(err) if is_lasm_postgres_reconnectable_error(&err) => {
                        reconnect_lasm_dynamic_postgres_client(state)?;
                        let reconnect_retry = if use_prepared {
                            let retry_statement =
                                prepare_lasm_postgres_statement_with_reconnect(state, query_template)?;
                            let client = lasm_dynamic_postgres_client_mut(state)?;
                            run_lasm_postgres_prepared_exec_with_count(
                                client,
                                &retry_statement,
                                params,
                            )
                        } else {
                            let client = lasm_dynamic_postgres_client_mut(state)?;
                            run_lasm_postgres_unprepared_exec_with_count(client, query_template)
                        }
                        .map_err(|retry_err| {
                            format_lasm_postgres_runtime_error(
                                "postgres execution failed after retryable conflict retries reconnect replay",
                                &retry_err,
                            )
                        })?;
                        state.db_postgres_retryable_conflict_retry_success_total = state
                            .db_postgres_retryable_conflict_retry_success_total
                            .saturating_add(1);
                        reconnect_retry
                    }
                    Err(err) => {
                        return Err(format_lasm_postgres_runtime_error(
                            "postgres execution failed after retryable conflict retries and reconnect",
                            &err,
                        ));
                    }
                }
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
        Err(err) => {
            return Err(format_lasm_postgres_runtime_error(
                "postgres execution failed",
                &err,
            ))
        }
    };
    Ok(affected_rows)
}

#[allow(dead_code)]
fn run_lasm_postgres_exec_tx_once(
    client: &mut PostgresClient,
    query_template: &str,
    params: &[LasmPostgresParam],
    prepared_statement: Option<&PostgresStatement>,
) -> Result<u64, postgres::Error> {
    let mut tx = client.transaction()?;
    let affected_rows = if let Some(statement) = prepared_statement {
        run_lasm_postgres_prepared_exec_with_count(&mut tx, statement, params)?
    } else {
        run_lasm_postgres_unprepared_exec_with_count(&mut tx, query_template)?
    };
    tx.commit()?;
    Ok(affected_rows)
}

#[allow(dead_code)]
pub(crate) fn run_lasm_postgres_exec_tx(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
    params: &[LasmPostgresParam],
) -> Result<u64, String> {
    let required_params = max_lasm_postgres_placeholder_index_cached(state, query_template);
    validate_lasm_postgres_parameter_arity(required_params, params.len())?;
    let use_prepared = required_params > 0;
    if use_prepared && has_lasm_sql_non_trailing_statement_separator(query_template) {
        return Err("postgres parameterized execution requires a single SQL statement".to_string());
    }
    let prepared_statement = if use_prepared {
        Some(prepare_lasm_postgres_statement_with_reconnect(
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
        Err(err) if is_lasm_postgres_reconnectable_error(&err) => {
            reconnect_lasm_dynamic_postgres_client(state)?;
            let retry_statement = if use_prepared {
                Some(prepare_lasm_postgres_statement_with_reconnect(
                    state,
                    query_template,
                )?)
            } else {
                None
            };
            let client = lasm_dynamic_postgres_client_mut(state)?;
            run_lasm_postgres_exec_tx_once(client, query_template, params, retry_statement.as_ref())
                .map_err(|retry_err| {
                    format_lasm_postgres_runtime_error(
                        "postgres transaction execution failed after reconnect",
                        &retry_err,
                    )
                })?
        }
        Err(err) if use_prepared && is_lasm_postgres_stale_prepared_statement_error(&err) => {
            evict_lasm_postgres_prepared_statement(state, query_template);
            let retry_statement = Some(prepare_lasm_postgres_statement_with_reconnect(
                state,
                query_template,
            )?);
            let client = lasm_dynamic_postgres_client_mut(state)?;
            run_lasm_postgres_exec_tx_once(client, query_template, params, retry_statement.as_ref())
                .map_err(|retry_err| {
                    format_lasm_postgres_runtime_error(
                        "postgres transaction execution failed after stale prepared statement refresh",
                        &retry_err,
                    )
                })?
        }
        Err(err) if is_lasm_postgres_retryable_tx_error(&err) => {
            let _ = err;
            let mut recovered = None;
            for attempt_index in 0..state.db_postgres_retryable_conflict_retry_max {
                state.db_postgres_retryable_conflict_retry_attempts_total = state
                    .db_postgres_retryable_conflict_retry_attempts_total
                    .saturating_add(1);
                let backoff_ms = lasm_postgres_retry_backoff_ms(attempt_index);
                if backoff_ms > 0 {
                    std::thread::sleep(Duration::from_millis(backoff_ms));
                }
                let retry_statement = if use_prepared {
                    Some(prepare_lasm_postgres_statement_with_reconnect(
                        state,
                        query_template,
                    )?)
                } else {
                    None
                };
                let client = lasm_dynamic_postgres_client_mut(state)?;
                let retry_result = run_lasm_postgres_exec_tx_once(
                    client,
                    query_template,
                    params,
                    retry_statement.as_ref(),
                );
                match retry_result {
                    Ok(count) => {
                        state.db_postgres_retryable_conflict_retry_success_total = state
                            .db_postgres_retryable_conflict_retry_success_total
                            .saturating_add(1);
                        recovered = Some(count);
                        break;
                    }
                    Err(err) if is_lasm_postgres_retryable_tx_error(&err) => {
                        let _ = err;
                    }
                    Err(err)
                        if use_prepared
                            && is_lasm_postgres_stale_prepared_statement_error(&err) =>
                    {
                        evict_lasm_postgres_prepared_statement(state, query_template);
                    }
                    Err(err) if is_lasm_postgres_reconnectable_error(&err) => {
                        reconnect_lasm_dynamic_postgres_client(state)?;
                    }
                    Err(err) => {
                        return Err(format_lasm_postgres_runtime_error(
                            "postgres transaction execution failed after retryable conflict retry",
                            &err,
                        ));
                    }
                }
            }
            if let Some(count) = recovered {
                count
            } else {
                reconnect_lasm_dynamic_postgres_client(state)?;
                state.db_postgres_retryable_conflict_retry_attempts_total = state
                    .db_postgres_retryable_conflict_retry_attempts_total
                    .saturating_add(1);
                let retry_statement = if use_prepared {
                    Some(prepare_lasm_postgres_statement_with_reconnect(
                        state,
                        query_template,
                    )?)
                } else {
                    None
                };
                let client = lasm_dynamic_postgres_client_mut(state)?;
                let final_retry = run_lasm_postgres_exec_tx_once(
                    client,
                    query_template,
                    params,
                    retry_statement.as_ref(),
                );
                match final_retry {
                    Ok(count) => {
                        state.db_postgres_retryable_conflict_retry_success_total = state
                            .db_postgres_retryable_conflict_retry_success_total
                            .saturating_add(1);
                        count
                    }
                    Err(err)
                        if use_prepared
                            && is_lasm_postgres_stale_prepared_statement_error(&err) =>
                    {
                        evict_lasm_postgres_prepared_statement(state, query_template);
                        let refresh_statement = if use_prepared {
                            Some(prepare_lasm_postgres_statement_with_reconnect(
                                state,
                                query_template,
                            )?)
                        } else {
                            None
                        };
                        let client = lasm_dynamic_postgres_client_mut(state)?;
                        let refresh = run_lasm_postgres_exec_tx_once(
                            client,
                            query_template,
                            params,
                            refresh_statement.as_ref(),
                        )
                        .map_err(|refresh_err| {
                            format_lasm_postgres_runtime_error(
                                "postgres transaction execution failed after retryable conflict retries reconnect stale prepared statement refresh",
                                &refresh_err,
                            )
                        })?;
                        state.db_postgres_retryable_conflict_retry_success_total = state
                            .db_postgres_retryable_conflict_retry_success_total
                            .saturating_add(1);
                        refresh
                    }
                    Err(err) if is_lasm_postgres_reconnectable_error(&err) => {
                        reconnect_lasm_dynamic_postgres_client(state)?;
                        let retry_statement = if use_prepared {
                            Some(prepare_lasm_postgres_statement_with_reconnect(
                                state,
                                query_template,
                            )?)
                        } else {
                            None
                        };
                        let client = lasm_dynamic_postgres_client_mut(state)?;
                        let reconnect_retry = run_lasm_postgres_exec_tx_once(
                            client,
                            query_template,
                            params,
                            retry_statement.as_ref(),
                        )
                        .map_err(|retry_err| {
                            format_lasm_postgres_runtime_error(
                                "postgres transaction execution failed after retryable conflict retries reconnect replay",
                                &retry_err,
                            )
                        })?;
                        state.db_postgres_retryable_conflict_retry_success_total = state
                            .db_postgres_retryable_conflict_retry_success_total
                            .saturating_add(1);
                        reconnect_retry
                    }
                    Err(err) => {
                        return Err(format_lasm_postgres_runtime_error(
                            "postgres transaction execution failed after retryable conflict retries and reconnect",
                            &err,
                        ));
                    }
                }
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
        Err(err) => {
            return Err(format_lasm_postgres_runtime_error(
                "postgres transaction execution failed",
                &err,
            ))
        }
    };
    Ok(affected_rows)
}

fn lasm_postgres_thread_local_client_key(config: &LasmPostgresThreadLocalConfig) -> String {
    let tls_mode = match config.tls_mode {
        LasmDbPostgresTlsMode::Auto => "auto",
        LasmDbPostgresTlsMode::Disable => "disable",
        LasmDbPostgresTlsMode::Require => "require",
    };
    format!(
        "{tls_mode}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}",
        config.dsn, config.statement_timeout_ms, config.lock_timeout_ms, config.connect_timeout_ms
    )
}

fn connect_lasm_postgres_thread_local_client(
    config: &LasmPostgresThreadLocalConfig,
) -> Result<PostgresClient, String> {
    let mut client = connect_lasm_dynamic_db_records_postgres(
        config.dsn.as_str(),
        config.tls_mode,
        config.statement_timeout_ms.max(1),
        config.lock_timeout_ms.max(1),
        config.connect_timeout_ms.max(1),
    )?;
    ensure_lasm_dynamic_db_records_postgres_schema(&mut client)?;
    Ok(client)
}

enum LasmPostgresThreadLocalRuntimeError {
    Connect(String),
    Query(postgres::Error),
}

fn run_lasm_postgres_thread_local_with_client<R>(
    config: &LasmPostgresThreadLocalConfig,
    operation: impl FnOnce(&mut PostgresClient) -> Result<R, postgres::Error>,
) -> Result<R, LasmPostgresThreadLocalRuntimeError> {
    let max_idle = resolve_lasm_postgres_shared_client_max_idle_per_key();
    let max_total_idle = resolve_lasm_postgres_shared_client_max_total_idle();
    let key = lasm_postgres_thread_local_client_key(config);
    let pool = LASM_POSTGRES_SHARED_CLIENTS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut client = {
        let mut guard = pool.lock().map_err(|_| {
            LasmPostgresThreadLocalRuntimeError::Connect(
                "postgres shared client pool unavailable".to_string(),
            )
        })?;
        guard
            .get_mut(key.as_str())
            .and_then(|clients| clients.pop())
    };
    if client.is_none() {
        client = Some(
            connect_lasm_postgres_thread_local_client(config)
                .map_err(LasmPostgresThreadLocalRuntimeError::Connect)?,
        );
    }
    let mut client = client.expect("postgres shared client should resolve");
    let result = operation(&mut client).map_err(LasmPostgresThreadLocalRuntimeError::Query);
    match &result {
        Ok(_) => {
            if let Ok(mut guard) = pool.lock() {
                let total_idle = lasm_postgres_shared_pool_idle_total_locked(&guard);
                if total_idle < max_total_idle {
                    let entry = guard.entry(key).or_default();
                    if entry.len() < max_idle {
                        entry.push(client);
                    }
                }
            }
        }
        Err(LasmPostgresThreadLocalRuntimeError::Query(err))
            if !is_lasm_postgres_reconnectable_error(err) =>
        {
            if let Ok(mut guard) = pool.lock() {
                let total_idle = lasm_postgres_shared_pool_idle_total_locked(&guard);
                if total_idle < max_total_idle {
                    let entry = guard.entry(key).or_default();
                    if entry.len() < max_idle {
                        entry.push(client);
                    }
                }
            }
        }
        Err(_) => {}
    }
    result
}

fn invalidate_lasm_postgres_thread_local_client(config: &LasmPostgresThreadLocalConfig) {
    let key = lasm_postgres_thread_local_client_key(config);
    let pool = LASM_POSTGRES_SHARED_CLIENTS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = pool.lock() {
        guard.remove(key.as_str());
    }
}

fn run_lasm_postgres_direct_exec_with_params(
    client: &mut impl GenericClient,
    query_template: &str,
    params: &[LasmPostgresParam],
) -> Result<u64, postgres::Error> {
    let param_refs = lasm_postgres_query_param_refs(params);
    match client.execute(query_template, param_refs.as_slice()) {
        Ok(count) => Ok(count),
        Err(err) if is_lasm_postgres_execute_rows_error(&err) => {
            let rows = client.query(query_template, param_refs.as_slice())?;
            Ok(rows.len() as u64)
        }
        Err(err) => Err(err),
    }
}

fn run_lasm_postgres_thread_local_operation<R>(
    config: &LasmPostgresThreadLocalConfig,
    context: &str,
    mut operation: impl FnMut(&mut PostgresClient) -> Result<R, postgres::Error>,
) -> Result<R, String> {
    let mut reconnect_attempted = false;
    let mut retry_attempts = 0usize;
    loop {
        match run_lasm_postgres_thread_local_with_client(config, |client| operation(client)) {
            Ok(value) => return Ok(value),
            Err(LasmPostgresThreadLocalRuntimeError::Connect(message)) => return Err(message),
            Err(LasmPostgresThreadLocalRuntimeError::Query(err))
                if is_lasm_postgres_reconnectable_error(&err) && !reconnect_attempted =>
            {
                reconnect_attempted = true;
                invalidate_lasm_postgres_thread_local_client(config);
            }
            Err(LasmPostgresThreadLocalRuntimeError::Query(err))
                if is_lasm_postgres_retryable_tx_error(&err)
                    && retry_attempts < config.retryable_conflict_retry_max =>
            {
                let backoff_ms = lasm_postgres_retry_backoff_ms(retry_attempts);
                retry_attempts = retry_attempts.saturating_add(1);
                if backoff_ms > 0 {
                    std::thread::sleep(Duration::from_millis(backoff_ms));
                }
            }
            Err(LasmPostgresThreadLocalRuntimeError::Query(err)) => {
                return Err(format_lasm_postgres_runtime_error(context, &err));
            }
        }
    }
}

pub(crate) fn run_lasm_postgres_exec_thread_local(
    config: &LasmPostgresThreadLocalConfig,
    query_template: &str,
    params: &[LasmPostgresParam],
) -> Result<u64, String> {
    let required_params = max_lasm_postgres_placeholder_index(query_template);
    validate_lasm_postgres_parameter_arity(required_params, params.len())?;
    if required_params > 0 && has_lasm_sql_non_trailing_statement_separator(query_template) {
        return Err("postgres parameterized execution requires a single SQL statement".to_string());
    }
    let bound_params = if required_params > 0 {
        params
    } else {
        &[] as &[LasmPostgresParam]
    };
    run_lasm_postgres_thread_local_operation(config, "postgres execution failed", |client| {
        if required_params > 0 {
            run_lasm_postgres_direct_exec_with_params(client, query_template, bound_params)
        } else {
            run_lasm_postgres_unprepared_exec_with_count(client, query_template)
        }
    })
}

fn run_lasm_postgres_exec_tx_direct_once(
    client: &mut PostgresClient,
    query_template: &str,
    params: &[LasmPostgresParam],
) -> Result<u64, postgres::Error> {
    let mut tx = client.transaction()?;
    let affected_rows = if params.is_empty() {
        run_lasm_postgres_unprepared_exec_with_count(&mut tx, query_template)?
    } else {
        run_lasm_postgres_direct_exec_with_params(&mut tx, query_template, params)?
    };
    tx.commit()?;
    Ok(affected_rows)
}

pub(crate) fn run_lasm_postgres_exec_tx_thread_local(
    config: &LasmPostgresThreadLocalConfig,
    query_template: &str,
    params: &[LasmPostgresParam],
) -> Result<u64, String> {
    let required_params = max_lasm_postgres_placeholder_index(query_template);
    validate_lasm_postgres_parameter_arity(required_params, params.len())?;
    if required_params > 0 && has_lasm_sql_non_trailing_statement_separator(query_template) {
        return Err("postgres parameterized execution requires a single SQL statement".to_string());
    }
    let bound_params = if required_params > 0 {
        params
    } else {
        &[] as &[LasmPostgresParam]
    };
    run_lasm_postgres_thread_local_operation(
        config,
        "postgres transaction execution failed",
        |client| run_lasm_postgres_exec_tx_direct_once(client, query_template, bound_params),
    )
}

pub(crate) fn persist_lasm_postgres_record_append_thread_local(
    config: &LasmPostgresThreadLocalConfig,
    record: &LasmDbRecord,
) -> Result<(), String> {
    let id = i64::try_from(record.id).map_err(|_| {
        format!(
            "could not persist LASM dynamic postgres record id {}: out of i64 range",
            record.id
        )
    })?;
    let created_at_ms = i64::try_from(record.created_at_ms).map_err(|_| {
        format!(
            "could not persist LASM dynamic postgres record timestamp {}: out of i64 range",
            record.created_at_ms
        )
    })?;
    let affected_rows = i64::try_from(record.affected_rows).map_err(|_| {
        format!(
            "could not persist LASM dynamic postgres record affected_rows {}: out of i64 range",
            record.affected_rows
        )
    })?;
    let insert_statement = format!(
        "INSERT INTO {} (id, op, db, template, params, tx, affected_rows, created_at_ms) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (id) DO UPDATE SET \
             op = EXCLUDED.op, \
             db = EXCLUDED.db, \
             template = EXCLUDED.template, \
             params = EXCLUDED.params, \
             tx = EXCLUDED.tx, \
             affected_rows = EXCLUDED.affected_rows, \
             created_at_ms = EXCLUDED.created_at_ms",
        LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE
    );
    run_lasm_postgres_thread_local_operation(
        config,
        "could not append LASM dynamic postgres record",
        |client| {
            client
                .execute(
                    insert_statement.as_str(),
                    &[
                        &id,
                        &record.op,
                        &record.db,
                        &record.template,
                        &record.params,
                        &record.tx,
                        &affected_rows,
                        &created_at_ms,
                    ],
                )
                .map(|_| ())
        },
    )
}

pub(crate) fn persist_lasm_postgres_record_append_batch_thread_local(
    config: &LasmPostgresThreadLocalConfig,
    records: &[LasmDbRecord],
) -> Result<(), String> {
    if records.is_empty() {
        return Ok(());
    }
    let mut ids = Vec::with_capacity(records.len());
    let mut ops = Vec::with_capacity(records.len());
    let mut dbs = Vec::with_capacity(records.len());
    let mut templates = Vec::with_capacity(records.len());
    let mut params_payloads = Vec::with_capacity(records.len());
    let mut txs = Vec::with_capacity(records.len());
    let mut affected_rows_values = Vec::with_capacity(records.len());
    let mut created_at_ms_values = Vec::with_capacity(records.len());
    for record in records {
        let id = i64::try_from(record.id).map_err(|_| {
            format!(
                "could not persist LASM dynamic postgres record id {}: out of i64 range",
                record.id
            )
        })?;
        let created_at_ms = i64::try_from(record.created_at_ms).map_err(|_| {
            format!(
                "could not persist LASM dynamic postgres record timestamp {}: out of i64 range",
                record.created_at_ms
            )
        })?;
        let affected_rows = i64::try_from(record.affected_rows).map_err(|_| {
            format!(
                "could not persist LASM dynamic postgres record affected_rows {}: out of i64 range",
                record.affected_rows
            )
        })?;
        ids.push(id);
        ops.push(record.op.clone());
        dbs.push(record.db);
        templates.push(record.template.clone());
        params_payloads.push(record.params.clone());
        txs.push(record.tx);
        affected_rows_values.push(affected_rows);
        created_at_ms_values.push(created_at_ms);
    }
    let insert_statement = format!(
        "INSERT INTO {} (id, op, db, template, params, tx, affected_rows, created_at_ms) \
         SELECT * FROM UNNEST( \
             $1::bigint[], \
             $2::text[], \
             $3::bigint[], \
             $4::text[], \
             $5::text[], \
             $6::bigint[], \
             $7::bigint[], \
             $8::bigint[] \
         ) AS _sec4_records(id, op, db, template, params, tx, affected_rows, created_at_ms) \
         ON CONFLICT (id) DO UPDATE SET \
             op = EXCLUDED.op, \
             db = EXCLUDED.db, \
             template = EXCLUDED.template, \
             params = EXCLUDED.params, \
             tx = EXCLUDED.tx, \
             affected_rows = EXCLUDED.affected_rows, \
             created_at_ms = EXCLUDED.created_at_ms",
        LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE
    );
    run_lasm_postgres_thread_local_operation(
        config,
        "could not append LASM dynamic postgres record batch",
        |client| {
            client
                .execute(
                    insert_statement.as_str(),
                    &[
                        &ids,
                        &ops,
                        &dbs,
                        &templates,
                        &params_payloads,
                        &txs,
                        &affected_rows_values,
                        &created_at_ms_values,
                    ],
                )
                .map(|_| ())
        },
    )
}

pub(crate) fn persist_lasm_postgres_records_full_sync_thread_local(
    config: &LasmPostgresThreadLocalConfig,
    records: &[LasmDbRecord],
) -> Result<(), String> {
    let mut ordered = records.to_vec();
    ordered.sort_by_key(|record| record.id);
    let mut ids = Vec::with_capacity(ordered.len());
    let mut ops = Vec::with_capacity(ordered.len());
    let mut dbs = Vec::with_capacity(ordered.len());
    let mut templates = Vec::with_capacity(ordered.len());
    let mut params_payloads = Vec::with_capacity(ordered.len());
    let mut txs = Vec::with_capacity(ordered.len());
    let mut affected_rows_values = Vec::with_capacity(ordered.len());
    let mut created_at_ms_values = Vec::with_capacity(ordered.len());
    for record in &ordered {
        let id = i64::try_from(record.id).map_err(|_| {
            format!(
                "could not persist LASM dynamic postgres record id {}: out of i64 range",
                record.id
            )
        })?;
        let created_at_ms = i64::try_from(record.created_at_ms).map_err(|_| {
            format!(
                "could not persist LASM dynamic postgres record timestamp {}: out of i64 range",
                record.created_at_ms
            )
        })?;
        let affected_rows = i64::try_from(record.affected_rows).map_err(|_| {
            format!(
                "could not persist LASM dynamic postgres record affected_rows {}: out of i64 range",
                record.affected_rows
            )
        })?;
        ids.push(id);
        ops.push(record.op.clone());
        dbs.push(record.db);
        templates.push(record.template.clone());
        params_payloads.push(record.params.clone());
        txs.push(record.tx);
        affected_rows_values.push(affected_rows);
        created_at_ms_values.push(created_at_ms);
    }
    let delete_statement = format!("DELETE FROM {}", LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE);
    let insert_statement = format!(
        "INSERT INTO {} (id, op, db, template, params, tx, affected_rows, created_at_ms) \
         SELECT * FROM UNNEST( \
             $1::bigint[], \
             $2::text[], \
             $3::bigint[], \
             $4::text[], \
             $5::text[], \
             $6::bigint[], \
             $7::bigint[], \
             $8::bigint[] \
         ) AS _sec4_records(id, op, db, template, params, tx, affected_rows, created_at_ms)",
        LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE
    );
    run_lasm_postgres_thread_local_operation(
        config,
        "could not full sync LASM dynamic postgres records store",
        |client| {
            let mut tx = client.transaction()?;
            tx.execute(delete_statement.as_str(), &[])?;
            if !ids.is_empty() {
                tx.execute(
                    insert_statement.as_str(),
                    &[
                        &ids,
                        &ops,
                        &dbs,
                        &templates,
                        &params_payloads,
                        &txs,
                        &affected_rows_values,
                        &created_at_ms_values,
                    ],
                )?;
            }
            tx.commit()
        },
    )
}

pub(crate) fn run_lasm_postgres_query_one_thread_local(
    config: &LasmPostgresThreadLocalConfig,
    query_template: &str,
    params: &[LasmPostgresParam],
) -> Result<Option<serde_json::Value>, String> {
    let normalized_query = normalize_lasm_postgres_query_for_subquery(query_template);
    if normalized_query.trim().is_empty() {
        return Err("postgres queryOne requires non-empty SQL statement".to_string());
    }
    if !is_lasm_postgres_query_one_select_like(normalized_query.as_str()) {
        return Err(
            "postgres queryOne requires row-returning SQL statement (SELECT/WITH/VALUES/TABLE or DML ... RETURNING)"
                .to_string(),
        );
    }
    if has_lasm_sql_non_trailing_statement_separator(normalized_query.as_str()) {
        return Err("postgres parameterized execution requires a single SQL statement".to_string());
    }
    let required_params = max_lasm_postgres_placeholder_index(normalized_query.as_str());
    validate_lasm_postgres_parameter_arity(required_params, params.len())?;
    let bound_params = if required_params > 0 {
        params
    } else {
        &[] as &[LasmPostgresParam]
    };
    let wrapped_query = format!(
        "SELECT row_to_json(_sec4_row)::text AS __sec4_row \
         FROM ({}) AS _sec4_row LIMIT 1",
        normalized_query
    );
    let mut reconnect_attempted = false;
    let mut retry_attempts = 0usize;
    let row = loop {
        match run_lasm_postgres_thread_local_with_client(config, |client| {
            let param_refs = lasm_postgres_query_param_refs(bound_params);
            client.query_opt(wrapped_query.as_str(), param_refs.as_slice())
        }) {
            Ok(row) => break row,
            Err(LasmPostgresThreadLocalRuntimeError::Connect(message)) => return Err(message),
            Err(LasmPostgresThreadLocalRuntimeError::Query(err))
                if is_lasm_postgres_reconnectable_error(&err) && !reconnect_attempted =>
            {
                reconnect_attempted = true;
                invalidate_lasm_postgres_thread_local_client(config);
            }
            Err(LasmPostgresThreadLocalRuntimeError::Query(err))
                if is_lasm_postgres_retryable_tx_error(&err)
                    && retry_attempts < config.retryable_conflict_retry_max =>
            {
                let backoff_ms = lasm_postgres_retry_backoff_ms(retry_attempts);
                retry_attempts = retry_attempts.saturating_add(1);
                if backoff_ms > 0 {
                    std::thread::sleep(Duration::from_millis(backoff_ms));
                }
            }
            Err(LasmPostgresThreadLocalRuntimeError::Query(err)) => {
                return Err(format_lasm_postgres_runtime_error(
                    "postgres queryOne execution failed",
                    &err,
                ));
            }
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

#[allow(dead_code)]
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
        return Err(
            "postgres queryOne requires row-returning SQL statement (SELECT/WITH/VALUES/TABLE or DML ... RETURNING)"
                .to_string(),
        );
    }
    if has_lasm_sql_non_trailing_statement_separator(normalized_query.as_str()) {
        return Err("postgres parameterized execution requires a single SQL statement".to_string());
    }
    let required_params =
        max_lasm_postgres_placeholder_index_cached(state, normalized_query.as_str());
    validate_lasm_postgres_parameter_arity(required_params, params.len())?;
    let bound_params = if required_params > 0 {
        params
    } else {
        &[] as &[LasmPostgresParam]
    };
    let wrapped_query = format!(
        "SELECT row_to_json(_sec4_row)::text AS __sec4_row \
         FROM ({}) AS _sec4_row LIMIT 1",
        normalized_query
    );
    let prepared_statement =
        prepare_lasm_postgres_statement_with_reconnect(state, wrapped_query.as_str())?;
    let execute_query = |client: &mut PostgresClient,
                         statement: &PostgresStatement|
     -> Result<Option<postgres::Row>, postgres::Error> {
        let param_refs = lasm_postgres_query_param_refs(bound_params);
        client.query_opt(statement, param_refs.as_slice())
    };
    let row = {
        let initial = {
            let client = lasm_dynamic_postgres_client_mut(state)?;
            execute_query(client, &prepared_statement)
        };
        match initial {
            Ok(row) => row,
            Err(err) if is_lasm_postgres_reconnectable_error(&err) => {
                reconnect_lasm_dynamic_postgres_client(state)?;
                let retry_statement =
                    prepare_lasm_postgres_statement_with_reconnect(state, wrapped_query.as_str())?;
                let client = lasm_dynamic_postgres_client_mut(state)?;
                execute_query(client, &retry_statement).map_err(|retry_err| {
                    format_lasm_postgres_runtime_error(
                        "postgres queryOne execution failed after reconnect",
                        &retry_err,
                    )
                })?
            }
            Err(err) if is_lasm_postgres_stale_prepared_statement_error(&err) => {
                evict_lasm_postgres_prepared_statement(state, wrapped_query.as_str());
                let retry_statement =
                    prepare_lasm_postgres_statement_with_reconnect(state, wrapped_query.as_str())?;
                let client = lasm_dynamic_postgres_client_mut(state)?;
                execute_query(client, &retry_statement).map_err(|retry_err| {
                    format_lasm_postgres_runtime_error(
                        "postgres queryOne execution failed after stale prepared statement refresh",
                        &retry_err,
                    )
                })?
            }
            Err(err) if is_lasm_postgres_retryable_tx_error(&err) => {
                let _ = err;
                let mut recovered = None;
                for attempt_index in 0..state.db_postgres_retryable_conflict_retry_max {
                    state.db_postgres_retryable_conflict_retry_attempts_total = state
                        .db_postgres_retryable_conflict_retry_attempts_total
                        .saturating_add(1);
                    let backoff_ms = lasm_postgres_retry_backoff_ms(attempt_index);
                    if backoff_ms > 0 {
                        std::thread::sleep(Duration::from_millis(backoff_ms));
                    }
                    let retry_statement = prepare_lasm_postgres_statement_with_reconnect(
                        state,
                        wrapped_query.as_str(),
                    )?;
                    let client = lasm_dynamic_postgres_client_mut(state)?;
                    let retry_result = execute_query(client, &retry_statement);
                    match retry_result {
                        Ok(row) => {
                            state.db_postgres_retryable_conflict_retry_success_total = state
                                .db_postgres_retryable_conflict_retry_success_total
                                .saturating_add(1);
                            recovered = Some(row);
                            break;
                        }
                        Err(err) if is_lasm_postgres_retryable_tx_error(&err) => {
                            let _ = err;
                        }
                        Err(err) if is_lasm_postgres_stale_prepared_statement_error(&err) => {
                            evict_lasm_postgres_prepared_statement(state, wrapped_query.as_str());
                        }
                        Err(err) if is_lasm_postgres_reconnectable_error(&err) => {
                            reconnect_lasm_dynamic_postgres_client(state)?;
                        }
                        Err(err) => {
                            return Err(format_lasm_postgres_runtime_error(
                                "postgres queryOne execution failed after retryable conflict retry",
                                &err,
                            ));
                        }
                    }
                }
                if let Some(row) = recovered {
                    row
                } else {
                    reconnect_lasm_dynamic_postgres_client(state)?;
                    state.db_postgres_retryable_conflict_retry_attempts_total = state
                        .db_postgres_retryable_conflict_retry_attempts_total
                        .saturating_add(1);
                    let retry_statement = prepare_lasm_postgres_statement_with_reconnect(
                        state,
                        wrapped_query.as_str(),
                    )?;
                    let client = lasm_dynamic_postgres_client_mut(state)?;
                    let final_retry = execute_query(client, &retry_statement);
                    match final_retry {
                        Ok(row) => {
                            state.db_postgres_retryable_conflict_retry_success_total = state
                                .db_postgres_retryable_conflict_retry_success_total
                                .saturating_add(1);
                            row
                        }
                        Err(err) if is_lasm_postgres_stale_prepared_statement_error(&err) => {
                            evict_lasm_postgres_prepared_statement(state, wrapped_query.as_str());
                            let refresh_statement = prepare_lasm_postgres_statement_with_reconnect(
                                state,
                                wrapped_query.as_str(),
                            )?;
                            let client = lasm_dynamic_postgres_client_mut(state)?;
                            let refresh = execute_query(client, &refresh_statement).map_err(
                                |refresh_err| {
                                    format_lasm_postgres_runtime_error(
                                        "postgres queryOne execution failed after retryable conflict retries reconnect stale prepared statement refresh",
                                        &refresh_err,
                                    )
                                },
                            )?;
                            state.db_postgres_retryable_conflict_retry_success_total = state
                                .db_postgres_retryable_conflict_retry_success_total
                                .saturating_add(1);
                            refresh
                        }
                        Err(err) if is_lasm_postgres_reconnectable_error(&err) => {
                            reconnect_lasm_dynamic_postgres_client(state)?;
                            let retry_statement = prepare_lasm_postgres_statement_with_reconnect(
                                state,
                                wrapped_query.as_str(),
                            )?;
                            let client = lasm_dynamic_postgres_client_mut(state)?;
                            let reconnect_retry = execute_query(client, &retry_statement).map_err(
                                |retry_err| {
                                    format_lasm_postgres_runtime_error(
                                        "postgres queryOne execution failed after retryable conflict retries reconnect replay",
                                        &retry_err,
                                    )
                                },
                            )?;
                            state.db_postgres_retryable_conflict_retry_success_total = state
                                .db_postgres_retryable_conflict_retry_success_total
                                .saturating_add(1);
                            reconnect_retry
                        }
                        Err(err) => {
                            return Err(format_lasm_postgres_runtime_error(
                                "postgres queryOne execution failed after retryable conflict retries and reconnect",
                                &err,
                            ));
                        }
                    }
                }
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
            Err(err) => {
                return Err(format_lasm_postgres_runtime_error(
                    "postgres queryOne execution failed",
                    &err,
                ))
            }
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
    use super::{
        is_lasm_postgres_reconnectable_error_message, is_lasm_postgres_reconnectable_prepare_error,
        is_lasm_postgres_reconnectable_sqlstate, is_lasm_postgres_retryable_tx_sqlstate,
        is_lasm_postgres_stale_prepare_error, is_lasm_postgres_stale_prepared_statement_sqlstate,
        lasm_postgres_retry_backoff_ms, max_lasm_postgres_placeholder_index_cached,
        parse_lasm_postgres_query_params, parse_lasm_postgres_query_template_and_params,
        validate_lasm_postgres_parameter_arity, LasmPostgresParam,
    };
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

    #[test]
    fn retryable_tx_sqlstate_detection_matches_serialization_and_deadlock_codes() {
        assert!(is_lasm_postgres_retryable_tx_sqlstate(Some("40001")));
        assert!(is_lasm_postgres_retryable_tx_sqlstate(Some("40P01")));
        assert!(!is_lasm_postgres_retryable_tx_sqlstate(Some("23505")));
        assert!(!is_lasm_postgres_retryable_tx_sqlstate(None));
    }

    #[test]
    fn reconnectable_sqlstate_detection_matches_connection_classes() {
        assert!(is_lasm_postgres_reconnectable_sqlstate(Some("08006")));
        assert!(is_lasm_postgres_reconnectable_sqlstate(Some("08001")));
        assert!(is_lasm_postgres_reconnectable_sqlstate(Some("57P01")));
        assert!(is_lasm_postgres_reconnectable_sqlstate(Some("57p03")));
        assert!(!is_lasm_postgres_reconnectable_sqlstate(Some("40001")));
        assert!(!is_lasm_postgres_reconnectable_sqlstate(None));
    }

    #[test]
    fn reconnectable_prepare_error_detection_matches_runtime_sqlstate_marker() {
        assert!(is_lasm_postgres_reconnectable_prepare_error(
            "postgres prepare failed: admin shutdown; sqlstate=57p01"
        ));
        assert!(is_lasm_postgres_reconnectable_prepare_error(
            "postgres prepare failed: admin shutdown; SQLSTATE=57P01"
        ));
        assert!(is_lasm_postgres_reconnectable_prepare_error(
            "postgres prepare failed: connection failure; sqlstate=08006"
        ));
        assert!(!is_lasm_postgres_reconnectable_prepare_error(
            "postgres prepare failed: syntax error at or near \"FROM\"; sqlstate=42601"
        ));
    }

    #[test]
    fn reconnectable_error_message_detection_matches_connection_drop_patterns() {
        assert!(is_lasm_postgres_reconnectable_error_message(
            "server closed the connection unexpectedly"
        ));
        assert!(is_lasm_postgres_reconnectable_error_message(
            "connection reset by peer"
        ));
        assert!(is_lasm_postgres_reconnectable_error_message("broken pipe"));
        assert!(is_lasm_postgres_reconnectable_error_message(
            "terminating connection due to administrator command"
        ));
        assert!(!is_lasm_postgres_reconnectable_error_message(
            "syntax error at or near \"FROM\""
        ));
    }

    #[test]
    fn stale_prepare_error_detection_matches_runtime_sqlstate_marker() {
        assert!(is_lasm_postgres_stale_prepare_error(
            "postgres prepare failed: prepared statement does not exist; sqlstate=26000"
        ));
        assert!(is_lasm_postgres_stale_prepare_error(
            "postgres prepare failed: prepared statement \"s1\" does not exist"
        ));
        assert!(is_lasm_postgres_stale_prepare_error(
            "postgres queryOne execution failed: cached plan must not change result type; sqlstate=0A000"
        ));
        assert!(!is_lasm_postgres_stale_prepare_error(
            "postgres prepare failed: admin shutdown; sqlstate=57P01"
        ));
    }

    #[test]
    fn stale_prepared_statement_sqlstate_detection_matches_invalid_statement_name() {
        assert!(is_lasm_postgres_stale_prepared_statement_sqlstate(Some(
            "26000"
        )));
        assert!(!is_lasm_postgres_stale_prepared_statement_sqlstate(Some(
            "08006"
        )));
        assert!(!is_lasm_postgres_stale_prepared_statement_sqlstate(None));
    }

    #[test]
    fn postgres_retry_backoff_increases_linearly() {
        assert_eq!(lasm_postgres_retry_backoff_ms(0), 2);
        assert_eq!(lasm_postgres_retry_backoff_ms(1), 4);
        assert_eq!(lasm_postgres_retry_backoff_ms(2), 6);
    }

    #[test]
    fn postgres_parameter_arity_rejects_extra_params() {
        let error =
            validate_lasm_postgres_parameter_arity(1, 2).expect_err("extra params should fail");
        assert_eq!(
            error,
            "postgres query expects exactly 1 sql parameters but received 2"
        );
    }

    #[test]
    fn postgres_parameter_arity_rejects_extra_params_when_sql_has_no_placeholders() {
        let error =
            validate_lasm_postgres_parameter_arity(0, 2).expect_err("extra params should fail");
        assert_eq!(
            error,
            "postgres query expects exactly 0 sql parameters but received 2"
        );
    }

    #[test]
    fn positional_object_params_expand_with_null_fill() {
        let params = parse_lasm_postgres_query_params("{\"1\":\"alice\",\"3\":true}")
            .expect("positional params should parse");
        assert_eq!(params.len(), 3);
        match &params[0] {
            LasmPostgresParam::Text(value) => assert_eq!(value, "alice"),
            _ => panic!("expected text param at position 1"),
        }
        match &params[1] {
            LasmPostgresParam::Null(_) => {}
            _ => panic!("expected null fill at position 2"),
        }
        match &params[2] {
            LasmPostgresParam::Bool(value) => assert!(*value),
            _ => panic!("expected bool param at position 3"),
        }
    }

    #[test]
    fn positional_object_params_preserve_large_unsigned_integer_as_text() {
        let params = parse_lasm_postgres_query_params("{\"1\":18446744073709551615}")
            .expect("positional params should parse");
        assert_eq!(params.len(), 1);
        match &params[0] {
            LasmPostgresParam::Text(value) => assert_eq!(value, "18446744073709551615"),
            _ => panic!("expected large unsigned integer param to be preserved as text"),
        }
    }

    #[test]
    fn mixed_object_params_return_validation_error() {
        let error = match parse_lasm_postgres_query_params("{\"1\":\"alice\",\"name\":\"bob\"}") {
            Ok(_) => panic!("mixed positional and named keys should fail"),
            Err(error) => error,
        };
        assert!(error.contains("postgres params object keys must be all positional"));
    }

    #[test]
    fn named_object_params_without_named_placeholders_return_validation_error() {
        let error = match parse_lasm_postgres_query_template_and_params(
            "SELECT $1::text",
            "{\"name\":\"alice\"}",
        ) {
            Ok(_) => panic!("named params with positional SQL placeholders should fail"),
            Err(error) => error,
        };
        assert!(error.contains(
            "postgres named parameterized execution requires SQL placeholders to be named"
        ));
    }

    #[test]
    fn named_object_params_reject_extra_params_not_present_in_sql() {
        let error = match parse_lasm_postgres_query_template_and_params(
            "SELECT :name::text",
            "{\"name\":\"alice\",\"role\":\"admin\"}",
        ) {
            Ok(_) => panic!("extra named params should fail"),
            Err(error) => error,
        };
        assert!(error.contains("postgres query parameter `role` is not present in SQL statement"));
    }

    #[test]
    fn named_object_params_reject_duplicate_normalized_keys() {
        let error = match parse_lasm_postgres_query_template_and_params(
            "SELECT :name::text",
            "{\"name\":\"alice\",\":name\":\"bob\"}",
        ) {
            Ok(_) => panic!("duplicate normalized named keys should fail"),
            Err(error) => error,
        };
        assert!(error.contains("postgres params object contains duplicate normalized key `name`"));
    }

    #[test]
    fn named_object_params_reject_mixed_named_and_positional_sql_placeholders() {
        let error = match parse_lasm_postgres_query_template_and_params(
            "SELECT $1::text, :name::text",
            "{\"name\":\"alice\"}",
        ) {
            Ok(_) => panic!("mixing named and positional placeholders should fail"),
            Err(error) => error,
        };
        assert!(error.contains(
            "postgres named parameterized execution does not support mixing named and positional SQL placeholders"
        ));
    }

    #[test]
    fn positional_object_params_accept_placeholder_prefixed_keys() {
        let params = parse_lasm_postgres_query_params("{\"$2\":\"alice\"}")
            .expect("positional params should parse");
        assert_eq!(params.len(), 2);
        match &params[0] {
            LasmPostgresParam::Null(_) => {}
            _ => panic!("expected null fill at position 1"),
        }
        match &params[1] {
            LasmPostgresParam::Text(value) => assert_eq!(value, "alice"),
            _ => panic!("expected text param at position 2"),
        }
    }

    #[test]
    fn named_object_params_rewrite_colon_placeholders() {
        let (template, params) = parse_lasm_postgres_query_template_and_params(
            "SELECT * FROM users WHERE email = :email AND active = :active",
            "{\"email\":\"alice@example.com\",\"active\":true}",
        )
        .expect("expected named postgres params to rewrite");
        assert_eq!(
            template,
            "SELECT * FROM users WHERE email = $1 AND active = $2"
        );
        assert_eq!(params.len(), 2);
        match &params[0] {
            LasmPostgresParam::Text(value) => assert_eq!(value, "alice@example.com"),
            _ => panic!("expected text param at position 1"),
        }
        match &params[1] {
            LasmPostgresParam::Bool(value) => assert!(*value),
            _ => panic!("expected bool param at position 2"),
        }
    }

    #[test]
    fn named_object_params_skip_cast_literals_comments_and_reuse_indices() {
        let (template, params) = parse_lasm_postgres_query_template_and_params(
            "SELECT :name::text AS n, ':name' AS literal /* :name */ WHERE id = @id OR backup = :name",
            "{\"name\":\"alice\",\"id\":42}",
        )
        .expect("expected named postgres params to rewrite");
        assert_eq!(
            template,
            "SELECT $1::text AS n, ':name' AS literal /* :name */ WHERE id = $2 OR backup = $1"
        );
        assert_eq!(params.len(), 2);
        match &params[0] {
            LasmPostgresParam::Text(value) => assert_eq!(value, "alice"),
            _ => panic!("expected text param at position 1"),
        }
        match &params[1] {
            LasmPostgresParam::Int(value) => assert_eq!(*value, 42),
            _ => panic!("expected int param at position 2"),
        }
    }
}
