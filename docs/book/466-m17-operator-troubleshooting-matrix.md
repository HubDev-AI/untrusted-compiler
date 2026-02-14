# 466 M17 Operator Troubleshooting Matrix

This chapter adds a deterministic troubleshooting matrix for the M17 operator handoff flow.

## 1) What it is

A script that prints stable troubleshooting guidance for the runtime-smoke + handoff verification pipeline:

- `scripts/print-m17-operator-troubleshooting-matrix.sh`

Output modes:

- text table (default),
- JSON contract (`--format json`).

## 2) Why it exists

M17-S1 and M17-S2 made the flow executable, but operators still needed a predictable way to map recurring diagnostics to the exact command that should be rerun. This matrix removes ambiguity during incident triage.

## 3) Matrix coverage

The matrix currently maps deterministic diagnostics such as:

- `missing runtime-smoke artifact file:`
- `run-metadata.txt runFlags field does not match expected runtime flag shape`
- `health.headers missing X-Trace-Id header`
- `users traceId mismatch between headers and body`
- `missing handoff chapter token:`
- `runtime-smoke bundle missing branch directory:`
- `overall: PENDING`

Each entry includes:

- stable `id`,
- diagnostic token,
- failing stage,
- primary command to run,
- remediation guidance.

## 4) Example usage

- `scripts/print-m17-operator-troubleshooting-matrix.sh`
- `scripts/print-m17-operator-troubleshooting-matrix.sh --format json`

## 5) Failure modes

- Unsupported format:
  - `unknown format: ...`
- Missing `jq` in JSON mode:
  - `missing required command: jq`

## 6) Verification

- `scripts/test-print-m17-operator-troubleshooting-matrix.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
