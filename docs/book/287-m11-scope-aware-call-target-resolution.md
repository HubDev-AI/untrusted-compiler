# 287 M11 Slice: Scope-Aware Call-Target Resolution

This chapter documents making LSP call-target resolution scope-aware so shadowed local names are not treated as function symbols.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- cursor target resolution (`definition`/`hover`/`references`/`prepareRename`/`rename` entry path) is now:
  - callsite-only (callee identifiers),
  - scope-aware (respects params, `let` bindings, and match-pattern bindings).
- shadowed names in local scope no longer resolve to top-level function declarations.
- callsite hit collection and position-hit resolution now share consistent shadowing semantics.

## Why it exists

Name-based resolution could misclassify local shadowed identifiers as function references, producing incorrect rename/navigation behavior. Scope-aware resolution reduces false positives and is a practical step toward true symbol-ID binding.

## How it works internally

1. Maintain lexical scope stack while traversing function bodies.
2. Register bindings from:
   - function params,
   - `let` statements (after initializer evaluation),
   - match-arm patterns.
3. Treat callsite callee as function target only if its identifier is not shadowed in active scope.
4. Return `null`/empty results for shadowed targets in navigation and rename flows.

## Inputs, outputs, and constraints

- Inputs:
  - AST traversal state + cursor position + target function name.
- Outputs:
  - more precise function-target selection for LSP requests.
- Constraints:
  - still scoped to current language subset and name-based declaration lookup.

## Failure modes and diagnostics

- ambiguous/missing declaration behavior remains unchanged (safe null/empty responses).
- non-callee identifiers remain out of function-target resolution scope by design.

## Tests added/updated

`compiler/ailang-lsp` now includes:
- `definition_returns_null_for_shadowed_function_name_call`
- `prepare_rename_returns_null_for_shadowed_function_name_call`

Existing precision tests remain green.

## Tradeoffs and next steps

- Tradeoff:
  - resolution precision is significantly better without full symbol table binding.
- Next:
  - complete symbol-ID-driven callsite binding end-to-end for references/rename/navigation.
