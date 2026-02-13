# M14 Slice: Replay Stub Registry Contract Bootstrap

This slice adds deterministic contract validation for replay stub registries used by future `replay.effects=mock` execution.

## What it is

Added:
- `captures/sample-replay-stubs.json`
- `scripts/check-replay-stub-registry-contract.sh`
- `scripts/test-replay-stub-registry-contract.sh`

Updated:
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

Replay capture contracts were already enforced, but there was no guardrail for stub artifact shape. Without a deterministic stub contract, `mock`-mode replay would drift and become unreliable.

## What changed

1. Added replay stub-registry contract checker
- `scripts/check-replay-stub-registry-contract.sh` validates:
  - `version == "0.1"`,
  - `stubs.net` is an array,
  - each net stub has deterministic request identity (`method`, `url`, optional `bodySha256`),
  - each response has valid status + payload (`bodyBase64` or `bodySha256`) + `truncated`,
  - redaction metadata exists (`headers`, `jsonPaths` arrays),
  - net request signatures are unique.

2. Added regression guard script
- `scripts/test-replay-stub-registry-contract.sh` covers:
  - valid fixture pass,
  - missing `stubs.net` fail,
  - missing response payload fail,
  - duplicate request signature fail.

3. Wired CI and closure audit enforcement
- Naming-lock CI now runs replay stub-registry contract tests.
- Closure audit adds `M14-C`:
  - `naming-lock CI enforces replay stub registry contract test`.
- Closure audit fixture/ordering tests were updated to include `M14-C`.

## Validation

```bash
scripts/test-replay-stub-registry-contract.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/test-check-milestone-closure.sh
```

## Tradeoffs

- This slice validates contract shape and determinism, not runtime stub ingestion/execution.
- Runtime net/db/fs stub playback remains a later M14 track once replay execution mode is wired.
