# M39: LASM DB records response module extraction

## What changed

Extracted `/db/records` response materialization from:

- `compiler/sec4-cli/src/main.rs`

into:

- `compiler/sec4-cli/src/lasm_db_records_response.rs`

The new module now owns:

- filter parsing/validation (`limit`, `op`, `db`, `tx`, `templateContains`),
- deterministic error envelope mapping for invalid filters,
- response summary shaping (`opCounts`, affected-row totals, cache/timeout telemetry),
- final record JSON emission.

## Why

This continues the M39 adapter-layer extraction track by removing DB-specific runtime/reporting logic from CLI orchestration.

The goal is cleaner module boundaries so DB runtime evolution can proceed without expanding `main.rs`.

## Behavior

No runtime semantics changed.

- Existing `/db/records` contracts remain stable.
- Existing deterministic diagnostics remain stable.
- Existing command integration tests continue to pass unchanged.
