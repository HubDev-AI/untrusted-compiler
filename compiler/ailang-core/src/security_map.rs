use crate::ast::{Block, Expr, ExprKind, ItemKind, Program, StmtKind};
use crate::diagnostics::Diagnostic;
use crate::policy::Policy;
use crate::Span;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SECURITY_MAP_FILE_NAME: &str = "security_map.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TagKind {
    Source,
    Sink,
    Gate,
    Effect,
    Capability,
    Policy,
    Middleware,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum TagAttr {
    String(String),
    Bool(bool),
    Int(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Tag {
    pub id: String,
    pub kind: TagKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<HashMap<String, TagAttr>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecuritySymbol {
    pub sym: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecurityCall {
    pub loc: SourceLocation,
    pub callee: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecurityMiddleware {
    pub loc: SourceLocation,
    pub tags: Vec<String>,
    pub attrs: HashMap<String, TagAttr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecurityAllow {
    pub loc: SourceLocation,
    pub policy: String,
    pub bypass: Vec<String>,
    pub reason: String,
    pub ticket: String,
    pub expires: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecurityMap {
    pub version: String,
    pub policy_hash: String,
    pub symbols: Vec<SecuritySymbol>,
    pub calls: Vec<SecurityCall>,
    pub middleware: Vec<SecurityMiddleware>,
    pub allows: Vec<SecurityAllow>,
}

const ALLOW_REQUIRED_FIELDS: [&str; 5] = ["policy", "bypass", "reason", "ticket", "expires"];

pub fn build_security_map(program: &Program, policy: &Policy) -> SecurityMap {
    let mut calls = Vec::new();
    let mut middleware = Vec::new();

    for item in &program.items {
        let ItemKind::Function(function) = &item.kind else {
            continue;
        };

        collect_block(&function.body, &mut calls, &mut middleware, policy);
    }

    SecurityMap {
        version: "0.1".to_string(),
        policy_hash: policy.policy_hash(),
        symbols: intrinsic_symbol_registry(),
        calls,
        middleware,
        allows: Vec::new(),
    }
}

pub fn build_security_map_with_allows(
    program: &Program,
    policy: &Policy,
    allows: Vec<SecurityAllow>,
) -> SecurityMap {
    let mut map = build_security_map(program, policy);
    map.allows = allows;
    map
}

pub fn strip_allow_annotations(source: &str) -> String {
    let mut stripped = String::with_capacity(source.len());
    let mut cursor = 0usize;

    while let Some(found) = source[cursor..].find("@allow(") {
        let start = cursor + found;
        stripped.push_str(&source[cursor..start]);

        let open_paren = start + "@allow".len();
        let Some(close_paren) = find_matching_paren(source, open_paren) else {
            stripped.push_str(&source[start..]);
            return stripped;
        };

        for ch in source[start..=close_paren].chars() {
            if ch == '\n' {
                stripped.push('\n');
            } else {
                stripped.push(' ');
            }
        }

        cursor = close_paren + 1;
    }

    stripped.push_str(&source[cursor..]);
    stripped
}

pub fn parse_allow_annotations(
    file_path: &Path,
    source: &str,
) -> Result<Vec<SecurityAllow>, Vec<Diagnostic>> {
    let mut annotations = Vec::new();
    let mut diagnostics = Vec::new();
    let mut cursor = 0usize;

    while let Some(found) = source[cursor..].find("@allow(") {
        let start = cursor + found;
        let open_paren = start + "@allow".len();
        let Some(close_paren) = find_matching_paren(source, open_paren) else {
            diagnostics.push(
                Diagnostic::error(
                    "A7001",
                    "unterminated @allow annotation",
                    span_from_offset(file_path, source, start),
                )
                .with_note("expected `)` to close @allow annotation"),
            );
            break;
        };

        let payload = &source[(open_paren + 1)..close_paren];
        let span = span_from_offset(file_path, source, start);
        match parse_allow_payload(file_path, payload, &span) {
            Ok(annotation) => annotations.push(annotation),
            Err(diags) => diagnostics.extend(diags),
        }

        cursor = close_paren + 1;
    }

    if diagnostics.is_empty() {
        Ok(annotations)
    } else {
        diagnostics.sort_by(|left, right| left.code.cmp(&right.code));
        Err(diagnostics)
    }
}

fn collect_block(
    block: &Block,
    calls: &mut Vec<SecurityCall>,
    middleware: &mut Vec<SecurityMiddleware>,
    policy: &Policy,
) {
    for stmt in &block.statements {
        match &stmt.kind {
            StmtKind::Let { value, .. } => collect_expr(value, calls, middleware, policy),
            StmtKind::Return { value } => {
                if let Some(expr) = value {
                    collect_expr(expr, calls, middleware, policy);
                }
            }
            StmtKind::Expr { expr } => collect_expr(expr, calls, middleware, policy),
        }
    }

    if let Some(tail) = &block.tail {
        collect_expr(tail, calls, middleware, policy);
    }
}

fn collect_expr(
    expr: &Expr,
    calls: &mut Vec<SecurityCall>,
    middleware: &mut Vec<SecurityMiddleware>,
    policy: &Policy,
) {
    match &expr.kind {
        ExprKind::Call { callee, args } => {
            if let Some(name) = callable_name(callee) {
                let mut tags = call_tags_for(name.as_str()).unwrap_or_default();
                tags.extend(dynamic_call_tags(name.as_str(), args));
                dedupe_tags(&mut tags);

                if !tags.is_empty() {
                    calls.push(SecurityCall {
                        loc: span_to_loc(&expr.span),
                        callee: name.clone(),
                        tags: tags.into_iter().map(str::to_string).collect(),
                    });
                }

                if let Some(middleware_entry) = middleware_for(name.as_str(), &expr.span, policy) {
                    middleware.push(middleware_entry);
                }
            }

            collect_expr(callee, calls, middleware, policy);
            for arg in args {
                collect_expr(arg, calls, middleware, policy);
            }
        }
        ExprKind::Unary { expr: inner, .. } => collect_expr(inner, calls, middleware, policy),
        ExprKind::Binary { left, right, .. } => {
            collect_expr(left, calls, middleware, policy);
            collect_expr(right, calls, middleware, policy);
        }
        ExprKind::Member { object, .. } => collect_expr(object, calls, middleware, policy),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_expr(condition, calls, middleware, policy);
            collect_block(then_branch, calls, middleware, policy);
            if let Some(else_expr) = else_branch {
                collect_expr(else_expr, calls, middleware, policy);
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            collect_expr(scrutinee, calls, middleware, policy);
            for arm in arms {
                collect_expr(&arm.value, calls, middleware, policy);
            }
        }
        ExprKind::Block(block) => collect_block(block, calls, middleware, policy),
        ExprKind::Identifier(_) | ExprKind::Number(_) | ExprKind::String(_) | ExprKind::Bool(_) => {
        }
    }
}

fn call_tags_for(name: &str) -> Option<Vec<&'static str>> {
    let tags = match name {
        "req_body" | "req.body" => vec!["source.http.body"],
        "req_query" | "req.query" => vec!["source.http.query"],
        "req_header" | "req.header" => vec!["source.http.header"],
        "req_path_param" | "req.pathParam" => vec!["source.http.path"],
        "req_json" | "req.json" => {
            vec!["source.http.body", "gate.schema.json_decode", "effect.net"]
        }
        "validate_header_value" | "validate.headerValue" => vec!["gate.header.value"],
        "sanitize_html" | "sanitize.html" => vec!["gate.sanitize.html"],
        "path_under" | "path.under" | "validate_path_under" | "validate.pathUnder" => {
            vec!["gate.path.under"]
        }
        "url_public" | "url.public" => vec!["gate.url.public", "effect.net"],
        "url_internal" | "url.internal" => vec!["gate.url.internal", "effect.net"],
        "db_read" | "db.queryOne" => vec!["sink.sql.query", "effect.db.read", "capability.db"],
        "db_write" | "db.exec" => vec!["sink.sql.exec", "effect.db.write", "capability.db"],
        "res_html" | "res.html" => vec!["sink.http.html"],
        "set_header" | "res.setHeader" => vec!["sink.http.header_set"],
        "set_cookie" | "res.addCookie" => vec!["sink.http.cookie_set"],
        "res_json" | "res.json" => vec!["sink.json.encode_http_response"],
        "net_call" | "httpClient.get" => {
            vec!["sink.net.public_request", "effect.net", "capability.net"]
        }
        "net_internal_call" | "httpClient.getInternal" => vec![
            "sink.net.internal_request",
            "effect.net",
            "capability.internal_net",
        ],
        "fs_read" | "fs.read" => vec!["sink.fs.read", "effect.fs.read", "capability.fs"],
        "fs_write" | "fs.write" => vec!["sink.fs.write", "effect.fs.write", "capability.fs"],
        "log" | "log.emit" | "log.info" | "log.warn" | "log.error" => {
            vec!["sink.log.emit", "effect.log"]
        }
        "secret_read" | "secrets.get" => vec!["effect.secrets.read", "capability.secrets"],
        "secret_reveal" | "secrets.reveal" => vec!["effect.secrets.reveal", "capability.secrets"],
        _ => return None,
    };

    Some(tags)
}

fn dynamic_call_tags(name: &str, args: &[Expr]) -> Vec<&'static str> {
    let mut tags = Vec::new();
    if sql_call_has_select_without_limit(name, args) {
        tags.push("sql.select_without_limit");
    }
    tags
}

fn sql_call_has_select_without_limit(name: &str, args: &[Expr]) -> bool {
    if !matches!(name, "db_write" | "db.exec" | "db_read" | "db.queryOne") {
        return false;
    }

    let Some(sql_text) = sql_query_arg_text(args) else {
        return false;
    };

    contains_sql_keyword(sql_text, "select") && !contains_sql_keyword(sql_text, "limit")
}

fn sql_query_arg_text(args: &[Expr]) -> Option<&str> {
    let query_arg = args.get(1)?;
    extract_sql_text(query_arg)
}

fn extract_sql_text(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::String(text) => Some(text.as_str()),
        ExprKind::Call { callee, args } => {
            let name = callable_name(callee)?;
            if !matches!(name.as_str(), "sql" | "sql_q" | "sql.q") {
                return None;
            }
            let first = args.first()?;
            if let ExprKind::String(text) = &first.kind {
                Some(text.as_str())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn contains_sql_keyword(sql: &str, keyword: &str) -> bool {
    let keyword_lower = keyword.to_ascii_lowercase();
    let mut current = String::new();

    for ch in sql.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            if current == keyword_lower {
                return true;
            }
            current.clear();
        }
    }

    if !current.is_empty() && current == keyword_lower {
        return true;
    }

    false
}

fn dedupe_tags(tags: &mut Vec<&'static str>) {
    let mut deduped = Vec::with_capacity(tags.len());
    for tag in tags.iter().copied() {
        if !deduped.contains(&tag) {
            deduped.push(tag);
        }
    }
    *tags = deduped;
}

fn middleware_for(name: &str, span: &Span, policy: &Policy) -> Option<SecurityMiddleware> {
    let mut attrs = HashMap::new();
    let (tag, attrs_fill): (&str, fn(&mut HashMap<String, TagAttr>, &Policy)) = match name {
        "withCors" | "cors_with" | "cors.withCors" => ("middleware.cors", |attrs, policy| {
            attrs.insert(
                "allowCredentials".to_string(),
                TagAttr::Bool(policy.cors.allow_credentials),
            );
            attrs.insert(
                "wildcard".to_string(),
                TagAttr::Bool(policy.cors.has_wildcard_origin()),
            );
            attrs.insert(
                "originsCount".to_string(),
                TagAttr::Int(policy.cors.allowed_origins.len() as i64),
            );
            attrs.insert(
                "requireVaryOrigin".to_string(),
                TagAttr::Bool(policy.cors.require_vary_origin),
            );
        }),
        "withSecurityHeaders" | "sec_with_security_headers" | "sec.withSecurityHeaders" => {
            ("middleware.security_headers", |attrs, policy| {
                attrs.insert(
                    "enabled".to_string(),
                    TagAttr::Bool(policy.security_headers.enabled),
                );
                attrs.insert(
                    "hsts".to_string(),
                    TagAttr::Bool(policy.security_headers.hsts_enabled),
                );
                attrs.insert(
                    "cspEnabled".to_string(),
                    TagAttr::Bool(policy.security_headers.csp_enabled),
                );
                attrs.insert(
                    "reportOnly".to_string(),
                    TagAttr::Bool(policy.security_headers.csp_report_only),
                );
                attrs.insert(
                    "xFrameOptions".to_string(),
                    TagAttr::String(policy.security_headers.x_frame_options.clone()),
                );
            })
        }
        "withCsrf" | "csrf_with" | "csrf.withCsrf" => ("middleware.csrf", |attrs, policy| {
            attrs.insert("enabled".to_string(), TagAttr::Bool(policy.csrf.enabled));
            attrs.insert(
                "mode".to_string(),
                TagAttr::String(policy.csrf.mode.clone()),
            );
            attrs.insert(
                "sameSite".to_string(),
                TagAttr::String(policy.csrf.same_site.clone()),
            );
            attrs.insert(
                "secureCookie".to_string(),
                TagAttr::Bool(policy.csrf.secure_cookie),
            );
        }),
        "withAuth" | "auth_with" | "auth.withAuth" => ("middleware.auth", |attrs, policy| {
            attrs.insert(
                "mode".to_string(),
                TagAttr::String(policy.auth.mode.clone()),
            );
            attrs.insert(
                "crossSiteFrontend".to_string(),
                TagAttr::Bool(policy.auth.cross_site_frontend),
            );
            attrs.insert(
                "cookieEnabled".to_string(),
                TagAttr::Bool(matches!(policy.auth.mode.as_str(), "cookie" | "mixed")),
            );
            attrs.insert(
                "tokenEnabled".to_string(),
                TagAttr::Bool(matches!(policy.auth.mode.as_str(), "token" | "mixed")),
            );
        }),
        _ => return None,
    };

    attrs_fill(&mut attrs, policy);

    Some(SecurityMiddleware {
        loc: span_to_loc(span),
        tags: vec![tag.to_string()],
        attrs,
    })
}

fn intrinsic_symbol_registry() -> Vec<SecuritySymbol> {
    vec![
        symbol("req_body", &[("source.http.body", TagKind::Source)]),
        symbol("req_query", &[("source.http.query", TagKind::Source)]),
        symbol("req_header", &[("source.http.header", TagKind::Source)]),
        symbol("req_path_param", &[("source.http.path", TagKind::Source)]),
        symbol(
            "db_write",
            &[
                ("sink.sql.exec", TagKind::Sink),
                ("effect.db.write", TagKind::Effect),
                ("capability.db", TagKind::Capability),
            ],
        ),
        symbol(
            "db_read",
            &[
                ("sink.sql.query", TagKind::Sink),
                ("effect.db.read", TagKind::Effect),
                ("capability.db", TagKind::Capability),
            ],
        ),
        symbol(
            "net_call",
            &[
                ("sink.net.public_request", TagKind::Sink),
                ("effect.net", TagKind::Effect),
                ("capability.net", TagKind::Capability),
            ],
        ),
        symbol(
            "net_internal_call",
            &[
                ("sink.net.internal_request", TagKind::Sink),
                ("effect.net", TagKind::Effect),
                ("capability.internal_net", TagKind::Capability),
            ],
        ),
        symbol(
            "secret_reveal",
            &[
                ("effect.secrets.reveal", TagKind::Effect),
                ("capability.secrets", TagKind::Capability),
            ],
        ),
        symbol(
            "log",
            &[
                ("sink.log.emit", TagKind::Sink),
                ("effect.log", TagKind::Effect),
            ],
        ),
        symbol(
            "req_json",
            &[
                ("source.http.body", TagKind::Source),
                ("gate.schema.json_decode", TagKind::Gate),
                ("effect.net", TagKind::Effect),
            ],
        ),
        symbol("res_html", &[("sink.http.html", TagKind::Sink)]),
        symbol("set_header", &[("sink.http.header_set", TagKind::Sink)]),
        symbol("set_cookie", &[("sink.http.cookie_set", TagKind::Sink)]),
        symbol(
            "fs_read",
            &[
                ("sink.fs.read", TagKind::Sink),
                ("effect.fs.read", TagKind::Effect),
                ("capability.fs", TagKind::Capability),
            ],
        ),
        symbol(
            "fs_write",
            &[
                ("sink.fs.write", TagKind::Sink),
                ("effect.fs.write", TagKind::Effect),
                ("capability.fs", TagKind::Capability),
            ],
        ),
        symbol(
            "validate_header_value",
            &[("gate.header.value", TagKind::Gate)],
        ),
        symbol("sanitize_html", &[("gate.sanitize.html", TagKind::Gate)]),
        symbol("path_under", &[("gate.path.under", TagKind::Gate)]),
        symbol(
            "url_public",
            &[
                ("gate.url.public", TagKind::Gate),
                ("effect.net", TagKind::Effect),
            ],
        ),
        symbol(
            "url_internal",
            &[
                ("gate.url.internal", TagKind::Gate),
                ("effect.net", TagKind::Effect),
            ],
        ),
        symbol(
            "res_json",
            &[("sink.json.encode_http_response", TagKind::Sink)],
        ),
        symbol(
            "db.exec",
            &[
                ("sink.sql.exec", TagKind::Sink),
                ("effect.db.write", TagKind::Effect),
                ("capability.db", TagKind::Capability),
            ],
        ),
        symbol(
            "db.queryOne",
            &[
                ("sink.sql.query", TagKind::Sink),
                ("effect.db.read", TagKind::Effect),
                ("capability.db", TagKind::Capability),
            ],
        ),
        symbol(
            "req.json",
            &[
                ("source.http.body", TagKind::Source),
                ("gate.schema.json_decode", TagKind::Gate),
                ("effect.net", TagKind::Effect),
            ],
        ),
        symbol("req.body", &[("source.http.body", TagKind::Source)]),
        symbol("req.query", &[("source.http.query", TagKind::Source)]),
        symbol("req.header", &[("source.http.header", TagKind::Source)]),
        symbol("req.pathParam", &[("source.http.path", TagKind::Source)]),
        symbol(
            "res.json",
            &[("sink.json.encode_http_response", TagKind::Sink)],
        ),
        symbol("res.html", &[("sink.http.html", TagKind::Sink)]),
        symbol("res.setHeader", &[("sink.http.header_set", TagKind::Sink)]),
        symbol("res.addCookie", &[("sink.http.cookie_set", TagKind::Sink)]),
        symbol(
            "fs.read",
            &[
                ("sink.fs.read", TagKind::Sink),
                ("effect.fs.read", TagKind::Effect),
                ("capability.fs", TagKind::Capability),
            ],
        ),
        symbol(
            "fs.write",
            &[
                ("sink.fs.write", TagKind::Sink),
                ("effect.fs.write", TagKind::Effect),
                ("capability.fs", TagKind::Capability),
            ],
        ),
        symbol(
            "validate.headerValue",
            &[("gate.header.value", TagKind::Gate)],
        ),
        symbol("sanitize.html", &[("gate.sanitize.html", TagKind::Gate)]),
        symbol("path.under", &[("gate.path.under", TagKind::Gate)]),
        symbol(
            "validate.pathUnder",
            &[("gate.path.under", TagKind::Gate)],
        ),
        symbol(
            "url.public",
            &[
                ("gate.url.public", TagKind::Gate),
                ("effect.net", TagKind::Effect),
            ],
        ),
        symbol(
            "url.internal",
            &[
                ("gate.url.internal", TagKind::Gate),
                ("effect.net", TagKind::Effect),
            ],
        ),
        symbol(
            "secrets.reveal",
            &[
                ("effect.secrets.reveal", TagKind::Effect),
                ("capability.secrets", TagKind::Capability),
            ],
        ),
        symbol(
            "log.info",
            &[
                ("sink.log.emit", TagKind::Sink),
                ("effect.log", TagKind::Effect),
            ],
        ),
        symbol(
            "httpClient.get",
            &[
                ("sink.net.public_request", TagKind::Sink),
                ("effect.net", TagKind::Effect),
                ("capability.net", TagKind::Capability),
            ],
        ),
        symbol(
            "httpClient.getInternal",
            &[
                ("sink.net.internal_request", TagKind::Sink),
                ("effect.net", TagKind::Effect),
                ("capability.internal_net", TagKind::Capability),
            ],
        ),
    ]
}

fn symbol(name: &str, tags: &[(&str, TagKind)]) -> SecuritySymbol {
    SecuritySymbol {
        sym: name.to_string(),
        tags: tags
            .iter()
            .map(|(id, kind)| Tag {
                id: (*id).to_string(),
                kind: kind.clone(),
                attrs: None,
            })
            .collect(),
    }
}

fn span_to_loc(span: &Span) -> SourceLocation {
    SourceLocation {
        file: span.file.display().to_string(),
        line: span.start_line,
        column: span.start_col,
    }
}

fn callable_name(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name.clone()),
        ExprKind::Member { object, field } => {
            let mut prefix = callable_name(object)?;
            prefix.push('.');
            prefix.push_str(field);
            Some(prefix)
        }
        _ => None,
    }
}

fn find_matching_paren(source: &str, open_paren_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut prev_was_escape = false;

    for (index, ch) in source.char_indices().skip(open_paren_index) {
        if in_string {
            if ch == '"' && !prev_was_escape {
                in_string = false;
            }
            prev_was_escape = ch == '\\' && !prev_was_escape;
            continue;
        }

        match ch {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

fn parse_allow_payload(
    file_path: &Path,
    payload: &str,
    span: &Span,
) -> Result<SecurityAllow, Vec<Diagnostic>> {
    let mut values = HashMap::<String, String>::new();
    let mut bypass = Vec::<String>::new();
    let mut diagnostics = Vec::new();

    for part in split_top_level(payload, ',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Some(eq_index) = find_assignment_equals(trimmed) else {
            diagnostics.push(
                Diagnostic::error("A7001", "invalid @allow assignment", span.clone())
                    .with_note(format!("expected `key = value`, got `{trimmed}`")),
            );
            continue;
        };

        let key = trimmed[..eq_index].trim().to_string();
        let value = trimmed[(eq_index + 1)..].trim();

        match key.as_str() {
            "bypass" => match parse_string_list(value) {
                Ok(list) => bypass = list,
                Err(message) => diagnostics.push(
                    Diagnostic::error("A7001", "invalid @allow bypass list", span.clone())
                        .with_note(message),
                ),
            },
            "policy" | "reason" | "ticket" | "expires" => match parse_quoted_string(value) {
                Ok(parsed) => {
                    values.insert(key, parsed);
                }
                Err(message) => diagnostics.push(
                    Diagnostic::error("A7001", "invalid @allow string value", span.clone())
                        .with_note(format!("{message}; key `{key}`")),
                ),
            },
            unknown => diagnostics.push(
                Diagnostic::error("A7001", "unknown @allow field", span.clone())
                    .with_note(format!("unsupported key `{unknown}` in @allow annotation")),
            ),
        }
    }

    for required in ALLOW_REQUIRED_FIELDS {
        if required == "bypass" {
            if bypass.is_empty() {
                diagnostics.push(
                    Diagnostic::error("A7001", "missing required @allow field", span.clone())
                        .with_note("`bypass` must include at least one tag id"),
                );
            }
            continue;
        }

        if !values.contains_key(required) {
            diagnostics.push(
                Diagnostic::error("A7001", "missing required @allow field", span.clone())
                    .with_note(format!("`{required}` is required in @allow annotation")),
            );
        }
    }

    if let Some(expiry) = values.get("expires") {
        let today = current_utc_iso_date();
        if !is_iso_date(expiry) {
            diagnostics.push(
                Diagnostic::error("A7001", "invalid @allow expiry format", span.clone())
                    .with_note("expires must be formatted as YYYY-MM-DD"),
            );
        } else if expiry.as_str() < today.as_str() {
            diagnostics.push(
                Diagnostic::error("A7002", "expired @allow annotation", span.clone()).with_note(
                    format!("expires `{}` is in the past (today is {})", expiry, today),
                ),
            );
        }
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(SecurityAllow {
        loc: SourceLocation {
            file: file_path.display().to_string(),
            line: span.start_line,
            column: span.start_col,
        },
        policy: values.remove("policy").unwrap_or_default(),
        bypass,
        reason: values.remove("reason").unwrap_or_default(),
        ticket: values.remove("ticket").unwrap_or_default(),
        expires: values.remove("expires").unwrap_or_default(),
    })
}

fn split_top_level(input: &str, delimiter: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut depth_square = 0usize;
    let mut depth_paren = 0usize;
    let mut in_string = false;
    let mut prev_escape = false;

    for (index, ch) in input.char_indices() {
        if in_string {
            if ch == '"' && !prev_escape {
                in_string = false;
            }
            prev_escape = ch == '\\' && !prev_escape;
            continue;
        }

        match ch {
            '"' => in_string = true,
            '[' => depth_square += 1,
            ']' => depth_square = depth_square.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            _ if ch == delimiter && depth_square == 0 && depth_paren == 0 => {
                parts.push(input[start..index].to_string());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    if start < input.len() {
        parts.push(input[start..].to_string());
    }

    parts
}

fn find_assignment_equals(input: &str) -> Option<usize> {
    let mut in_string = false;
    let mut depth_square = 0usize;
    let mut prev_escape = false;

    for (index, ch) in input.char_indices() {
        if in_string {
            if ch == '"' && !prev_escape {
                in_string = false;
            }
            prev_escape = ch == '\\' && !prev_escape;
            continue;
        }

        match ch {
            '"' => in_string = true,
            '[' => depth_square += 1,
            ']' => depth_square = depth_square.saturating_sub(1),
            '=' if depth_square == 0 => return Some(index),
            _ => {}
        }
    }

    None
}

fn parse_string_list(raw: &str) -> Result<Vec<String>, String> {
    let trimmed = raw.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return Err("bypass must be a list literal like [\"tag.one\", \"tag.two\"]".to_string());
    }

    let inner = &trimmed[1..(trimmed.len() - 1)];
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut values = Vec::new();
    for part in split_top_level(inner, ',') {
        values.push(parse_quoted_string(part.trim())?);
    }
    Ok(values)
}

fn parse_quoted_string(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.len() < 2 || !trimmed.starts_with('"') || !trimmed.ends_with('"') {
        return Err(format!("expected quoted string, got `{trimmed}`"));
    }
    Ok(trimmed[1..(trimmed.len() - 1)].to_string())
}

fn is_iso_date(input: &str) -> bool {
    let bytes = input.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    matches!(bytes[4], b'-')
        && matches!(bytes[7], b'-')
        && bytes.iter().enumerate().all(|(index, byte)| {
            if index == 4 || index == 7 {
                true
            } else {
                byte.is_ascii_digit()
            }
        })
}

fn span_from_offset(file_path: &Path, source: &str, offset: usize) -> Span {
    let mut line = 1usize;
    let mut col = 1usize;
    for ch in source[..offset].chars() {
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    Span::point(file_path.to_path_buf(), line, col)
}

fn current_utc_iso_date() -> String {
    let days_since_unix_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| (duration.as_secs() / 86_400) as i64)
        .unwrap_or(0);
    let (year, month, day) = civil_from_days(days_since_unix_epoch);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    // Gregorian calendar conversion adapted from Howard Hinnant's civil-from-days algorithm.
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    (year, month, day)
}
