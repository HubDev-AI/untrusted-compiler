# 398 M16 Slice: Runtime Status Propagation for Success Responses

This chapter documents M16-S3: hardening runtime success-response helpers so explicit status values from handler calls are preserved.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` for:

- `sec4_rt_res_ok(status, schema, value)`
- `sec4_rt_res_ok_meta(status, schema, value, meta)`

Both functions now apply the provided `status` to the HTTP response line.

## Why it exists

M16-S2 materialized JSON responses, but `res.ok`/`res.okMeta` used a fixed `201` status. That prevented routes from expressing alternative success statuses (`202`, `204`, etc.) even when call sites passed explicit status values.

## Implementation details

1. Updated runtime function definitions to consume status arguments.
2. Added defensive fallback:
   - if `status <= 0`, runtime falls back to `201`.
3. Preserved existing deterministic JSON body/content-type behavior from M16-S2.
4. Kept schema/value/meta arguments bridge-stage ignored in runtime (semantic/type system remains the primary correctness gate).

## Validation

Added CLI integration test in `compiler/sec4-cli/tests/json_output.rs`:

- `c_bin_http_runtime_res_ok_honors_custom_status_when_clang_available`

Flow:
1. Build temp project via `--emit c-bin`.
2. Route handler calls `res.ok(202, ...)`.
3. Send `POST /users` in oneshot runtime mode.
4. Assert response includes:
   - `HTTP/1.1 202 ...`
   - JSON content-type,
   - deterministic JSON body `{"ok":true}`.

## Tradeoffs and next steps

- Status propagation is now correct for runtime success helpers.
- Body serialization is still fixed bridge output; schema-driven value encoding remains a follow-up runtime slice.
