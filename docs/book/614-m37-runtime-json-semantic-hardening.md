# M37-S2 Runtime JSON Semantic Hardening

## What it is

M37-S2 hardens runtime JSON contract behavior for request-gated decoding and response encoding:

1. `req.json` / `json.decode` now enforce non-zero schema descriptors.
2. `json.decode` is gate-aware for active HTTP request bodies.
3. `json.encode` and JSON responders now materialize real payload fragments into the standard success envelope.

## Why it exists

No-stub alpha needs runtime JSON behavior to be explicit and deterministic instead of placeholder passthrough logic.
Before this slice, decode/encode paths did not fully enforce gate ordering, schema alignment, or envelope materialization.

## How it works

### Request-gated decode enforcement

- Runtime request state now tracks `json_schema_handle` when `req.json(schema)` executes.
- `req.json(0)` fails deterministically with:
  - `JSON.SCHEMA_INVALID`
- `json.decode(ctx, schema, raw)` now rejects:
  - missing schema (`JSON.SCHEMA_INVALID`)
  - decode on request body before `req.json` gate (`JSON.GATE_REQUIRED`)
  - decode after a failed request gate (`JSON.GATE_FAILED`)
  - tracked schema mismatch between `req.json` and `json.decode` (`JSON.SCHEMA_MISMATCH`)

### JSON encoding and envelope materialization

- `json.encode(schema, value)` now validates schema and returns tracked JSON fragments (instead of passthrough values).
- `res.json`, `res.ok`, and `res.okMeta` now:
  - call `json.encode` for response data,
  - render deterministic envelope payloads with materialized `data`,
  - include materialized `meta` for `res.okMeta`.
- Internal encode/materialization failures emit deterministic:
  - `JSON.ENCODE_INTERNAL`

### c-bin compile compatibility bridge

- `sec4 build --emit c-bin` now adds `-Wno-int-conversion` to keep descriptor-literal schema callsites compile-compatible in the current bridge ABI.

## Validation

- `cargo test -p sec4 --test json_output build_emit_c_bin_compiles_hello_api_example_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_res_ok_meta_includes_meta_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_json_decode_enforces_gate_and_schema_alignment_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_validators_and_sanitizer_enforce_checks_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_http_runtime_req_json_rejects_invalid_body_when_clang_available`
- `cargo test -p sec4 --test json_output build_emit_c_bin_handles_req_res_intrinsics_when_clang_available`
- `cargo test -p sec4 --test commands`
- `cargo test -p sec4 --test alpha_smoke`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- The compile compatibility flag is a bridge, not the final schema ABI.
- Schema mismatch enforcement is strongest when schema descriptors are tracked handles; descriptor-literal ABI tightening remains future work.

## Next

1. M37-S3 middleware policy materialization hardening.
2. M37-S4 structured runtime log emission path.
3. Final alpha no-stub verification pass and publish checklist updates.
