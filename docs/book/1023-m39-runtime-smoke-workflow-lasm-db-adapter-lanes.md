# 1023 M39 Slice: Runtime-Smoke Workflow LASM DB Adapter Lanes

## What It Is

This slice extends runtime-smoke CI workflow coverage to execute both LASM DB adapter smoke lanes:

- `records-log` lane
- `sqlite` lane

It updates:

- `.github/workflows/runtime-smoke.yml`
- `scripts/test-runtime-smoke-workflow-contract.sh`
- `scripts/test-runtime-smoke-workflow-contract-guard.sh`

## Why It Exists

The LASM DB adapter smoke script and naming-lock coverage were already in place, but `runtime-smoke` workflow still only executed hello-api smoke paths.

Adding explicit adapter lanes to runtime-smoke makes adapter coverage part of routine workflow execution, not only naming-lock validation.

## How It Works Internally

1. Runtime-smoke workflow:
   - keeps existing hello-api default/max-body smoke steps,
   - adds:
     - `scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter records-log --artifacts-dir build/runtime-smoke/lasm-db-records-log`
     - `scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter sqlite --artifacts-dir build/runtime-smoke/lasm-db-sqlite`
   - keeps existing runtime-smoke bundle validation and artifact upload.

2. Contract test updates:
   - workflow contract now requires both DB adapter smoke step tokens.

3. Guard test updates:
   - pass fixture includes both new DB adapter smoke steps,
   - negative fixture verifies deterministic failure when sqlite DB-adapter step is missing,
   - existing bundle-check missing-step guard remains enforced.

## Inputs / Outputs and Constraints

Inputs:
- `runtime-smoke` workflow trigger (`push` to `main`).

Outputs:
- workflow execution now includes hello-api lanes plus LASM DB adapter lanes,
- artifacts continue to publish under `build/runtime-smoke`.

Constraints:
- runtime-smoke bundle checker still validates default/max-body branch index contract only,
- DB adapter smoke artifacts are additional workflow evidence and do not alter branch-index schema in this slice.

## Failure Modes and Diagnostics

- workflow drift on DB adapter smoke steps fails contract with deterministic missing-token diagnostics,
- guard test fails if either sqlite adapter token or bundle-check token is absent in fixture workflows.

## Example Usage

Contract checks:

```bash
scripts/test-runtime-smoke-workflow-contract.sh
scripts/test-runtime-smoke-workflow-contract-guard.sh
```

## Tradeoffs and Next Steps

Tradeoffs:
- runtime-smoke workflow runtime increases due two extra smoke executions.

Next steps:
1. extend runtime-smoke bundle/index scripts if DB adapter lanes should become first-class indexed branches,
2. consider separate adapter-specific summary artifacts for faster CI triage.
