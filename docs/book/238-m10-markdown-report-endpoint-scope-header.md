# 238 M10 Slice: Markdown Report Endpoint-Scope Header

This chapter documents endpoint-scope visibility improvements in published benchmark markdown reports.

## What it is

Updated:
- `benchmark-suite/scripts/publish_report.sh`
- `benchmark-suite/scripts/test_publish_report.sh`
- `benchmark-suite/README.md`

Key changes:
- markdown publish output header now includes endpoint scope:
  - `Endpoints in matrix (<count>): <endpoint-list>`
- publish test now verifies endpoint-scope header presence.

## Why it exists

When matrix runs are filtered (for example `ENDPOINTS=ping`), published reports must make scope explicit so readers do not mistake partial data for full-suite results.

## How it works internally

1. `publish_report.sh` reads endpoint list and count from `compare-matrix.json`.
2. It writes one metadata bullet under report header with count and comma-separated endpoint names.
3. All later sections continue to render from matrix and analysis artifacts as before.

## Inputs, outputs, and constraints

- Inputs:
  - `compare-matrix.json` endpoint array.
- Outputs:
  - markdown header metadata including endpoint scope.
- Constraints:
  - compare matrix must contain at least one endpoint (already validated by script).

## Failure modes and diagnostics

- no additional failure modes beyond existing matrix validation checks.

## Example output

```markdown
- Endpoints in matrix (3): ping, decode, users-post
```

## Tradeoffs and next steps

- Tradeoff:
  - adds one additional metadata line to report header.
- Next:
  - optionally include implementation list and selected policy profile in the same header block.
