# 1015 M39 Slice: LASM DB SQLite Adapter Baseline

## What It Is

This slice adds a first embedded DB adapter option for LASM DB intrinsic persistence:

- default adapter remains file-backed `records.log`,
- optional SQLite adapter is enabled with `SEC4_RT_LASM_DB_ADAPTER=sqlite`,
- DB intrinsic runtime behavior (`db.exec`, `db.execTx`, `db.queryOne`) stays unchanged at the HTTP contract level.

## Why It Exists

Roadmap ordering after LASM DB intrinsic parity requires DB adapter progression with SQLite first, while preserving the existing alpha-safe file adapter path.

This slice delivers that progression with minimal surface-area change:

- no new CLI flags,
- no schema-switch fallback reintroduction,
- explicit opt-in via environment selection.

## How It Works Internally

1. Adapter selection:
   - runtime reads `SEC4_RT_LASM_DB_ADAPTER`,
   - accepted values:
     - `sqlite` -> SQLite adapter,
     - empty / `records` / `records.log` / `records-log` -> file adapter,
   - unknown values emit a warning and fall back to `records.log`.

2. Store paths (under `--db-base` or `SEC4_RT_LASM_DB_BASE`):
   - file adapter: `records.log`,
   - SQLite adapter: `records.sqlite3`.

3. State bootstrap:
   - dynamic state loads existing DB records from the selected adapter,
   - tx-handle and next-record-id derivation continue using loaded records.

4. Persistence dispatch:
   - file adapter keeps existing newline-delimited JSON write path,
   - SQLite adapter writes deterministic ordered records into table `lasm_db_records` inside a single transaction (`DELETE` + ordered `INSERT`),
   - SQLite schema is initialized with `CREATE TABLE IF NOT EXISTS`.

5. Integration coverage:
   - existing `records.log` integration test remains as default-adapter guard,
   - new SQLite integration test executes the same end-to-end intrinsic flow with `SEC4_RT_LASM_DB_ADAPTER=sqlite` and verifies:
     - `records.sqlite3` persists deterministic records,
     - legacy `records.log` is not emitted in SQLite mode,
     - `db.queryOne` parity remains deterministic.

## Inputs / Outputs and Constraints

Inputs:
- optional runtime env `SEC4_RT_LASM_DB_ADAPTER`,
- existing DB intrinsic marker extraction and runtime operation plans.

Outputs:
- persisted DB records in either `records.log` or `records.sqlite3` based on adapter selection.

Constraints:
- adapter selection is LASM-runtime-only,
- default behavior remains backward-compatible (`records.log`),
- SQLite path requires writable DB base directory when persistence is enabled.

## Failure Modes and Diagnostics

- unknown adapter value:
  - warning emitted,
  - runtime falls back to `records.log`.
- SQLite open/schema/query/persist failures:
  - startup load failures log warnings and continue with empty in-memory records,
  - persist failures are surfaced through existing runtime warning path used by DB intrinsic writes.

## Example Usage

```bash
SEC4_RT_LASM_DB_ADAPTER=sqlite \
cargo run -q -p sec4 -- run \
  --path examples/lasm-alpha-full \
  --backend lasm \
  --db-base /tmp/sec4-lasm-db
```

Expected DB record persistence path:

- `/tmp/sec4-lasm-db/records.sqlite3`

## Tradeoffs and Next Steps

Tradeoffs:
- SQLite adapter currently rewrites persisted record table per write transaction to keep deterministic ordering simple.

Next steps:
1. add adapter-level benchmark comparisons (`records.log` vs SQLite) for throughput/latency/memory deltas,
2. define post-alpha external DB adapter contract under the same intrinsic surface.
