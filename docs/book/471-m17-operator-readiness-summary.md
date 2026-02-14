# 471 M17 Operator Readiness Summary

This chapter adds a consolidated readiness summary command for the M17 operator handoff track.

## 1) What it is

Script:

- `scripts/summarize-m17-operator-handoff-readiness.sh`

The script runs and summarizes:

1. handoff readiness verifier,
2. handoff CI smoke wrapper,
3. artifact inspector,
4. strict closure audit.

## 2) Why it exists

Operators and reviewers need one compact readiness output rather than manually combining multiple command results.

## 3) Output modes

- `--format text` (default) prints concise summary lines.
- `--format json` returns a machine-readable payload with:
  - step statuses,
  - inspector branch summary,
  - closure overall/pending counts.

## 4) Example usage

- `scripts/summarize-m17-operator-handoff-readiness.sh`
- `scripts/summarize-m17-operator-handoff-readiness.sh --format json`
- `scripts/summarize-m17-operator-handoff-readiness.sh --artifacts-root build/operator-handoff-smoke`

## 5) Verification

- `scripts/test-summarize-m17-operator-handoff-readiness.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
