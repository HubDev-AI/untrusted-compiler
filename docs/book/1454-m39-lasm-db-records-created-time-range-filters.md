# M39: LASM DB records created-time range filters

## What changed

`DbListRecordsResponse` now supports timestamp-window filters:

- `createdFromMs=<u64>`
- `createdToMs=<u64>`

Both filters apply to `record.created_at_ms` and compose with existing `limit`, `op`, `db`, `tx`, and `templateContains` filters.

## Validation behavior

Deterministic validation now rejects:

- non-integer `createdFromMs`/`createdToMs`,
- range inversions where `createdFromMs > createdToMs`.

Rejected requests return:

- code: `DB.RECORDS_FILTER_INVALID`
- kind: `validation`

## Response metadata

`filters` payload now includes:

- `createdFromMs`
- `createdToMs`

This keeps runtime filter state explicit for operator debugging and reproducible traces.
