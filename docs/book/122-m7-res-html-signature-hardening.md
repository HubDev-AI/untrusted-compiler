# 122 M7 Slice: `res.html` Signature Hardening

This chapter documents a focused M7 signature-contract hardening step for HTML response sinks.

## What it is

Added semantic call-shape enforcement for `res.html`:
- call form must be `res.html(htmlSafeValue)`,
- argument type must be `HtmlSafe` (not raw `String`, numeric placeholders, `Untrusted<_>`, etc.).

Diagnostics for violations are emitted as `E4001` and tagged as `security` + `sink`.

## Why it exists

Bridge-stage integration fixtures still accepted placeholder forms like `res.html(1)`. That weakens the sink contract and drifts from the security-first requirement that HTML output must be explicitly safe.

This slice locks the contract at compile time before runtime behavior gets richer.

## How it works internally

In semantic intrinsic enforcement:
1. added `enforce_res_html_signature(...)` alongside existing req/res call-shape checks,
2. routed trust-gate enforcement through this helper,
3. enforced:
   - exact arity (`1`),
   - argument type `HtmlSafe`,
4. tagged diagnostics with `security` and `sink` for editor/tooling consumers.

`is_res_html_call(...)` now recognizes both canonical forms:
- `res_html`
- `res.html`

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/semantic.rs`
  - semantic fixture: `invalid_res_html_requires_htmlsafe.ai`
  - diagnostic tag coverage in `compiler/ailang-core/tests/diagnostic_tags.rs`
  - req/res CLI integration fixture in `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - compile-time rejection of non-`HtmlSafe` `res.html(...)` calls,
  - req/res integration path updated to a valid gate flow:
    - `req.query(...)` -> `sanitize.html(...)` -> `res.html(...)`
- Constraint:
  - this is still bridge-stage HTTP behavior; richer response typing can be layered later.

## Failure modes and diagnostics

Example:
- `res.html(1)` -> `E4001` (`res.html argument must be HtmlSafe`) with fix-oriented note to use `sanitize.html(...)` or another `HtmlSafe` gate.

Arity mismatch example:
- `res.html(a, b)` -> `E4001` (`res.html expects exactly one argument`).

## Example usage

```ailang
fn render() effects { net } -> Int {
  let raw = req.query(1, 2);
  let safe = sanitize.html(raw);
  res.html(safe);
  0
}
```

## Tradeoffs and next steps

- Tradeoff: stricter sink contracts require fixture/sample updates where placeholders were used.
- Next:
  - continue tightening req/res contracts (`res.setHeader`, `res.addCookie`) to typed constructors only,
  - surface these sink diagnostics via LSP quick-fixes in upcoming editor-tooling milestones.
