# M14 Slice: Replay Capture URL-Derivation Contract Enforcement

This slice hardens replay capture contract validation so request signatures are always derivable from capture input.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/check-replay-capture-contract.sh`
- `scripts/test-replay-capture-contract.sh`
- `scripts/test-replay-capture-compat.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Replay signature matching relies on deterministic net-request signatures.
Before this slice, captures without `request.url` and without `request.scheme`/`request.host` could pass some contract paths and fail later in mode-specific execution.

Locking URL-derivation requirements directly in the capture contract makes failures earlier, deterministic, and mode-independent.

## How it works internally

1. Capture contract validation now requires non-empty `request.path`.
2. Capture contract validation now calls `capture_net_request_signature(...)` during validation.
3. Signature derivation enforces one of:
- non-empty `request.url`, or
- non-empty `request.scheme` + `request.host` (with `request.path`, optional `request.query`).
4. Shell contract checker mirrors the same rule via `jq` predicates.

## Inputs/outputs and constraints

Inputs:
- replay capture JSON under `request.*`.

Outputs:
- deterministic validation pass/fail before mode-specific replay logic.

Constraints:
- `request.path` must be a non-empty string.
- URL derivation fields must be present in one supported form.

## Failure modes and diagnostics

Contract validation now fails with deterministic URL-derivation diagnostics when request signature fields are insufficient, using the same canonical message already used by replay signature derivation.

## Example usage

Valid capture request shapes:
- `{ "url": "https://example.com/ping", "path": "/ping", ... }`
- `{ "scheme": "https", "host": "example.com", "path": "/ping", ... }`

Invalid shape:
- `{ "path": "/ping", ... }` (missing both `url` and `scheme`/`host`).

## Tradeoffs and next steps

Tradeoffs:
- Slightly stricter contract may reject older loose fixtures.
- Validation now depends on shared signature-derivation logic.

Next steps:
- Add explicit error-code mapping in replay JSON output for contract-level URL-derivation failures if machine consumers need code-first triage.
