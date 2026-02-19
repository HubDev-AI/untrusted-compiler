# 1036 M39 Slice: Zed Plugin Formatting and Smoke Project

This slice upgrades the Zed editor path from scaffold-only wiring to a practical day-one workflow: LSP document formatting plus a dedicated plugin smoke project.

## What changed

1. Implemented LSP formatting in `sec4audit-language-server`:
   - advertises `documentFormattingProvider` during `initialize`,
   - handles `textDocument/formatting`,
   - emits deterministic full-document edits using the same baseline policy as `sec4 fmt` (trim trailing whitespace + enforce final newline).
2. Added language-server coverage for formatting:
   - formatting returns one full-document edit when content needs rewriting,
   - formatting returns an empty edit list when content is already canonical.
3. Added `examples/zed-plugin-smoke/`:
   - multi-file module project for definition/references/rename checks,
   - `playground/lsp-errors.ut` for diagnostics + quickfix checks,
   - `playground/format-me.ut` for format-document checks,
   - operator README with step-by-step Zed validation flow.
4. Updated extension docs to describe active feature surface and point to the new smoke project.

## Why

The previous extension scaffold proved process wiring, but not day-to-day editor usability. Formatting and a concrete smoke fixture close the main usability gap and let operators validate the plugin experience quickly.

## Validation

- `cargo test -p sec4audit-language-server`
- `sec4 check --path examples/zed-plugin-smoke`
- `sec4 fmt --path examples/zed-plugin-smoke`
- `sec4 check --path examples/zed-plugin-smoke`
