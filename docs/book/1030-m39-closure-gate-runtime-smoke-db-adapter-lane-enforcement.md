# 1030 M39 Slice: Closure Gate Enforcement for Runtime-Smoke DB Adapter Lanes

## What It Is

This slice updates milestone closure gating so `M16-C` requires runtime-smoke workflow coverage for both LASM DB adapter lanes.

Updated files:

- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`

## Why It Exists

Runtime-smoke workflow now executes DB adapter lanes (`records-log`, `sqlite`), but closure checks still validated only the earlier hello-api dual-branch tokens.

Without closure-gate alignment, `M16-C` could pass even if DB adapter runtime-smoke steps drifted out of workflow.

## How It Works Internally

1. `M16-C` workflow token contract in closure checker now additionally requires:
   - `scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter records-log --artifacts-dir build/runtime-smoke/lasm-db-records-log`
   - `scripts/smoke-sec4-run-lasm-db-adapter.sh --db-adapter sqlite --artifacts-dir build/runtime-smoke/lasm-db-sqlite`

2. `M16-C` emitted check description is updated to reflect:
   - hello-api default/max-body lanes,
   - LASM DB adapter lanes,
   - bundle validation/index/upload flow.

3. Closure fixture coverage:
   - pass fixture runtime-smoke workflow in `scripts/test-check-milestone-closure.sh` now includes both DB adapter steps so the closure test remains deterministic.

## Inputs / Outputs and Constraints

Inputs:
- repository runtime-smoke workflow definition.

Outputs:
- `M16-C` now fails if either DB adapter runtime-smoke lane is missing.

Constraints:
- this slice keeps `M16` gate identity unchanged; only contract strictness and description were expanded.

## Failure Modes and Diagnostics

- when workflow is missing either DB adapter lane token, `M16-C` reports `PENDING` in closure output with runtime-smoke workflow path evidence.
- closure fixture tests fail if pass fixture omits newly required lane tokens.

## Example Usage

```bash
scripts/test-check-milestone-closure.sh
```

## Tradeoffs and Next Steps

Tradeoffs:
- closure checks become stricter, which may require fixture updates whenever runtime-smoke lane commands change.

Next steps:
1. mirror this stricter lane contract in any release-oriented workflow contract checks that reuse runtime-smoke assumptions,
2. consider dedicated closure gate note for adapter-specific artifact expectations if bundle semantics are expanded later.
