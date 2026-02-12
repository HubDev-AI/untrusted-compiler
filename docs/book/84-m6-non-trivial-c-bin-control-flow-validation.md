# 84 M6 Slice: Non-Trivial `c-bin` Control-Flow Validation

This chapter documents the next M6 vertical slice: proving the C backend pipeline on a non-trivial program shape (function calls plus control flow), not only the minimal hello case.

## Control-Flow + Call Integration Coverage

### What it is

A new clang-gated CLI integration test now builds and runs a temporary AILang project that includes:
- a helper function call
- `if/else` branch control flow
- a `main` function that returns through that helper path

### Why it exists

M6 exit criteria require runnable support beyond trivial linear returns. This test hardens evidence that the current MIR->C->clang pipeline handles basic real control-flow and cross-function call paths end-to-end.

### How it works internally

- Test creates a temporary project on disk:
  - `ailang.toml`
  - `src/main.ai` with `choose(flag: Bool)` + `main()`
- Runs:
  - `ailang build --emit c-bin --path <temp-project>`
- Validates:
  - generated C contains expected function signature and branch/call patterns
  - compiled binary exists
  - binary executes successfully

### Inputs, outputs, and constraints

- Input: temporary project source exercising call + branch.
- Output:
  - compiled binary in `<temp>/build/flowdemo`
  - generated C in `<temp>/build/generated.c`
- Constraint: test is clang-gated and skipped when `clang` is unavailable.

### Failure modes and diagnostics

- Missing `clang`: test is skipped with an explicit message.
- C emitter/regression issues surface as assertion failures in generated C shape checks.
- Compile/link/runtime failures surface as failing build/run assertions.

### Example usage

Representative source used by the test:

```ailang
fn choose(flag: Bool) -> Int {
  if flag { 0 } else { 1 }
}

fn main() -> Int {
  choose(true)
}
```

### Tradeoffs and next steps

- This still validates a narrow program shape; it does not yet cover complex value types, runtime IO effects, or Result-heavy paths.
- It strengthens M6 confidence while keeping tests fast and deterministic.
- Next step: extend runtime-oriented integration fixtures toward M7 HTTP/JSON vertical slices.

## Tests updated

- `compiler/ailang-cli/tests/json_output.rs`
  - added `build_emit_c_bin_handles_calls_and_control_flow_when_clang_available`
