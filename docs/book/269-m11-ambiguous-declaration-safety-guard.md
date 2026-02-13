# 269 M11 Slice: Ambiguous Declaration Safety Guard

This chapter documents ambiguity-safe declaration resolution behavior for workspace-aware navigation and rename.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- workspace declaration lookup now treats duplicate declaration names as ambiguous.
- when ambiguous:
  - `definition` returns `null`,
  - `prepareRename` returns `null`,
  - `rename` returns empty change set.

## Why it exists

Name-based symbol matching can produce incorrect edits/navigation in the presence of duplicate symbol names. Failing closed is safer than applying potentially wrong cross-file edits.

## How it works internally

1. Workspace declaration scan tracks first matching declaration.
2. If a second declaration with same name is found, lookup is marked ambiguous and returns `None`.
3. Callers interpret missing declaration as non-actionable for risky operations (rename/definition).

## Inputs, outputs, and constraints

- Inputs:
  - workspace declarations sharing the same symbol name.
- Outputs:
  - safe no-op/null responses instead of guessing.
- Constraints:
  - ambiguity handling is conservative and may suppress actions in valid overloaded-like scenarios not yet modeled.

## Failure modes and diagnostics

- duplicate declaration names intentionally block rename automation in this slice.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- ambiguous declaration returns `null` for definition.
- ambiguous declaration returns empty edits for rename.

## Tradeoffs and next steps

- Tradeoff:
  - conservative ambiguity handling avoids incorrect edits but can reduce feature availability.
- Next:
  - replace name-based matching with stable symbol IDs and module-aware resolution to recover precision without unsafe guessing.
