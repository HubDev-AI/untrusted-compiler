# M14 Slice: Replay DB/FS Stub Contract and Summary Ingestion

This slice extends replay stub handling beyond net signatures by validating DB/FS stub entry shapes and surfacing deterministic DB/FS ingestion summaries.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-stub-registry-contract.sh`
- `scripts/test-replay-stub-registry-contract.sh`
- `scripts/test-replay-cli-json-contract.sh`
- `scripts/test-replay-cli-json-contract-guard.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Replay previously treated `stubs.db` and `stubs.fs` as optional arrays without validating entry contracts or exposing ingestion details. That left mock replay observability incomplete and allowed malformed DB/FS stubs to pass too far.

## What changed

1. DB/FS stub contract validation
- CLI and shell contract checks now validate DB/FS entries when present.

DB entries require:
- `request.queryTemplateId` (non-empty string)
- optional `request.paramsSha256` (non-empty string)
- `response.rowCount` (integer >= 0)
- `response.truncated` (boolean)

FS entries require:
- `request.op` (non-empty string)
- `request.pathSha256` (non-empty string)
- `response.ok` (boolean)
- `response.truncated` (boolean)
- optional `response.bytes` (integer >= 0)

2. Replay output summary
- Replay now emits deterministic DB/FS summary under `stubDetails`:
  - `db.entries`
  - `db.uniqueQueryTemplateIds`
  - `fs.entries`
  - `fs.readOps`
  - `fs.writeOps`
  - `fs.otherOps`
- Text output includes a `replay stub details: ...` line.

3. Contract lock
- Replay JSON contract scripts now require `stubDetails` to prevent silent output drift.

4. Regression coverage
- Added CLI tests for invalid DB/FS stub shapes.
- Added mock JSON test for DB/FS summary aggregation.
- Added shell contract tests for valid and invalid DB/FS stubs.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
scripts/test-replay-stub-registry-contract.sh
scripts/test-replay-cli-json-contract.sh
scripts/test-replay-cli-json-contract-guard.sh
```

## Tradeoffs

- This slice validates and summarizes DB/FS stubs but does not execute DB/FS replay side effects yet.
- `stubDetails` is observability-oriented aggregate output; per-stub execution traces remain future work.
