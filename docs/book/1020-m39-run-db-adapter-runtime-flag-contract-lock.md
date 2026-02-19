# 1020 M39 Slice: `sec4 run` DB Adapter Runtime-Flag Contract Lock

## What It Is

This slice hardens naming-lock contract coverage for `sec4 run` DB adapter wiring by extending:

- `scripts/test-sec4-run-runtime-flag-contract.sh`
- `scripts/test-sec4-run-runtime-flag-contract-guard.sh`

The contract now explicitly checks `--db-adapter` runtime-flag invariants in CLI source.

## Why It Exists

`--db-adapter` was added recently for LASM persistence selection, but runtime-flag drift protection only covered older `run` flags (`--port`, `--oneshot`, `--max-body-bytes`, `--serve-timeout-ms`).

Without source-level contract checks, future refactors could silently break:

- LASM-only backend guard behavior,
- LASM dynamic-state adapter bridge wiring,
- cluster worker forwarding of adapter selection.

## How It Works Internally

1. Contract assertions expanded:
   - requires `Run { db_adapter: Option<RunDbAdapter> }` field presence,
   - requires deterministic LASM-only guard diagnostic string:
     - `run failed: --db-adapter is only supported with --backend lasm`,
   - requires LASM dynamic-state adapter bridge pattern:
     - `db_adapter.map(run_db_adapter_to_lasm_db_records_adapter)`,
   - requires cluster worker adapter forwarding bridge pattern:
     - `push_optional_db_adapter_run_arg(&mut cmd, config.db_adapter);`.

2. Guard fixture rewritten:
   - positive synthetic fixture includes all required token patterns,
   - negative fixtures mutate one invariant at a time to ensure deterministic failure for:
     - lasm-only guard drift,
     - cluster bridge drift,
     - existing timeout bridge drift.

## Inputs / Outputs and Constraints

Inputs:
- CLI source file (`compiler/sec4-cli/src/main.rs` by default).

Outputs:
- pass/fail result from script-level contract checks.

Constraints:
- checks are token/pattern contracts (source-level), not semantic Rust compilation.

## Failure Modes and Diagnostics

- when a required pattern is missing, contract test exits non-zero with:
  - `missing run-flag contract pattern (<label>) in <path>`
- guard script asserts those failures stay deterministic for drifted fixtures.

## Example Usage

```bash
scripts/test-sec4-run-runtime-flag-contract.sh
scripts/test-sec4-run-runtime-flag-contract-guard.sh
```

## Tradeoffs and Next Steps

Tradeoffs:
- source-pattern contracts can be brittle across large structural rewrites.

Next steps:
1. keep this contract synchronized whenever new LASM-only `sec4 run` flags are introduced,
2. add equivalent operator-facing smoke-script coverage for canonical LASM DB adapter flows.
