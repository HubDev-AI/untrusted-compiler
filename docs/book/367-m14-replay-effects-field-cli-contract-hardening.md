# M14 Slice: Replay Effects-Field CLI Contract Hardening

This slice extends the static CLI command contract so replay effects-mode wiring is CI-enforced.

## What it is

Updated:
- `scripts/test-sec4-cli-command-contract.sh`
- `scripts/test-sec4-cli-command-contract-guard.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/339-m12-sec4-cli-command-contract-test.md`

## Why it exists

After adding `--effects deny|mock|allow` to `sec4 replay`, command-surface drift could still remove the replay effects field without immediate CI signal. This hardening keeps replay policy controls part of the locked CLI contract.

## What changed

1. Contract test now requires replay effects field
- Static command contract asserts:
  - top-level `Replay { ... }` exists,
  - replay variant includes `effects: ReplayEffectsMode`.

2. Guard fixtures now cover replay effects drift
- Passing fixtures include replay effects field wiring.
- Added failing fixture where replay effects field is missing.

3. Docs/roadmap alignment
- Roadmap and command-contract chapter now state replay effects-mode wiring is part of command-surface enforcement.

## Validation

```bash
scripts/test-sec4-cli-command-contract.sh
scripts/test-sec4-cli-command-contract-guard.sh
```

## Tradeoffs

- Contract check remains static source-shape validation, not runtime command invocation.
- Static checks keep naming-lock CI fast and deterministic.
