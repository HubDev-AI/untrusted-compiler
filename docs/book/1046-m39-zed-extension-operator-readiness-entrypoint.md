# 1046 M39 Slice: Zed Extension Operator Readiness Entrypoint

This slice consolidates Zed extension local operator checks behind a single command.

## What changed

1. Added `scripts/check-zed-extension-operator-readiness.sh`.
2. The command now runs:
   - `scripts/check-zed-extension-release.sh --skip-smoke`
   - `scripts/run-zed-extension-operator-smoke.sh`
   - `scripts/stage-zed-extension-bundle.sh --clean`
3. Added deterministic staged-manifest presence assertion:
   - `build/zed-extension-bundle/bundle-manifest.json`
4. Updated `zed-extension/README.md` with one-command readiness usage.

## Why

Operator steps were already deterministic but spread across multiple commands. This slice reduces execution friction for local release prep and parallel agent handoff by creating one canonical readiness entrypoint.

## Validation

- `bash -n scripts/check-zed-extension-operator-readiness.sh`
- `scripts/check-zed-extension-operator-readiness.sh`

## Notes

- `--skip-release`, `--skip-operator-smoke`, and `--skip-stage` allow targeted debug loops.
- `--stage-output <path>` stages the bundle outside default `build/` when needed.
