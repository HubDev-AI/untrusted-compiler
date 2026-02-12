# 74 M5 Slice: Return-Site Control-Flow Lowering

This chapter captures the next M5 vertical slice after MIR bootstrap: lowering control flow when it appears inside explicit `return` expressions.

Follow-up continuation-based statement control-flow lowering is documented in `docs/book/75-m5-statement-control-flow-continuations.md`.

## Scope delivered
- MIR lowering now recognizes explicit return expressions that are control flow:
  - `return if ... { ... } else { ... };`
  - `return match ... { ... };`
- These forms are lowered into explicit MIR control-flow blocks instead of inline expression text.
- Existing tail-expression lowering behavior is preserved.

## What changed
- `lower_function` now distinguishes simple return values from control-flow return values.
- `return if` lowers to:
  - entry block with `branch <cond> ? bb1 : bb2`
  - `bb1`/`bb2` branch blocks with explicit `return` terminators.
- `return match` lowers to:
  - entry block with `switch <scrutinee> { ... }`
  - one block per arm with explicit `return` terminators.
- Shared helpers now drive both tail and return-site lowering for:
  - if-branch block construction
  - match-switch block construction

## Why this matters
- Moves MIR closer to backend-ready control-flow normalization.
- Removes expression-string fallback for common non-tail control-flow returns.
- Keeps lowering deterministic and easier to inspect in text/JSON MIR outputs.

## Tests added
- Core MIR tests:
  - `mir_lowering_splits_return_if_into_branch_blocks`
  - `mir_lowering_splits_return_match_into_switch_blocks`
- MIR fixture/golden coverage:
  - `valid_return_if.ai` + `.golden`
  - `valid_return_match.ai` + `.golden`

## Tradeoffs
- This slice handles control flow at function return sites only.
- General statement-level lowering (for example non-tail `if`/`match` expressions requiring continuation/join blocks) is still pending.

## Next step
- Extend MIR with recursive nested CFG lowering inside branch bodies and block-tail expressions.
