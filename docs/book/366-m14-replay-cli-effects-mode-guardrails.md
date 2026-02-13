# M14 Slice: Replay CLI Effects-Mode Guardrails

This slice adds deterministic replay mode-policy checks to `sec4 replay` for `deny|mock|allow` effect handling.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Replay policy spec defines `replay.effects = deny|mock|allow`. Without explicit mode handling in CLI, replay behavior could drift or hide risky execution posture.

## What changed

1. Added replay effects mode flag
- `sec4 replay` now accepts:
  - `--effects deny|mock|allow`
- Default remains `deny`.

2. Added deterministic mode checks
- `mock` mode now requires `--stubs <path>` and fails fast when missing.
- `allow` mode now emits explicit warning to stderr.
- Existing capture/stub contract checks still run as before.

3. Added regression tests
- replay check fails in `mock` mode without stubs.
- replay check emits warning in `allow` mode.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
```

## Tradeoffs

- This slice enforces mode-policy guards but does not execute real replay IO.
- Runtime effect withholding and full `mock` execution remain later M14 runtime slices.
