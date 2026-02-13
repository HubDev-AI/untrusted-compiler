# 233 M10 Slice: Benchmark Preflight and Early-Fail Checks

This chapter documents benchmark preflight checks that fail fast on missing tooling and reduce “looks stuck” run behavior.

## What it is

Updated:
- `benchmark-suite/scripts/preflight.sh`
- `benchmark-suite/scripts/run_comparison_matrix.sh`
- `benchmark-suite/scripts/test_preflight.sh`
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- added a dedicated preflight script for benchmark dependencies (`curl`, `jq`, implementation toolchains, and `wrk2` for real runs),
- integrated preflight into orchestrator startup (`run_comparison_matrix.sh`) before any service/process work,
- added support for `--impls=...` argument style in preflight/orchestrator,
- added explicit readiness progress line in orchestrator to improve run visibility,
- exposed `make` targets: `preflight` and `preflight-dry`.

## Why it exists

When tool dependencies were missing (most often `wrk2`), runs could appear stalled while startup/readiness checks were still in progress. This slice makes dependency failures explicit at the beginning of the run and gives immediate operator feedback.

## How it works internally

1. `preflight.sh` parses implementation list and checks required commands.
2. In dry-run-only mode, it skips `wrk2` requirement.
3. `run_comparison_matrix.sh` runs preflight first:
   - dry-run: `preflight --dry-run-only`
   - real run: full preflight (includes `wrk2`)
4. If preflight fails, orchestrator exits before starting benchmark services.
5. Orchestrator prints a readiness wait line per implementation (`waiting for readiness ...`) before profile execution.

## Inputs, outputs, and constraints

- Inputs:
  - `--impls` / `--impls=...`
  - `--dry-run`
- Outputs:
  - deterministic preflight pass/fail lines,
  - early exit on missing tools.
- Constraints:
  - real benchmark matrix still requires local installation of all selected implementation toolchains plus `wrk2`.

## Failure modes and diagnostics

- unsupported implementation in preflight/orchestrator list:
  - exits with usage error.
- missing required tool:
  - prints `MISSING ...` line and exits with `preflight failed: missing required tooling`.
- readiness delay:
  - now visible via explicit waiting log line before readiness polling.

## Example usage

Dry-run preflight:

```bash
make -C benchmark-suite preflight-dry IMPLS=sec4,node,go,rust,c
```

Real-run preflight:

```bash
make -C benchmark-suite preflight IMPLS=sec4,node,go,rust
```

Matrix dry run (includes orchestrator-integrated preflight):

```bash
make -C benchmark-suite bench-matrix-dry IMPLS=sec4,node
```

## Tradeoffs and next steps

- Tradeoff:
  - dry-run now checks local toolchains for selected implementations, which may be stricter than command-plan-only workflows.
- Next:
  - optionally add per-platform install hints for missing tools (`wrk2`, `go`, `node`, `cargo`),
  - include preflight summary in generated benchmark markdown report metadata.
