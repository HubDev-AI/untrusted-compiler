use crate::lasm_db_runtime_common::{
    insert_lasm_bounded_cache_entry, lasm_dynamic_postgres_client_mut,
    lasm_dynamic_postgres_prepared_statement, reconnect_lasm_dynamic_postgres_client,
    touch_lasm_bounded_cache_entry,
};
use crate::{has_lasm_sql_non_trailing_statement_separator, LasmDynamicResponseState};
use postgres::types::ToSql;
use postgres::{Client as PostgresClient, GenericClient, Statement as PostgresStatement};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone)]
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
    matches!(code, Some("57P01") | Some("57P02") | Some("57P03"))
        || matches!(code, Some(value) if value.starts_with("08"))
}

fn is_lasm_postgres_reconnectable_error(err: &postgres::Error) -> bool {
    err.is_closed() || is_lasm_postgres_reconnectable_sqlstate(err.code().map(|code| code.code()))
}

fn is_lasm_postgres_stale_prepared_statement_sqlstate(code: Option<&str>) -> bool {
    matches!(code, Some("26000"))
}

fn is_lasm_postgres_stale_prepared_statement_error(err: &postgres::Error) -> bool {
    if is_lasm_postgres_stale_prepared_statement_sqlstate(err.code().map(|code| code.code())) {
        return true;
    }
    let normalized = err.to_string().to_ascii_lowercase();
    normalized.contains("prepared statement") && normalized.contains("does not exist")
}

fn evict_lasm_postgres_prepared_statement(
    state: &mut LasmDynamicResponseState,
    query_template: &str,
) {
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
    if required_params == 0 {
        return Ok(());
    }
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
        Some(lasm_dynamic_postgres_prepared_statement(
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
                    lasm_dynamic_postgres_prepared_statement(state, query_template)?;
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
            let retry_statement = lasm_dynamic_postgres_prepared_statement(state, query_template)?;
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
                        lasm_dynamic_postgres_prepared_statement(state, query_template)?;
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
                        lasm_dynamic_postgres_prepared_statement(state, query_template)?;
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
        Err(err) if is_lasm_postgres_reconnectable_error(&err) => {
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
                    format_lasm_postgres_runtime_error(
                        "postgres transaction execution failed after reconnect",
                        &retry_err,
                    )
                })?
        }
        Err(err) if use_prepared && is_lasm_postgres_stale_prepared_statement_error(&err) => {
            evict_lasm_postgres_prepared_statement(state, query_template);
            let retry_statement = Some(lasm_dynamic_postgres_prepared_statement(
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
                    Some(lasm_dynamic_postgres_prepared_statement(
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
                    Some(lasm_dynamic_postgres_prepared_statement(
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
        lasm_dynamic_postgres_prepared_statement(state, wrapped_query.as_str())?;
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
                    lasm_dynamic_postgres_prepared_statement(state, wrapped_query.as_str())?;
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
                    lasm_dynamic_postgres_prepared_statement(state, wrapped_query.as_str())?;
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
                    let retry_statement =
                        lasm_dynamic_postgres_prepared_statement(state, wrapped_query.as_str())?;
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
                    let retry_statement =
                        lasm_dynamic_postgres_prepared_statement(state, wrapped_query.as_str())?;
                    let client = lasm_dynamic_postgres_client_mut(state)?;
                    let final_retry = execute_query(client, &retry_statement);
                    match final_retry {
                        Ok(row) => {
                            state.db_postgres_retryable_conflict_retry_success_total = state
                                .db_postgres_retryable_conflict_retry_success_total
                                .saturating_add(1);
                            row
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
        is_lasm_postgres_reconnectable_sqlstate, is_lasm_postgres_retryable_tx_sqlstate,
        is_lasm_postgres_stale_prepared_statement_sqlstate, lasm_postgres_retry_backoff_ms,
        max_lasm_postgres_placeholder_index_cached, parse_lasm_postgres_query_params,
        parse_lasm_postgres_query_template_and_params, validate_lasm_postgres_parameter_arity,
        LasmPostgresParam,
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
        assert!(!is_lasm_postgres_reconnectable_sqlstate(Some("40001")));
        assert!(!is_lasm_postgres_reconnectable_sqlstate(None));
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
    fn postgres_parameter_arity_allows_extra_params_when_sql_has_no_placeholders() {
        validate_lasm_postgres_parameter_arity(0, 2)
            .expect("zero-placeholder sql should ignore extra params for compatibility");
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
