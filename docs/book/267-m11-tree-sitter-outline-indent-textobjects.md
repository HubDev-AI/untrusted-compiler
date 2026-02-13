# 267 M11 Slice: Tree-Sitter Outline/Indent/Textobjects

This chapter documents expanding the Tree-sitter query surface beyond syntax highlighting.

## What it is

Updated:
- `tree-sitter-ailang/queries/indents.scm`
- `tree-sitter-ailang/queries/outline.scm`
- `tree-sitter-ailang/queries/textobjects.scm`
- `tree-sitter-ailang/README.md`

Key additions:
- block indent/outdent hints,
- function outline capture,
- baseline function textobjects.

## Why it exists

M11 editor experience is not only diagnostics/navigation. Structural editor features (outline, indentation behavior, textobjects) depend on query coverage. This slice improves practical editing ergonomics for AILang files in Tree-sitter-aware clients.

## How it works internally

1. `indents.scm` maps `{`/`}` block boundaries to indent/outdent captures.
2. `outline.scm` maps function declarations to item/name captures.
3. `textobjects.scm` maps function scope captures (`around`/`inside`).
4. Query files are consumed by editor integrations (for example Zed/Vim-like motions where supported).

## Inputs, outputs, and constraints

- Inputs:
  - parse tree nodes produced by `tree-sitter-ailang`.
- Outputs:
  - additional structural metadata for editor UX.
- Constraints:
  - grammar/query coverage is still intentionally baseline.

## Tradeoffs and next steps

- Tradeoff:
  - broad coverage was deferred in favor of a minimal reliable starting set.
- Next:
  - extend queries to schemas/enums/match blocks as grammar expands,
  - validate query behavior end-to-end in Zed extension smoke tests.
