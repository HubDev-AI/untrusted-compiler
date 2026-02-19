# 1019 M39 Slice: `sec4 run --db-adapter` Flag for LASM DB Persistence

## What It Is

This slice adds an explicit CLI surface for LASM DB persistence adapter selection:

- `sec4 run --db-adapter records-log`
- `sec4 run --db-adapter sqlite`

The flag is LASM-only and now works in both:

- direct single-instance LASM run mode,
- LASM cluster mode worker spawn path.

## Why It Exists

Adapter selection previously required `SEC4_RT_LASM_DB_ADAPTER`, which is functional but less ergonomic for operator workflows and reproducible command-line runbooks.

Adding `--db-adapter` improves alpha usability by making adapter choice explicit in command invocations and CI scripts without losing environment fallback compatibility.

## How It Works Internally

1. CLI surface:
   - `Commands::Run` now accepts optional `db_adapter: Option<RunDbAdapter>`.
   - supported values:
     - `records-log` (aliases: `records`, `records.log`),
     - `sqlite`.

2. Backend guard:
   - `cmd_run` enforces LASM-only usage:
     - explicit C backend + `--db-adapter` returns deterministic exit code `2`,
     - stderr diagnostic: `run failed: --db-adapter is only supported with --backend lasm`.

3. Adapter resolution precedence:
   - explicit `--db-adapter` is mapped to internal `LasmDbRecordsAdapter` and passed into dynamic-state construction,
   - when flag is omitted, runtime keeps existing env-based fallback via `SEC4_RT_LASM_DB_ADAPTER`.

4. Cluster propagation:
   - `LasmClusterConfig` now carries optional adapter selection,
   - worker spawn command forwards `--db-adapter ...` so cluster workers use the same persistence adapter as the front proxy process.

5. Test updates:
   - new command test for deterministic C-backend rejection of `--db-adapter`,
   - existing SQLite intrinsic integration flow now selects adapter via `--db-adapter sqlite` instead of environment mutation.

## Inputs / Outputs and Constraints

Inputs:
- `sec4 run` invocation with optional `--db-adapter`.

Outputs:
- deterministic adapter selection in LASM runtime persistence path (`records.log` vs `records.sqlite3`).

Constraints:
- `--db-adapter` is invalid with `--backend c`,
- omitting `--db-adapter` preserves existing compatibility behavior (env fallback + default records-log adapter).

## Failure Modes and Diagnostics

- invalid backend/flag combination:
  - stderr: `run failed: --db-adapter is only supported with --backend lasm`
  - exit status: `2`.

- adapter-specific persistence/open failures retain existing LASM warnings/errors from dynamic DB state persistence code.

## Example Usage

Use SQLite persistence explicitly:

```bash
sec4 run \
  --path examples/lasm-alpha-full \
  --backend lasm \
  --db-base /tmp/sec4-lasm-db \
  --db-adapter sqlite
```

Use file-backed adapter explicitly:

```bash
sec4 run \
  --path examples/lasm-alpha-full \
  --backend lasm \
  --db-base /tmp/sec4-lasm-db \
  --db-adapter records-log
```

## Tradeoffs and Next Steps

Tradeoffs:
- introduces another LASM-only run flag, which increases CLI surface area slightly.

Next steps:
1. update canonical operator docs/runbooks to prefer explicit `--db-adapter` over env toggles for reproducibility,
2. add parity checks for adapter selection in cluster-focused benchmark/evidence workflows.
