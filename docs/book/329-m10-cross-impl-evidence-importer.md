# M10 Slice: Cross-Impl Evidence Importer (CI -> compare-matrix)

This slice adds a deterministic path to import live cross-implementation benchmark evidence into the canonical matrix used by strict closure checks.

## Why this exists

M10 closure gate `M10-A` requires committed live matrix evidence that includes `sec4`, `go`, `node`, and `rust`.

Before this slice, the benchmark harness could generate matrix outputs, but there was no single command that:

1. pulled the latest successful CI artifact, and
2. validated required implementation coverage before writing `compare-matrix.json`.

## What was implemented

1. Cross-impl evidence workflow:
- `.github/workflows/benchmark-cross-impl-evidence.yml`
- Manual dispatch workflow that runs:
  - implementations: `sec4,node,go,rust`
  - endpoints: `ping,decode`
- Enforces strict evidence quality before artifact upload:
  - `scripts/check-benchmark-evidence-quality.sh --fail-on-warning`
- Uploads artifact: `benchmark-cross-impl-evidence`

2. Evidence importer command:
- `benchmark-suite/scripts/update_cross_impl_matrix_from_ci.sh`
- Supports two modes:
  - artifact-fetch mode (default): uses `gh run list` + `gh run download`
  - direct mode: `--matrix <path>` for local/offline import
- Enforces matrix coverage for required impl IDs:
  - `sec4`, `go`, `node`, `rust` in each endpoint group
- Enforces benchmark evidence quality before import:
  - strict by default: `check-benchmark-evidence-quality.sh --fail-on-warning`
  - optional local override: `--quality-allow-warning`
- Writes canonical target:
  - `benchmark-suite/results/summaries/compare-matrix.json`

3. Command-level contract tests:
- `benchmark-suite/scripts/test_update_cross_impl_matrix_from_ci.sh`
- Validates:
  - dry-run command composition
  - strict-vs-allow quality mode command composition
  - successful import from a valid matrix fixture
  - rejection of matrix inputs that do not include required per-endpoint impl coverage
  - strict-quality rejection for non-constant-rate leader matrices, with explicit opt-out behavior

4. Smoke CI integration:
- `.github/workflows/benchmark-smoke.yml` now runs:
  - `benchmark-suite/scripts/test_update_cross_impl_matrix_from_ci.sh`

## Operator usage

Run live workflow first (GitHub Actions manual dispatch):

```bash
Workflow: benchmark-cross-impl-evidence.yml
```

Then import latest successful artifact:

```bash
benchmark-suite/scripts/update_cross_impl_matrix_from_ci.sh
```

Optional local import (no fetch):

```bash
benchmark-suite/scripts/update_cross_impl_matrix_from_ci.sh \
  --matrix benchmark-suite/scripts/testdata/sample-cross-impl-compare-matrix.json \
  --target benchmark-suite/results/summaries/compare-matrix.json
```

## Closure relationship

This slice enables, but does not by itself satisfy, `M10-A`.
`M10-A` becomes `PASS` only after a live artifact-backed matrix is imported and committed.
