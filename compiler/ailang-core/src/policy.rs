#![allow(dead_code)]

use crate::diagnostics::{Diagnostic, Span};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub const POLICY_FILE_NAME: &str = "ailang.policy";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyMode {
    Enforce,
    Warn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub name: String,
    pub version: String,
    pub mode: PolicyMode,
    pub forbidden_effects: HashSet<String>,
    pub cors_forbid_any_origin: bool,
    pub cors_forbid_reflect_origin: bool,
    pub cors_require_vary_origin: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            name: "default-secure".to_string(),
            version: "0.1".to_string(),
            mode: PolicyMode::Enforce,
            forbidden_effects: ["shell", "unsafe", "secrets.reveal"]
                .into_iter()
                .map(|item| item.to_string())
                .collect(),
            cors_forbid_any_origin: true,
            cors_forbid_reflect_origin: true,
            cors_require_vary_origin: true,
        }
    }
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
    max_age_seconds: Option<i64>,
    #[serde(default)]
    forbid_any_origin: Option<bool>,
    #[serde(default)]
    forbid_reflect_origin: Option<bool>,
    #[serde(default)]
    require_vary_origin: Option<bool>,
}

pub fn load_policy(project_root: &Path) -> Result<Policy, Vec<Diagnostic>> {
    let policy_path = project_root.join(POLICY_FILE_NAME);
    if !policy_path.exists() {
        return Ok(Policy::default());
    }

    let source = fs::read_to_string(&policy_path).map_err(|err| {
        vec![
            Diagnostic::error(
                "P6001",
                "could not read policy file",
                Span::point(policy_path.clone(), 1, 1),
            )
            .with_note(err.to_string()),
        ]
    })?;

    parse_policy_str(&policy_path, &source)
}

pub fn parse_policy_str(policy_path: &Path, source: &str) -> Result<Policy, Vec<Diagnostic>> {
    let raw: PolicyFile = toml::from_str(source).map_err(|err| {
        vec![
            Diagnostic::error(
                "P6002",
                "failed to parse policy file",
                Span::point(policy_path.to_path_buf(), 1, 1),
            )
            .with_note(err.to_string()),
        ]
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
    }

    if let Some(section) = raw.effects {
        if let Some(forbid) = section.forbid {
            policy.forbidden_effects = forbid.into_iter().collect();
        }
    }

    if let Some(section) = raw.cors {
        if let Some(value) = section.forbid_any_origin {
            policy.cors_forbid_any_origin = value;
        }
        if let Some(value) = section.forbid_reflect_origin {
            policy.cors_forbid_reflect_origin = value;
        }
        if let Some(value) = section.require_vary_origin {
            policy.cors_require_vary_origin = value;
        }

        if section.allow_credentials.unwrap_or(false)
            && section
                .allowed_origins
                .as_ref()
                .is_some_and(|origins| origins.iter().any(|item| item == "*"))
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
            && section
                .allowed_origins
                .as_ref()
                .is_some_and(|origins| origins.iter().any(|item| item == "*"))
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
    }

    if let Some(net) = raw.net {
        if let Some(public) = net.public {
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
                        .with_note("allowed schemes must be non-empty and only include `http` or `https`"),
                    );
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
    }

    if let Some(json) = raw.json {
        if json.max_depth.unwrap_or(1) < 1 {
            diagnostics.push(
                Diagnostic::error(
                    "P6003",
                    "invalid json.max_depth",
                    Span::point(policy_path.to_path_buf(), 1, 1),
                )
                .with_note("json.max_depth must be >= 1"),
            );
        }
    }

    if diagnostics.is_empty() {
        Ok(policy)
    } else {
        Err(diagnostics)
    }
}
