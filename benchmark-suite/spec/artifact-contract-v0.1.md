# Benchmark Artifact Contract v0.1

This document is the canonical schema/naming contract for benchmark output artifacts.

Machine-validated schema assets:
- `benchmark-suite/spec/schemas/report.schema.json`
- `benchmark-suite/spec/schemas/summary.schema.json`
- `benchmark-suite/spec/schemas/step-summary.schema.json`
- `benchmark-suite/spec/schemas/compare-matrix.schema.json`
- `benchmark-suite/spec/schemas/analysis.schema.json`
- `benchmark-suite/spec/schemas/step-matrix.schema.json`
- `benchmark-suite/spec/schemas/artifact-manifest.schema.json`

## Artifact naming

Canonical filenames:

- Per-implementation summary files: `<impl>-<endpoint>.json`
- Per-implementation report bundle: `<impl>-report.json`
- Matrix comparison bundle: `compare-matrix.json`
- Matrix analysis bundle: `analysis.json`
- Step-load matrix bundle: `step-matrix.json`
- Published markdown report: `benchmark-report.md`
- Artifact manifest: `artifact-manifest.json`
- Service logs: `<impl>-service.log`

Canonical implementation IDs:
- `sec4`
- `go`
- `node`
- `rust`
- `c`

## JSON artifact schema keys

All JSON artifacts are versioned with:
- `"version": "0.1"` (where applicable)

### `<impl>-report.json`

Top-level keys:
- `version`
- `impl`
- `env`
- `summaries`
- `secAudit`
- `selectedEndpoints` (nullable)

### `<impl>-<endpoint>.json`

Top-level keys:
- `impl`
- `endpoint`
- `targetRps`
- `requestsPerSec`
- `latency`

### `compare-matrix.json`

Top-level keys:
- `version`
- `endpoints`

Each endpoint entry:
- `endpoint`
- `compared`
- `leader`

### `analysis.json`

Top-level keys:
- `version`
- `endpoints`
- `summary`

### `step-matrix.json`

Top-level keys:
- `version`
- `endpoints`
- `summary`

### `artifact-manifest.json`

Top-level keys:
- `version`
- `generatedAt`
- `artifacts`

Each artifact entry:
- `path`
- `sha256`
- `sizeBytes`

## Change policy

Any artifact key/name changes must update, in the same change:

1. this file (`artifact-contract-v0.1.md`),
2. benchmark script tests under `benchmark-suite/scripts/test_*.sh`,
3. naming-lock validation in `scripts/check-naming-lock.sh`.
