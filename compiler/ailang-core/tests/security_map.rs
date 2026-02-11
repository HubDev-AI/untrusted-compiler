use ailang_core::{
    build_security_map, parse_allow_annotations, parse_source, strip_allow_annotations, Policy,
};
use std::path::Path;

#[test]
fn security_map_collects_sensitive_calls_and_middleware() {
    let source = r#"
fn boot() -> Int {
  withSecurityHeaders();
  withCors();
  withCsrf();
  withAuth();
  db_write(DbCap());
  secret_reveal(SecretsCap());
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
        .middleware
        .iter()
        .any(|entry| entry.tags.iter().any(|tag| tag == "middleware.cors")));
    assert!(map.middleware.iter().any(|entry| entry
        .tags
        .iter()
        .any(|tag| tag == "middleware.security_headers")));
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
