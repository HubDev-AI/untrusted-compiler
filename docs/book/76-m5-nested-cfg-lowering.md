# 76 M5 Slice: Nested CFG Lowering

This chapter documents the next M5 step: recursive MIR lowering for nested control-flow expressions inside branch bodies and block tails.

Follow-up MIR block-id normalization is documented in `docs/book/77-m5-canonical-block-id-normalization.md`.

## Scope delivered
- MIR lowering now recursively lowers nested `if`/`match` expressions in:
  - return-context block tails
  - continuation-context block tails
  - nested branch/arm expression values
- Return and continuation lowering now share recursive CFG helpers.
- Nested control flow no longer falls back to inline expression strings for these paths.

## What changed
- Introduced recursive lowering helpers in `mir.rs`:
  - `lower_block_to_return_blocks`
  - `lower_block_to_continuation_blocks`
  - `lower_expr_to_return_blocks`
  - `lower_expr_to_continuation_blocks`
- Tail and explicit-return lowering now route through the same recursive expression lowering path.
- Statement-level continuation lowering composes with nested branch lowering and keeps explicit CFG edges (`branch`/`switch`/`goto`).

## Why this matters
- MIR control flow is now substantially closer to backend-ready normalization.
- Nested branching structure is explicit in MIR blocks, improving debuggability and lowering correctness.
- This reduces backend complexity because less control flow must be recovered from expression text.

## Tests added
- Core MIR tests:
  - `mir_lowering_recurses_nested_statement_if_in_branch_tail`
  - `mir_lowering_recurses_nested_return_if_match`
- MIR fixture/golden coverage:
  - `valid_nested_stmt_if.ut` + `.golden`
  - `valid_nested_return_if_match.ut` + `.golden`

## Tradeoffs
- Continuation block ids are reserved early in statement-context lowering, so block numbering may place join blocks before branch-arm ids.
- Some expression rendering remains for non-control-flow values (expected in current MIR stage).

## Next step
- Normalize block ids after lowering so nested CFG output remains compact and deterministic for tooling/backends.
