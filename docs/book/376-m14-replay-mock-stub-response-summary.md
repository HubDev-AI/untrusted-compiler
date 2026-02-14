# M14 Slice: Replay Mock Stub Response Summary

This slice extends `sec4 replay --effects mock` to expose deterministic details of the matched net stub response.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

After adding deterministic mock signature matching, operators could tell that a stub existed but not which response shape was selected. A small, stable matched-response summary improves replay observability without introducing full runtime side-effect execution yet.

## What changed

1. Matched net-stub ingestion
- Replay now parses and retains matched net-stub response fields in mock mode:
  - `status`
  - `truncated`
  - `bodyKind` (`base64` or `sha256`)

2. Deterministic output surface
- Text output now includes:
  - `replay mock stub response: status=<n> truncated=<bool> bodyKind=<kind>`
- JSON output now includes:
  - `mockMatchedStub: { status, truncated, bodyKind }`

3. Regression coverage
- Added mock text-mode test asserting matched signature and response summary lines.
- Extended JSON-mode replay test to assert `mockMatchedStub` fields.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
scripts/test-replay-cli-json-contract.sh
scripts/test-replay-cli-json-contract-guard.sh
```

## Tradeoffs

- This is observability-only ingestion for net stubs; replay does not yet execute DB/FS side effects from stubs.
- When both `bodyBase64` and `bodySha256` are present, summary reports `bodyKind=base64` by precedence for deterministic output.
