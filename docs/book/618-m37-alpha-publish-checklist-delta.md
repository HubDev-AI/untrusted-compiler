# M37-S6 Alpha Publish Checklist Delta

## What it is

M37-S6 adds a deterministic publish-checklist delta artifact for alpha decisions:

- `scripts/build-m37-alpha-publish-checklist-delta.sh`
- `build/m37-alpha-publish-checklist-delta.json`
- `build/m37-alpha-publish-checklist-delta.md`

It consumes release evidence from `build/release-alpha-gate/` and outputs one canonical pass/pending summary with `tagDecision` (`GO` or `HOLD`).

## Why it exists

M37-S5 proved runtime/compiler behavior in tests, but tagging alpha requires one explicit release-readiness summary artifact.

This slice makes the tag decision reproducible and auditable from concrete evidence instead of manual interpretation.

## How it works

1. Validate required release-gate artifacts exist:
   - `summary.txt`
   - `checksums.txt`
   - `hello` and `hello-api` build metadata + audit reports.
2. Read identity hashes from `checksums.txt`:
   - `policy_identity_hash`
   - `compiler_identity_hash`
   - `runtime_identity_hash`
3. Validate per-sample consistency:
   - metadata identity hashes match release identity hashes,
   - audit identity hashes match release identity hashes,
   - audit highest severity is below `HIGH`.
4. Build deterministic output:
   - `checks[]` statuses (release gate, naming lock, milestone closure),
   - `samples[]` statuses with evidence,
   - `overall` (`PASS` or `PENDING`),
   - `tagDecision` (`GO` or `HOLD`),
   - `nextAction`.

## Validation

- `bash -n scripts/build-m37-alpha-publish-checklist-delta.sh`
- `bash -n scripts/test-build-m37-alpha-publish-checklist-delta.sh`
- `scripts/test-build-m37-alpha-publish-checklist-delta.sh`
- `cargo test -p sec4 --test commands`
- `cargo test -p sec4 --test alpha_smoke`

## Trade-offs

- This artifact does not run tests by itself; it summarizes release evidence produced elsewhere (`release-alpha-gate` + build/audit outputs).
- It intentionally fails fast on missing/invalid release artifacts to avoid silently producing misleading pass states.

## Next

1. Draft M37-S7 alpha release-note package directly from this checklist delta artifact.
2. Record final alpha tag decision outcome and closure note for M37.
