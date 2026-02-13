# M14 Slice: sec4 Replay CLI Command Contract Hardening

This slice extends the existing CLI command-surface contract so `sec4 replay` is treated as a locked top-level command, not an accidental implementation detail.

## What it is

Updated:
- `scripts/test-sec4-cli-command-contract.sh`
- `scripts/test-sec4-cli-command-contract-guard.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/339-m12-sec4-cli-command-contract-test.md`

## Why it exists

`sec4 replay` is now a first-class command. The previous contract test only guarded `audit|gate|explain`, so replay support could regress without CI catching it.

## What changed

1. CLI command contract now requires replay
- The static contract script now asserts a top-level `Replay { ... }` subcommand exists in `compiler/sec4-cli/src/main.rs`.

2. Guard regression coverage for replay drift
- The guard script fixtures were updated so the passing fixtures include `Replay`.
- Added a failing fixture that omits `Replay` and verifies the contract test fails deterministically.

3. Naming and roadmap alignment
- Roadmap command-surface references now include `sec4 replay` wherever canonical `sec4` commands are listed.
- M12 command-contract chapter now reflects replay as part of the locked command set.

## Validation

```bash
scripts/test-sec4-cli-command-contract.sh
scripts/test-sec4-cli-command-contract-guard.sh
```

## Tradeoffs

- This remains a static source-shape guard, not a runtime CLI invocation test.
- Static checks are intentionally preferred here to keep naming-lock CI fast and deterministic.
