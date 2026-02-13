# tree-sitter-ailang (Scaffold)

This directory provides a minimal Tree-sitter grammar scaffold for AILang.

Included:
- `grammar.js` with baseline parsing for:
  - function declarations
  - blocks/statements
  - identifiers, numbers, strings, comments
- `queries/highlights.scm` with basic highlighting captures
- `queries/indents.scm` with baseline block indentation hints
- `queries/outline.scm` with function outline capture
- `queries/textobjects.scm` with basic function textobjects
- `package.json` with `generate`/`test` scripts

## Notes

- This is a bootstrap grammar, not full language coverage.
- Expand grammar and query files as parser/type surface grows.
- Once published as a standalone repository, update `zed-extension/extension.toml` with the final repo URL and commit SHA.
