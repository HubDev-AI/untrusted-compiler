use ailang_core::{
    build_security_map, parse_allow_annotations, parse_source, strip_allow_annotations, Policy,
};
use std::path::Path;

#[test]
fn security_map_collects_sensitive_calls_and_middleware() {
    let source = r#"
fn boot() -> Int {
  sec.withSecurityHeaders();
  cors.withCors();
  csrf.withCsrf();
  auth.withAuth();
  let raw = req.query("q");
  validate.email(raw);
  validate.headerValue(raw);
  sanitize.html(raw);
  url.public(raw);
  path.under(PathSafe(), raw);
  res.json("UserSchema", 1);
  db.exec(DbCap());
  db.exec(Ctx(), DbCap(), raw);
  secrets.reveal(SecretsCap());
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let map = build_security_map(&program, &Policy::default());

    assert!(map
        .calls
        .iter()
        .any(|call| call.tags.iter().any(|tag| tag == "sink.sql.exec")));
    assert!(map
        .calls
        .iter()
        .any(|call| call.tags.iter().any(|tag| tag == "effect.secrets.reveal")));
    assert!(map
        .calls
        .iter()
        .any(|call| call.tags.iter().any(|tag| tag == "source.http.query")));
    assert!(map
        .calls
        .iter()
        .any(|call| call.tags.iter().any(|tag| tag == "gate.validate.email")));
    assert!(map
        .calls
        .iter()
        .any(|call| call.tags.iter().any(|tag| tag == "gate.header.value")));
    assert!(map
        .calls
        .iter()
        .any(|call| call.tags.iter().any(|tag| tag == "gate.sanitize.html")));
    assert!(map
        .calls
        .iter()
        .any(|call| call.tags.iter().any(|tag| tag == "gate.url.public")));
    assert!(map
        .calls
        .iter()
        .any(|call| call.tags.iter().any(|tag| tag == "gate.path.under")));
    assert!(map.calls.iter().any(|call| {
        call.callee == "res.json"
            && call
                .arg_roles
                .as_ref()
                .is_some_and(|roles| roles == &vec!["schema".to_string(), "value".to_string()])
    }));
    assert!(map.calls.iter().any(|call| {
        call.callee == "db.exec"
            && call
                .arg_roles
                .as_ref()
                .is_some_and(|roles| roles == &vec!["capability".to_string()])
    }));
    assert!(map.calls.iter().any(|call| {
        call.callee == "db.exec"
            && call.arg_roles.as_ref().is_some_and(|roles| {
                roles
                    == &vec![
                        "context".to_string(),
                        "capability".to_string(),
                        "query".to_string(),
                    ]
            })
            && call.origin_edges.as_ref().is_some_and(|edges| {
                edges.iter().any(|edge| {
                    edge.arg_index == 2
                        && edge.origin == "call:req.query"
                        && edge.tags.iter().any(|tag| tag == "source.http.query")
                })
            })
    }));
    assert!(map
        .middleware
        .iter()
        .any(|entry| entry.tags.iter().any(|tag| tag == "middleware.cors")));
    assert!(map.middleware.iter().any(|entry| entry
        .tags
        .iter()
        .any(|tag| tag == "middleware.security_headers")));
}

#[test]
fn security_map_symbol_registry_contains_gate_and_source_tags() {
    let source = r#"
fn boot() -> Int {
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let map = build_security_map(&program, &Policy::default());

    let has_symbol_tag = |symbol: &str, tag: &str| {
        map.symbols.iter().any(|entry| {
            entry.sym == symbol && entry.tags.iter().any(|symbol_tag| symbol_tag.id == tag)
        })
    };

    assert!(has_symbol_tag("req.query", "source.http.query"));
    assert!(has_symbol_tag("req.pathParam", "source.http.path"));
    assert!(has_symbol_tag("validate.headerValue", "gate.header.value"));
    assert!(has_symbol_tag("validate.email", "gate.validate.email"));
    assert!(has_symbol_tag("sanitize.html", "gate.sanitize.html"));
    assert!(has_symbol_tag("path.under", "gate.path.under"));
    assert!(has_symbol_tag("url.public", "gate.url.public"));
    assert!(has_symbol_tag("url.internal", "gate.url.internal"));
}

#[test]
fn security_map_tags_sql_select_without_limit_calls() {
    let source = r#"
fn bad() -> Int {
  db.exec(DbCap(), "SELECT id FROM users");
  db.exec(Ctx(), DbCap(), "SELECT email FROM users");
  db.exec(DbCap(), "SELECT id FROM users LIMIT 1");
  db.exec(Ctx(), DbCap(), "SELECT email FROM users LIMIT 10");
  db.exec(DbCap(), sql.q("SELECT name FROM users", List()));
  db.exec(DbCap(), sql.q("SELECT name FROM users LIMIT 5", List()));
  db.exec(DbCap(), "SELECT 'limit' as marker FROM users");
  db.exec(DbCap(), "SELECT id FROM users -- limit\n");
  db.exec(DbCap(), "SELECT id FROM users /* limit */");
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let map = build_security_map(&program, &Policy::default());
    let flagged = map
        .calls
        .iter()
        .filter(|call| {
            call.tags
                .iter()
                .any(|tag| tag == "sql.select_without_limit")
        })
        .count();

    assert_eq!(flagged, 6);
}

#[test]
fn security_map_still_supports_underscore_intrinsic_names() {
    let source = r#"
fn boot() -> Int {
  db_write(DbCap());
  secret_reveal(SecretsCap());
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let map = build_security_map(&program, &Policy::default());
    assert!(map.calls.iter().any(
        |call| call.callee == "db_write" && call.tags.iter().any(|tag| tag == "sink.sql.exec")
    ));
    assert!(map.calls.iter().any(|call| {
        call.callee == "secret_reveal" && call.tags.iter().any(|tag| tag == "effect.secrets.reveal")
    }));
}

#[test]
fn security_map_tracks_origin_edges_for_sink_arguments() {
    let source = r#"
fn boot() -> Int {
  let raw = req.query("q");
  db.exec(DbCap(), raw);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let map = build_security_map(&program, &Policy::default());

    assert!(map.calls.iter().any(|call| {
        call.callee == "db.exec"
            && call
                .arg_roles
                .as_ref()
                .is_some_and(|roles| roles == &vec!["capability".to_string(), "query".to_string()])
            && call.origin_edges.as_ref().is_some_and(|edges| {
                edges.iter().any(|edge| {
                    edge.arg_index == 1
                        && edge.origin == "call:req.query"
                        && edge.tags.iter().any(|tag| tag == "source.http.query")
                })
            })
    }));
}

#[test]
fn security_map_tracks_origin_edges_through_composite_expressions() {
    let source = r#"
fn boot() -> Int {
  let raw = req.query("q");
  let merged = raw + " suffix";
  db.exec(DbCap(), merged);
  db.exec(DbCap(), if true { raw } else { raw });
  db.exec(DbCap(), match true { true => raw, false => raw });
  db.exec(DbCap(), { let shadow = raw; shadow });
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let map = build_security_map(&program, &Policy::default());
    let traced = map
        .calls
        .iter()
        .filter(|call| call.callee == "db.exec")
        .filter(|call| {
            call.origin_edges.as_ref().is_some_and(|edges| {
                edges.iter().any(|edge| {
                    edge.arg_index == 1
                        && edge.origin == "call:req.query"
                        && edge.tags.iter().any(|tag| tag == "source.http.query")
                })
            })
        })
        .count();

    assert_eq!(traced, 4);
}

#[test]
fn security_map_tracks_origin_edges_across_forwarding_functions() {
    let source = r#"
fn queryParam() -> String {
  req.query("q")
}

fn passThrough(input: String) -> String {
  input
}

fn boot() -> Int {
  let raw = passThrough(queryParam());
  db.exec(DbCap(), raw);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let map = build_security_map(&program, &Policy::default());
    assert!(map.calls.iter().any(|call| {
        call.callee == "db.exec"
            && call.origin_edges.as_ref().is_some_and(|edges| {
                edges.iter().any(|edge| {
                    edge.arg_index == 1
                        && edge.origin == "call:queryParam"
                        && edge.tags.iter().any(|tag| tag == "source.http.query")
                })
            })
    }));
}

#[test]
fn security_map_resolves_intrinsic_alias_value_calls() {
    let source = r#"
fn boot() -> Int {
  let raw = req.query("q");
  let exec = db.exec;
  exec(DbCap(), raw);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let map = build_security_map(&program, &Policy::default());
    assert!(map.calls.iter().any(|call| {
        call.callee == "db.exec"
            && call.tags.iter().any(|tag| tag == "sink.sql.exec")
            && call
                .arg_roles
                .as_ref()
                .is_some_and(|roles| roles == &vec!["capability".to_string(), "query".to_string()])
            && call.origin_edges.as_ref().is_some_and(|edges| {
                edges.iter().any(|edge| {
                    edge.arg_index == 1
                        && edge.origin == "call:req.query"
                        && edge.tags.iter().any(|tag| tag == "source.http.query")
                })
            })
    }));
}

#[test]
fn parse_allow_annotations_reads_valid_annotation() {
    let source = r#"
@allow(
  policy = "net.internal.enabled",
  bypass = ["sink.net.internal_request"],
  reason = "Calls inventory service",
  ticket = "SEC-123",
  expires = "2099-01-01",
)
fn main() -> Int { 1 }
"#;

    let annotations = parse_allow_annotations(Path::new("main.ai"), source)
        .expect("@allow annotation should parse");
    assert_eq!(annotations.len(), 1);
    let allow = &annotations[0];
    assert_eq!(allow.policy, "net.internal.enabled");
    assert_eq!(allow.bypass, vec!["sink.net.internal_request".to_string()]);
    assert_eq!(allow.reason, "Calls inventory service");
    assert_eq!(allow.ticket, "SEC-123");
    assert_eq!(allow.expires, "2099-01-01");
}

#[test]
fn parse_allow_annotations_rejects_missing_required_fields() {
    let source = r#"
@allow(
  policy = "effects.forbid",
  bypass = ["effect.secrets.reveal"],
  reason = "temporary debugging",
  expires = "2099-01-01",
)
fn main() -> Int { 1 }
"#;

    let diagnostics = parse_allow_annotations(Path::new("main.ai"), source)
        .expect_err("missing ticket must be rejected");
    assert!(diagnostics.iter().any(|diag| diag.code == "A7001"));
}

#[test]
fn parse_allow_annotations_rejects_expired_annotations() {
    let source = r#"
@allow(
  policy = "effects.forbid",
  bypass = ["effect.secrets.reveal"],
  reason = "legacy path",
  ticket = "SEC-124",
  expires = "2000-01-01",
)
fn main() -> Int { 1 }
"#;

    let diagnostics = parse_allow_annotations(Path::new("main.ai"), source)
        .expect_err("expired annotation must be rejected");
    assert!(diagnostics.iter().any(|diag| diag.code == "A7002"));
}

#[test]
fn strip_allow_annotations_keeps_source_parseable() {
    let source = r#"
@allow(
  policy = "effects.forbid",
  bypass = ["effect.secrets.reveal"],
  reason = "legacy path",
  ticket = "SEC-124",
  expires = "2099-01-01",
)
fn main() -> Int { 1 }
"#;
    let stripped = strip_allow_annotations(source);
    let program =
        parse_source(Path::new("main.ai"), &stripped).expect("source should parse after stripping");
    assert_eq!(program.items.len(), 1);
}
