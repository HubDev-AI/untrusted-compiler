# M14 Slice: Replay Output-Field CLI Contract Hardening

This slice extends static CLI contract guards so replay output-format wiring is locked in CI.

## What it is

Updated:
- `scripts/test-sec4-cli-command-contract.sh`
- `scripts/test-sec4-cli-command-contract-guard.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/339-m12-sec4-cli-command-contract-test.md`

## Why it exists

`sec4 replay` now supports `--format json`. Without contract guard coverage, replay output-mode wiring could regress silently while command names still appear intact.

## What changed

1. Contract test now requires replay output-format field
- Static command contract asserts:
  - replay variant includes `format: ReplayOutputFormat`.

2. Guard fixtures now cover output-field drift
- Passing fixtures include replay `effects` and `format` fields.
- Added failing fixture where replay `format` field is missing.

3. Docs alignment
- Roadmap and CLI contract chapter now document replay output-field enforcement as part of command-surface lock.

## Validation

```bash
scripts/test-sec4-cli-command-contract.sh
scripts/test-sec4-cli-command-contract-guard.sh
```

## Tradeoffs

- Static source-shape checks remain lightweight and deterministic.
- Runtime replay behavior is still covered separately by CLI integration tests.
