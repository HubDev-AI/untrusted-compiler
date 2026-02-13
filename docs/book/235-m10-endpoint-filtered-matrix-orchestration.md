# 235 M10 Slice: Endpoint-Filtered Matrix Orchestration

This chapter documents endpoint filtering for the M10 matrix orchestrator so focused benchmark runs can be executed without modifying scripts.

## What it is

Updated:
- `benchmark-suite/scripts/run_comparison_matrix.sh`
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- added orchestrator argument `--endpoints` (and `--endpoints=...`) with supported values:
  - `ping`
  - `decode`
  - `users-post`
  - `users-get`
- added endpoint validation with explicit errors for unsupported names,
- switched real and dry-run profile loops to iterate configured endpoint set,
- exposed `ENDPOINTS` variable in Make targets for matrix runs.

## Why it exists

Full matrix runs are expensive and slower to debug. Endpoint filtering allows short focused loops (for example `ping` only) while preserving the same orchestration/reporting pipeline.

## How it works internally

1. Orchestrator parses `--endpoints` into a normalized list.
2. Every endpoint is validated before execution.
3. In dry-run mode, command plan output includes only selected endpoints.
4. In real mode, `run_profile.sh` executes only selected endpoints per implementation.
5. Compare/analyze/publish steps run on whatever summaries were produced.

## Inputs, outputs, and constraints

- Inputs:
  - `--endpoints ping,decode,...` or `--endpoints=...`
  - Make variable `ENDPOINTS=...`
- Outputs:
  - endpoint-specific summary/report artifacts matching selected workload set.
- Constraints:
  - passing unsupported endpoint values fails fast before preflight execution.

## Failure modes and diagnostics

- `unsupported endpoint for orchestrator: <name>` if endpoint is not in allowed set.
- empty endpoint list fails with `no endpoints provided`.

## Example usage

Focused ping-only dry run:

```bash
make -C benchmark-suite bench-matrix-dry IMPLS=sec4,node ENDPOINTS=ping
```

Focused decode/users-post real run:

```bash
make -C benchmark-suite bench-matrix IMPLS=sec4,node,go,rust ENDPOINTS=decode,users-post
```

## Tradeoffs and next steps

- Tradeoff:
  - partial endpoint sets can produce incomplete comparison matrices by design.
- Next:
  - optionally annotate matrix/report metadata with selected endpoint set for easier downstream interpretation.
