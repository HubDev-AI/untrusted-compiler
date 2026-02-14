# M14 Slice: Replay Capture Route Type Contract Enforcement

This slice hardens replay capture validation for optional request route metadata.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-capture-contract.sh`
- `scripts/test-replay-capture-contract.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Capture files may include normalized route patterns for observability and replay context.
Invalid route value types make metadata unreliable and reduce deterministic contract quality.

## How it works internally

1. CLI validator now enforces non-empty string type for `request.route` when present.
2. Shell contract checker mirrors this rule in `jq`.
3. Regression coverage adds explicit invalid-route fixtures in shell and Rust tests.

## Inputs/outputs and constraints

Inputs:
- optional `capture.request.route`.

Outputs:
- deterministic validation errors for invalid route metadata.

Constraints:
- if present, `request.route` must be a non-empty string.

## Failure modes and diagnostics

Invalid route metadata fails with:
- `capture.request.route must be a non-empty string when present`

## Example usage

Valid:

```json
{ "request": { "route": "/users/:id" } }
```

Invalid:

```json
{ "request": { "route": 5 } }
```

## Tradeoffs and next steps

Tradeoffs:
- Stricter contract can reject older fixtures that carried non-string route placeholders.

Next steps:
- Add optional route-pattern normalization checks once route canonicalization rules are finalized.
