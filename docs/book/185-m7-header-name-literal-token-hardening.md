# 185 M7 Slice: Header Name Literal Token Hardening

This chapter documents a focused M7 hardening step for constant `headers.name(...)` literals.

## What it is

`headers.name("...")` now rejects literal values containing invalid token characters (for example whitespace or `:`). Allowed literal characters are ASCII alphanumerics plus `-`.

Violations emit `E4001` diagnostics tagged with `security` + `sink`.

## Why it exists

Typed header constructors already enforce arity and string input. This slice adds a static token check for constant literals to reduce malformed header-name construction early.

## How it works internally

In `enforce_header_builder_signatures(...)`:
1. existing `headers.name` one-argument string checks run first,
2. for string literals, the analyzer validates token characters,
3. invalid tokens emit a dedicated diagnostic.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/semantic.rs`
  - semantic fixture: `invalid_headers_name_literal_characters.ut`
  - tag test coverage in `compiler/sec4-core/tests/diagnostic_tags.rs`
- Outputs:
  - compile-time rejection for malformed header-name literals.
- Constraint:
  - this is literal-only validation; dynamic names still rely on runtime/header-gate validation.

## Failure modes and diagnostics

Example:
- `headers.name("X Bad:Name")` -> `E4001`: headers.name literal contains invalid characters.

## Example usage

```ut
fn ok() -> Int {
  headers.name("X-Trace-Id");
  0
}
```

## Tradeoffs and next steps

- Tradeoff: strict literal token checks reject non-standard header-name forms during compile-time.
- Next:
  - expand diagnostics with offending-character snippets to improve fix suggestions.
