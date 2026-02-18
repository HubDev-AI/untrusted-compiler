# M39: LASM Dynamic Set-Cookie Placeholder Materialization

## Why

LASM placeholder materialization already covered response body and response headers, but `res.addCookie(cookie.build(...))` still treated request-derived cookie value expressions as non-materializable in extraction.

That caused dynamic cookie flows to be dropped back to static-only behavior in LASM run mode.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Extended `extract_cookie_literal(...)` so `cookie.build(name, value)` accepts request-derived placeholder-bearing value expressions in addition to static string bindings.
2. Cookie value extraction now resolves placeholder tokens from:
   - direct `req.pathParam(...)` / `req.header(...)` / `req.query(...)` calls,
   - `validate.nonEmpty(...)` wrappers around those calls.
3. Existing response-header materialization path now resolves the emitted `Set-Cookie` header value tokens at write time, giving deterministic runtime values.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_emits_set_cookie_from_add_cookie`
   - uses `cookie.build("session", validate.nonEmpty(req.query("session")))`,
   - verifies decoded query-driven cookie emission (`Set-Cookie: session=demo+token`).
2. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
   - sanity check to keep existing dynamic body/header placeholder behavior green.
