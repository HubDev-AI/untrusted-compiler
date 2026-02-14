# M14 Slice: Replay Mock Dependency Stub Summary Output Contract

This slice extends replay mock-mode observability by exposing matched DB/FS stub response summaries for each dependency signature and locking the JSON contract.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/test-replay-cli-json-contract.sh`
- `scripts/test-replay-cli-json-contract-guard.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

`mockDependencyMatches` and `mockDependencySignatures` show how many dependencies matched and which signatures were matched, but they do not show what replay will return for those dependencies.

For deterministic debugging and operator visibility, replay output should include the matched stub response summaries too.

## How it works internally

1. Stub ingestion is upgraded from signature-only sets to typed maps:
- DB map key: `queryTemplateId|paramsSha256OrDash`
- FS map key: `lowercase(op)|pathSha256`

2. In mock mode, after signature presence checks pass:
- DB dependency summaries are built in capture order with:
  - `signature`, `rowCount`, `truncated`
- FS dependency summaries are built in capture order with:
  - `signature`, `ok`, `truncated`, `bytes`

3. Output rendering now emits:
- Text: `replay mock dependency stub summaries: db=... fs=...`
- JSON: `mockDependencyStubSummaries`

## Inputs/outputs and constraints

Inputs:
- replay capture with optional `dependencies.db/fs`
- replay stub registry with validated `stubs.db/fs` entries

Outputs:
- New JSON field:
  - `mockDependencyStubSummaries.db[]` entries with `signature`, `rowCount`, `truncated`
  - `mockDependencyStubSummaries.fs[]` entries with `signature`, `ok`, `truncated`, `bytes`

Constraints:
- Signature matching and summaries are deterministic and order-preserving by capture dependency order.
- Empty dependency sets render as `db=[]`, `fs=[]` in JSON and `db=- fs=-` in text.

## Failure modes and diagnostics

Existing deterministic failures remain unchanged:
- `REPLAY.DB_STUB_MISSING: <signature>`
- `REPLAY.FS_STUB_MISSING: <signature>`
- `REPLAY.CAPTURE_DB_DEPENDENCY_DUPLICATE: <signature>`
- `REPLAY.CAPTURE_FS_DEPENDENCY_DUPLICATE: <signature>`

This slice does not add new error codes; it adds observability after successful matching.

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

## Tradeoffs and next steps

Tradeoffs:
- Adds output surface area and contract-lock maintenance burden.
- Still summary-level; not full dependency execution transcript.

Next steps:
- Add optional per-dependency replay trace IDs/events for deeper deterministic debugging.
- Consider exposing dependency summary hashes for compact CI assertions.
