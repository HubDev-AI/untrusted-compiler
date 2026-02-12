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
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.id == "SECRETS_REVEAL_USED"
            && finding.severity == AuditSeverity::CRITICAL));
    assert!(should_fail(&report, AuditSeverity::HIGH));
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
fn boot() -> Int { 1 }
"#;

    let stripped = ailang_core::strip_allow_annotations(source);
    let program = parse_source(Path::new("main.ai"), &stripped).expect("source should parse");
    let allows = parse_allow_annotations(Path::new("main.ai"), source).expect("allow should parse");
    let policy = Policy::default();
    let map = build_security_map_with_allows(&program, &policy, allows);
    let report = run_security_audit(&policy, &map);

    assert!(report.findings.iter().any(|finding| {
        finding.id == "SECRETS_REVEAL_ALLOWLISTED" && finding.severity == AuditSeverity::HIGH
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

    assert!(report.findings.iter().any(|finding| {
        finding.id == "LOG_STRUCTURED_ONLY_DISABLED" && finding.severity == AuditSeverity::HIGH
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.id == "LOG_REMOTE_IP_ENABLED" && finding.severity == AuditSeverity::MEDIUM
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.id == "LOG_USER_AGENT_ENABLED" && finding.severity == AuditSeverity::LOW
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.id == "SQL_RAW_ALLOWED_BY_POLICY" && finding.severity == AuditSeverity::HIGH
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.id == "SQL_LIMIT_RULE_DISABLED" && finding.severity == AuditSeverity::MEDIUM
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
    assert!(warn_report.findings.iter().any(|finding| {
        finding.id == "SQL_SELECT_WITHOUT_LIMIT" && finding.severity == AuditSeverity::MEDIUM
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
    assert!(enforce_report.findings.iter().any(|finding| {
        finding.id == "SQL_SELECT_WITHOUT_LIMIT" && finding.severity == AuditSeverity::HIGH
    }));
}
