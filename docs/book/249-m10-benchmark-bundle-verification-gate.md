# 249 M10 Slice: Benchmark Bundle Verification Gate

This chapter documents a deterministic verifier for benchmark output completeness and structural validity.

## What it is

Updated:
- `benchmark-suite/scripts/verify_benchmark_bundle.sh`
- `benchmark-suite/scripts/test_verify_benchmark_bundle.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- added `verify_benchmark_bundle.sh` to validate:
  - required core artifacts exist (`compare-matrix`, `analysis`, `step-matrix`, report, manifest),
  - selected impl/endpoint summary/report/step artifacts exist,
  - JSON files parse successfully,
  - baseline report title is present.
- added Make target:
  - `verify-bundle`.

## Why it exists

As M10 outputs grew, it became easy to miss one artifact and still think a run was complete. This verifier provides a single reproducibility/completeness gate for CI and manual validation.

## How it works internally

1. Validate `results/` + `results/summaries/` structure.
2. Check required top-level artifacts.
3. For each selected impl and endpoint, require all corresponding fixed and step artifacts.
4. Validate JSON parseability with `jq`.
5. Enforce basic sanity checks on matrix/analysis summaries.

## Inputs, outputs, and constraints

- Inputs:
  - `verify_benchmark_bundle.sh <results_dir> [impls_csv] [endpoints_csv]`.
- Outputs:
  - success message on complete bundle,
  - non-zero exit with explicit missing/invalid artifact error otherwise.
- Constraints:
  - default impl/endpoint scope assumes full core set unless overridden.

## Failure modes and diagnostics

- missing artifact path -> explicit failure with file path.
- invalid JSON artifact -> explicit parse failure.
- empty endpoint summaries in matrix/analysis -> explicit structural failure.

## Example usage

```bash
make -C benchmark-suite verify-bundle IMPLS=ailang,node,go,rust ENDPOINTS=ping,decode,users-post,users-get
```

## Tradeoffs and next steps

- Tradeoff:
  - strict scope checking requires IMPLS/ENDPOINTS values aligned with the run.
- Next:
  - optionally verify artifact hashes against `artifact-manifest.json` inside the same command.
