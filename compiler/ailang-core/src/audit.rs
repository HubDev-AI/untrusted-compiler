use crate::policy::Policy;
use crate::security_map::{SecurityAllow, SecurityMap, TagAttr};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

const ALLOW_EXPIRING_SOON_DAYS: i64 = 14;
const ALLOW_EXPIRING_SOON_HIGH_THRESHOLD: usize = 5;
const ALLOW_COUNT_HIGH_THRESHOLD: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditPolicySummary {
    pub name: String,
    pub version: String,
    pub hash: String,
    pub mode: String,
    pub env: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditBuildSummary {
    #[serde(rename = "compilerHash")]
    pub compiler_hash: String,
    #[serde(rename = "runtimeHash")]
    pub runtime_hash: String,
    #[serde(rename = "timeMs")]
    pub time_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditCorsPosture {
    pub enabled: bool,
    #[serde(rename = "allowCredentials")]
    pub allow_credentials: bool,
    pub wildcard: bool,
    pub origins: Vec<String>,
    #[serde(rename = "allowRedirects")]
    pub allow_redirects: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditCspPosture {
    pub enabled: bool,
    #[serde(rename = "reportOnly")]
    pub report_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditCsrfPosture {
    pub enabled: bool,
    pub mode: String,
    #[serde(rename = "sameSite")]
    pub same_site: String,
    #[serde(rename = "secureCookie")]
    pub secure_cookie: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditCapturePosture {
    pub mode: String,
    #[serde(rename = "redactHeaders")]
    pub redact_headers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditReplayPosture {
    pub effects: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditPosture {
    pub cors: AuditCorsPosture,
    #[serde(rename = "securityHeaders")]
    pub security_headers: AuditSecurityHeadersPosture,
    pub csrf: AuditCsrfPosture,
    pub capture: AuditCapturePosture,
    pub replay: AuditReplayPosture,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditException {
    #[serde(rename = "policyKey")]
    pub policy_key: String,
    pub location: AuditLocation,
    pub reason: String,
    pub ticket: String,
    pub expires: String,
    pub severity: AuditSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditFinding {
    pub id: String,
    pub severity: AuditSeverity,
    pub category: String,
    pub evidence: Value,
    pub suggestion: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditSummary {
    #[serde(rename = "riskScore")]
    pub risk_score: i64,
    #[serde(rename = "highestSeverity")]
    pub highest_severity: AuditSeverity,
    #[serde(rename = "findingCounts")]
    pub finding_counts: HashMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend: Option<AuditTrend>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTrend {
    #[serde(rename = "baselinePolicyHash")]
    pub baseline_policy_hash: String,
    #[serde(rename = "baselineRiskScore")]
    pub baseline_risk_score: i64,
    #[serde(rename = "riskScoreDelta")]
    pub risk_score_delta: i64,
    #[serde(rename = "findingCountDelta")]
    pub finding_count_delta: i64,
    #[serde(rename = "severityDeltas")]
    pub severity_deltas: HashMap<String, i64>,
    #[serde(rename = "addedFindingIds")]
    pub added_finding_ids: Vec<String>,
    #[serde(rename = "resolvedFindingIds")]
    pub resolved_finding_ids: Vec<String>,
}

pub fn run_security_audit(policy: &Policy, security_map: &SecurityMap) -> AuditReport {
    run_security_audit_with_baseline(policy, security_map, None)
}

pub fn run_security_audit_with_baseline(
    policy: &Policy,
    security_map: &SecurityMap,
    baseline: Option<&AuditReport>,
) -> AuditReport {
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
    let today = current_utc_iso_date();
    let today_days = days_from_iso_date(&today).unwrap_or(0);
    let mut expired_allows = Vec::new();
    let mut expiring_soon_allows = Vec::new();

    if posture.cors.allow_credentials && posture.cors.wildcard {
        findings.push(finding(
            "CORS_CREDENTIALS_WITH_WILDCARD",
            AuditSeverity::CRITICAL,
            "cors",
            json!({
                "allowCredentials": true,
                "wildcard": true,
                "sampleCalls": call_samples_for_tag(security_map, "middleware.cors", 5),
            }),
            "Use explicit allowed origins; wildcard cannot be combined with credentials.",
        ));
    }

    if posture.cors.wildcard && !posture.cors.allow_credentials {
        findings.push(finding(
            "CORS_ANY_ORIGIN",
            AuditSeverity::MEDIUM,
            "cors",
            json!({
                "wildcard": true,
                "allowCredentials": false,
                "sampleCalls": call_samples_for_tag(security_map, "middleware.cors", 5),
            }),
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
            json!({
                "requireVaryOrigin": true,
                "sampleCalls": call_samples_for_tag(security_map, "middleware.cors", 5),
            }),
            "Ensure CORS middleware emits Vary: Origin for allowlist origin mode.",
        ));
    }

    if posture.security_headers.enabled && !posture.security_headers.csp.enabled {
        findings.push(finding(
            "CSP_DISABLED",
            AuditSeverity::HIGH,
            "headers",
            json!({
                "cspEnabled": false,
                "sampleCalls": call_samples_for_tag(
                    security_map,
                    "middleware.security_headers",
                    5,
                ),
            }),
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
            json!({
                "reportOnly": true,
                "env": policy.env,
                "sampleCalls": call_samples_for_tag(
                    security_map,
                    "middleware.security_headers",
                    5,
                ),
            }),
            "Use report-only temporarily, then enforce CSP in production.",
        ));
    }

    if policy.env == "prod" && posture.security_headers.enabled && !posture.security_headers.hsts {
        findings.push(finding(
            "HSTS_DISABLED_IN_PROD",
            AuditSeverity::MEDIUM,
            "headers",
            json!({
                "hstsEnabled": false,
                "env": "prod",
                "sampleCalls": call_samples_for_tag(
                    security_map,
                    "middleware.security_headers",
                    5,
                ),
            }),
            "Enable HSTS for HTTPS production deployments.",
        ));
    }

    if matches!(policy.auth.mode.as_str(), "cookie" | "mixed") && !policy.csrf.enabled {
        findings.push(finding(
            "CSRF_REQUIRED_BUT_DISABLED",
            AuditSeverity::HIGH,
            "csrf",
            json!({
                "authMode": policy.auth.mode,
                "csrfEnabled": false,
                "sampleCalls": call_samples_for_tags(
                    security_map,
                    &["middleware.auth", "middleware.csrf"],
                    5,
                ),
            }),
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
            json!({
                "crossSiteFrontend": true,
                "allowCredentials": false,
                "sampleCalls": call_samples_for_tags(
                    security_map,
                    &["middleware.cors", "middleware.auth"],
                    5,
                ),
            }),
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
            json!({
                "crossSiteFrontend": true,
                "wildcard": true,
                "sampleCalls": call_samples_for_tags(
                    security_map,
                    &["middleware.cors", "middleware.auth"],
                    5,
                ),
            }),
            "Cross-site cookie auth requires explicit CORS origin allowlist.",
        ));
    }

    if policy.csrf.same_site == "None" && !policy.csrf.secure_cookie {
        findings.push(finding(
            "COOKIE_SAMESITE_NONE_WITHOUT_SECURE",
            AuditSeverity::HIGH,
            "csrf",
            json!({
                "sameSite": "None",
                "secureCookie": false,
                "sampleCalls": call_samples_for_tags(
                    security_map,
                    &["middleware.csrf", "middleware.auth"],
                    5,
                ),
            }),
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
            json!({
                "internalNetEnabled": true,
                "allowedCidrs": [],
                "allowedDomains": [],
                "sampleCalls": call_samples_for_tag(security_map, "sink.net.internal_request", 5),
            }),
            "Disable internal net or define explicit CIDR/domain allowlists.",
        ));
    }

    if policy.net_public.allow_redirects && !policy.net_ssrf.revalidate_redirects {
        findings.push(finding(
            "PUBLIC_REDIRECTS_ENABLED_WITHOUT_REVALIDATION",
            AuditSeverity::HIGH,
            "ssrf",
            json!({
                "allowRedirects": true,
                "revalidateRedirects": false,
                "sampleCalls": call_samples_for_tag(security_map, "sink.net.public_request", 5),
            }),
            "Disable redirects or revalidate every redirect hop against SSRF checks.",
        ));
    }

    if policy.fs.enabled && policy.fs.allowed_base_paths.is_empty() {
        findings.push(finding(
            "FS_ENABLED_NO_BASE_ALLOWLIST",
            AuditSeverity::HIGH,
            "fs",
            json!({
                "fsEnabled": true,
                "allowedBasePaths": [],
                "sampleCalls": call_samples_for_tags(
                    security_map,
                    &["sink.fs.read", "sink.fs.write"],
                    5,
                ),
            }),
            "Restrict filesystem access to explicit allowed base paths.",
        ));
    }

    if policy.capture.mode != "off" && !required_redactions_present(&policy.capture.redact_headers)
    {
        findings.push(finding(
            "CAPTURE_REDACTION_INCOMPLETE",
            AuditSeverity::HIGH,
            "capture",
            json!({
                "redactHeaders": policy.capture.redact_headers,
                "sampleCalls": call_samples_for_tags(
                    security_map,
                    &["source.http.header", "source.http.body"],
                    5,
                ),
            }),
            "Capture redaction must include authorization, cookie, and set-cookie headers.",
        ));
    }

    if policy.capture.mode == "all" && policy.env == "prod" {
        findings.push(finding(
            "CAPTURE_ALL_IN_PROD",
            AuditSeverity::HIGH,
            "capture",
            json!({
                "mode": "all",
                "env": "prod",
                "sampleCalls": call_samples_for_tags(
                    security_map,
                    &["source.http.body", "source.http.query", "source.http.header", "source.http.path"],
                    5,
                ),
            }),
            "Use errors or sample capture mode in production.",
        ));
    }

    if policy.replay.effects == "allow" {
        findings.push(finding(
            "REPLAY_EFFECTS_ALLOW",
            AuditSeverity::MEDIUM,
            "replay",
            json!({
                "effects": "allow",
                "sampleCalls": call_samples_for_tags(
                    security_map,
                    &[
                        "effect.net",
                        "effect.db.read",
                        "effect.db.write",
                        "effect.fs.read",
                        "effect.fs.write",
                        "effect.secrets.read",
                        "effect.secrets.reveal",
                    ],
                    5,
                ),
            }),
            "Prefer replay effects=deny or mock outside isolated staging workflows.",
        ));
    }

    if !policy.logging.structured_only {
        findings.push(finding(
            "LOG_STRUCTURED_ONLY_DISABLED",
            AuditSeverity::HIGH,
            "logging",
            json!({
                "structuredOnly": false,
                "sampleCalls": call_samples_for_tag(security_map, "sink.log.emit", 5),
            }),
            "Enable structured-only logging to reduce injection and data-leak risk.",
        ));
    }

    if policy.logging.include_remote_ip {
        findings.push(finding(
            "LOG_REMOTE_IP_ENABLED",
            AuditSeverity::MEDIUM,
            "logging",
            json!({
                "includeRemoteIp": true,
                "sampleCalls": call_samples_for_tag(security_map, "sink.log.emit", 5),
            }),
            "Review remote IP logging necessity and privacy impact for this environment.",
        ));
    }

    if policy.logging.include_user_agent {
        findings.push(finding(
            "LOG_USER_AGENT_ENABLED",
            AuditSeverity::LOW,
            "logging",
            json!({
                "includeUserAgent": true,
                "sampleCalls": call_samples_for_tag(security_map, "sink.log.emit", 5),
            }),
            "User-Agent logging can increase PII footprint; keep only if operationally required.",
        ));
    }

    if !policy.sql.forbid_raw {
        findings.push(finding(
            "SQL_RAW_ALLOWED_BY_POLICY",
            AuditSeverity::HIGH,
            "sql",
            json!({
                "forbidRaw": false,
                "sampleCalls": call_samples_for_tags(
                    security_map,
                    &["sink.sql.exec", "sink.sql.query"],
                    5,
                ),
            }),
            "Set sql.forbid_raw=true to keep raw SQL execution disabled by policy.",
        ));
    }

    if policy.sql.require_limit_on_select == "off" {
        findings.push(finding(
            "SQL_LIMIT_RULE_DISABLED",
            AuditSeverity::MEDIUM,
            "sql",
            json!({
                "requireLimitOnSelect": "off",
                "sampleCalls": call_samples_for_tags(
                    security_map,
                    &["sink.sql.exec", "sink.sql.query"],
                    5,
                ),
            }),
            "Enable SELECT limit policy (`warn` or `enforce`) to reduce unbounded query risk.",
        ));
    }

    if policy.sql.require_limit_on_select != "off"
        && has_call_tag(security_map, "sql.select_without_limit")
    {
        let severity = if policy.sql.require_limit_on_select == "enforce" {
            AuditSeverity::HIGH
        } else {
            AuditSeverity::MEDIUM
        };
        findings.push(finding(
            "SQL_SELECT_WITHOUT_LIMIT",
            severity,
            "sql",
            json!({
                "requireLimitOnSelect": policy.sql.require_limit_on_select,
                "count": count_call_tag(security_map, "sql.select_without_limit"),
                "sampleCalls": call_samples_for_tag(security_map, "sql.select_without_limit", 5),
            }),
            "Add LIMIT to SELECT queries or justify the exception with a narrowly scoped allowlist.",
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
            json!({
                "forbiddenByPolicy": policy.forbidden_effects.contains("secrets.reveal"),
                "sampleCalls": call_samples_for_tag(security_map, "effect.secrets.reveal", 5),
            }),
            "Avoid secrets.reveal or isolate it behind audited modules and strict expiry-bound allowlists.",
        ));
    }

    for allow in &security_map.allows {
        let evidence = json!({
            "policyKey": &allow.policy,
            "ticket": &allow.ticket,
            "location": {
                "file": &allow.loc.file,
                "line": allow.loc.line,
                "column": allow.loc.column,
            },
            "expires": &allow.expires,
            "bypass": &allow.bypass,
            "sampleCalls": call_samples_for_bypass_tags(security_map, &allow.bypass, 5),
        });

        if allow
            .bypass
            .iter()
            .any(|tag| tag == "effect.secrets.reveal")
        {
            findings.push(finding(
                "SECRETS_REVEAL_ALLOWLISTED",
                AuditSeverity::HIGH,
                "secrets",
                evidence.clone(),
                "Confine secret reveal bypasses to minimal dev-only scope and enforce short expiry windows.",
            ));
        }

        if allow
            .bypass
            .iter()
            .any(|tag| tag == "sink.net.internal_request")
        {
            findings.push(finding(
                "INTERNAL_NET_CALL_ALLOWLISTED",
                AuditSeverity::HIGH,
                "ssrf",
                evidence.clone(),
                "Validate internal net bypass scope and keep CIDR/domain allowlists strict.",
            ));
        }

        if let Some(expiry_days) = days_from_iso_date(&allow.expires) {
            if expiry_days < today_days {
                expired_allows.push(allow.clone());
                findings.push(finding(
                    "ALLOW_EXPIRED",
                    AuditSeverity::HIGH,
                    "policy",
                    evidence.clone(),
                    "Remove or renew expired @allow exceptions immediately.",
                ));
            } else if (expiry_days - today_days) <= ALLOW_EXPIRING_SOON_DAYS {
                expiring_soon_allows.push(allow.clone());
                findings.push(finding(
                    "ALLOW_EXPIRING_SOON",
                    AuditSeverity::MEDIUM,
                    "policy",
                    evidence,
                    "Review and renew/remove this @allow before expiry.",
                ));
            }
        }
    }

    if !expired_allows.is_empty() || !expiring_soon_allows.is_empty() {
        let mut window = expired_allows.clone();
        window.extend(expiring_soon_allows.clone());
        let rollup_severity = if !expired_allows.is_empty()
            || expiring_soon_allows.len() >= ALLOW_EXPIRING_SOON_HIGH_THRESHOLD
        {
            AuditSeverity::HIGH
        } else {
            AuditSeverity::MEDIUM
        };
        let stats = expiry_window_stats(&window, today_days);
        let mut evidence = json!({
            "expiredCount": expired_allows.len(),
            "expiringSoonCount": expiring_soon_allows.len(),
            "windowDays": ALLOW_EXPIRING_SOON_DAYS,
            "sampleExceptions": exception_samples(&window, 5),
            "severityInputs": {
                "expiredTriggersHigh": true,
                "expiringSoonHighThreshold": ALLOW_EXPIRING_SOON_HIGH_THRESHOLD,
            },
        });
        if let Some(stats) = stats {
            if let Some(obj) = evidence.as_object_mut() {
                obj.insert(
                    "minDaysUntilExpiry".to_string(),
                    json!(stats.min_days_until_expiry),
                );
                obj.insert(
                    "maxDaysUntilExpiry".to_string(),
                    json!(stats.max_days_until_expiry),
                );
                obj.insert(
                    "medianDaysUntilExpiry".to_string(),
                    json!(stats.median_days_until_expiry),
                );
                obj.insert(
                    "expiringIn7DaysCount".to_string(),
                    json!(stats.expiring_in_7_days),
                );
                obj.insert(
                    "expiringIn30DaysCount".to_string(),
                    json!(stats.expiring_in_30_days),
                );
            }
        }
        findings.push(finding(
            "ALLOW_EXPIRY_WINDOW_ROLLUP",
            rollup_severity,
            "policy",
            evidence,
            "Reduce expiring/expired @allow exceptions and keep exception windows short and explicit.",
        ));
    }

    if security_map.allows.len() > ALLOW_COUNT_HIGH_THRESHOLD {
        findings.push(finding(
            "ALLOW_COUNT_HIGH",
            AuditSeverity::LOW,
            "policy",
            json!({
                "count": security_map.allows.len(),
                "threshold": ALLOW_COUNT_HIGH_THRESHOLD,
                "sampleExceptions": exception_samples(&security_map.allows, 5),
            }),
            "Reduce active @allow exceptions to keep security posture maintainable.",
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

    let mut report = AuditReport {
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
        trend: None,
    };

    report.trend = baseline.map(|baseline_report| compute_trend(&report, baseline_report));
    report
}

pub fn should_fail(report: &AuditReport, threshold: AuditSeverity) -> bool {
    report
        .findings
        .iter()
        .any(|finding| finding.severity >= threshold)
}

fn compute_trend(current: &AuditReport, baseline: &AuditReport) -> AuditTrend {
    let severity_keys = ["LOW", "MEDIUM", "HIGH", "CRITICAL"];
    let mut severity_deltas = HashMap::new();
    for key in severity_keys {
        let current_count = current
            .summary
            .finding_counts
            .get(key)
            .copied()
            .unwrap_or(0);
        let baseline_count = baseline
            .summary
            .finding_counts
            .get(key)
            .copied()
            .unwrap_or(0);
        severity_deltas.insert(key.to_string(), current_count - baseline_count);
    }

    let current_ids = current
        .findings
        .iter()
        .map(|finding| finding.id.clone())
        .collect::<BTreeSet<_>>();
    let baseline_ids = baseline
        .findings
        .iter()
        .map(|finding| finding.id.clone())
        .collect::<BTreeSet<_>>();

    let added_finding_ids = current_ids
        .difference(&baseline_ids)
        .take(20)
        .cloned()
        .collect::<Vec<_>>();
    let resolved_finding_ids = baseline_ids
        .difference(&current_ids)
        .take(20)
        .cloned()
        .collect::<Vec<_>>();

    AuditTrend {
        baseline_policy_hash: baseline.policy.hash.clone(),
        baseline_risk_score: baseline.summary.risk_score,
        risk_score_delta: current.summary.risk_score - baseline.summary.risk_score,
        finding_count_delta: current.findings.len() as i64 - baseline.findings.len() as i64,
        severity_deltas,
        added_finding_ids,
        resolved_finding_ids,
    }
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
            for sample_line in sample_call_preview_lines(&finding.evidence) {
                lines.push(format!("    {sample_line}"));
            }
        }
    }

    lines.push(format!(
        "Summary: riskScore={}, highestSeverity={:?}, counts={:?}",
        report.summary.risk_score, report.summary.highest_severity, report.summary.finding_counts
    ));
    if let Some(trend) = &report.trend {
        lines.push(format!(
            "Trend: baselinePolicyHash={}, baselineRiskScore={}, riskScoreDelta={}, findingCountDelta={}, severityDeltas={:?}",
            trend.baseline_policy_hash,
            trend.baseline_risk_score,
            trend.risk_score_delta,
            trend.finding_count_delta,
            trend.severity_deltas
        ));
        if !trend.added_finding_ids.is_empty() {
            lines.push(format!(
                "Trend added: {}",
                trend.added_finding_ids.join(", ")
            ));
        }
        if !trend.resolved_finding_ids.is_empty() {
            lines.push(format!(
                "Trend resolved: {}",
                trend.resolved_finding_ids.join(", ")
            ));
        }
    }

    lines.join("\n")
}

fn sample_call_preview_lines(evidence: &Value) -> Vec<String> {
    let Some(samples) = evidence
        .get("sampleCalls")
        .and_then(|value| value.as_array())
    else {
        return Vec::new();
    };

    let mut lines = Vec::new();
    for sample in samples.iter().take(2) {
        let callee = sample
            .get("callee")
            .and_then(|value| value.as_str())
            .unwrap_or("<unknown>");
        let location = sample
            .get("location")
            .and_then(|value| value.as_object())
            .map(|location| {
                let file = location
                    .get("file")
                    .and_then(|value| value.as_str())
                    .unwrap_or("<unknown>");
                let line = location
                    .get("line")
                    .and_then(|value| value.as_i64())
                    .unwrap_or(0);
                format!("{file}:{line}")
            })
            .unwrap_or_else(|| "<unknown>".to_string());

        let trace_preview = sample
            .get("originEdges")
            .and_then(|value| value.as_array())
            .and_then(|edges| edges.first())
            .and_then(|edge| edge.get("trace"))
            .and_then(|value| value.as_array())
            .map(|steps| {
                steps
                    .iter()
                    .filter_map(|value| value.as_str())
                    .take(4)
                    .collect::<Vec<_>>()
                    .join(" -> ")
            })
            .filter(|text| !text.is_empty());

        match trace_preview {
            Some(trace) => lines.push(format!("sample: {callee}@{location} trace={trace}")),
            None => lines.push(format!("sample: {callee}@{location}")),
        }
    }

    if samples.len() > 2 {
        lines.push(format!("sample: +{} more", samples.len() - 2));
    }
    lines
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

fn count_call_tag(security_map: &SecurityMap, tag: &str) -> usize {
    security_map
        .calls
        .iter()
        .filter(|call| call.tags.iter().any(|candidate| candidate == tag))
        .count()
}

fn call_samples_for_tag(security_map: &SecurityMap, tag: &str, limit: usize) -> Vec<Value> {
    call_samples_for_tags(security_map, &[tag], limit)
}

fn call_samples_for_tags(security_map: &SecurityMap, tags: &[&str], limit: usize) -> Vec<Value> {
    security_map
        .calls
        .iter()
        .filter(|call| {
            call.tags
                .iter()
                .any(|candidate| tags.iter().any(|tag| candidate == tag))
        })
        .take(limit)
        .map(|call| {
            let mut sample = serde_json::Map::new();
            sample.insert("callee".to_string(), json!(call.callee));
            sample.insert(
                "location".to_string(),
                json!({
                    "file": call.loc.file,
                    "line": call.loc.line,
                    "column": call.loc.column,
                }),
            );
            if let Some(roles) = &call.arg_roles {
                sample.insert("argRoles".to_string(), json!(roles));
            }
            if let Some(edges) = &call.origin_edges {
                let mapped = edges
                    .iter()
                    .map(|edge| {
                        let mut item = serde_json::Map::new();
                        item.insert("argIndex".to_string(), json!(edge.arg_index));
                        item.insert("origin".to_string(), json!(edge.origin));
                        item.insert("tags".to_string(), json!(edge.tags));
                        if !edge.trace.is_empty() {
                            item.insert("trace".to_string(), json!(edge.trace));
                        }
                        Value::Object(item)
                    })
                    .collect::<Vec<_>>();
                if !mapped.is_empty() {
                    sample.insert("originEdges".to_string(), Value::Array(mapped));
                }
            }
            Value::Object(sample)
        })
        .collect()
}

fn call_samples_for_bypass_tags(
    security_map: &SecurityMap,
    bypass_tags: &[String],
    limit: usize,
) -> Vec<Value> {
    let tags = bypass_tags.iter().map(String::as_str).collect::<Vec<_>>();
    if tags.is_empty() {
        Vec::new()
    } else {
        call_samples_for_tags(security_map, &tags, limit)
    }
}

fn exception_samples(allows: &[SecurityAllow], limit: usize) -> Vec<Value> {
    allows
        .iter()
        .take(limit)
        .map(|allow| {
            json!({
                "policyKey": allow.policy,
                "ticket": allow.ticket,
                "expires": allow.expires,
                "bypass": allow.bypass,
                "location": {
                    "file": allow.loc.file,
                    "line": allow.loc.line,
                    "column": allow.loc.column,
                },
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpiryWindowStats {
    min_days_until_expiry: i64,
    max_days_until_expiry: i64,
    median_days_until_expiry: i64,
    expiring_in_7_days: usize,
    expiring_in_30_days: usize,
}

fn expiry_window_stats(allows: &[SecurityAllow], today_days: i64) -> Option<ExpiryWindowStats> {
    let mut deltas = allows
        .iter()
        .filter_map(|allow| days_from_iso_date(&allow.expires).map(|expiry| expiry - today_days))
        .collect::<Vec<_>>();
    if deltas.is_empty() {
        return None;
    }
    deltas.sort_unstable();

    let min_days_until_expiry = *deltas.first().unwrap_or(&0);
    let max_days_until_expiry = *deltas.last().unwrap_or(&0);
    let median_days_until_expiry = deltas[deltas.len() / 2];
    let expiring_in_7_days = deltas
        .iter()
        .filter(|delta| **delta >= 0 && **delta <= 7)
        .count();
    let expiring_in_30_days = deltas
        .iter()
        .filter(|delta| **delta >= 0 && **delta <= 30)
        .count();

    Some(ExpiryWindowStats {
        min_days_until_expiry,
        max_days_until_expiry,
        median_days_until_expiry,
        expiring_in_7_days,
        expiring_in_30_days,
    })
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
        severity: exception_severity(allow),
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn exception_severity(allow: &SecurityAllow) -> AuditSeverity {
    if allow.bypass.iter().any(|tag| {
        matches!(
            tag.as_str(),
            "effect.secrets.reveal" | "sink.net.internal_request" | "sink.fs.write"
        )
    }) {
        AuditSeverity::HIGH
    } else {
        AuditSeverity::MEDIUM
    }
}

fn current_utc_iso_date() -> String {
    let days_since_unix_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| (duration.as_secs() / 86_400) as i64)
        .unwrap_or(0);
    let (year, month, day) = civil_from_days(days_since_unix_epoch);
    format!("{year:04}-{month:02}-{day:02}")
}

fn days_from_iso_date(input: &str) -> Option<i64> {
    let mut parts = input.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day))
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let adjusted_year = year - if month <= 2 { 1 } else { 0 };
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let yoe = adjusted_year - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    (year, month, day)
}
