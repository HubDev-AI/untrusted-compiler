# 1021 M39 Slice: LASM DB Adapter Operator Smoke Script

## What It Is

This slice adds a canonical operator smoke script for LASM DB adapter flows:

- `scripts/smoke-sec4-run-lasm-db-adapter.sh`

and adds source-contract protection for that script:

- `scripts/test-smoke-sec4-run-lasm-db-adapter-script-contract.sh`
- `scripts/test-smoke-sec4-run-lasm-db-adapter-script-contract-guard.sh`

## Why It Exists

`sec4 run --db-adapter` is now explicit, but operators still needed a reusable runbook-grade smoke flow that:

- executes real LASM DB intrinsic routes end-to-end,
- validates adapter-specific persistence expectations (`records.log` vs `records.sqlite3`),
- stays reproducible with stable script-contract checks.

## How It Works Internally

1. Script setup and input contract:
   - accepts `--db-adapter <records-log|sqlite>`, `--db-base`, `--serve-timeout-ms`, and optional artifacts path,
   - copies `examples/lasm-alpha-full` into a temporary workspace,
   - warms `sec4` build once before request loop.

2. Deterministic oneshot request flow:
   - runs `sec4 run --backend lasm --oneshot --db-base ... --db-adapter ...` per request,
   - sends authenticated requests for:
     - `POST /db/exec?...`,
     - `POST /db/exec-tx?...`,
     - `GET /db/query-one?...`,
     - `GET /db/records`,
   - validates deterministic response invariants (`recordId`, `op`, `rowSchema`, adapter label, ordered record list).

3. Adapter-specific persistence checks:
   - `records-log` mode requires `records.log` and forbids `records.sqlite3`,
   - `sqlite` mode requires `records.sqlite3` and forbids `records.log`.

4. Contract hardening:
   - token contract test locks key smoke-script invariants,
   - guard test mutates required tokens and enforces deterministic failure diagnostics.

## Inputs / Outputs and Constraints

Inputs:
- LASM project path (default: `examples/lasm-alpha-full`),
- adapter selection (`records-log` or `sqlite`),
- optional DB base/artifacts path overrides.

Outputs:
- pass/fail smoke execution,
- optional captured request/run artifacts and persisted DB files.

Constraints:
- requires local tooling (`cargo`, `curl`, `jq`, `python3`),
- executes multiple `sec4 run --oneshot` invocations sequentially.

## Failure Modes and Diagnostics

- invalid script arguments produce deterministic usage/validation errors,
- request-timeout/early-exit paths emit explicit request-name diagnostics and print run logs,
- payload/persistence mismatches emit deterministic endpoint-specific failure messages,
- contract drift in script content is caught by token-contract + guard scripts.

## Example Usage

```bash
scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter records-log
scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter sqlite
```

## Tradeoffs and Next Steps

Tradeoffs:
- smoke script is intentionally strict and token-contract based, which increases maintenance when structure changes.

Next steps:
1. wire this smoke script into broader operator/runtime smoke orchestration,
2. add adapter-specific artifact summaries for quick triage in CI logs.
