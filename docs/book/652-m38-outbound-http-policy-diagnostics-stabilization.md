# M38-S32 Outbound HTTP Policy Diagnostics Stabilization

## What it is

M38-S32 is a stabilization slice that aligns redirect policy diagnostic naming across tests and docs, then runs a final targeted runtime matrix sweep to lock behavior.

Files:

- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/644-m38-outbound-http-redirect-policy-surface-validation-diagnostics.md`
- `docs/book/645-m38-outbound-http-redirect-cycle-diagnostics.md`
- `docs/book/647-m38-outbound-http-redirect-policy-field-specific-diagnostics.md`
- `docs/book/649-m38-outbound-http-redirect-policy-structured-details-diagnostics.md`

## Why it exists

Redirect diagnostics were already implemented, but one policy-invalid test name still used a generic label. This slice removes naming drift so runtime/test/docs references stay deterministic and auditable.

## How it works

1. Renamed the allow-redirects policy-invalid harness test to an explicit, field-specific name.
2. Updated roadmap/chapter validation references to the same canonical test name.
3. Ran a final matrix sweep for redirect/tls/parser subsets to verify no behavior regression from naming-only edits.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_https_`
- `cargo test -p sec4 --test json_output c_bin_runtime_redirect_resolver_`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- This slice is intentionally non-functional; it improves consistency and operator clarity.
- Broad matrix runs cost more time than single-target checks, but they reduce drift risk before the next runtime hardening slice.

## Next

1. Extend SSRF policy/runtime parity for block toggles beyond `resolve_dns` and `revalidate_redirects`.
2. Add deterministic coverage proving those toggles affect `url.public(...)` behavior through both direct host and DNS-resolved paths.
