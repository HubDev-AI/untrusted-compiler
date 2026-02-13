# 116 M7 Slice: Typed Router Security Bootstrap Contracts

This chapter documents the M7 slice that makes canonical router security bootstrap calls type-checked and composable.

## What it is

Added semantic contracts for canonical dotted bootstrap APIs:
- `sec.defaultHeaders()` -> `SecurityHeadersConfig`
- `cors.fromPolicy()` -> `CorsConfig`
- `csrf.fromPolicy()` -> `CsrfConfig`
- `auth.fromPolicy()` -> `AuthConfig`
- `sec.withSecurityHeaders(router, cfg)` -> `Router`
- `cors.withCors(router, cfg)` -> `Router`
- `csrf.withCsrf(router, cfg)` -> `Router`
- `auth.withAuth(router, cfg)` -> `Router`

## Why it exists

M7 requires a policy-driven security bootstrap path for APIs. Before this slice these calls were mostly placeholder bridges with weak semantic contracts. The new checks make canonical wiring auditable and reject invalid composition early.

## How it works internally

Semantic analysis now:
1. Assigns named return types for policy/bootstrap constructors and middleware wrappers.
2. Validates canonical dotted calls for arity and argument types.
3. Emits `E4001` diagnostics for invalid constructor arity or middleware argument types.

Checks are intentionally scoped to canonical dotted APIs in this slice, preserving compatibility for legacy bridge aliases used in targeted bootstrap tests.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures and CLI c-bin integration fixtures
- Outputs:
  - typed middleware bootstrap chaining in semantic layer
  - deterministic errors for malformed bootstrap calls
- Constraints:
  - this validates call shape/types; it does not yet validate runtime security behavior (preflight/header/CSRF execution) end-to-end.

## Failure modes and diagnostics

Representative diagnostics:
- `E4001`: policy/bootstrap constructor takes no arguments
- `E4001`: security middleware bootstrap call has invalid argument count
- `E4001`: security middleware bootstrap argument has invalid type

## Example usage

```ut
fn main() -> Int {
  let headers = sec.defaultHeaders();
  let corsCfg = cors.fromPolicy();
  let csrfCfg = csrf.fromPolicy();
  let authCfg = auth.fromPolicy();

  let base = http.router();
  let withHeaders = sec.withSecurityHeaders(base, headers);
  let withCors = cors.withCors(withHeaders, corsCfg);
  let withCsrf = csrf.withCsrf(withCors, csrfCfg);
  auth.withAuth(withCsrf, authCfg);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: alias/non-canonical forms remain permissive to avoid breaking existing bootstrap bridge coverage.
- Next:
  - move deeper into M7 runtime behavior (real middleware effects and HTTP lifecycle tests),
  - then tighten/phase out permissive alias forms once full canonical surface is covered.
