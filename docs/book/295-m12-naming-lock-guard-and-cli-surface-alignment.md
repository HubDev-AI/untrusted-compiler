# 295 M12 Slice: Naming-Lock Guard and CLI Surface Alignment

This chapter documents the first enforcement slice for M12 naming alignment.

## What it is

Added:
- `scripts/check-naming-lock.sh`

Updated:
- `scripts/release-alpha-gate.sh`
- `compiler/sec4-cli/src/main.rs`
- `compiler/sec4-cli/tests/json_output.rs`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/210-release-notes-and-compatibility.md`

## Why it exists

The naming contract is now part of product compatibility:
- language/docs: `Untrusted<T>`
- tooling command surface: `sec4 audit`, `sec4 explain`, `sec4 gate`
- source extension: `.ut`
- namespace contract: `ut/std`, `ut/http`, `ut/sec`

Without an automated guard, naming drift can reappear in docs, scripts, and test fixtures.

## How it works internally

1. `scripts/check-naming-lock.sh` scans tracked source/docs paths for banned legacy patterns:
   - legacy language/extension identifiers, old editor/server package names, and legacy nested command form.
2. The same script asserts required contract tokens are present:
   - `Untrusted<T>`, `.ut`, canonical `sec4` security commands, and namespace strings.
3. `scripts/release-alpha-gate.sh` now runs naming-lock validation before release artifact checks.
4. CLI removed the legacy nested `sec` subcommand path, leaving canonical top-level security commands.
5. CLI integration tests were migrated from `["sec", "audit", ...]` to `["audit", ...]`.

## Inputs, outputs, and constraints

- Input: repository text surface (docs, compiler, runtime, scripts, examples, benchmark harness, editor scaffolds).
- Output: deterministic pass/fail for naming compliance.
- Constraint: guard is regex-based and should be extended carefully if new allowed terms overlap banned patterns.

## Failure modes and diagnostics

- If legacy naming appears, the script prints:
  - exact pattern that failed,
  - file+line matches.
- If canonical tokens disappear, the script fails with explicit missing-contract messages.

## Example usage

```bash
scripts/check-naming-lock.sh
```

Run through release gate:

```bash
scripts/release-alpha-gate.sh --skip-tests
```

## Tradeoffs and next steps

- Tradeoff:
  - regex-based checks are intentionally simple and may need pattern tuning as docs evolve.
- Next:
  - add benchmark artifact filename/schema assertions once M10 output contracts are frozen.
