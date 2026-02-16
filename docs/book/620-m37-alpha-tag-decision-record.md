# M37-S8 Alpha Tag Decision Record

## What it is

M37-S8 adds a deterministic tag-decision execution record:

- `scripts/build-m37-alpha-tag-decision-record.sh`
- `build/m37-alpha-tag-decision-record.json`
- `build/m37-alpha-tag-decision-record.md`

The record consumes M37-S6 and M37-S7 artifacts and captures the executed alpha decision (`GO` or `HOLD`) with closure outcome.

## Why it exists

The checklist and release-note package can say a candidate is ready, but we still need one explicit artifact that records what decision was executed.

This slice closes the loop from evidence -> notes -> decision, so M37 closure is auditable.

## How it works

1. Validate input contracts:
   - checklist marker must be `M37-S6`,
   - release-notes marker must be `M37-S7`,
   - identity hashes must match across both artifacts.
2. Resolve decision:
   - explicit `--decision go|hold`, or default from release-notes package.
3. Enforce GO safety:
   - GO is rejected unless checklist/release notes are `PASS/GO` and `alpha-ready`.
4. Emit decision record with:
   - marker `M37-S8`,
   - executed decision,
   - closure outcome (`PASS` for GO, `PENDING` for HOLD),
   - identity stamps and next action.

## Validation

- `bash -n scripts/build-m37-alpha-tag-decision-record.sh`
- `bash -n scripts/test-build-m37-alpha-tag-decision-record.sh`
- `scripts/test-build-m37-alpha-tag-decision-record.sh`
- `scripts/build-m37-alpha-publish-checklist-delta.sh --format json > build/m37-alpha-publish-checklist-delta.json`
- `scripts/build-m37-alpha-release-notes.sh --format json > build/m37-alpha-release-notes.json`
- `scripts/build-m37-alpha-tag-decision-record.sh --decision go --format json`

## Trade-offs

- The record is a control-plane artifact; it does not perform tagging itself.
- A strict GO guard can block accidental promotion, but it means checklist/notes must be refreshed before decision execution.

## Next

1. Mark M37 closed in roadmap with decision artifact references.
2. Start M38 on post-alpha hardening priorities.
