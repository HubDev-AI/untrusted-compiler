# 03 - Real Postgres with LASM

This track is the canonical alpha path for a non-trivial LASM server with a real Postgres DB.

Use this in order:

1. canonical app/operator README:
   - `benchmark-suite/services/sec4-lasm-workbench/README.md`
2. canonical smoke contracts:
   - `benchmark-suite/services/sec4-lasm-workbench/smoke.sh`
   - `benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh`
3. guided build-from-scratch tutorial (supplementary):
   - `docs/book/1552-beginner-zed-plugin-postgres-lasm-tutorial.md`

## Quick run (canonical workbench)

```bash
infra/local-postgres/scripts/up.sh

set -a
if [ -f infra/local-postgres/.runtime.env ]; then
  source infra/local-postgres/.runtime.env
else
  source infra/local-postgres/.env
fi
set +a

cargo run -p sec4 -- check --path benchmark-suite/services/sec4-lasm-workbench

cargo run -p sec4 -- run \
  --path benchmark-suite/services/sec4-lasm-workbench \
  --backend lasm \
  --db-adapter postgres \
  --db-postgres-dsn "$SEC4_RT_LASM_DB_POSTGRES_DSN" \
  --port 18088
```

In another terminal:

```bash
benchmark-suite/services/sec4-lasm-workbench/smoke.sh
benchmark-suite/services/sec4-lasm-workbench/smoke-public.sh
```

Direct SQL verification:

```bash
psql "$SEC4_RT_LASM_DB_POSTGRES_DSN" -c "select id, title, status, priority from wb_tasks order by created_at_ms desc limit 10;"
psql "$SEC4_RT_LASM_DB_POSTGRES_DSN" -c "select id, task_id, body from wb_comments order by created_at_ms desc limit 10;"
```

## Adapter behavior summary

- `records`: writes runtime metadata to `<db-base>/records.log`
- `sqlite`: writes runtime metadata to `<db-base>/records.sqlite3`
- `postgres`: uses live Postgres tables (no local `records.log`)

For the canonical workbench app, use `postgres` only.

## Common runtime flags

- `--backend lasm`
- `--db-adapter records|sqlite|postgres`
- `--db-base <path>`
- `--db-postgres-dsn <dsn>`
- `--db-postgres-dsn-file <path>`
- `--max-keep-alive-requests <n>`

For benchmark-heavy runs, use a higher keep-alive budget (example: `4096`) to reduce reconnect churn:

```bash
cargo run -p sec4 -- run \
  --path benchmark-suite/services/sec4-lasm-workbench \
  --backend lasm \
  --db-adapter postgres \
  --db-postgres-dsn "$SEC4_RT_LASM_DB_POSTGRES_DSN" \
  --max-keep-alive-requests 4096 \
  --port 18088
```

## Next step

- Continue to `04-example-catalog.md`.
