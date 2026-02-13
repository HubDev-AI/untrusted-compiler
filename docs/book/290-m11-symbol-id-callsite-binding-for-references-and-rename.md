# 290 M11 Slice: Symbol-ID Callsite Binding for References and Rename

This chapter documents symbol-identity-based callsite binding for definition, hover, references, prepare-rename, and rename.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`
- `docs/05-sec4-master-roadmap.md`

Key changes:
- Added local-first symbol resolution helper flow:
  - resolve call target to a concrete `symbolId` in the active document first,
  - fallback to workspace-unique resolution only when local declaration is absent.
- Added declaration lookup by `symbolId` and switched definition/hover to use it.
- Updated references and rename to collect callsites only from documents that resolve the call target to the same `symbolId`.
- Added LSP tests proving duplicate-name symbols in different files no longer cross-bind references/rename edits.

## Why it exists

M11 still had a binding precision gap: callsite collection was call-name based, so duplicate function names could leak cross-file hits or block deterministic editing behavior.

This slice makes symbol identity the primary key for navigation and rename behavior.

## How it works internally

1. For a callsite identifier under cursor, the server resolves target symbol identity:
   - prefer exactly one local declaration in the current document,
   - otherwise require exactly one workspace declaration.
2. The resolved `symbolId` is used to find the declaration location.
3. References/rename scan workspace documents, but only include callsites from documents that resolve the same call name to the same `symbolId`.
4. Ambiguous target resolution still returns safe null/empty responses.

## Tests added or updated

Updated:
- `compiler/sec4-lsp/src/main.rs`
  - `references_bind_to_local_symbol_id_when_workspace_has_duplicate_names`
  - `rename_binds_to_local_symbol_id_when_workspace_has_duplicate_names`

Existing ambiguity guard tests remain green:
- `definition_returns_null_when_workspace_declaration_is_ambiguous`
- `rename_returns_empty_changes_when_workspace_declaration_is_ambiguous`

## Tradeoffs and next steps

- Tradeoff:
  - current symbol binding is still function-call focused and intentionally conservative when resolution is ambiguous.
- Next:
  - finish remaining M11 items:
    - parser-backed dependency invalidation for unopened/module-wide changes,
    - fully AST-aware code-action rewrites for complex signature edits.
