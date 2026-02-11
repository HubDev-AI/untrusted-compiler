# 63 Security Middleware Baseline (CORS, Security Headers, CSRF) v0

This chapter defines first-class web security posture in AILang runtime/stdlib using typed middleware, typed config, and policy enforcement.

## 1) Canonical router bootstrap

```ailang
function buildRouter(): Router {
  let r = http.router()

  r = sec.withSecurityHeaders(r, sec.defaultHeaders())
  r = cors.withCors(r, cors.fromPolicy())
  r = csrf.withCsrf(r, csrf.fromPolicy())

  r = routes.register(r)
  return r
}
```

Policy determines what is enabled; application bootstrap stays consistent.

## 2) CORS

### Types

```ailang
type Origin = opaque
type OriginPattern = opaque

type CorsOrigins =
  | { exact: List<Origin> }
  | { any: true }
  | { pattern: List<OriginPattern> }

type CorsConfig = {
  enabled: Bool
  allowedOrigins: CorsOrigins
  allowedMethods: List<HttpMethod>
  allowedHeaders: List<HeaderName>
  exposedHeaders?: List<HeaderName>
  allowCredentials: Bool
  maxAgeSeconds?: Int
}
```

### API

```ailang
module cors {
  fn origin(u: Untrusted<String>) -> Result<Origin, ValidationError>
  fn withCors(r: Router, cfg: CorsConfig) -> Router
  fn fromPolicy() -> CorsConfig
}
```

### Required rules
- If `allowCredentials=true`, wildcard origin is forbidden.
- Never reflect arbitrary request origin; only matched allowlist/pattern origins are allowed.
- Emit `Vary: Origin` whenever non-wildcard origin policy is used.

## 3) Security headers middleware

### Types

```ailang
type SecurityHeadersConfig = {
  enabled: Bool

  hsts?: { enabled: Bool, maxAgeSeconds: Int, includeSubDomains: Bool, preload: Bool }
  xContentTypeOptions?: Bool
  xFrameOptions?: "DENY" | "SAMEORIGIN"
  referrerPolicy?: String
  permissionsPolicy?: String
  csp?: CspConfig

  stripServerHeader?: Bool
}

type CspConfig = {
  enabled: Bool
  policy: CspPolicy
  reportOnly: Bool
}

type CspPolicy = opaque
```

### API

```ailang
module sec {
  fn defaultHeaders() -> SecurityHeadersConfig
  fn withSecurityHeaders(r: Router, cfg: SecurityHeadersConfig) -> Router

  fn csp() -> CspPolicy
  fn cspAdd(p: CspPolicy, directive: String, sources: List<String>) -> CspPolicy
}
```

### Secure defaults
`sec.defaultHeaders()` should enable:
- HSTS for HTTPS deployments (policy-gated in development)
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY` (or `SAMEORIGIN` when explicitly needed)
- `Referrer-Policy: strict-origin-when-cross-origin`
- CSP with conservative defaults (`reportOnly` optional in development)

## 4) CSRF middleware

CSRF is required for cookie/session auth and optional for token-only auth.

### Types

```ailang
type CsrfMode = "off" | "double_submit" | "synchronizer_token"

type CsrfConfig = {
  enabled: Bool
  mode: CsrfMode

  cookieName: String
  headerName: HeaderName
  paramName?: String

  sameSite: "Lax" | "Strict" | "None"
  secureCookie: Bool
  httpOnlyCookie: Bool

  protectedMethods: List<HttpMethod>
}
```

### API

```ailang
module csrf {
  fn withCsrf(r: Router, cfg: CsrfConfig) -> Router
  fn fromPolicy() -> CsrfConfig
  fn issueToken(ctx: Ctx) effects { net } -> Result<{ token: String }, HttpError>
}
```

### Required behavior
- `double_submit`: same token must exist in cookie and header.
- `synchronizer_token`: server-tracked token must match request token.
- For `SameSite=None`, `secureCookie=true` is mandatory.

## 5) Policy keys additions

```toml
[cors]
enabled = true
allowed_origins = ["https://app.example.com"]
allowed_methods = ["GET","POST","PUT","DELETE"]
allowed_headers = ["content-type","authorization"]
exposed_headers = []
allow_credentials = true
max_age_seconds = 600
forbid_any_origin = true
forbid_reflect_origin = true
require_vary_origin = true

[security_headers]
enabled = true
strip_server_header = true
x_content_type_options = true
x_frame_options = "DENY"
referrer_policy = "strict-origin-when-cross-origin"

[security_headers.hsts]
enabled = true
max_age_seconds = 15552000
include_subdomains = true
preload = false

[security_headers.csp]
enabled = true
report_only = true

[csrf]
enabled = true
mode = "double_submit"
cookie_name = "csrf"
header_name = "x-csrf-token"
same_site = "Lax"
secure_cookie = true
http_only_cookie = false
protected_methods = ["POST","PUT","PATCH","DELETE"]
```

### Enforcement examples
- Reject wildcard-origin + credentials CORS config.
- Reject `same_site="None"` when `secure_cookie=false`.
- Warn or reject HSTS in development HTTP-only mode based on policy mode.

## 6) Typed header primitives

```ailang
module headers {
  fn name(s: String) -> HeaderName
  fn value(s: String) -> HeaderValue
}
```

No middleware path should emit raw header strings.

## 7) Acceptance tests
1. CORS preflight returns expected allow headers/methods/origin and does not reflect unmatched origins.
2. Credentials plus wildcard origin is rejected by policy validation.
3. Security headers are present on success and error responses.
4. CSRF rejects protected requests without valid token in cookie-auth mode.
5. `Vary: Origin` is present for allowlist-based CORS responses.
