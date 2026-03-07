# M39: LASM Postgres Active-Pool Operator Surfaces And Workbench Tuning

## What changed

- Added LASM CLI/runtime wiring for Postgres shared-client active-pool limits:
  - `--db-postgres-shared-client-max-active-per-key`
  - `--db-postgres-shared-client-max-active-total`
- Forwarded those limits through LASM cluster worker spawning so proxy/fixed cluster workers inherit the same Postgres checkout caps.
- Exposed the new active-pool settings and live active counts in:
  - LASM cluster status JSON
  - LASM DB smoke summary
  - LASM DB records response payload
- Extended workbench benchmark runners to accept and record the same active-pool settings:
  - `run_workbench_benchmark_matrix.sh`
  - `run_workbench_step_matrix.sh`
  - `run_workbench_full_benchmark_suite.sh`
  - `run_workbench_lasm_mode_compare.sh`
- Fixed a real workbench harness bug in `run_workbench_profile.sh` where wrk Lua temp-file generation could collide on a literal `XXXXXX` filename across repeated mode-compare runs.

## Why

The previous runtime slice bounded Postgres active client creation internally, but operators and benchmark lanes could not control or observe those limits cleanly. That left the canonical Postgres workload hard to tune and made cluster behavior partially opaque.

The benchmark harness bug mattered for the same reason: once tuning moved into repeated same-workload comparisons, a temporary-file collision that only appeared on later modes was enough to generate false 4xx failures and poison the comparison data.

## How it works

### Runtime / operator path

- `sec4 run` and `sec4 lasm-smoke` now accept explicit Postgres active-pool limits.
- Those values are written into the LASM runtime env:
  - `SEC4_RT_LASM_DB_POSTGRES_SHARED_CLIENT_MAX_ACTIVE_PER_KEY`
  - `SEC4_RT_LASM_DB_POSTGRES_SHARED_CLIENT_MAX_ACTIVE_TOTAL`
- Cluster workers receive the same flags during spawn, so worker runtime state stays aligned with the top-level operator command.

### Visibility

Cluster status JSON now includes:

- `dbPostgresSharedClientMaxActivePerKey`
- `dbPostgresSharedClientMaxActiveTotal`
- `dbPostgresSharedClientPoolActiveKeys`
- `dbPostgresSharedClientPoolActiveTotal`

The smoke summary and DB records endpoint expose the same active-pool picture so operators can inspect one node without a separate profiler.

### Benchmark path

The canonical workbench runners now accept:

- `--lasm-db-postgres-shared-client-max-active-per-key`
- `--lasm-db-postgres-shared-client-max-active-total`

Those values are carried into the LASM service launch and persisted into run metadata. That makes later tuning runs reproducible instead of relying on ad hoc env exports.

### Harness fix

`run_workbench_profile.sh` now renders wrk Lua scripts with a safer `mktemp` template whose random segment is always at the end of the filename. This removes the repeated-run collision path that produced a literal `XXXXXX` temp file and then broke later mode runs.

## Validation

- `cargo check -p sec4`
- `cargo build -p sec4`
- `bash -n` on:
  - `benchmark-suite/scripts/run_workbench_profile.sh`
  - `benchmark-suite/scripts/run_workbench_benchmark_matrix.sh`
  - `benchmark-suite/scripts/run_workbench_step_matrix.sh`
  - `benchmark-suite/scripts/run_workbench_full_benchmark_suite.sh`
  - `benchmark-suite/scripts/run_workbench_lasm_mode_compare.sh`
- Real LASM proxy-cluster run with Postgres + status JSON verified the new fields are emitted live.
- Real short workbench mode-compare run passed with the new active-pool flags on `wb-tasks-with-comment`.

## Result

The Postgres active-pool limits are now:

1. configurable from the real CLI
2. visible in operator surfaces
3. benchmark-tunable on the canonical DB-backed workload
4. stable under repeated mode-compare runs

## Next

- Use the new benchmark-tunable active-pool controls to tune the canonical Postgres transactional hotspot (`wb-tasks-with-comment`) on the real LASM cluster path.
- Keep the next slice focused on runtime/scaling hot-path improvements, not new subsystems.
