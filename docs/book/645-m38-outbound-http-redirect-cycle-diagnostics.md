# M38-S25 Outbound HTTP Redirect Cycle Diagnostics

## What it is

M38-S25 hardens redirect hop accounting by detecting cyclic redirect chains and returning a dedicated deterministic runtime diagnostic.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Redirect limits protect against long chains, but cycle-specific failures were not explicitly classified. A redirect loop should be detected deterministically and surfaced as a clear policy/debug signal.

## How it works

1. Runtime now tracks visited redirect URLs in redirect follow-up flow.
2. Before each hop transition, runtime checks whether the next URL has already been visited.
3. On repeat URL detection, runtime emits deterministic code:
   - `NET.REDIRECT_CYCLE_DETECTED`
4. Existing max-hop and redirect-policy diagnostics remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_cycle_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_allow_redirects_policy_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_scheme_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Redirect handling now does extra per-hop URL comparisons for cycle detection.
- The small overhead is acceptable for deterministic security diagnostics and better incident triage.

## Next

1. Split redirect cap-exhaustion diagnostics from cycle diagnostics for clearer hop-budget behavior.
2. Keep redirect hardening matrix tests green as hop-policy semantics are refined.
