# M38-S53 Outbound HTTP Internal-Policy Allow-Token Coverage

## What it is

M38-S53 adds direct runtime harness coverage for truthy token acceptance on `SEC4_RT_ALLOW_INTERNAL_NET`.

The harness verifies that truthy tokens bypass internal-policy denial and reach downstream URL/parser/transport outcomes deterministically.

Files:

- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

M38-S52 locked deny precedence for valid internal URLs when policy is disabled.

This slice completes the policy-toggle contract by proving the allow path works for all accepted truthy token forms, not only `"1"`.

## How it works

1. Added clang-gated harness test:
   - `c_bin_runtime_internal_policy_allow_truthy_tokens_bypass_denial_when_clang_available`
2. The harness runs matrix checks over truthy tokens:
   - `"1"`, `"true"`, `"yes"`, `"on"`, `"allow"`
   - case variants (for example `"TRUE"`, `"YeS"`, `"On"`)
3. For each token, harness validates two deterministic downstream outcomes:
   - parser stage: `NET.REQUEST_TARGET_INVALID` (`kind=validation`) for malformed target URL
   - transport stage: `NET.TLS_UNSUPPORTED` (`kind=runtime`) for HTTPS in non-TLS runtime build
4. It also asserts allow-path exclusions:
   - no `NET.INTERNAL_DENIED`
   - no `NET.GET_INTERNAL_INVALID`

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_allow_truthy_tokens_bypass_denial_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_denial_precedence_for_valid_internal_urls_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Adds one focused harness without runtime behavior changes.
- Increases targeted runtime-contract test count, but keeps policy-deny and policy-allow behavior isolated and easier to debug.

## Next

1. Cover unknown-token fallback for `SEC4_RT_ALLOW_INTERNAL_NET` and assert deny-by-default behavior remains deterministic.
2. Keep policy-token parsing contracts isolated from wrapper invalid-handle and parser diagnostics contracts.
