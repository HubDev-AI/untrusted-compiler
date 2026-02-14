# M14 Slice: Replay Mock Stub JSON Contract Lock

This slice locks the replay JSON output contract for mock-specific fields so downstream automation cannot regress silently.

## What it is

Updated:
- `scripts/test-replay-cli-json-contract.sh`
- `scripts/test-replay-cli-json-contract-guard.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Replay JSON output now includes mock-specific observability (`mockRequestSignature`, `mockMatchedStub`). Without contract locking, these fields could drift or be removed while tests still pass on core keys.

## What changed

1. Contract script keys
- Replay JSON contract checker now requires:
  - `mockRequestSignature`
  - `mockMatchedStub`

2. Guard fixtures
- Passing guard fixtures now include both mock fields.
- Added a dedicated failing fixture missing `mockMatchedStub`.

3. Roadmap alignment
- M14 tracking and exit criteria now explicitly mention mock field contract enforcement.

## Validation

```bash
scripts/test-replay-cli-json-contract.sh
scripts/test-replay-cli-json-contract-guard.sh
```

## Tradeoffs

- This locks JSON field presence, not deeper semantic meaning of values.
- Value-shape semantics remain primarily enforced by CLI integration tests (`compiler/sec4-cli/tests/json_output.rs`).
