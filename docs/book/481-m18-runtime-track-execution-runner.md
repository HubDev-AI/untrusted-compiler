# 481 M18 Runtime-Track Execution Runner

This chapter documents M18-S6: runtime-track execution from selector output with deterministic command contracts.

## 1) What changed

- Added runtime-track runner script:
  - `scripts/run-m18-runtime-track.sh`
- Added runtime-track runner contract test:
  - `scripts/test-run-m18-runtime-track.sh`

The runner consumes selector JSON and executes (or dry-runs) the runtime confidence commands.

## 2) Why it matters

M18 prioritization now has executable follow-through for runtime work, not only planning artifacts. The runtime track is now machine-invokable from selector output and closure-gated in naming-lock CI.

## 3) How it works

- Validates selector contract (`selectedTrack`, `recommendation.id`).
- Requires:
  - `selectedTrack == runtime`
  - `recommendation.id` matches `M18-S6-runtime-*`
- Defines runtime-track command set:
  - `scripts/check-runtime-smoke-bundle.sh ...`
  - `scripts/check-milestone-closure.sh --fail-on-pending`
- Supports:
  - `--dry-run`
  - `--format text|json`

## 4) Verification

- `scripts/test-run-m18-runtime-track.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending` (includes `M18-F` gate)

## 5) Tradeoff

The runner currently executes a fixed two-command contract to stay deterministic. Future slices can extend command composition once runtime-track outcomes require broader checks.
