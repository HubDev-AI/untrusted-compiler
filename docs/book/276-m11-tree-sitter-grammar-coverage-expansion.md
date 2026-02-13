# 276 M11 Slice: Tree-sitter Grammar Coverage Expansion

This chapter documents expanding the AILang tree-sitter grammar/query coverage beyond the initial scaffold.

## What it is

Updated:
- `tree-sitter-ailang/grammar.js`
- `tree-sitter-ailang/queries/highlights.scm`
- `tree-sitter-ailang/queries/outline.scm`

Key changes:
- function signatures now parse optional `effects { ... }` clauses.
- grammar now supports:
  - dotted effect identifiers (for example `db.write`),
  - member expressions (`ctx.log.info`),
  - call expressions over identifiers and member expressions,
  - boolean literals.
- query coverage improved with structured captures:
  - function names, function calls, parameter names/types, return types, effects, member properties.
- outline query now anchors function names via named field capture.

## Why it exists

The initial grammar was intentionally minimal and did not parse several common AILang forms used in security/effects-first code. This caused incomplete highlighting/navigation structure in editor clients.

## How it works internally

1. Extend grammar production rules for effects/member-call syntax.
2. Introduce fielded nodes (`name`, `type`, `return_type`, `callee`, `property`) for stable query targeting.
3. Expand highlight queries to classify important symbols/types/effects in a predictable way.
4. Keep outline query aligned to new fielded function declaration shape.

## Inputs, outputs, and constraints

- Inputs:
  - AILang source with function/effects/member-call syntax.
- Outputs:
  - richer parse tree nodes and editor-facing highlight/outline captures.
- Constraints:
  - grammar remains a pragmatic v0 subset, not full language coverage yet.

## Failure modes and diagnostics

- if grammar tooling is unavailable locally, parser/query validation cannot be executed in CI-like mode.
- runtime/editor behavior still depends on the grammar revision pinned in extension metadata.

## Tests added/updated

- Attempted: `npm --prefix tree-sitter-ailang test`
- Result: blocked in this environment (`tree-sitter` CLI missing).

## Tradeoffs and next steps

- Tradeoff:
  - improved coverage now, but still no pinned published grammar revision in extension metadata.
- Next:
  - publish grammar revision and pin `zed-extension/extension.toml` to immutable commit SHA,
  - add grammar corpus fixtures and run them in CI once `tree-sitter` tooling is available.
