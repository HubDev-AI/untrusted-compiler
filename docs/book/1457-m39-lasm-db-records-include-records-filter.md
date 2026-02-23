# M39: LASM DB records includeRecords filter

## What changed

`DbListRecordsResponse` now supports:

- `includeRecords=true|false|1|0`

Default remains `true`.

## Behavior

- `includeRecords=true`: existing behavior, full `records` array is emitted.
- `includeRecords=false`: response still includes counts/filters/telemetry, but emits `records: []`.

This enables lightweight summary polling without transferring full retained record payloads.

## Validation

Invalid values return deterministic validation envelopes:

- code: `DB.RECORDS_FILTER_INVALID`
- message: `db records includeRecords filter must be one of true, false, 1, 0`
