# 231 M10 Slice: AILang Report sec.audit Embedding

This chapter documents sec.audit embedding into AILang benchmark report bundles during matrix orchestration.

## What it is

Updated:
- `benchmark-suite/scripts/run_comparison_matrix.sh`
- `benchmark-suite/scripts/test_run_comparison_matrix.sh`
- `benchmark-suite/README.md`

During orchestrated runs, AILang report bundling now includes sec.audit JSON when the baseline artifact path is available.

## Why it exists

M10 requires AILang-specific security posture evidence in benchmark outputs. Embedding sec.audit into `ailang-report.json` keeps security and performance artifacts aligned in one bundle.

## How it works internally

1. Orchestrator resolves `sec_audit_path` (default baseline if present).
2. For each implementation:
   - `ailang` uses `build_report.sh ... <sec_audit_path>`.
   - other implementations use standard `build_report.sh` without sec audit.
3. Dry-run output reflects the same conditional command path for auditability.

## Inputs, outputs, and constraints

- Inputs:
  - optional sec audit artifact path.
- Outputs:
  - `results/summaries/ailang-report.json` with non-null `secAudit` payload when available.
- Constraints:
  - sec audit embedding is currently AILang-only in orchestrator flow.

## Failure modes and diagnostics

- missing sec audit path when explicitly provided -> orchestrator failure via `build_report.sh` argument validation.
- malformed sec audit JSON -> `build_report.sh` parse failure.

## Example usage

```bash
benchmark-suite/scripts/run_comparison_matrix.sh --dry-run --impls ailang
```

## Tradeoffs and next steps

- Tradeoff:
  - non-AILang implementations do not yet have equivalent security posture payloads in their report bundles.
- Next:
  - define optional external posture schema hooks for non-AILang comparators,
  - include sec.audit summary snippets in matrix-level analysis artifacts.
