# 465 M17 Operator Bootstrap Profile Helper

This chapter adds the first executable M17-S2 helper that runs the operator bootstrap profile in one command.

## 1) What it is

A wrapper script that executes the deterministic three-step operator profile:

1. default runtime-smoke branch,
2. max-body runtime-smoke branch,
3. runtime-smoke bundle check + branch-index generation.

Script:

- `scripts/run-m17-operator-bootstrap.sh`

## 2) Why it exists

M17-S1 documented the checklist and readiness verifier, but operators still had to run each profile step manually. This helper reduces drift by locking the branch flow into one command.

## 3) How it works

`scripts/run-m17-operator-bootstrap.sh`:

- validates required scripts and numeric inputs,
- runs:
  - `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <artifacts-root>/default`
  - `scripts/smoke-sec4-run-hello-api.sh --max-body-bytes <N> --artifacts-dir <artifacts-root>/max-body`
- validates bundle:
  - `scripts/check-runtime-smoke-bundle.sh --artifacts-root <artifacts-root> --index-path <artifacts-root>/runtime-smoke-branch-index.json`

Default values:

- `--artifacts-root`: `build/runtime-smoke`
- `--serve-timeout-ms`: `12000`
- `--max-body-bytes`: `2048`

## 4) Inputs and constraints

- `--project` must point to an existing project directory.
- `--serve-timeout-ms` must be a positive integer.
- `--max-body-bytes` must be a positive integer.
- required scripts must exist and be executable:
  - `scripts/smoke-sec4-run-hello-api.sh`
  - `scripts/check-runtime-smoke-bundle.sh`

## 5) Failure modes and diagnostics

- Missing required script:
  - `missing runtime smoke bundle checker: ...`
- Non-executable script:
  - `runtime smoke script is not executable: ...`
- Invalid numeric args:
  - `invalid --serve-timeout-ms value ...`
  - `invalid --max-body-bytes value ...`

## 6) Example usage

- `scripts/run-m17-operator-bootstrap.sh`
- `scripts/run-m17-operator-bootstrap.sh --serve-timeout-ms 9000 --max-body-bytes 4096`
- `scripts/run-m17-operator-bootstrap.sh --artifacts-root build/runtime-smoke-nightly`

Success output:

- `m17 operator bootstrap profile passed`

## 7) Verification

- `scripts/test-run-m17-operator-bootstrap.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`
