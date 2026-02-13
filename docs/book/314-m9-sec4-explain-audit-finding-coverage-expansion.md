# 314 M9 Slice: sec4 explain Audit-Finding Coverage Expansion

This chapter documents extending `sec4 explain` mapping coverage for additional `sec4 audit` finding IDs.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

`sec4 explain` already covered core diagnostic codes and initial allowlist findings. Operators also need direct guidance for high-signal audit findings surfaced by policy/security posture checks.

## New mapping coverage

Added explicit explain mappings for:
- `CORS_CREDENTIALS_WITH_WILDCARD`
- `INTERNAL_NET_ENABLED_NO_ALLOWLIST`
- `CAPTURE_REDACTION_INCOMPLETE`
- `SECRETS_REVEAL_USED`
- `ALLOW_COUNT_HIGH`
- `ALLOW_EXPIRY_WINDOW_ROLLUP`

Mappings include deterministic topic, summary, likely actions, and docs pointer.

## Tests

Added integration coverage in `json_output.rs`:
- `explain_cors_wildcard_finding_prints_targeted_guidance`
- `explain_allow_count_high_finding_in_json_mode_is_parseable`

These complement prior explain text/json mapping tests.

## Inputs, outputs, and constraints

- Input:
  - finding ID passed to `sec4 explain`.
- Output:
  - exact mapping guidance in text or JSON mode.
- Constraint:
  - mapping set is static; new finding IDs should be added with test coverage.

## Example usage

```bash
sec4 explain CORS_CREDENTIALS_WITH_WILDCARD
sec4 explain ALLOW_COUNT_HIGH --format json
```

## Tradeoffs and next steps

- Tradeoff:
  - not all low-frequency finding IDs are mapped yet.
- Next:
  - continue coverage hardening for remaining lower-frequency audit findings.
