# 440 M16 Slice: Closure Gate for sec4 run Runtime-Flag CI Contracts

This chapter documents adding a strict closure gate for naming-lock enforcement of `sec4 run` runtime-flag CI contracts.

## 1) What it is

New closure gate:

- `M16-E`: naming-lock CI enforces run-command runtime-flag contract + guard tests.

Gate evidence source:

- `.github/workflows/naming-lock.yml`

Required workflow steps:

- `scripts/test-sec4-run-runtime-flag-contract.sh`
- `scripts/test-sec4-run-runtime-flag-contract-guard.sh`

## 2) Why it exists

`M16-S31` introduced run-flag CI contracts, but strict closure auditing did not yet reflect that guardrail. Adding `M16-E` makes closure status consistent with actual required CI protections.

## 3) How it works internally

1. `scripts/check-milestone-closure.sh` now computes `bool_has_m16_run_runtime_flag_ci_guard`.
2. Adds `emit_check "M16-E" ...` row in deterministic gate output.
3. `scripts/test-check-milestone-closure.sh` updated to:
   - include new run-flag contract steps in naming-lock fixture blocks,
   - expect `M16-E` in ordered gate list,
   - assert JSON output includes `M16-E`.
4. Roadmap strict closure table/interpreation updated accordingly.

## 4) Inputs, outputs, constraints

Inputs:

- naming-lock workflow file content
- closure gate expectations in test harness fixtures

Outputs:

- closure audit now emits `M16-E` PASS/PENDING status

Constraints:

- deterministic gate ordering remains part of JSON contract
- any workflow step rename/removal requires synchronized updates in closure script + closure tests + roadmap

## 5) Failure modes and diagnostics

- if either run-flag contract step is removed from naming-lock workflow:
  - `M16-E` becomes `PENDING`
  - strict mode (`--fail-on-pending`) exits non-zero

## 6) Example usage

```bash
scripts/check-milestone-closure.sh --format json --fail-on-pending
scripts/test-check-milestone-closure.sh
```

## 7) Trade-offs and next steps

Trade-offs:

- closure gate set grows and requires synchronized maintenance across script/tests/roadmap.

Next steps:

- keep closure gate additions atomic (gate logic + fixture updates + roadmap table in one slice) to avoid alignment drift.

## Verification

- `scripts/test-check-milestone-closure.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-naming-lock.sh`
