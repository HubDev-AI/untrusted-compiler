# 112 M7 Slice: SQL Query Builder Intrinsic Bridge

This chapter documents the next M7 bootstrap slice: adding `sql.q` query-construction helper bridging.

## SQL Query Construction Helper

### What it is

Added intrinsic support for:
- `sql.q` / `sql_q`

Lowered runtime ABI stub:
- `sec4_rt_sql_q`

### Why it exists

The v0 stdlib surface includes an explicit SQL query-construction helper before execution sinks (`db.exec`, `db.execTx`, `db.queryOne`). Without this bridge, spec-shaped query builder calls cannot compile through `c-bin`.

### How it works internally

- Added semantic intrinsic entry:
  - `sql.q` returns `SqlQuery` (no effect/capability requirement).
- Added `sql` namespace recognition in semantic intrinsic namespace catalog.
- Added C rewrite mapping:
  - `sql.q` / `sql_q` -> `sec4_rt_sql_q`
- Added runtime C declaration/definition for `sec4_rt_sql_q`.
- Added security-map tagging:
  - `sql.q` -> `gate.sql.parameterize`
- Added argument role labeling for security-map call records:
  - `template`, `params`
- Updated db/fs/net integration fixtures to exercise `sql.q` together with DB sinks.

### Inputs, outputs, and constraints

- Input: calls like `sql.q(template, params)`.
- Output: generated C invoking `sec4_rt_sql_q(...)`.
- Constraints:
  - runtime behavior remains stubbed in M7 bootstrap mode.
  - this slice validates compile/link and metadata coverage, not full SQL templating semantics.

### Failure modes and diagnostics

- Misspelled `sql.q` helper names fail semantic/C compile phases.
- SQL hygiene findings (`SQL_SELECT_WITHOUT_LIMIT`) now also observe transaction sink calls because they share query extraction paths with the same query object flow.

### Example usage

```ut
fn txDemo(db: DbCap, tx: TxCap) effects { db.tx, db.write } -> Int {
  let built = sql.q("SELECT 1", 2);
  db.tx(db);
  db.execTx(tx, built);
  0
}
```

### Tradeoffs and next steps

- This adds the query-construction surface with minimal complexity while keeping execution sinks unchanged.
- Next steps are stronger query-shape validation and eventual tag-template lowering for `sql\`...\`` syntax.

## Tests updated

- `compiler/sec4-core/tests/c_backend.rs`
  - runtime ABI assertions include `sec4_rt_sql_q`
  - db/fs/net rewrite coverage now asserts `sql.q` lowering with transaction sinks
- `compiler/sec4-cli/tests/json_output.rs`
  - db/fs/net `c-bin` integration fixture now builds query via `sql.q`
  - generated C assertions include `sec4_rt_sql_q(...)`
