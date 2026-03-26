use crate::lasm_db_client::build_lasm_postgres_thread_local_config;
use crate::lasm_db_runtime_postgres::{
    run_lasm_postgres_exec_returning_one_thread_local, run_lasm_postgres_exec_thread_local,
    run_lasm_postgres_query_many_thread_local, run_lasm_postgres_query_one_thread_local,
    LasmPostgresParam,
};
use crate::{LasmDbRecordsAdapter, LasmDynamicResponseState};
use rusqlite::types::Value as SqliteValue;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Mutex;
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
    pub unique: bool,
    pub optional: bool,
}

// ---------------------------------------------------------------------------
// Header constants
// ---------------------------------------------------------------------------

const UUID_STRING_LEN: usize = 36;

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
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    db_adapter: LasmDbRecordsAdapter,
) -> bool {
    let Some((op, plan)) = parse_resource_plan(route_headers) else {
        return false;
    };

    // Build a DB executor if adapter is SQLite; otherwise fall back to dry-run SQL.
    let db_exec = ResourceDbExecutor::new(dynamic_state, db_adapter, &plan);

    match op.as_str() {
        "create" => dispatch_create(response, request_body, &plan, trace_id, &db_exec),
        "get" => dispatch_get(response, path_params, &plan, trace_id, &db_exec),
        "list" => dispatch_list(response, query_params, &plan, trace_id, &db_exec),
        "update" => dispatch_update(
            response,
            request_body,
            path_params,
            &plan,
            trace_id,
            &db_exec,
        ),
        "delete" => dispatch_delete(response, path_params, &plan, trace_id, &db_exec),
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
    db_exec: &ResourceDbExecutor,
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
            // Default values are validated at compile time (semantic.rs E5004).
            // Runtime trusts the compiler's validation — no re-validation needed.
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

        if field.optional {
            // Optional field absent from request — insert SQL NULL.
            columns.push(field.name.clone());
            values.push(Value::Null);
            continue;
        }

        // Non-optional, non-auto, no default, not provided — error.
        set_json_response(
            response,
            400,
            &error_envelope(
                400,
                "RESOURCE.MISSING_FIELD",
                "validation",
                &format!("missing required field '{}'", field.name),
                trace_id,
            ),
        );
        return;
    }

    match generate_create_sql(plan, &columns, &values) {
        Ok((sql, params)) => {
            match db_exec.exec_returning_one(&sql, &params) {
                DbExecResult::Row(row) => {
                    set_json_response(response, 201, &success_envelope(201, &row, trace_id));
                }
                DbExecResult::Error(msg) => {
                    let is_conflict = msg.to_lowercase().contains("unique")
                        || msg.to_lowercase().contains("duplicate")
                        || msg.to_lowercase().contains("constraint");
                    if is_conflict {
                        set_json_response(
                            response,
                            409,
                            &error_envelope(
                                409,
                                "RESOURCE.CONFLICT",
                                "conflict",
                                "duplicate value for unique field",
                                trace_id,
                            ),
                        );
                    } else {
                        set_json_response(
                            response,
                            500,
                            &error_envelope(500, "RESOURCE.DB_ERROR", "runtime", &msg, trace_id),
                        );
                    }
                }
                DbExecResult::DryRun => {
                    // Build a synthetic record from the column/value pairs for dry-run.
                    let mut record = serde_json::Map::new();
                    for (col, val) in columns.iter().zip(values.iter()) {
                        record.insert(col.clone(), val.clone());
                    }
                    set_json_response(
                        response,
                        201,
                        &success_envelope(201, &Value::Object(record), trace_id),
                    );
                }
            }
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
    db_exec: &ResourceDbExecutor,
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
    let params = vec![Value::String(id.clone())];
    match db_exec.query_one(&sql, &params) {
        DbExecResult::Row(row) => {
            set_json_response(response, 200, &success_envelope(200, &row, trace_id));
        }
        DbExecResult::Error(msg) => {
            if msg.contains("NOT_FOUND") {
                set_json_response(
                    response,
                    404,
                    &error_envelope(
                        404,
                        "RESOURCE.NOT_FOUND",
                        "not_found",
                        "resource not found",
                        trace_id,
                    ),
                );
            } else {
                set_json_response(
                    response,
                    500,
                    &error_envelope(500, "RESOURCE.DB_ERROR", "runtime", &msg, trace_id),
                );
            }
        }
        DbExecResult::DryRun => {
            set_json_response(
                response,
                200,
                &success_envelope(200, &json!({ "sql": sql, "params": [id] }), trace_id),
            );
        }
    }
}

fn dispatch_list(
    response: &mut sec4_core::HttpResponse,
    query_params: &BTreeMap<String, String>,
    plan: &LasmResourcePlan,
    trace_id: &str,
    db_exec: &ResourceDbExecutor,
) {
    // Parse limit/offset from request query params, clamping to policy bounds.
    // Limit is clamped to [1, max_list_limit]. Zero, negative, or non-numeric values
    // fall through to default_list_limit. This prevents unbounded queries.
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

    // --- Filtering ---
    // Reserved params that are not field filters.
    let reserved_params = ["limit", "offset", "sort", "order"];
    let mut where_clauses: Vec<String> = Vec::new();
    let mut where_params: Vec<Value> = Vec::new();
    let mut param_idx: usize = 1; // $1, $2, ... for WHERE; LIMIT and OFFSET appended last

    for (key, value) in query_params {
        if reserved_params.contains(&key.as_str()) {
            continue;
        }
        // Only filter on known resource fields.
        if let Some(field) = plan.fields.iter().find(|f| f.name == *key) {
            let json_value = Value::String(value.clone());
            if validate_field(field, &json_value).is_ok() {
                where_clauses.push(format!("{} = ${}", quote_ident(key), param_idx));
                where_params.push(json_value);
                param_idx += 1;
            }
        }
    }

    // --- Sorting ---
    let sort_field = query_params
        .get("sort")
        .and_then(|s| plan.fields.iter().find(|f| f.name == *s))
        .map(|f| f.name.clone());
    let sort_order = query_params
        .get("order")
        .map(|o| {
            if o.eq_ignore_ascii_case("asc") {
                "ASC"
            } else {
                "DESC"
            }
        })
        .unwrap_or("DESC");

    // --- Build SQL ---
    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };

    let default_order_column = primary_key_column(plan);
    let order_column = sort_field.as_deref().unwrap_or(&default_order_column);

    let limit_placeholder = param_idx;
    let offset_placeholder = param_idx + 1;

    let sql = format!(
        "SELECT * FROM {}{} ORDER BY {} {} LIMIT ${} OFFSET ${}",
        quote_ident(&plan.table),
        where_sql,
        quote_ident(order_column),
        sort_order,
        limit_placeholder,
        offset_placeholder,
    );

    // Combine params: where_params first, then limit, then offset.
    let mut params: Vec<Value> = where_params;
    params.push(Value::Number(serde_json::Number::from(limit)));
    params.push(Value::Number(serde_json::Number::from(offset)));

    match db_exec.query_many(&sql, &params) {
        DbExecResult::Row(rows_value) => {
            let items = rows_value.as_array().cloned().unwrap_or_default();
            let count = items.len();
            set_json_response(
                response,
                200,
                &success_envelope(
                    200,
                    &json!({ "items": items, "count": count, "limit": limit, "offset": offset }),
                    trace_id,
                ),
            );
        }
        DbExecResult::Error(msg) => {
            set_json_response(
                response,
                500,
                &error_envelope(500, "RESOURCE.DB_ERROR", "runtime", &msg, trace_id),
            );
        }
        DbExecResult::DryRun => {
            set_json_response(
                response,
                200,
                &success_envelope(200, &json!({ "sql": sql, "params": params }), trace_id),
            );
        }
    }
}

fn dispatch_update(
    response: &mut sec4_core::HttpResponse,
    request_body: &[u8],
    path_params: &BTreeMap<String, String>,
    plan: &LasmResourcePlan,
    trace_id: &str,
    db_exec: &ResourceDbExecutor,
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
            match db_exec.exec_returning_one(&sql, &params) {
                DbExecResult::Row(row) => {
                    set_json_response(response, 200, &success_envelope(200, &row, trace_id));
                }
                DbExecResult::Error(msg) => {
                    if msg.contains("NOT_FOUND") {
                        set_json_response(
                            response,
                            404,
                            &error_envelope(
                                404,
                                "RESOURCE.NOT_FOUND",
                                "not_found",
                                "resource not found",
                                trace_id,
                            ),
                        );
                    } else {
                        set_json_response(
                            response,
                            500,
                            &error_envelope(500, "RESOURCE.DB_ERROR", "runtime", &msg, trace_id),
                        );
                    }
                }
                DbExecResult::DryRun => {
                    set_json_response(
                        response,
                        200,
                        &success_envelope(200, &json!({ "sql": sql, "params": params }), trace_id),
                    );
                }
            }
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
    db_exec: &ResourceDbExecutor,
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
    let params = vec![Value::String(id.clone())];
    match db_exec.exec_returning_one(&sql, &params) {
        DbExecResult::Row(_row) => {
            set_json_response(
                response,
                200,
                &success_envelope(200, &json!({ "deleted": true }), trace_id),
            );
        }
        DbExecResult::Error(msg) => {
            if msg.contains("NOT_FOUND") {
                set_json_response(
                    response,
                    404,
                    &error_envelope(
                        404,
                        "RESOURCE.NOT_FOUND",
                        "not_found",
                        "resource not found",
                        trace_id,
                    ),
                );
            } else {
                set_json_response(
                    response,
                    500,
                    &error_envelope(500, "RESOURCE.DB_ERROR", "runtime", &msg, trace_id),
                );
            }
        }
        DbExecResult::DryRun => {
            set_json_response(
                response,
                200,
                &success_envelope(200, &json!({ "sql": sql, "params": [id] }), trace_id),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// DB execution layer
// ---------------------------------------------------------------------------

enum DbExecResult {
    /// Successful row(s) returned from the database.
    Row(Value),
    /// DB execution error message.
    Error(String),
    /// No DB adapter configured; caller should fall back to dry-run SQL envelope.
    DryRun,
}

/// Encapsulates access to the DB adapter for resource CRUD operations.
/// For SQLite, it holds a reference to the dynamic state mutex and ensures
/// the table exists before the first operation.
struct ResourceDbExecutor<'a> {
    dynamic_state: &'a Mutex<LasmDynamicResponseState>,
    adapter: LasmDbRecordsAdapter,
    table_ddl: Option<String>,
}

impl<'a> ResourceDbExecutor<'a> {
    fn new(
        dynamic_state: &'a Mutex<LasmDynamicResponseState>,
        adapter: LasmDbRecordsAdapter,
        plan: &LasmResourcePlan,
    ) -> Self {
        let table_ddl = match adapter {
            LasmDbRecordsAdapter::Sqlite => Some(generate_create_table_ddl(plan)),
            LasmDbRecordsAdapter::Postgres => Some(generate_create_table_ddl_postgres(plan)),
            _ => None,
        };
        ResourceDbExecutor {
            dynamic_state,
            adapter,
            table_ddl,
        }
    }

    fn is_live(&self) -> bool {
        self.adapter == LasmDbRecordsAdapter::Sqlite
            || self.adapter == LasmDbRecordsAdapter::Postgres
    }

    /// Ensure the table exists (dev convenience). Works for both SQLite and Postgres.
    fn ensure_table(&self) -> Result<(), String> {
        let Some(ref ddl) = self.table_ddl else {
            return Ok(());
        };
        match self.adapter {
            LasmDbRecordsAdapter::Sqlite => {
                let mut state = self
                    .dynamic_state
                    .lock()
                    .map_err(|_| "resource db: state lock unavailable".to_string())?;
                let conn = resource_sqlite_connection_mut(&mut state)?;
                conn.execute_batch(ddl)
                    .map_err(|err| format!("resource db: auto-create table failed: {err}"))
            }
            LasmDbRecordsAdapter::Postgres => {
                let config = {
                    let mut state = self
                        .dynamic_state
                        .lock()
                        .map_err(|_| "resource db: state lock unavailable".to_string())?;
                    build_lasm_postgres_thread_local_config(&mut state)
                        .map_err(|err| format!("resource db: postgres config unavailable: {err}"))?
                };
                run_lasm_postgres_exec_thread_local(&config, ddl, &[])
                    .map(|_| ())
                    .map_err(|err| format!("resource db: auto-create table failed: {err}"))
            }
            _ => Ok(()),
        }
    }

    /// Execute a DML statement with RETURNING * and return the first row.
    fn exec_returning_one(&self, sql: &str, params: &[Value]) -> DbExecResult {
        if !self.is_live() {
            return DbExecResult::DryRun;
        }
        if let Err(e) = self.ensure_table() {
            return DbExecResult::Error(e);
        }
        match self.adapter {
            LasmDbRecordsAdapter::Postgres => {
                let pg_params = json_values_to_postgres_params(params);
                let config = match self.resource_postgres_config() {
                    Ok(c) => c,
                    Err(e) => return DbExecResult::Error(e),
                };
                match run_lasm_postgres_exec_returning_one_thread_local(&config, sql, &pg_params) {
                    Ok(Some(row)) => DbExecResult::Row(row),
                    Ok(None) => DbExecResult::Error("NOT_FOUND".to_string()),
                    Err(e) => DbExecResult::Error(format!("resource db exec error: {e}")),
                }
            }
            LasmDbRecordsAdapter::Sqlite => {
                let sqlite_sql = postgres_placeholders_to_sqlite(sql);
                let sqlite_params = json_values_to_sqlite_params(params);
                let mut state = match self.dynamic_state.lock() {
                    Ok(s) => s,
                    Err(_) => {
                        return DbExecResult::Error(
                            "resource db: state lock unavailable".to_string(),
                        )
                    }
                };
                let conn = match resource_sqlite_connection_mut(&mut state) {
                    Ok(c) => c,
                    Err(e) => return DbExecResult::Error(e),
                };
                match resource_sqlite_query_one(conn, &sqlite_sql, &sqlite_params) {
                    Ok(Some(row)) => DbExecResult::Row(row),
                    Ok(None) => DbExecResult::Error("NOT_FOUND".to_string()),
                    Err(e) => DbExecResult::Error(format!("resource db exec error: {e}")),
                }
            }
            _ => DbExecResult::DryRun,
        }
    }

    /// Execute a SELECT query and return a single row.
    fn query_one(&self, sql: &str, params: &[Value]) -> DbExecResult {
        if !self.is_live() {
            return DbExecResult::DryRun;
        }
        if let Err(e) = self.ensure_table() {
            return DbExecResult::Error(e);
        }
        match self.adapter {
            LasmDbRecordsAdapter::Postgres => {
                let pg_params = json_values_to_postgres_params(params);
                let config = match self.resource_postgres_config() {
                    Ok(c) => c,
                    Err(e) => return DbExecResult::Error(e),
                };
                match run_lasm_postgres_query_one_thread_local(&config, sql, &pg_params) {
                    Ok(Some(row)) => DbExecResult::Row(row),
                    Ok(None) => DbExecResult::Error("NOT_FOUND".to_string()),
                    Err(e) => DbExecResult::Error(format!("resource db query error: {e}")),
                }
            }
            LasmDbRecordsAdapter::Sqlite => {
                let sqlite_sql = postgres_placeholders_to_sqlite(sql);
                let sqlite_params = json_values_to_sqlite_params(params);
                let mut state = match self.dynamic_state.lock() {
                    Ok(s) => s,
                    Err(_) => {
                        return DbExecResult::Error(
                            "resource db: state lock unavailable".to_string(),
                        )
                    }
                };
                let conn = match resource_sqlite_connection_mut(&mut state) {
                    Ok(c) => c,
                    Err(e) => return DbExecResult::Error(e),
                };
                match resource_sqlite_query_one(conn, &sqlite_sql, &sqlite_params) {
                    Ok(Some(row)) => DbExecResult::Row(row),
                    Ok(None) => DbExecResult::Error("NOT_FOUND".to_string()),
                    Err(e) => DbExecResult::Error(format!("resource db query error: {e}")),
                }
            }
            _ => DbExecResult::DryRun,
        }
    }

    /// Execute a SELECT query and return all matching rows as a JSON array.
    fn query_many(&self, sql: &str, params: &[Value]) -> DbExecResult {
        if !self.is_live() {
            return DbExecResult::DryRun;
        }
        if let Err(e) = self.ensure_table() {
            return DbExecResult::Error(e);
        }
        match self.adapter {
            LasmDbRecordsAdapter::Postgres => {
                let pg_params = json_values_to_postgres_params(params);
                let config = match self.resource_postgres_config() {
                    Ok(c) => c,
                    Err(e) => return DbExecResult::Error(e),
                };
                match run_lasm_postgres_query_many_thread_local(&config, sql, &pg_params) {
                    Ok(rows) => DbExecResult::Row(Value::Array(rows)),
                    Err(e) => DbExecResult::Error(format!("resource db list error: {e}")),
                }
            }
            LasmDbRecordsAdapter::Sqlite => {
                let sqlite_sql = postgres_placeholders_to_sqlite(sql);
                let sqlite_params = json_values_to_sqlite_params(params);
                let mut state = match self.dynamic_state.lock() {
                    Ok(s) => s,
                    Err(_) => {
                        return DbExecResult::Error(
                            "resource db: state lock unavailable".to_string(),
                        )
                    }
                };
                let conn = match resource_sqlite_connection_mut(&mut state) {
                    Ok(c) => c,
                    Err(e) => return DbExecResult::Error(e),
                };
                match resource_sqlite_query_many(conn, &sqlite_sql, &sqlite_params) {
                    Ok(rows) => DbExecResult::Row(Value::Array(rows)),
                    Err(e) => DbExecResult::Error(format!("resource db list error: {e}")),
                }
            }
            _ => DbExecResult::DryRun,
        }
    }

    /// Build a Postgres thread-local config by briefly locking the dynamic state.
    fn resource_postgres_config(
        &self,
    ) -> Result<crate::lasm_db_runtime_postgres::LasmPostgresThreadLocalConfig, String> {
        let mut state = self
            .dynamic_state
            .lock()
            .map_err(|_| "resource db: state lock unavailable".to_string())?;
        build_lasm_postgres_thread_local_config(&mut state)
            .map_err(|err| format!("resource db: postgres config unavailable: {err}"))
    }
}

// ---------------------------------------------------------------------------
// Postgres helpers for resource dispatch
// ---------------------------------------------------------------------------

/// Convert a slice of `serde_json::Value` to Postgres typed params.
fn json_values_to_postgres_params(values: &[Value]) -> Vec<LasmPostgresParam> {
    values
        .iter()
        .map(|v| match v {
            Value::Null => LasmPostgresParam::Null(None),
            Value::Bool(b) => LasmPostgresParam::Bool(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    LasmPostgresParam::Int(i)
                } else if let Some(f) = n.as_f64() {
                    LasmPostgresParam::Float(f)
                } else {
                    LasmPostgresParam::Text(n.to_string())
                }
            }
            Value::String(s) => LasmPostgresParam::Text(s.clone()),
            other => LasmPostgresParam::Text(serde_json::to_string(other).unwrap_or_default()),
        })
        .collect()
}

/// Generate `CREATE TABLE IF NOT EXISTS` DDL with Postgres-native types.
fn generate_create_table_ddl_postgres(plan: &LasmResourcePlan) -> String {
    let mut cols = Vec::new();
    for field in &plan.fields {
        // Use TEXT for string-like types (including UUID and Time) to avoid
        // prepared-statement parameter type mismatches. The postgres crate's
        // prepared statements infer param types from columns, and TEXT params
        // cannot be bound to UUID/TIMESTAMPTZ columns at the protocol level.
        // Numeric and boolean types work because LasmPostgresParam::Int/Bool
        // send the correct protocol-level type OIDs.
        let col_type = match field.field_type.as_str() {
            "Int64" => "BIGINT",
            "Int" => "INTEGER",
            "Bool" => "BOOLEAN",
            _ => "TEXT",
        };
        let pk = if field.primary { " PRIMARY KEY" } else { "" };
        let not_null_clause = if !field.primary && !field.optional {
            " NOT NULL"
        } else {
            ""
        };
        let unique_clause = if !field.primary && field.unique {
            " UNIQUE"
        } else {
            ""
        };
        let default_clause: String = if !field.primary {
            if let Some(val) = field.default_value.as_deref() {
                format!(" DEFAULT '{}'", val.replace('\'', "''"))
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        cols.push(format!(
            "{} {col_type}{pk}{not_null_clause}{unique_clause}{default_clause}",
            quote_ident(&field.name)
        ));
    }
    format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        quote_ident(&plan.table),
        cols.join(", "),
    )
}

// ---------------------------------------------------------------------------
// SQLite helpers for resource dispatch
// ---------------------------------------------------------------------------

/// Get a mutable reference to the SQLite connection from dynamic state.
fn resource_sqlite_connection_mut(
    state: &mut LasmDynamicResponseState,
) -> Result<&mut rusqlite::Connection, String> {
    if state.db_records_sqlite_connection.is_none() {
        let path = state
            .db_records_sqlite_store_path
            .as_ref()
            .ok_or_else(|| "resource db: sqlite store path unavailable".to_string())?;
        let connection = crate::lasm_db_adapter_state::connect_lasm_dynamic_db_records_sqlite(
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
        .ok_or_else(|| "resource db: sqlite connection unavailable".to_string())
}

/// Convert Postgres-style `$1`, `$2`, ... placeholders to SQLite `?1`, `?2`, ...
fn postgres_placeholders_to_sqlite(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let bytes = sql.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > start {
                result.push('?');
                result.push_str(&sql[start..end]);
                i = end;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// Convert a slice of `serde_json::Value` to positional SQLite params.
fn json_values_to_sqlite_params(values: &[Value]) -> Vec<SqliteValue> {
    values
        .iter()
        .map(|v| match v {
            Value::Null => SqliteValue::Null,
            Value::Bool(b) => SqliteValue::Integer(if *b { 1 } else { 0 }),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    SqliteValue::Integer(i)
                } else if let Some(u) = n.as_u64() {
                    if u <= i64::MAX as u64 {
                        SqliteValue::Integer(u as i64)
                    } else {
                        SqliteValue::Text(n.to_string())
                    }
                } else if let Some(f) = n.as_f64() {
                    SqliteValue::Real(f)
                } else {
                    SqliteValue::Text(n.to_string())
                }
            }
            Value::String(s) => SqliteValue::Text(s.clone()),
            other => SqliteValue::Text(serde_json::to_string(other).unwrap_or_default()),
        })
        .collect()
}

/// Execute a SQL statement against the SQLite connection and return the first row as JSON.
fn resource_sqlite_query_one(
    conn: &mut rusqlite::Connection,
    sql: &str,
    params: &[SqliteValue],
) -> Result<Option<Value>, String> {
    let mut stmt = conn
        .prepare_cached(sql)
        .map_err(|e| format!("sqlite prepare failed: {e}"))?;
    let mut rows = stmt
        .query(rusqlite::params_from_iter(params.iter()))
        .map_err(|e| format!("sqlite query failed: {e}"))?;
    let Some(row) = rows
        .next()
        .map_err(|e| format!("sqlite row fetch failed: {e}"))?
    else {
        return Ok(None);
    };
    let row_ref = row.as_ref();
    let mut object = serde_json::Map::new();
    for index in 0..row_ref.column_count() {
        let name = row_ref.column_name(index).unwrap_or("").to_string();
        let value = row
            .get_ref(index)
            .map(resource_sqlite_value_to_json)
            .map_err(|e| format!("sqlite column decode failed: {e}"))?;
        object.insert(name, value);
    }
    Ok(Some(Value::Object(object)))
}

/// Execute a SQL SELECT against the SQLite connection and return all matching rows.
fn resource_sqlite_query_many(
    conn: &mut rusqlite::Connection,
    sql: &str,
    params: &[SqliteValue],
) -> Result<Vec<Value>, String> {
    let mut stmt = conn
        .prepare_cached(sql)
        .map_err(|e| format!("sqlite prepare failed: {e}"))?;
    let mut rows = stmt
        .query(rusqlite::params_from_iter(params.iter()))
        .map_err(|e| format!("sqlite query failed: {e}"))?;
    let mut results = Vec::new();
    loop {
        let row = match rows.next() {
            Ok(Some(row)) => row,
            Ok(None) => break,
            Err(e) => return Err(format!("sqlite row iteration failed: {e}")),
        };
        let row_ref = row.as_ref();
        let mut object = serde_json::Map::new();
        for index in 0..row_ref.column_count() {
            let name = row_ref.column_name(index).unwrap_or("").to_string();
            let value = row
                .get_ref(index)
                .map(resource_sqlite_value_to_json)
                .map_err(|e| format!("sqlite column decode failed: {e}"))?;
            object.insert(name, value);
        }
        results.push(Value::Object(object));
    }
    Ok(results)
}

/// Convert a SQLite ValueRef to serde_json::Value.
fn resource_sqlite_value_to_json(value: rusqlite::types::ValueRef<'_>) -> Value {
    match value {
        rusqlite::types::ValueRef::Null => Value::Null,
        rusqlite::types::ValueRef::Integer(i) => Value::Number(serde_json::Number::from(i)),
        rusqlite::types::ValueRef::Real(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        rusqlite::types::ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).to_string()),
        rusqlite::types::ValueRef::Blob(b) => Value::String(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            b,
        )),
    }
}

/// Generate `CREATE TABLE IF NOT EXISTS` DDL for the resource table.
fn generate_create_table_ddl(plan: &LasmResourcePlan) -> String {
    let mut cols = Vec::new();
    for field in &plan.fields {
        let col_type = match field.field_type.as_str() {
            "Int64" | "Int" => "INTEGER",
            "Bool" => "INTEGER",
            _ => "TEXT",
        };
        let pk = if field.primary { " PRIMARY KEY" } else { "" };
        let not_null_clause = if !field.primary && !field.optional {
            " NOT NULL"
        } else {
            ""
        };
        let unique_clause = if !field.primary && field.unique {
            " UNIQUE"
        } else {
            ""
        };
        let default_clause = if field.primary {
            String::new()
        } else if let Some(value) = field.default_value.as_deref() {
            match field.field_type.as_str() {
                "Int64" | "Int" => format!(" DEFAULT {value}"),
                "Bool" => {
                    let sqlite_bool = if value.eq_ignore_ascii_case("true") {
                        1
                    } else {
                        0
                    };
                    format!(" DEFAULT {sqlite_bool}")
                }
                _ => format!(" DEFAULT '{}'", value.replace('\'', "''")),
            }
        } else if field.auto_fill && field.field_type == "Time" {
            " DEFAULT CURRENT_TIMESTAMP".to_string()
        } else {
            String::new()
        };
        cols.push(format!(
            "{} {col_type}{pk}{not_null_clause}{unique_clause}{default_clause}",
            quote_ident(&field.name)
        ));
    }
    format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        quote_ident(&plan.table),
        cols.join(", "),
    )
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
                "message": "no updatable fields provided; primary key cannot be updated — provide at least one non-primary, non-auto field",
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
    if value.is_null() {
        if field.optional {
            return Ok(());
        }
        return Err("null values are not allowed".to_string());
    }
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
            let s = value.as_str().ok_or("expected string for Email field")?;
            let parts: Vec<&str> = s.split('@').collect();
            if parts.len() != 2
                || parts[0].is_empty()
                || parts[1].is_empty()
                || !parts[1].contains('.')
            {
                return Err(format!("invalid email: {}", s));
            }
            Ok(())
        }
        "String" => {
            let s = value.as_str().ok_or("expected string")?;
            if s.trim().is_empty() {
                return Err("string must not be empty or whitespace-only".to_string());
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
                .ok_or("expected ISO 8601 string for Time field")?;
            if s.len() < 10 || (!s.contains('T') && !s.contains(' ')) {
                return Err(format!("invalid time format (expected ISO 8601): {}", s));
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
            "String" => {
                if id.trim().is_empty() {
                    return Err("path parameter must not be empty".to_string());
                }
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
    if value.len() != UUID_STRING_LEN {
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
    use std::fs::File;
    use std::io::Read;
    use std::sync::atomic::AtomicU64;

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let mut bytes = [0u8; 16];
    if let Ok(mut f) = File::open("/dev/urandom") {
        if f.read_exact(&mut bytes).is_err() {
            fill_fallback_bytes(&mut bytes, &COUNTER);
        }
    } else {
        fill_fallback_bytes(&mut bytes, &COUNTER);
    }

    // Set version 4 and variant bits
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    )
}

fn fill_fallback_bytes(bytes: &mut [u8; 16], counter: &std::sync::atomic::AtomicU64) {
    use std::sync::atomic::Ordering;
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let count = counter.fetch_add(1, Ordering::Relaxed);
    let combined = nanos as u64 ^ (count.wrapping_mul(6364136223846793005));
    bytes[..8].copy_from_slice(&combined.to_le_bytes());
    bytes[8..16].copy_from_slice(&(combined.wrapping_add(count)).to_le_bytes());
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
        .expect("resource must have a @primary field (compiler should have rejected this)")
}

fn iso8601_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days = (secs / 86400) as i64;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm)
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

fn auto_fill_value(field_type: &str) -> Value {
    match field_type {
        "Uuid" => {
            let uuid = generate_uuid();
            debug_assert!(
                uuid.len() == UUID_STRING_LEN,
                "generated UUID has wrong length"
            );
            Value::String(uuid)
        }
        "Time" => Value::String(iso8601_now()),
        "Int64" | "Int" => Value::Number(serde_json::Number::from(epoch_ms() as i64)),
        _ => {
            let uuid = generate_uuid();
            debug_assert!(
                uuid.len() == UUID_STRING_LEN,
                "generated UUID has wrong length"
            );
            Value::String(uuid)
        }
    }
}
