# 228 M10 Slice: AILang Baseline Service

This chapter documents a runnable AILang benchmark comparator service for M10.

## What it is

Added:
- `benchmark-suite/services/ailang/ailang.toml`
- `benchmark-suite/services/ailang/src/main.ai`
- `benchmark-suite/services/ailang/runtime/benchmark_runtime.h`
- `benchmark-suite/services/ailang/runtime/benchmark_runtime.c`
- `benchmark-suite/services/ailang/build.sh`
- `benchmark-suite/services/ailang/smoke.sh`
- `benchmark-suite/services/ailang/README.md`

Updated:
- `benchmark-suite/scripts/run_comparison_matrix.sh` (supports `ailang` in default impl set)
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/Makefile` help text
- `benchmark-suite/README.md`

The AILang service now runs the benchmark endpoint contract through generated C linked with a benchmark-specific runtime adapter.

## Why it exists

M10 comparison requirements include AILang itself, not only external-language baselines. This slice provides a runnable AILang service path suitable for benchmark harness parity and cross-language reporting.

## How it works internally

1. AILang source (`src/main.ai`) defines route handlers for `ping`, `decode`, `users` write/read endpoints.
2. `build.sh` runs:
   - `ailang build --emit c` (captured to `build/generated.c`),
   - links generated C with custom runtime adapter (`benchmark_runtime.c`) into `ailang-bench-server`.
3. Runtime adapter implements the required ABI intrinsics used by generated code:
   - route registration/serve,
   - request decoding/path param extraction,
   - response builders (`res.text`, `res.ok`, `res.json`),
   - deterministic error envelopes and trace ids.
4. Adapter keeps in-memory user storage to satisfy `POST /users` and `GET /users/:id` contract behavior.

## Inputs, outputs, and constraints

- Inputs:
  - benchmark HTTP requests for shared endpoints.
- Outputs:
  - responses matching benchmark contract shapes.
- Constraints:
  - benchmark-specific runtime adapter is contract-oriented,
  - in-memory storage only,
  - generated C capture filters CLI status text before compilation.

## Failure modes and diagnostics

- build command emits unsupported flags or non-C prelude -> build script fails (covered by smoke).
- invalid payload -> `VALIDATION.INVALID` envelope.
- invalid/missing user id path -> `VALIDATION.UUID_INVALID` envelope.
- unknown route/user -> `HTTP.NOT_FOUND` envelope.

## Example usage

```bash
benchmark-suite/services/ailang/smoke.sh
```

Manual build + run:

```bash
benchmark-suite/services/ailang/build.sh
PORT=8080 benchmark-suite/services/ailang/ailang-bench-server
```

Orchestrator dry-run with defaults:

```bash
benchmark-suite/scripts/run_comparison_matrix.sh --dry-run
```

## Tradeoffs and next steps

- Tradeoff:
  - runtime adapter currently focuses on benchmark endpoint parity rather than full stdlib/runtime completeness.
- Next:
  - migrate benchmark adapter logic into production runtime as core HTTP behavior hardens,
  - add DB-backed variant for persistent-path benchmark parity.
