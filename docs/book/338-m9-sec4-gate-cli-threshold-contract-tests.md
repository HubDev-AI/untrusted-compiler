# M9 Slice: sec4 gate CLI Threshold Contract Tests

This slice adds direct CLI regression tests for `sec4 gate` threshold behavior and JSON output contract.

## What it is

Updated:
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`sec4 gate` is part of the locked command surface, but it had no direct tests proving:
- default threshold behavior (`risk>=HIGH`),
- custom threshold behavior with JSON mode.

Without this, regressions in gate semantics could ship unnoticed while `sec4 audit` tests still pass.

## What changed

1. Added default-threshold test
- `sec_gate_defaults_to_risk_high_threshold`
- creates a temp project with policy forcing a HIGH finding (`CSP_DISABLED`),
- verifies `sec4 gate` fails by default and reports HIGH-threshold failure.

2. Added custom-threshold JSON-mode test
- `sec_gate_json_mode_allows_high_when_threshold_is_critical`
- reuses HIGH-only project,
- verifies `sec4 gate --format json --fail-on risk>=CRITICAL` succeeds,
- verifies JSON payload is parseable and still contains `CSP_DISABLED`,
- verifies auxiliary `security map:` line stays on stderr in JSON mode.

## Validation

```bash
cargo test -q --manifest-path Cargo.toml -p sec4 --test json_output sec_gate_defaults_to_risk_high_threshold
cargo test -q --manifest-path Cargo.toml -p sec4 --test json_output sec_gate_json_mode_allows_high_when_threshold_is_critical
```

## Tradeoffs

- Adds focused CLI-level coverage rather than deep core-audit coverage.
- Keeps command-surface regressions detectable where users invoke `sec4 gate` directly.
