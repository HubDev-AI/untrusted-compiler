# M37-S4 Structured Runtime Log Emission

## What it is

M37-S4 replaces handle-only runtime logging behavior with real structured JSON log emission.

1. Runtime `log_any` now emits JSON log lines.
2. Log value helpers (`log.str/i64/bool/redacted/attrRedacted`) now materialize JSON-safe payload values.
3. Log builder helpers (`log.withAttr/withHttp/withError`) now contribute deterministic structured metadata to emitted events.

## Why it exists

No-stub alpha cannot claim working observability if log paths only return opaque handles.
This slice makes runtime logging operational, deterministic, and testable, while preserving existing intrinsic ABI surface.

## How it works

### Runtime log state model

`runtime/c/sec4_runtime.c` now keeps explicit log registries:

- log value registry: maps runtime handles to JSON values,
- log event registry: tracks event name, attrs, http metadata, and attached error handle,
- emitted-line builder: renders one deterministic JSON line per `sec4_rt_log_any(...)` call.

### Structured fields emitted by default

Each emitted log line now includes:

- `timeMs` from runtime clock,
- `level` (currently `info` baseline for the unified sink ABI),
- `traceId` (request trace if present, otherwise `rt-0`),
- `event`.

Optional blocks are emitted when present:

- `attrs`
- `http`
- `error`

### JSON-safe materialization and redaction

- Runtime string payloads are escaped via JSON escaping helper.
- Redaction helpers emit explicit redaction markers as structured values.
- `SEC4_RT_LOG_OUTPUT=off` disables emission; default output target remains stderr.

## Validation

- `cargo test -p sec4 --test json_output build_emit_c_bin_handles_log_intrinsic_when_clang_available`
- `cargo test -p sec4 --test json_output build_emit_c_bin_handles_log_builder_intrinsics_when_clang_available`
- `cargo test -p sec4 --test json_output c_bin_runtime_log_builders_emit_structured_json_when_clang_available`
- `cargo test -p sec4 --test commands`
- `cargo test -p sec4 --test alpha_smoke`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

## Trade-offs

- Current ABI routes `log.info/warn/error` through one runtime sink, so emitted level is currently baseline `info`.
- `error` attachment currently carries deterministic handle metadata; full StdError object embedding remains future work.

## Next

1. Run final no-stub alpha verification pass on `main` (full smoke + targeted runtime harness matrix).
2. Publish alpha checklist delta from verified no-stub evidence.
3. Close M37 with a strict residual-gap note before alpha tag decision.
