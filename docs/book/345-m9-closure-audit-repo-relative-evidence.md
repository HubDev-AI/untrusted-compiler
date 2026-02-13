# M9 Slice: Closure Audit Repo-Relative Evidence

This slice hardens closure-audit output to avoid local absolute path leakage.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

Closure audit currently prints evidence paths for matrix/workflow files. Without normalization, local runs can expose machine-specific absolute workspace paths in logs.

This conflicts with the naming/privacy requirement to avoid committing or publishing local absolute paths.

## What changed

1. Added evidence-path rendering helper
- `check-milestone-closure.sh` now renders evidence values as:
  - `.` when evidence equals repo root,
  - repo-relative paths when evidence is under repo root,
  - unchanged values for non-path or external strings.

2. Added regression test coverage
- `test-check-milestone-closure.sh` now captures successful audit output and asserts it does not contain the temporary repo-root absolute path.

3. Synced docs
- Roadmap and closure checklist now state that closure evidence output is repo-relative.

## Validation

```bash
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
```

## Tradeoffs

- Output normalization affects display only; closure gate semantics are unchanged.
- External/non-repo evidence strings are preserved as-is.
