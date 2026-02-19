# 1022 M39 Slice: Naming-Lock Coverage for LASM DB Adapter Smoke Contracts

## What It Is

This slice wires the new LASM DB adapter smoke script contracts into the naming-lock suite by updating:

- `scripts/run-naming-lock-contract-suite.sh`

New enforced steps:

- `scripts/test-smoke-sec4-run-lasm-db-adapter-script-contract.sh`
- `scripts/test-smoke-sec4-run-lasm-db-adapter-script-contract-guard.sh`

## Why It Exists

The LASM DB adapter operator smoke script and its guard tests were already implemented, but naming-lock orchestration did not yet execute them.

Without suite integration, CI could pass while those script contracts drifted.

## How It Works Internally

1. Naming-lock suite step list now includes the two LASM DB adapter smoke contract checks directly after existing hello-api smoke script contract checks.
2. Because naming-lock workflow executes `scripts/run-naming-lock-contract-suite.sh`, no workflow structure change is required.
3. Drift in the smoke script now fails naming-lock through the same deterministic contract/guard mechanism as other script contracts.

## Inputs / Outputs and Constraints

Inputs:
- existing naming-lock suite invocation.

Outputs:
- naming-lock pass/fail now includes LASM DB adapter smoke contract status.

Constraints:
- integration is list-based; contract behavior remains owned by the underlying test scripts.

## Failure Modes and Diagnostics

- if LASM DB smoke script contract fails, naming-lock exits non-zero at the corresponding `run_step` command and prints the failing script diagnostic.
- guard-test drift remains deterministic through required missing-token messages.

## Example Usage

```bash
scripts/run-naming-lock-contract-suite.sh
```

Targeted checks only:

```bash
scripts/test-smoke-sec4-run-lasm-db-adapter-script-contract.sh
scripts/test-smoke-sec4-run-lasm-db-adapter-script-contract-guard.sh
```

## Tradeoffs and Next Steps

Tradeoffs:
- naming-lock suite runtime increases slightly as more script contracts are added.

Next steps:
1. consider grouped runtime-smoke contract phases to keep naming-lock output easier to triage,
2. add workflow-level contract tokens if direct naming-lock workflow visibility for these steps becomes required.
