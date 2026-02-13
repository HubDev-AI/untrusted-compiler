# 247 M10 Slice: Full-Suite Orchestrator

This chapter documents a top-level benchmark runner that executes fixed-target and step-load suites together, then publishes one combined report.

## What it is

Updated:
- `benchmark-suite/scripts/run_full_benchmark_suite.sh`
- `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- added `run_full_benchmark_suite.sh` to orchestrate:
  - fixed-target matrix run (`run_comparison_matrix.sh`),
  - step-load matrix run (`run_step_matrix.sh`),
  - final report publish with step matrix included.
- added Make targets:
  - `bench-full`
  - `bench-full-dry`.

## Why it exists

M10 now has two complementary execution tracks (fixed and step). Running them manually in correct sequence is repetitive and error-prone. This runner provides one deterministic command path.

## How it works internally

1. Parse scope args (`impls`, `endpoints`, optional sec-audit path).
2. Execute fixed-target orchestrator phase.
3. Execute step-load orchestrator phase.
4. Re-run markdown publish using:
   - compare matrix
   - matrix analysis
   - step matrix
   - optional sec-audit
5. Emit final report path.

## Inputs, outputs, and constraints

- Inputs:
  - same scope controls as matrix orchestrators.
- Outputs:
  - `results/summaries/compare-matrix.json`
  - `results/summaries/analysis.json`
  - `results/summaries/step-matrix.json`
  - `results/benchmark-report.md`
- Constraints:
  - full run inherits all runtime/tooling requirements from both underlying orchestrators.

## Failure modes and diagnostics

- delegated validation/preflight failures from underlying scripts propagate with phase context.
- missing generated artifacts during final publish causes explicit report-generation failure.

## Example usage

```bash
make -C benchmark-suite bench-full IMPLS=sec4,node,go,rust ENDPOINTS=decode,users-post
```

Dry run:

```bash
make -C benchmark-suite bench-full-dry IMPLS=node,go ENDPOINTS=ping
```

## Tradeoffs and next steps

- Tradeoff:
  - longest-running benchmark command path in suite.
- Next:
  - optionally add a single final JSON manifest that records all generated artifact paths and hashes.
