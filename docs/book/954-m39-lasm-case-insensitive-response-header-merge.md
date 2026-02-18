# M39: LASM Case-Insensitive Response Header Merge

## Why

LASM response writing previously treated header map keys as case-sensitive.

That allowed duplicate semantic headers (for example `Content-Type` and `content-type`) when user handlers set lowercase names while response defaults injected canonical casing.

The same class of issue could also duplicate or inconsistently override runtime-injected headers like `X-Trace-Id`.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Added case-insensitive normalization pass for response headers before default/header injection in the LASM response writer.
2. Default header insertion now uses case-insensitive "insert-if-missing" behavior.
3. Connection header injection now uses case-insensitive upsert semantics.
4. Trace header injection now updates existing `x-trace-id` keys case-insensitively instead of always inserting canonical key casing.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_merges_header_names_case_insensitively`
   - handler sets lowercase `content-type` and `x-trace-id`,
   - verifies exactly one `content-type` header is emitted,
   - verifies fallback default content type is not injected,
   - verifies deterministic runtime trace value (`rt-1`) overrides user trace value with no duplicate trace header.
2. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
3. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_emits_set_cookie_from_add_cookie`
