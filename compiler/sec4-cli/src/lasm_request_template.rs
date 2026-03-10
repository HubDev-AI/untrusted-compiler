use crate::{find_lasm_cookie_value, find_lasm_header_value, LasmRunRequest};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const LASM_WORKBENCH_PUBLIC_REQUEST_MARKER: &str = "__sec4_workbench_public";

pub(crate) fn contains_lasm_request_placeholder_tokens(value: &str) -> bool {
    value.contains("{{req.pathParam:")
        || value.contains("{{req.header:")
        || value.contains("{{req.query:")
        || value.contains("{{req.cookie:")
        || value.contains("{{req.method}}")
        || value.contains("{{req.path}}")
        || value.contains("{{req.httpVersion}}")
        || value.contains("{{req.body}}")
        || value.contains("{{sanitizeHtml:req.pathParam:")
        || value.contains("{{sanitizeHtml:req.header:")
        || value.contains("{{sanitizeHtml:req.query:")
        || value.contains("{{sanitizeHtml:req.cookie:")
        || value.contains("{{sanitizeHtml:req.method}}")
        || value.contains("{{sanitizeHtml:req.path}}")
        || value.contains("{{sanitizeHtml:req.httpVersion}}")
        || value.contains("{{sanitizeHtml:req.body}}")
}

pub(crate) fn escape_lasm_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub(crate) fn materialize_lasm_request_placeholders(
    value: &str,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    trace_id: &str,
) -> String {
    let with_method = value.replace("{{req.method}}", request.method.as_str());
    let with_path = with_method.replace("{{req.path}}", request.path.as_str());
    let with_http_version = with_path.replace("{{req.httpVersion}}", request.http_version.as_str());
    let request_body = String::from_utf8_lossy(&request.body);
    let with_body = with_http_version.replace("{{req.body}}", request_body.as_ref());
    let with_path_params =
        replace_lasm_response_placeholder_tokens(&with_body, "{{req.pathParam:", |key| {
            path_params.get(key.trim()).cloned()
        });
    let with_headers =
        replace_lasm_response_placeholder_tokens(&with_path_params, "{{req.header:", |key| {
            find_lasm_header_value(&request.headers, key.trim()).map(ToOwned::to_owned)
        });
    let with_cookies =
        replace_lasm_response_placeholder_tokens(&with_headers, "{{req.cookie:", |key| {
            find_lasm_cookie_value(&request.headers, key.trim())
        });
    let with_queries =
        replace_lasm_response_placeholder_tokens(&with_cookies, "{{req.query:", |key| {
            resolve_lasm_request_query_value(key.trim(), request, path_params, trace_id)
        });

    let escaped_method = escape_lasm_html(request.method.as_str());
    let escaped_path = escape_lasm_html(request.path.as_str());
    let escaped_http_version = escape_lasm_html(request.http_version.as_str());
    let escaped_body = escape_lasm_html(request_body.as_ref());

    let with_sanitized_method =
        with_queries.replace("{{sanitizeHtml:req.method}}", escaped_method.as_str());
    let with_sanitized_path =
        with_sanitized_method.replace("{{sanitizeHtml:req.path}}", escaped_path.as_str());
    let with_sanitized_http_version = with_sanitized_path.replace(
        "{{sanitizeHtml:req.httpVersion}}",
        escaped_http_version.as_str(),
    );
    let with_sanitized_body =
        with_sanitized_http_version.replace("{{sanitizeHtml:req.body}}", escaped_body.as_str());
    let with_sanitized_path_params = replace_lasm_response_placeholder_tokens(
        &with_sanitized_body,
        "{{sanitizeHtml:req.pathParam:",
        |key| {
            path_params
                .get(key.trim())
                .map(|value| escape_lasm_html(value.as_str()))
        },
    );
    let with_sanitized_headers = replace_lasm_response_placeholder_tokens(
        &with_sanitized_path_params,
        "{{sanitizeHtml:req.header:",
        |key| find_lasm_header_value(&request.headers, key.trim()).map(escape_lasm_html),
    );
    let with_sanitized_cookies = replace_lasm_response_placeholder_tokens(
        &with_sanitized_headers,
        "{{sanitizeHtml:req.cookie:",
        |key| {
            find_lasm_cookie_value(&request.headers, key.trim())
                .map(|value| escape_lasm_html(value.as_str()))
        },
    );
    replace_lasm_response_placeholder_tokens(
        &with_sanitized_cookies,
        "{{sanitizeHtml:req.query:",
        |key| {
            resolve_lasm_request_query_value(key.trim(), request, path_params, trace_id)
                .map(|value| escape_lasm_html(value.as_str()))
        },
    )
}

pub(crate) fn resolve_lasm_request_query_value(
    key: &str,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    trace_id: &str,
) -> Option<String> {
    request
        .query_params
        .get(key)
        .cloned()
        .or_else(|| resolve_lasm_workbench_query_body_fallback(key, request, path_params, trace_id))
}

pub(crate) fn augment_lasm_workbench_query_params_from_body(
    method: &str,
    path: &str,
    headers: &BTreeMap<String, String>,
    body: &[u8],
    query_params: &mut BTreeMap<String, String>,
) {
    augment_lasm_workbench_public_query_defaults(method, path, query_params);

    if method.eq_ignore_ascii_case("POST") && path == "/wb/tasks" {
        let payload = parse_lasm_workbench_body_object(headers, body);
        let seed = payload.as_ref().map(|_| lasm_workbench_request_seed());
        if !query_params.contains_key("params") {
            if let (Some(payload), Some(seed)) = (payload.as_ref(), seed.as_ref()) {
                if let Some(params) = build_lasm_workbench_create_task_params(payload, seed) {
                    query_params.insert("params".to_string(), params);
                    query_params.insert(
                        LASM_WORKBENCH_PUBLIC_REQUEST_MARKER.to_string(),
                        "1".to_string(),
                    );
                }
            }
        }
        if !query_params.contains_key("label_params") {
            if let Some(label_params) =
                build_lasm_workbench_create_task_label_params(payload.as_ref(), query_params)
            {
                query_params.insert("label_params".to_string(), label_params);
            }
        }
        return;
    }

    if method.eq_ignore_ascii_case("POST")
        && (path == "/wb/tasks/with-comment" || path == "/wb/tasks/with-comment-tx")
    {
        let payload = parse_lasm_workbench_body_object(headers, body);
        let seed = payload.as_ref().map(|_| lasm_workbench_request_seed());
        let tx_payload_params = match (payload.as_ref(), seed.as_ref()) {
            (Some(payload), Some(seed)) => build_lasm_workbench_tx_payload_params(payload, seed),
            _ => None,
        };
        if !query_params.contains_key("params") {
            if path == "/wb/tasks/with-comment" {
                if let Some(params) = tx_payload_params
                    .as_ref()
                    .map(|params| &params.combined_params)
                {
                    query_params.insert("params".to_string(), params.clone());
                    query_params.insert(
                        LASM_WORKBENCH_PUBLIC_REQUEST_MARKER.to_string(),
                        "1".to_string(),
                    );
                }
                if !query_params.contains_key("params") {
                    if let Some(params) =
                        build_lasm_workbench_tx_combined_params_from_query(query_params)
                    {
                        query_params.insert("params".to_string(), params);
                    }
                }
            }
        }
        if !query_params.contains_key("task_params") {
            if let Some(params) = tx_payload_params.as_ref().map(|params| &params.task_params) {
                query_params.insert("task_params".to_string(), params.clone());
                query_params.insert(
                    LASM_WORKBENCH_PUBLIC_REQUEST_MARKER.to_string(),
                    "1".to_string(),
                );
            }
        }
        if !query_params.contains_key("comment_params") {
            if let Some(params) = tx_payload_params
                .as_ref()
                .map(|params| &params.comment_params)
            {
                query_params.insert("comment_params".to_string(), params.clone());
                query_params.insert(
                    LASM_WORKBENCH_PUBLIC_REQUEST_MARKER.to_string(),
                    "1".to_string(),
                );
            }
        }
        return;
    }

    if method.eq_ignore_ascii_case("POST")
        && path.starts_with("/wb/tasks/")
        && path.ends_with("/comments")
        && !query_params.contains_key("params")
    {
        let payload = parse_lasm_workbench_body_object(headers, body);
        let seed = payload.as_ref().map(|_| lasm_workbench_request_seed());
        if let (Some(payload), Some(seed)) = (payload.as_ref(), seed.as_ref()) {
            if let Some(task_id) = extract_lasm_workbench_comment_path_task_id(path) {
                let mut path_params = BTreeMap::new();
                path_params.insert("id".to_string(), task_id);
                if let Some(params) =
                    build_lasm_workbench_add_comment_params(payload, &path_params, seed)
                {
                    query_params.insert("params".to_string(), params);
                    query_params.insert(
                        LASM_WORKBENCH_PUBLIC_REQUEST_MARKER.to_string(),
                        "1".to_string(),
                    );
                }
            }
        }
    }
}

fn parse_lasm_workbench_body_object(
    headers: &BTreeMap<String, String>,
    body: &[u8],
) -> Option<Map<String, Value>> {
    if body.is_empty() || !lasm_headers_expect_json(headers) {
        return None;
    }
    parse_lasm_body_object(body)
}

fn augment_lasm_workbench_public_query_defaults(
    method: &str,
    path: &str,
    query_params: &mut BTreeMap<String, String>,
) {
    if method.eq_ignore_ascii_case("GET")
        && path.starts_with("/wb/tasks/")
        && !path.ends_with("/comments")
    {
        query_params
            .entry("row_schema".to_string())
            .or_insert_with(|| "1".to_string());
    }

    if method.eq_ignore_ascii_case("GET") && path == "/wb/tasks" {
        query_params
            .entry("row_schema".to_string())
            .or_insert_with(|| "1".to_string());
        if let Some(existing) = query_params.get("params").cloned() {
            if let Some(expanded) = expand_lasm_workbench_list_params(existing.as_str()) {
                if expanded != existing {
                    query_params.insert("params".to_string(), expanded);
                }
            }
        } else if let Some(params) = build_lasm_workbench_list_params_from_query(query_params) {
            query_params.insert("params".to_string(), params);
            query_params.insert(
                LASM_WORKBENCH_PUBLIC_REQUEST_MARKER.to_string(),
                "1".to_string(),
            );
        }
    }
}

fn resolve_lasm_workbench_query_body_fallback(
    key: &str,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    trace_id: &str,
) -> Option<String> {
    if request.body.is_empty() || !lasm_request_content_type_is_json(request) {
        return None;
    }
    let payload = parse_lasm_request_body_object(request)?;
    let method = request.method.as_str();
    let path = request.path.as_str();

    if method.eq_ignore_ascii_case("POST") && path == "/wb/tasks" {
        return match key {
            "params" => build_lasm_workbench_create_task_params(&payload, trace_id),
            "label_params" => {
                build_lasm_workbench_create_task_label_params(Some(&payload), &request.query_params)
            }
            _ => None,
        };
    }

    if method.eq_ignore_ascii_case("POST")
        && (path == "/wb/tasks/with-comment" || path == "/wb/tasks/with-comment-tx")
    {
        return match key {
            "params" if path == "/wb/tasks/with-comment" => {
                build_lasm_workbench_tx_combined_params_from_payload(&payload, trace_id).or_else(
                    || build_lasm_workbench_tx_combined_params_from_query(&request.query_params),
                )
            }
            "task_params" => build_lasm_workbench_tx_task_params(&payload, trace_id),
            "comment_params" => build_lasm_workbench_tx_comment_params(&payload, trace_id),
            _ => None,
        };
    }

    if method.eq_ignore_ascii_case("POST")
        && path.starts_with("/wb/tasks/")
        && path.ends_with("/comments")
    {
        return match key {
            "params" => build_lasm_workbench_add_comment_params(&payload, path_params, trace_id),
            _ => None,
        };
    }

    None
}

fn lasm_request_content_type_is_json(request: &LasmRunRequest) -> bool {
    lasm_headers_expect_json(&request.headers)
}

fn lasm_headers_expect_json(headers: &BTreeMap<String, String>) -> bool {
    find_lasm_header_value(headers, "Content-Type")
        .map(|value| value.to_ascii_lowercase().contains("application/json"))
        .unwrap_or(false)
}

fn parse_lasm_request_body_object(request: &LasmRunRequest) -> Option<Map<String, Value>> {
    parse_lasm_body_object(&request.body)
}

fn parse_lasm_body_object(body: &[u8]) -> Option<Map<String, Value>> {
    let parsed = serde_json::from_slice::<Value>(body).ok()?;
    parsed.as_object().cloned()
}

fn extract_lasm_workbench_comment_path_task_id(path: &str) -> Option<String> {
    let mut segments = path.split('/');
    let _ = segments.next();
    let wb = segments.next()?;
    let tasks = segments.next()?;
    let task_id = segments.next()?;
    let comments = segments.next()?;
    if wb != "wb" || tasks != "tasks" || comments != "comments" || task_id.trim().is_empty() {
        return None;
    }
    Some(task_id.trim().to_string())
}

fn build_lasm_workbench_create_task_params(
    payload: &Map<String, Value>,
    trace_id: &str,
) -> Option<String> {
    let title = workbench_required_string(payload, "title")?;
    let description = workbench_optional_string(payload, "description");
    let status = {
        let value = workbench_optional_string(payload, "status");
        if value.trim().is_empty() {
            "open".to_string()
        } else {
            value
        }
    };
    let priority = workbench_optional_i64(payload, "priority").unwrap_or(3);
    let created_at_ms = lasm_workbench_generated_created_at_ms(trace_id, 0);
    serde_json::to_string(&serde_json::json!([
        lasm_workbench_generated_task_id(trace_id),
        title,
        description,
        status,
        priority,
        created_at_ms
    ]))
    .ok()
}

fn build_lasm_workbench_create_task_label_params(
    payload: Option<&Map<String, Value>>,
    query_params: &BTreeMap<String, String>,
) -> Option<String> {
    let task_id = query_params
        .get("params")
        .and_then(|raw| parse_lasm_flat_json_array_string_element(raw.as_str(), 0))?;
    let labels = payload
        .and_then(|object| object.get("labels"))
        .and_then(normalize_lasm_workbench_labels_value)
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let labels_json = serde_json::to_string(&labels).ok()?;
    serde_json::to_string(&serde_json::json!([task_id, labels_json])).ok()
}

fn build_lasm_workbench_tx_task_params(
    payload: &Map<String, Value>,
    trace_id: &str,
) -> Option<String> {
    build_lasm_workbench_tx_payload_params(payload, trace_id).map(|params| params.task_params)
}

fn build_lasm_workbench_tx_comment_params(
    payload: &Map<String, Value>,
    trace_id: &str,
) -> Option<String> {
    build_lasm_workbench_tx_payload_params(payload, trace_id).map(|params| params.comment_params)
}

struct LasmWorkbenchTxPayloadParams {
    task_params: String,
    comment_params: String,
    combined_params: String,
}

fn build_lasm_workbench_tx_payload_params(
    payload: &Map<String, Value>,
    trace_id: &str,
) -> Option<LasmWorkbenchTxPayloadParams> {
    let task = payload.get("task")?.as_object()?;
    let comment = payload.get("comment")?.as_object()?;
    let title = workbench_required_string(task, "title")?;
    let description = workbench_optional_string(task, "description");
    let status = {
        let value = workbench_optional_string(task, "status");
        if value.trim().is_empty() {
            "open".to_string()
        } else {
            value
        }
    };
    let priority = workbench_optional_i64(task, "priority").unwrap_or(3);
    let body = workbench_required_string(comment, "body")?;
    let task_id = lasm_workbench_generated_task_id(trace_id);
    let comment_id = lasm_workbench_generated_comment_id(trace_id);
    let task_created_at_ms = lasm_workbench_generated_created_at_ms(trace_id, 0);
    let comment_created_at_ms = lasm_workbench_generated_created_at_ms(trace_id, 1);
    let task_params = serde_json::to_string(&serde_json::json!([
        task_id.clone(),
        title.clone(),
        description.clone(),
        status.clone(),
        priority,
        task_created_at_ms
    ]))
    .ok()?;
    let comment_params = serde_json::to_string(&serde_json::json!([
        comment_id.clone(),
        task_id.clone(),
        body.clone(),
        comment_created_at_ms
    ]))
    .ok()?;
    let combined_params = serde_json::to_string(&serde_json::json!([
        task_id,
        title,
        description,
        status,
        priority,
        task_created_at_ms,
        comment_id,
        body,
        comment_created_at_ms
    ]))
    .ok()?;
    Some(LasmWorkbenchTxPayloadParams {
        task_params,
        comment_params,
        combined_params,
    })
}

fn build_lasm_workbench_tx_combined_params_from_payload(
    payload: &Map<String, Value>,
    trace_id: &str,
) -> Option<String> {
    build_lasm_workbench_tx_payload_params(payload, trace_id).map(|params| params.combined_params)
}

fn build_lasm_workbench_tx_combined_params_from_query(
    query_params: &BTreeMap<String, String>,
) -> Option<String> {
    let task_params = query_params.get("task_params")?.as_str();
    let comment_params = query_params.get("comment_params")?.as_str();
    let task_values = parse_lasm_flat_json_array_elements(task_params)?;
    let comment_values = parse_lasm_flat_json_array_elements(comment_params)?;
    if task_values.len() < 6 || comment_values.len() < 4 {
        return None;
    }

    let selected = [
        task_values[0],
        task_values[1],
        task_values[2],
        task_values[3],
        task_values[4],
        task_values[5],
        comment_values[0],
        comment_values[2],
        comment_values[3],
    ];

    let mut combined =
        String::with_capacity(task_params.len() + comment_params.len().saturating_sub(4));
    combined.push('[');
    for (index, value) in selected.iter().enumerate() {
        if index > 0 {
            combined.push(',');
        }
        combined.push_str(value.trim());
    }
    combined.push(']');
    Some(combined)
}

fn build_lasm_workbench_add_comment_params(
    payload: &Map<String, Value>,
    path_params: &BTreeMap<String, String>,
    trace_id: &str,
) -> Option<String> {
    let body = workbench_required_string(payload, "body")?;
    let task_id = path_params
        .get("id")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())?
        .to_string();
    let created_at_ms = lasm_workbench_generated_created_at_ms(trace_id, 1);
    serde_json::to_string(&serde_json::json!([
        lasm_workbench_generated_comment_id(trace_id),
        task_id,
        body,
        created_at_ms
    ]))
    .ok()
}

fn build_lasm_workbench_list_params_from_query(
    query_params: &BTreeMap<String, String>,
) -> Option<String> {
    let status = query_params
        .get("status")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    let priority_min = query_params
        .get("priorityMin")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    let priority_max = query_params
        .get("priorityMax")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    let label = query_params
        .get("label")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    let limit = query_params
        .get("limit")
        .and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or(20);
    let offset = query_params
        .get("offset")
        .and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or(0);
    serde_json::to_string(&serde_json::json!([
        status,
        priority_min,
        priority_max,
        label,
        limit,
        offset,
    ]))
    .ok()
}

fn expand_lasm_workbench_list_params(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let values = parse_lasm_flat_json_array_elements(trimmed)?;
    match values.as_slice() {
        [status, limit, offset] => Some(format!("[{status},\"\",\"\",\"\",{limit},{offset}]")),
        [_, _, _, _, _, _] => Some(trimmed.to_string()),
        _ => None,
    }
}

pub(crate) fn parse_lasm_flat_json_array_elements(raw: &str) -> Option<Vec<&str>> {
    let raw = raw.trim();
    if raw.len() < 2 || !raw.starts_with('[') || !raw.ends_with(']') {
        return None;
    }

    let bytes = raw.as_bytes();
    let mut cursor = 1usize;
    let end = raw.len() - 1;
    let mut values = Vec::new();

    while cursor < end {
        while cursor < end && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= end {
            break;
        }

        let value_start = cursor;
        if bytes[cursor] == b'"' {
            cursor += 1;
            let mut escaped = false;
            while cursor < end {
                let byte = bytes[cursor];
                cursor += 1;
                if escaped {
                    escaped = false;
                    continue;
                }
                if byte == b'\\' {
                    escaped = true;
                    continue;
                }
                if byte == b'"' {
                    break;
                }
            }
            if cursor > end || bytes[cursor.saturating_sub(1)] != b'"' {
                return None;
            }
        } else {
            while cursor < end && bytes[cursor] != b',' && bytes[cursor] != b']' {
                cursor += 1;
            }
        }

        let value_end = cursor;
        let value = raw.get(value_start..value_end)?.trim();
        if value.is_empty() {
            return None;
        }
        values.push(value);

        while cursor < end && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor < end {
            if bytes[cursor] != b',' {
                return None;
            }
            cursor += 1;
        }
    }

    Some(values)
}

pub(crate) fn parse_lasm_flat_json_array_string_element(raw: &str, index: usize) -> Option<String> {
    let values = parse_lasm_flat_json_array_elements(raw)?;
    let value = values.get(index)?.trim();
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return None;
    }
    serde_json::from_str::<String>(value).ok()
}

fn normalize_lasm_workbench_labels_value(value: &Value) -> Option<Value> {
    match value {
        Value::Array(values) => Some(Value::Array(
            values
                .iter()
                .filter_map(|entry| entry.as_str().map(str::trim))
                .filter(|entry| !entry.is_empty())
                .map(|entry| Value::String(entry.to_string()))
                .collect(),
        )),
        Value::Null => Some(Value::Array(Vec::new())),
        _ => None,
    }
}

fn workbench_required_string(payload: &Map<String, Value>, key: &str) -> Option<String> {
    let value = payload.get(key)?.as_str()?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn workbench_optional_string(payload: &Map<String, Value>, key: &str) -> String {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

fn workbench_optional_i64(payload: &Map<String, Value>, key: &str) -> Option<i64> {
    let value = payload.get(key)?;
    value
        .as_i64()
        .or_else(|| {
            value
                .as_u64()
                .and_then(|candidate| i64::try_from(candidate).ok())
        })
        .or_else(|| {
            value
                .as_str()
                .and_then(|candidate| candidate.trim().parse::<i64>().ok())
        })
}

fn lasm_workbench_generated_task_id(trace_id: &str) -> String {
    format!("wb-task-{trace_id}")
}

fn lasm_workbench_generated_comment_id(trace_id: &str) -> String {
    format!("wb-comment-{trace_id}")
}

fn lasm_workbench_generated_created_at_ms(trace_id: &str, offset: u64) -> u64 {
    let base = trace_id
        .rsplit('-')
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .or_else(|| trace_id.parse::<u64>().ok())
        .unwrap_or_else(lasm_workbench_now_ms);
    base.saturating_add(offset)
}

fn lasm_workbench_request_seed() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    nanos.to_string()
}

fn lasm_workbench_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn replace_lasm_response_placeholder_tokens(
    body: &str,
    prefix: &str,
    resolve: impl Fn(&str) -> Option<String>,
) -> String {
    let mut output = String::with_capacity(body.len());
    let mut rest = body;
    loop {
        let Some(start) = rest.find(prefix) else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..start]);
        let value_start = start + prefix.len();
        let after_value_start = &rest[value_start..];
        let Some(value_end) = after_value_start.find("}}") else {
            output.push_str(&rest[start..]);
            break;
        };
        let token_value = &after_value_start[..value_end];
        if let Some(value) = resolve(token_value) {
            output.push_str(value.as_str());
        }
        rest = &after_value_start[value_end + 2..];
    }
    output
}
