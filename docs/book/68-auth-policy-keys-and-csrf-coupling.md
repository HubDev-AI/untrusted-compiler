# 68 Auth Policy Keys and CSRF Coupling (v0)

This chapter makes CSRF posture deterministic by introducing explicit auth policy mode.

## 1) Required auth policy keys

```toml
[auth]
mode = "token"  # "token" | "cookie" | "mixed"
cross_site_frontend = false
```

Meaning:
- `token`: header-based auth, CSRF usually not required
- `cookie`: cookie/session auth, CSRF required
- `mixed`: both modes, CSRF required for cookie-authenticated requests

## 2) Optional auth sub-keys

### Cookie settings

```toml
[auth.cookie]
cookie_name = "session"
same_site = "Lax"
secure = true
http_only = true
domain = ""
path = "/"
```

### Token settings

```toml
[auth.token]
header_name = "authorization"
scheme = "bearer"
```

## 3) Deterministic enforcement rules

### Cookie or mixed auth
- `csrf.enabled=true` required (unless explicit audited allowlist)
- `csrf.protected_methods` must include unsafe methods

### Cookie auth with cross-site frontend
- `cors.allow_credentials=true` required
- wildcard CORS origins forbidden
- `SameSite=None` requires secure cookie

### Token-only auth
- `csrf.enabled=false` is acceptable
- CORS remains policy-governed independently

## 4) sec.audit finding updates
- `CSRF_REQUIRED_BUT_DISABLED` (HIGH)
- `COOKIE_CROSS_SITE_WITHOUT_CORS_CREDS` (MEDIUM)
- `COOKIE_CROSS_SITE_WITH_WILDCARD_ORIGIN` (HIGH)
- `COOKIE_SAMESITE_NONE_WITHOUT_SECURE` (MEDIUM/HIGH)

## 5) Minimal start
If policy footprint must stay small:

```toml
[auth]
mode = "token"
cross_site_frontend = false
```

This alone removes guesswork from CSRF findings.
