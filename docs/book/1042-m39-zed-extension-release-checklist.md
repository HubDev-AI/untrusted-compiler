# 1042 M39 Slice: Zed Extension Release Checklist

This slice adds a deterministic release checklist command for the Zed extension lane.

## What changed

1. Added `$REPO_ROOT/scripts/check-zed-extension-release.sh`.
2. Checklist now validates:
   - required extension files exist,
   - required `zed-extension/extension.toml` wiring tokens are present,
   - grammar pin check passes,
   - plugin smoke runner passes in fast mode (unless `--skip-smoke` is used).
3. Updated `$REPO_ROOT/zed-extension/README.md` so release validation points to the new checklist.

## Why

Release readiness was spread across multiple commands. This checklist consolidates them into one deterministic command so operators and parallel agents can run the same gate before publication.

## Validation

- `bash -n scripts/check-zed-extension-release.sh`
- `scripts/check-zed-extension-release.sh`

## Notes

Use `--skip-smoke` only for temporary diagnostic runs. Normal release validation should execute the fast smoke runner.
