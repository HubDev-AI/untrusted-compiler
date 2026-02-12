# 124 M7 Slice: Header Constructor Signature Hardening

This chapter documents a focused M7 gate-contract hardening step for header constructor helpers.

## What it is

Added semantic call-shape enforcement for:
- `headers.name(value)`:
  - exactly one argument,
  - argument must be `String`.
- `headers.value(value)`:
  - exactly one argument,
  - argument must be `String`.

Violations emit `E4001` diagnostics tagged with `security`.

## Why it exists

After tightening `res.setHeader` to require typed `HeaderName`/`HeaderValue`, constructor calls still accepted placeholder numerics. That left a weak path where typed sinks were strict but typed constructors were loose.

This slice tightens the constructor edge so typed header flow is consistent end-to-end.

## How it works internally

In semantic intrinsic enforcement:
1. added `enforce_header_builder_signatures(...)`,
2. recognized canonical constructor names:
   - `headers_name` / `headers.name`,
   - `headers_value` / `headers.value`,
3. enforced exact arity and `String` argument type,
4. emitted fix-oriented notes for valid call forms.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixtures for invalid header constructor calls
  - diagnostic-tag coverage in `compiler/ailang-core/tests/diagnostic_tags.rs`
  - header/cookie and gate `c-bin` integration fixtures in `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of invalid `headers.name/value` call shapes,
  - integration fixtures updated to explicit string constructor arguments.
- Constraint:
  - string-type validation is compile-time only here; runtime header normalization/validation policy remains owned by runtime gates.

## Failure modes and diagnostics

Examples:
- `headers.name(1)` -> `E4001` (`headers.name argument must be String`).
- `headers.value("ok", "extra")` -> `E4001` (`headers.value expects exactly one argument`).

## Example usage

```ailang
fn configure() effects { net } -> Int {
  let name = headers.name("X-Test");
  let value = headers.value("ok");
  res.setHeader(name, value);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: stricter constructor contracts require fixture updates where numeric placeholders were used.
- Next:
  - continue tightening remaining helper constructors toward fully typed stdlib surfaces,
  - align LSP quick-fix suggestions with these constructor signatures.
