# 467 M17 Operator Handoff Quickstart

This chapter adds the M17-S4 orchestrator command for running handoff validation as one deterministic flow.

## 1) What it is

A single script that runs the handoff pipeline in strict order:

1. M17 readiness verifier,
2. M17 bootstrap profile helper,
3. strict closure gate check,
4. troubleshooting matrix print.

Script:

- `scripts/run-m17-operator-handoff-quickstart.sh`

## 2) Why it exists

Operators no longer need to manually coordinate multiple scripts for local go/no-go checks. The quickstart command ensures the same ordered sequence runs every time.

## 3) Command flow

Default run:

- `scripts/run-m17-operator-handoff-quickstart.sh`

With overrides:

- `scripts/run-m17-operator-handoff-quickstart.sh --serve-timeout-ms 9000 --max-body-bytes 4096`

Skip troubleshooting matrix print:

- `scripts/run-m17-operator-handoff-quickstart.sh --no-matrix`

## 4) Failure behavior

The script fails fast on the first failing stage and propagates diagnostics from that stage unchanged, for example:

- readiness verifier failure,
- bootstrap profile failure,
- strict closure gate pending failures.

## 5) Verification

- `scripts/test-run-m17-operator-handoff-quickstart.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
