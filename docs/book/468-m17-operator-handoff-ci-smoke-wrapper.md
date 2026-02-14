# 468 M17 Operator Handoff CI Smoke Wrapper

This chapter adds an explicit CI-safe wrapper for the M17 handoff quickstart flow.

## 1) What it is

A thin wrapper script:

- `scripts/run-m17-operator-handoff-ci-smoke.sh`

It runs:

- `scripts/run-m17-operator-handoff-quickstart.sh` with forced `--no-matrix`.

## 2) Why it exists

The quickstart flow is useful for both local and CI runs. CI jobs need deterministic, concise output and no extra troubleshooting-table printout, while preserving the same core handoff checks.

## 3) Behavior

- forwards repo/project/artifacts/timeout/max-body inputs to quickstart,
- forces `--no-matrix`,
- generates a temporary artifacts root when not supplied,
- fails fast if quickstart script is missing or non-executable.

## 4) Example usage

- `scripts/run-m17-operator-handoff-ci-smoke.sh`
- `scripts/run-m17-operator-handoff-ci-smoke.sh --serve-timeout-ms 9000 --max-body-bytes 4096`

Success output:

- `m17 operator handoff ci smoke passed`

## 5) Verification

- `scripts/test-run-m17-operator-handoff-ci-smoke.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
