# 280 M11 Slice: Callsite-Focused Reference Hit Collection

This chapter documents tightening rename/reference hit collection to function callsites instead of all matching identifiers.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- identifier hit collection now records function references from call callee positions only.
- non-callee identifiers that happen to share the same name are excluded from function reference hit sets.
- rename/reference flows now rely on this more precise callsite-oriented hit model.

## Why it exists

Name-only identifier matching over-collected hits and could include non-reference identifiers. Restricting hits to callsite callees is a concrete precision step toward symbol-ID-based navigation.

## How it works internally

1. Traverse expression trees as before.
2. For `Call` expressions:
   - if callee is `Identifier(target_name)`, capture that span as a reference hit,
   - continue traversal for nested expressions/arguments.
3. For plain `Identifier` expressions, do not record hits in function-reference mode.

## Inputs, outputs, and constraints

- Inputs:
  - parsed AST and target symbol name.
- Outputs:
  - narrower hit set aligned to function callsites.
- Constraints:
  - this remains name-based matching; full symbol-ID callsite binding is still pending.

## Failure modes and diagnostics

- call patterns not represented as direct identifier callees (for example advanced callable values) may still require future binding refinement.

## Tests added/updated

`compiler/ailang-lsp` now includes:
- `callsite_hit_collection_ignores_non_callee_identifiers`:
  - validates only direct callee hit is collected (argument identifier with same name is excluded).

## Tradeoffs and next steps

- Tradeoff:
  - precision improved significantly for common function call patterns.
- Next:
  - attach callsites to declaration symbol IDs directly to complete end-to-end symbol-ID precision for definition/references/rename.
