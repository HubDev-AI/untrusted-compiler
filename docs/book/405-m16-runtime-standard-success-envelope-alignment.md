# 405 M16 Slice: Runtime Standard Success Envelope Alignment

This chapter documents M16-S10: aligning runtime JSON success responses with the standard success-envelope shape.

## What it is

A runtime update in `runtime/c/sec4_runtime.c` introducing a helper for structured success payloads and migrating JSON success responders to use it.

Affected responders:

- `sec4_rt_res_json(...)`
- `sec4_rt_res_ok(...)`
- `sec4_rt_res_ok_meta(...)`

## Why it exists

Runtime success paths previously emitted simplified payloads (`{"ok":true}` / `{"ok":true,"meta":{}}`). That shape was deterministic but not aligned with the project’s standard success-envelope contract.

## Implementation details

1. Added runtime helper:
   - `sec4_rt_store_std_success_response(status, include_meta)`.
2. Envelope fields now include:
   - `ok`
   - `status`
   - `traceId`
   - `timeMs`
   - `data`
   - optional `meta`
3. Existing status propagation behavior is preserved:
   - `res.ok(status, ...)` keeps explicit caller status.
   - `res.okMeta(status, ...)` keeps explicit caller status.

## Validation

Updated integration assertions in `compiler/sec4-cli/tests/json_output.rs` verify success envelope fields for:

- runtime `POST /users` success path (`status:201` + `traceId:rt-1`),
- custom status path (`status:202` + `traceId:rt-1`),
- `sec4 run` command-path runtime roundtrip (`status:201` + `traceId:rt-1`).

All relevant runtime suites remain green.

## Tradeoffs and next steps

- Data serialization remains bridge-stage placeholder (`data: {}`) in this bootstrap phase.
- Future slices can connect typed schema/value serialization to `data` while preserving the same envelope contract.
