# 462 M16 Slice: Runtime-Smoke Single-Pass Bundle Checker Contract

This chapter documents consolidating runtime-smoke validation into one deterministic bundle checker command.

## 1) What it is

`scripts/check-runtime-smoke-bundle.sh` now provides a single pass that:

- validates both runtime-smoke branches (`default`, `max-body`)
- runs per-branch artifact checks
- builds and validates the aggregated branch index contract

## 2) Why it exists

Before this slice, workflow validation was split across separate checker and index-builder steps. A single command reduces workflow drift and makes operator validation simpler.

## 3) How it works internally

1. Require `default` and `max-body` branch directories.
2. Run `scripts/check-runtime-smoke-artifacts.sh` for each branch.
3. Run `scripts/build-runtime-smoke-branch-index.sh` for aggregate index output.
4. Validate index contract shape (`version`, `branchOrder`, branch count).

## 4) Inputs, outputs, constraints

Inputs:

- `build/runtime-smoke/default/*`
- `build/runtime-smoke/max-body/*`

Outputs:

- `build/runtime-smoke/runtime-smoke-branch-index.json`
- deterministic pass/fail bundle-check result

Constraints:

- fixed branch set (`default`, `max-body`)
- index contract shape is strict and deterministic

## 5) Failure modes and diagnostics

- missing branch directory:
  - `runtime-smoke bundle missing branch directory: <branch>`
- branch checker failure:
  - underlying deterministic diagnostics from `check-runtime-smoke-artifacts.sh`
- index shape mismatch:
  - `runtime-smoke branch index does not match expected bundle contract`

## 6) Example usage

```bash
scripts/check-runtime-smoke-bundle.sh \
  --artifacts-root build/runtime-smoke \
  --index-path build/runtime-smoke/runtime-smoke-branch-index.json
```

## 7) Trade-offs and next steps

Trade-offs:

- bundle checker still depends on lower-level branch checker/index scripts, so those contracts remain part of maintenance surface.

Next steps:

- expose bundle-check summary as a short JSON report for CI annotation tooling.

## Verification

- `scripts/test-check-runtime-smoke-bundle.sh`
- `scripts/test-runtime-smoke-workflow-contract.sh`
- `scripts/test-runtime-smoke-workflow-contract-guard.sh`
- `scripts/test-check-milestone-closure.sh`
