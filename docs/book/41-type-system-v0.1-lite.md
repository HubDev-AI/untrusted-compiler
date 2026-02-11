# 41 Type System (v0.1-lite)

This chapter documents the M2 minimal type checker in `compiler/ailang-core/src/semantic.rs`.

## What it is

A structural, explicit type checker for M1 AST covering variable bindings, returns, operators, calls, and `match` expressions.

## Why it exists

M2 must reject basic semantic bugs with deterministic diagnostics before MIR/backend work starts.

## How it works internally

- Resolves declared types into internal semantic types.
- Infers expression types for literals, unary/binary ops, blocks, calls, and control-flow expressions.
- Validates:
  - variable annotation vs assigned value type
  - return expressions vs function return type
  - call argument count/type compatibility
  - `if` condition type (`Bool`) and branch compatibility
  - `match` arm type compatibility
- Supports built-in constructors:
  - `Some` / `None` -> `Option<T>`
  - `Ok` / `Err` -> `Result<T, E>`

## Inputs/outputs and constraints

- Input: AST and symbol catalog.
- Output: semantic diagnostics or type-checked acceptance.
- Constraint: type inference is intentionally shallow (function-local, no polymorphic inference engine).

## Failure modes and diagnostics

- `T3101`: type mismatch (bindings, operators, branch/call compatibility)
- `T3102`: non-boolean `if` condition
- `T3103`: argument count mismatch (calls/constructors/pattern payload)
- `T3104`: unknown/unsupported callable target
- `T3105`: return or function-body type mismatch
- `T3107`: non-exhaustive `match`
- `T3108`: pattern type mismatch
- `T3109`: unknown enum variant in pattern

## Example usage

```bash
cargo test -p ailang-core --test golden_semantic
```

The semantic golden suite includes unknown-name, type-mismatch, and non-exhaustive `match` fixtures.

## Tradeoffs and next steps

- Tradeoff: no flow-sensitive nullability or advanced inference yet.
- Tradeoff: no full generic unification.
- Next: add richer type narrowing and stricter constructor typing as M2 evolves.
