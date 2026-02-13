# M4 Security Foundation Implementation (Current Slice)

This chapter documents the first implemented code slice of M4 (security foundation hardening).

## Scope delivered in this slice
- Added compiler policy loading from `sec4.policy` (defaulting to secure policy when file is absent).
- Added policy parser/schema validation for major sections with strict unknown-key rejection.
- Enforced default forbidden effects (`shell`, `unsafe`, `secrets.reveal`) at semantic analysis time.
- Added capability-aware intrinsic checks for sensitive operations:
  - DB calls require `DbCap`
  - NET calls require `NetCap` / `InternalNetCap`
  - FS calls require `FsCap`
  - Secret calls require `SecretsCap`
- Introduced diagnostics for capability errors:
  - `E2003` operation requires capability
  - `E2004` capability type mismatch

## Integration points
- `analyze_entry` now loads policy before semantic analysis and passes it into `analyze_program_with_policy`.
- Semantic analyzer now tracks policy in analysis state and applies policy constraints during function collection/body checks.

## Test coverage added
- Semantic fixtures for:
  - missing capability
  - capability mismatch
  - forbidden effect by default policy
  - valid capability + effects use
- Policy parser tests for:
  - default policy behavior
  - unknown key rejection
  - invalid CORS wildcard+credentials combination
  - invalid redirect policy combination

## What remains in later M4/M8 slices
- Typed sink flow checks (`Untrusted` and `Secret` misuse into sinks).
- Schema-gated boundary enforcement rules.
- Strict logging/header/cookie/public/internal URL sink typing checks.
- Full policy-driven allowlist annotation flow.
