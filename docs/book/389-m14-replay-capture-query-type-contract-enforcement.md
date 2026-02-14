# M14 Slice: Replay Capture Query Type Contract Enforcement

This slice tightens replay capture validation so `request.query` is type-safe when present.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-capture-contract.sh`
- `scripts/test-replay-capture-contract.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Replay signature derivation consumes optional query input.
Allowing non-string query values creates ambiguous serialization behavior and weakens deterministic replay guarantees.

## How it works internally

1. CLI capture validator now checks `request.query` type when key is present.
2. Shell contract checker mirrors this as a `jq` predicate.
3. Regression tests cover explicit failure for numeric query payloads.

## Inputs/outputs and constraints

Inputs:
- `capture.request.query` (optional).

Outputs:
- deterministic contract pass/fail for query typing.

Constraints:
- if `request.query` exists, it must be a JSON string.

## Failure modes and diagnostics

When invalid:
- `capture.request.query must be a string when present`

## Example usage

Valid:

```json
{ "request": { "query": "a=1&b=2" } }
```

Invalid:

```json
{ "request": { "query": 5 } }
```

## Tradeoffs and next steps

Tradeoffs:
- Stricter payload typing may reject legacy fixtures that used non-string query forms.

Next steps:
- Add optional normalization checks for query formatting (`?` prefix handling) if consumers require canonical query style on capture write.
