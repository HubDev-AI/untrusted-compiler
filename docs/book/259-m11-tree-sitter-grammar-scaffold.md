# 259 M11 Slice: Tree-Sitter Grammar Scaffold

This chapter documents adding a bootstrap `tree-sitter-untrusted` project to support syntax highlighting integration for editor clients (starting with Zed).

## What it is

Added:
- `tree-sitter-untrusted/package.json`
- `tree-sitter-untrusted/grammar.js`
- `tree-sitter-untrusted/queries/highlights.scm`
- `tree-sitter-untrusted/README.md`

Key scope:
- baseline grammar rules for:
  - function declarations,
  - parameters/types,
  - blocks/statements,
  - identifiers/calls/literals/comments.
- baseline highlight captures for keywords, comments, strings, numbers, identifiers.

## Why it exists

LSP covers diagnostics/navigation/completion, but editor UX also depends on syntax highlighting and structural parsing. This scaffold establishes a concrete grammar project that can be expanded incrementally with language growth.

## How it works internally

1. `grammar.js` defines core parse rules and token classes.
2. `queries/highlights.scm` maps parsed nodes/tokens to highlight categories.
3. `package.json` defines local generation/test commands (`tree-sitter generate`, `tree-sitter test`).
4. `zed-extension` can reference this grammar repository once published/pinned.

## Inputs, outputs, and constraints

- Inputs:
  - Untrusted<T> source text.
- Outputs:
  - parse trees + highlight captures via Tree-sitter tooling.
- Constraints:
  - grammar is intentionally partial and bootstrap-level in this slice.
  - does not yet model full Untrusted<T> syntax surface (effects, schemas, enums, match arms, etc.).

## Verification

- LSP regression suite remains green:
  - `cargo test -p sec4audit-language-server`

## Tradeoffs and next steps

- Tradeoff:
  - bootstrap grammar accelerates integration but is not feature-complete.
- Next:
  - publish grammar as standalone repository,
  - pin real commit SHA in `zed-extension/extension.toml`,
  - expand grammar coverage with query files (`indents.scm`, `outline.scm`, `textobjects.scm`).
