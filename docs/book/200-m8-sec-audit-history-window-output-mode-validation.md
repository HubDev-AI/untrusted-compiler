# 200 M8 Slice: sec.audit History-Window Output-Mode Validation

This chapter documents integration coverage that validates history-window summary behavior across output modes.

## What it is

Added CLI integration coverage proving:
- JSON mode (`--format json`): history-window summary is emitted on stderr.
- Text mode (default): history-window summary is emitted on stdout.

## Why it exists

`sec audit` has strict output contracts:
- JSON mode must keep stdout machine-parseable.
- Text mode should keep stderr clean on success.

History-window summaries are auxiliary output, so mode-specific routing must stay deterministic.

## How it works internally

1. Added a text-mode history-window integration test:
   - runs two history-backed audits
   - verifies summary appears on stdout
   - verifies stderr stays empty.
2. Existing JSON-mode history-window test continues to verify stderr routing.
3. Full `json_output` integration suite confirms no regressions in CLI output contracts.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-cli/tests/json_output.rs`
- Output:
  - explicit regression guard for output-channel routing of history-window summaries.
- Constraint:
  - JSON stdout remains reserved for the report body.

## Failure modes and diagnostics

- If routing regresses:
  - JSON-mode tests fail when summary leaks into stdout.
  - text-mode tests fail if summary moves to stderr.

## Example usage

Text mode:

```bash
ailang sec audit --history-dir .ailang/audit-history --history-window 2
```

JSON mode:

```bash
ailang sec audit --format json --history-dir .ailang/audit-history --history-window 2
```

## Tradeoffs and next steps

- Tradeoff: mode-specific routing is intentionally strict, which adds test overhead but preserves automation contracts.
- Next:
  - keep applying the same routing discipline to any future auxiliary outputs.
