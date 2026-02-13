# 179 M7 Slice: Log Event Helper Signature Hardening

This chapter documents a focused M7 hardening step for structured log-event helper call contracts.

## What it is

Added compile-time signature checks for:
- `log.attrRedacted(label)`
- `log.withAttr(event, key, value)`
- `log.withHttp(event, method, path, status, latencyMs)`
- `log.withError(event, error)`

Violations emit `E4001` diagnostics tagged with `security`.

## Why it exists

The helper bridge existed, but placeholder-style calls could pass untyped numeric values in key positions. Hardening these signatures prevents malformed structured-log construction and keeps log-event APIs aligned with the security-first stdlib contract.

## How it works internally

In `semantic.rs`, `enforce_error_helper_signatures(...)` now also validates log-event helper signatures:
1. `log.attrRedacted` enforces one `String` argument.
2. `log.withAttr` enforces `(LogEvent|LogValue, String, LogAttr)`.
3. `log.withHttp` enforces `(LogEvent, String, String, Int/Int64, Int/Int64)`.
4. `log.withError` enforces `(LogEvent, StdError)`.

The checks run in the same semantic enforcement pass as other stdlib helper contracts.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixtures under `compiler/sec4-core/tests/fixtures/semantic/`
  - diagnostic tag tests in `compiler/sec4-core/tests/diagnostic_tags.rs`
  - log-builder integration fixtures in:
    - `compiler/sec4-core/tests/c_backend.rs`
    - `compiler/sec4-cli/tests/json_output.rs`
- Outputs:
  - malformed helper usage fails fast during semantic analysis.
  - `c-bin` log-builder integration continues to pass with typed helper arguments.
- Constraint:
  - this slice hardens call signatures only; runtime log serialization semantics remain unchanged.

## Failure modes and diagnostics

Examples:
- `log.attrRedacted(1)` -> `E4001`: argument must be `String`.
- `log.withAttr(event, 1, attr)` -> `E4001`: key argument must be `String`.
- `log.withHttp(event, 1, "/users", 200, 42)` -> `E4001`: method argument must be `String`.
- `log.withError(event, 1)` -> `E4001`: error argument must be `StdError`.

## Example usage

```ut
fn main() -> Int {
  let event = log.event(1);
  let attr = log.attrRedacted("token");
  let withAttr = log.withAttr(event, "token", attr);
  let withHttp = log.withHttp(withAttr, "POST", "/users", 200, 42);
  let err = err.internal("boom");
  log.withError(withHttp, err);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: bridge-mode examples that used loose numeric placeholders for helper arguments now fail and must be rewritten with typed values.
- Next:
  - enforce deeper `LogEvent`/`LogAttr` payload-shape constraints once richer log-structure typing is introduced.
