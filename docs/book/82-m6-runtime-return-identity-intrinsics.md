# 82 M6 Slice: Runtime Return Identity Intrinsics

This chapter documents the next M6 vertical slice: routing generated C return values through typed runtime ABI intrinsics.

## Runtime Return Identity Wiring

### What it is

Generated C now routes typed return expressions through runtime ABI identity functions:
- `Int`/`Int64` returns -> `ailang_rt_identity_i64(...)`
- `Bool` returns -> `ailang_rt_identity_bool(...)`

### Why it exists

This makes runtime ABI usage real in emitted code, not only linked as passive files. It verifies that MIR->C return paths can call runtime helpers deterministically, which is required before adding richer intrinsic/runtime behaviors.

### How it works internally

- C emitter computes a per-function return identity hook from declared function return type.
- On `MirTerminator::Return { value: Some(...) }`, emitter now generates:
  - `return <identity_fn>(<value>);` when the return type is supported.
  - plain `return <value>;` for unsupported/unknown types.
- Runtime identity functions were already scaffolded in `ailang_runtime.h/.c`; this slice integrates call sites with those ABI functions.

### Inputs, outputs, and constraints

- Input: MIR with typed function return metadata.
- Output: generated C return statements that explicitly call runtime identity intrinsics for supported scalar types.
- Constraints:
  - mapping is currently limited to `Bool`, `Int`, and `Int64`.
  - complex/domain types still return directly until dedicated runtime ABI conversion rules are introduced.

### Failure modes and diagnostics

- If runtime artifacts are missing/unwritable, build still fails via existing C compile pipeline diagnostics.
- If type mapping is unsupported, emitter falls back to direct return expression (no extra runtime wrapping in this slice).

### Example usage

AILang input:

```ailang
fn truthy(flag: Bool) -> Bool {
  flag
}
```

Generated C return (simplified):

```c
return ailang_rt_identity_bool(flag);
```

### Tradeoffs and next steps

- Identity calls are semantically neutral; they exist to establish the ABI dispatch path.
- Extra call wrapping adds small overhead but keeps C emission and runtime contracts explicit.
- Next step: replace identity stubs with real runtime-backed intrinsics and widen lowering coverage for stdlib/effectful operations.

## Tests updated

- `compiler/ailang-core/tests/c_backend.rs`
  - existing return assertions now verify runtime identity calls
  - new test `c_backend_routes_bool_returns_through_runtime_identity`
- Existing CLI clang-gated compile test remains green with runtime-linked return wrappers.
