# 441 M16 Slice: Runtime-Smoke Source/Work Metadata Field Checks

This chapter documents hardening the runtime-smoke artifact checker with explicit source/work project provenance requirements.

## 1) What it is

`scripts/check-runtime-smoke-artifacts.sh` now requires two additional metadata keys in `run-metadata.txt`:

- `sourceProject=...`
- `workProject=...`

Regression coverage in `scripts/test-check-runtime-smoke-artifacts.sh` now includes a dedicated missing-`sourceProject` failure fixture.

## 2) Why it exists

Runtime-smoke artifacts already locked runtime flags and status envelopes, but they did not enforce provenance fields. Requiring source/work paths makes artifact context explicit and prevents silent metadata drift in smoke evidence.

## 3) How it works internally

1. Checker adds two deterministic regex checks:
   - `^sourceProject=.+$`
   - `^workProject=.+$`
2. On absence, checker emits stable diagnostics:
   - `run-metadata.txt missing sourceProject field`
   - `run-metadata.txt missing workProject field`
3. Test harness creates `bad-source` fixture with `sourceProject` removed and asserts the exact failure message.
4. End-to-end smoke flow remains validated by running smoke script and checker against real generated artifacts.

## 4) Inputs, outputs, constraints

Inputs:

- runtime-smoke artifact directory with `run-metadata.txt`

Outputs:

- pass/fail status from checker with deterministic diagnostics

Constraints:

- metadata keys must remain non-empty
- key names are contract-locked (`sourceProject`, `workProject`)

## 5) Failure modes and diagnostics

- missing `sourceProject`:
  - checker exits non-zero with `run-metadata.txt missing sourceProject field`
- missing `workProject`:
  - checker exits non-zero with `run-metadata.txt missing workProject field`

## 6) Example usage

```bash
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir /tmp/sec4-runtime-smoke
scripts/test-check-runtime-smoke-artifacts.sh
```

## 7) Trade-offs and next steps

Trade-offs:

- stricter metadata contract means smoke-script metadata edits must be synchronized with checker/tests.

Next steps:

- add complementary missing-`workProject` regression fixture for symmetric negative coverage.

## Verification

- `scripts/test-check-runtime-smoke-artifacts.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
