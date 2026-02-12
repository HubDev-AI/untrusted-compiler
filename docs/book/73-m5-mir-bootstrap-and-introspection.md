# 73 M5 Bootstrap: MIR Lowering and Introspection (Current Slice)

This chapter documents the first M5 slice that introduces a backend-neutral MIR layer and an introspection path in the CLI.

## Scope delivered
- Added a new MIR module in `ailang-core`:
  - `MirProgram`
  - `MirFunction`
  - `MirBlock`
  - `MirInstruction` and `MirTerminator`
- Added AST -> MIR lowering entrypoint:
  - `lower_program_to_mir(&Program) -> MirProgram`
- Added first control-flow lowering step:
  - tail `if` expressions lower into branch terminators and separate blocks (`bb0`, `bb1`, `bb2`)
- Added deterministic textual MIR rendering:
  - `MirProgram::render_text()`
- Added CLI integration:
  - `ailang build --emit mir`
- Added tests:
  - core MIR lowering tests in `compiler/ailang-core/tests/mir.rs`
  - fixture-based MIR golden tests in `compiler/ailang-core/tests/golden_mir.rs`
  - CLI emit test in `compiler/ailang-cli/tests/json_output.rs`

## What it is
MIR is a compiler-internal, backend-neutral representation between AST/semantic phases and backend code generation. In this slice, MIR is intentionally minimal and inspection-first.

## Why it exists
- Creates a stable handoff layer before backend work.
- Makes lowering behavior visible and testable.
- Provides deterministic text output for debugging and future golden tests.

## How it works

### 1) Function selection
- MIR currently lowers function items only.
- Struct/enum declarations are not lowered in this slice.

### 2) Block model
- Linear functions lower to one block: `bb0`.
- Tail `if` expressions lower to branch form:
  - `bb0` with `branch <cond> ? bb1 : bb2`
  - `bb1` for then-branch
  - `bb2` for else-branch

### 3) Instruction lowering
- `let name = expr;` -> `MirInstructionKind::Let { name, value }`
- expression statement -> `MirInstructionKind::Eval { value }`
- `return expr;` -> block terminator `Return { value: Some(expr) }`

### 4) Terminator rule
- If an explicit `return` appears in statements, it becomes the terminator and lowering stops for subsequent statements.
- Otherwise:
  - use block tail expression as `return <tail>`
  - or bare `return` when no tail exists.

### 5) Expression rendering
- Expressions lower to deterministic textual forms (identifier, calls, binary/unary, member, block/if/match inline forms).
- This keeps introspection readable while backend-neutral data structures remain simple.

## CLI behavior

Build now supports:
- `ailang build --path <project> --emit mir`

Behavior:
1. project validation + semantic analysis run as before
2. lockfile stub is written as before
3. MIR is lowered from analyzed AST and printed in text form

## Example output shape
```text
fn main() -> Int
  bb0:
    return 0
```

## Tests added
- `mir_lowering_builds_single_block_for_simple_function`
- `mir_lowering_honors_explicit_return_terminator`
- `mir_fixtures_match_golden_output` (fixture + golden snapshots)
- `build_emit_mir_prints_textual_mir`

## Tradeoffs
- Current MIR is linear and single-block per function.
- Control-flow normalization into multiple basic blocks is deferred to next M5 slices.
- Expression values are currently represented via deterministic rendered forms; richer typed MIR values will come incrementally.

## Next steps
1. Introduce multi-block lowering for `if`/`match`.
2. Add explicit temporary locals and branch terminators.
3. Add MIR golden fixtures for deterministic regression checks.
4. Extend multi-block lowering beyond tail `if` into broader `if`/`match` placements.
