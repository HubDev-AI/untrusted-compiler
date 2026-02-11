# 67 sec.audit Examples and Policy Profiles

This chapter provides baseline example outputs and aligned policy profiles for deterministic audit behavior.

## 1) Example posture A: default-secure production

Expected posture:
- explicit allowlist CORS with credentials
- security headers enabled (HSTS, CSP enforce, XFO, nosniff)
- CSRF enabled for cookie auth
- capture mode errors-only with required redactions
- replay effects deny

Expected result:
- no findings
- risk score 0

## 2) Example posture B: permissive development

Typical intentionally weaker posture:
- wildcard CORS (without credentials)
- redirects enabled without revalidation
- CSP report-only
- CSRF disabled
- capture mode all with incomplete redaction
- replay effects allow
- internal net enabled without allowlist

Expected result:
- mixed LOW/MEDIUM/HIGH findings
- at least one CRITICAL when internal net has no allowlist

## 3) Policy profile A (`default-secure-prod`)

Reference profile requirements:
- `policy.mode = "enforce"`
- `policy.env = "prod"`
- `effects.forbid` includes `shell`, `unsafe`, `secrets.reveal`
- `net.internal.enabled = false`
- `net.public.allow_redirects = false`
- `security_headers.csp.enabled = true` and enforce in prod
- `cors.allow_credentials = true` only with explicit origin allowlist
- `csrf.enabled = true` for cookie-auth posture
- capture redaction must include auth and cookie headers
- replay effects must be deny

## 4) Policy profile B (`permissive-dev`)

Reference profile characteristics:
- `policy.mode = "warn"`
- `policy.env = "dev"`
- development-only loosening may be allowed for CSP/capture/replay
- risky toggles should still be surfaced by audit findings
- any production-equivalent critical misconfiguration remains critical

## 5) Profile guidance
- Keep prod profile strict by default.
- Allow dev profile flexibility but require explicit risk visibility.
- Run `sec.audit --format json` in CI for both profiles.
- Gate production release with `--fail-on risk>=HIGH` unless explicit security approval exists.
