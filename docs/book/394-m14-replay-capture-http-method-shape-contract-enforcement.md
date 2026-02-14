# M14 Slice: Replay Capture HTTP-Method Shape Contract Enforcement

This slice hardens replay capture validation by enforcing a canonical HTTP method shape.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-capture-contract.sh`
- `scripts/test-replay-capture-contract.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Replay signature generation depends on request method identity.
Allowing arbitrary/lowercase values weakens determinism and lets malformed fixtures slip through.

## How it works internally

1. CLI validator now requires `request.method` to be a non-empty uppercase string.
2. Shell checker enforces the same via regex (`^[A-Z]+$`).
3. Regression tests cover invalid lowercase method inputs.

## Inputs/outputs and constraints

Inputs:
- `capture.request.method`.

Outputs:
- deterministic failure for non-canonical method shapes.

Constraints:
- method must be non-empty and uppercase.

## Failure modes and diagnostics

Invalid method values fail with:
- `capture.request.method must be a non-empty uppercase string`

## Example usage

Valid:

```json
{ "request": { "method": "GET" } }
```

Invalid:

```json
{ "request": { "method": "get" } }
```

## Tradeoffs and next steps

Tradeoffs:
- Stricter method normalization rejects legacy lowercase fixtures.

Next steps:
- Expand method validation to full RFC token charset if non-letter extension methods are required.
