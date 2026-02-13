# M14 Slice: Replay Redaction Header Baseline Enforcement

This slice hardens replay stub-contract validation by requiring a minimum redaction header baseline.

## What it is

Updated:
- `scripts/check-replay-stub-registry-contract.sh`
- `scripts/test-replay-stub-registry-contract.sh`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Replay captures and stubs are security-sensitive artifacts. A stub registry that omits critical redaction headers can leak auth material during capture/replay workflows.

## What changed

1. Enforced mandatory redaction headers in shell validator
- `check-replay-stub-registry-contract.sh` now requires `redaction.headers` to include:
  - `authorization`
  - `cookie`
  - `set-cookie`

2. Enforced same rule in CLI replay validator
- `validate_replay_stub_registry_contract` now enforces the same required header baseline with deterministic error messages.

3. Added regression coverage
- Shell contract tests now fail when redaction headers are incomplete.
- Replay CLI tests now fail when required redaction headers are missing from stub registries.

## Validation

```bash
scripts/test-replay-stub-registry-contract.sh
cargo test -p sec4 --test json_output replay_check
```

## Tradeoffs

- This is a strict baseline and may require explicit migration for legacy stub artifacts.
- The baseline is intentionally opinionated for security-first defaults in v0/v0.1.
