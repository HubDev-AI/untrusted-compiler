use crate::policy::Policy;
use crate::security_map::{SecurityAllow, SecurityMap, TagAttr};
use serde::Serialize;
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum AuditSeverity {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL,
}

impl AuditSeverity {
    pub fn parse_threshold(input: &str) -> Option<Self> {
        let normalized = input.trim().to_uppercase();
        let value = if let Some(rest) = normalized.strip_prefix("RISK>=") {
            rest.trim()
        } else {
            normalized.as_str()
        };

        match value {
            "LOW" => Some(Self::LOW),
            "MEDIUM" => Some(Self::MEDIUM),
            "HIGH" => Some(Self::HIGH),
            "CRITICAL" => Some(Self::CRITICAL),
            _ => None,
        }
    }

    fn weight(self) -> i64 {
        match self {
            Self::LOW => 1,
            Self::MEDIUM => 3,
            Self::HIGH => 7,
            Self::CRITICAL => 15,
        }
    }

    fn rank(self) -> u8 {
        match self {
            Self::LOW => 0,
            Self::MEDIUM => 1,
            Self::HIGH => 2,
            Self::CRITICAL => 3,
        }
    }
}

impl Ord for AuditSeverity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for AuditSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditPolicySummary {
    pub name: String,
    pub version: String,
    pub hash: String,
    pub mode: String,
    pub env: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditBuildSummary {
    #[serde(rename = "compilerHash")]
    pub compiler_hash: String,
    #[serde(rename = "runtimeHash")]
    pub runtime_hash: String,
    #[serde(rename = "timeMs")]
    pub time_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditCorsPosture {
    pub enabled: bool,
    #[serde(rename = "allowCredentials")]
    pub allow_credentials: bool,
    pub wildcard: bool,
    pub origins: Vec<String>,
    #[serde(rename = "allowRedirects")]
    pub allow_redirects: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditCspPosture {
    pub enabled: bool,
    #[serde(rename = "reportOnly")]
    pub report_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditSecurityHeadersPosture {
    pub enabled: bool,
    pub hsts: bool,
    pub csp: AuditCspPosture,
    #[serde(rename = "xFrameOptions")]
    pub x_frame_options: String,
    pub nosniff: bool,
    #[serde(rename = "referrerPolicy")]
    pub referrer_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditCsrfPosture {
    pub enabled: bool,
    pub mode: String,
    #[serde(rename = "sameSite")]
    pub same_site: String,
    #[serde(rename = "secureCookie")]
    pub secure_cookie: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditCapturePosture {
    pub mode: String,
    #[serde(rename = "redactHeaders")]
    pub redact_headers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditReplayPosture {
    pub effects: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditPosture {
    pub cors: AuditCorsPosture,
    #[serde(rename = "securityHeaders")]
    pub security_headers: AuditSecurityHeadersPosture,
    pub csrf: AuditCsrfPosture,
    pub capture: AuditCapturePosture,
    pub replay: AuditReplayPosture,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditAttackSurface {
    #[serde(rename = "sqlRaw")]
    pub sql_raw: bool,
    #[serde(rename = "htmlRaw")]
    pub html_raw: bool,
    #[serde(rename = "secretsReveal")]
    pub secrets_reveal: bool,
    #[serde(rename = "internalNet")]
    pub internal_net: bool,
    #[serde(rename = "fsEnabled")]
    pub fs_enabled: bool,
    #[serde(rename = "publicRedirects")]
    pub public_redirects: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditException {
    #[serde(rename = "policyKey")]
    pub policy_key: String,
    pub location: AuditLocation,
    pub reason: String,
    pub ticket: String,
    pub expires: String,
    pub severity: AuditSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuditFinding {
    pub id: String,
    pub severity: AuditSeverity,
    pub category: String,
    pub evidence: Value,
    pub suggestion: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuditSummary {
    #[serde(rename = "riskScore")]
    pub risk_score: i64,
    #[serde(rename = "highestSeverity")]
    pub highest_severity: AuditSeverity,
    #[serde(rename = "findingCounts")]
    pub finding_counts: HashMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuditReport {
    pub version: String,
    pub policy: AuditPolicySummary,
    pub build: AuditBuildSummary,
    pub posture: AuditPosture,
    #[serde(rename = "attackSurface")]
    pub attack_surface: AuditAttackSurface,
    pub exceptions: Vec<AuditException>,
    pub findings: Vec<AuditFinding>,
    pub summary: AuditSummary,
}

pub fn run_security_audit(policy: &Policy, security_map: &SecurityMap) -> AuditReport {
    let posture = AuditPosture {
        cors: AuditCorsPosture {
            enabled: policy.cors.enabled,
            allow_credentials: policy.cors.allow_credentials,
            wildcard: policy.cors.has_wildcard_origin(),
            origins: policy.cors.allowed_origins.clone(),
            allow_redirects: policy.net_public.allow_redirects,
        },
        security_headers: AuditSecurityHeadersPosture {
            enabled: policy.security_headers.enabled,
            hsts: policy.security_headers.hsts_enabled,
            csp: AuditCspPosture {
                enabled: policy.security_headers.csp_enabled,
                report_only: policy.security_headers.csp_report_only,
            },
            x_frame_options: policy.security_headers.x_frame_options.clone(),
            nosniff: policy.security_headers.x_content_type_options,
            referrer_policy: policy.security_headers.referrer_policy.clone(),
        },
        csrf: AuditCsrfPosture {
            enabled: policy.csrf.enabled,
            mode: policy.csrf.mode.clone(),
            same_site: policy.csrf.same_site.clone(),
            secure_cookie: policy.csrf.secure_cookie,
        },
        capture: AuditCapturePosture {
            mode: policy.capture.mode.clone(),
            redact_headers: policy.capture.redact_headers.clone(),
        },
        replay: AuditReplayPosture {
            effects: policy.replay.effects.clone(),
        },
    };

    let attack_surface = AuditAttackSurface {
        sql_raw: has_call_tag(security_map, "sink.sql.raw"),
        html_raw: has_call_tag(security_map, "sink.http.html.raw"),
        secrets_reveal: has_call_tag(security_map, "effect.secrets.reveal"),
        internal_net: policy.net_internal.enabled,
        fs_enabled: policy.fs.enabled,
        public_redirects: policy.net_public.allow_redirects,
    };

    let mut findings = Vec::new();

    if posture.cors.allow_credentials && posture.cors.wildcard {
        findings.push(finding(
            "CORS_CREDENTIALS_WITH_WILDCARD",
            AuditSeverity::CRITICAL,
            "cors",
            json!({"allowCredentials": true, "wildcard": true}),
            "Use explicit allowed origins; wildcard cannot be combined with credentials.",
        ));
    }

    if posture.cors.wildcard && !posture.cors.allow_credentials {
        findings.push(finding(
            "CORS_ANY_ORIGIN",
            AuditSeverity::MEDIUM,
            "cors",
            json!({"wildcard": true, "allowCredentials": false}),
            "Prefer explicit origin allowlist even when credentials are disabled.",
        ));
    }

    if posture.cors.enabled
        && !posture.cors.wildcard
        && policy.cors.require_vary_origin
        && !middleware_has_vary_origin(security_map)
    {
        findings.push(finding(
            "CORS_VARY_ORIGIN_MISSING",
            AuditSeverity::LOW,
            "cors",
            json!({"requireVaryOrigin": true}),
            "Ensure CORS middleware emits Vary: Origin for allowlist origin mode.",
        ));
    }

    if posture.security_headers.enabled && !posture.security_headers.csp.enabled {
        findings.push(finding(
            "CSP_DISABLED",
            AuditSeverity::HIGH,
            "headers",
            json!({"cspEnabled": false}),
            "Enable CSP in security_headers policy (report-only can be used for rollout).",
        ));
    }

    if posture.security_headers.csp.enabled && posture.security_headers.csp.report_only {
        let severity = if policy.env == "prod" {
            AuditSeverity::MEDIUM
        } else {
            AuditSeverity::LOW
        };
        findings.push(finding(
            "CSP_REPORT_ONLY",
            severity,
            "headers",
            json!({"reportOnly": true, "env": policy.env}),
            "Use report-only temporarily, then enforce CSP in production.",
        ));
    }

    if policy.env == "prod" && posture.security_headers.enabled && !posture.security_headers.hsts {
        findings.push(finding(
            "HSTS_DISABLED_IN_PROD",
            AuditSeverity::MEDIUM,
            "headers",
            json!({"hstsEnabled": false, "env": "prod"}),
            "Enable HSTS for HTTPS production deployments.",
        ));
    }

    if matches!(policy.auth.mode.as_str(), "cookie" | "mixed") && !policy.csrf.enabled {
        findings.push(finding(
            "CSRF_REQUIRED_BUT_DISABLED",
            AuditSeverity::HIGH,
            "csrf",
            json!({"authMode": policy.auth.mode, "csrfEnabled": false}),
            "Enable CSRF for cookie/mixed auth or switch to token-only auth mode.",
        ));
    }

    if policy.auth.cross_site_frontend
        && matches!(policy.auth.mode.as_str(), "cookie" | "mixed")
        && !policy.cors.allow_credentials
    {
        findings.push(finding(
            "COOKIE_CROSS_SITE_WITHOUT_CORS_CREDS",
            AuditSeverity::MEDIUM,
            "cors",
            json!({"crossSiteFrontend": true, "allowCredentials": false}),
            "Enable CORS credentials for cross-site cookie authentication.",
        ));
    }

    if policy.auth.cross_site_frontend
        && matches!(policy.auth.mode.as_str(), "cookie" | "mixed")
        && policy.cors.has_wildcard_origin()
    {
        findings.push(finding(
            "COOKIE_CROSS_SITE_WITH_WILDCARD_ORIGIN",
            AuditSeverity::HIGH,
            "cors",
            json!({"crossSiteFrontend": true, "wildcard": true}),
            "Cross-site cookie auth requires explicit CORS origin allowlist.",
        ));
    }

    if policy.csrf.same_site == "None" && !policy.csrf.secure_cookie {
        findings.push(finding(
            "COOKIE_SAMESITE_NONE_WITHOUT_SECURE",
            AuditSeverity::HIGH,
            "csrf",
            json!({"sameSite": "None", "secureCookie": false}),
            "Set secure_cookie=true when same_site is None.",
        ));
    }

    if policy.net_internal.enabled
        && policy.net_internal.allowed_cidrs.is_empty()
        && policy.net_internal.allowed_domains.is_empty()
    {
        findings.push(finding(
            "INTERNAL_NET_ENABLED_NO_ALLOWLIST",
            AuditSeverity::CRITICAL,
            "ssrf",
            json!({"internalNetEnabled": true, "allowedCidrs": [], "allowedDomains": []}),
            "Disable internal net or define explicit CIDR/domain allowlists.",
        ));
    }

    if policy.net_public.allow_redirects && !policy.net_ssrf.revalidate_redirects {
        findings.push(finding(
            "PUBLIC_REDIRECTS_ENABLED_WITHOUT_REVALIDATION",
            AuditSeverity::HIGH,
            "ssrf",
            json!({"allowRedirects": true, "revalidateRedirects": false}),
            "Disable redirects or revalidate every redirect hop against SSRF checks.",
        ));
    }

    if policy.fs.enabled && policy.fs.allowed_base_paths.is_empty() {
        findings.push(finding(
            "FS_ENABLED_NO_BASE_ALLOWLIST",
            AuditSeverity::HIGH,
            "fs",
            json!({"fsEnabled": true, "allowedBasePaths": []}),
            "Restrict filesystem access to explicit allowed base paths.",
        ));
    }

    if policy.capture.mode != "off" && !required_redactions_present(&policy.capture.redact_headers)
    {
        findings.push(finding(
            "CAPTURE_REDACTION_INCOMPLETE",
            AuditSeverity::HIGH,
            "capture",
            json!({"redactHeaders": policy.capture.redact_headers}),
            "Capture redaction must include authorization, cookie, and set-cookie headers.",
        ));
    }

    if policy.capture.mode == "all" && policy.env == "prod" {
        findings.push(finding(
            "CAPTURE_ALL_IN_PROD",
            AuditSeverity::HIGH,
            "capture",
            json!({"mode": "all", "env": "prod"}),
            "Use errors or sample capture mode in production.",
        ));
    }

    if policy.replay.effects == "allow" {
        findings.push(finding(
            "REPLAY_EFFECTS_ALLOW",
            AuditSeverity::MEDIUM,
            "replay",
            json!({"effects": "allow"}),
            "Prefer replay effects=deny or mock outside isolated staging workflows.",
        ));
    }

    if attack_surface.secrets_reveal {
        let severity = if policy.forbidden_effects.contains("secrets.reveal") {
            AuditSeverity::CRITICAL
        } else {
            AuditSeverity::HIGH
        };
        findings.push(finding(
            "SECRETS_REVEAL_USED",
            severity,
            "secrets",
            json!({"forbiddenByPolicy": policy.forbidden_effects.contains("secrets.reveal")}),
            "Avoid secrets.reveal or isolate it behind audited modules and strict expiry-bound allowlists.",
        ));
    }

    findings.sort_by(|left, right| {
        right
            .severity
            .cmp(&left.severity)
            .then_with(|| left.id.cmp(&right.id))
    });

    let exceptions = security_map
        .allows
        .iter()
        .map(map_allow_to_exception)
        .collect::<Vec<_>>();

    let mut risk_score = 0i64;
    let mut highest_severity = AuditSeverity::LOW;
    let mut finding_counts = HashMap::from([
        ("LOW".to_string(), 0i64),
        ("MEDIUM".to_string(), 0i64),
        ("HIGH".to_string(), 0i64),
        ("CRITICAL".to_string(), 0i64),
    ]);

    for finding in &findings {
        risk_score += finding.severity.weight();
        if finding.severity > highest_severity {
            highest_severity = finding.severity;
        }

        let key = format!("{:?}", finding.severity);
        if let Some(value) = finding_counts.get_mut(&key) {
            *value += 1;
        }
    }

    AuditReport {
        version: "0.1".to_string(),
        policy: AuditPolicySummary {
            name: policy.name.clone(),
            version: policy.version.clone(),
            hash: policy.policy_hash(),
            mode: policy.mode_as_str().to_string(),
            env: policy.env.clone(),
        },
        build: AuditBuildSummary {
            compiler_hash: format!("cpl_{}", env!("CARGO_PKG_VERSION").replace('.', "_")),
            runtime_hash: "rt_v0_stub".to_string(),
            time_ms: now_ms(),
        },
        posture,
        attack_surface,
        exceptions,
        findings,
        summary: AuditSummary {
            risk_score,
            highest_severity,
            finding_counts,
        },
    }
}

pub fn should_fail(report: &AuditReport, threshold: AuditSeverity) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.severity >= threshold)
}

pub fn render_security_audit_text(report: &AuditReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Security Audit (v{}) - {} ({})",
        report.version, report.policy.name, report.policy.hash
    ));
    lines.push(format!(
        "Posture: CORS(enabled={}, creds={}, wildcard={}), Headers(CSP={}, HSTS={}), CSRF(enabled={}, mode={}), Capture(mode={}), Replay(effects={})",
        report.posture.cors.enabled,
        report.posture.cors.allow_credentials,
        report.posture.cors.wildcard,
        report.posture.security_headers.csp.enabled,
        report.posture.security_headers.hsts,
        report.posture.csrf.enabled,
        report.posture.csrf.mode,
        report.posture.capture.mode,
        report.posture.replay.effects,
    ));
    lines.push(format!(
        "Attack surface: sql_raw={}, html_raw={}, secrets_reveal={}, internal_net={}, fs_enabled={}, public_redirects={}",
        report.attack_surface.sql_raw,
        report.attack_surface.html_raw,
        report.attack_surface.secrets_reveal,
        report.attack_surface.internal_net,
        report.attack_surface.fs_enabled,
        report.attack_surface.public_redirects,
    ));

    if report.exceptions.is_empty() {
        lines.push("Exceptions: none".to_string());
    } else {
        lines.push(format!("Exceptions: {}", report.exceptions.len()));
        for exception in &report.exceptions {
            lines.push(format!(
                "  - {:?} {} @ {}:{} ticket={} expires={} ({})",
                exception.severity,
                exception.policy_key,
                exception.location.file,
                exception.location.line,
                exception.ticket,
                exception.expires,
                exception.reason,
            ));
        }
    }

    if report.findings.is_empty() {
        lines.push("Findings: none".to_string());
    } else {
        lines.push("Findings:".to_string());
        for finding in &report.findings {
            lines.push(format!(
                "  - {:?} {} ({}) -> {}",
                finding.severity, finding.id, finding.category, finding.suggestion
            ));
        }
    }

    lines.push(format!(
        "Summary: riskScore={}, highestSeverity={:?}, counts={:?}",
        report.summary.risk_score, report.summary.highest_severity, report.summary.finding_counts
    ));

    lines.join("\n")
}

fn required_redactions_present(redact_headers: &[String]) -> bool {
    let normalized = redact_headers
        .iter()
        .map(|item| item.to_ascii_lowercase())
        .collect::<Vec<_>>();
    normalized.contains(&"authorization".to_string())
        && normalized.contains(&"cookie".to_string())
        && normalized.contains(&"set-cookie".to_string())
}

fn middleware_has_vary_origin(security_map: &SecurityMap) -> bool {
    security_map.middleware.iter().any(|entry| {
        entry.tags.iter().any(|tag| tag == "middleware.cors")
            && entry
                .attrs
                .get("requireVaryOrigin")
                .is_some_and(|value| matches!(value, TagAttr::Bool(true)))
    })
}

fn has_call_tag(security_map: &SecurityMap, tag: &str) -> bool {
    security_map
        .calls
        .iter()
        .any(|call| call.tags.iter().any(|candidate| candidate == tag))
}

fn finding(
    id: &str,
    severity: AuditSeverity,
    category: &str,
    evidence: Value,
    suggestion: &str,
) -> AuditFinding {
    AuditFinding {
        id: id.to_string(),
        severity,
        category: category.to_string(),
        evidence,
        suggestion: suggestion.to_string(),
    }
}

fn map_allow_to_exception(allow: &SecurityAllow) -> AuditException {
    AuditException {
        policy_key: allow.policy.clone(),
        location: AuditLocation {
            file: allow.loc.file.clone(),
            line: allow.loc.line,
            column: allow.loc.column,
        },
        reason: allow.reason.clone(),
        ticket: allow.ticket.clone(),
        expires: allow.expires.clone(),
        severity: AuditSeverity::HIGH,
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
