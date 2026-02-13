# 199 M8 Slice: sec.audit History-Window Range Metadata

This chapter documents the addition of explicit range anchors for history-window summaries.

## What it is

Extended history-window summaries (stderr JSON and exported JSON) with range metadata:
- `oldestTimeMs`
- `latestTimeMs`
- `oldestPolicyHash`
- `latestPolicyHash`

## Why it exists

Window metrics are more actionable when they are anchored to concrete report boundaries. Without range metadata, downstream tooling cannot reliably tell which report interval a summary represents.

## How it works internally

1. During window computation, the CLI captures oldest/latest reports in the sampled set.
2. It extracts:
   - build timestamps from `report.build.time_ms`
   - policy hashes from `report.policy.hash`
3. It includes these anchors in:
   - stderr JSON summary payload
   - `--write-history-summary` artifact payload.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-cli/src/main.rs`
  - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - richer history summary payload with deterministic interval anchors.
- Constraint:
  - anchors reflect the sampled window after sorting and truncation to requested size.

## Failure modes and diagnostics

- If range metadata regresses, integration tests fail because exported summary JSON no longer contains oldest/latest timestamp/hash keys.

## Example usage

```bash
ailang sec audit \
  --path examples/hello \
  --format json \
  --history-dir .ailang/audit-history \
  --history-window 5 \
  --write-history-summary artifacts/audit/history-window.json
```

The written JSON now includes both policy-hash and time-range boundaries.

## Tradeoffs and next steps

- Tradeoff: range metadata currently lives in CLI summary payloads, not the core `AuditReport` structure.
- Next:
  - evaluate lifting these anchors into core report/trend schema once format compatibility window allows it.
