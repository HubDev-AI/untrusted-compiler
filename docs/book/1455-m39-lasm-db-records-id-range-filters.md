# M39: LASM DB records id-range filters

## What changed

`DbListRecordsResponse` now accepts deterministic record-id filters:

- `idFrom=<u64>=1+`
- `idTo=<u64>=1+`

Both filters apply to persisted DB record ids and compose with existing filter dimensions.

## Validation behavior

Deterministic validation rejects:

- non-integer `idFrom`/`idTo`,
- values below `1`,
- invalid ranges where `idFrom > idTo`.

Invalid inputs return:

- code: `DB.RECORDS_FILTER_INVALID`
- kind: `validation`

## Response metadata

`filters` payload now includes:

- `idFrom`
- `idTo`

This gives operators explicit id-window traceability in runtime responses.
