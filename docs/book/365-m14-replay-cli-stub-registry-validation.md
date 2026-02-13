# M14 Slice: Replay CLI Stub Registry Validation

This slice extends the `sec4 replay` command so replay compatibility checks can validate a stub-registry contract directly from CLI.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

Replay capture compatibility was already validated, but stub contract checks were script-only. Adding optional stub validation to `sec4 replay` keeps replay checks toolchain-native and CI/ops friendly.

## What changed

1. Extended `sec4 replay` command surface
- New optional flag:
  - `--stubs <path>`

2. Added CLI-side stub contract validation
- When `--stubs` is provided, CLI now validates:
  - registry version (`0.1`),
  - `stubs.net` shape,
  - response payload contract (`bodyBase64` or `bodySha256`),
  - redaction metadata shape,
  - deterministic uniqueness of net request signatures.

3. Added regression tests
- Replay CLI tests now cover:
  - valid stub registry pass,
  - invalid stub contract fail,
  - duplicate request-signature fail.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
```

## Tradeoffs

- Contract logic now exists in both shell checker and CLI path.
- Duplication is acceptable in this slice to keep `sec4 replay` self-contained; later refactor can centralize contract helpers in `sec4-core`.
