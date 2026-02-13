# M14 Slice: Replay Stub Inventory Output

This slice adds deterministic stub inventory reporting to replay command output.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Replay mock-mode checks should make loaded stub scope visible. Without explicit counts, operators cannot quickly verify which stub sets were actually applied.

## What changed

1. Added stub inventory tracking in replay flow
- When `--stubs` is provided, replay now counts:
  - `stubs.net`
  - `stubs.db`
  - `stubs.fs`

2. Exposed counts in output formats
- Text mode appends:
  - `replay stubs loaded: net=<n> db=<n> fs=<n>`
- JSON mode now includes:
  - `stubCounts: { net, db, fs }`

3. Added regression assertions
- Replay text-mode stub test now asserts inventory summary line.
- Replay JSON-mode test now asserts `stubCounts` values.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
```

## Tradeoffs

- Counts are summary-level observability only; they do not guarantee per-call stub hit coverage.
- Per-request stub hit/miss tracing can be added later if needed.
