# 248 M10 Slice: Artifact Manifest and Provenance

This chapter documents deterministic artifact-manifest generation for benchmark results.

## What it is

Updated:
- `benchmark-suite/scripts/build_artifact_manifest.sh`
- `benchmark-suite/scripts/test_build_artifact_manifest.sh`
- `benchmark-suite/scripts/run_full_benchmark_suite.sh`
- `benchmark-suite/scripts/test_run_full_benchmark_suite.sh`
- `benchmark-suite/Makefile`
- `benchmark-suite/README.md`

Key changes:
- added `build_artifact_manifest.sh` to produce:
  - artifact relative path,
  - size in bytes,
  - sha256 checksum.
- full-suite runner now emits `results/artifact-manifest.json`,
- added `artifact-manifest` Make target.

## Why it exists

M10 results are now multi-artifact and cross-script. A manifest gives deterministic provenance and makes CI/archival verification straightforward.

## How it works internally

1. Scan `results/` files (excluding logs and `.gitkeep`).
2. Compute `sizeBytes` and `sha256` for each artifact.
3. Emit one JSON manifest with generation timestamp and artifact list.
4. Full-suite orchestrator calls manifest generator after combined report publish.

## Inputs, outputs, and constraints

- Inputs:
  - `build_artifact_manifest.sh <results_dir> <out_manifest.json>`.
- Outputs:
  - manifest JSON (`version: 0.1`) with deterministic file metadata.
- Constraints:
  - requires `shasum` availability.

## Failure modes and diagnostics

- missing results directory -> explicit error.
- missing `shasum` binary -> explicit dependency error.

## Example usage

```bash
make -C benchmark-suite artifact-manifest
```

## Tradeoffs and next steps

- Tradeoff:
  - manifest reflects filesystem state at generation time; stale artifacts are included unless cleaned beforehand.
- Next:
  - optionally include expected/required artifact set and mark extras as warnings for stricter CI gating.
