# 564 M30 Runtime De-stub Plan

This chapter documents M30-S3: deterministic runtime-first execution planning for db/fs/net/validator/secrets de-stubbing.

## 1) What changed

- Added runtime de-stub planner:
  - `scripts/plan-m30-runtime-destub.sh`
- Added planner contract test:
  - `scripts/test-plan-m30-runtime-destub.sh`
- Wired naming-lock + closure gate:
  - `.github/workflows/naming-lock.yml`
  - `scripts/check-milestone-closure.sh` (`M30-C`)
  - `scripts/test-check-milestone-closure.sh`

## 2) Why it matters

M30 is runtime-first by design, but kickoff/matrix artifacts still need to produce one deterministic execution order. This planner turns those artifacts into an auditable sequence of runtime de-stub slices.

## 3) How it works

- Validates kickoff + matrix contracts.
- Enters stabilization mode when closure/convergence is not PASS or pending gates remain.
- Emits deterministic five-step plan with explicit domain ordering:
  - `validators`, `net`, `db`, `fs`, `secrets` in stabilization mode
  - track-biased runtime ordering in non-stabilization mode.
- Emits `closureGate: M30-C` and deterministic `nextAction`.

## 4) Verification

- `scripts/test-plan-m30-runtime-destub.sh`
- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`

## 5) Tradeoff

Planner output is intentionally linear and fixed-size (five domains) to keep execution auditable. Parallel fan-out can be introduced later only after runtime evidence pipelines are mature.
