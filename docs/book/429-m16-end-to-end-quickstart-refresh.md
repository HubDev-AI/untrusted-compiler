# 429 M16 Slice: End-to-End Quickstart Refresh

This chapter documents the README refresh that makes current runtime maturity and end-to-end execution entrypoints explicit.

## What it is

Updated `README.md` to reflect current status beyond early milestones and added explicit quickstart commands for live runtime serving.

## Why it exists

The README still described early-phase progress while the project already had replay/runtime/security middleware slices and closure gates through M16. This mismatch made onboarding slower and obscured the easiest path to run the language end-to-end.

## What changed

1. Replaced stale milestone snapshot with current implementation summary:
   - live compile/runtime path,
   - replay/runtime contracts,
   - M16 HTTP runtime behavior,
   - closure gate coverage.
2. Added a deterministic smoke entrypoint:
   - `scripts/smoke-sec4-run-hello-api.sh`.
3. Added manual live-run commands with exact auth/CSRF request examples.

## Validation

- `scripts/smoke-sec4-run-hello-api.sh`
- `scripts/check-naming-lock.sh`
- `scripts/check-no-local-path-leaks.sh`

Result: pass.

## Tradeoffs and next steps

- README remains concise while linking detailed specs/chapters for deeper context.
- Future refreshes should keep the top-level status synced with closure-audit gates to avoid drift.
