# M14 Slice: Replay Capture Base64 Syntax Contract Enforcement

This slice hardens replay capture validation by requiring syntactically valid base64 text for `encoding=base64` bodies.

## What it is

Updated:
- `compiler/sec4-cli/Cargo.toml`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-capture-contract.sh`
- `scripts/test-replay-capture-contract.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

A non-empty `bytes` field is not sufficient if the content is malformed base64.
Without syntax checks, invalid body payloads can pass contract checks and fail later in replay consumers.

## How it works internally

1. CLI validator decodes `request.body.bytes` via standard base64 decoder when `encoding=base64`.
2. Invalid decode now produces deterministic contract failure.
3. Shell checker adds a strict base64-shape predicate (`allowed charset + padding + mod4 length`).
4. Regression tests cover malformed base64 inputs in both CLI and shell paths.

## Inputs/outputs and constraints

Inputs:
- `capture.request.body.encoding`
- `capture.request.body.bytes`

Outputs:
- deterministic pass/fail for base64 body syntax.

Constraints:
- if `encoding=base64`, `bytes` must be non-empty and syntactically valid base64.

## Failure modes and diagnostics

Malformed base64 now fails with:
- `capture.request.body.bytes must be valid base64 for encoding=base64`

## Example usage

Valid:

```json
{ "encoding": "base64", "bytes": "e30=", "sha256": "body_sha256", "truncated": false }
```

Invalid:

```json
{ "encoding": "base64", "bytes": "%%%invalid%%%", "sha256": "body_sha256", "truncated": false }
```

## Tradeoffs and next steps

Tradeoffs:
- Slightly stricter validation can reject legacy captures that were never decode-checked.
- Shell-level regex checks are syntactic; CLI decoder remains the source of truth.

Next steps:
- Optionally enforce canonical base64 formatting or decoded-size limits if capture policies require tighter normalization.
