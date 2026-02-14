# M14 Slice: Replay Mock Stub Signature Match

This slice adds deterministic mock-mode request matching in `sec4 replay` so mock runs fail when required net stubs are missing.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Replay already validated capture/stub contracts, but `--effects mock` still passed without confirming that the captured request actually existed in `stubs.net`. That allowed false-positive mock runs.

## What changed

1. Capture request signature derivation
- Added deterministic signature derivation for replay capture requests:
  - format: `METHOD|URL|BODY_SHA`
  - URL is resolved from `request.url` or built from `scheme + host + path + query`.
  - `BODY_SHA` uses `request.body.sha256` when present, otherwise `-`.

2. Mock-mode stub match enforcement
- In `--effects mock` mode, replay now requires that the derived capture signature exists in `stubs.net`.
- Missing matches now fail deterministically with:
  - `REPLAY.STUB_MISSING: <signature>`

3. JSON/text observability
- JSON output now includes `mockRequestSignature` for successful mock runs.
- Text output now includes `replay mock stub matched: <signature>` when mock matching succeeds.

4. Test coverage
- Added regression test for mock-mode stub-miss failure (`REPLAY.STUB_MISSING`).
- Updated JSON-mode replay test to assert `mockRequestSignature`.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
```

## Tradeoffs

- Matching currently enforces captured net request presence only; it does not validate downstream DB/FS stub hit paths yet.
- Signature matching depends on canonical URL fields in captures (`url` or `scheme`/`host` + path).
