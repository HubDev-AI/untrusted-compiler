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
- middleware detection currently relies on known callable identifiers (member-call syntax is future work).
- finding set is intentionally baseline-focused and will expand in M4/M8.

## Next implementation steps
1. Expand sink tagging beyond intrinsic call names into typed stdlib API symbols.
2. Add soon-expiring exception findings in `sec.audit` (not just expired compile-time rejection).
3. Add more policy-aware findings for SQL limits and logging privacy settings.
4. Add richer source-origin traces in security findings and exception rendering.
