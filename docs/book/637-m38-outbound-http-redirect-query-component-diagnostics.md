# M38-S17 Outbound HTTP Redirect Query-Component Diagnostics

## What it is

M38-S17 hardens redirect query handling by validating malformed query components explicitly and emitting deterministic runtime diagnostics.

Files:

- `runtime/c/sec4_runtime.c`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Redirect validation already covered host/path/fragment and invalid target characters, but malformed query payloads were not split into a dedicated error class.

This slice adds deterministic query validation so malformed redirect query data does not collapse into generic redirect failures.

## How it works

1. Redirect resolver status expansion:
   - adds `SEC4_RT_REDIRECT_RESOLVE_QUERY_INVALID`.
2. Query-component validation:
   - rejects repeated query separators (`?` inside query payload),
   - rejects malformed percent escapes (for example `%zz`, `%a`, trailing `%`).
3. Runtime mapping:
   - query-invalid outcomes now emit deterministic `NET.REDIRECT_QUERY_INVALID`.
4. Existing fragment, target-character, target-path, and host diagnostics remain unchanged.

## Validation

- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_query_percent_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_query_separator_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_fragment_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_target_char_invalid_returns_deterministic_code_when_clang_available`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Query validation is now stricter and may reject malformed redirect targets that previously flowed into generic paths.
- The stricter behavior is intentional to keep redirect security and debugging deterministic.

## Next

1. Continue redirect hardening by splitting deterministic scope-revalidation diagnostics for absolute redirect targets.
2. Keep redirect parser matrix tests green while expanding deterministic error coverage.
