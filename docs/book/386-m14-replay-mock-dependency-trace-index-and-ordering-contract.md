# M14 Slice: Replay Mock Dependency Trace Index and Ordering Contract

This slice extends replay dependency traces with explicit numeric indices and locks deterministic ordering behavior for multi-entry captures.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

`mockDependencyTraces` already exposed `traceId` strings (for example `db:0`), but machine-side consumers still had to parse strings to assert ordering.

Adding an explicit numeric `index` field improves contract clarity and enables strict ordering checks without string parsing.

## How it works internally

1. While building replay trace entries, the capture-order loop index is preserved:
- DB traces: `index = 0..N-1`
- FS traces: `index = 0..N-1`

2. `traceId` is still emitted and remains consistent with index:
- `traceId = "db:<index>"` for DB
- `traceId = "fs:<index>"` for FS

3. JSON output now includes both:
- `index` (number)
- `traceId` (string)

## Inputs/outputs and constraints

Inputs:
- capture dependency arrays (`dependencies.db`, `dependencies.fs`) in canonical order

Outputs:
- `mockDependencyTraces.db[].index`
- `mockDependencyTraces.fs[].index`

Constraints:
- Indexing restarts per family (DB and FS each start at zero).
- Ordering is deterministic and tied to capture dependency order.

## Failure modes and diagnostics

No new failure modes were introduced.
Existing deterministic replay failures remain unchanged for missing/duplicate signatures.

## Example usage

With two DB and two FS dependencies in capture order, replay JSON includes:
- DB traces: indices `0`, `1` with `traceId` `db:0`, `db:1`
- FS traces: indices `0`, `1` with `traceId` `fs:0`, `fs:1`

## Tradeoffs and next steps

Tradeoffs:
- Small output surface growth (`index` per trace entry).
- Requires keeping `index` + `traceId` semantics aligned as contracts evolve.

Next steps:
- Add optional trace family totals/checksum fields if downstream CI consumers need compact order validation.
