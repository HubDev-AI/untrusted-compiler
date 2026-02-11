use crate::ast::{Block, Expr, ExprKind, ItemKind, Program, StmtKind};
use crate::policy::Policy;
use crate::Span;
use serde::Serialize;
use std::collections::HashMap;

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
            if let ExprKind::Identifier(name) = &callee.kind {
                if let Some(tags) = call_tags_for(name) {
                    calls.push(SecurityCall {
                        loc: span_to_loc(&expr.span),
                        callee: name.clone(),
                        tags: tags.into_iter().map(str::to_string).collect(),
                    });
                }

                if let Some(middleware_entry) = middleware_for(name, &expr.span, policy) {
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
        "req_body" => vec!["source.http.body"],
        "req_query" => vec!["source.http.query"],
        "req_header" => vec!["source.http.header"],
        "req_path_param" => vec!["source.http.path"],
        "req_json" => vec!["source.http.body", "gate.schema.json_decode", "effect.net"],
        "db_read" => vec!["sink.sql.query", "effect.db.read", "capability.db"],
        "db_write" => vec!["sink.sql.exec", "effect.db.write", "capability.db"],
        "res_html" => vec!["sink.http.html"],
        "set_header" => vec!["sink.http.header_set"],
        "set_cookie" => vec!["sink.http.cookie_set"],
        "res_json" => vec!["sink.json.encode_http_response"],
        "net_call" => vec!["sink.net.public_request", "effect.net", "capability.net"],
        "net_internal_call" => vec![
            "sink.net.internal_request",
            "effect.net",
            "capability.internal_net",
        ],
        "fs_read" => vec!["sink.fs.read", "effect.fs.read", "capability.fs"],
        "fs_write" => vec!["sink.fs.write", "effect.fs.write", "capability.fs"],
        "log" => vec!["sink.log.emit", "effect.log"],
        "secret_read" => vec!["effect.secrets.read", "capability.secrets"],
        "secret_reveal" => vec!["effect.secrets.reveal", "capability.secrets"],
        _ => return None,
    };

    Some(tags)
}

fn middleware_for(name: &str, span: &Span, policy: &Policy) -> Option<SecurityMiddleware> {
    let mut attrs = HashMap::new();
    let (tag, attrs_fill): (&str, fn(&mut HashMap<String, TagAttr>, &Policy)) = match name {
        "withCors" | "cors_with" => ("middleware.cors", |attrs, policy| {
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
        "withSecurityHeaders" | "sec_with_security_headers" => {
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
        "withCsrf" | "csrf_with" => ("middleware.csrf", |attrs, policy| {
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
        "withAuth" | "auth_with" => ("middleware.auth", |attrs, policy| {
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
        symbol(
            "res_json",
            &[("sink.json.encode_http_response", TagKind::Sink)],
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
