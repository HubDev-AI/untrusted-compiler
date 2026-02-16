# M38-S26 Outbound HTTP Redirect Hop-Cap Diagnostics

## What it is

M38-S26 hardens redirect hop accounting by splitting operator-configured redirect limit exhaustion from runtime safety-cap exhaustion with deterministic diagnostics.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Redirect limit behavior now has two distinct controls:

1. configurable policy limit (`SEC4_RT_NET_PUBLIC_MAX_REDIRECTS`)
2. runtime hard safety cap (`SEC4_RT_MAX_OUTBOUND_HTTP_REDIRECTS`)

These two failure modes should produce distinct diagnostics for operational clarity.

## How it works

1. Runtime keeps configured redirect max and computes effective max under hard cap.
2. If configured max exceeds runtime cap, runtime applies cap deterministically.
3. On hop exhaustion:
   - configured-limit exhaustion -> `NET.REDIRECT_LIMIT`
   - runtime-cap exhaustion -> `NET.REDIRECT_CAP_LIMIT`
4. Cycle detection (`NET.REDIRECT_CYCLE_DETECTED`) remains independent.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_cap_limit_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_limit_exceeded_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_cycle_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Redirect policy handling now has slightly more internal state (configured max + effective cap).
- The extra complexity is intentional for deterministic operator-facing diagnostics.

## Next

1. Split shared redirect policy-invalid diagnostics into field-specific deterministic codes.
2. Keep redirect matrix tests green while policy diagnostics are made more granular.
