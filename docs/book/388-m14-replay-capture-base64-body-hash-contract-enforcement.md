# M14 Slice: Replay Capture Base64 Body Hash Contract Enforcement

This slice hardens replay capture validation by requiring `request.body.sha256` when request bodies are stored with `encoding=base64`.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-capture-contract.sh`
- `scripts/test-replay-capture-contract.sh`
- `scripts/test-replay-capture-compat.sh`
- `captures/sample-capture.json`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Replay and audit flows need deterministic body identity, not only body bytes.
Before this slice, `encoding=base64` captures could omit `sha256`, which weakened integrity checks and parity with hash-only (`encoding=none`) captures.

## How it works internally

1. CLI capture contract validation now requires non-empty `request.body.sha256` for `encoding=base64`.
2. Existing `encoding=none` hash requirement remains unchanged.
3. Shell contract checker (`jq`) mirrors the same requirement.
4. Regression coverage now includes an explicit failure case for base64 payloads without hash.

## Inputs/outputs and constraints

Inputs:
- replay capture JSON `request.body` fields.

Outputs:
- deterministic validation success/failure for body hash presence.

Constraints:
- `encoding=base64` requires non-empty `bytes` and non-empty `sha256`.
- `encoding=none` requires non-empty `sha256`.

## Failure modes and diagnostics

If `encoding=base64` lacks `sha256`, replay fails with:
- `capture.request.body.sha256 must be present for encoding=base64`

## Example usage

Valid base64 body:

```json
{
  "encoding": "base64",
  "bytes": "e30=",
  "sha256": "body_sha256",
  "truncated": false
}
```

Invalid base64 body:

```json
{
  "encoding": "base64",
  "bytes": "e30=",
  "truncated": false
}
```

## Tradeoffs and next steps

Tradeoffs:
- Older fixtures without base64 hash must be updated.
- Slightly stricter contract for replay intake.

Next steps:
- Optionally validate `sha256` format (hex-length) once capture producers are standardized.
