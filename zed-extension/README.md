# AILang Zed Extension (Scaffold)

This directory contains an initial scaffold for Zed integration:

- `extension.toml`:
  - registers `AILang` grammar metadata,
  - registers `ailang-lsp` language server wiring.
- `languages/ailang/config.toml`:
  - language name/grammar/file suffix/comment/bracket config.
- `src/lib.rs`:
  - launches `ailang-language-server --stdio` for Zed LSP requests.

## Notes

- `grammars.ailang.rev` is a placeholder and must be updated to a real commit SHA from the `tree-sitter-ailang` grammar repository.
- This scaffold is intentionally minimal and focused on LSP wiring.

## Validation

Use the repository check script before release:

```bash
scripts/check-zed-grammar-pin.sh
```

The script fails when `grammars.ailang.rev` is missing, placeholder, or not commit-like.
