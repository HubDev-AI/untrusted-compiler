use ailang_core::{
    build_security_map, build_security_map_with_allows, parse_allow_annotations, parse_source,
    policy::parse_policy_str, render_security_audit_text, run_security_audit,
    run_security_audit_with_baseline, security_map::SourceLocation, should_fail, AuditSeverity,
    Policy, SecurityAllow,
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
fn sec_audit_flags_incomplete_csrf_protected_methods_with_sample_calls() {
    let source = r#"
fn boot() -> Int {
  withCsrf();
  withAuth();
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let policy_source = r#"
[policy]
mode = "warn"
env = "prod"

[csrf]
enabled = true
protected_methods = ["POST"]
"#;

    let policy = parse_policy_str(Path::new("ailang.policy"), policy_source)
        .expect("policy should parse for csrf protected-method coverage test");
    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.id == "CSRF_PROTECTED_METHODS_INCOMPLETE")
        .expect("CSRF_PROTECTED_METHODS_INCOMPLETE should be present");
    assert_eq!(finding.severity, AuditSeverity::LOW);
    let missing = finding
        .evidence
        .get("missingMethods")
        .and_then(|value| value.as_array())
        .expect("csrf protected-method finding should include missingMethods");
    assert!(missing.iter().any(|value| value.as_str() == Some("PUT")));
    assert!(missing.iter().any(|value| value.as_str() == Some("PATCH")));
    assert!(missing.iter().any(|value| value.as_str() == Some("DELETE")));
    let samples = finding
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("csrf protected-method finding should include sampleCalls");
    assert!(samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| matches!(callee, "withCsrf" | "csrf.withCsrf"))
    }));
}

#[test]
fn sec_audit_flags_cors_origin_reflection_with_sample_calls() {
    let source = r#"
fn boot() -> Int {
  withCors();
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mut policy = Policy::default();
    policy.cors.reflect_origin = true;
    policy.cors.forbid_reflect_origin = false;

    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    let reflect = report
        .findings
        .iter()
        .find(|finding| finding.id == "CORS_REFLECT_ORIGIN_ENABLED")
        .expect("CORS_REFLECT_ORIGIN_ENABLED finding should be present");
    assert_eq!(reflect.severity, AuditSeverity::HIGH);
    let samples = reflect
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("reflect-origin finding should include sampleCalls");
    assert!(samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| matches!(callee, "withCors" | "cors.withCors"))
    }));
}

#[test]
fn sec_audit_flags_weak_referrer_policy_with_sample_calls() {
    let source = r#"
fn boot() -> Int {
  withSecurityHeaders();
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mut policy = Policy::default();
    policy.security_headers.referrer_policy = "unsafe-url".to_string();

    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.id == "REFERRER_POLICY_WEAK")
        .expect("REFERRER_POLICY_WEAK should be present");
    assert_eq!(finding.severity, AuditSeverity::LOW);
    let samples = finding
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("REFERRER_POLICY_WEAK should include sampleCalls");
    assert!(samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| {
                matches!(callee, "withSecurityHeaders" | "sec.withSecurityHeaders")
            })
    }));
}

#[test]
fn sec_audit_flags_weak_xfo_and_nosniff_with_sample_calls() {
    let source = r#"
fn boot() -> Int {
  withSecurityHeaders();
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mut policy = Policy::default();
    policy.security_headers.x_frame_options = "SAMEORIGIN".to_string();
    policy.security_headers.x_content_type_options = false;

    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    let xfo = report
        .findings
        .iter()
        .find(|finding| finding.id == "XFO_DISABLED")
        .expect("XFO_DISABLED should be present");
    assert_eq!(xfo.severity, AuditSeverity::LOW);
    let xfo_samples = xfo
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("XFO_DISABLED should include sampleCalls");
    assert!(xfo_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| {
                matches!(callee, "withSecurityHeaders" | "sec.withSecurityHeaders")
            })
    }));

    let nosniff = report
        .findings
        .iter()
        .find(|finding| finding.id == "NOSNIFF_DISABLED")
        .expect("NOSNIFF_DISABLED should be present");
    assert_eq!(nosniff.severity, AuditSeverity::LOW);
    let nosniff_samples = nosniff
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("NOSNIFF_DISABLED should include sampleCalls");
    assert!(nosniff_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| {
                matches!(callee, "withSecurityHeaders" | "sec.withSecurityHeaders")
            })
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
    let replay = report
        .findings
        .iter()
        .find(|finding| finding.id == "REPLAY_EFFECTS_ALLOW")
        .expect("replay finding should be present");
    assert_eq!(replay.severity, AuditSeverity::MEDIUM);
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
fn sec_audit_flags_weak_symlink_policy_with_sample_calls() {
    let source = r#"
fn risky() -> Int {
  fs.write(FsCap(), "/tmp/a", "payload");
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let weak_policy = parse_policy_str(
        Path::new("ailang.policy"),
        r#"
[policy]
mode = "warn"
env = "dev"

[fs]
enabled = true
allowed_base_paths = ["/srv/data"]
forbid_symlinks = "warn"
"#,
    )
    .expect("weak symlink policy should parse");
    let weak_report = run_security_audit(&weak_policy, &build_security_map(&program, &weak_policy));

    let weak = weak_report
        .findings
        .iter()
        .find(|finding| finding.id == "SYMLINK_POLICY_WEAK")
        .expect("SYMLINK_POLICY_WEAK should be present");
    assert_eq!(weak.severity, AuditSeverity::MEDIUM);
    let weak_samples = weak
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("SYMLINK_POLICY_WEAK should include sampleCalls");
    assert!(weak_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "fs.write")
    }));

    let enforce_policy = parse_policy_str(
        Path::new("ailang.policy"),
        r#"
[policy]
mode = "warn"
env = "dev"

[fs]
enabled = true
allowed_base_paths = ["/srv/data"]
forbid_symlinks = "enforce"
"#,
    )
    .expect("enforced symlink policy should parse");
    let enforce_report = run_security_audit(
        &enforce_policy,
        &build_security_map(&program, &enforce_policy),
    );
    assert!(
        enforce_report
            .findings
            .iter()
            .all(|finding| finding.id != "SYMLINK_POLICY_WEAK"),
        "SYMLINK_POLICY_WEAK should not be emitted when fs.forbid_symlinks=\"enforce\""
    );
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
resolve_dns = false

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

    let dns = report
        .findings
        .iter()
        .find(|finding| finding.id == "DNS_RESOLUTION_DISABLED")
        .expect("dns resolution finding should be present");
    let dns_samples = dns
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("dns resolution finding should include sampleCalls");
    assert!(dns_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "httpClient.get")
    }));

    let egress = report
        .findings
        .iter()
        .find(|finding| finding.id == "PUBLIC_EGRESS_NO_DOMAIN_POLICY")
        .expect("public egress domain finding should be present");
    assert_eq!(egress.severity, AuditSeverity::MEDIUM);
    let egress_samples = egress
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("public egress domain finding should include sampleCalls");
    assert!(egress_samples.iter().any(|sample| {
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
    assert_eq!(replay.severity, AuditSeverity::HIGH);
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
fn sec_audit_skips_public_egress_domain_finding_when_policy_is_configured() {
    let source = r#"
fn safe(net: NetCap) effects { net } -> Int {
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
allow_redirects = false
allowed_domains = ["api.example.com"]
"#;

    let policy = parse_policy_str(Path::new("ailang.policy"), policy_source)
        .expect("policy should parse for public egress domain policy test");
    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.id != "PUBLIC_EGRESS_NO_DOMAIN_POLICY"),
        "PUBLIC_EGRESS_NO_DOMAIN_POLICY should not be emitted when domain policy is configured"
    );
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
    let expiry_rollup = report
        .findings
        .iter()
        .find(|finding| finding.id == "ALLOW_EXPIRY_WINDOW_ROLLUP")
        .expect("ALLOW_EXPIRY_WINDOW_ROLLUP should be present");
    assert_eq!(expiry_rollup.severity, AuditSeverity::HIGH);
    assert_eq!(
        expiry_rollup
            .evidence
            .get("expiredCount")
            .and_then(|value| value.as_u64()),
        Some(1)
    );
    assert!(
        expiry_rollup
            .evidence
            .get("sampleExceptions")
            .and_then(|value| value.as_array())
            .is_some_and(|samples| !samples.is_empty()),
        "rollup should include sampled exceptions"
    );
    assert!(
        expiry_rollup
            .evidence
            .get("minDaysUntilExpiry")
            .and_then(|value| value.as_i64())
            .is_some(),
        "rollup should expose minDaysUntilExpiry"
    );
    assert!(
        expiry_rollup
            .evidence
            .get("medianDaysUntilExpiry")
            .and_then(|value| value.as_i64())
            .is_some(),
        "rollup should expose medianDaysUntilExpiry"
    );
    assert!(
        expiry_rollup
            .evidence
            .get("maxDaysUntilExpiry")
            .and_then(|value| value.as_i64())
            .is_some(),
        "rollup should expose maxDaysUntilExpiry"
    );
    assert_eq!(
        expiry_rollup
            .evidence
            .get("severityInputs")
            .and_then(|value| value.get("expiringSoonHighThreshold"))
            .and_then(|value| value.as_u64()),
        Some(5)
    );
    let allow_count = report
        .findings
        .iter()
        .find(|finding| finding.id == "ALLOW_COUNT_HIGH")
        .expect("ALLOW_COUNT_HIGH should be present");
    assert_eq!(allow_count.severity, AuditSeverity::LOW);
    let samples = allow_count
        .evidence
        .get("sampleExceptions")
        .and_then(|value| value.as_array())
        .expect("ALLOW_COUNT_HIGH should include sampleExceptions");
    assert!(!samples.is_empty(), "expected non-empty exception samples");
    assert!(
        samples.len() <= 5,
        "sampleExceptions should be bounded to at most 5 entries"
    );
    assert!(samples.iter().any(|sample| {
        sample
            .get("ticket")
            .and_then(|value| value.as_str())
            .is_some_and(|ticket| ticket == "SEC-000")
    }));
}

#[test]
fn sec_audit_computes_trend_against_baseline_report() {
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
    let baseline_policy = Policy::default();
    let baseline_map = build_security_map(&program, &baseline_policy);
    let baseline_report = run_security_audit(&baseline_policy, &baseline_map);
    assert_eq!(baseline_report.summary.risk_score, 0);

    let mut current_policy = baseline_policy.clone();
    current_policy.cors.allowed_origins = vec!["*".to_string()];
    current_policy.cors.allow_credentials = false;
    current_policy.cors.forbid_any_origin = false;
    let current_map = build_security_map(&program, &current_policy);
    let report =
        run_security_audit_with_baseline(&current_policy, &current_map, Some(&baseline_report));

    let trend = report.trend.expect("trend data should be present");
    assert_eq!(trend.baseline_policy_hash, baseline_report.policy.hash);
    assert_eq!(trend.baseline_risk_score, 0);
    assert!(trend.risk_score_delta > 0);
    assert!(trend.finding_count_delta > 0);
    assert!(trend
        .added_finding_ids
        .iter()
        .any(|id| id == "CORS_ANY_ORIGIN"));
    assert_eq!(trend.resolved_finding_ids.len(), 0);
    assert!(trend
        .severity_deltas
        .get("MEDIUM")
        .is_some_and(|delta| *delta > 0));
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

    let user_agent = report
        .findings
        .iter()
        .find(|finding| finding.id == "LOG_USER_AGENT_ENABLED")
        .expect("user agent finding should be present");
    assert_eq!(user_agent.severity, AuditSeverity::LOW);
    let user_agent_samples = user_agent
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("user agent finding should include sampleCalls");
    assert!(user_agent_samples.iter().any(|sample| {
        sample
            .get("callee")
            .and_then(|value| value.as_str())
            .is_some_and(|callee| callee == "log.info")
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

#[test]
fn sec_audit_sample_calls_include_origin_trace_chains() {
    let source = r#"
fn queryParam() -> String {
  req.query("q")
}

fn passThrough(input: String) -> String {
  input
}

fn wrap() -> String {
  passThrough(queryParam())
}

fn boot() -> Int {
  let raw = wrap();
  db.exec(DbCap(), raw);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let policy_source = r#"
[policy]
mode = "warn"
env = "dev"

[sql]
forbid_raw = false
"#;

    let policy = parse_policy_str(Path::new("ailang.policy"), policy_source)
        .expect("policy should parse for trace-chain sample test");
    let map = build_security_map(&program, &policy);
    let report = run_security_audit(&policy, &map);

    let sql_raw = report
        .findings
        .iter()
        .find(|finding| finding.id == "SQL_RAW_ALLOWED_BY_POLICY")
        .expect("sql raw finding should be present");
    let samples = sql_raw
        .evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
        .expect("sql raw finding should include sampleCalls");
    let traced = samples.iter().any(|sample| {
        sample
            .get("originEdges")
            .and_then(|value| value.as_array())
            .is_some_and(|edges| {
                edges.iter().any(|edge| {
                    let trace = edge.get("trace").and_then(|value| value.as_array());
                    trace.is_some_and(|trace| {
                        let steps = trace
                            .iter()
                            .filter_map(|value| value.as_str())
                            .collect::<Vec<_>>();
                        steps.contains(&"call:req.query")
                            && steps.contains(&"call:queryParam")
                            && steps.contains(&"call:passThrough")
                            && steps.contains(&"call:wrap")
                    })
                })
            })
    });
    assert!(traced, "expected SQL sample call with origin trace chain");
}

#[test]
fn sec_audit_text_renderer_includes_sample_trace_previews() {
    let source = r#"
fn queryParam() -> String {
  req.query("q")
}

fn wrap() -> String {
  queryParam()
}

fn boot() -> Int {
  let raw = wrap();
  db.exec(DbCap(), raw);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let policy = parse_policy_str(
        Path::new("ailang.policy"),
        r#"
[policy]
mode = "warn"
env = "dev"

[sql]
forbid_raw = false
"#,
    )
    .expect("policy should parse");

    let report = run_security_audit(&policy, &build_security_map(&program, &policy));
    let text = render_security_audit_text(&report);
    assert!(
        text.contains("SQL_RAW_ALLOWED_BY_POLICY"),
        "expected SQL_RAW_ALLOWED_BY_POLICY in text report"
    );
    assert!(text.contains("sample: db.exec@"), "expected sample preview");
    assert!(
        text.contains("trace=call:req.query"),
        "expected origin trace preview in text report"
    );
}
