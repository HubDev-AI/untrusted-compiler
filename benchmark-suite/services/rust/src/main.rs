use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct UserPayload {
    id: String,
    email: String,
    age: i64,
    name: String,
    tags: Vec<String>,
    address: Address,
    meta: Meta,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Address {
    zip: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Meta {
    flags: Flags,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Flags {
    a: bool,
    b: bool,
    c: bool,
}

fn main() {
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind = format!("0.0.0.0:{port}");
    let server = Server::http(&bind).expect("failed to bind rust benchmark server");
    let users = Arc::new(Mutex::new(HashMap::<String, UserPayload>::new()));

    eprintln!("rust benchmark service listening on :{port}");

    for request in server.incoming_requests() {
        let users = Arc::clone(&users);
        handle_request(request, users);
    }
}

fn handle_request(mut request: Request, users: Arc<Mutex<HashMap<String, UserPayload>>>) {
    let trace_id = new_trace_id();
    let method = request.method().clone();
    let path = request.url().to_string();

    if method == Method::Get && path == "/ping" {
        respond_text(request, StatusCode(200), &trace_id, "ok");
        return;
    }

    if method == Method::Post && path == "/decode" {
        let raw = match read_body(&mut request) {
            Ok(raw) => raw,
            Err(_) => {
                respond_error(
                    request,
                    StatusCode(400),
                    &trace_id,
                    "HTTP.BAD_REQUEST",
                    "could not read body",
                );
                return;
            }
        };
        let payload = match parse_payload(&raw) {
            Ok(payload) => payload,
            Err((code, message)) => {
                respond_error(request, StatusCode(400), &trace_id, code, &message);
                return;
            }
        };
        respond_json(
            request,
            StatusCode(200),
            &trace_id,
            &json!({"ok": true, "id": payload.id}),
        );
        return;
    }

    if method == Method::Post && path == "/users" {
        let raw = match read_body(&mut request) {
            Ok(raw) => raw,
            Err(_) => {
                respond_error(
                    request,
                    StatusCode(400),
                    &trace_id,
                    "HTTP.BAD_REQUEST",
                    "could not read body",
                );
                return;
            }
        };
        let payload = match parse_payload(&raw) {
            Ok(payload) => payload,
            Err((code, message)) => {
                respond_error(request, StatusCode(400), &trace_id, code, &message);
                return;
            }
        };
        let user_id = payload.id.clone();
        users
            .lock()
            .expect("users mutex poisoned")
            .insert(user_id.clone(), payload);
        respond_json(
            request,
            StatusCode(201),
            &trace_id,
            &json!({"ok": true, "userId": user_id}),
        );
        return;
    }

    if method == Method::Get && path.starts_with("/users/") {
        let id = path.trim_start_matches("/users/");
        if !is_uuid_v4(id) {
            respond_error(
                request,
                StatusCode(400),
                &trace_id,
                "VALIDATION.UUID_INVALID",
                "id must be UUID v4",
            );
            return;
        }

        let user = users.lock().expect("users mutex poisoned").get(id).cloned();
        match user {
            Some(user) => respond_json(request, StatusCode(200), &trace_id, &json!(user)),
            None => respond_error(
                request,
                StatusCode(404),
                &trace_id,
                "HTTP.NOT_FOUND",
                "user not found",
            ),
        }
        return;
    }

    respond_error(
        request,
        StatusCode(404),
        &trace_id,
        "HTTP.NOT_FOUND",
        "route not found",
    );
}

fn read_body(request: &mut Request) -> Result<String, ()> {
    let mut raw = String::new();
    request
        .as_reader()
        .read_to_string(&mut raw)
        .map_err(|_| ())?;
    Ok(raw)
}

fn parse_payload(raw: &str) -> Result<UserPayload, (&'static str, String)> {
    let payload = match serde_json::from_str::<UserPayload>(&raw) {
        Ok(payload) => payload,
        Err(_) => return Err(("JSON.INVALID_SYNTAX", "invalid JSON payload".to_string())),
    };

    if let Some(message) = validate_payload(&payload) {
        return Err(("VALIDATION.INVALID", message));
    }

    Ok(payload)
}

fn validate_payload(payload: &UserPayload) -> Option<String> {
    if !is_uuid_v4(&payload.id) {
        return Some("id must be a UUID v4 string".to_string());
    }
    if !is_email(&payload.email) {
        return Some("email must be a valid email string".to_string());
    }
    if !(0..=150).contains(&payload.age) {
        return Some("age must be an integer between 0 and 150".to_string());
    }
    if payload.tags.len() > 16 {
        return Some("tags must be an array of length <= 16".to_string());
    }
    if payload
        .tags
        .iter()
        .any(|tag| tag.is_empty() || tag.len() > 32)
    {
        return Some("tags must contain strings of length 1..32".to_string());
    }
    if !is_zip(&payload.address.zip) {
        return Some("address.zip must be a digit string of length 4..10".to_string());
    }
    None
}

fn is_uuid_v4(input: &str) -> bool {
    Uuid::parse_str(input)
        .map(|uuid| uuid.get_version_num() == 4)
        .unwrap_or(false)
}

fn is_email(input: &str) -> bool {
    let mut parts = input.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    parts.next().is_none() && !local.is_empty() && domain.contains('.')
}

fn is_zip(input: &str) -> bool {
    (4..=10).contains(&input.len()) && input.chars().all(|c| c.is_ascii_digit())
}

fn respond_text(request: Request, status: StatusCode, trace_id: &str, body: &str) {
    let response = Response::from_string(body.to_string())
        .with_status_code(status)
        .with_header(content_type_header("text/plain; charset=utf-8"))
        .with_header(trace_header(trace_id));
    let _ = request.respond(response);
}

fn respond_json(request: Request, status: StatusCode, trace_id: &str, value: &serde_json::Value) {
    let response = Response::from_string(value.to_string())
        .with_status_code(status)
        .with_header(content_type_header("application/json; charset=utf-8"))
        .with_header(trace_header(trace_id));
    let _ = request.respond(response);
}

fn respond_error(request: Request, status: StatusCode, trace_id: &str, code: &str, message: &str) {
    let kind = if status.0 >= 500 {
        "internal"
    } else if status.0 == 404 {
        "not_found"
    } else {
        "validation"
    };
    let payload = json!({
        "error": {
            "code": code,
            "kind": kind,
            "message": message,
            "status": status.0,
            "traceId": trace_id,
            "timeMs": now_ms()
        }
    });
    respond_json(request, status, trace_id, &payload);
}

fn content_type_header(value: &str) -> Header {
    Header::from_bytes(b"content-type", value.as_bytes()).expect("valid content-type header")
}

fn trace_header(trace_id: &str) -> Header {
    Header::from_bytes(b"x-trace-id", trace_id.as_bytes()).expect("valid trace header")
}

fn new_trace_id() -> String {
    format!("trace-{}", Uuid::new_v4())
}

fn now_ms() -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch");
    now.as_millis() as i64
}
