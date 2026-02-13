# 318 M9 Slice: sec4 explain Low-Frequency Audit Finding Coverage

This chapter documents extending `sec4 explain` so current low-frequency `sec4 audit` posture findings resolve to exact guidance instead of fallback messaging.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

`sec4 explain` already covered major diagnostics and initial audit finding IDs, but several active posture findings still routed to generic fallback output. That reduced operator guidance quality during triage.

## New mapping coverage

Added exact mappings for:
- header/CSP posture findings: `HSTS_DISABLED_IN_PROD`, `REFERRER_POLICY_WEAK`, `NOSNIFF_DISABLED`, `XFO_DISABLED`
- cookie/CORS coupling findings: `COOKIE_CROSS_SITE_WITH_WILDCARD_ORIGIN`, `COOKIE_SAMESITE_NONE_WITHOUT_SECURE`
- SSRF/egress findings: `DNS_RESOLUTION_DISABLED`, `PUBLIC_EGRESS_NO_DOMAIN_POLICY`
- filesystem/logging posture findings: `SYMLINK_POLICY_WEAK`, `LOG_STRUCTURED_ONLY_DISABLED`, `LOG_REMOTE_IP_ENABLED`, `LOG_USER_AGENT_ENABLED`

Each mapping includes deterministic topic, summary, likely actions, and a direct docs chapter pointer.

## Tests

Added CLI integration coverage:
- `explain_all_current_audit_finding_ids_in_json_mode_have_exact_mappings`

The test verifies each currently emitted `sec4 audit` finding ID:
- succeeds in `--format json`,
- does not return `"Unknown Diagnostic Family"`,
- does not use generic roadmap docs fallback path.

Coverage drift is also guarded by:
- `scripts/check-sec4-explain-audit-coverage.sh` (workflow-level audit-id to explain-map parity check).

## Inputs, outputs, and constraints

- Input:
  - `sec4 explain <FINDING_ID> --format text|json`.
- Output:
  - exact guidance payload for each mapped posture finding.
- Constraint:
  - mapping remains static and must be updated as new `sec4 audit` finding IDs are introduced.

## Example usage

```bash
sec4 explain DNS_RESOLUTION_DISABLED
sec4 explain LOG_STRUCTURED_ONLY_DISABLED --format json
```

## Tradeoffs and next steps

- Tradeoff:
  - explain mapping coverage still requires explicit maintenance as audit rules evolve.
- Next:
  - keep adding exact mappings in lockstep with any new `sec4 audit` finding IDs and preserve JSON-mode regression coverage.
