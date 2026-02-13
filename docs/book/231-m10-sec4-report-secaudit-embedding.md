# 231 M10 Slice: Untrusted<T> Report sec4 audit Embedding

This chapter documents sec4 audit embedding into Untrusted<T> benchmark report bundles during matrix orchestration.

## What it is

Updated:
- `benchmark-suite/scripts/run_comparison_matrix.sh`
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/README.md`

During orchestrated runs, Untrusted<T> report bundling now includes sec4 audit JSON when the baseline artifact path is available.

## Why it exists

M10 requires Untrusted<T>-specific security posture evidence in benchmark outputs. Embedding sec4 audit into `sec4-report.json` keeps security and performance artifacts aligned in one bundle.

## How it works internally

1. Orchestrator resolves `sec_audit_path` (default baseline if present).
2. For each implementation:
   - `sec4` uses `build_report.sh ... <sec_audit_path>`.
   - other implementations use standard `build_report.sh` without audit.
3. Dry-run output reflects the same conditional command path for auditability.

## Inputs, outputs, and constraints

- Inputs:
  - optional audit artifact path.
- Outputs:
  - `results/summaries/sec4-report.json` with non-null `secAudit` payload when available.
- Constraints:
  - audit embedding is currently Untrusted<T>-only in orchestrator flow.

## Failure modes and diagnostics

- missing audit path when explicitly provided -> orchestrator failure via `build_report.sh` argument validation.
- malformed audit JSON -> `build_report.sh` parse failure.

## Example usage

```bash
benchmark-suite/scripts/run_comparison_matrix.sh --dry-run --impls sec4
```

## Tradeoffs and next steps

- Tradeoff:
  - non-Untrusted<T> implementations do not yet have equivalent security posture payloads in their report bundles.
- Next:
  - define optional external posture schema hooks for non-Untrusted<T> comparators,
  - include sec4 audit summary snippets in matrix-level analysis artifacts.
