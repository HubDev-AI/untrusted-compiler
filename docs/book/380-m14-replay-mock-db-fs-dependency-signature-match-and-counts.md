# M14 Slice: Replay Mock DB/FS Dependency Signature Match and Counts

This slice extends mock-mode determinism from net signatures to captured DB/FS dependency signatures and exposes deterministic match counts in replay output.

## What it is

Updated:
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`

(Replay runtime wiring for these checks was already present in `sec4 replay`; this slice closes coverage and roadmap/docs alignment.)

## Why it exists

Mock mode already guaranteed deterministic matching for the captured inbound net request, but captured dependency intent (`capture.dependencies.db/fs`) was not explicitly covered by integration tests. That left room for regressions where DB/FS dependency misses or output counters could drift silently.

## What changed

1. DB/FS dependency-miss integration tests
- Added replay CLI tests that fail deterministically when capture dependency signatures are absent from stubs:
  - `REPLAY.DB_STUB_MISSING: <queryTemplateId|paramsSha256OrDash>`
  - `REPLAY.FS_STUB_MISSING: <lowercase(op)|pathSha256>`

2. Dependency-match count assertions
- Added replay CLI JSON-mode test coverage for `mockDependencyMatches` values when DB/FS dependencies are present and matched.
- Strengthened existing JSON contract assertions so mock runs without dependency entries are pinned to:
  - `mockDependencyMatches.db = 0`
  - `mockDependencyMatches.fs = 0`

3. Roadmap/book alignment
- Updated M14 build tasks, live tracking, and exit criteria to explicitly include DB/FS dependency-signature matching and `mockDependencyMatches` output requirements.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
scripts/test-replay-cli-json-contract.sh
scripts/test-replay-cli-json-contract-guard.sh
scripts/test-replay-capture-contract.sh
scripts/check-replay-capture-contract.sh
scripts/test-replay-stub-registry-contract.sh
scripts/check-replay-stub-registry-contract.sh
```

## Tradeoffs

- This slice validates deterministic DB/FS dependency matching behavior and output contracts only; it does not introduce DB/FS side-effect replay execution.
- Dependency-match counting is aggregate observability (`mockDependencyMatches`), not per-step execution tracing.
