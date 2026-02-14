# 458 M16 Slice: Runtime-Smoke Shape-Aware runFlags Metadata Contract Hardening

This chapter documents hardening runtime-smoke metadata so `runFlags` reflects whether `--max-body-bytes` was actually used.

## 1) What it is

`scripts/smoke-sec4-run-hello-api.sh` now computes `runFlags` dynamically:

- baseline: `--port,--oneshot,--serve-timeout-ms`
- plus `,--max-body-bytes` only when `--max-body-bytes <N>` is passed.

`scripts/check-runtime-smoke-artifacts.sh` now validates this branch-aware shape against `maxBodyBytes` metadata.

## 2) Why it exists

Before this slice, `runFlags` was fixed even when optional max-body wiring was active. That allowed metadata drift between what the smoke run used and what metadata claimed.

## 3) How it works internally

1. Smoke script builds `run_flags` before writing `run-metadata.txt`.
2. Checker parses `maxBodyBytes` and computes expected `runFlags`.
3. Checker fails if the field is missing or does not match branch-specific expected shape.
4. Regression fixtures cover both branch-drift directions.

## 4) Inputs, outputs, constraints

Inputs:

- `run-metadata.txt` (`maxBodyBytes`, `runFlags`)
- run logs (`health.run.log`, `users.run.log`)

Outputs:

- deterministic contract validation pass/fail

Constraints:

- `runFlags` order is fixed and deterministic.
- branch shape is tied to `maxBodyBytes=unset|<positive int>`.

## 5) Failure modes and diagnostics

- Missing `runFlags`:
  - `run-metadata.txt missing deterministic runFlags field`
- Wrong branch shape:
  - `run-metadata.txt runFlags field does not match expected runtime flag shape`

## 6) Example usage

```bash
scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048 --artifacts-dir /tmp/sec4-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- contract strictness increases fixture maintenance when metadata fields evolve.

Next steps:

- execute both `maxBodyBytes=unset` and `maxBodyBytes=<N>` branches in runtime-smoke CI to keep branch coverage always live.

## Verification

- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`
- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh`
- `scripts/smoke-sec4-run-hello-api.sh --max-body-bytes 2048`
