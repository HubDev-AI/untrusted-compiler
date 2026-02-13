# M14 Slice: Replay CLI Compatibility Command

This slice adds an official `sec4 replay` command for deterministic replay-capture compatibility checks.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Replay contract checks existed as shell scripts, but replay validation should also be available through the primary `sec4` CLI surface.

This keeps replay compatibility checks toolchain-native and script-independent for operators and CI.

## What changed

1. Added `sec4 replay` command surface
- New command:
  - `sec4 replay --capture <path> --policy-hash <hash> --compiler-hash <hash> --runtime-hash <hash> [--allow-policy-mismatch]`

2. Added replay contract + compatibility validation in CLI
- CLI validates capture JSON contract shape (required identity/request/body/determinism/redaction fields).
- Compatibility behavior:
  - compiler/runtime hash mismatches always fail,
  - policy mismatch fails by default,
  - `--allow-policy-mismatch` allows policy mismatch with warning.

3. Added CLI integration tests
- `json_output.rs` now covers:
  - matching-hash success,
  - policy mismatch failure without allow,
  - policy mismatch allowed with warning,
  - compiler mismatch failure even with allow.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
```

## Tradeoffs

- Capture contract checks now exist in both shell and CLI layers.
- Duplicated validation logic is acceptable in this slice to keep the CLI self-contained; later refactor can move shared replay-validation primitives into `sec4-core`.
