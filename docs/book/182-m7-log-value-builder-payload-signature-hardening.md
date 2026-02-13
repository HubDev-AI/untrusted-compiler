# 182 M7 Slice: Log Value Builder Payload Signature Hardening

This chapter documents a focused M7 hardening step for core log value-builder payload types.

## What it is

Added compile-time signature checks so:
- `log.str(value)` requires `String`.
- `log.i64(value)` requires numeric values.
- `log.bool(value)` requires `Bool`.
- `log.field(key, value)` requires `String` key and `LogValue` value.
- `log.obj(fields)` enforces single-argument arity.

Violations emit `E4001` diagnostics tagged with `security`.

## Why it exists

Previous bridge-mode fixtures used numeric placeholders for log-builder inputs. This slice tightens structured logging construction and keeps builder contracts aligned with typed log sink requirements.

## How it works internally

`enforce_log_value_builder_signatures(...)` in `semantic.rs` now validates:
1. call-shape/arity for `log.str`, `log.i64`, `log.bool`, `log.field`, and `log.obj`,
2. argument types for scalar builders and `log.field` key/value inputs.

Checks run before log sinks, so malformed builder payloads fail early.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures:
    - `invalid_log_str_argument_type.ai`
    - `invalid_log_i64_argument_type.ai`
    - `invalid_log_bool_argument_type.ai`
    - `invalid_log_field_argument_type.ai`
  - tag tests in `compiler/ailang-core/tests/diagnostic_tags.rs`
  - log builder integration fixtures in:
    - `compiler/ailang-core/tests/c_backend.rs`
    - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection for malformed log builder payload arguments.
  - typed log builder fixtures continue to pass `c-bin` integration tests.
- Constraint:
  - this slice does not yet validate full structural shape of `log.obj(fields)` contents.

## Failure modes and diagnostics

Examples:
- `log.str(1)` -> `E4001`: argument must be `String`.
- `log.i64(true)` -> `E4001`: argument must be numeric.
- `log.bool(1)` -> `E4001`: argument must be `Bool`.
- `log.field(1, log.i64(1))` -> `E4001`: key argument must be `String`.

## Example usage

```ailang
fn main() -> Int {
  let count = log.i64(1);
  let field = log.field("count", count);
  let text = log.str("ok");
  let flag = log.bool(true);
  let obj = log.obj(field);
  count;
  field;
  text;
  flag;
  obj;
  0
}
```

## Tradeoffs and next steps

- Tradeoff: bridge fixtures using non-typed placeholders for builder payloads now fail and must be rewritten.
- Next:
  - add deeper `log.obj` structural validation once collection typing for `LogValue` maps/lists is formalized.
