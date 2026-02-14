# 482 M18 Track-Convergence Summary

This chapter documents M18-S7: deterministic convergence reporting across the three executed post-handoff tracks.

## 1) What changed

- Added convergence summary builder:
  - `scripts/build-m18-track-convergence-summary.sh`
- Added convergence summary contract test:
  - `scripts/test-build-m18-track-convergence-summary.sh`

The summary consumes closure JSON and reports unified track status for:

- editor (`M18-D`)
- release (`M18-E`)
- runtime (`M18-F`)

## 2) Why it matters

After executing multiple selector-driven tracks, operators need one artifact showing whether the full M18 set converged. This avoids manually checking separate scripts/gates.

## 3) How it works

- Accepts:
  - `--closure-json <path>` (optional; otherwise runs live closure audit)
  - `--format markdown|json`
  - `--output <path>` for markdown mode
- Requires closure gates `M18-D`, `M18-E`, `M18-F` to exist in input.
- Produces:
  - per-track status table,
  - overall convergence (`PASS` or `PENDING`),
  - deterministic next-action message.

## 4) Verification

- `scripts/test-build-m18-track-convergence-summary.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending` now includes `M18-G`.

## 5) Tradeoff

This summary is closure-gate driven and intentionally high-level. It does not replace detailed per-track evidence; it provides deterministic aggregation for operator handoff and planning.
