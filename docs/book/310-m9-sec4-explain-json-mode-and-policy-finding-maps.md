# 310 M9 Slice: sec4 explain JSON Mode and Policy Finding Maps

This chapter documents extending `sec4 explain` for tooling-friendly output and policy finding IDs.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

As the diagnostics/finding surface grows, triage consumers need both richer exact mappings and a machine-readable explain output for integration with scripts and editors.

## How it works internally

### 1) New explain output mode

`sec4 explain` now accepts:
- `--format text` (default)
- `--format json`

JSON mode emits:
- `code`
- `topic`
- `summary`
- `likelyActions`
- `relatedCommands`
- `docsPath`

### 2) Policy finding mapping support

`explain_topic` now includes explicit mappings for:
- `ALLOW_EXPIRED`
- `ALLOW_EXPIRING_SOON`

These map to policy-focused guidance and deterministic docs pointers.

## Tests

Added/updated CLI integration coverage in `json_output.rs`:
- `explain_policy_allow_expired_code_prints_targeted_guidance`
- `explain_json_mode_writes_parseable_payload`

Existing exact-code mapping tests continue to validate text mode behavior.

## Inputs, outputs, and constraints

- Input:
  - explain code/finding ID string.
- Output:
  - deterministic text or JSON guidance payload.
- Constraint:
  - mapping table remains static and must be extended intentionally with tests.

## Example usage

```bash
sec4 explain E2001
sec4 explain ALLOW_EXPIRED
sec4 explain E4001 --format json
```

## Tradeoffs and next steps

- Tradeoff:
  - JSON mode currently mirrors text-mode semantic payload only (no severity/category metadata).
- Next:
  - extend mapping coverage for additional `sec4 audit` finding IDs and evaluate optional metadata fields in explain JSON output.
