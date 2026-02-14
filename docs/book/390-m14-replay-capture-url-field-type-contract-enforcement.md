# M14 Slice: Replay Capture URL-Field Type Contract Enforcement

This slice hardens replay capture validation for optional URL-related request fields.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-capture-contract.sh`
- `scripts/test-replay-capture-contract.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Replay signature derivation can read optional URL-related fields.
If these keys exist but hold invalid values, behavior becomes ambiguous and contract quality drops.

## How it works internally

1. CLI validator now enforces:
- `request.url` is non-empty string when present.
- `request.scheme` is non-empty string when present.
- `request.host` is non-empty string when present.
2. Shell checker mirrors the same constraints via `jq`.
3. Regression coverage adds explicit invalid-`url` fixtures.

## Inputs/outputs and constraints

Inputs:
- optional `request.url`, `request.scheme`, `request.host` in capture JSON.

Outputs:
- deterministic contract failures for invalid optional URL-field types.

Constraints:
- each optional URL field must be a non-empty string when provided.

## Failure modes and diagnostics

Example diagnostic:
- `capture.request.url must be a non-empty string when present`

## Example usage

Valid:

```json
{ "request": { "scheme": "https", "host": "example.com", "path": "/ping" } }
```

Invalid:

```json
{ "request": { "url": 5, "scheme": "https", "host": "example.com", "path": "/ping" } }
```

## Tradeoffs and next steps

Tradeoffs:
- Stricter optional-field validation can reject loose legacy fixtures.

Next steps:
- Add optional `request.route` type validation if replay consumers begin relying on route metadata.
