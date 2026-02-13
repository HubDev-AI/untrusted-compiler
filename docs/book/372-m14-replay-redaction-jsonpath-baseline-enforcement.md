# M14 Slice: Replay Redaction JSONPath Baseline Enforcement

This slice extends replay redaction hardening by requiring a minimum JSONPath redaction set in stub registries.

## What it is

Updated:
- `scripts/check-replay-stub-registry-contract.sh`
- `scripts/test-replay-stub-registry-contract.sh`
- `captures/sample-replay-stubs.json`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Header redaction alone is insufficient for replay artifacts. Sensitive body fields must also be consistently redacted to avoid token/password leakage through captured or mocked payloads.

## What changed

1. Enforced required redaction JSONPath baseline
- Stub-contract validators now require:
  - `$.password`
  - `$.token`
  - `$.secret`
  - `$.apiKey`

2. Kept shell + CLI validator parity
- Shell checker and `sec4 replay` CLI validator enforce the same required JSONPath set.

3. Added regression tests and fixture updates
- Shell stub-contract tests include failure case for incomplete JSONPath redaction set.
- Replay CLI tests include failure case for missing required JSONPath redaction entries.
- Sample replay stubs fixture now includes the full required JSONPath baseline.

## Validation

```bash
scripts/test-replay-stub-registry-contract.sh
cargo test -p sec4 --test json_output replay_check
```

## Tradeoffs

- This is a strict baseline and may require upgrades for older stub artifacts.
- Strict defaults are intentional for security-first replay posture.
