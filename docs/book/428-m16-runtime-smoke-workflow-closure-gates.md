# 428 M16 Slice: Runtime-Smoke Workflow Closure Gates

This chapter documents CI workflow closure hardening for the operator smoke path.

## What it is

A new runtime smoke workflow and two strict closure gates:

- Workflow: `.github/workflows/runtime-smoke.yml`
- `M16-C`: workflow contract gate
- `M16-D`: naming-lock CI enforcement gate for workflow contract + guard tests

## Why it exists

The operator smoke script is useful locally, but production confidence requires CI execution and CI contract drift protection. This slice ensures both are enforced.

## Implementation details

1. Added `.github/workflows/runtime-smoke.yml`:
   - triggers on `pull_request` and `push` to `main`,
   - runs checkout + `scripts/smoke-sec4-run-hello-api.sh`.
2. Added workflow contract scripts:
   - `scripts/test-runtime-smoke-workflow-contract.sh`
   - `scripts/test-runtime-smoke-workflow-contract-guard.sh`
3. Wired contract scripts into naming-lock workflow.
4. Expanded strict closure audit with `M16-C` and `M16-D`.
5. Updated closure fixture tests and roadmap closure table to keep gate alignment deterministic.

## Validation

Executed locally:

- `scripts/test-runtime-smoke-workflow-contract.sh`
- `scripts/test-runtime-smoke-workflow-contract-guard.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --format json --fail-on-pending`

Result: pass.

## Tradeoffs and next steps

- Contract tests validate workflow structure and required smoke-step wiring; they do not replace runtime execution itself (which is handled by `runtime-smoke.yml`).
- Future expansion can add artifact upload (logs/report) for post-failure diagnostics in runtime smoke runs.
