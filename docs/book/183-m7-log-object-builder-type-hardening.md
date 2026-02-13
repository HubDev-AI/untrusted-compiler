# 183 M7 Slice: Log Object Builder Type Hardening

This chapter documents a focused M7 hardening step for `log.obj` input typing.

## What it is

`log.obj(...)` now enforces:
- exactly one argument,
- argument type must be `LogValue`.

Type violations emit `E4001` diagnostics tagged with `security`.

## Why it exists

`log.obj` previously had only arity checks. This left room for non-structured placeholder payloads to pass. Tightening to `LogValue` keeps object-builder flows aligned with the typed logging model.

## How it works internally

In `semantic.rs`, `enforce_log_value_builder_signatures(...)` now:
1. validates `log.obj` arity,
2. validates `log.obj` argument type (`LogValue`),
3. emits typed diagnostics with expected/found notes.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture: `invalid_log_obj_argument_type.ai`
  - tag test coverage in `compiler/ailang-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection for non-`LogValue` `log.obj` payloads.
- Constraint:
  - this slice keeps the current bridge form (`log.obj(singleValue)`) and does not yet model richer map/list object constructors.

## Failure modes and diagnostics

Example:
- `log.obj(1)` -> `E4001`: log.obj argument must be `LogValue`.

## Example usage

```ailang
fn main() -> Int {
  let field = log.field("count", log.i64(1));
  let obj = log.obj(field);
  obj;
  0
}
```

## Tradeoffs and next steps

- Tradeoff: numeric placeholder calls to `log.obj` now fail and must use composed `LogValue` payloads.
- Next:
  - add richer typed collection/object constructors once `LogValue` aggregate typing is formalized.
