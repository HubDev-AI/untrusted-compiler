# 52 Security-First Priorities for AILang v0

This chapter captures the high-leverage security changes required to keep AILang secure from day one without adding Rust-like complexity.

## 1) Capability-based security model for sensitive operations

Effects are required for auditing, but dangerous operations also require explicit capabilities (tokens) for authorization.

```ailang
function fetchUser(net: NetCap, url: UrlSafe) effects { net } -> Result<Bytes, NetError>
```

Why this matters:
- Prevents hidden privilege usage in random helper functions.
- Makes privilege boundaries explicit and testable.
- Fits dependency injection and capability-object design.

Minimum capability set:
- `DbCap`, `NetCap`, `FsCap`, `SecretsCap`

Runtime should provide these only to allowed entrypoints.

## 2) Policy-as-code enforced by compiler

A policy file (`policy.ai` or `ailang.policy`) must be enforced as hard compile errors, not warnings.

Examples:
- forbid `shell`
- forbid `secrets.reveal`
- forbid internal URL usage outside approved modules
- require schemas for all handler input
- require `SqlQuery` only (no raw query escape hatch unless allowlisted)

Build artifacts should include policy identity/hash.

## 3) SSRF and egress hardening

Split URL safety into two types:
- `PublicUrl` (default outbound)
- `InternalUrl` (extra permissions required)

Validation requirements:
- block private/link-local/loopback/metadata ranges by default
- re-validate redirects (or disallow by default)
- support explicit allowlists for hardened mode

## 4) Logging safe by default (structured + redaction)

Rules:
- `log.*` accepts only `LogValue`
- `Secret<T>` cannot become `LogValue` except redacted forms

```ailang
log.info({ event: "user_created", userId, email: redact(emailSecret) })
```

Add correlation IDs as standard context and propagate automatically.

## 5) Header injection / response splitting prevention

Use typed HTTP primitives:
- `HeaderName`, `HeaderValue` with CRLF-safe validation
- `Response.setHeader(HeaderName, HeaderValue)` only
- `Cookie` builder type for safe cookie assembly

## 6) Resource limits for DoS resistance

Required defaults:
- request body size limits at router level
- JSON depth/size limits in decoder
- timeouts/deadlines in net/db capability APIs
- rate-limit hooks in runtime

Use explicit budget types:
- `Budget { maxBytes, maxJsonDepth, deadlineMs }`

## 7) Constant-time and crypto footgun policy

If crypto is exposed:
- forbid `Secret<Bytes>` comparisons with `==`
- require `crypto.ctEq(a, b)`
- preserve no-log/no-encode secret restrictions

## 8) SQL safety beyond injection

In addition to typed `SqlQuery`:
- policy/lint for unbounded queries (`LIMIT` required unless allowlisted)
- no dynamic `ORDER BY` unless `SqlIdentSafe`
- typed identifiers from allowlist
- `TxCap` requirements for selected write paths

## 9) Safe concurrency defaults

Even before full async:
- enforce max concurrent requests
- propagate cancellation per request
- forbid fire-and-forget spawn unless explicitly allowed
- use `TaskGroup` for structured concurrency

## 10) Supply-chain and build integrity

Required baseline:
- mandatory lockfile
- hash-pinned dependencies
- reproducible builds
- policy hash + compiler version in binary metadata
- optional SBOM generation

## If only three additions ship early

1. Capabilities + effects (authorization + auditing)
2. SSRF hardening with `PublicUrl`/`InternalUrl` + redirect/DNS/IP checks
3. Safe logging + typed headers
