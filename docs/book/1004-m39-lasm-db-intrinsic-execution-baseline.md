# 1004 M39 Slice: LASM DB Intrinsic Execution Baseline

## What It Is

This slice introduces intrinsic-driven LASM DB execution for route handlers that call:

- `sql.q(...)`
- `db.exec(...)`
- `db.execTx(...)`

Instead of relying on schema-name response hints to trigger DB writes, LASM now extracts DB call plans from handler call graphs and executes those DB operations directly during request materialization.

## Why It Exists
The previous LASM DB path depended on response-schema branching (`DbExecResponse` / `DbExecTxResponse`) as a compatibility bridge.
This slice moves DB writes to the intrinsic path so behavior follows actual handler code.

## How It Works Internally

1. Route planning now extracts DB operation plans from handler AST call graphs:
- detects `db.exec` / `db.execTx` calls,
- resolves `sql.q(template, params)` query descriptors,
- resolves `db.tx(db)` tx sources for `db.execTx`.

2. The extracted plan is encoded into internal LASM route headers:
- `X-Sec4-Internal-Db-Op`
- `X-Sec4-Internal-Db-*` argument markers.

3. At request time, LASM materialization consumes these internal markers and executes the DB operation against dynamic state:
- appends deterministic records into in-memory DB state,
- persists records into `records.log` under `SEC4_RT_LASM_DB_BASE` / `--db-base`,
- emits deterministic JSON payloads matching existing DB response envelopes.

4. `db.execTx` now gets deterministic tx-handle registration in LASM runtime state when tx comes from `db.tx(...)` extraction.

## Inputs / Outputs and Constraints

Inputs:
- handler call graph containing `sql.q` + `db.exec` or `db.execTx`,
- optional request-derived placeholders in extracted DB marker values,
- optional `--db-base` / `SEC4_RT_LASM_DB_BASE` for persisted records.

Outputs:
- deterministic DB write payloads (`recordId`, `db`, `op`, `template`, `params`, `tx`),
- persisted `records.log` records for write operations.

Constraints:
- this slice applies to extracted `db.exec` / `db.execTx` / `db.queryOne` intrinsic paths,
- DB list responses use the `listRecords` intrinsic marker, not response-schema branching.

## Failure Modes and Diagnostics

Deterministic DB diagnostics preserved on intrinsic path include:

- `DB.SQL_TEMPLATE_INVALID` when extracted `sql.q` template is empty,
- `DB.EXEC_INVALID` for invalid `db.exec` DB/query handles,
- `DB.EXEC_TX_INVALID` for invalid `db.execTx` tx/query handles,
- `DB.EXEC_TX_HANDLE_INVALID` when tx handle did not originate from `db.tx`,
- `HTTP.INTERNAL` when LASM dynamic state lock is unavailable.

Persistence write failures are emitted as deterministic warnings to stderr and do not crash route extraction.

## Example Usage

```ut
fn dbExec() effects { net, db.write } -> Int {
  let db = DbCap();
  let template = validate.nonEmpty(req.query("template"));
  let params = validate.nonEmpty(req.query("params"));
  let query = sql.q(template, params);
  db.exec(db, query);
  res.json(200, "DbExecRuntimeResponse", 0);
  0
}

fn dbExecTx() effects { net, db.write, db.tx } -> Int {
  let db = DbCap();
  let template = validate.nonEmpty(req.query("template"));
  let params = validate.nonEmpty(req.query("params"));
  let query = sql.q(template, params);
  let tx = db.tx(db);
  db.execTx(tx, query);
  res.json(200, "DbExecTxRuntimeResponse", 0);
  0
}
```

## Tradeoffs and Next Steps

Tradeoffs:
- tx-handle lifecycle is deterministic and process-local (in-memory) rather than persisted.

Next steps:
1. tighten parity against C runtime DB handle/query validation across all LASM DB operations,
2. continue package/module extraction for runtime layers without semantic changes.
