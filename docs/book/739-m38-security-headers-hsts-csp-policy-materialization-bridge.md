# M38-S116 Security-Headers HSTS/CSP Policy Materialization Bridge

## What it is

M38-S116 closes security-headers policy bridge gaps by persisting HSTS/CSP runtime-shape fields in compiler policy and wiring them through `sec4 run` runtime env materialization.

Files:

- `compiler/sec4-core/src/policy.rs`
- `compiler/sec4-core/tests/policy.rs`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/commands.rs`

## Why it exists

Runtime already supports HSTS/CSP env controls (`SEC4_RT_SECURITY_HEADERS_*`), but policy flow only covered a subset of security-headers fields. This slice removes drift by making policy the canonical source for HSTS/CSP materialization.

## How it works

1. `SecurityHeadersPolicyConfig` now persists:
   - `hsts_max_age_seconds`
   - `hsts_include_subdomains`
   - `hsts_preload`
   - `csp_policy`
2. Policy parser ingests and validates:
   - `security_headers.hsts.max_age_seconds` (>= 0, and >= 1 when HSTS enabled)
   - `security_headers.hsts.include_subdomains`
   - `security_headers.hsts.preload`
   - `security_headers.csp.policy` (non-empty when provided)
3. `sec4 run` exports runtime bridge env keys for HSTS/CSP state:
   - `SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_INCLUDE_SUBDOMAINS`
   - `SEC4_RT_SECURITY_HEADERS_HSTS_PRELOAD`
   - `SEC4_RT_SECURITY_HEADERS_CSP_ENABLED`
   - `SEC4_RT_SECURITY_HEADERS_CSP_REPORT_ONLY`
   - `SEC4_RT_SECURITY_HEADERS_CSP_POLICY`
4. Run-command e2e verifies emitted success headers reflect policy-configured HSTS/CSP behavior.

## Validation

- `cargo test -p sec4-core --test policy`
- `cargo test -p sec4 --test commands run_command_oneshot_applies_security_headers_hsts_and_csp_from_policy`
- `cargo test -p sec4 --test commands run_command_oneshot_disables_security_headers_from_policy`
- `cargo test -p sec4 --test commands`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- Expands policy surface and bridge-key coupling that must stay synchronized.
- Improves determinism by ensuring security-header runtime behavior is policy-driven instead of implicit-env-driven.

## Next

1. Continue policy bridge hardening for remaining runtime knobs with partial/legacy env-only behavior.
2. Keep command e2e tests as the contract layer for policy-to-runtime materialization correctness.
