# 86 M6 Slice: `time.now` End-to-End `c-bin` Validation

This chapter documents the next M6 vertical slice: end-to-end validation that `time.now` flows through semantic checks, MIR/C lowering, runtime intrinsic rewriting, and native compilation.

## `time.now` Pipeline Coverage

### What it is

Added a clang-gated CLI integration test that builds and runs a temporary project containing:
- required `effects { time.now }` declaration
- `time.now()` call in `main`

### Why it exists

We already added intrinsic rewriting for `time.now` in the C emitter, but this slice proves the full user path via `ailang build --emit c-bin`, not only unit-level emitter tests.

### How it works internally

- Test writes a temporary project with:
  - `ailang.toml`
  - `src/main.ai` using `time.now`
- Runs:
  - `ailang build --emit c-bin --path <temp-project>`
- Asserts:
  - build succeeds
  - generated C contains `ailang_rt_time_now()`
  - compiled binary exists and executes successfully

### Inputs, outputs, and constraints

- Input: tiny intrinsic-using AILang project.
- Output:
  - generated C and runtime C compilation artifacts
  - executable binary in temp `build/`
- Constraint: clang must be available; test is skipped otherwise.

### Failure modes and diagnostics

- Missing effect declaration would fail earlier in semantic analysis (`E4002` effect usage error).
- Intrinsic rewrite regression would fail generated-C assertions.
- Runtime declaration/link regressions would fail `c-bin` build or binary run assertions.

### Example usage

```ailang
fn main() effects { time.now } -> Int64 {
  time.now()
}
```

### Tradeoffs and next steps

- Current runtime `time.now` is deterministic stub behavior (`0`) in M6.
- This slice focuses on pipeline integrity, not real clock semantics.
- Next step: implement policy-aware/runtime-backed clock behavior in later runtime milestones.

## Tests updated

- `compiler/ailang-cli/tests/json_output.rs`
  - added `build_emit_c_bin_handles_time_now_intrinsic_when_clang_available`
