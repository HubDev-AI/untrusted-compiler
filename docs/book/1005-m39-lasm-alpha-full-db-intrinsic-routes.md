# 1005 M39 Slice: LASM Alpha-Full DB Intrinsic Routes

## What It Is

This slice updates `examples/lasm-alpha-full` so DB write routes execute real DB intrinsics in handler code:

- `/db/exec` uses `DbCap` + `sql.q` + `db.exec`
- `/db/exec-tx` uses `DbCap` + `sql.q` + `db.tx` + `db.execTx`

The response schemas remain compatible (`DbExecResponse`, `DbExecTxResponse`), but DB writes are now driven by intrinsic execution markers extracted from the handler call graph.

## Why It Exists

The LASM roadmap priority requires moving DB behavior off schema-name-only write bridges and onto intrinsic execution. The canonical operator example (`lasm-alpha-full`) must reflect that direction so users exercise the real path by default.

## How It Works Internally

1. Handler code now explicitly constructs DB capability/query/tx values.
2. LASM route extraction sees these intrinsic calls and records internal DB operation markers.
3. LASM runtime materialization executes those markers:
- appends deterministic records into in-memory DB state,
- persists entries to `records.log` under `--db-base` / `SEC4_RT_LASM_DB_BASE`.
4. Response payloads remain deterministic and backward-compatible for existing route contracts.

## Inputs / Outputs and Constraints

Inputs:
- request query values `template` and `params` for DB write routes,
- optional DB base path via `--db-base` / `SEC4_RT_LASM_DB_BASE`.

Outputs:
- deterministic DB write envelopes (`recordId`, `db`, `op`, `template`, `params`, `tx`),
- persisted `records.log` entries.

Constraints:
- `db.queryOne` and list-record query paths are now intrinsic-backed in current LASM `lasm-alpha-full` flow.
- tx handles are deterministic but process-local runtime state.

## Failure Modes and Diagnostics

Route calls preserve deterministic DB diagnostics from runtime materialization:

- `DB.SQL_TEMPLATE_INVALID` for missing/empty SQL template,
- `DB.EXEC_INVALID` for invalid `db.exec` inputs,
- `DB.EXEC_TX_INVALID` / `DB.EXEC_TX_HANDLE_INVALID` for invalid tx flows,
- `HTTP.INTERNAL` when dynamic state lock is unavailable.

## Example Usage

```ut
fn dbExec() effects { net, db.write } -> Int {
  let db = DbCap();
  let template = validate.nonEmpty(req.query("template"));
  let params = validate.nonEmpty(req.query("params"));
  let query = sql.q(template, params);
  db.exec(db, query);
  res.json(200, "DbExecResponse", 0);
  0
}

fn dbExecTx() effects { net, db.write, db.tx } -> Int {
  let db = DbCap();
  let template = validate.nonEmpty(req.query("template"));
  let params = validate.nonEmpty(req.query("params"));
  let query = sql.q(template, params);
  let tx = db.tx(db);
  db.execTx(tx, query);
  res.json(200, "DbExecTxResponse", 0);
  0
}
```

## Tradeoffs and Next Steps

Tradeoffs:
- the sample now reflects intrinsic DB write/read behavior through explicit handlers and marker-based runtime dispatch.

Next steps:
1. keep tightening deterministic DB handle lifecycle parity against C runtime behavior,
2. validate multi-op ordering/diagnostic parity across adapters,
3. continue post-DB default-runtime rollout and load hardening tasks.
