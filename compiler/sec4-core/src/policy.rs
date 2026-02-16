#![allow(dead_code)]

use crate::diagnostics::{Diagnostic, Span};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub const POLICY_FILE_NAME: &str = "sec4.policy";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyMode {
    Enforce,
    Warn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorsPolicyConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub max_age_seconds: i64,
    pub allow_credentials: bool,
    pub allow_private_network: bool,
    pub reflect_origin: bool,
    pub forbid_any_origin: bool,
    pub forbid_reflect_origin: bool,
    pub require_vary_origin: bool,
}

impl CorsPolicyConfig {
    pub fn has_wildcard_origin(&self) -> bool {
        self.allowed_origins.iter().any(|origin| origin == "*")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecurityHeadersPolicyConfig {
    pub enabled: bool,
    pub hsts_enabled: bool,
    pub hsts_max_age_seconds: i64,
    pub hsts_include_subdomains: bool,
    pub hsts_preload: bool,
    pub csp_enabled: bool,
    pub csp_report_only: bool,
    pub csp_policy: String,
    pub x_frame_options: String,
    pub x_content_type_options: bool,
    pub referrer_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CsrfPolicyConfig {
    pub enabled: bool,
    pub mode: String,
    pub cookie_name: String,
    pub header_name: String,
    pub same_site: String,
    pub secure_cookie: bool,
    pub protected_methods: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthPolicyConfig {
    pub mode: String,
    pub cross_site_frontend: bool,
    pub cookie_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapturePolicyConfig {
    pub mode: String,
    pub redact_headers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplayPolicyConfig {
    pub effects: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoggingPolicyConfig {
    pub structured_only: bool,
    pub include_remote_ip: bool,
    pub include_user_agent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SqlPolicyConfig {
    pub forbid_raw: bool,
    pub require_limit_on_select: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JsonPolicyConfig {
    pub max_bytes: i64,
    pub max_depth: i64,
    pub require_schema_for_encode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpPolicyConfig {
    pub max_body_bytes: i64,
    pub max_concurrency: i64,
    pub max_header_bytes: i64,
    pub max_multipart_bytes: i64,
    pub default_timeout_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NetPublicPolicyConfig {
    pub allow_redirects: bool,
    pub max_redirects: i64,
    pub allowed_schemes: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub blocked_domains: Vec<String>,
    pub allowed_ports: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NetInternalPolicyConfig {
    pub enabled: bool,
    pub allowed_cidrs: Vec<String>,
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NetSsrfPolicyConfig {
    pub block_private_ranges: bool,
    pub block_loopback: bool,
    pub block_link_local: bool,
    pub block_metadata_ips: bool,
    pub revalidate_redirects: bool,
    pub resolve_dns: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FsPolicyConfig {
    pub enabled: bool,
    pub allowed_base_paths: Vec<String>,
    pub forbid_symlinks: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub name: String,
    pub version: String,
    pub mode: PolicyMode,
    pub env: String,
    pub forbidden_effects: HashSet<String>,
    pub cors_forbid_any_origin: bool,
    pub cors_forbid_reflect_origin: bool,
    pub cors_require_vary_origin: bool,
    pub cors: CorsPolicyConfig,
    pub security_headers: SecurityHeadersPolicyConfig,
    pub csrf: CsrfPolicyConfig,
    pub auth: AuthPolicyConfig,
    pub capture: CapturePolicyConfig,
    pub replay: ReplayPolicyConfig,
    pub logging: LoggingPolicyConfig,
    pub sql: SqlPolicyConfig,
    pub json: JsonPolicyConfig,
    pub http: HttpPolicyConfig,
    pub net_public: NetPublicPolicyConfig,
    pub net_internal: NetInternalPolicyConfig,
    pub net_ssrf: NetSsrfPolicyConfig,
    pub fs: FsPolicyConfig,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            name: "default-secure".to_string(),
            version: "0.1".to_string(),
            mode: PolicyMode::Enforce,
            env: "prod".to_string(),
            forbidden_effects: ["shell", "unsafe", "secrets.reveal"]
                .into_iter()
                .map(|item| item.to_string())
                .collect(),
            cors_forbid_any_origin: true,
            cors_forbid_reflect_origin: true,
            cors_require_vary_origin: true,
            cors: CorsPolicyConfig {
                enabled: true,
                allowed_origins: vec!["https://app.example.com".to_string()],
                allowed_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "PATCH".to_string(),
                    "DELETE".to_string(),
                    "OPTIONS".to_string(),
                ],
                allowed_headers: vec!["content-type".to_string(), "authorization".to_string()],
                exposed_headers: Vec::new(),
                max_age_seconds: 600,
                allow_credentials: true,
                allow_private_network: false,
                reflect_origin: false,
                forbid_any_origin: true,
                forbid_reflect_origin: true,
                require_vary_origin: true,
            },
            security_headers: SecurityHeadersPolicyConfig {
                enabled: true,
                hsts_enabled: true,
                hsts_max_age_seconds: 15552000,
                hsts_include_subdomains: true,
                hsts_preload: false,
                csp_enabled: true,
                csp_report_only: false,
                csp_policy:
                    "default-src 'self'; frame-ancestors 'none'; base-uri 'self'".to_string(),
                x_frame_options: "DENY".to_string(),
                x_content_type_options: true,
                referrer_policy: "strict-origin-when-cross-origin".to_string(),
            },
            csrf: CsrfPolicyConfig {
                enabled: true,
                mode: "double_submit".to_string(),
                cookie_name: "csrf".to_string(),
                header_name: "X-CSRF-Token".to_string(),
                same_site: "Lax".to_string(),
                secure_cookie: true,
                protected_methods: vec![
                    "POST".to_string(),
                    "PUT".to_string(),
                    "PATCH".to_string(),
                    "DELETE".to_string(),
                ],
            },
            auth: AuthPolicyConfig {
                mode: "token".to_string(),
                cross_site_frontend: false,
                cookie_name: "session".to_string(),
            },
            capture: CapturePolicyConfig {
                mode: "errors".to_string(),
                redact_headers: vec![
                    "authorization".to_string(),
                    "cookie".to_string(),
                    "set-cookie".to_string(),
                    "x-api-key".to_string(),
                    "x-auth-token".to_string(),
                ],
            },
            replay: ReplayPolicyConfig {
                effects: "deny".to_string(),
            },
            logging: LoggingPolicyConfig {
                structured_only: true,
                include_remote_ip: false,
                include_user_agent: false,
            },
            sql: SqlPolicyConfig {
                forbid_raw: true,
                require_limit_on_select: "warn".to_string(),
            },
            json: JsonPolicyConfig {
                max_bytes: 1_048_576,
                max_depth: 32,
                require_schema_for_encode: true,
            },
            http: HttpPolicyConfig {
                max_body_bytes: 4_096,
                max_concurrency: 256,
                max_header_bytes: 8_191,
                max_multipart_bytes: 4_096,
                default_timeout_ms: 200,
            },
            net_public: NetPublicPolicyConfig {
                allow_redirects: false,
                max_redirects: 0,
                allowed_schemes: vec!["https".to_string()],
                allowed_domains: Vec::new(),
                blocked_domains: Vec::new(),
                allowed_ports: Vec::new(),
            },
            net_internal: NetInternalPolicyConfig {
                enabled: false,
                allowed_cidrs: Vec::new(),
                allowed_domains: Vec::new(),
            },
            net_ssrf: NetSsrfPolicyConfig {
                block_private_ranges: true,
                block_loopback: true,
                block_link_local: true,
                block_metadata_ips: true,
                revalidate_redirects: true,
                resolve_dns: true,
            },
            fs: FsPolicyConfig {
                enabled: false,
                allowed_base_paths: Vec::new(),
                forbid_symlinks: "enforce".to_string(),
            },
        }
    }
}

impl Policy {
    pub fn mode_as_str(&self) -> &'static str {
        match self.mode {
            PolicyMode::Enforce => "enforce",
            PolicyMode::Warn => "warn",
        }
    }

    pub fn policy_hash(&self) -> String {
        let mut forbidden_effects = self.forbidden_effects.iter().cloned().collect::<Vec<_>>();
        forbidden_effects.sort();

        let normalized = serde_json::json!({
            "name": self.name,
            "version": self.version,
            "mode": self.mode_as_str(),
            "env": self.env,
            "forbidden_effects": forbidden_effects,
            "cors": self.cors,
            "security_headers": self.security_headers,
            "csrf": self.csrf,
            "auth": self.auth,
            "capture": self.capture,
            "replay": self.replay,
            "logging": self.logging,
            "sql": self.sql,
            "json": self.json,
            "http": self.http,
            "net_public": self.net_public,
            "net_internal": self.net_internal,
            "net_ssrf": self.net_ssrf,
            "fs": self.fs,
        });

        let serialized =
            serde_json::to_vec(&normalized).expect("policy normalization should serialize");
        let hash = fnv1a64(&serialized);
        format!("pol_{hash:016x}")
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyFile {
    #[serde(default)]
    policy: Option<PolicySection>,
    #[serde(default)]
    effects: Option<EffectsSection>,
    #[serde(default)]
    secrets: Option<SecretsSection>,
    #[serde(default)]
    json: Option<JsonSection>,
    #[serde(default)]
    http: Option<HttpSection>,
    #[serde(default)]
    budget: Option<BudgetSection>,
    #[serde(default)]
    logging: Option<LoggingSection>,
    #[serde(default)]
    sql: Option<SqlSection>,
    #[serde(default)]
    net: Option<NetSection>,
    #[serde(default)]
    fs: Option<FsSection>,
    #[serde(default)]
    capture: Option<CaptureSection>,
    #[serde(default)]
    replay: Option<ReplaySection>,
    #[serde(default)]
    cors: Option<CorsSection>,
    #[serde(default)]
    security_headers: Option<SecurityHeadersSection>,
    #[serde(default)]
    csrf: Option<CsrfSection>,
    #[serde(default)]
    auth: Option<AuthSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicySection {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    env: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectsSection {
    #[serde(default)]
    forbid: Option<Vec<String>>,
    #[serde(default)]
    require_explicit_on_function_fields: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SecretsSection {
    #[serde(default)]
    forbid_reveal: Option<bool>,
    #[serde(default)]
    allow_redact: Option<bool>,
    #[serde(default)]
    forbid_formatting: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonSection {
    #[serde(default)]
    max_bytes: Option<i64>,
    #[serde(default)]
    max_depth: Option<i64>,
    #[serde(default)]
    require_schema_for_encode: Option<bool>,
    #[serde(default)]
    forbid_secret_fields_in_schema: Option<bool>,
    #[serde(default)]
    max_string_bytes: Option<i64>,
    #[serde(default)]
    max_container_entries: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HttpSection {
    #[serde(default)]
    max_body_bytes: Option<i64>,
    #[serde(default)]
    max_concurrency: Option<i64>,
    #[serde(default)]
    default_timeout_ms: Option<i64>,
    #[serde(default)]
    max_header_bytes: Option<i64>,
    #[serde(default)]
    max_multipart_bytes: Option<i64>,
    #[serde(default)]
    success_envelope: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BudgetSection {
    #[serde(default)]
    max_body_bytes: Option<i64>,
    #[serde(default)]
    max_json_bytes: Option<i64>,
    #[serde(default)]
    max_json_depth: Option<i64>,
    #[serde(default)]
    max_deadline_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LoggingSection {
    #[serde(default)]
    structured_only: Option<bool>,
    #[serde(default)]
    include_remote_ip: Option<bool>,
    #[serde(default)]
    include_user_agent: Option<bool>,
    #[serde(default)]
    include_auth_user_id: Option<bool>,
    #[serde(default)]
    max_attr_depth: Option<i64>,
    #[serde(default)]
    max_attr_bytes: Option<i64>,
    #[serde(default)]
    sampling: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SqlSection {
    #[serde(default)]
    forbid_raw: Option<bool>,
    #[serde(default)]
    require_limit_on_select: Option<String>,
    #[serde(default)]
    max_returned_rows: Option<i64>,
    #[serde(default)]
    default_timeout_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NetSection {
    #[serde(default)]
    public: Option<NetPublicSection>,
    #[serde(default)]
    internal: Option<NetInternalSection>,
    #[serde(default)]
    ssrf: Option<NetSsrfSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NetPublicSection {
    #[serde(default)]
    allowed_schemes: Option<Vec<String>>,
    #[serde(default)]
    allow_redirects: Option<bool>,
    #[serde(default)]
    max_redirects: Option<i64>,
    #[serde(default)]
    allowed_domains: Option<Vec<String>>,
    #[serde(default)]
    blocked_domains: Option<Vec<String>>,
    #[serde(default)]
    allowed_ports: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NetInternalSection {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    allowed_cidrs: Option<Vec<String>>,
    #[serde(default)]
    allowed_domains: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NetSsrfSection {
    #[serde(default)]
    block_private_ranges: Option<bool>,
    #[serde(default)]
    block_loopback: Option<bool>,
    #[serde(default)]
    block_link_local: Option<bool>,
    #[serde(default)]
    block_metadata_ips: Option<bool>,
    #[serde(default)]
    resolve_dns: Option<bool>,
    #[serde(default)]
    revalidate_redirects: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FsSection {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    allowed_base_paths: Option<Vec<String>>,
    #[serde(default)]
    forbid_absolute_paths: Option<bool>,
    #[serde(default)]
    forbid_symlinks: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureSection {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    sample_rate: Option<f64>,
    #[serde(default)]
    max_body_bytes: Option<i64>,
    #[serde(default)]
    max_capture_bytes: Option<i64>,
    #[serde(default)]
    redact_headers: Option<Vec<String>>,
    #[serde(default)]
    redact_json_paths: Option<Vec<String>>,
    #[serde(default)]
    hash_only_non_json_bodies: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplaySection {
    #[serde(default)]
    effects: Option<String>,
    #[serde(default)]
    allow_policy_mismatch: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorsSection {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    allowed_origins: Option<Vec<String>>,
    #[serde(default)]
    allowed_methods: Option<Vec<String>>,
    #[serde(default)]
    allowed_headers: Option<Vec<String>>,
    #[serde(default)]
    exposed_headers: Option<Vec<String>>,
    #[serde(default)]
    allow_credentials: Option<bool>,
    #[serde(default)]
    allow_private_network: Option<bool>,
    #[serde(default)]
    reflect_origin: Option<bool>,
    #[serde(default)]
    max_age_seconds: Option<i64>,
    #[serde(default)]
    forbid_any_origin: Option<bool>,
    #[serde(default)]
    forbid_reflect_origin: Option<bool>,
    #[serde(default)]
    require_vary_origin: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SecurityHeadersSection {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    strip_server_header: Option<bool>,
    #[serde(default)]
    x_content_type_options: Option<bool>,
    #[serde(default)]
    x_frame_options: Option<String>,
    #[serde(default)]
    referrer_policy: Option<String>,
    #[serde(default)]
    hsts: Option<SecurityHeadersHstsSection>,
    #[serde(default)]
    csp: Option<SecurityHeadersCspSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SecurityHeadersHstsSection {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    max_age_seconds: Option<i64>,
    #[serde(default)]
    include_subdomains: Option<bool>,
    #[serde(default)]
    preload: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SecurityHeadersCspSection {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    report_only: Option<bool>,
    #[serde(default)]
    policy: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CsrfSection {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    cookie_name: Option<String>,
    #[serde(default)]
    header_name: Option<String>,
    #[serde(default)]
    same_site: Option<String>,
    #[serde(default)]
    secure_cookie: Option<bool>,
    #[serde(default)]
    http_only_cookie: Option<bool>,
    #[serde(default)]
    protected_methods: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthSection {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    cross_site_frontend: Option<bool>,
    #[serde(default)]
    cookie: Option<AuthCookieSection>,
    #[serde(default)]
    token: Option<AuthTokenSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthCookieSection {
    #[serde(default)]
    cookie_name: Option<String>,
    #[serde(default)]
    same_site: Option<String>,
    #[serde(default)]
    secure: Option<bool>,
    #[serde(default)]
    http_only: Option<bool>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthTokenSection {
    #[serde(default)]
    header_name: Option<String>,
    #[serde(default)]
    scheme: Option<String>,
}

pub fn load_policy(project_root: &Path) -> Result<Policy, Vec<Diagnostic>> {
    let policy_path = project_root.join(POLICY_FILE_NAME);
    if !policy_path.exists() {
        return Ok(Policy::default());
    }

    let source = fs::read_to_string(&policy_path).map_err(|err| {
        vec![Diagnostic::error(
            "P6001",
            "could not read policy file",
            Span::point(policy_path.clone(), 1, 1),
        )
        .with_note(err.to_string())]
    })?;

    parse_policy_str(&policy_path, &source)
}

pub fn parse_policy_str(policy_path: &Path, source: &str) -> Result<Policy, Vec<Diagnostic>> {
    let raw: PolicyFile = toml::from_str(source).map_err(|err| {
        vec![Diagnostic::error(
            "P6002",
            "failed to parse policy file",
            Span::point(policy_path.to_path_buf(), 1, 1),
        )
        .with_note(err.to_string())]
    })?;

    build_policy(policy_path, raw)
}

fn build_policy(policy_path: &Path, raw: PolicyFile) -> Result<Policy, Vec<Diagnostic>> {
    let mut policy = Policy::default();
    let mut diagnostics = Vec::new();

    if let Some(section) = raw.policy {
        if let Some(name) = section.name {
            policy.name = name;
        }
        if let Some(version) = section.version {
            policy.version = version;
        }
        if let Some(mode) = section.mode {
            policy.mode = match mode.as_str() {
                "enforce" => PolicyMode::Enforce,
                "warn" => PolicyMode::Warn,
                _ => {
                    diagnostics.push(
                        Diagnostic::error(
                            "P6003",
                            "invalid policy mode",
                            Span::point(policy_path.to_path_buf(), 1, 1),
                        )
                        .with_note("policy.mode must be `enforce` or `warn`"),
                    );
                    PolicyMode::Enforce
                }
            };
        }
        if let Some(env) = section.env {
            match env.as_str() {
                "dev" | "staging" | "prod" => policy.env = env,
                _ => diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid policy environment",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("policy.env must be `dev`, `staging`, or `prod`"),
                ),
            }
        }
    }

    if let Some(section) = raw.effects {
        if let Some(forbid) = section.forbid {
            policy.forbidden_effects = forbid.into_iter().collect();
        }
    }

    if let Some(section) = raw.logging {
        if let Some(structured_only) = section.structured_only {
            policy.logging.structured_only = structured_only;
        }
        if let Some(include_remote_ip) = section.include_remote_ip {
            policy.logging.include_remote_ip = include_remote_ip;
        }
        if let Some(include_user_agent) = section.include_user_agent {
            policy.logging.include_user_agent = include_user_agent;
        }
    }

    if let Some(section) = raw.sql {
        if let Some(forbid_raw) = section.forbid_raw {
            policy.sql.forbid_raw = forbid_raw;
        }
        if let Some(require_limit_on_select) = section.require_limit_on_select {
            match require_limit_on_select.as_str() {
                "off" | "warn" | "enforce" => {
                    policy.sql.require_limit_on_select = require_limit_on_select;
                }
                _ => diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid sql.require_limit_on_select",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("sql.require_limit_on_select must be `off`, `warn`, or `enforce`"),
                ),
            }
        }
    }

    if let Some(section) = raw.http {
        if let Some(max_body_bytes) = section.max_body_bytes {
            if max_body_bytes < 1 {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid http.max_body_bytes",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("http.max_body_bytes must be >= 1"),
                );
            } else {
                policy.http.max_body_bytes = max_body_bytes;
            }
        }

        if let Some(max_concurrency) = section.max_concurrency {
            if max_concurrency < 1 {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid http.max_concurrency",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("http.max_concurrency must be >= 1"),
                );
            } else {
                policy.http.max_concurrency = max_concurrency;
            }
        }

        if let Some(max_header_bytes) = section.max_header_bytes {
            if max_header_bytes < 1 {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid http.max_header_bytes",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("http.max_header_bytes must be >= 1"),
                );
            } else {
                policy.http.max_header_bytes = max_header_bytes;
            }
        }

        if let Some(max_multipart_bytes) = section.max_multipart_bytes {
            if max_multipart_bytes < 1 {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid http.max_multipart_bytes",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("http.max_multipart_bytes must be >= 1"),
                );
            } else {
                policy.http.max_multipart_bytes = max_multipart_bytes;
            }
        }

        if let Some(default_timeout_ms) = section.default_timeout_ms {
            if default_timeout_ms < 1 {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid http.default_timeout_ms",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("http.default_timeout_ms must be >= 1"),
                );
            } else {
                policy.http.default_timeout_ms = default_timeout_ms;
            }
        }
    }

    if let Some(section) = raw.cors {
        if let Some(value) = section.enabled {
            policy.cors.enabled = value;
        }
        if let Some(value) = section.allowed_origins {
            policy.cors.allowed_origins = value;
        }
        if let Some(value) = section.allowed_methods {
            policy.cors.allowed_methods = value;
        }
        if let Some(value) = section.allowed_headers {
            policy.cors.allowed_headers = value;
        }
        if let Some(value) = section.exposed_headers {
            policy.cors.exposed_headers = value;
        }
        if let Some(value) = section.max_age_seconds {
            if value <= 0 {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid cors.max_age_seconds",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("cors.max_age_seconds must be >= 1"),
                );
            } else {
                policy.cors.max_age_seconds = value;
            }
        }
        if let Some(value) = section.allow_credentials {
            policy.cors.allow_credentials = value;
        }
        if let Some(value) = section.allow_private_network {
            policy.cors.allow_private_network = value;
        }
        if let Some(value) = section.reflect_origin {
            policy.cors.reflect_origin = value;
        }
        if let Some(value) = section.forbid_any_origin {
            policy.cors_forbid_any_origin = value;
            policy.cors.forbid_any_origin = value;
        }
        if let Some(value) = section.forbid_reflect_origin {
            policy.cors_forbid_reflect_origin = value;
            policy.cors.forbid_reflect_origin = value;
        }
        if let Some(value) = section.require_vary_origin {
            policy.cors_require_vary_origin = value;
            policy.cors.require_vary_origin = value;
        }

        if policy.cors.allow_credentials
            && policy
                .cors
                .allowed_origins
                .iter()
                .any(|origin| origin == "*")
        {
            diagnostics.push(
                Diagnostic::error(
                    "P6003",
                    "invalid CORS policy: wildcard origin cannot be used with credentials",
                    Span::point(policy_path.to_path_buf(), 1, 1),
                )
                .with_note("set explicit allowed origins when `cors.allow_credentials=true`"),
            );
        }

        if policy.cors_forbid_any_origin
            && policy
                .cors
                .allowed_origins
                .iter()
                .any(|origin| origin == "*")
        {
            diagnostics.push(
                Diagnostic::error(
                    "P6003",
                    "invalid CORS policy: wildcard origin is forbidden",
                    Span::point(policy_path.to_path_buf(), 1, 1),
                )
                .with_note("replace `*` with explicit origins or disable `cors.forbid_any_origin`"),
            );
        }

        if policy.cors_forbid_reflect_origin && policy.cors.reflect_origin {
            diagnostics.push(
                Diagnostic::error(
                    "P6003",
                    "invalid CORS policy: origin reflection is forbidden",
                    Span::point(policy_path.to_path_buf(), 1, 1),
                )
                .with_note(
                    "set `cors.reflect_origin=false` or disable `cors.forbid_reflect_origin`",
                ),
            );
        }
    }

    if let Some(net) = raw.net {
        if let Some(public) = net.public {
            if let Some(allow_redirects) = public.allow_redirects {
                policy.net_public.allow_redirects = allow_redirects;
            }
            if let Some(max_redirects) = public.max_redirects {
                if max_redirects < 0 {
                    diagnostics.push(
                        Diagnostic::error(
                            "P6003",
                            "invalid net.public.max_redirects",
                            Span::point(policy_path.to_path_buf(), 1, 1),
                        )
                        .with_note("net.public.max_redirects must be >= 0"),
                    );
                } else {
                    policy.net_public.max_redirects = max_redirects;
                }
            }
            if let Some(allowed_domains) = public.allowed_domains {
                policy.net_public.allowed_domains = allowed_domains;
            }
            if let Some(blocked_domains) = public.blocked_domains {
                policy.net_public.blocked_domains = blocked_domains;
            }
            if let Some(allowed_ports) = public.allowed_ports {
                let invalid_ports = allowed_ports
                    .iter()
                    .copied()
                    .filter(|port| !(1..=65535).contains(port))
                    .collect::<Vec<_>>();
                if invalid_ports.is_empty() {
                    policy.net_public.allowed_ports =
                        allowed_ports.into_iter().map(|port| port as u16).collect();
                } else {
                    diagnostics.push(
                        Diagnostic::error(
                            "P6003",
                            "invalid net.public.allowed_ports",
                            Span::point(policy_path.to_path_buf(), 1, 1),
                        )
                        .with_note(format!(
                            "allowed ports must be in range 1..65535 (invalid: {})",
                            invalid_ports
                                .iter()
                                .map(|port| port.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                    );
                }
            }
            if let Some(allowed_schemes) = public.allowed_schemes {
                if allowed_schemes.is_empty()
                    || !allowed_schemes
                        .iter()
                        .all(|item| item == "http" || item == "https")
                {
                    diagnostics.push(
                        Diagnostic::error(
                            "P6003",
                            "invalid net.public.allowed_schemes",
                            Span::point(policy_path.to_path_buf(), 1, 1),
                        )
                        .with_note(
                            "allowed schemes must be non-empty and only include `http` or `https`",
                        ),
                    );
                } else {
                    policy.net_public.allowed_schemes = allowed_schemes;
                }
            }

            if public.allow_redirects == Some(false) && public.max_redirects.unwrap_or(0) > 0 {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid net.public redirect settings",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("set max_redirects=0 when allow_redirects=false"),
                );
            }
        }

        if let Some(internal) = net.internal {
            if let Some(enabled) = internal.enabled {
                policy.net_internal.enabled = enabled;
            }
            if let Some(allowed_cidrs) = internal.allowed_cidrs {
                policy.net_internal.allowed_cidrs = allowed_cidrs;
            }
            if let Some(allowed_domains) = internal.allowed_domains {
                policy.net_internal.allowed_domains = allowed_domains;
            }
        }

        if let Some(ssrf) = net.ssrf {
            if let Some(block_private_ranges) = ssrf.block_private_ranges {
                policy.net_ssrf.block_private_ranges = block_private_ranges;
            }
            if let Some(block_loopback) = ssrf.block_loopback {
                policy.net_ssrf.block_loopback = block_loopback;
            }
            if let Some(block_link_local) = ssrf.block_link_local {
                policy.net_ssrf.block_link_local = block_link_local;
            }
            if let Some(block_metadata_ips) = ssrf.block_metadata_ips {
                policy.net_ssrf.block_metadata_ips = block_metadata_ips;
            }
            if let Some(revalidate_redirects) = ssrf.revalidate_redirects {
                policy.net_ssrf.revalidate_redirects = revalidate_redirects;
            }
            if let Some(resolve_dns) = ssrf.resolve_dns {
                policy.net_ssrf.resolve_dns = resolve_dns;
            }
        }
    }

    if let Some(json) = raw.json {
        if let Some(require_schema_for_encode) = json.require_schema_for_encode {
            policy.json.require_schema_for_encode = require_schema_for_encode;
        }
        if let Some(max_bytes) = json.max_bytes {
            if max_bytes < 1 {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid json.max_bytes",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("json.max_bytes must be >= 1"),
                );
            } else {
                policy.json.max_bytes = max_bytes;
            }
        }
        if let Some(max_depth) = json.max_depth {
            if max_depth < 1 {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid json.max_depth",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("json.max_depth must be >= 1"),
                );
            } else {
                policy.json.max_depth = max_depth;
            }
        }
    }

    if let Some(fs) = raw.fs {
        if let Some(enabled) = fs.enabled {
            policy.fs.enabled = enabled;
        }
        if let Some(allowed_base_paths) = fs.allowed_base_paths {
            policy.fs.allowed_base_paths = allowed_base_paths;
        }
        if let Some(forbid_symlinks) = fs.forbid_symlinks {
            match forbid_symlinks.as_str() {
                "off" | "warn" | "enforce" => policy.fs.forbid_symlinks = forbid_symlinks,
                _ => diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid fs.forbid_symlinks",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("fs.forbid_symlinks must be `off`, `warn`, or `enforce`"),
                ),
            }
        }
    }

    if let Some(capture) = raw.capture {
        if let Some(mode) = capture.mode {
            policy.capture.mode = mode;
        }
        if let Some(redact_headers) = capture.redact_headers {
            policy.capture.redact_headers = redact_headers;
        }
    }

    if let Some(replay) = raw.replay {
        if let Some(effects) = replay.effects {
            policy.replay.effects = effects;
        }
    }

    if let Some(section) = raw.security_headers {
        if let Some(enabled) = section.enabled {
            policy.security_headers.enabled = enabled;
        }
        if let Some(x_content_type_options) = section.x_content_type_options {
            policy.security_headers.x_content_type_options = x_content_type_options;
        }
        if let Some(x_frame_options) = section.x_frame_options {
            match x_frame_options.as_str() {
                "DENY" | "SAMEORIGIN" => policy.security_headers.x_frame_options = x_frame_options,
                _ => diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid security_headers.x_frame_options",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("x_frame_options must be `DENY` or `SAMEORIGIN`"),
                ),
            }
        }
        if let Some(referrer_policy) = section.referrer_policy {
            policy.security_headers.referrer_policy = referrer_policy;
        }

        if let Some(hsts) = section.hsts {
            if let Some(enabled) = hsts.enabled {
                policy.security_headers.hsts_enabled = enabled;
            }
            if let Some(max_age_seconds) = hsts.max_age_seconds {
                if max_age_seconds < 0 {
                    diagnostics.push(
                        Diagnostic::error(
                            "P6003",
                            "invalid security_headers.hsts.max_age_seconds",
                            Span::point(policy_path.to_path_buf(), 1, 1),
                        )
                        .with_note("security_headers.hsts.max_age_seconds must be >= 0"),
                    );
                } else {
                    policy.security_headers.hsts_max_age_seconds = max_age_seconds;
                }
            }
            if let Some(include_subdomains) = hsts.include_subdomains {
                policy.security_headers.hsts_include_subdomains = include_subdomains;
            }
            if let Some(preload) = hsts.preload {
                policy.security_headers.hsts_preload = preload;
            }
            if policy.security_headers.hsts_enabled
                && policy.security_headers.hsts_max_age_seconds <= 0
            {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid security_headers.hsts.max_age_seconds",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note(
                        "security_headers.hsts.max_age_seconds must be >= 1 when HSTS is enabled",
                    ),
                );
            }
        }

        if let Some(csp) = section.csp {
            if let Some(enabled) = csp.enabled {
                policy.security_headers.csp_enabled = enabled;
            }
            if let Some(report_only) = csp.report_only {
                policy.security_headers.csp_report_only = report_only;
            }
            if let Some(csp_policy) = csp.policy {
                if csp_policy.trim().is_empty() {
                    diagnostics.push(
                        Diagnostic::error(
                            "P6003",
                            "invalid security_headers.csp.policy",
                            Span::point(policy_path.to_path_buf(), 1, 1),
                        )
                        .with_note("security_headers.csp.policy must be a non-empty string"),
                    );
                } else {
                    policy.security_headers.csp_policy = csp_policy;
                }
            }
        }
    }

    if let Some(section) = raw.csrf {
        if let Some(enabled) = section.enabled {
            policy.csrf.enabled = enabled;
        }
        if let Some(mode) = section.mode {
            match mode.as_str() {
                "off" | "double_submit" | "synchronizer_token" => policy.csrf.mode = mode,
                _ => diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid csrf.mode",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("csrf.mode must be `off`, `double_submit`, or `synchronizer_token`"),
                ),
            }
        }
        if let Some(cookie_name) = section.cookie_name {
            if cookie_name.trim().is_empty() {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid csrf.cookie_name",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("csrf.cookie_name must be a non-empty string"),
                );
            } else {
                policy.csrf.cookie_name = cookie_name;
            }
        }
        if let Some(header_name) = section.header_name {
            if header_name.trim().is_empty() {
                diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid csrf.header_name",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("csrf.header_name must be a non-empty string"),
                );
            } else {
                policy.csrf.header_name = header_name;
            }
        }
        if let Some(same_site) = section.same_site {
            match same_site.as_str() {
                "Lax" | "Strict" | "None" => policy.csrf.same_site = same_site,
                _ => diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid csrf.same_site",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("csrf.same_site must be `Lax`, `Strict`, or `None`"),
                ),
            }
        }
        if let Some(secure_cookie) = section.secure_cookie {
            policy.csrf.secure_cookie = secure_cookie;
        }
        if let Some(methods) = section.protected_methods {
            policy.csrf.protected_methods = methods;
        }

        if policy.csrf.same_site == "None" && !policy.csrf.secure_cookie {
            diagnostics.push(
                Diagnostic::error(
                    "P6003",
                    "invalid csrf policy: SameSite=None requires secure_cookie=true",
                    Span::point(policy_path.to_path_buf(), 1, 1),
                )
                .with_note("set csrf.secure_cookie=true when csrf.same_site=\"None\""),
            );
        }
    }

    if let Some(section) = raw.auth {
        if let Some(mode) = section.mode {
            match mode.as_str() {
                "token" | "cookie" | "mixed" => policy.auth.mode = mode,
                _ => diagnostics.push(
                    Diagnostic::error(
                        "P6003",
                        "invalid auth.mode",
                        Span::point(policy_path.to_path_buf(), 1, 1),
                    )
                    .with_note("auth.mode must be `token`, `cookie`, or `mixed`"),
                ),
            }
        }
        if let Some(cross_site_frontend) = section.cross_site_frontend {
            policy.auth.cross_site_frontend = cross_site_frontend;
        }

        if let Some(cookie) = section.cookie {
            if let Some(cookie_name) = cookie.cookie_name {
                if cookie_name.trim().is_empty() {
                    diagnostics.push(
                        Diagnostic::error(
                            "P6003",
                            "invalid auth.cookie.cookie_name",
                            Span::point(policy_path.to_path_buf(), 1, 1),
                        )
                        .with_note("auth.cookie.cookie_name must be a non-empty string"),
                    );
                } else {
                    policy.auth.cookie_name = cookie_name;
                }
            }
            if let Some(same_site) = cookie.same_site {
                if same_site == "None" && cookie.secure == Some(false) {
                    diagnostics.push(
                        Diagnostic::error(
                            "P6003",
                            "invalid auth.cookie policy: SameSite=None requires secure=true",
                            Span::point(policy_path.to_path_buf(), 1, 1),
                        )
                        .with_note(
                            "set auth.cookie.secure=true when auth.cookie.same_site=\"None\"",
                        ),
                    );
                }
            }
        }
    }

    if matches!(policy.auth.mode.as_str(), "cookie" | "mixed") && !policy.csrf.enabled {
        diagnostics.push(
            Diagnostic::error(
                "P6003",
                "invalid auth/csrf policy: csrf.enabled is required for cookie or mixed auth",
                Span::point(policy_path.to_path_buf(), 1, 1),
            )
            .with_note("set csrf.enabled=true or switch auth.mode=token"),
        );
    }

    if policy.auth.cross_site_frontend && matches!(policy.auth.mode.as_str(), "cookie" | "mixed") {
        if !policy.cors.allow_credentials {
            diagnostics.push(
                Diagnostic::error(
                    "P6003",
                    "invalid auth/cors policy: cross-site cookie auth requires cors.allow_credentials=true",
                    Span::point(policy_path.to_path_buf(), 1, 1),
                )
                .with_note("set cors.allow_credentials=true for cross-site cookie auth"),
            );
        }
        if policy.cors.has_wildcard_origin() {
            diagnostics.push(
                Diagnostic::error(
                    "P6003",
                    "invalid auth/cors policy: cross-site cookie auth forbids wildcard origin",
                    Span::point(policy_path.to_path_buf(), 1, 1),
                )
                .with_note("set explicit cors.allowed_origins for cross-site cookie auth"),
            );
        }
    }

    if diagnostics.is_empty() {
        Ok(policy)
    } else {
        Err(diagnostics)
    }
}
