# 1006 M39 Slice: Remove LASM Schema-Fallback DB Write Arms

## What It Is

This slice removes legacy LASM response-schema DB write fallback branches for:

- `DbExecResponse`
- `DbExecTxResponse`

from `apply_lasm_dynamic_response_materialization` in `compiler/sec4-cli/src/main.rs`.

## Why It Exists

P0 LASM DB parity requires DB writes to execute through intrinsic flow, not response-schema switching.

With intrinsic extraction now available for `sql.q` + `db.exec` / `db.execTx`, keeping schema-only write arms risks drift and duplicate behavior paths.

## How It Works Internally

Before this slice, LASM had two write paths:

1. intrinsic marker execution path (newer),
2. schema-name fallback write path (`DbExecResponse` / `DbExecTxResponse`).

This slice deletes path (2) for write operations.

Result:
- DB write execution now happens only when intrinsic markers are present,
- schema-hint-only write handlers no longer trigger runtime DB writes.

## Inputs / Outputs and Constraints

Inputs:
- route handlers that include intrinsic DB calls (`db.exec`, `db.execTx`) and therefore emit internal LASM DB markers.

Outputs:
- deterministic DB write envelopes and persisted `records.log` entries via intrinsic path only.

Constraints:
- `DbQueryOneResponse` schema-switch fallback was already removed in this path; query-one execution now requires intrinsic `db.queryOne` dispatch.
- `DbListRecordsResponse` remains a response schema contract, while execution uses intrinsic `listRecords` dispatch.

## Failure Modes and Diagnostics

No new diagnostic codes were introduced.

Write-route diagnostics continue to come from intrinsic materialization path:
- `DB.SQL_TEMPLATE_INVALID`
- `DB.EXEC_INVALID`
- `DB.EXEC_TX_INVALID`
- `DB.EXEC_TX_HANDLE_INVALID`
- `HTTP.INTERNAL`

## Example Usage

Use intrinsic-backed handlers (example pattern):

```ut
fn dbExec() effects { net, db.write } -> Int {
  let db = DbCap();
  let query = sql.q(validate.nonEmpty(req.query("template")), validate.nonEmpty(req.query("params")));
  db.exec(db, query);
  res.json(200, "DbExecResponse", 0);
  0
}
```

## Tradeoffs and Next Steps

Tradeoffs:
- behavior is now cleaner and parity-focused, but schema-hint-only DB write stubs no longer provide compatibility writes.

Next steps:
1. continue LASM-default `sec4 run` rollout once intrinsic parity is stable,
2. verify adapter-agnostic parity for DB path under SQLite/Postgres records persistence,
3. keep expanding high-load runtime hardening without reintroducing schema-switch write branches.
