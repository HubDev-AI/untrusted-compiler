# 265 M11 Slice: Unopened Workspace-File References and Rename

This chapter documents extending references/rename resolution beyond open documents to include unopened `.ut` files in the project workspace.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`

Key changes:
- workspace scan now includes unopened `.ut` files discovered from project root.
- project root detection:
  - walk upward from active file URI until `sec4.toml` is found,
  - fallback to active file directory.
- references/rename/declaration lookup now operate on:
  - open document state,
  - plus discovered unopened workspace files (bounded by request deadline).

## Why it exists

Open-document-only resolution was insufficient for realistic refactors where declarations/usages exist in files not currently opened by the editor. This slice improves determinism for multi-file workflows and better aligns with M11 rename/reference expectations.

## How it works internally

1. Build base document set from `state.documents` + active document.
2. Discover workspace `.ut` files via recursive directory scan from project root.
3. Add unopened file entries not already present in open-document set.
4. Apply existing parse/cache, deadline checks, and symbol collection logic across the expanded set.

## Inputs, outputs, and constraints

- Inputs:
  - active file URI (used for root detection and request context),
  - on-disk workspace files.
- Outputs:
  - references include hits from unopened files,
  - rename emits workspace edits for unopened files when matches exist.
- Constraints:
  - workspace scan is filesystem-based and currently rooted per active file.
  - matching remains name-based in this slice.

## Failure modes and diagnostics

- unreadable files/directories are skipped safely.
- deadline expiry truncates scan deterministically.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- references request including hits from an unopened file in a temp workspace.
- rename request emitting edits for an unopened file in a temp workspace.

## Tradeoffs and next steps

- Tradeoff:
  - recursive filesystem scanning improves coverage but can become expensive on large repos.
- Next:
  - replace ad-hoc scans with indexed workspace graph and persistent symbol index,
  - upgrade name-based matching to stable symbol IDs for higher rename precision.
