# 75 M5 Slice: Statement Control-Flow Continuations

This chapter documents the next M5 step: lowering statement-level control flow into explicit continuation blocks.

## Scope delivered
- Function-body expression statements now lower structured control flow for:
  - `if ... { ... } else ...;`
  - `match ... { ... };`
- MIR now uses `goto` continuation blocks for non-tail control flow.
- Existing tail and return-site control-flow lowering remains in place.

## What changed
- `lower_function` now maintains a block id allocator and current block cursor.
- Statement-level `if` lowers to:
  - current block terminator `branch ... ? bb_then : bb_else`
  - branch blocks that end with `goto bb_cont` unless they explicitly `return`.
- Statement-level `match` lowers to:
  - current block terminator `switch ...`
  - one block per arm ending with `goto bb_cont` unless explicitly `return`.
- Tail and explicit return control-flow lowering now uses dynamic block ids, so these forms compose with prior statement-level splits.

## Why this matters
- MIR now models non-tail branching without falling back to inline expression rendering.
- Backends can consume explicit CFG edges (`branch`/`switch`/`goto`) instead of recovering control flow from expression text.
- This is a direct step toward full MIR normalization for later C backend emission.

## Tests added
- Core MIR tests:
  - `mir_lowering_splits_statement_if_into_continuation_blocks`
  - `mir_lowering_splits_statement_match_into_continuation_blocks`
- MIR fixture/golden coverage:
  - `valid_stmt_if.ai` + `.golden`
  - `valid_stmt_match.ai` + `.golden`

## Tradeoffs
- Branch-body lowering still uses expression rendering for nested control-flow expressions that are not yet recursively CFG-lowered.
- Continuation blocks may be emitted even when both predecessor branches return.

## Next step
- Add recursive/nested control-flow CFG lowering inside branch bodies and block tails to reduce remaining inline-expression fallback paths.
