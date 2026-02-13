# 328 Milestone Closure Audit Checklist

This chapter defines the strict closure audit for milestone completion claims.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`

This adds an evidence-based closure checker for the highest-risk late milestones (M9/M10/M13).

## Why it exists

Implementation progress and milestone closure are not the same. This checklist prevents us from claiming completion when required live evidence is still missing.

## Closure checks (current scope)

`check-milestone-closure.sh` verifies:

1. M9 release hardening foundations
- release gate script exists,
- release gate workflow exists,
- promotion verifier + publish manifest verifier chain exists.
- release gate script includes strict closure enforcement (`check-milestone-closure.sh --fail-on-pending`).
- release-contract-smoke workflow keeps release verifier/publish contract tests:
  - `scripts/test-alpha-release-workflow-contract.sh`
  - `scripts/test-verify-release-promotion-inputs.sh`
  - `scripts/test-generate-release-publish-manifest.sh`
  - `scripts/test-verify-release-publish-manifest.sh`
- naming-lock CI keeps the release-contract-smoke workflow contract guard:
  - `scripts/test-release-contract-smoke-workflow-contract.sh`
  - `scripts/test-release-contract-smoke-workflow-contract-guard.sh`
- alpha-release workflow keeps release/promotion/publish/upload contract:
  - `scripts/release-alpha-gate.sh`
  - `scripts/verify-release-promotion-inputs.sh`
  - `scripts/generate-release-publish-manifest.sh`
  - `scripts/verify-release-publish-manifest.sh`
  - artifact upload (`alpha-release-gate-artifacts`, `build/release-alpha-gate`)

2. M10 live comparison evidence
- compare matrix includes implementation IDs per endpoint:
  - `sec4`
  - `go`
  - `node`
  - `rust`
- compare matrix contract is aligned:
  - non-empty endpoint and compared rows,
  - leader endpoint matches entry endpoint,
  - compared rows keep endpoint alignment,
  - leader row is present in compared rows.
- cross-impl evidence workflow contract is aligned:
  - scoped run includes `--impls sec4,node,go,rust`,
  - scoped run includes `--endpoints ping,decode`,
  - strict quality gate is present (`--fail-on-warning`),
  - artifact upload contract is present (`benchmark-cross-impl-evidence`, `benchmark-suite/results`).

3. M13 live trend evidence
- trend-note chapter contains at least one:
  - `## Trend Entry (YYYY-MM-DD)`
- scheduled trend workflow keeps hard guards:
  - strict quality check (`check-benchmark-evidence-quality.sh --fail-on-warning`)
  - regression threshold checks (`check_regression_thresholds.sh`)
- scheduled trend workflow uploads artifacts for trend-note ingestion:
  - uses `actions/upload-artifact@v4`
  - artifact name follows `benchmark-trend-*`
  - artifact path includes `benchmark-suite/results`
- benchmark-smoke workflow keeps closure gate contract:
  - runs `scripts/test-benchmark-smoke-closure-gate.sh`
  - runs `scripts/test-check-milestone-closure.sh`
  - runs `scripts/check-milestone-closure.sh --fail-on-pending`

## Example usage

```bash
scripts/check-milestone-closure.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/test-check-milestone-closure.sh
```

## Current result (2026-02-13)

- M9 gate checks: PASS
- M10 live cross-impl evidence and row-contract alignment: PASS
- M13 live trend-note evidence, trend-workflow guards, and artifact-upload contract: PASS

## Inputs, outputs, and constraints

- Inputs:
  - repository files and optional matrix/trend-note paths.
- Output:
  - structured PASS/PENDING table and overall status.
  - evidence paths are rendered repository-relative when possible.
- Constraints:
  - this checker validates closure evidence shape/guardrails; it does not replace full benchmark analysis.

## Tradeoffs and next steps

- Tradeoff:
  - strict checks intentionally focus on late-stage closure evidence and do not re-audit all early milestones.
- Next:
  - keep closure evidence refreshed via `scripts/refresh-closure-evidence-from-ci.sh` after new benchmark or trend runs.
