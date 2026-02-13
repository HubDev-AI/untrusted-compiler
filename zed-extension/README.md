# Untrusted<T> Zed Extension (Scaffold)

This directory contains an initial scaffold for Zed integration:

- `extension.toml`:
  - registers `Untrusted<T>` grammar metadata,
  - registers `sec4audit-lsp` language server wiring.
- `languages/untrusted/config.toml`:
  - language name/grammar/file suffix/comment/bracket config.
- `src/lib.rs`:
  - launches `sec4audit-language-server --stdio` for Zed LSP requests.

## Notes

- `grammars.untrusted.rev` is pinned to a commit SHA and should be updated whenever grammar changes are intentionally rolled forward.
- This scaffold is intentionally minimal and focused on LSP wiring.

## Validation

Use the repository check script before release:

```bash
scripts/check-zed-grammar-pin.sh
```

The script fails when `grammars.untrusted.rev` is missing, placeholder, or not commit-like.
