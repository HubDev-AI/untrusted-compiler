# M14 Slice: Replay Mock Dependency Trace Output Contract

This slice adds deterministic per-dependency replay trace entries for matched DB/FS dependencies and locks the replay JSON contract.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/test-replay-cli-json-contract.sh`
- `scripts/test-replay-cli-json-contract-guard.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

`mockDependencySignatures` and `mockDependencyStubSummaries` provide useful replay visibility, but they do not give a canonical trace identifier per dependency entry.

When debugging deterministic replay runs, a stable `traceId` per dependency makes it easier to correlate output, logs, and future replay diagnostics.

## How it works internally

1. During mock dependency matching, replay now records trace entries in capture order:
- DB trace IDs: `db:0`, `db:1`, ...
- FS trace IDs: `fs:0`, `fs:1`, ...

2. Each trace entry includes both identity and matched summary data:
- DB trace fields:
  - `traceId`, `signature`, `rowCount`, `truncated`
- FS trace fields:
  - `traceId`, `signature`, `ok`, `truncated`, `bytes`

3. Output surfaces:
- JSON field: `mockDependencyTraces`
- Text line: `replay mock dependency traces: db=... fs=...`

## Inputs/outputs and constraints

Inputs:
- replay capture dependencies (`capture.dependencies.db/fs`)
- replay stub registry (`stubs.db/fs`)

Outputs:
- `mockDependencyTraces.db[]`
- `mockDependencyTraces.fs[]`

Constraints:
- Trace sequencing is deterministic and tied to capture dependency order.
- Empty dependency sets produce empty JSON arrays and `db=- fs=-` in text output.

## Failure modes and diagnostics

No new failure codes were introduced in this slice.
Existing deterministic failures remain authoritative:
- `REPLAY.DB_STUB_MISSING`
- `REPLAY.FS_STUB_MISSING`
- `REPLAY.CAPTURE_DB_DEPENDENCY_DUPLICATE`
- `REPLAY.CAPTURE_FS_DEPENDENCY_DUPLICATE`

## Example usage

```bash
sec4 replay \
  --capture captures/sample-capture.json \
  --stubs captures/sample-replay-stubs.json \
  --effects mock \
  --format json \
  --policy-hash pol_abc \
  --compiler-hash cpl_abc \
  --runtime-hash rt_abc
```

The JSON payload now includes:
- `mockDependencyMatches`
- `mockDependencySignatures`
- `mockDependencyStubSummaries`
- `mockDependencyTraces`

## Tradeoffs and next steps

Tradeoffs:
- Increases replay output size and contract surface area.
- Trace IDs are local to a replay run and not globally unique.

Next steps:
- Add optional trace-level replay event timestamps or ordering metadata if multi-stage dependency replay output is introduced.
- Allow downstream tooling to filter replay output by trace ID.
