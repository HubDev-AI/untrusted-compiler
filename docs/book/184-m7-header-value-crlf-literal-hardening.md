# 184 M7 Slice: Header Value CRLF Literal Hardening

This chapter documents a focused M7 hardening step for constant `headers.value(...)` literals.

## What it is

`headers.value("...")` now rejects literal payloads containing CR/LF sequences (`\r`, `\n`, or escaped forms) at compile time.

Violations emit `E4001` diagnostics tagged with `security` + `sink`.

## Why it exists

Typed header constructors already enforce call shape/type. This slice adds an explicit compile-time guard for a common response-splitting footgun in constant literals.

## How it works internally

In `enforce_header_builder_signatures(...)`:
1. existing one-argument string checks remain,
2. for `headers.value` string literals, the analyzer scans literal text for CR/LF markers,
3. flagged literals emit a security sink diagnostic.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture: `invalid_headers_value_crlf_literal.ut`
  - tag test coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection of CR/LF header-value literals.
- Constraint:
  - this check is literal-only; dynamic values still rely on runtime/header-gate validation paths.

## Failure modes and diagnostics

Example:
- `headers.value("\\n")` -> `E4001`: headers.value literal cannot contain CR/LF.

## Example usage

```ut
fn ok() -> Int {
  headers.value("application/json");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: conservative escaped-sequence detection may reject intentionally escaped strings in compile-time literals.
- Next:
  - extend diagnostics with precise offending sequence snippets and suggestions for safe normalization helpers.
