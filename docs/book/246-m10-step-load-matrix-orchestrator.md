# 246 M10 Slice: Step-Load Matrix Orchestrator

This chapter documents one-command orchestration for step-load benchmarking across implementations and endpoints.

## What it is

Updated:
- `benchmark-suite/scripts/run_step_matrix.sh`
- `benchmark-suite/scripts/test_run_step_matrix.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- added cross-implementation step orchestrator with:
  - `--impls` / `--impls=...`
  - `--endpoints` / `--endpoints=...`
  - `--dry-run`
- orchestrator flow:
  - preflight checks,
  - per-impl service startup/readiness,
  - per-endpoint `run_step_profile`,
  - per-endpoint `analyze_step_profile`,
  - final `compare_step_matrix` generation.
- added Make targets:
  - `bench-step-matrix`
  - `bench-step-matrix-dry`.

## Why it exists

Step-load tooling was available but required manual command choreography. This orchestrator provides reproducible, scoped, and testable end-to-end step-load execution.

## How it works internally

1. Validate implementation and endpoint lists.
2. Run preflight in dry/full mode.
3. For each implementation:
   - start service on orchestrator port,
   - wait for `/ping == ok`,
   - run step profiles for selected endpoints,
   - analyze each produced step summary.
4. Build one scoped step matrix output.

## Inputs, outputs, and constraints

- Inputs:
  - impl/endpoint scope args,
  - optional `BENCH_PORT`,
  - optional `STEP_KNEE_THRESHOLD`.
- Outputs:
  - per-impl endpoint step summaries and analyses,
  - `results/summaries/step-matrix.json`.
- Constraints:
  - real runs require `wrk2` and selected toolchains.

## Failure modes and diagnostics

- unsupported impl/endpoint -> explicit validation error.
- readiness failure -> service log tail printed.
- missing step outputs -> downstream analyzer/matrix errors.

## Example usage

```bash
make -C benchmark-suite bench-step-matrix IMPLS=sec4,node,go,rust ENDPOINTS=decode,users-post
```

Dry run:

```bash
make -C benchmark-suite bench-step-matrix-dry IMPLS=node,go ENDPOINTS=ping
```

## Tradeoffs and next steps

- Tradeoff:
  - step matrix orchestration can be significantly longer than fixed-target matrix runs.
- Next:
  - optionally wire step matrix directly into full report publishing workflow when both matrices are produced in the same session.
