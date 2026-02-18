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
    assert_eq!(
        policy.cors.allowed_methods,
        vec![
            "GET".to_string(),
            "POST".to_string(),
            "PUT".to_string(),
            "PATCH".to_string(),
            "DELETE".to_string(),
            "OPTIONS".to_string(),
        ]
    );
    assert_eq!(
        policy.cors.allowed_headers,
        vec!["content-type".to_string(), "authorization".to_string()]
    );
    assert!(policy.cors.exposed_headers.is_empty());
    assert_eq!(policy.cors.max_age_seconds, 600);
    assert_eq!(policy.json.max_bytes, 1_048_576);
    assert_eq!(policy.json.max_depth, 32);
    assert!(policy.json.require_schema_for_encode);
    assert_eq!(policy.http.max_body_bytes, 4_096);
    assert_eq!(policy.http.max_concurrency, 256);
    assert_eq!(policy.http.max_keep_alive_requests, 256);
    assert_eq!(policy.http.max_header_bytes, 8_191);
    assert_eq!(policy.http.max_multipart_bytes, 4_096);
    assert_eq!(policy.http.default_timeout_ms, 200);
    assert_eq!(policy.net_public.max_redirects, 0);
    assert!(policy.net_ssrf.block_private_ranges);
    assert!(policy.net_ssrf.block_loopback);
    assert!(policy.net_ssrf.block_link_local);
    assert!(policy.net_ssrf.block_metadata_ips);
    assert!(!policy.cors.allow_private_network);
    assert_eq!(policy.security_headers.hsts_max_age_seconds, 15552000);
    assert!(policy.security_headers.hsts_include_subdomains);
    assert!(!policy.security_headers.hsts_preload);
    assert!(policy.security_headers.csp_enabled);
    assert!(!policy.security_headers.csp_report_only);
    assert_eq!(
        policy.security_headers.csp_policy,
        "default-src 'self'; frame-ancestors 'none'; base-uri 'self'"
    );
    assert_eq!(policy.csrf.cookie_name, "csrf");
    assert_eq!(policy.csrf.header_name, "X-CSRF-Token");
    assert_eq!(policy.auth.cookie_name, "session");
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
fn policy_parses_cors_allow_private_network_toggle() {
    let source = r#"
[cors]
allow_private_network = true
"#;

    let policy = parse_policy_str(Path::new("sec4.policy"), source)
        .expect("cors.allow_private_network should parse");
    assert!(policy.cors.allow_private_network);
}

#[test]
fn policy_parses_cors_preflight_lists_and_max_age() {
    let source = r#"
[cors]
allowed_methods = ["POST", "OPTIONS"]
allowed_headers = ["x-auth-token", "content-type"]
exposed_headers = ["x-trace-id", "x-request-id"]
max_age_seconds = 7200
"#;

    let policy = parse_policy_str(Path::new("sec4.policy"), source)
        .expect("cors preflight lists and max_age should parse");
    assert_eq!(
        policy.cors.allowed_methods,
        vec!["POST".to_string(), "OPTIONS".to_string()]
    );
    assert_eq!(
        policy.cors.allowed_headers,
        vec!["x-auth-token".to_string(), "content-type".to_string()]
    );
    assert_eq!(
        policy.cors.exposed_headers,
        vec!["x-trace-id".to_string(), "x-request-id".to_string()]
    );
    assert_eq!(policy.cors.max_age_seconds, 7200);
}

#[test]
fn policy_rejects_invalid_cors_max_age_seconds() {
    let source = r#"
[cors]
max_age_seconds = 0
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("cors.max_age_seconds must be >= 1");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
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
fn policy_rejects_negative_public_max_redirects() {
    let source = r#"
[net.public]
allow_redirects = true
max_redirects = -1
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("negative max_redirects must be rejected");
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
fn policy_parses_csrf_cookie_and_header_names() {
    let source = r#"
[csrf]
cookie_name = "sid"
header_name = "x-sid-csrf"
"#;

    let policy = parse_policy_str(Path::new("sec4.policy"), source)
        .expect("csrf cookie/header names should parse");
    assert_eq!(policy.csrf.cookie_name, "sid");
    assert_eq!(policy.csrf.header_name, "x-sid-csrf");
}

#[test]
fn policy_rejects_empty_csrf_cookie_name() {
    let source = r#"
[csrf]
cookie_name = "   "
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("empty csrf.cookie_name must fail");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_empty_csrf_header_name() {
    let source = r#"
[csrf]
header_name = "  "
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("empty csrf.header_name must fail");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_parses_security_headers_hsts_and_csp_fields() {
    let source = r#"
[security_headers]
enabled = true
x_content_type_options = true
x_frame_options = "DENY"
referrer_policy = "strict-origin"

[security_headers.hsts]
enabled = true
max_age_seconds = 63072000
include_subdomains = false
preload = true

[security_headers.csp]
enabled = true
report_only = true
policy = "default-src 'none'; frame-ancestors 'none'"
"#;

    let policy = parse_policy_str(Path::new("sec4.policy"), source)
        .expect("security headers hsts/csp fields should parse");
    assert!(policy.security_headers.enabled);
    assert!(policy.security_headers.hsts_enabled);
    assert_eq!(policy.security_headers.hsts_max_age_seconds, 63072000);
    assert!(!policy.security_headers.hsts_include_subdomains);
    assert!(policy.security_headers.hsts_preload);
    assert!(policy.security_headers.csp_enabled);
    assert!(policy.security_headers.csp_report_only);
    assert_eq!(
        policy.security_headers.csp_policy,
        "default-src 'none'; frame-ancestors 'none'"
    );
}

#[test]
fn policy_rejects_invalid_security_headers_hsts_max_age() {
    let source = r#"
[security_headers.hsts]
enabled = true
max_age_seconds = 0
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("hsts max_age must be >= 1 when hsts enabled");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_empty_security_headers_csp_policy() {
    let source = r#"
[security_headers.csp]
policy = "   "
"#;

    let diagnostics =
        parse_policy_str(Path::new("sec4.policy"), source).expect_err("empty csp policy must fail");
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
fn policy_parses_auth_cookie_name() {
    let source = r#"
[auth]
mode = "cookie"

[auth.cookie]
cookie_name = "sid"
"#;

    let policy =
        parse_policy_str(Path::new("sec4.policy"), source).expect("auth cookie_name should parse");
    assert_eq!(policy.auth.mode, "cookie");
    assert_eq!(policy.auth.cookie_name, "sid");
}

#[test]
fn policy_rejects_empty_auth_cookie_name() {
    let source = r#"
[auth]
mode = "cookie"

[auth.cookie]
cookie_name = "   "
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("empty auth.cookie.cookie_name must fail");
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
fn policy_parses_json_and_redirect_limits() {
    let source = r#"
[json]
max_bytes = 4096
max_depth = 8

[net.public]
allow_redirects = true
max_redirects = 7
"#;

    let policy = parse_policy_str(Path::new("sec4.policy"), source).expect("limits should parse");
    assert_eq!(policy.json.max_bytes, 4096);
    assert_eq!(policy.json.max_depth, 8);
    assert_eq!(policy.net_public.max_redirects, 7);
}

#[test]
fn policy_parses_net_ssrf_block_toggles() {
    let source = r#"
[net.ssrf]
block_private_ranges = false
block_loopback = false
block_link_local = false
block_metadata_ips = false
resolve_dns = false
revalidate_redirects = false
"#;

    let policy = parse_policy_str(Path::new("sec4.policy"), source)
        .expect("net.ssrf block toggles should parse");
    assert!(!policy.net_ssrf.block_private_ranges);
    assert!(!policy.net_ssrf.block_loopback);
    assert!(!policy.net_ssrf.block_link_local);
    assert!(!policy.net_ssrf.block_metadata_ips);
    assert!(!policy.net_ssrf.resolve_dns);
    assert!(!policy.net_ssrf.revalidate_redirects);
}

#[test]
fn policy_rejects_invalid_json_max_bytes() {
    let source = r#"
[json]
max_bytes = 0
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("json.max_bytes must be >= 1");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_invalid_json_max_depth() {
    let source = r#"
[json]
max_depth = 0
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("json.max_depth must be >= 1");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_parses_http_body_and_timeout_limits() {
    let source = r#"
[http]
max_body_bytes = 262144
max_concurrency = 96
max_keep_alive_requests = 64
max_header_bytes = 16384
max_multipart_bytes = 131072
default_timeout_ms = 7500
"#;

    let policy =
        parse_policy_str(Path::new("sec4.policy"), source).expect("http limits should parse");
    assert_eq!(policy.http.max_body_bytes, 262144);
    assert_eq!(policy.http.max_concurrency, 96);
    assert_eq!(policy.http.max_keep_alive_requests, 64);
    assert_eq!(policy.http.max_header_bytes, 16384);
    assert_eq!(policy.http.max_multipart_bytes, 131072);
    assert_eq!(policy.http.default_timeout_ms, 7500);
}

#[test]
fn policy_rejects_invalid_http_max_body_bytes() {
    let source = r#"
[http]
max_body_bytes = 0
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("http.max_body_bytes must be >= 1");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_invalid_http_max_header_bytes() {
    let source = r#"
[http]
max_header_bytes = 0
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("http.max_header_bytes must be >= 1");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_invalid_http_max_concurrency() {
    let source = r#"
[http]
max_concurrency = 0
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("http.max_concurrency must be >= 1");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_invalid_http_max_keep_alive_requests() {
    let source = r#"
[http]
max_keep_alive_requests = 0
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("http.max_keep_alive_requests must be >= 1");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_invalid_http_max_multipart_bytes() {
    let source = r#"
[http]
max_multipart_bytes = 0
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("http.max_multipart_bytes must be >= 1");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
}

#[test]
fn policy_rejects_invalid_http_default_timeout_ms() {
    let source = r#"
[http]
default_timeout_ms = 0
"#;

    let diagnostics = parse_policy_str(Path::new("sec4.policy"), source)
        .expect_err("http.default_timeout_ms must be >= 1");
    assert!(diagnostics.iter().any(|diag| diag.code == "P6003"));
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
    assert_eq!(
        policy.cors.allowed_methods,
        vec![
            "GET".to_string(),
            "POST".to_string(),
            "PUT".to_string(),
            "DELETE".to_string()
        ]
    );
    assert_eq!(
        policy.cors.allowed_headers,
        vec!["content-type".to_string(), "authorization".to_string()]
    );
    assert!(policy.cors.exposed_headers.is_empty());
    assert_eq!(policy.cors.max_age_seconds, 600);
    assert!(policy.cors.allow_credentials);
    assert!(!policy.cors.allow_private_network);
    assert_eq!(policy.json.max_bytes, 1_048_576);
    assert_eq!(policy.json.max_depth, 32);
    assert_eq!(policy.http.max_body_bytes, 1_048_576);
    assert_eq!(policy.http.max_concurrency, 256);
    assert_eq!(policy.http.max_keep_alive_requests, 256);
    assert_eq!(policy.http.max_header_bytes, 32768);
    assert_eq!(policy.http.max_multipart_bytes, 4_096);
    assert_eq!(policy.http.default_timeout_ms, 5000);
    assert!(policy.security_headers.hsts_enabled);
    assert_eq!(policy.security_headers.hsts_max_age_seconds, 15552000);
    assert!(policy.security_headers.hsts_include_subdomains);
    assert!(!policy.security_headers.hsts_preload);
    assert!(policy.security_headers.csp_enabled);
    assert!(!policy.security_headers.csp_report_only);
    assert_eq!(
        policy.security_headers.csp_policy,
        "default-src 'self'; frame-ancestors 'none'; base-uri 'self'"
    );
    assert_eq!(policy.csrf.cookie_name, "csrf");
    assert_eq!(policy.csrf.header_name, "x-csrf-token");
    assert_eq!(policy.net_public.max_redirects, 0);
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
    assert_eq!(
        policy.cors.allowed_methods,
        vec!["GET".to_string(), "POST".to_string()]
    );
    assert_eq!(
        policy.cors.allowed_headers,
        vec!["content-type".to_string()]
    );
    assert!(policy.cors.exposed_headers.is_empty());
    assert_eq!(policy.cors.max_age_seconds, 600);
    assert!(!policy.cors.allow_credentials);
    assert!(!policy.cors.allow_private_network);
    assert_eq!(policy.json.max_bytes, 1_048_576);
    assert_eq!(policy.json.max_depth, 64);
    assert_eq!(policy.http.max_body_bytes, 1_048_576);
    assert_eq!(policy.http.max_concurrency, 1024);
    assert_eq!(policy.http.max_keep_alive_requests, 256);
    assert_eq!(policy.http.max_header_bytes, 8_191);
    assert_eq!(policy.http.max_multipart_bytes, 4_096);
    assert_eq!(policy.http.default_timeout_ms, 15000);
    assert!(!policy.security_headers.hsts_enabled);
    assert_eq!(policy.security_headers.hsts_max_age_seconds, 0);
    assert!(!policy.security_headers.hsts_include_subdomains);
    assert!(!policy.security_headers.hsts_preload);
    assert!(policy.security_headers.csp_enabled);
    assert!(policy.security_headers.csp_report_only);
    assert_eq!(
        policy.security_headers.csp_policy,
        "default-src 'self'; frame-ancestors 'none'; base-uri 'self'"
    );
    assert_eq!(policy.csrf.cookie_name, "csrf");
    assert_eq!(policy.csrf.header_name, "x-csrf-token");
    assert_eq!(policy.net_public.max_redirects, 5);
    assert!(policy.net_internal.enabled);
    assert!(policy.fs.enabled);
    assert_eq!(policy.replay.effects, "allow");
}
