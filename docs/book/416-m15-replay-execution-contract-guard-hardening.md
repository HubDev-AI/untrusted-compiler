# 416 M15 Slice: Replay Execution Contract Guard Hardening

This chapter documents M15-S1b: hardening replay contract scripts for execution-evidence fields.

## What it is

A script-level contract update for replay JSON output that now requires:

- `mockExecutionCounts`
- `mockExecutionTraces`

in addition to existing replay compatibility and dependency summary fields.

## Why it exists

M15-S1a introduced runtime execution evidence in replay output. Without guard enforcement, those keys could be removed or renamed silently.

This slice prevents drift by making the output schema test-enforced in CI.

## Implementation details

1. Updated `scripts/test-replay-cli-json-contract.sh` to require literal JSON keys for execution fields.
2. Updated `scripts/test-replay-cli-json-contract-guard.sh`:
   - positive fixture now includes execution fields,
   - added negative fixtures for missing execution keys.
3. Verified scripts pass with hardened contracts.

## Validation

Executed locally:

- `scripts/test-replay-cli-json-contract.sh`
- `scripts/test-replay-cli-json-contract-guard.sh`

Both passed after guard hardening.

## Tradeoffs and next steps

- This slice locks replay JSON schema evolution more tightly and reduces accidental compatibility regressions.
- Next M15 step should add explicit closure gates (`M15-*`) so milestone-closure auditing tracks replay runtime execution enforcement, not only contract scripts.
