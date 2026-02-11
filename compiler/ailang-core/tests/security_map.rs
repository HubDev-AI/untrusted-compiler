use ailang_core::{build_security_map, parse_source, Policy};
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
