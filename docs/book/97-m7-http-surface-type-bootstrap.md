# 97 M7 Slice: HTTP Surface Type Bootstrap

This chapter documents the next M7 bootstrap slice: recognizing HTTP-oriented type names in semantic analysis so API-shaped signatures compile cleanly.

## HTTP Surface Primitive Types

### What it is

Added these names to the semantic primitive type catalog:
- `Router`
- `Request`
- `Response`
- `HttpError`
- `Handler`

### Why it exists

M7 docs and examples are HTTP-first. Without these names in the catalog, API-shaped function signatures trigger unknown-type diagnostics and force placeholder types in early runtime slices.

### How it works internally

- Extended `Catalog::new()` primitive type seed list in `semantic.rs`.
- No runtime behavior changes were needed for this slice.
- Added CLI integration coverage to ensure projects using these type names compile through `c-bin`.

### Inputs, outputs, and constraints

- Input: AILang signatures referencing HTTP surface types.
- Output: successful semantic type resolution and compile pipeline progression.
- Constraints:
  - backend still lowers unknown runtime structures to placeholder C scalar forms in M7 bootstrap mode.
  - this slice is type-name recognition only, not full typed runtime struct semantics.

### Failure modes and diagnostics

- Unknown-type diagnostics (`N3001`) are now avoided for this specific HTTP surface set.
- Other undeclared user-defined types still fail as before.

### Example usage

```ailang
fn wireRoutes(router: Router, request: Request, response: Response) effects { net } -> Int {
  http.get(router, 1, 1);
  http.post(router, 1, 1);
  http.serve(1, router);
  0
}
```

### Tradeoffs and next steps

- This improves ergonomics and alignment with the spec but does not yet enforce rich runtime contracts for these types.
- Next step: introduce concrete runtime representations/ABI conventions for request/response/router values.

## Tests updated

- `compiler/ailang-cli/tests/json_output.rs`
  - added `build_emit_c_bin_accepts_http_surface_types_when_clang_available`
