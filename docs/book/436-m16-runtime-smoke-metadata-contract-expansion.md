# 436 M16 Slice: Runtime-Smoke Metadata Contract Expansion

This chapter documents expanding runtime-smoke artifact metadata with deterministic run-profile fields and enforcing those fields in the artifact checker.

## 1) What it is

`run-metadata.txt` produced by `scripts/smoke-sec4-run-hello-api.sh` now includes:

- `oneshot=true`
- `serveTimeoutMs=12000`
- `runFlags=--port,--oneshot,--serve-timeout-ms`

`scripts/check-runtime-smoke-artifacts.sh` now requires those fields.

## 2) Why it exists

The previous artifact contract guaranteed request/response files and numeric port. It did not explicitly capture runtime launch profile, making artifact provenance weaker when investigating drift between runs.

Deterministic run-profile metadata improves debuggability and keeps smoke artifact semantics explicit.

## 3) How it works internally

1. Smoke script writes new fixed metadata keys during setup.
2. Artifact checker validates:
   - `oneshot=true`
   - numeric `serveTimeoutMs`
   - deterministic `runFlags` token list
3. Checker regression script now includes a negative fixture missing `oneshot=true` and asserts deterministic failure message.

## 4) Inputs, outputs, constraints

Inputs:

- smoke script execution with optional `--artifacts-dir`

Outputs:

- metadata file with run-profile fields
- checker pass/fail based on expanded metadata contract

Constraints:

- metadata values are deterministic by design for contract stability.
- timeout metadata currently assumes smoke default (`12000`).

## 5) Failure modes and diagnostics

New checker diagnostics:

- `run-metadata.txt missing oneshot=true field`
- `run-metadata.txt missing numeric serveTimeoutMs field`
- `run-metadata.txt missing deterministic runFlags field`

## 6) Example usage

```bash
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir build/runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- metadata checker is now stricter and may require coordinated updates if smoke launch profile changes.

Next steps:

- if smoke profile becomes configurable, evolve metadata/checker to support explicit profile variants while preserving deterministic defaults.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
- `scripts/test-runtime-smoke-workflow-contract.sh`
- `scripts/test-runtime-smoke-workflow-contract-guard.sh`
