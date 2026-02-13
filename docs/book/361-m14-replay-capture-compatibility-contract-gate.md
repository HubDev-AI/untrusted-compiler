# M14 Slice: Replay Capture Compatibility Contract Gate

This slice adds a deterministic replay-compatibility checker and wires it into CI closure gating.

## What it is

Added/updated:
- `scripts/check-replay-capture-compat.sh`
- `scripts/test-replay-capture-compat.sh`
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

Replay capture shape validation (`M14-A`) is necessary but not sufficient.

For deterministic replay safety, captures also need compatibility checks against expected build identity hashes (policy/compiler/runtime), with explicit policy-mismatch override behavior.

## What changed

1. Added replay compatibility checker
- `scripts/check-replay-capture-compat.sh` validates capture compatibility against expected:
  - `--policy-hash`
  - `--compiler-hash`
  - `--runtime-hash`
- checker behavior:
  - compiler/runtime hash mismatches always fail,
  - policy hash mismatch fails by default,
  - `--allow-policy-mismatch` permits policy mismatch with warning.

2. Added replay compatibility guard tests
- `scripts/test-replay-capture-compat.sh` covers:
  - full-match pass,
  - policy mismatch failure by default,
  - policy mismatch allowed with override,
  - compiler/runtime mismatch failure even with policy override.

3. Wired CI + closure gate
- naming-lock now runs `scripts/test-replay-capture-compat.sh`.
- closure checker adds gate `M14-B`:
  - naming-lock CI enforces replay capture compatibility test.
- closure fixture tests now include a negative case for missing replay compatibility coverage.

## Validation

```bash
scripts/check-replay-capture-compat.sh \
  --capture captures/sample-capture.json \
  --policy-hash pol_9f4c3e1a \
  --compiler-hash cpl_0d2a7c11 \
  --runtime-hash rt_7b18e9aa
scripts/test-replay-capture-compat.sh
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
```

## Tradeoffs

- Adds one additional naming-lock contract test.
- Keeps replay policy-mismatch behavior explicit and deterministic before deeper runtime `mock` stubbing slices.
