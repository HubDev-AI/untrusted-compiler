# 306 M9 Slice: sec4 explain High-Frequency Exact Mappings

This chapter documents extending `sec4 explain` exact-code mappings beyond the initial baseline set.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

`sec4 explain` had only a small exact-code set, so many high-frequency diagnostics fell back to family-level guidance. This slice improves precision for day-to-day triage.

## How it works internally

Added exact mappings in `explain_topic` for:
- `E1002`
- `E1003`
- `E2001`
- `E2002`
- `E4001`

Existing exact mappings remain:
- `E2003`
- `E4004`
- `E5001`
- `E6001`

Each exact code now resolves to:
1. targeted topic title,
2. targeted summary,
3. action set,
4. direct documentation pointer.

## Tests

`compiler/sec4-cli/tests/json_output.rs` now covers new exact mappings:
- `explain_exact_effect_code_uses_specific_mapping`
- `explain_exact_schema_call_contract_code_uses_specific_mapping`
- updated `explain_known_security_code_prints_targeted_guidance` assertions for `E1002` exact mapping.

## Inputs, outputs, and constraints

- Input:
  - diagnostic code passed to `sec4 explain`.
- Output:
  - deterministic, code-specific guidance for mapped codes,
  - family fallback for unmapped codes.
- Constraint:
  - mapping table is static in CLI and should be extended deliberately with tests.

## Failure modes and diagnostics

- Unknown code:
  - still routes to generic family/unknown guidance.
- Missing test coverage for new exact mapping:
  - risk of regressions in topic text/docs pointer consistency.

## Example usage

```bash
sec4 explain E1002
sec4 explain E2001
sec4 explain E4001
```

## Tradeoffs and next steps

- Tradeoff:
  - exact mapping table is manual and needs intentional maintenance as code taxonomy grows.
- Next:
  - add machine-readable `sec4 explain --format json` output for IDE/tooling integration.
