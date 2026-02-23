# M39: LASM DB records affected-rows range filters

## What changed

`DbListRecordsResponse` now supports write-impact range filters:

- `affectedRowsMin=<u64>`
- `affectedRowsMax=<u64>`

These filters compose with all existing filters (`op`, `db`, `tx`, template/time/id ranges, order, includeRecords).

## Validation behavior

Deterministic validation rejects:

- non-integer `affectedRowsMin`/`affectedRowsMax`,
- invalid ranges where `affectedRowsMin > affectedRowsMax`.

Invalid requests return:

- code: `DB.RECORDS_FILTER_INVALID`
- kind: `validation`

## Response metadata

`filters` now includes:

- `affectedRowsMin`
- `affectedRowsMax`

This gives operators explicit write-impact range visibility during DB runtime investigation.
