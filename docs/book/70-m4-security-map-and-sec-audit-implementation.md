# 70 M4 Implementation: security_map and sec.audit (Current Slice)

This chapter documents the M4 implementation slice that introduced compiler-emitted security metadata and a first deterministic security audit command.

## Scope delivered
- Added security metadata generation:
  - compiler builds `SecurityMap` from AST call sites and policy context
  - output file: `build/security_map.json`
- Added initial sensitive marker/tag coverage:
  - source/gate/sink/effect/capability/middleware tags for v0 baseline intrinsics and middleware call names
- Added security audit engine:
  - deterministic finding rules (CORS, headers, CSRF/auth coupling, SSRF/internal-net, filesystem, capture/replay, secrets reveal)
  - risk scoring and severity ranking
  - text + JSON report output
- Added CLI command:
  - `ailang sec audit --path <project> [--format text|json] [--fail-on 'risk>=HIGH']`
- Added `@allow(...)` annotation pipeline:
  - parser-compatible source preprocessing strips `@allow(...)` before AST parsing
  - annotation parser validates required fields (`policy`, `bypass`, `reason`, `ticket`, `expires`)
  - expired annotations are rejected during analysis
  - parsed allowlist entries are emitted in `security_map.allows` and surfaced in `sec.audit` exceptions
- Added deterministic allowlist hygiene findings in `sec.audit`:
  - high-risk bypass findings (`SECRETS_REVEAL_ALLOWLISTED`, `INTERNAL_NET_CALL_ALLOWLISTED`)
  - expiry hygiene findings (`ALLOW_EXPIRED`, `ALLOW_EXPIRING_SOON`)
  - exception-volume hygiene finding (`ALLOW_COUNT_HIGH`)
- Added dotted stdlib call support for security/effect metadata paths:
  - parser now supports member-call expressions (`module.fn(...)`)
  - semantic intrinsic/effect/capability checks accept dotted aliases (e.g. `db.exec`, `req.json`, `secrets.reveal`)
  - `security_map` call and middleware tag extraction supports dotted call names
  - intrinsic symbol registry now includes dotted stdlib symbols alongside underscore aliases
- Added semantic sink flow enforcement for secrets/untrusted data:
  - generic wrappers `Secret<T>` and `Untrusted<T>` are recognized in the type catalog
  - log sinks reject `Secret<_>` and `Untrusted<_>` values with origin-aware diagnostics
  - JSON response sinks reject `Secret<_>` and `Untrusted<_>` values with origin-aware diagnostics
  - additional typed sinks now enforce `Secret`/`Untrusted` boundaries:
    - SQL sinks (`db.exec`, `db.queryOne`)
    - URL/network sinks (`httpClient.get`, `httpClient.getInternal`)
    - filesystem sinks (`fs.read`, `fs.write`)
    - header/cookie sinks (`res.setHeader`, `res.addCookie`)
  - error codes actively exercised in semantic fixtures: `E1002`, `E1003`, `E1004`, `E1005`
- Added first-class trust-gate semantics for request boundary APIs:
  - request sources now return explicit untrusted wrappers:
    - `req.body` -> `Untrusted<Bytes>`
    - `req.query`, `req.pathParam`, `req.header` -> `Untrusted<String>`
  - `req.json(schema)` is enforced as an explicit schema gate:
    - missing schema argument is rejected with `E4001`
    - diagnostic explains canonical gate usage and trust-boundary intent
  - semantic fixtures added for:
    - missing schema gate argument rejection
    - untrusted `req.query` flow into SQL sink rejection

## Core architecture
- `policy` remains source of truth for effective security posture and validation.
- `security_map` is generated statically from parsed program calls + policy-derived middleware attrs.
- `sec.audit` consumes policy + `security_map` and computes findings via a deterministic rule table.

## Files added/updated
- `compiler/ailang-core/src/security_map.rs`
- `compiler/ailang-core/src/audit.rs`
- `compiler/ailang-core/src/policy.rs` (expanded policy fields + validation for security_headers/csrf/auth coupling)
- `compiler/ailang-core/src/lib.rs` (module exports + `write_security_map`)
- `compiler/ailang-cli/src/main.rs` (new `sec audit` command)

## Tests added
- `compiler/ailang-core/tests/security_map.rs`
- `compiler/ailang-core/tests/sec_audit.rs`
- extended `compiler/ailang-core/tests/policy.rs` for csrf/auth/cors validations

## Current limitations
- middleware detection currently relies on known callable names (dynamic dispatch and indirect call targets are not yet mapped).
- finding set is intentionally baseline-focused and will expand in M4/M8.

## Next implementation steps
1. Expand sink tagging beyond intrinsic call names into typed stdlib API symbols.
2. Add more policy-aware findings for SQL limits and logging privacy settings.
3. Extend trust-gate typing from `req.*` primitives to validator/sanitizer gate functions that yield trusted sink types.
4. Add deeper callsite metadata (argument role labels, source-origin edges) for audit explainability.
