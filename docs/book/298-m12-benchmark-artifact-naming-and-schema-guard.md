# 298 M12 Slice: Benchmark Artifact Naming and Schema Guard

This chapter documents extending naming-lock enforcement to benchmark artifact naming and schema contracts.

## What it is

Updated:
- `scripts/check-naming-lock.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

M12 required expanding naming lock from language/tooling labels into benchmark artifacts. Without this, benchmark scripts/testdata could drift on artifact names or JSON shape and silently break report tooling.

## How it works internally

`scripts/check-naming-lock.sh` now includes `check_benchmark_artifact_contract`:

1. Verifies benchmark scripts still include canonical artifact filename patterns:
   - `${impl}-report.json`
   - `${impl}-${endpoint}.json`
   - `compare-matrix.json`
   - `analysis.json`
   - `step-matrix.json`
   - `artifact-manifest.json`
   - `benchmark-report.md`
   - `${impl}-service.log`
2. Verifies required benchmark report testdata files exist:
   - `sample-sec4-report.json`
   - `sample-go-report.json`
   - `sample-node-report.json`
   - `sample-rust-report.json`
3. Validates report testdata schema keys and `impl` identity matching filename.
4. Validates summary/step-summary/step-matrix sample schema keys.

## Inputs, outputs, and constraints

- Inputs:
  - `benchmark-suite/scripts/*.sh`
  - `benchmark-suite/scripts/testdata/*.json`
- Output:
  - deterministic pass/fail for benchmark naming/schema contract.
- Constraint:
  - contract is intentionally strict for current M10/M12 output model; future schema changes must update this guard in the same slice.

## Failure modes and diagnostics

- Missing artifact pattern in scripts:
  - emits explicit missing-pattern error.
- Missing sample file:
  - emits missing-path error.
- Schema/key drift:
  - emits sample-schema mismatch error for the exact file.

## Example usage

```bash
scripts/check-naming-lock.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - schema checks are keyed to current benchmark testdata fixtures; changes are fail-fast and require intentional updates.
- Next:
  - if benchmark artifact schema evolves, update testdata + guard checks in the same commit to preserve deterministic review.
