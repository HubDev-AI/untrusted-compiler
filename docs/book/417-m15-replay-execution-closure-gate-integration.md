# 417 M15 Slice: Replay Execution Closure Gate Integration

This chapter documents M15 closure-gate integration for replay execution contracts.

## What it is

A new strict closure gate in milestone auditing:

- `M15-A` — replay JSON contract scripts must enforce execution fields and missing-key guard cases.

The gate is implemented in `scripts/check-milestone-closure.sh` and validated in `scripts/test-check-milestone-closure.sh`.

## Why it exists

M15 execution evidence (`mockExecutionCounts`, `mockExecutionTraces`) was implemented and guard-tested, but closure audit did not track it.

Without a closure gate, milestone status could report green while replay execution contract coverage regressed.

## Implementation details

1. Extended closure checker state with `bool_has_replay_execution_contract_guard`.
2. Added detection rules:
   - `scripts/test-replay-cli-json-contract.sh` must include execution keys,
   - `scripts/test-replay-cli-json-contract-guard.sh` must include missing-key negative fixtures.
3. Added `emit_check` entry:
   - gate `M15-A` with deterministic evidence string.
4. Updated closure test fixture harness:
   - includes minimal replay contract/guard script files,
   - expected gate ordering now includes `M15-A`.
5. Updated roadmap strict closure table to include `M15-A`.

## Validation

Executed locally:

- `scripts/test-check-milestone-closure.sh`
- `scripts/test-roadmap-closure-gate-alignment.sh`
- `scripts/check-milestone-closure.sh --fail-on-pending`

Result: all pass, and closure report includes `M15-A: PASS`.

## Tradeoffs and next steps

- Closure auditing is now aligned with M15 replay execution contract enforcement.
- Next slice can start M16 follow-up work or deeper M15 runtime replay behavior (if needed) without losing contract-level regression protection.
