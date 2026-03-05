# M39: LASM DB records affected-row filtered/global totals

## What changed

`DbListRecordsResponse` now emits two additional affected-row totals:

- `affectedRowsFilteredTotal`: sum of `affected_rows` across all records that match active filters, before `limit` windowing.
- `affectedRowsGlobalTotal`: sum of `affected_rows` across all retained records in runtime history.

Existing `affectedRowsTotal` is preserved and continues to represent only the returned response-window records.

## Why

With filters and limit windows enabled, window-only totals are useful but not sufficient for operator analysis.

The new totals make it explicit how write impact differs between:

- the response window,
- the full filtered set,
- global retained history.

## Compatibility

- No intrinsic contracts changed.
- Existing `affectedRowsTotal` behavior is unchanged.
- New fields are additive and deterministic.
