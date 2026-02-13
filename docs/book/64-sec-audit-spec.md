# 64 sec4 audit Spec (Security Posture Report) v0

`sec4 audit` provides a deterministic security posture report for CI and human review.

## 1) Command and formats

CLI:
- `sec4 audit [--format text|json] [--fail-on risk>=HIGH]`

Output:
- `text`: posture summary + ranked findings
- `json`: machine-readable report for CI and dashboards

## 2) Required report sections

### 2.1 Build identity and policy
- `policy.name`, `policy.version`, `policyHash`
- `compilerHash`, `runtimeHash`
- report timestamp

### 2.2 Effective middleware posture
- CORS effective config
- Security headers effective config
- CSRF effective config
- capture/replay effective config

Effective means after defaults, policy clamps, and environment-specific overrides.

### 2.3 Attack surface flags
At minimum:
- raw SQL path exists?
- raw HTML path exists?
- `secrets.reveal` used?
- internal net enabled and used?
- filesystem enabled and constrained?
- capture safety posture

### 2.4 Exceptions / allowlist usage
For each `@allow(...)`:
- file/line/column
- policy key bypassed
- reason
- ticket
- expires
- risk rating

Hard rule:
- missing ticket or expiry is a compile error.

### 2.5 Findings and scoring
Each finding includes:
- stable id
- severity
- category
- evidence
- canonical suggestion

## 3) Core risk rules

### Critical baseline examples
- wildcard+credentials CORS
- `secrets.reveal` allowed/used without strict containment
- internal net enabled without allowlist
- capture enabled without auth/cookie redaction

### High baseline examples
- redirects enabled without SSRF redirect revalidation
- CSP disabled in production
- CSRF disabled for cookie-auth mode
- filesystem enabled without base-path allowlist

### Medium baseline examples
- HSTS disabled in production HTTPS
- capture all mode in production
- replay effects set to allow outside controlled environment

### Low baseline examples
- missing vary-origin in allowlist CORS mode
- exception count above hygiene threshold

## 4) Data sources
`sec4 audit` uses:
1. policy file
2. router bootstrap middleware usage
3. compiler metadata (tags, effects, allowlist annotations)
4. stdlib sink usage map
5. optional environment mode (`dev|staging|prod`)

No runtime execution required.

## 5) JSON output schema

```json
{
  "version": "0.1",
  "policy": { "name": "default-secure", "version": "0.1", "hash": "pol_...", "mode": "enforce" },
  "build": { "compilerHash": "cpl_...", "runtimeHash": "rt_...", "timeMs": 1760000000000 },
  "posture": { },
  "attackSurface": { },
  "exceptions": [ ],
  "findings": [ ],
  "summary": { "riskScore": 0, "highestSeverity": "LOW", "findingCounts": { "LOW": 0, "MEDIUM": 0, "HIGH": 0, "CRITICAL": 0 } }
}
```

## 6) Text output requirements
Text output must include:
- policy identity
- middleware posture summary
- attack surface summary
- exceptions with ticket+expiry
- finding list ordered by severity

## 7) CI integration
With `--fail-on risk>=HIGH`:
- exit non-zero if any finding at/above threshold
- exit non-zero for expired exceptions
- exit non-zero for malformed exception metadata

## 8) Minimal implementation path
1. parse policy into typed effective config
2. scan compiler metadata for middleware/sink/effect/allow tags
3. compute finding set via deterministic rule table
4. emit text and json formats
5. wire failure threshold to process exit code
