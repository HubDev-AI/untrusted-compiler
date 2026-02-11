# 66 Deterministic Severity Mapping for sec.audit (v0)

This chapter defines rule-based severity mapping from policy state + metadata tags + allowlist state to stable findings.

## 1) Severity levels
- LOW
- MEDIUM
- HIGH
- CRITICAL

## 2) Inputs
- effective policy state
- tag inventory from `security_map.json`
- allowlist entries with expiry

Priority order:
1. CRITICAL rules
2. HIGH rules
3. MEDIUM rules
4. LOW rules

## 3) Finding format
Each finding includes:
- `id`
- `severity`
- `category`
- `evidence`
- `locations` (optional)
- `suggestion`

## 4) Core mapping table

### CORS
- `CORS_CREDENTIALS_WITH_WILDCARD` -> CRITICAL
- `CORS_REFLECT_ORIGIN_ENABLED` -> HIGH
- `CORS_ANY_ORIGIN` -> MEDIUM
- `CORS_VARY_ORIGIN_MISSING` -> LOW

### Security headers
- `CSP_DISABLED` -> HIGH
- `CSP_REPORT_ONLY` -> MEDIUM in prod, LOW in dev
- `HSTS_DISABLED_IN_PROD` -> MEDIUM
- `XFO_DISABLED` / `NOSNIFF_DISABLED` / weak referrer policy -> LOW

### CSRF
- `CSRF_REQUIRED_BUT_DISABLED` -> HIGH
- `COOKIE_SAMESITE_NONE_WITHOUT_SECURE` -> MEDIUM or HIGH (policy-driven)
- `CSRF_PROTECTED_METHODS_INCOMPLETE` -> LOW

### Secrets
- `SECRETS_REVEAL_USED` -> CRITICAL when policy forbids
- `SECRETS_REVEAL_ALLOWLISTED` -> HIGH

### SSRF and egress
- `INTERNAL_NET_ENABLED_NO_ALLOWLIST` -> CRITICAL
- `PUBLIC_REDIRECTS_ENABLED_WITHOUT_REVALIDATION` -> HIGH
- `DNS_RESOLUTION_DISABLED` -> HIGH in production
- `PUBLIC_EGRESS_NO_DOMAIN_POLICY` -> MEDIUM (policy stance)
- `INTERNAL_NET_CALL_ALLOWLISTED` -> HIGH

### Filesystem
- `FS_ENABLED_NO_BASE_ALLOWLIST` -> HIGH
- `SYMLINK_POLICY_WEAK` -> MEDIUM

### Capture and replay
- `CAPTURE_ALL_IN_PROD` -> HIGH
- `CAPTURE_REDACTION_INCOMPLETE` -> HIGH
- `REPLAY_EFFECTS_ALLOW` -> MEDIUM (or HIGH outside controlled env)

### SQL hygiene
- `SQL_SELECT_WITHOUT_LIMIT` -> MEDIUM for warn policy; HIGH for enforced+allowlisted risk

### Exception hygiene
- `ALLOW_EXPIRED` -> HIGH
- `ALLOW_EXPIRING_SOON` -> MEDIUM
- `ALLOW_COUNT_HIGH` -> LOW

## 5) Allowlist impact
- Allowlists do not silence CRITICAL by default.
- Allowlists produce explicit evidence and call-site findings.
- Any severity reduction via allowlist must be policy-driven and explicit.

## 6) Deterministic riskScore
Recommended weights:
- LOW = 1
- MEDIUM = 3
- HIGH = 7
- CRITICAL = 15

`riskScore = sum(weight by finding)`.

## 7) Environment mode handling
If policy includes `env.mode`:
- production mode raises severity for CSP/HSTS/capture/replay misconfigurations
- development mode can downgrade specific posture-only findings

If no mode exists, treat as production for safety.
