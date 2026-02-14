# 430 M16 Slice: Runtime-Smoke Artifact Checker

This chapter documents artifact validation hardening for runtime smoke runs.

## What it is

A new checker script:

- `scripts/check-runtime-smoke-artifacts.sh`

It validates the artifact directory produced by `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir ...`.

## Why it exists

Runtime-smoke previously uploaded artifacts but did not verify their contract before upload. This checker turns artifact shape/content into an explicit enforceable contract.

## Validation contract

The checker requires:

- files exist and are non-empty:
  - `run-metadata.txt`
  - `health.headers`, `health.body`, `health.run.log`
  - `users.headers`, `users.body`, `users.run.log`
- `health.headers` contains `HTTP/1.1 200 OK`
- `users.headers` contains `HTTP/1.1 201 Created`
- `health.body` equals `ok`
- `users.body` matches standard success envelope contract (`ok`, `status:201`, `traceId`, `data`)
- `run-metadata.txt` includes numeric `port=`

## Workflow integration

`.github/workflows/runtime-smoke.yml` now runs:

1. smoke script,
2. artifact checker,
3. artifact upload (`if: always()`).

## Tests

Added deterministic checker test:

- `scripts/test-check-runtime-smoke-artifacts.sh`

It validates both pass fixture and missing-artifact failure behavior.

## Tradeoffs and next steps

- Contract is intentionally minimal and response-focused.
- Future hardening can include stricter metadata invariants (for example trace-id format or log-content shape) if needed.
