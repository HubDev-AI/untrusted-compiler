# 415 M15 Slice: Replay Mock Execution Trace and Count Contract

This chapter documents M15-S1a: exposing deterministic mock execution evidence in `sec4 replay` output.

## What it is

A replay output contract extension for `sec4 replay --effects mock` that adds explicit execution evidence:

- `mockExecutionCounts` (net/db/fs operation counts)
- `mockExecutionTraces` (deterministic per-family executed trace entries)

The fields are emitted in JSON mode and mirrored in text mode summaries.

## Why it exists

Replay output previously focused on compatibility and signature matching. That validated contract shape, but did not expose an explicit execution summary suitable for runtime-stubbing evidence.

This slice makes replay results auditable as materialized mock execution, not only precheck state.

## Implementation details

1. Added replay execution models in CLI replay flow:
   - net execution trace (`net:0`) with signature + status/truncated/body kind,
   - db/fs execution traces derived from matched dependency ordering.
2. Added deterministic counters:
   - `net=1` for the matched capture request in mock mode,
   - db/fs counts equal matched dependency counts.
3. Emitted new fields in JSON payload:
   - `mockExecutionCounts`
   - `mockExecutionTraces`
4. Emitted text summary lines:
   - `replay mock executed stubs: ...`
   - `replay mock execution traces: ...`

## Validation

Updated CLI integration tests in `compiler/sec4-cli/tests/json_output.rs`:

- `replay_check_mock_mode_text_reports_matched_stub_response_summary`
- `replay_check_mock_mode_text_reports_dependency_stub_summaries`
- `replay_check_mock_mode_json_reports_dependency_match_counts`
- `replay_check_json_mode_writes_parseable_payload`

All assert deterministic execution counts/traces for both no-dependency and dependency-backed capture fixtures.

## Tradeoffs and next steps

- This slice improves observability and CI contractability for replay runtime stubbing.
- Remaining M15 work should add dedicated runtime replay guard scripts and closure gates so execution-contract drift is blocked in CI.
