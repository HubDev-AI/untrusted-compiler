# 443 M16 Slice: Smoke-Script Source/Work Metadata Token Contract Lock

This chapter documents hardening static smoke-script contract checks to lock provenance metadata token emission.

## 1) What it is

`scripts/test-smoke-sec4-run-hello-api-script-contract.sh` now requires these emitted metadata tokens:

- `sourceProject=${source_project}`
- `workProject=${work_project}`

`scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh` now includes deterministic negative cases for both token removals.

## 2) Why it exists

Runtime artifact checker already enforces `sourceProject` and `workProject` fields, but static smoke-script contract checks did not pin the metadata emit block itself. This slice closes that gap and catches script drift earlier.

## 3) How it works internally

1. Add source/work provenance tokens to the smoke-script required-token array.
2. Guard harness mutates a copied fixture script:
   - replaces `sourceProject=${source_project}`
   - replaces `workProject=${work_project}`
3. For each mutation, contract checker must fail and emit deterministic missing-token diagnostics.
4. Existing `runFlags` metadata-guard path is preserved as an additional scenario.

## 4) Inputs, outputs, constraints

Inputs:

- smoke-script source file
- static contract token list
- guard fixture mutations

Outputs:

- deterministic pass/fail contract results for source/work metadata token presence

Constraints:

- token literals are intentionally exact; script text changes require synchronized contract updates
- guard diagnostics are message-contract locked

## 5) Failure modes and diagnostics

- removing `sourceProject` emit token causes:
  - `missing required smoke-script token: sourceProject=${source_project}`
- removing `workProject` emit token causes:
  - `missing required smoke-script token: workProject=${work_project}`

## 6) Example usage

```bash
scripts/test-smoke-sec4-run-hello-api-script-contract.sh
scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh
```

## 7) Trade-offs and next steps

Trade-offs:

- strict literal-token checks require careful synchronization when smoke-script metadata layout evolves.

Next steps:

- add companion contract checks for metadata key ordering if deterministic artifact diffs become an operator requirement.

## Verification

- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
