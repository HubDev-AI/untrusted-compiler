use ailang_core::{
    build_security_map, build_security_map_with_allows, parse_allow_annotations, parse_source,
    policy::parse_policy_str, run_security_audit, security_map::SourceLocation, should_fail,
    AuditSeverity, Policy, SecurityAllow,
};
use std::path::Path;

#[test]
fn sec_audit_default_policy_can_be_clean_with_router_security_middleware() {
    let source = r#"
fn boot() -> Int {
  withSecurityHeaders();
  withCors();
  withCsrf();
  withAuth();
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let policy = Policy::default();
    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    assert!(
        report.findings.is_empty(),
        "expected clean report, got: {:?}",
        report.findings
    );
    assert_eq!(report.summary.risk_score, 0);
    assert_eq!(report.summary.highest_severity, AuditSeverity::LOW);
}

#[test]
fn sec_audit_includes_sample_calls_for_cors_headers_and_csrf_auth_findings() {
    let source = r#"
fn boot() -> Int {
  withSecurityHeaders();
  withCors();
  withCsrf();
  withAuth();
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mut policy = Policy::default();
    policy.cors.allowed_origins = vec!["*".to_string()];
    policy.cors.allow_credentials = false;
    policy.cors.forbid_any_origin = false;
    policy.security_headers.csp_enabled = false;
    policy.auth.mode = "cookie".to_string();
    policy.auth.cross_site_frontend = true;
    policy.csrf.enabled = false;

    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    let cors_any = report
        .findings
        .iter()
        .find(|finding| finding.id == "CORS_ANY_ORIGIN")
        .expect("CORS_ANY_ORIGIN finding should be present");
    let cors_samples = cors_any
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("CORS_ANY_ORIGIN should include sampleCalls");
    assert!(cors_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| matches!(callee, "withCors" | "cors.withCors"))
    }));

    let csp_disabled = report
        .findings
        .iter()
        .find(|finding| finding.id == "CSP_DISABLED")
        .expect("CSP_DISABLED finding should be present");
    let csp_samples = csp_disabled
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("CSP_DISABLED should include sampleCalls");
    assert!(csp_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| {
                matches!(callee, "withSecurityHeaders" | "sec.withSecurityHeaders")
            })
    }));

    let csrf_missing = report
        .findings
        .iter()
        .find(|finding| finding.id == "CSRF_REQUIRED_BUT_DISABLED")
        .expect("CSRF_REQUIRED_BUT_DISABLED finding should be present");
    let csrf_samples = csrf_missing
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("CSRF_REQUIRED_BUT_DISABLED should include sampleCalls");
    assert!(csrf_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| matches!(callee, "withAuth" | "auth.withAuth"))
    }));
}

#[test]
fn sec_audit_flags_critical_internal_net_and_secret_reveal_usage() {
    let source = r#"
fn risky() -> Int {
  secret_reveal(SecretsCap());
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let policy_source = r#"
[policy]
mode = "warn"
env = "dev"

[cors]
allowed_origins = ["*"]
allow_credentials = false
forbid_any_origin = false

[net.internal]
enabled = true
allowed_cidrs = []
allowed_domains = []

[capture]
mode = "all"
redact_headers = ["authorization"]

[replay]
effects = "allow"
"#;

    let policy = parse_policy_str(Path::new("ailang.policy"), policy_source)
        .expect("policy should parse for audit test");
    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    assert!(report
        .findings
        .iter()
        .any(|finding| finding.id == "INTERNAL_NET_ENABLED_NO_ALLOWLIST"
            && finding.severity == AuditSeverity::CRITICAL));
    let reveal = report
        .findings
        .iter()
        .find(|finding| finding.id == "SECRETS_REVEAL_USED")
        .expect("secrets reveal finding should be present");
    assert_eq!(reveal.severity, AuditSeverity::CRITICAL);
    let reveal_samples = reveal
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("secrets reveal finding should include sampleCalls evidence");
    assert!(reveal_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "secret_reveal")
    }));
    assert!(should_fail(&report, AuditSeverity::HIGH));
}

#[test]
fn sec_audit_includes_sample_calls_for_internal_net_and_fs_findings() {
    let source = r#"
fn risky() -> Int {
  httpClient.getInternal(InternalNetCap(), "http://internal.local");
  fs.write(FsCap(), "/tmp/a", "payload");
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let policy_source = r#"
[policy]
mode = "warn"
env = "dev"

[net.internal]
enabled = true
allowed_cidrs = []
allowed_domains = []

[fs]
enabled = true
allowed_base_paths = []
"#;

    let policy = parse_policy_str(Path::new("ailang.policy"), policy_source)
        .expect("policy should parse for internal net/fs sample evidence test");
    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    let internal = report
        .findings
        .iter()
        .find(|finding| finding.id == "INTERNAL_NET_ENABLED_NO_ALLOWLIST")
        .expect("internal net finding should be present");
    let internal_samples = internal
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("internal net finding should include sampleCalls");
    assert!(internal_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "httpClient.getInternal")
    }));

    let fs = report
        .findings
        .iter()
        .find(|finding| finding.id == "FS_ENABLED_NO_BASE_ALLOWLIST")
        .expect("filesystem finding should be present");
    let fs_samples = fs
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("filesystem finding should include sampleCalls");
    assert!(fs_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "fs.write")
    }));
}

#[test]
fn sec_audit_includes_sample_calls_for_redirect_capture_and_replay_findings() {
    let source = r#"
fn risky(net: NetCap) effects { net } -> Int {
  req.header("authorization");
  httpClient.get(net, "https://example.com");
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let policy_source = r#"
[policy]
mode = "warn"
env = "prod"

[net.public]
allow_redirects = true

[net.ssrf]
revalidate_redirects = false

[capture]
mode = "all"
redact_headers = ["authorization"]

[replay]
effects = "allow"
"#;

    let policy = parse_policy_str(Path::new("ailang.policy"), policy_source)
        .expect("policy should parse for redirect/capture/replay sample evidence test");
    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    let redirects = report
        .findings
        .iter()
        .find(|finding| finding.id == "PUBLIC_REDIRECTS_ENABLED_WITHOUT_REVALIDATION")
        .expect("redirect finding should be present");
    let redirect_samples = redirects
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("redirect finding should include sampleCalls");
    assert!(redirect_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "httpClient.get")
    }));

    let replay = report
        .findings
        .iter()
        .find(|finding| finding.id == "REPLAY_EFFECTS_ALLOW")
        .expect("replay finding should be present");
    let replay_samples = replay
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("replay finding should include sampleCalls");
    assert!(replay_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "httpClient.get")
    }));

    let capture_redaction = report
        .findings
        .iter()
        .find(|finding| finding.id == "CAPTURE_REDACTION_INCOMPLETE")
        .expect("capture redaction finding should be present");
    let capture_redaction_samples = capture_redaction
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("capture redaction finding should include sampleCalls");
    assert!(capture_redaction_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "req.header")
    }));

    let capture_all = report
        .findings
        .iter()
        .find(|finding| finding.id == "CAPTURE_ALL_IN_PROD")
        .expect("capture-all finding should be present");
    let capture_all_samples = capture_all
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("capture-all finding should include sampleCalls");
    assert!(capture_all_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "req.header")
    }));
}

#[test]
fn sec_audit_reports_allow_annotations_as_exceptions() {
    let source = r#"
@allow(
  policy = "net.internal.enabled",
  bypass = ["sink.net.internal_request"],
  reason = "Calls internal inventory service",
  ticket = "SEC-123",
  expires = "2099-06-01",
)
fn boot() -> Int {
  withCors();
  withSecurityHeaders();
  withCsrf();
  withAuth();
  1
}
"#;

    let stripped = ailang_core::strip_allow_annotations(source);
    let program = parse_source(Path::new("main.ai"), &stripped).expect("source should parse");
    let allows = parse_allow_annotations(Path::new("main.ai"), source).expect("allow should parse");
    let policy = Policy::default();
    let map = build_security_map_with_allows(&program, &policy, allows);
    let report = run_security_audit(&policy, &map);

    assert_eq!(report.exceptions.len(), 1);
    assert_eq!(report.exceptions[0].policy_key, "net.internal.enabled");
    assert_eq!(report.exceptions[0].ticket, "SEC-123");
    assert_eq!(report.exceptions[0].severity, AuditSeverity::HIGH);
    assert!(report.findings.iter().any(|finding| {
        finding.id == "INTERNAL_NET_CALL_ALLOWLISTED" && finding.severity == AuditSeverity::HIGH
    }));
}

#[test]
fn sec_audit_flags_secret_reveal_allowlisted_bypass() {
    let source = r#"
@allow(
  policy = "effects.forbid",
  bypass = ["effect.secrets.reveal"],
  reason = "dev debug helper",
  ticket = "SEC-200",
  expires = "2099-06-01",
)
fn boot() -> Int {
  secret_reveal(SecretsCap());
  1
}
"#;

    let stripped = ailang_core::strip_allow_annotations(source);
    let program = parse_source(Path::new("main.ai"), &stripped).expect("source should parse");
    let allows = parse_allow_annotations(Path::new("main.ai"), source).expect("allow should parse");
    let policy = Policy::default();
    let map = build_security_map_with_allows(&program, &policy, allows);
    let report = run_security_audit(&policy, &map);

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.id == "SECRETS_REVEAL_ALLOWLISTED")
        .expect("SECRETS_REVEAL_ALLOWLISTED should be present");
    assert_eq!(finding.severity, AuditSeverity::HIGH);
    let samples = finding
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("allowlisted reveal finding should include sampleCalls");
    assert!(samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| matches!(callee, "secret_reveal" | "secrets.reveal"))
    }));
}

#[test]
fn sec_audit_includes_sample_calls_for_internal_net_allowlisted_bypass() {
    let source = r#"
@allow(
  policy = "net.internal.enabled",
  bypass = ["sink.net.internal_request"],
  reason = "internal inventory call",
  ticket = "SEC-301",
  expires = "2099-06-01",
)
fn boot() -> Int {
  httpClient.getInternal(InternalNetCap(), "http://internal.local");
  1
}
"#;

    let stripped = ailang_core::strip_allow_annotations(source);
    let program = parse_source(Path::new("main.ai"), &stripped).expect("source should parse");
    let allows = parse_allow_annotations(Path::new("main.ai"), source).expect("allow should parse");
    let policy = Policy::default();
    let map = build_security_map_with_allows(&program, &policy, allows);
    let report = run_security_audit(&policy, &map);

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.id == "INTERNAL_NET_CALL_ALLOWLISTED")
        .expect("INTERNAL_NET_CALL_ALLOWLISTED should be present");
    let samples = finding
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("allowlisted internal-net finding should include sampleCalls");
    assert!(samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| matches!(callee, "httpClient.getInternal" | "net_internal_call"))
    }));
}

#[test]
fn sec_audit_flags_allow_hygiene_findings() {
    let source = "fn boot() -> Int { 1 }";
    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let policy = Policy::default();

    let mut allows = Vec::new();
    for index in 0..11 {
        allows.push(SecurityAllow {
            loc: SourceLocation {
                file: "main.ai".to_string(),
                line: index + 1,
                column: 1,
            },
            policy: "effects.forbid".to_string(),
            bypass: vec!["sink.net.public_request".to_string()],
            reason: "temporary exception".to_string(),
            ticket: format!("SEC-{index:03}"),
            expires: if index == 0 {
                "2000-01-01".to_string()
            } else {
                "2099-01-01".to_string()
            },
        });
    }

    let map = build_security_map_with_allows(&program, &policy, allows);
    let report = run_security_audit(&policy, &map);

    assert!(report
        .findings
        .iter()
        .any(|finding| finding.id == "ALLOW_EXPIRED" && finding.severity == AuditSeverity::HIGH));
    assert!(report.findings.iter().any(|finding| {
        finding.id == "ALLOW_COUNT_HIGH" && finding.severity == AuditSeverity::LOW
    }));
}

#[test]
fn sec_audit_flags_logging_and_sql_policy_weakening() {
    let source = r#"
fn boot() -> Int {
  withSecurityHeaders();
  withCors();
  withCsrf();
  withAuth();
  log.info("event");
  db.exec(DbCap(), "SELECT id FROM users");
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let policy_source = r#"
[policy]
mode = "warn"
env = "dev"

[logging]
structured_only = false
include_remote_ip = true
include_user_agent = true

[sql]
forbid_raw = false
require_limit_on_select = "off"
"#;

    let policy = parse_policy_str(Path::new("ailang.policy"), policy_source)
        .expect("policy should parse for logging/sql audit test");
    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    let log_structured = report
        .findings
        .iter()
        .find(|finding| finding.id == "LOG_STRUCTURED_ONLY_DISABLED")
        .expect("structured logging finding should be present");
    assert_eq!(log_structured.severity, AuditSeverity::HIGH);
    let log_samples = log_structured
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("structured logging finding should include sampleCalls");
    assert!(log_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "log.info")
    }));

    let remote_ip = report
        .findings
        .iter()
        .find(|finding| finding.id == "LOG_REMOTE_IP_ENABLED")
        .expect("remote ip finding should be present");
    assert_eq!(remote_ip.severity, AuditSeverity::MEDIUM);
    let remote_samples = remote_ip
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("remote ip finding should include sampleCalls");
    assert!(remote_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "log.info")
    }));

    assert!(report.findings.iter().any(|finding| {
        finding.id == "LOG_USER_AGENT_ENABLED" && finding.severity == AuditSeverity::LOW
    }));
    let sql_raw = report
        .findings
        .iter()
        .find(|finding| finding.id == "SQL_RAW_ALLOWED_BY_POLICY")
        .expect("sql raw policy finding should be present");
    assert_eq!(sql_raw.severity, AuditSeverity::HIGH);
    let sql_raw_samples = sql_raw
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("sql raw finding should include sampleCalls");
    assert!(sql_raw_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "db.exec")
    }));

    let sql_limit = report
        .findings
        .iter()
        .find(|finding| finding.id == "SQL_LIMIT_RULE_DISABLED")
        .expect("sql limit finding should be present");
    assert_eq!(sql_limit.severity, AuditSeverity::MEDIUM);
    let sql_limit_samples = sql_limit
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("sql limit finding should include sampleCalls");
    assert!(sql_limit_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "db.exec")
    }));
}

#[test]
fn sec_audit_flags_select_without_limit_with_policy_severity_mapping() {
    let source = r#"
fn bad() -> Int {
  db.exec(DbCap(), "SELECT id FROM users");
  db.exec(DbCap(), "SELECT id FROM users LIMIT 1");
  1
}
"#;
    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");

    let warn_policy = parse_policy_str(
        Path::new("ailang.policy"),
        r#"
[policy]
mode = "warn"
env = "dev"

[sql]
require_limit_on_select = "warn"
"#,
    )
    .expect("warn policy should parse");
    let warn_report = run_security_audit(&warn_policy, &build_security_map(&program, &warn_policy));
    let warn_finding = warn_report
        .findings
        .iter()
        .find(|finding| finding.id == "SQL_SELECT_WITHOUT_LIMIT")
        .expect("warn report should include sql select without limit finding");
    assert_eq!(warn_finding.severity, AuditSeverity::MEDIUM);
    let warn_samples = warn_finding
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("sql select finding should include sampleCalls evidence");
    assert!(warn_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "db.exec")
    }));

    let enforce_policy = parse_policy_str(
        Path::new("ailang.policy"),
        r#"
[policy]
mode = "warn"
env = "dev"

[sql]
require_limit_on_select = "enforce"
"#,
    )
    .expect("enforce policy should parse");
    let enforce_report = run_security_audit(
        &enforce_policy,
        &build_security_map(&program, &enforce_policy),
    );
    let enforce_finding = enforce_report
        .findings
        .iter()
        .find(|finding| finding.id == "SQL_SELECT_WITHOUT_LIMIT")
        .expect("enforce report should include sql select without limit finding");
    assert_eq!(enforce_finding.severity, AuditSeverity::HIGH);
    let enforce_samples = enforce_finding
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("sql select finding should include sampleCalls evidence");
    assert!(enforce_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "db.exec")
    }));
}
