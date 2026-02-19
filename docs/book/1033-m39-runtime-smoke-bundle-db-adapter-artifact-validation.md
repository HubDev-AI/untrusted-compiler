# 1033 M39 Slice: Runtime-Smoke Bundle DB Adapter Artifact Validation

## What It Is

This slice extends runtime-smoke bundle validation so LASM DB adapter artifact branches are first-class bundle checks:

- `scripts/check-runtime-smoke-db-adapter-artifacts.sh`
- `scripts/test-check-runtime-smoke-db-adapter-artifacts.sh`
- `scripts/check-runtime-smoke-bundle.sh`
- `scripts/test-check-runtime-smoke-bundle.sh`

It also tightens naming-lock/closure enforcement so the new checker is required in CI contracts.

## Why It Exists

Runtime-smoke workflow already executes DB adapter lanes (`lasm-db-records-log`, `lasm-db-sqlite`), but bundle validation previously only verified hello-api lanes (`default`, `max-body`).

Without db-adapter artifact validation inside bundle checks, workflow evidence could drift for adapter lanes without failing the bundle contract.

## How It Works Internally

1. New db-adapter artifact checker:
   - validates required db-adapter artifact files (`run-metadata`, per-endpoint headers/body/run logs, warmup log),
   - validates deterministic metadata shape (`dbAdapter`, `dbAdapterLabel`, `runFlags`, port/timeout/oneshot invariants),
   - validates endpoint payload contracts (`db-exec`, `db-exec-tx`, `db-query-one`, `db-records`),
   - validates adapter persistence shape:
     - `records-log` requires `records.log` and forbids `records.sqlite3`,
     - `sqlite` requires `records.sqlite3` and forbids `records.log`.

2. Bundle checker expansion:
   - still validates hello-api branches (`default`, `max-body`) via `check-runtime-smoke-artifacts.sh`,
   - now additionally requires:
     - `lasm-db-records-log`,
     - `lasm-db-sqlite`,
     and validates them via `check-runtime-smoke-db-adapter-artifacts.sh`.

3. Branch index compatibility:
   - bundle still emits the existing hello-api branch index contract (`default`, `max-body`) unchanged,
   - db-adapter branch validation is enforced as bundle pass/fail gating, not index-schema expansion.

4. CI/closure enforcement:
   - naming-lock contract suite now runs `scripts/test-check-runtime-smoke-db-adapter-artifacts.sh`,
   - closure gate `M16-D` now requires this checker token in naming-lock CI contract source.

## Inputs / Outputs and Constraints

Inputs:
- runtime-smoke artifact root containing hello-api and db-adapter branch directories.

Outputs:
- deterministic pass/fail artifact validation across all runtime-smoke workflow lanes.

Constraints:
- branch index schema remains intentionally unchanged in this slice to avoid downstream consumer churn.

## Failure Modes and Diagnostics

- Missing db-adapter branch directory:
  - `runtime-smoke bundle missing branch directory: lasm-db-sqlite`
- DB-adapter metadata drift:
  - `run-metadata.txt runFlags field does not match expected db-adapter runtime flag shape`
- Persistence drift:
  - `sqlite db-adapter artifacts missing records.sqlite3`
  - `records-log db-adapter artifacts should not include records.sqlite3`

## Example Usage

```bash
scripts/test-check-runtime-smoke-db-adapter-artifacts.sh
scripts/test-check-runtime-smoke-bundle.sh
scripts/check-runtime-smoke-bundle.sh --artifacts-root build/runtime-smoke --index-path build/runtime-smoke/runtime-smoke-branch-index.json
```

## Tradeoffs and Next Steps

Tradeoffs:
- bundle validation now enforces a broader artifact surface, so db-adapter smoke output changes require synchronized checker updates.

Next steps:
1. consider optional db-adapter summary index generation for faster CI triage views,
2. extend troubleshooting matrix mappings with db-adapter checker diagnostics if operator support needs increase.
