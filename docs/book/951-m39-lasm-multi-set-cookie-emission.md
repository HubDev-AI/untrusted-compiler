# M39: LASM Multi Set-Cookie Emission

## Why

LASM header extraction previously stored response headers in a single map entry per name.

That meant multiple `res.addCookie(...)` calls in one handler collapsed into a single `Set-Cookie` value, silently losing cookies.

## What Changed

In `compiler/sec4-cli/src/main.rs`:

1. Added `append_lasm_set_cookie_header(...)` to accumulate cookie values in deterministic call order during LASM response-header extraction.
2. Updated response writer path to emit one HTTP header line per cookie value for `Set-Cookie` instead of collapsing into one line.
3. Preserved existing dynamic placeholder flow, so query/header/path-derived cookie values still materialize before response write.

## Validation

1. `cargo test -p sec4 --test commands run_command_oneshot_lasm_backend_emits_set_cookie_from_add_cookie`
   - route now sets two cookies,
   - verifies both headers are present (`Set-Cookie: session=demo+token` and `Set-Cookie: mode=active`).
2. `cargo test -p sec4 --test commands run_command_lasm_backend_materializes_req_placeholders_in_res_text`
   - sanity check for existing LASM placeholder materialization path.
