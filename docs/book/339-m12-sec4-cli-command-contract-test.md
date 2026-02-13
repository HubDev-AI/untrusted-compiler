# M12 Slice: sec4 CLI Command Contract Test

This slice adds a static command-surface contract test for the locked `sec4` CLI security commands.

## What it is

Updated:
- `scripts/test-sec4-cli-command-contract.sh`
- `scripts/test-sec4-cli-command-contract-guard.sh`
- `.github/workflows/naming-lock.yml`

## Why it exists

Naming lock checks token presence, but command wiring can still drift in code:
- command removed from enum,
- legacy alias reintroduced,
- `gate` default threshold changed.

This slice enforces the executable source contract directly in CI.

## What changed

1. Added CLI command contract test
- Asserts `compiler/sec4-cli/src/main.rs` contains:
  - top-level `Audit(AuditArgs)` subcommand,
  - top-level `Gate { ... }` subcommand,
  - top-level `Explain { ... }` subcommand,
  - gate default fail threshold wiring: `risk>=HIGH`.
- Asserts legacy nested `Sec` alias patterns are absent.

2. Wired into naming-lock CI
- `naming-lock.yml` now runs:
  - `scripts/test-sec4-cli-command-contract.sh`
  - `scripts/test-sec4-cli-command-contract-guard.sh`

## Validation

```bash
scripts/test-sec4-cli-command-contract.sh
```

## Tradeoffs

- Static source-level guard, not runtime invocation test.
- Designed to be lightweight and deterministic in naming-lock CI without building the CLI binary.
