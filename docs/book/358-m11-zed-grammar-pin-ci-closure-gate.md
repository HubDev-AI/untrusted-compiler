# M11 Slice: Zed Grammar Pin CI Closure Gate

This slice promotes Zed grammar pin validation from a local script into CI-enforced closure gating.

## What it is

Updated:
- `scripts/check-zed-grammar-pin.sh`
- `scripts/test-zed-grammar-pin.sh`
- `scripts/test-zed-grammar-pin-guard.sh`
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

The repository already had `check-zed-grammar-pin.sh`, but closure and naming-lock enforcement did not require that contract to run.

Without CI/closure wiring, `extension.toml` grammar pin drift could bypass milestone status while still breaking deterministic editor packaging guarantees.

## What changed

1. Extended grammar-pin checker for fixture-driven tests
- `check-zed-grammar-pin.sh` now supports:
  - `--extension <path>`
- revision validation now requires a full 40-character commit SHA.

2. Added contract and guard regression tests
- New `test-zed-grammar-pin.sh`:
  - runs the checker,
  - enforces grammar repository pin to `https://github.com/HubDev-AI/untrusted-compiler`.
- New `test-zed-grammar-pin-guard.sh` validates failures for:
  - placeholder revision,
  - non-SHA revision,
  - repository drift.

3. Wired naming-lock CI enforcement
- `.github/workflows/naming-lock.yml` now runs:
  - `scripts/test-zed-grammar-pin.sh`
  - `scripts/test-zed-grammar-pin-guard.sh`

4. Added strict closure gate `M11-A`
- `check-milestone-closure.sh` now enforces:
  - naming-lock includes both zed grammar pin contract and guard tests.
- `test-check-milestone-closure.sh` now includes:
  - passing fixtures with zed pin checks,
  - negative fixture for missing zed pin guard step.

5. Synced roadmap closure table
- `docs/05-sec4-master-roadmap.md` strict closure table now includes `M11-A`.

## Validation

```bash
scripts/check-zed-grammar-pin.sh
scripts/test-zed-grammar-pin.sh
scripts/test-zed-grammar-pin-guard.sh
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/check-naming-lock.sh
```

## Tradeoffs

- Adds two static CI checks in naming-lock.
- Strengthens deterministic packaging guarantees for Zed integration and keeps closure status aligned with actual editor-pin enforcement.
