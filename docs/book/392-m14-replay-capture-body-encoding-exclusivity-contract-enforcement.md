# M14 Slice: Replay Capture Body-Encoding Exclusivity Contract Enforcement

This slice hardens replay capture validation by rejecting mixed body representations when `encoding=none`.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-capture-contract.sh`
- `scripts/test-replay-capture-contract.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Capture body representation should be unambiguous:
- `encoding=base64` carries bytes + hash.
- `encoding=none` carries hash-only metadata.

Allowing `bytes` alongside `encoding=none` creates conflicting contracts and undermines deterministic replay semantics.

## How it works internally

1. CLI validator now rejects `request.body.bytes` when `request.body.encoding` is `none`.
2. Shell contract checker mirrors this with `jq`.
3. Regression tests assert deterministic failures for this mixed representation.

## Inputs/outputs and constraints

Inputs:
- `capture.request.body.encoding`
- optional/present `capture.request.body.bytes`

Outputs:
- deterministic validation failure for invalid mixed shape.

Constraints:
- `encoding=none` requires `sha256` and forbids `bytes`.

## Failure modes and diagnostics

Invalid mixed body shape fails with:
- `capture.request.body.bytes must be absent for encoding=none`

## Example usage

Valid (`encoding=none`):

```json
{ "encoding": "none", "sha256": "body_sha256", "truncated": false }
```

Invalid (`encoding=none` + bytes):

```json
{ "encoding": "none", "bytes": "e30=", "sha256": "body_sha256", "truncated": false }
```

## Tradeoffs and next steps

Tradeoffs:
- Legacy captures that mixed representations must be rewritten.

Next steps:
- Add explicit decode checks for base64 payload bytes when replay consumers require syntactic base64 validation at contract time.
