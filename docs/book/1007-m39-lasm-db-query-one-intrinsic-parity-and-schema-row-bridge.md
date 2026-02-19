# 1007 M39 Slice: LASM Query-One Intrinsic Parity and `schema.row` Bridge

## What It Is

This slice completes the remaining active LASM DB parity gap for `db.queryOne` route flows by:

- adding a new `schema.row(...)` bridge intrinsic for row-schema handles in zero-arg handlers,
- wiring `schema.row` through semantic typing, C backend lowering, and C runtime ABI,
- migrating LASM DB query-one routes/examples/tests to intrinsic-backed `db.queryOne(...)`,
- removing the `DbQueryOneResponse` schema-switch fallback branch from LASM response materialization.

## Why It Exists

After write-path parity (`db.exec`, `db.execTx`), query-one still depended on response-schema fallback for route-facing handlers because row-schema values could not be expressed in zero-arg bridge handlers without typed schema parameters.

`schema.row(...)` provides a minimal bridge that preserves existing row-schema safety rules while allowing intrinsic query-one paths to run in current handler constraints.

## How It Works Internally

1. Semantic layer (`compiler/sec4-core/src/semantic.rs`):
   - adds intrinsic `schema.row` / `schema_row`,
   - enforces one numeric, non-secret, non-untrusted argument,
   - returns `Schema<Int>` so `db.queryOne(..., rowSchema)` keeps descriptor typing checks.

2. C backend/runtime (`compiler/sec4-core/src/c_backend.rs`, `runtime/c/sec4_runtime.{h,c}`):
   - lowers `schema.row(...)` to `sec4_rt_schema_row(...)`,
   - adds runtime ABI function `sec4_rt_schema_row(int64_t)`,
   - runtime bridge returns `0` for non-positive handles (letting existing query-one validation envelopes fire deterministically).

3. LASM extraction/runtime (`compiler/sec4-cli/src/main.rs`):
   - DB value-template extraction now unwraps `schema.row(...)` for internal DB marker generation,
   - `DbQueryOneResponse` fallback materialization arm is removed.

4. Route fixtures/examples:
   - query-one routes now call:
     - `let rowSchema = schema.row(validate.int64(req.query("row_schema")));`
     - `db.queryOne(db, query, rowSchema);`

## Inputs / Outputs and Constraints

Inputs:
- query-one handlers using `DbCap` + `sql.q` + `schema.row(validate.int64(...))` + `db.queryOne`.

Outputs:
- deterministic intrinsic-backed query-one envelopes and row materialization from persisted `records.log` state.

Constraints:
- current bridge still requires zero-arg route handlers,
- `row_schema` must be numeric and positive at runtime,
- this slice does not change list-records compatibility path (`DbListRecordsResponse`).

## Failure Modes and Diagnostics

Existing deterministic query-one/runtime diagnostics remain primary:

- `DB.SQL_TEMPLATE_INVALID`
- `DB.QUERY_ONE_INVALID`
- `DB.QUERY_ONE_NOT_FOUND`
- `HTTP.INTERNAL`

New semantic diagnostic:

- `E4001`: `schema.row argument must be numeric`

## Example Usage

```ut
fn dbQueryOne() effects { net, db.read } -> Int {
  let db = DbCap();
  let template = validate.nonEmpty(req.query("template"));
  let params = validate.nonEmpty(req.query("params"));
  let rowSchema = schema.row(validate.int64(req.query("row_schema")));
  let query = sql.q(template, params);
  db.queryOne(db, query, rowSchema);
  res.json(200, "DbQueryOneResponse", 0);
  0
}
```

## Tradeoffs and Next Steps

Tradeoffs:
- introduces a narrow bridge intrinsic (`schema.row`) to work within zero-arg handler constraints without reopening generic route-parameter dispatch.

Next steps:
1. move to post-DB fixed order: make LASM default backend for `sec4 run`,
2. run LASM default-path stability/load hardening,
3. progress DB adapters (SQLite first) behind the same intrinsic surface.
