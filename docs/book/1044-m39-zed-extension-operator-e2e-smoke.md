# 1044 M39 Slice: Zed Extension Operator End-to-End Smoke

This slice adds a deterministic clean-profile operator smoke command for the Zed extension flow.

## What changed

1. Added `scripts/run-zed-extension-operator-smoke.sh`.
2. The smoke command now executes:
   - local extension install via `scripts/manage-zed-extension-local.sh install`,
   - plugin smoke runner via `scripts/run-zed-plugin-smoke.sh --fast`,
   - rollback via `scripts/manage-zed-extension-local.sh rollback`.
3. The smoke flow seeds a previous install marker and asserts rollback restores that marker.
4. Updated `zed-extension/README.md` to document the end-to-end operator smoke command.

## Why

Individual commands already existed, but local operator confidence still required manual choreography. This slice locks the real install/update/rollback lifecycle into one repeatable command.

## Validation

- `bash -n scripts/run-zed-extension-operator-smoke.sh`
- `scripts/run-zed-extension-operator-smoke.sh`

## Notes

- By default, the command uses a temporary clean extension profile and deletes it after success.
- Use `--keep-temp` for debugging local failures.
