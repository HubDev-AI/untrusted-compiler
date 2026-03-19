use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LasmResourcePlan {
    pub name: String,
    pub table: String,
    pub fields: Vec<LasmResourceFieldPlan>,
    pub route_prefix: String,
    pub max_list_limit: u32,
    pub default_list_limit: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LasmResourceFieldPlan {
    pub name: String,
    pub field_type: String,
    pub primary: bool,
    pub auto_fill: bool,
    pub default_value: Option<String>,
}

// ---------------------------------------------------------------------------
// Header constants
// ---------------------------------------------------------------------------

const RESOURCE_OP_HEADER: &str = "X-Sec4-Internal-Resource-Op";
const RESOURCE_PLAN_HEADER: &str = "X-Sec4-Internal-Resource-Plan";

// ---------------------------------------------------------------------------
// Route detection + plan parsing
// ---------------------------------------------------------------------------

pub fn is_resource_route(headers: &BTreeMap<String, String>) -> bool {
    headers.contains_key(RESOURCE_OP_HEADER)
}

pub fn parse_resource_plan(
    headers: &BTreeMap<String, String>,
) -> Option<(String, LasmResourcePlan)> {
    let op = headers.get(RESOURCE_OP_HEADER)?.clone();
    let plan_json = headers.get(RESOURCE_PLAN_HEADER)?;
    let plan: LasmResourcePlan = serde_json::from_str(plan_json).ok()?;
    Some((op, plan))
}

// ---------------------------------------------------------------------------
// Main dispatch entry point
// ---------------------------------------------------------------------------

/// Returns `true` if the request was handled as a resource operation.
pub fn try_dispatch_resource_operation(
    response: &mut sec4_core::HttpResponse,
    request_body: &[u8],
    path_params: &BTreeMap<String, String>,
    query_params: &BTreeMap<String, String>,
    route_headers: &BTreeMap<String, String>,
    trace_id: &str,
) -> bool {
    let Some((op, plan)) = parse_resource_plan(route_headers) else {
        return false;
    };

    match op.as_str() {
        "create" => dispatch_create(response, request_body, &plan, trace_id),
        "get" => dispatch_get(response, path_params, &plan, trace_id),
        "list" => dispatch_list(response, query_params, &plan, trace_id),
        "update" => dispatch_update(response, request_body, path_params, &plan, trace_id),
        "delete" => dispatch_delete(response, path_params, &plan, trace_id),
        _ => {
            set_json_response(
                response,
                400,
                &error_envelope(
                    400,
                    "RESOURCE.UNKNOWN_OP",
                    "validation",
                    &format!("unknown resource operation: {op}"),
                    trace_id,
                ),
            );
        }
    }

    // Clean internal headers so they never leak to the client.
    response.headers.remove(RESOURCE_OP_HEADER);
    response.headers.remove(RESOURCE_PLAN_HEADER);
    true
}

// ---------------------------------------------------------------------------
// CRUD dispatch functions
// ---------------------------------------------------------------------------

fn dispatch_create(
    response: &mut sec4_core::HttpResponse,
    request_body: &[u8],
    plan: &LasmResourcePlan,
    trace_id: &str,
) {
    let body: Value = match serde_json::from_slice(request_body) {
        Ok(v) => v,
        Err(_) => {
            set_json_response(
                response,
                400,
                &error_envelope(
                    400,
                    "RESOURCE.INVALID_JSON",
                    "validation",
                    "request body is not valid JSON",
                    trace_id,
                ),
            );
            return;
        }
    };

    let obj = match body.as_object() {
        Some(o) => o,
        None => {
            set_json_response(
                response,
                400,
                &error_envelope(
                    400,
                    "RESOURCE.INVALID_BODY",
                    "validation",
                    "request body must be a JSON object",
                    trace_id,
                ),
            );
            return;
        }
    };

    // Build the final row values, validating and filling as we go.
    let mut columns: Vec<String> = Vec::new();
    let mut values: Vec<Value> = Vec::new();

    for field in &plan.fields {
        if field.primary && field.auto_fill {
            // Auto-generated primary key
            let generated = auto_fill_value(&field.field_type);
            columns.push(field.name.clone());
            values.push(generated);
            continue;
        }

        if field.auto_fill {
            let generated = auto_fill_value(&field.field_type);
            columns.push(field.name.clone());
            values.push(generated);
            continue;
        }

        if let Some(provided) = obj.get(&field.name) {
            if let Err(msg) = validate_field(field, provided) {
                set_json_response(
                    response,
                    400,
                    &error_envelope(
                        400,
                        "RESOURCE.VALIDATION",
                        "validation",
                        &format!("field '{}': {}", field.name, msg),
                        trace_id,
                    ),
                );
                return;
            }
            columns.push(field.name.clone());
            values.push(provided.clone());
            continue;
        }

        // Field not provided — try default, else skip if primary (already handled) or error.
        if let Some(ref default) = field.default_value {
            columns.push(field.name.clone());
            values.push(Value::String(default.clone()));
            continue;
        }

        if field.primary {
            // Primary without auto and without value — error.
            set_json_response(
                response,
                400,
                &error_envelope(
                    400,
                    "RESOURCE.MISSING_FIELD",
                    "validation",
                    &format!("missing required primary field '{}'", field.name),
                    trace_id,
                ),
            );
            return;
        }

        // Non-required, non-auto, no default, not provided — skip.
    }

    match generate_create_sql(plan, &columns, &values) {
        Ok((sql, params)) => {
            set_json_response(
                response,
                201,
                &success_envelope(201, &json!({ "sql": sql, "params": params }), trace_id),
            );
        }
        Err(err_payload) => {
            set_json_response(response, 400, &err_payload);
        }
    }
}

fn dispatch_get(
    response: &mut sec4_core::HttpResponse,
    path_params: &BTreeMap<String, String>,
    plan: &LasmResourcePlan,
    trace_id: &str,
) {
    let Some(id) = path_params.get("id") else {
        set_json_response(
            response,
            400,
            &error_envelope(
                400,
                "RESOURCE.MISSING_ID",
                "validation",
                "missing :id path parameter",
                trace_id,
            ),
        );
        return;
    };

    if let Err(msg) = validate_id_param(plan, id) {
        set_json_response(
            response,
            400,
            &error_envelope(400, "RESOURCE.INVALID_ID", "validation", &msg, trace_id),
        );
        return;
    }

    let sql = generate_get_sql(plan);
    set_json_response(
        response,
        200,
        &success_envelope(
            200,
            &json!({ "sql": sql, "params": [id] }),
            trace_id,
        ),
    );
}

fn dispatch_list(
    response: &mut sec4_core::HttpResponse,
    query_params: &BTreeMap<String, String>,
    plan: &LasmResourcePlan,
    trace_id: &str,
) {
    // Parse limit/offset from request query params, clamping to policy bounds.
    let raw_limit = query_params
        .get("limit")
        .and_then(|v| v.parse::<u64>().ok());
    let limit = match raw_limit {
        Some(v) => v.max(1).min(plan.max_list_limit as u64),
        None => plan.default_list_limit as u64,
    };
    let offset = query_params
        .get("offset")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let sql = generate_list_sql(plan);
    set_json_response(
        response,
        200,
        &success_envelope(
            200,
            &json!({ "sql": sql, "params": [limit, offset] }),
            trace_id,
        ),
    );
}

fn dispatch_update(
    response: &mut sec4_core::HttpResponse,
    request_body: &[u8],
    path_params: &BTreeMap<String, String>,
    plan: &LasmResourcePlan,
    trace_id: &str,
) {
    let Some(id) = path_params.get("id") else {
        set_json_response(
            response,
            400,
            &error_envelope(
                400,
                "RESOURCE.MISSING_ID",
                "validation",
                "missing :id path parameter",
                trace_id,
            ),
        );
        return;
    };

    if let Err(msg) = validate_id_param(plan, id) {
        set_json_response(
            response,
            400,
            &error_envelope(400, "RESOURCE.INVALID_ID", "validation", &msg, trace_id),
        );
        return;
    }

    let body: Value = match serde_json::from_slice(request_body) {
        Ok(v) => v,
        Err(_) => {
            set_json_response(
                response,
                400,
                &error_envelope(
                    400,
                    "RESOURCE.INVALID_JSON",
                    "validation",
                    "request body is not valid JSON",
                    trace_id,
                ),
            );
            return;
        }
    };

    let obj = match body.as_object() {
        Some(o) => o,
        None => {
            set_json_response(
                response,
                400,
                &error_envelope(
                    400,
                    "RESOURCE.INVALID_BODY",
                    "validation",
                    "request body must be a JSON object",
                    trace_id,
                ),
            );
            return;
        }
    };

    match generate_update_sql(plan, obj) {
        Ok((sql, mut params)) => {
            // Append the id as the final param (WHERE clause).
            params.push(Value::String(id.clone()));
            set_json_response(
                response,
                200,
                &success_envelope(200, &json!({ "sql": sql, "params": params }), trace_id),
            );
        }
        Err(err_payload) => {
            set_json_response(response, 400, &err_payload);
        }
    }
}

fn dispatch_delete(
    response: &mut sec4_core::HttpResponse,
    path_params: &BTreeMap<String, String>,
    plan: &LasmResourcePlan,
    trace_id: &str,
) {
    let Some(id) = path_params.get("id") else {
        set_json_response(
            response,
            400,
            &error_envelope(
                400,
                "RESOURCE.MISSING_ID",
                "validation",
                "missing :id path parameter",
                trace_id,
            ),
        );
        return;
    };

    if let Err(msg) = validate_id_param(plan, id) {
        set_json_response(
            response,
            400,
            &error_envelope(400, "RESOURCE.INVALID_ID", "validation", &msg, trace_id),
        );
        return;
    }

    let sql = generate_delete_sql(plan);
    set_json_response(
        response,
        200,
        &success_envelope(
            200,
            &json!({ "sql": sql, "params": [id] }),
            trace_id,
        ),
    );
}

// ---------------------------------------------------------------------------
// SQL generation
// ---------------------------------------------------------------------------

fn generate_create_sql(
    plan: &LasmResourcePlan,
    columns: &[String],
    values: &[Value],
) -> Result<(String, Vec<Value>), Value> {
    if columns.is_empty() {
        return Err(json!({
            "error": {
                "code": "RESOURCE.EMPTY_INSERT",
                "kind": "validation",
                "message": "no columns to insert",
            }
        }));
    }

    let col_list: Vec<String> = columns.iter().map(|c| quote_ident(c)).collect();
    let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("${i}")).collect();

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
        quote_ident(&plan.table),
        col_list.join(", "),
        placeholders.join(", "),
    );

    Ok((sql, values.to_vec()))
}

fn generate_get_sql(plan: &LasmResourcePlan) -> String {
    let pk = primary_key_column(plan);
    format!(
        "SELECT * FROM {} WHERE {} = $1 LIMIT 1",
        quote_ident(&plan.table),
        quote_ident(&pk),
    )
}

fn generate_list_sql(plan: &LasmResourcePlan) -> String {
    let pk = primary_key_column(plan);
    format!(
        "SELECT * FROM {} ORDER BY {} LIMIT $1 OFFSET $2",
        quote_ident(&plan.table),
        quote_ident(&pk),
    )
}

fn generate_update_sql(
    plan: &LasmResourcePlan,
    body: &serde_json::Map<String, Value>,
) -> Result<(String, Vec<Value>), Value> {
    let mut set_clauses: Vec<String> = Vec::new();
    let mut params: Vec<Value> = Vec::new();
    let mut param_idx = 1usize;

    for field in &plan.fields {
        if field.primary {
            continue; // Never update the primary key.
        }
        if let Some(value) = body.get(&field.name) {
            if let Err(msg) = validate_field(field, value) {
                return Err(json!({
                    "error": {
                        "code": "RESOURCE.VALIDATION",
                        "kind": "validation",
                        "message": format!("field '{}': {}", field.name, msg),
                    }
                }));
            }
            set_clauses.push(format!("{} = ${param_idx}", quote_ident(&field.name)));
            params.push(value.clone());
            param_idx += 1;
        }
    }

    if set_clauses.is_empty() {
        return Err(json!({
            "error": {
                "code": "RESOURCE.EMPTY_UPDATE",
                "kind": "validation",
                "message": "no fields to update",
            }
        }));
    }

    let pk = primary_key_column(plan);
    let sql = format!(
        "UPDATE {} SET {} WHERE {} = ${param_idx} RETURNING *",
        quote_ident(&plan.table),
        set_clauses.join(", "),
        quote_ident(&pk),
    );

    Ok((sql, params))
}

fn generate_delete_sql(plan: &LasmResourcePlan) -> String {
    let pk = primary_key_column(plan);
    format!(
        "DELETE FROM {} WHERE {} = $1 RETURNING *",
        quote_ident(&plan.table),
        quote_ident(&pk),
    )
}

// ---------------------------------------------------------------------------
// Field validation
// ---------------------------------------------------------------------------

fn validate_field(field: &LasmResourceFieldPlan, value: &Value) -> Result<(), String> {
    match field.field_type.as_str() {
        "Uuid" => {
            let s = value
                .as_str()
                .ok_or_else(|| "expected string for Uuid".to_string())?;
            if !is_valid_uuid(s) {
                return Err("invalid UUID format (expected 8-4-4-4-12 hex)".to_string());
            }
            Ok(())
        }
        "Email" => {
            let s = value
                .as_str()
                .ok_or_else(|| "expected string for Email".to_string())?;
            if s.len() < 3 || !s.contains('@') {
                return Err("invalid email address".to_string());
            }
            Ok(())
        }
        "String" => {
            let s = value
                .as_str()
                .ok_or_else(|| "expected string".to_string())?;
            if s.is_empty() {
                return Err("string must not be empty".to_string());
            }
            Ok(())
        }
        "Int64" | "Int" => {
            if value.is_i64() {
                return Ok(());
            }
            if let Some(s) = value.as_str() {
                s.parse::<i64>()
                    .map_err(|_| "expected integer value".to_string())?;
                return Ok(());
            }
            Err("expected integer value".to_string())
        }
        "Bool" => {
            if value.is_boolean() {
                Ok(())
            } else {
                Err("expected boolean value".to_string())
            }
        }
        "Time" => {
            let s = value
                .as_str()
                .ok_or_else(|| "expected string for Time (ISO 8601)".to_string())?;
            if s.is_empty() {
                return Err("time string must not be empty".to_string());
            }
            Ok(())
        }
        _ => {
            // Unknown type — accept as-is.
            Ok(())
        }
    }
}

/// Validate the :id path parameter against the primary key type.
fn validate_id_param(plan: &LasmResourcePlan, id: &str) -> Result<(), String> {
    let pk_field = plan.fields.iter().find(|f| f.primary);
    if let Some(field) = pk_field {
        match field.field_type.as_str() {
            "Uuid" => {
                if !is_valid_uuid(id) {
                    return Err("invalid UUID format for :id (expected 8-4-4-4-12 hex)".to_string());
                }
            }
            "Int64" | "Int" => {
                id.parse::<i64>()
                    .map_err(|_| "invalid integer for :id".to_string())?;
            }
            _ => {}
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// UUID validation (manual, no uuid crate)
// ---------------------------------------------------------------------------

fn is_valid_uuid(value: &str) -> bool {
    // 8-4-4-4-12 hex digits with dashes: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    if value.len() != 36 {
        return false;
    }
    for (i, b) in value.as_bytes().iter().enumerate() {
        match i {
            8 | 13 | 18 | 23 => {
                if *b != b'-' {
                    return false;
                }
            }
            _ => {
                if !b.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Response envelope builders
// ---------------------------------------------------------------------------

fn success_envelope(status: u16, data: &Value, trace_id: &str) -> Value {
    json!({
        "ok": true,
        "status": status,
        "traceId": trace_id,
        "timeMs": epoch_ms(),
        "data": data,
    })
}

fn error_envelope(status: u16, code: &str, kind: &str, message: &str, trace_id: &str) -> Value {
    json!({
        "ok": false,
        "status": status,
        "traceId": trace_id,
        "timeMs": epoch_ms(),
        "error": {
            "code": code,
            "kind": kind,
            "message": message,
        }
    })
}

fn set_json_response(response: &mut sec4_core::HttpResponse, status: u16, payload: &Value) {
    response.status = status;
    response.headers.insert(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );
    response.body = serde_json::to_vec(payload).unwrap_or_else(|_| b"{}".to_vec());
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn quote_ident(name: &str) -> String {
    // SQL identifier quoting: wrap in double quotes, escape embedded quotes.
    let escaped = name.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn generate_uuid() -> String {
    // Timestamp-based UUID-like string (not a real v4 but unique enough for dry-run).
    let now = epoch_ms();
    let high = (now >> 32) as u32;
    let low = now as u32;
    let extra = (now.wrapping_mul(6364136223846793005).wrapping_add(1)) as u32;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        high,
        (low >> 16) & 0xffff,
        low & 0xffff,
        (extra >> 16) & 0xffff,
        extra as u64 | ((now & 0xffff_0000_0000) >> 8),
    )
}

fn epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn primary_key_column(plan: &LasmResourcePlan) -> String {
    plan.fields
        .iter()
        .find(|f| f.primary)
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "id".to_string())
}

fn auto_fill_value(field_type: &str) -> Value {
    match field_type {
        "Uuid" => Value::String(generate_uuid()),
        "Time" => Value::String(epoch_ms().to_string()),
        "Int64" | "Int" => Value::Number(serde_json::Number::from(epoch_ms() as i64)),
        _ => Value::String(generate_uuid()),
    }
}
