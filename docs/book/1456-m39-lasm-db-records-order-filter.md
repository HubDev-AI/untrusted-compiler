# M39: LASM DB records order filter

## What changed

`DbListRecordsResponse` now supports:

- `order=asc`
- `order=desc`

Default remains `asc`.

## Behavior

- `asc`: existing chronological ordering behavior.
- `desc`: latest-first ordering for the filtered record set.
- In `desc` mode, `limit` applies as a leading window over latest-first records.

## Validation

Invalid values return deterministic validation envelopes:

- code: `DB.RECORDS_FILTER_INVALID`
- message: `db records order filter must be one of asc or desc`

## Response metadata

`filters.order` now exposes the effective ordering mode for deterministic operator traces.
