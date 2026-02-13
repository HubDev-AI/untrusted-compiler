# 241 M10 Slice: Markdown Report Implementation-Scope Header

This chapter documents implementation-scope visibility in published benchmark markdown reports.

## What it is

Updated:
- `benchmark-suite/scripts/publish_report.sh`
- `benchmark-suite/scripts/test_publish_report.sh`
- `benchmark-suite/README.md`

Key changes:
- markdown report header now includes:
  - `Implementations in matrix (<count>): <impl-list>`
- publish test coverage validates implementation-scope header.

## Why it exists

Implementation-filtered runs (for example `IMPLS=node,go`) should be obvious in published output. Without explicit scope, partial implementation runs can be misread as full comparisons.

## How it works internally

1. `publish_report.sh` derives implementation list from matrix rows.
2. Unique implementation names are sorted and joined for display.
3. Header now includes both implementation scope and endpoint scope metadata.

## Inputs, outputs, and constraints

- Inputs:
  - `compare-matrix.json` `compared[].impl` rows.
- Outputs:
  - markdown header with implementation scope.
- Constraints:
  - matrix must contain at least one compared row per endpoint to produce useful scope output.

## Failure modes and diagnostics

- no additional failure modes beyond existing matrix validity checks.

## Example output

```markdown
- Implementations in matrix (4): sec4, go, node, rust
```

## Tradeoffs and next steps

- Tradeoff:
  - one additional metadata header line.
- Next:
  - optionally include selected policy profile/build metadata summary in report header for full comparison provenance.
