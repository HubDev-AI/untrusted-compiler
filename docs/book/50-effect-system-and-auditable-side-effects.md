# 50 Effect System and Auditable Side Effects

This chapter documents the implemented M3 effects slice.

## What it is

Untrusted<T> now supports function effect declarations via `effects { ... }` and compiler checks that used effects are a subset of declared effects.

## Why it exists

Effect declarations make side effects explicit and machine-checkable, which is required for auditable backend code and safer AI-generated implementations.

## How it works internally

- Parser accepts effect clauses in function signatures:
  - `fn handler() effects { db.write, log } -> Int { ... }`
- AST stores effect paths on each function declaration.
- Semantic analyzer:
  - validates effect names against the known v0 set,
  - rejects duplicate effect declarations,
  - collects used effects from function calls and selected intrinsics,
  - rejects functions that use undeclared effects.

Used effects currently propagate from:
- calls to user-defined functions (callee declared effects),
- built-in intrinsic call names mapped to effects (`log`, `time_now`, `net_call`, `secret_read`, `secret_reveal`, `db_read`, `db_write`, `fs_read`, `fs_write`).

## Inputs/outputs and constraints

- Input: parsed AST with function effect declarations.
- Output: semantic pass success or effect diagnostics.
- Constraints:
  - single-file semantic scope in current implementation,
  - effect checking is declaration and usage based (no capability-object type checking yet).

## Failure modes and diagnostics

- `E4001`: unknown effect name
- `E4002`: effect used but not declared
- `E4003`: duplicate effect declaration

Example failing case:
- a function calls another function that requires `db.write`, but caller has no `effects { db.write }` declaration.

## Example usage

```ut
fn do_write() effects { db.write } -> Int {
  1
}

fn handler() effects { db.write } -> Int {
  do_write()
}
```

## Tradeoffs and next steps

- Tradeoff: effect usage currently derives primarily from function calls and a fixed intrinsic map.
- Tradeoff: no capability-field effect propagation yet.
- Next:
  - extend effect propagation through structural capability object calls,
  - integrate boundary-specific effects with trust-gate rules in the next M3 slice.
