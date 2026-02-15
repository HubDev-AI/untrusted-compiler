use sec4_core::policy::parse_policy_str;
use std::path::Path;

#[test]
fn policy_defaults_when_empty() {
    let policy = parse_policy_str(Path::new("sec4.policy"), "").expect("empty policy should parse");
    assert_eq!(policy.name, "default-secure");
    assert_eq!(policy.version, "0.1");
    assert!(policy.forbidden_effects.contains("shell"));
    assert!(policy.forbidden_effects.contains("unsafe"));
    assert!(policy.forbidden_effects.contains("secrets.reveal"));
    assert!(policy.json.require_schema_for_encode);
}

#[test]
fn policy_rejects_unknown_keys() {
    let source = r#"
[policy]
name = "x"
bogus = true
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
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

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("wildcard credentials CORS must be rejected");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_forbidden_cors_origin_reflection() {
    let source = r#"
[cors]
reflect_origin = true
forbid_reflect_origin = true
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("forbidden CORS reflection must be rejected");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_allows_cors_origin_reflection_when_not_forbidden() {
    let source = r#"
[cors]
reflect_origin = true
forbid_reflect_origin = false
"#;

    let policy = parse_policy_str(Path::new("sec4.policy"), source)
        .expect("CORS reflection should parse when forbid_reflect_origin is false");
    assert!(policy.cors.reflect_origin);
    assert!(!policy.cors.forbid_reflect_origin);
}

#[test]
fn policy_rejects_invalid_redirect_settings() {
    let source = r#"
[net.public]
allow_redirects = false
max_redirects = 1
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("invalid redirect combo must be rejected");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_parses_public_egress_domain_lists() {
    let source = r#"
[net.public]
allowed_schemes = ["http", "https"]
allowed_domains = ["api.example.com"]
blocked_domains = ["169.254.169.254"]
"#;

    let policy = parse_policy_str(Path::new("sec4.policy"), source)
        .expect("public egress domain lists should parse");
    assert_eq!(
        policy.net_public.allowed_schemes,
        vec!["http".to_string(), "https".to_string()]
    );
    assert_eq!(
        policy.net_public.allowed_domains,
        vec!["api.example.com".to_string()]
    );
    assert_eq!(
        policy.net_public.blocked_domains,
        vec!["169.254.169.254".to_string()]
    );
}

#[test]
fn policy_parses_public_egress_allowed_ports() {
    let source = r#"
[net.public]
allowed_ports = [80, 443, 8443]
"#;

    let policy =
        parse_policy_str(Path::new("sec4.policy"), source).expect("allowed ports should parse");
    assert_eq!(policy.net_public.allowed_ports, vec![80, 443, 8443]);
}

#[test]
fn policy_rejects_invalid_public_egress_allowed_ports() {
    let source = r#"
[net.public]
allowed_ports = [0, 65536, -1]
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("invalid allowed_ports must fail");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_parses_fs_forbid_symlinks_mode() {
    let source = r#"
[fs]
enabled = true
forbid_symlinks = "warn"
"#;

    let policy =
        parse_policy_str(Path::new("sec4.policy"), source).expect("fs symlink mode should parse");
    assert_eq!(policy.fs.forbid_symlinks, "warn");
}

#[test]
fn policy_rejects_invalid_fs_forbid_symlinks_mode() {
    let source = r#"
[fs]
forbid_symlinks = "invalid"
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("invalid fs.forbid_symlinks must fail");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_csrf_none_without_secure_cookie() {
    let source = r#"
[csrf]
same_site = "None"
secure_cookie = false
"#;

    let diagnostics =
        parse_policy_str(Path::new("sec4.policy"), source).expect_err("invalid csrf must fail");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_cookie_auth_without_csrf() {
    let source = r#"
[auth]
mode = "cookie"

[csrf]
enabled = false
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("cookie auth must require csrf");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_cross_site_cookie_without_cors_credentials() {
    let source = r#"
[auth]
mode = "cookie"
cross_site_frontend = true

[csrf]
enabled = true

[cors]
allowed_origins = ["https://app.example.com"]
allow_credentials = false
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("cross-site cookie auth must require cors credentials");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_invalid_sql_limit_mode() {
    let source = r#"
[sql]
require_limit_on_select = "invalid"
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("invalid sql.require_limit_on_select must fail");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_allows_toggling_json_schema_encode_requirement() {
    let source = r#"
[json]
require_schema_for_encode = false
"#;

    let policy = parse_policy_str(Path::new("sec4.policy"), source)
        .expect("json schema encode flag should parse");
    assert!(!policy.json.require_schema_for_encode);
}

#[test]
fn policy_profile_default_secure_prod_parses() {
    let source = include_str!("../../../policies/default-secure-prod.sec4.policy");
    let policy = parse_policy_str(Path::new("sec4.policy"), source)
        .expect("default-secure-prod policy should parse");

    assert_eq!(policy.name, "default-secure-prod");
    assert_eq!(policy.env, "prod");
    assert_eq!(policy.mode_as_str(), "enforce");
    assert_eq!(
        policy.cors.allowed_origins,
        vec!["https://app.example.com".to_string()]
    );
    assert!(policy.cors.allow_credentials);
    assert!(policy.security_headers.hsts_enabled);
    assert!(!policy.net_internal.enabled);
    assert!(!policy.fs.enabled);
    assert_eq!(policy.replay.effects, "deny");
}

#[test]
fn policy_profile_permissive_dev_parses() {
    let source = include_str!("../../../policies/permissive-dev.sec4.policy");
    let policy =
        parse_policy_str(Path::new("sec4.policy"), source).expect("permissive-dev should parse");

    assert_eq!(policy.name, "permissive-dev");
    assert_eq!(policy.env, "dev");
    assert_eq!(policy.mode_as_str(), "warn");
    assert_eq!(policy.cors.allowed_origins, vec!["*".to_string()]);
    assert!(!policy.cors.allow_credentials);
    assert!(policy.net_internal.enabled);
    assert!(policy.fs.enabled);
    assert_eq!(policy.replay.effects, "allow");
}
