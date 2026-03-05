# 1546 M39 Slice: LASM DB records `queryOne` client-path unification

## What it is

This slice moves `records.log` adapter `db.queryOne` lookup/materialization into the shared LASM DB client operation entrypoint:

- `compiler/sec4-cli/src/lasm_db_client/operations.rs`
  - `run_lasm_db_query_one_operation(...)`

Dispatch no longer performs direct records-log query-one lookup/materialization logic.

## Why it exists

Before this change, adapter behavior was split:

- `postgres`/`sqlite` queryOne paths already executed through `lasm_db_client`.
- `records.log` queryOne path was still implemented directly in runtime dispatch.

That split made adapter parity harder to maintain and left compatibility-style branching in an alpha-critical intrinsic path.

## How it works internally

1. `LasmDbQueryOneOperationResult` now includes `RecordsLog { row }`.
2. `run_lasm_db_query_one_operation(...)` now accepts:
   - `db` handle,
   - normalized `params` string,
   - `row_schema` handle,
   so records adapter can resolve and materialize row payload internally.
3. For `records.log` adapter, the client operation function now:
   - resolves latest matching record by signature,
   - builds deterministic row object payload,
   - returns `NotFound` when no match exists.
4. Runtime dispatch now calls the same client operation entrypoint for all adapters and keeps only orchestration duties:
   - response shaping,
   - runtime record append,
   - envelope/status handling.

## Inputs, outputs, and constraints

- Input contract (unchanged): `db.queryOne` requires db handle, template, params, row schema.
- Output contract (unchanged): deterministic success envelope with row payload, or deterministic `404` not-found envelope.
- Constraint: preparation mismatch remains a deterministic internal preparse mismatch response.

## Failure modes and diagnostics

- Missing row: deterministic `DB.QUERY_ONE_NOT_FOUND` (`404`).
- Runtime errors from adapter path: deterministic `DB.QUERY_ONE_FAILED` family mapping (existing classifier path).
- Preparation mismatch: deterministic preparse mismatch response.

## Example usage

No language-surface changes. Existing `db.queryOne(...)` handlers continue to behave identically for:

- `records.log`
- `sqlite`
- `postgres`

but now all are executed through the same client-layer queryOne entrypoint.

## Tradeoffs and next steps

- Tradeoff: `run_lasm_db_query_one_operation(...)` takes a wider argument set to support all adapters in one function.
- Benefit: adapter parity and future hardening are now centralized in one client operation path.
- Next step: continue reducing dispatch-owned adapter branching where operation logic can be centralized without changing intrinsic contracts.

