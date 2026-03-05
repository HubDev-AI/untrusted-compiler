# M39: LASM Request-Header Case-Insensitive Merge

## Why

LASM request parsing previously inserted incoming headers directly into a map with case-sensitive keys.

That allowed duplicate semantic headers (`X-Request-Id` + `x-request-id`) to diverge by casing and made request-derived header reads depend on map-key ordering instead of deterministic merge behavior.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Replaced direct request-header map insertions with `insert_lasm_request_header_case_insensitive(...)`.
2. Added deterministic duplicate-header merge behavior:
   - `Host` and `Content-Length` keep existing conflict checks/semantics.
   - `Cookie` duplicates merge with `; `.
   - all other duplicate headers merge with `, ` in arrival order.
3. Kept existing host/content-length hard-failure checks unchanged.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_merges_duplicate_request_headers_case_insensitively`
   - verifies duplicate request headers with different casing are merged deterministically and visible through `req.header(...)`.
2. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_returns_400_for_conflicting_host_headers`
   - verifies host conflict rejection behavior is preserved.
