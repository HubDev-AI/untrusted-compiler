use ailang_core::{
    build_security_map, build_security_map_with_allows, parse_allow_annotations, parse_source,
    policy::parse_policy_str, run_security_audit, should_fail, AuditSeverity, Policy,
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
}
