# M14 Slice: Replay CLI JSON Contract Closure Gate

This slice introduces naming-lock + closure-gate enforcement for replay JSON output contract stability.

## What it is

Added:
- `scripts/test-replay-cli-json-contract.sh`
- `scripts/test-replay-cli-json-contract-guard.sh`

Updated:
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

Replay JSON output and fields are now part of operational tooling. Without explicit CI closure gates, JSON contract drift could pass unnoticed and break downstream automation.

## What changed

1. Added replay JSON contract checker
- `scripts/test-replay-cli-json-contract.sh` enforces static source contract:
  - `ReplayOutputFormat` enum exists,
  - replay command includes `format: ReplayOutputFormat`,
  - default replay format remains text,
  - replay JSON payload keeps required keys (`policyHashMatched`, `compilerHashMatched`, `runtimeHashMatched`, `effectsMode`, `warnings`, `stubCounts`).

2. Added replay JSON contract guard script
- `scripts/test-replay-cli-json-contract-guard.sh` validates:
  - passing fixture with full contract,
  - failure on default-format drift,
  - failure on missing required JSON payload keys.

3. Wired CI + closure enforcement
- Naming-lock CI now runs replay JSON contract + guard tests.
- Closure audit now includes strict gate:
  - `M14-D` naming-lock CI enforces replay CLI JSON contract + guard tests.
- Closure fixture tests and deterministic gate ordering were updated for `M14-D`.

## Validation

```bash
scripts/test-replay-cli-json-contract.sh
scripts/test-replay-cli-json-contract-guard.sh
scripts/test-check-milestone-closure.sh
scripts/check-milestone-closure.sh --fail-on-pending
```

## Tradeoffs

- This gate is static source-shape enforcement, not runtime replay execution.
- Runtime replay semantics remain covered by CLI integration tests and future runtime slices.
