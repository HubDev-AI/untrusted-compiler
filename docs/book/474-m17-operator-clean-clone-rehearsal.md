# 474 M17 Operator Clean-Clone Rehearsal

This chapter adds a deterministic clean-clone rehearsal runner for final operator handoff verification.

## 1) What it is

Script:

- `scripts/run-m17-operator-clean-clone-rehearsal.sh`

Contract test:

- `scripts/test-run-m17-operator-clean-clone-rehearsal.sh`

Primary artifact:

- `build/operator-clean-clone-rehearsal/rehearsal-report.json`

## 2) Why it exists

The final playbook chapter defines command bundles, but closeout still needs one repeatable rehearsal command that executes the sequence and captures friction evidence.

## 3) Inputs

- `--repo-root <path>`
- `--clone-root <path>` (optional; when omitted, a temporary clone root is created)
- `--output-dir <path>`
- `--skip-clone` (run in place for tests/local dry workflows)

## 4) Behavior

The runner executes six ordered steps and writes per-step logs:

1. playbook contract check,
2. quickstart flow,
3. CI smoke wrapper,
4. release packet build,
5. readiness summary snapshot,
6. strict closure snapshot.

If any step fails, the report captures:

- failed step id,
- first diagnostic line as friction evidence,
- PASS/FAIL status per step.

## 5) Outputs

- `rehearsal-report.json` with:
  - `overall`,
  - `cloneMode`,
  - `failedStep`,
  - `steps[]`,
  - `friction[]`.
- `logs/<step>.log` files for deterministic debugging.

## 6) Verification

- `scripts/test-run-m17-operator-clean-clone-rehearsal.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
