# M14 Restart Checkpoint

This is the handoff checkpoint to resume work immediately after machine restart.

## Current state

- Branch: `main`
- Working tree: clean
- Latest commits:
  - `cbd96d0` - `M14: add mock dependency signature output contract`
  - `32dc8e8` - `M14: harden replay dependency matching and capture contracts`

## M14 slices completed in these commits

- Mock net stub signature matching (`REPLAY.STUB_MISSING`).
- Mock matched net response summary output (`status`, `truncated`, `bodyKind`).
- JSON contract locks for replay mock output fields.
- DB/FS stub contract validation and replay summary ingestion (`stubDetails`).
- DB/FS stub request-signature uniqueness checks.
- Mock-mode DB/FS capture dependency signature matching (`REPLAY.DB_STUB_MISSING`, `REPLAY.FS_STUB_MISSING`).
- Mock dependency match counts output (`mockDependencyMatches`).
- Capture dependency signature uniqueness enforcement (`REPLAY.CAPTURE_DB_DEPENDENCY_DUPLICATE`, `REPLAY.CAPTURE_FS_DEPENDENCY_DUPLICATE`).
- Mock dependency signature list output contract (`mockDependencySignatures`).

## Validation status

All of these passed at checkpoint time:

```bash
cargo test -p sec4 --test json_output replay_check
scripts/test-replay-capture-contract.sh
scripts/check-replay-capture-contract.sh
scripts/test-replay-cli-json-contract.sh
scripts/test-replay-cli-json-contract-guard.sh
scripts/test-replay-stub-registry-contract.sh
scripts/check-replay-stub-registry-contract.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-naming-lock.sh
```

## Docs/roadmap coverage

Roadmap and book index are aligned through chapter `382`.
This checkpoint is chapter `383`.

## Immediate next step after restart

Start next M14 slice:

1. Add replay output attachment for matched DB/FS mock response summaries (not only signatures/counts).
2. Lock new output fields in replay JSON contract scripts.
3. Add CLI integration tests for success + miss + malformed summary paths.
4. Update roadmap M14 tracking and add the next book chapter.

## Resume commands

```bash
# from repository root
git log --oneline -n 5
cargo test -p sec4 --test json_output replay_check
```
