# M39: LASM DB records paramsContains filter

## What changed

`DbListRecordsResponse` now supports payload substring filtering on persisted DB records:

- `paramsContains=<non-empty string>`

The filter matches against each record's normalized `params` string and composes with the existing filter set (`op`, `db`, `tx`, `templateContains`, time/id ranges, order, includeRecords, and affected-rows ranges).

## Validation behavior

Deterministic validation rejects empty or whitespace-only values for `paramsContains`.

Invalid requests return:

- code: `DB.RECORDS_FILTER_INVALID`
- kind: `validation`

## Response metadata

`filters` now includes:

- `paramsContains`

This makes payload-centric record investigations deterministic without changing existing DB intrinsic behavior.
