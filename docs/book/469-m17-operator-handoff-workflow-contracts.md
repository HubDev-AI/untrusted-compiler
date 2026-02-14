# 469 M17 Operator Handoff Workflow Contracts

This chapter captures the dedicated CI workflow and contract tests for M17 operator handoff automation.

## 1) What it is

New workflow:

- `.github/workflows/operator-handoff-smoke.yml`

New contract tests:

- `scripts/test-operator-handoff-workflow-contract.sh`
- `scripts/test-operator-handoff-workflow-contract-guard.sh`

## 2) Why it exists

Local scripts are not enough for closure confidence. M17 needs a dedicated CI workflow contract that proves the handoff flow executes consistently on pull requests and on main.

## 3) Workflow contract

Required workflow behaviors:

- triggers on `pull_request` and `push` to `main`,
- runs:
  - `scripts/run-m17-operator-handoff-ci-smoke.sh --artifacts-root build/operator-handoff-smoke`
- uploads artifacts with:
  - `name: operator-handoff-smoke-artifacts`
  - `path: build/operator-handoff-smoke`

## 4) Guard coverage

The guard test fails when the CI smoke wrapper command is removed from the workflow, ensuring deterministic drift detection.

## 5) Verification

- `scripts/test-operator-handoff-workflow-contract.sh`
- `scripts/test-operator-handoff-workflow-contract-guard.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
