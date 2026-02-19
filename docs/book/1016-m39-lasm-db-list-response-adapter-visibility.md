# 1016 M39 Slice: LASM DB List Response Adapter Visibility

## What It Is

This slice extends LASM `DbListRecordsResponse` payloads to include the active DB adapter label:

- `records.log` when running the default file adapter,
- `sqlite` when `SEC4_RT_LASM_DB_ADAPTER=sqlite` is active.

## Why It Exists

After introducing adapter selection (`1015`), operators need a direct runtime-visible signal confirming which persistence backend is currently active.

Adding adapter visibility to the existing record-list inspection endpoint improves alpha usability without changing intrinsic semantics.

## How It Works Internally

1. Runtime adapter label helper:
   - maps internal adapter enum to a stable external string (`records.log` / `sqlite`).

2. DB list response materialization:
   - when handling `DbListRecordsResponse`, runtime now emits:
     - `ok`,
     - `count`,
     - `adapter`,
     - `records`.

3. Integration tests:
   - existing records-log integration flow now asserts `"adapter":"records.log"`,
   - SQLite integration flow now asserts `"adapter":"sqlite"`.

## Inputs / Outputs and Constraints

Inputs:
- same DB intrinsic runtime state used for record listing.

Outputs:
- `DbListRecordsResponse` JSON now includes `adapter` alongside count and records.

Constraints:
- adapter label is informational; no DB execution behavior changes,
- response remains deterministic and backward-compatible for existing fields.

## Failure Modes and Diagnostics

- no new failure paths were introduced;
- existing dynamic-state lock failure path remains unchanged (`500 HTTP.INTERNAL`).

## Example Usage

```bash
SEC4_RT_LASM_DB_ADAPTER=sqlite \
cargo run -q -p sec4 -- run \
  --path examples/lasm-alpha-full \
  --backend lasm \
  --db-base /tmp/sec4-lasm-db
```

`GET /db/records` now includes:

```json
{
  "ok": true,
  "count": 2,
  "adapter": "sqlite",
  "records": [ ... ]
}
```

## Tradeoffs and Next Steps

Tradeoffs:
- adds one new response field that consumers may choose to ignore, but does not alter existing fields.

Next steps:
1. expose adapter label in benchmark/report summaries where DB paths are exercised,
2. add adapter-selection guidance to canonical operator playbooks for reproducible LASM+DB runs.
