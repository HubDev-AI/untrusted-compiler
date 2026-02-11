# 53 AILang v0 Security Baseline

This chapter defines compile-time rules and runtime requirements for the v0 security-first baseline.

## 1) Mandatory security primitives (types)

### 1.1 Trust and secrecy
- `Untrusted<T>`: values from external boundaries (HTTP, env, raw JSON, etc.)
- `Secret<T>`: sensitive values (tokens, keys, credentials)

### 1.2 Typed sinks
- `SqlQuery`
- `HtmlSafe`
- `PublicUrl`
- `InternalUrl`
- `PathSafe`
- `HeaderName`
- `HeaderValue`
- `Cookie`
- `LogValue`

### 1.3 Resource control
- `Budget` with at least:
  - `maxBodyBytes`
  - `maxJsonBytes`
  - `maxJsonDepth`
  - `deadlineMs` or `deadline`
  - optional `maxDbRows`, `maxNetBytes`

### 1.4 Capabilities
- `DbCap`, `TxCap`
- `NetCap`, `InternalNetCap`
- `FsCap`
- `SecretsCap`

## 2) Mandatory compile-time rules (hard errors)

### 2.1 No implicit unwrapping
- Forbid implicit conversion:
  - `Untrusted<T> -> T`
  - `Secret<T> -> T`

### 2.2 Canonical trust gates
- HTTP body to trusted data must be schema-gated:
  - `req.json(Schema)`
- Other conversions must pass explicit validators/sanitizers.

### 2.3 Typed sinks only
Sink APIs must reject raw `String`/`Untrusted<String>` where safe typed values are required.

### 2.4 Secrets cannot leak
Forbid `Secret<_>` in:
- logging
- JSON encoding
- string interpolation/formatting
- response headers/bodies and unsafe sink parameters

Allow:
- `redact(secret)`
- `reveal(secret)` only with effect + capability + policy allowance

### 2.5 Effects + capabilities both required
Sensitive operations must satisfy both:
- declared effect requirement
- capability value availability

### 2.6 Response splitting prevention
- `HeaderValue` requires validation that rejects CR/LF
- raw header string APIs forbidden

### 2.7 DoS hooks required
- JSON decoding enforces `Budget`
- request body extraction enforces body size budget

## 3) Runtime requirements

### 3.1 SSRF hardening
`validate.publicUrl` must:
- parse safely
- resolve DNS and validate final IP
- block loopback, link-local, private ranges, metadata ranges
- re-validate redirects (or disallow)
- enforce scheme/port/domain policy constraints

`validate.internalUrl` requires explicit internal-network policy and capability.

### 3.2 JSON limits
Decoder must enforce:
- max bytes
- max depth
- optional max field/string/array limits

Errors should include path + reason + exceeded limit.

### 3.3 Budgets and deadlines
Runtime must enforce:
- request deadlines/timeouts
- cancellation propagation
- max concurrent requests

### 3.4 Logging safety
Runtime must:
- keep logs structured (`LogValue`)
- enforce redaction boundaries
- include correlation ID by default

### 3.5 SQL safety extension
- parameterized queries only
- optional row/time limits driven by `Budget`/policy

## 4) Policy-as-code enforced at compile time

Policy keys should include:
- forbidden effects
- URL and redirect rules
- JSON/body/concurrency limits
- structured logging requirements
- encoding strictness (`schema-required` mode)

Allowlist escape hatches require explicit annotation fields (`reason`, `ticket`, optional expiry).

## 5) Acceptance test baseline

Compiler must reject at least:
1. `db.exec("SELECT " + userInput)`
2. raw HTML output without `HtmlSafe`
3. logging/encoding `Secret<_>`
4. effect usage without declaration
5. missing schema gate for trusted request-body decode
6. internal URL usage without capability/policy
7. unsafe header setting without `HeaderValue`
8. request JSON decode without required budget context (if configured)

## 6) Minimal ergonomics

Keep TS-like usage while enforcing safety:
- standard handler context carries `log`, `budget`, `caps`, `traceId`
- safe constructors are one-line and explicit
- diagnostics include origin, sink, and concrete fix path
