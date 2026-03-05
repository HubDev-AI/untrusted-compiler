# 1041 M39 Slice: Zed Plugin Smoke Runner Script

This slice adds a single-command smoke runner for the Zed plugin workflow.

## What changed

1. Added `$REPO_ROOT/scripts/run-zed-plugin-smoke.sh`:
   - validates Zed grammar pin guard,
   - runs `sec4 check` for `examples/zed-plugin-smoke`,
   - runs `sec4 fmt` in a temp project copy,
   - asserts deterministic formatter output for `playground/format-me.ut`,
   - optionally runs targeted language-server formatting tests.
2. Added fast loop mode:
   - `--fast` flag and `FAST=1` env skip targeted language-server tests.
3. Updated Zed extension operator docs to point to the smoke runner command.

## Why

Editor integration confidence should be one command, not a manual checklist scattered across files. This runner gives deterministic, local, repeatable checks while preserving the tracked sample project state.

## Validation

- `scripts/run-zed-plugin-smoke.sh --fast`
- `scripts/check-zed-grammar-pin.sh`

## Notes

`--fast` is intended for tight implementation loops. Use full mode before PR merge when you need direct language-server formatting test confirmation.
