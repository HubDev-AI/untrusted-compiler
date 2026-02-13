# 95 M7 Slice: `hello-api` Run Flow Validation

This chapter documents the next M7 slice: pinning `sec4 run` behavior for the new `hello-api` sample.

## Run-Path Coverage for `examples/hello-api`

### What it is

Added a clang-gated integration test:
- `run_command_executes_hello_api_example_when_clang_available`

The test verifies `sec4 run --path examples/hello-api` succeeds and compiles through the C backend pipeline.

### Why it exists

`hello-api` was added as a bootstrap sample in the previous slice. This slice ensures it is exercised through the same user-facing run command used by day-to-day development, not only explicit build invocations.

### How it works internally

- Test invokes:
  - `sec4 run --path <workspace>/examples/hello-api`
- Asserts:
  - command exits successfully
  - stdout contains `compiled binary:` (proves `c-bin` pipeline execution)

### Inputs, outputs, and constraints

- Input: existing `examples/hello-api` project.
- Output: compiled binary under `examples/hello-api/build/` and successful run command exit.
- Constraint: clang must be available; test is skipped otherwise.

### Failure modes and diagnostics

- Pipeline regressions in `run` or `c-bin` now fail this integration test.
- Missing effects/intrinsic lowering regressions in `hello-api` source would fail compile phase before execution.

### Example usage

```bash
cargo run -p sec4 -- run --path examples/hello-api
```

### Tradeoffs and next steps

- Current runtime behavior is still stubbed; success means pipeline correctness, not real HTTP serving.
- This test gives a stable guardrail while we replace stubs with real M7 runtime behavior.

## Tests updated

- `compiler/sec4-cli/tests/json_output.rs`
  - added `run_command_executes_hello_api_example_when_clang_available`
