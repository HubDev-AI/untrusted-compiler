# M39: LASM Dynamic Cookie-Name Placeholder Materialization

## Why

LASM dynamic cookie placeholder support previously covered cookie values, but `cookie.build(name, value)` still required static/literal cookie names during extraction.

That created a parity gap where valid typed flows like `cookie.build(validate.nonEmpty(req.query("cookie_name")), ...)` compiled but could not materialize in LASM run mode.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Extended `extract_cookie_literal(...)` so the cookie name argument can resolve request-derived placeholders, not only static string bindings.
2. Reused existing request-placeholder extraction path to support:
   - `req.pathParam(...)`
   - `req.header(...)`
   - `req.query(...)`
   and `validate.nonEmpty(...)` wrappers for cookie names.
3. Preserved existing dynamic cookie value extraction and multi-`Set-Cookie` emission behavior.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_emits_set_cookie_from_add_cookie`
   - uses `cookie.build(cookie_name, validate.nonEmpty(req.query("session")))` where `cookie_name` comes from `req.query("cookie_name")`,
   - verifies emitted cookie header includes dynamic name and value (`Set-Cookie: demo_session=demo+token`),
   - verifies second cookie still emits on a separate `Set-Cookie` line.
2. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
   - sanity check for dynamic placeholder materialization path stability.
