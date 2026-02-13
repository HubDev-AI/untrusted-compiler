# 198 M8 Slice: sec.audit History-Window Bounds Validation

This chapter documents a hardening pass for `sec audit` history-window argument validation.

## What it is

Added explicit validation that rejects:
- `--history-window 0`

with a deterministic CLI error:
- `--history-window must be >= 1`

## Why it exists

The initial implementation silently coerced zero to one (`max(1)` behavior). That can hide caller mistakes and make automation behavior ambiguous. Explicit validation keeps the interface auditable and predictable.

## How it works internally

1. `cmd_sec_audit(...)` now validates `history_window` before analysis:
   - if `history_window == 0`, return usage error code.
2. Existing integration tests now cover the zero-window rejection path.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-cli/src/main.rs`
  - `compiler/ailang-cli/tests/json_output.rs`
- Output:
  - strict argument validation for history-window bounds.
- Constraint:
  - valid window values are positive integers only.

## Failure modes and diagnostics

- Invalid zero window:
  - stderr: `--history-window must be >= 1`
  - non-zero exit status.

## Example usage

Valid:

```bash
ailang sec audit --history-dir .ailang/audit-history --history-window 3
```

Invalid:

```bash
ailang sec audit --history-dir .ailang/audit-history --history-window 0
```

## Tradeoffs and next steps

- Tradeoff: strict validation is less forgiving but prevents silent coercion.
- Next:
  - keep validating new CLI security/reporting flags with explicit bounds and dependency contracts.
