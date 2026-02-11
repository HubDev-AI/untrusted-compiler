use ailang_core::policy::parse_policy_str;
use std::path::Path;

#[test]
fn policy_defaults_when_empty() {
    let policy = parse_policy_str(Path::new("ailang.policy"), "").expect("empty policy should parse");
    assert_eq!(policy.name, "default-secure");
    assert_eq!(policy.version, "0.1");
    assert!(policy.forbidden_effects.contains("shell"));
    assert!(policy.forbidden_effects.contains("unsafe"));
    assert!(policy.forbidden_effects.contains("secrets.reveal"));
}

#[test]
fn policy_rejects_unknown_keys() {
    let source = r#"
[policy]
name = "x"
bogus = true
"#;

    let diagnostics = parse_policy_str(Path::new("ailang.policy"), source)
        .expect_err("unknown keys must fail parsing");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6002"));
}

#[test]
fn policy_rejects_cors_wildcard_with_credentials() {
    let source = r#"
[cors]
allowed_origins = ["*"]
allow_credentials = true
"#;

    let diagnostics = parse_policy_str(Path::new("ailang.policy"), source)
        .expect_err("wildcard credentials CORS must be rejected");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_invalid_redirect_settings() {
    let source = r#"
[net.public]
allow_redirects = false
max_redirects = 1
"#;

    let diagnostics = parse_policy_str(Path::new("ailang.policy"), source)
        .expect_err("invalid redirect combo must be rejected");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}
