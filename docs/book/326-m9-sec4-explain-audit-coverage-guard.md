# 326 M9 Slice: sec4 explain Audit Coverage Guard

This chapter documents the automated guard that ensures `sec4 explain` keeps parity with `sec4 audit` finding IDs.

## What it is

Updated:
- `scripts/check-sec4-explain-audit-coverage.sh`
- `scripts/test-check-sec4-explain-audit-coverage.sh`
- `.github/workflows/naming-lock.yml`
- `docs/book/318-m9-sec4-explain-low-frequency-audit-finding-coverage.md`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Manual mapping updates can drift when new audit finding IDs are introduced. This guard enforces that every `finding("...")` ID in `sec4-core` has a corresponding `explain_topic` mapping in the CLI.

## How it works internally

`check-sec4-explain-audit-coverage.sh`:
1. extracts audit finding IDs from `compiler/sec4-core/src/audit.rs`,
2. extracts mapped IDs from `compiler/sec4-cli/src/main.rs` match arms,
3. fails if any audit IDs are missing from explain mappings.

It supports `--audit-file` and `--explain-file` overrides for testing.

## Tests and CI

- `test-check-sec4-explain-audit-coverage.sh` validates:
  - pass on current repo files,
  - fail on synthetic missing mapping case,
  - pass after synthetic mapping completion.
- `naming-lock.yml` now runs the coverage check script on PRs and `main` pushes.

## Inputs, outputs, and constraints

- Inputs:
  - audit source file and explain source file.
- Output:
  - pass/fail parity signal.
- Constraints:
  - regex-based extraction assumes current `finding("ID", ...)` and `"ID" => {` code shapes.

## Example usage

```bash
scripts/check-sec4-explain-audit-coverage.sh
scripts/test-check-sec4-explain-audit-coverage.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - parser-independent regex extraction is lightweight but tied to source formatting patterns.
- Next:
  - optionally migrate guard to AST-level extraction inside `sec4-core` tooling for stronger shape independence.
