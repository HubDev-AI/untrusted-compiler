# Untrusted<T> Zed Extension

This directory contains the official Zed integration for Untrusted<T>.

## What it provides

- `extension.toml`:
  - registers `Untrusted<T>` grammar metadata,
  - registers `sec4audit-lsp` language server wiring.
- `languages/untrusted/config.toml`:
  - language name/grammar/file suffix/comment/bracket config.
- `src/lib.rs`:
  - launches `sec4audit-language-server --stdio` for Zed LSP requests.

## Feature surface (from LSP)

- diagnostics (`publishDiagnostics`)
- go to definition / hover
- references / rename
- quickfix code actions
- document formatting (`textDocument/formatting`)

## Local smoke project

Use:

- `/Users/vladimirtrifonov/src/ai/AILang/examples/zed-plugin-smoke`

It includes multi-file `.ut` modules plus playground files for diagnostics and formatter checks.

## Notes

- `grammars.untrusted.rev` is pinned to a commit SHA and should be updated whenever grammar changes are intentionally rolled forward.

## Validation

Use the repository check script before release:

```bash
scripts/check-zed-grammar-pin.sh
```

The script fails when `grammars.untrusted.rev` is missing, placeholder, or not commit-like.
