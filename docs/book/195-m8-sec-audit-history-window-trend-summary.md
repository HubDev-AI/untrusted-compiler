# 195 M8 Slice: sec.audit History-Window Trend Summary

This chapter documents the M8 CLI slice that extends audit trend visibility from a single baseline to configurable multi-run windows.

## What it is

Added `--history-window <N>` to `ailang sec audit` (requires `--history-dir`) and emit a history-window trend summary over the most recent `N` persisted audit reports.

## Why it exists

`sec.audit` already supported latest-baseline trend deltas, but operators could not quickly inspect how risk scores moved across several recent runs. This slice adds a bounded, deterministic multi-run summary without changing stdout JSON report shape.

## How it works internally

1. CLI command surface:
   - new flag: `--history-window <usize>`
   - validation: flag requires `--history-dir`.
2. History loading:
   - reads JSON reports from history directory
   - sorts deterministically
   - loads the most recent `N` reports.
3. Summary computation:
   - report count
   - oldest/latest risk score
   - min/max risk score across window
   - risk-score delta (`latest - oldest`).
4. Output:
   - text mode: human-readable summary line
   - JSON mode: summary emitted on stderr as a compact JSON payload line, preserving stdout as parseable audit report JSON.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-cli/src/main.rs`
  - `compiler/ailang-cli/tests/json_output.rs`
- Output:
  - optional history-window summary in audit command auxiliary output.
- Constraints:
  - requires `--history-dir`
  - window uses available reports up to requested `N`
  - stdout JSON report schema remains unchanged.

## Failure modes and diagnostics

- `--history-window` without `--history-dir` returns usage error:
  - `--history-window requires --history-dir`
- unreadable or invalid history report JSON returns explicit load/parse errors (existing baseline-path error handling path).

## Example usage

```bash
ailang sec audit \
  --path examples/hello \
  --format json \
  --history-dir .ailang/audit-history \
  --history-window 5
```

Output behavior:
- stdout: full audit report JSON
- stderr: `history window summary: {...}` plus baseline/history/security-map paths.

## Tradeoffs and next steps

- Tradeoff: summary is currently CLI-side auxiliary output, not embedded in the core report schema.
- Next:
  - add persisted multi-run severity rollups/time-window aggregates in core audit model once format stability requirements are finalized.
