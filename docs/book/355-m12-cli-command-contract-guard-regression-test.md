# M12 Slice: CLI Command Contract Guard Regression Test

This slice adds fixture-based regression testing for `sec4` CLI command-surface contract checks.

## What it is

Updated:
- `scripts/test-sec4-cli-command-contract.sh`
- `scripts/test-sec4-cli-command-contract-guard.sh`
- `.github/workflows/naming-lock.yml`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/339-m12-sec4-cli-command-contract-test.md`

## Why it exists

The command contract checker previously ran only against the live CLI source file.

That catches source drift, but not regressions in checker behavior itself (for example accidentally allowing a weaker gate threshold or legacy alias patterns).

## What changed

1. Added source override support
- `test-sec4-cli-command-contract.sh` now supports:
  - `--cli <path>`

2. Added guard regression script
- New `test-sec4-cli-command-contract-guard.sh` validates:
  - passing fixture with expected command surface,
  - failure when gate default threshold changes,
  - failure when legacy `Sec` alias returns.

3. Wired guard into naming-lock CI
- `naming-lock.yml` now runs:
  - `scripts/test-sec4-cli-command-contract.sh`
  - `scripts/test-sec4-cli-command-contract-guard.sh`

## Validation

```bash
scripts/test-sec4-cli-command-contract.sh
scripts/test-sec4-cli-command-contract-guard.sh
scripts/check-naming-lock.sh
```

## Tradeoffs

- Adds one extra static CI test script.
- Improves confidence that command-contract enforcement logic itself remains strict over time.
