# M39: LASM DB records filter and op-count expansion

## What changed

- Extended `DbListRecordsResponse` filtering with:
  - `tx=<integer>=0+`
  - `templateContains=<non-empty substring>`
- Kept existing `limit`, `op`, and `db` filters and made the full filter surface deterministic.
- Added operation count summaries:
  - `opCounts` for the filtered record set.
  - `opCountsGlobal` for full retained DB record history.

## Why this was needed

Operators needed better runtime visibility when debugging LASM DB behavior under load:

- isolate record windows by tx handle and template fragment,
- quickly compare filtered behavior to global retained history,
- avoid silent bad filter typos.

## Runtime behavior

- `tx` filter accepts integer values `>= 0`.
- `templateContains` filter requires a non-empty string.
- Invalid filter values return deterministic error envelopes:
  - code: `DB.RECORDS_FILTER_INVALID`
  - kind: `validation`

## Output additions

`DbListRecordsResponse` now includes:

- `filters.tx`
- `filters.templateContains`
- `opCounts.exec`
- `opCounts.execTx`
- `opCounts.queryOne`
- `opCountsGlobal.exec`
- `opCountsGlobal.execTx`
- `opCountsGlobal.queryOne`

All additions preserve existing intrinsic/runtime contracts.
