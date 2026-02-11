# 42 Structural Composition, Not Inheritance

This chapter records the M2 alignment with AILang's no-inheritance direction.

## What it is

A type-checking approach centered on explicit data shapes and function signatures, without trait/interface hierarchy mechanics.

## Why it exists

AILang v0.1-lite targets TypeScript-like ergonomics while avoiding inheritance/trait complexity and hidden implementation graphs.

## How it works internally

- Type references are resolved by explicit names and arity.
- Behavior is modeled through concrete functions and values, not nominal inheritance trees.
- Pattern matching and constructor checks operate on explicit enum variants.

## Inputs/outputs and constraints

- Input: concrete AST declarations and expressions.
- Output: explicit acceptance/rejection diagnostics.
- Constraint: no nominal trait system, no interface-implementation resolution graph in M2.

## Failure modes and diagnostics

- Unknown names/types are direct semantic errors.
- Variant mismatches surface as typed pattern/call diagnostics.
- Missing match coverage is reported as `T3107`.

## Example usage

Use `enum` + `match` for variation and explicit behavior boundaries:

```ailang
enum State { On, Off }

fn to_int(s: State) -> Int {
  match s {
    On => 1,
    Off => 0,
  }
}
```

## Tradeoffs and next steps

- Tradeoff: fewer abstraction mechanisms in v0.1-lite.
- Benefit: simpler semantics and clearer AI-generated code.
- Next: introduce structural shape compatibility checks for capability-object style APIs.
