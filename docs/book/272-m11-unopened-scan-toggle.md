# 272 M11 Slice: Unopened-File Scan Toggle

This chapter documents adding a runtime toggle for unopened workspace-file scanning in `sec4audit-language-server`.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`
- `compiler/sec4-lsp/README.md`

Key changes:
- new env-configured toggle:
  - `SEC4AUDIT_LSP_SCAN_UNOPENED_FILES`
- default remains enabled (`true`).
- false-like values disable scan:
  - `false`, `0`, `off`, `no` (case-insensitive).

## Why it exists

Unopened-file scanning improves feature coverage but can be expensive on large workspaces. This toggle gives operators a deterministic performance knob without changing code.

## How it works internally

1. Parse env value through normalized boolean parser.
2. If enabled, workspace assembly includes on-disk unopened `.ut` files.
3. If disabled, workspace assembly uses open-document set only.

## Inputs, outputs, and constraints

- Inputs:
  - process environment.
- Outputs:
  - feature/performance tradeoff mode at runtime.
- Constraints:
  - toggle is global per server process.

## Tests added/updated

`compiler/sec4-lsp` unit tests now also cover:
- boolean parser behavior for common true/false string forms.

## Tradeoffs and next steps

- Tradeoff:
  - disabling scan improves performance but can reduce cross-file navigation/edit coverage.
- Next:
  - move from scan-vs-no-scan to indexed workspace model with predictable bounds and precision.
