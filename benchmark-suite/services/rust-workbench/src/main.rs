use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

static TRACE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
struct HttpError {
    code: &'static str,
    kind: &'static str,
    status: u16,
    message: String,
}

impl HttpError {
    fn new(
        code: &'static str,
        kind: &'static str,
        status: u16,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            kind,
            status,
            message: message.into(),
        }
    }
}

struct ApiServerConfig {
    pg_dsn: String,
    auth_token: String,
}

struct TaskInput {
    id: String,
    title: String,
    description: String,
    status: String,
    priority: i64,
    created_at_ms: i64,
}

struct TaskWithCommentInput {
    task_id: String,
    title: String,
    description: String,
    status: String,
    priority: i64,
    created_at_ms: i64,
    comment_id: String,
    comment_task_id: String,
    comment_body: String,
    comment_created_at_ms: i64,
}

struct CommentInput {
    id: String,
    body: String,
    created_at_ms: i64,
}

struct ListInput {
    status: String,
    limit: i64,
    offset: i64,
}

enum RouteBody {
    Json(Value),
    Text(String),
}

struct RouteResponse {
    status: u16,
    body: RouteBody,
}

impl RouteResponse {
    fn json(status: u16, body: Value) -> Self {
        Self {
            status,
            body: RouteBody::Json(body),
        }
    }

    fn text(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            body: RouteBody::Text(body.into()),
        }
    }
}

fn main() {
    let port = env::var("PORT").unwrap_or_else(|_| "18091".to_string());
    let pg_dsn = env::var("BENCH_WORKBENCH_PG_DSN")
        .or_else(|_| env::var("SEC4_RT_LASM_DB_POSTGRES_DSN"))
        .unwrap_or_else(|_| "postgresql://127.0.0.1:5432/postgres?sslmode=disable".to_string());
    let auth_token =
        env::var("BENCH_WORKBENCH_AUTH_TOKEN").unwrap_or_else(|_| "token123".to_string());

    let config = ApiServerConfig { pg_dsn, auth_token };

    let bind_addr = format!("0.0.0.0:{port}");
    let server = Server::http(&bind_addr).expect("failed to bind rust-workbench server");
    eprintln!("rust-workbench service listening on :{port}");

    for request in server.incoming_requests() {
        handle_request(request, &config);
    }
}

fn handle_request(request: Request, config: &ApiServerConfig) {
    let trace_id = format!("rt-{}", TRACE_SEQUENCE.fetch_add(1, Ordering::Relaxed));
    let method = request.method().clone();
    let raw_url = request.url().to_string();
    let (path, query) = split_path_and_query(&raw_url);

    let response = dispatch_route(config, &request, &method, &path, &query, &trace_id);

    match response {
        Ok(route) => match route.body {
            RouteBody::Json(payload) => respond_json(request, route.status, &trace_id, &payload),
            RouteBody::Text(payload) => respond_text(request, route.status, &trace_id, &payload),
        },
        Err(err) => {
            let status = err.status;
            let payload = error_envelope(&err, &trace_id);
            respond_json(request, status, &trace_id, &payload);
        }
    }
}

fn dispatch_route(
    config: &ApiServerConfig,
    request: &Request,
    method: &Method,
    path: &str,
    query: &HashMap<String, String>,
    trace_id: &str,
) -> Result<RouteResponse, HttpError> {
    if *method == Method::Get && path == "/health" {
        return Ok(RouteResponse::text(200, "ok"));
    }

    if *method == Method::Post && path == "/wb/setup" {
        require_auth(request, config)?;
        setup_schema(config)?;
        return Ok(RouteResponse::json(
            200,
            success_envelope(200, trace_id, json!({ "setup": true })),
        ));
    }

    if *method == Method::Post && path == "/wb/tasks" {
        require_auth(request, config)?;
        let data = insert_task(config, query)?;
        return Ok(RouteResponse::json(
            201,
            success_envelope(201, trace_id, data),
        ));
    }

    if *method == Method::Post && path == "/wb/tasks/with-comment" {
        require_auth(request, config)?;
        let data = create_task_with_comment(config, query)?;
        return Ok(RouteResponse::json(
            201,
            success_envelope(201, trace_id, data),
        ));
    }

    if *method == Method::Post && path.starts_with("/wb/tasks/") && path.ends_with("/comments") {
        require_auth(request, config)?;
        let task_id = path
            .trim_start_matches("/wb/tasks/")
            .trim_end_matches("/comments");
        if task_id.is_empty() || task_id.contains('/') {
            return Err(HttpError::new(
                "HTTP.NOT_FOUND",
                "missing_dependency",
                404,
                "route not found",
            ));
        }
        let data = insert_comment(config, task_id, query)?;
        return Ok(RouteResponse::json(
            201,
            success_envelope(201, trace_id, data),
        ));
    }

    if *method == Method::Get && path == "/wb/tasks" {
        let data = list_tasks(config, query)?;
        return Ok(RouteResponse::json(
            200,
            success_envelope(200, trace_id, data),
        ));
    }

    if *method == Method::Get && path.starts_with("/wb/tasks/") {
        let task_id = path.trim_start_matches("/wb/tasks/");
        if task_id.is_empty() || task_id.contains('/') {
            return Err(HttpError::new(
                "HTTP.NOT_FOUND",
                "missing_dependency",
                404,
                "route not found",
            ));
        }
        let data = get_task(config, task_id)?;
        return Ok(RouteResponse::json(
            200,
            success_envelope(200, trace_id, data),
        ));
    }

    Err(HttpError::new(
        "HTTP.NOT_FOUND",
        "missing_dependency",
        404,
        "route not found",
    ))
}

fn require_auth(request: &Request, config: &ApiServerConfig) -> Result<(), HttpError> {
    let actual = request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Authorization"))
        .map(|header| header.value.as_str())
        .unwrap_or_default();
    let expected = format!("Bearer {}", config.auth_token);
    if actual != expected {
        return Err(HttpError::new(
            "AUTH.REQUIRED",
            "auth",
            401,
            "authorization token is required",
        ));
    }
    Ok(())
}

fn setup_schema(config: &ApiServerConfig) -> Result<(), HttpError> {
    run_psql(
        config,
        "create table if not exists wb_tasks (id text primary key, title text not null, description text not null default '', status text not null, priority integer not null, created_at_ms bigint not null);",
    )?;
    run_psql(
        config,
        "create table if not exists wb_comments (id text primary key, task_id text not null, body text not null, created_at_ms bigint not null);",
    )?;
    run_psql(
        config,
        "create table if not exists wb_labels (task_id text not null, name text not null, primary key(task_id, name));",
    )?;
    Ok(())
}

fn insert_task(
    config: &ApiServerConfig,
    query: &HashMap<String, String>,
) -> Result<Value, HttpError> {
    let input = parse_task_input(query)?;

    validate_status(&input.status)?;
    validate_priority(input.priority)?;

    let sql = format!(
        "insert into wb_tasks (id, title, description, status, priority, created_at_ms) values ({}, {}, {}, {}, {}, {});",
        sql_literal(&input.id),
        sql_literal(&input.title),
        sql_literal(&input.description),
        sql_literal(&input.status),
        input.priority,
        input.created_at_ms
    );
    run_psql(config, &sql)?;
    Ok(json!({ "id": input.id }))
}

fn create_task_with_comment(
    config: &ApiServerConfig,
    query: &HashMap<String, String>,
) -> Result<Value, HttpError> {
    let input = parse_task_with_comment_input(query)?;

    validate_status(&input.status)?;
    validate_priority(input.priority)?;
    if !input.comment_task_id.is_empty() && input.comment_task_id != input.task_id {
        return Err(HttpError::new(
            "VALIDATION.INVALID",
            "validation",
            400,
            "comment task id must match task id",
        ));
    }

    run_psql_tx(
        config,
        &[
            format!(
                "insert into wb_tasks (id, title, description, status, priority, created_at_ms) values ({}, {}, {}, {}, {}, {});",
                sql_literal(&input.task_id),
                sql_literal(&input.title),
                sql_literal(&input.description),
                sql_literal(&input.status),
                input.priority,
                input.created_at_ms
            ),
            format!(
                "insert into wb_comments (id, task_id, body, created_at_ms) values ({}, {}, {}, {});",
                sql_literal(&input.comment_id),
                sql_literal(&input.task_id),
                sql_literal(&input.comment_body),
                input.comment_created_at_ms
            ),
        ],
    )?;

    Ok(json!({
        "taskId": input.task_id,
        "commentId": input.comment_id
    }))
}

fn insert_comment(
    config: &ApiServerConfig,
    task_id: &str,
    query: &HashMap<String, String>,
) -> Result<Value, HttpError> {
    let input = parse_comment_input(query, task_id)?;
    let sql = format!(
        "insert into wb_comments (id, task_id, body, created_at_ms) values ({}, {}, {}, {});",
        sql_literal(&input.id),
        sql_literal(task_id),
        sql_literal(&input.body),
        input.created_at_ms
    );
    run_psql(config, &sql)?;
    Ok(json!({ "id": input.id }))
}

fn get_task(config: &ApiServerConfig, task_id: &str) -> Result<Value, HttpError> {
    let row_json = run_psql(
        config,
        &format!(
            "select row_to_json(t)::text from (select id, title, description, status, priority, created_at_ms, (select count(*)::int from wb_comments c where c.task_id = wb_tasks.id) as comments_count from wb_tasks where id = {} limit 1) t;",
            sql_literal(task_id)
        ),
    )?;
    if row_json.is_empty() {
        return Err(HttpError::new(
            "TASK.NOT_FOUND",
            "missing_dependency",
            404,
            "task not found",
        ));
    }
    serde_json::from_str::<Value>(&row_json).map_err(|_| {
        HttpError::new(
            "DB.QUERY_FAILED",
            "internal",
            500,
            "postgres returned invalid json payload",
        )
    })
}

fn list_tasks(
    config: &ApiServerConfig,
    query: &HashMap<String, String>,
) -> Result<Value, HttpError> {
    let input = parse_list_input(query)?;

    let bounded_limit = input.limit.clamp(1, 100);
    let bounded_offset = input.offset.max(0);

    let where_sql = if input.status.is_empty() {
        "true".to_string()
    } else {
        format!("status = {}", sql_literal(&input.status))
    };

    let items_json = run_psql(
        config,
        &format!(
            "select coalesce(json_agg(t), '[]'::json)::text from (select id, title, description, status, priority, created_at_ms from wb_tasks where {} order by created_at_ms desc, id desc limit {} offset {}) t;",
            where_sql, bounded_limit, bounded_offset
        ),
    )?;
    let count_raw = run_psql(
        config,
        &format!("select count(*)::text from wb_tasks where {};", where_sql),
    )?;

    let items = if items_json.is_empty() {
        json!([])
    } else {
        serde_json::from_str::<Value>(&items_json).map_err(|_| {
            HttpError::new(
                "DB.QUERY_FAILED",
                "internal",
                500,
                "postgres returned invalid list payload",
            )
        })?
    };
    let count = count_raw.parse::<i64>().unwrap_or(0);

    Ok(json!({
        "items": items,
        "count": count,
        "limit": bounded_limit,
        "offset": bounded_offset
    }))
}

fn validate_status(value: &str) -> Result<(), HttpError> {
    if value == "open" || value == "in_progress" || value == "done" {
        return Ok(());
    }
    Err(HttpError::new(
        "VALIDATION.INVALID",
        "validation",
        400,
        "status must be one of: open, in_progress, done",
    ))
}

fn validate_priority(value: i64) -> Result<(), HttpError> {
    if (1..=5).contains(&value) {
        return Ok(());
    }
    Err(HttpError::new(
        "VALIDATION.INVALID",
        "validation",
        400,
        "priority must be in range 1..5",
    ))
}

fn require_text_param(
    query: &HashMap<String, String>,
    key: &'static str,
) -> Result<String, HttpError> {
    match query.get(key) {
        Some(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err(HttpError::new(
            "VALIDATION.INVALID",
            "validation",
            400,
            format!("missing required query param: {key}"),
        )),
    }
}

fn optional_text_param(
    query: &HashMap<String, String>,
    key: &'static str,
    fallback: &str,
) -> String {
    query
        .get(key)
        .filter(|value| !value.is_empty())
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}

fn require_int_param(query: &HashMap<String, String>, key: &'static str) -> Result<i64, HttpError> {
    let raw = require_text_param(query, key)?;
    raw.parse::<i64>().map_err(|_| {
        HttpError::new(
            "VALIDATION.INVALID",
            "validation",
            400,
            format!("invalid integer query param: {key}"),
        )
    })
}

fn optional_int_param(
    query: &HashMap<String, String>,
    key: &'static str,
    fallback: i64,
) -> Result<i64, HttpError> {
    match query.get(key) {
        Some(raw) if !raw.is_empty() => raw.parse::<i64>().map_err(|_| {
            HttpError::new(
                "VALIDATION.INVALID",
                "validation",
                400,
                format!("invalid integer query param: {key}"),
            )
        }),
        _ => Ok(fallback),
    }
}

fn parse_json_array_param(
    query: &HashMap<String, String>,
    key: &'static str,
    min_len: usize,
) -> Result<Option<Vec<Value>>, HttpError> {
    let Some(raw) = query.get(key) else {
        return Ok(None);
    };
    if raw.trim().is_empty() {
        return Ok(None);
    }
    let parsed = serde_json::from_str::<Value>(raw).map_err(|_| {
        HttpError::new(
            "VALIDATION.INVALID",
            "validation",
            400,
            format!("invalid JSON array query param: {key}"),
        )
    })?;
    let Some(values) = parsed.as_array() else {
        return Err(HttpError::new(
            "VALIDATION.INVALID",
            "validation",
            400,
            format!("invalid JSON array query param: {key}"),
        ));
    };
    if values.len() < min_len {
        return Err(HttpError::new(
            "VALIDATION.INVALID",
            "validation",
            400,
            format!("invalid JSON array query param: {key}"),
        ));
    }
    Ok(Some(values.clone()))
}

fn parse_i64_value(value: &Value, key: &'static str) -> Result<i64, HttpError> {
    if let Some(number) = value.as_i64() {
        return Ok(number);
    }
    if let Some(text) = value.as_str() {
        return text.parse::<i64>().map_err(|_| {
            HttpError::new(
                "VALIDATION.INVALID",
                "validation",
                400,
                format!("invalid integer query param: {key}"),
            )
        });
    }
    Err(HttpError::new(
        "VALIDATION.INVALID",
        "validation",
        400,
        format!("invalid integer query param: {key}"),
    ))
}

fn parse_string_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        _ => value.to_string().trim_matches('"').to_string(),
    }
}

fn parse_task_input(query: &HashMap<String, String>) -> Result<TaskInput, HttpError> {
    if let Some(params) = parse_json_array_param(query, "params", 6)? {
        return Ok(TaskInput {
            id: parse_string_value(&params[0]),
            title: parse_string_value(&params[1]),
            description: parse_string_value(&params[2]),
            status: parse_string_value(&params[3]),
            priority: parse_i64_value(&params[4], "priority")?,
            created_at_ms: parse_i64_value(&params[5], "created_at_ms")?,
        });
    }

    Ok(TaskInput {
        id: require_text_param(query, "id")?,
        title: require_text_param(query, "title")?,
        description: optional_text_param(query, "description", ""),
        status: require_text_param(query, "status")?,
        priority: require_int_param(query, "priority")?,
        created_at_ms: require_int_param(query, "created_at_ms")?,
    })
}

fn parse_task_with_comment_input(
    query: &HashMap<String, String>,
) -> Result<TaskWithCommentInput, HttpError> {
    if let (Some(task_params), Some(comment_params)) = (
        parse_json_array_param(query, "task_params", 6)?,
        parse_json_array_param(query, "comment_params", 4)?,
    ) {
        return Ok(TaskWithCommentInput {
            task_id: parse_string_value(&task_params[0]),
            title: parse_string_value(&task_params[1]),
            description: parse_string_value(&task_params[2]),
            status: parse_string_value(&task_params[3]),
            priority: parse_i64_value(&task_params[4], "priority")?,
            created_at_ms: parse_i64_value(&task_params[5], "created_at_ms")?,
            comment_id: parse_string_value(&comment_params[0]),
            comment_task_id: parse_string_value(&comment_params[1]),
            comment_body: parse_string_value(&comment_params[2]),
            comment_created_at_ms: parse_i64_value(&comment_params[3], "comment_created_at_ms")?,
        });
    }

    Ok(TaskWithCommentInput {
        task_id: require_text_param(query, "id")?,
        title: require_text_param(query, "title")?,
        description: optional_text_param(query, "description", ""),
        status: require_text_param(query, "status")?,
        priority: require_int_param(query, "priority")?,
        created_at_ms: require_int_param(query, "created_at_ms")?,
        comment_id: require_text_param(query, "comment_id")?,
        comment_task_id: String::new(),
        comment_body: require_text_param(query, "comment_body")?,
        comment_created_at_ms: require_int_param(query, "comment_created_at_ms")?,
    })
}

fn parse_comment_input(
    query: &HashMap<String, String>,
    route_task_id: &str,
) -> Result<CommentInput, HttpError> {
    if let Some(params) = parse_json_array_param(query, "params", 4)? {
        let params_task_id = parse_string_value(&params[1]);
        if params_task_id != route_task_id {
            return Err(HttpError::new(
                "VALIDATION.INVALID",
                "validation",
                400,
                "comment task id must match route task id",
            ));
        }
        return Ok(CommentInput {
            id: parse_string_value(&params[0]),
            body: parse_string_value(&params[2]),
            created_at_ms: parse_i64_value(&params[3], "comment_created_at_ms")?,
        });
    }

    Ok(CommentInput {
        id: require_text_param(query, "comment_id")?,
        body: require_text_param(query, "comment_body")?,
        created_at_ms: require_int_param(query, "comment_created_at_ms")?,
    })
}

fn parse_list_input(query: &HashMap<String, String>) -> Result<ListInput, HttpError> {
    if let Some(params) = parse_json_array_param(query, "params", 3)? {
        return Ok(ListInput {
            status: parse_string_value(&params[0]),
            limit: parse_i64_value(&params[1], "limit")?,
            offset: parse_i64_value(&params[2], "offset")?,
        });
    }

    Ok(ListInput {
        status: optional_text_param(query, "status", ""),
        limit: optional_int_param(query, "limit", 20)?,
        offset: optional_int_param(query, "offset", 0)?,
    })
}

fn split_path_and_query(raw_url: &str) -> (String, HashMap<String, String>) {
    let (path, query_raw) = match raw_url.split_once('?') {
        Some((path, query)) => (path.to_string(), query),
        None => (raw_url.to_string(), ""),
    };

    let mut query = HashMap::new();
    for pair in query_raw.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = match pair.split_once('=') {
            Some((key, value)) => (key, value),
            None => (pair, ""),
        };
        query.insert(url_decode(raw_key), url_decode(raw_value));
    }
    (path, query)
}

fn url_decode(input: &str) -> String {
    let mut output = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte == b'+' {
            output.push(b' ');
            i += 1;
            continue;
        }
        if byte == b'%' && i + 2 < bytes.len() {
            if let (Some(a), Some(b)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                output.push((a << 4) | b);
                i += 3;
                continue;
            }
        }
        output.push(byte);
        i += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(10 + (byte - b'a')),
        b'A'..=b'F' => Some(10 + (byte - b'A')),
        _ => None,
    }
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn run_psql(config: &ApiServerConfig, sql: &str) -> Result<String, HttpError> {
    let output = Command::new("psql")
        .arg(&config.pg_dsn)
        .args(["-t", "-A", "-v", "ON_ERROR_STOP=1", "-c", sql])
        .output()
        .map_err(|err| {
            HttpError::new(
                "DB.QUERY_FAILED",
                "internal",
                500,
                format!("failed to execute psql: {err}"),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            "postgres command failed".to_string()
        } else {
            stderr
        };
        return Err(HttpError::new("DB.QUERY_FAILED", "internal", 500, message));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_psql_tx(config: &ApiServerConfig, statements: &[String]) -> Result<(), HttpError> {
    let mut sql = String::from("BEGIN;");
    for statement in statements {
        sql.push_str(statement);
        if !statement.trim_end().ends_with(';') {
            sql.push(';');
        }
    }
    sql.push_str("COMMIT;");
    run_psql(config, &sql)?;
    Ok(())
}

fn success_envelope(status: u16, trace_id: &str, data: Value) -> Value {
    json!({
        "ok": true,
        "status": status,
        "traceId": trace_id,
        "timeMs": now_ms(),
        "data": data
    })
}

fn error_envelope(err: &HttpError, trace_id: &str) -> Value {
    json!({
        "ok": false,
        "status": err.status,
        "traceId": trace_id,
        "timeMs": now_ms(),
        "error": {
            "code": err.code,
            "kind": err.kind,
            "message": err.message
        }
    })
}

fn now_ms() -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_millis() as i64
}

fn respond_json(request: Request, status: u16, trace_id: &str, body: &Value) {
    let response = Response::from_string(body.to_string())
        .with_status_code(StatusCode(status))
        .with_header(make_header(
            "content-type",
            "application/json; charset=utf-8",
        ))
        .with_header(make_header("x-trace-id", trace_id));
    let _ = request.respond(response);
}

fn respond_text(request: Request, status: u16, trace_id: &str, body: &str) {
    let response = Response::from_string(body.to_string())
        .with_status_code(StatusCode(status))
        .with_header(make_header("content-type", "text/plain; charset=utf-8"))
        .with_header(make_header("x-trace-id", trace_id));
    let _ = request.respond(response);
}

fn make_header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("invalid static response header")
}
