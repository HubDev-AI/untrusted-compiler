# 04 - Example Catalog (Current Alpha)

This page maps every active example to its purpose and best use case.

## Summary table

| Example | Purpose | DB mode | Notes |
|---|---|---|---|
| `examples/hello` | Minimal language smoke | none | fastest compile sanity |
| `examples/hello-api` | minimal HTTP route flow | none | baseline server path |
| `examples/showcase-api` | typed sinks + DB/FS/NET route coverage | records/FS/NET runtime | backend-agnostic showcase |
| `examples/zed-plugin-smoke` | plugin/LSP smoke project | n/a | diagnostics/format/nav fixture |
| `benchmark-suite/services/sec4-lasm-workbench` | canonical LASM alpha app (tutorial/operator/benchmark contract) | postgres | primary operator and benchmark proof path |
| `examples/lasm-db-alpha` | LASM DB adapter validation | records/sqlite/postgres | focused DB alpha sample |
| `examples/postgres-e2e` | LASM + real Postgres end-to-end | postgres | strict DB reality check |
| `examples/lasm-alpha-full` | full feature reference app (middleware, users, DB) | records/sqlite/postgres | supplementary operator sample (non-canonical) |
| `examples/lasm-alpha-showcase` | multi-file showcase with db/fs/net | records/sqlite/postgres | module layout + runtime behavior |
| `examples/lasm-workbench` | legacy/manual workbench variant | records/sqlite/postgres | keep for local experiments; canonical flow is under benchmark-suite |

## Recommended usage order

1. `examples/hello-api` for first run.
2. `examples/zed-plugin-smoke` for IDE workflow.
3. `benchmark-suite/services/sec4-lasm-workbench` for canonical real DB/operator proof.
4. `examples/postgres-e2e` for focused DB sanity checks.
5. `examples/lasm-alpha-full` for full feature/reference behavior.

## Command quick refs

### hello-api

```bash
cargo run -p sec4 -- check --path examples/hello-api
cargo run -p sec4 -- run --path examples/hello-api --backend lasm --port 8080
```

### zed-plugin-smoke

```bash
sec4 check --path examples/zed-plugin-smoke
sec4 fmt --path examples/zed-plugin-smoke
scripts/run-zed-plugin-smoke.sh --fast
```

### sec4-lasm-workbench (canonical)

```bash
cargo run -p sec4 -- check --path benchmark-suite/services/sec4-lasm-workbench
benchmark-suite/services/sec4-lasm-workbench/smoke.sh
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
```

### postgres-e2e

```bash
infra/local-postgres/scripts/up.sh
set -a; if [ -f infra/local-postgres/.runtime.env ]; then source infra/local-postgres/.runtime.env; else source infra/local-postgres/.env; fi; set +a
examples/postgres-e2e/scripts/smoke.sh
```

### lasm-alpha-full

```bash
cd examples/lasm-alpha-full
make check
make run-records
# or: make run-sqlite / make run-postgres
make smoke
```

### lasm-workbench (legacy/manual)

```bash
cd examples/lasm-workbench
./scripts/run-workbench.sh
./scripts/run-smoke.sh --db-adapter records
```

## Deep references

- `examples/lasm-alpha-full/README.md`
- `examples/lasm-alpha-showcase/README.md`
- `examples/lasm-db-alpha/README.md`
- `examples/lasm-workbench/README.md`
- `examples/postgres-e2e/README.md`
- `examples/showcase-api/README.md`
- `examples/zed-plugin-smoke/README.md`
- `benchmark-suite/services/sec4-lasm-workbench/README.md`

## Next step

- Continue to `05-benchmarks-capacity.md`.
