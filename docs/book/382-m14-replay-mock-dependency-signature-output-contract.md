# M14 Slice: Replay Mock Dependency Signature Output Contract

This slice adds explicit matched-dependency signature output for mock replay and locks the JSON contract so the field cannot drift silently.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `scripts/test-replay-cli-json-contract.sh`
- `scripts/test-replay-cli-json-contract-guard.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

`mockDependencyMatches` gives aggregate counts, but debugging replay mismatches still benefits from seeing the exact dependency signatures that were matched. Without contract locking, this detail field could be removed or renamed without obvious breakage.

## What changed

1. Replay output field
- `sec4 replay --effects mock` now emits `mockDependencySignatures`:
  - JSON shape:
    - `mockDependencySignatures.db: string[]`
    - `mockDependencySignatures.fs: string[]`
  - Text output line:
    - `replay mock dependency signatures: db=<...> fs=<...>`
    - uses `-` placeholders when lists are empty.

2. Runtime data flow
- Replay now stores matched capture dependency signatures after DB/FS stub validation and reuses the same values in output rendering.

3. Contract locks and regression coverage
- JSON contract script now requires `"mockDependencySignatures"` key presence.
- Guard script includes positive fixtures with this field and negative fixture coverage when it is missing.
- CLI integration tests assert:
  - empty arrays for captures without dependencies,
  - concrete signatures for captures with matched DB/FS dependencies.

## Validation

```bash
cargo test -p sec4 --test json_output replay_check
scripts/test-replay-cli-json-contract.sh
scripts/test-replay-cli-json-contract-guard.sh
```

## Tradeoffs

- Signature list output improves debuggability but does not add execution tracing for each dependency step.
- Text output joins signatures with commas; JSON remains the canonical machine-readable contract.
