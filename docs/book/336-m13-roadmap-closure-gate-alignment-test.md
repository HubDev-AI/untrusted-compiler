# M13 Slice: Roadmap Closure Gate Alignment Test

This slice adds a contract check that keeps roadmap closure table gates synchronized with the executable closure-audit script.

## What it is

Updated:
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `.github/workflows/naming-lock.yml`
- `docs/05-sec4-master-roadmap.md` (closure table rows normalized per gate)

## Why it exists

Closure gate IDs are enforced by `scripts/check-milestone-closure.sh`, but roadmap status is documented manually.

Without synchronization checks, docs can drift:
- missing new gates,
- stale removed gates,
- grouped rows that hide exact gate contract.

## What changed

1. Added alignment test
- Extracts gate IDs from `emit_check` calls in `check-milestone-closure.sh`.
- Extracts gate IDs from the roadmap “Current strict closure result” table.
- Fails on missing or stale gate IDs.

2. Wired to CI
- `naming-lock.yml` now runs:
  - `scripts/test-roadmap-closure-gate-alignment.sh`

3. Normalized roadmap closure table
- Expanded grouped M9 row into explicit per-gate rows (`M9-A` .. `M9-D`) to keep comparison deterministic.

## Validation

```bash
scripts/test-roadmap-closure-gate-alignment.sh
```

## Tradeoffs

- The check is strict and expects exact gate-ID parity.
- It deliberately pushes updates to script + roadmap together when closure gates change.
