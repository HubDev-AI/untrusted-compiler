# 300 M9 Slice: `sec4 explain` Exact-Code Mappings

This chapter documents adding exact-code guidance mappings to `sec4 explain`.

## What it is

Updated:
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`

## Why it exists

`sec4 explain` already mapped diagnostic families (`E1xxx`, `E2xxx`, etc.). For frequently hit errors, users need more specific guidance and direct links without manual code-family interpretation.

## How it works internally

`explain_topic` now checks exact codes before family fallbacks:

- `E2003` -> missing capability guidance
- `E4004` -> schema contract misuse guidance
- `E5001` -> SQL template parameter guidance
- `E6001` -> internal-network policy block guidance

Each mapping includes:
- specific topic title,
- specific summary,
- existing action list reuse,
- direct canonical chapter link.

If no exact match exists, existing family-level mapping logic remains unchanged.

## Inputs, outputs, and constraints

- Input: diagnostic code passed to `sec4 explain`.
- Output: exact-code explanation when matched; otherwise family explanation.
- Constraint: mapping table is explicit/manual and must be extended as new high-frequency codes emerge.

## Failure modes and diagnostics

- Unknown code:
  - still returns successful generic guidance (`Unknown Diagnostic Family`).
- Known family but unmapped exact code:
  - continues to use family-level mapping.

## Example usage

```bash
sec4 explain E2003
sec4 explain E4004
sec4 explain E5001
sec4 explain E6001
```

## Tradeoffs and next steps

- Tradeoff:
  - explicit mappings require maintenance when taxonomy evolves.
- Next:
  - add exact mappings for additional high-frequency codes from CI/test telemetry.
