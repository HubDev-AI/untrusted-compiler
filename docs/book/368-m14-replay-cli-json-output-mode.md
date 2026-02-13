# M14 Slice: Replay CLI JSON Output Mode

This slice adds machine-readable replay compatibility output for CI and tooling.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`sec4 replay` was text-only on success. CI and automation need structured output so replay checks can be parsed deterministically without brittle text matching.

## What changed

1. Added replay output format flag
- `sec4 replay` now supports:
  - `--format text` (default)
  - `--format json`

2. Added JSON success payload
- JSON mode emits structured fields:
  - `ok`
  - `capture`
  - `stubs`
  - `effectsMode`
  - `policyHashMatched`
  - `compilerHashMatched`
  - `runtimeHashMatched`
  - `allowPolicyMismatch`
  - `warnings`

3. Added regression test coverage
- `replay_check_json_mode_writes_parseable_payload` verifies JSON payload parseability and key contract fields.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
```

## Tradeoffs

- Failures still report via stderr + non-zero exit code in this slice.
- Future slices can add JSON-structured failure payloads for full machine-readable parity.
