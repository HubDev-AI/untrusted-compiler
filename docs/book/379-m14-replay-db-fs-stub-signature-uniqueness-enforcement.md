# M14 Slice: Replay DB/FS Stub Signature Uniqueness Enforcement

This slice enforces deterministic uniqueness for DB and FS stub request signatures, aligning them with existing net-stub uniqueness rules.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-stub-registry-contract.sh`
- `scripts/test-replay-stub-registry-contract.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Net stubs already rejected duplicate request signatures, but DB/FS stubs did not. That allowed ambiguous replay artifacts where multiple entries could match the same DB/FS request identity.

## What changed

1. CLI validator uniqueness
- `stubs.db` now rejects duplicate signatures of:
  - `queryTemplateId|paramsSha256OrDash`
- `stubs.fs` now rejects duplicate signatures of:
  - `lowercase(op)|pathSha256`

2. Shell contract parity
- `scripts/check-replay-stub-registry-contract.sh` now enforces the same DB/FS uniqueness rules via `jq`.
- Shell contract now also requires integer `stubs.net[].response.status` values (no fractional status codes), matching CLI validator strictness.

3. Regression coverage
- Added CLI replay tests for duplicate DB and duplicate FS signature failures.
- Added shell contract guard fixtures for duplicate DB and FS signatures.
- Added shell contract guard fixture for non-integer net response status rejection.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
scripts/test-replay-stub-registry-contract.sh
```

## Tradeoffs

- Signature uniqueness validates registry determinism only; it does not yet execute DB/FS replay side effects.
- DB signatures currently key on template id and optional params hash, so callers must maintain stable template identifiers.
