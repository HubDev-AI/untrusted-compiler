# 459 M16 Slice: Runtime-Smoke Workflow Dual-Branch maxBody Coverage

This chapter documents extending runtime-smoke CI to run both metadata-shape branches (`unset` and `--max-body-bytes`).

## 1) What it is

`.github/workflows/runtime-smoke.yml` now executes four runtime-smoke steps:

- smoke + checker for `build/runtime-smoke/default`
- smoke + checker for `build/runtime-smoke/max-body` with `--max-body-bytes 2048`

The upload step keeps publishing the combined `build/runtime-smoke` tree.

## 2) Why it exists

After enforcing shape-aware `runFlags`, CI still exercised only the default branch. This slice keeps both branches live in CI so drift in optional max-body wiring is caught immediately.

## 3) How it works internally

1. Workflow runs default smoke into `build/runtime-smoke/default`.
2. Checker validates default artifacts.
3. Workflow runs max-body smoke into `build/runtime-smoke/max-body`.
4. Checker validates max-body artifacts.
5. Contract, guard, and closure scripts pin both command tokens.

## 4) Inputs, outputs, constraints

Inputs:

- runtime-smoke workflow definition
- workflow contract/guard scripts
- closure audit contract scripts

Outputs:

- deterministic CI coverage of both maxBody metadata branches
- stricter workflow contract enforcement in closure gates

Constraints:

- command tokens and artifact paths are contract-locked.
- both branches must pass checker validation.

## 5) Failure modes and diagnostics

- missing default/max-body workflow token:
  - `missing runtime-smoke workflow token: <expected command>`
- closure fixture drift:
  - `check-milestone-closure` test fails due missing workflow command token.

## 6) Example usage

```bash
scripts/test-runtime-smoke-workflow-contract.sh
scripts/test-runtime-smoke-workflow-contract-guard.sh
scripts/test-check-milestone-closure.sh
```

## 7) Trade-offs and next steps

Trade-offs:

- runtime-smoke CI duration increases due second smoke/check branch.

Next steps:

- make max-body branch value configurable via workflow input while keeping a deterministic default.

## Verification

- `scripts/test-runtime-smoke-workflow-contract.sh`
- `scripts/test-runtime-smoke-workflow-contract-guard.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
