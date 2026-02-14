# M14 Slice: Replay Capture Dependency Signature Uniqueness Enforcement

This slice hardens replay capture contracts by rejecting duplicate DB/FS dependency request signatures, so dependency matching remains deterministic.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-capture-contract.sh`
- `scripts/test-replay-capture-contract.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Mock replay now matches captured DB/FS dependency signatures against stub registries. If the capture artifact contains duplicated dependency signatures, matching and count summaries can become ambiguous. Deterministic replay requires uniqueness at the capture boundary as well.

## What changed

1. CLI capture-contract uniqueness checks
- Replay capture validation now rejects duplicate DB dependency signatures:
  - `queryTemplateId|paramsSha256OrDash`
  - error: `REPLAY.CAPTURE_DB_DEPENDENCY_DUPLICATE: <signature>`
- Replay capture validation now rejects duplicate FS dependency signatures:
  - `lowercase(op)|pathSha256`
  - error: `REPLAY.CAPTURE_FS_DEPENDENCY_DUPLICATE: <signature>`

2. Shell contract parity
- `scripts/check-replay-capture-contract.sh` now enforces the same uniqueness rules for `dependencies.db` and `dependencies.fs`.

3. Regression coverage
- Added CLI integration tests for duplicate DB and FS capture dependency signatures in mock mode.
- Added shell contract guard fixtures that fail on duplicate capture dependency signatures.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
scripts/test-replay-capture-contract.sh
scripts/check-replay-capture-contract.sh
```

## Tradeoffs

- This enforces deterministic artifact shape, not dependency execution semantics.
- Signature uniqueness is based on request identity only; response/stub behavior remains governed by separate stub-contract rules.
