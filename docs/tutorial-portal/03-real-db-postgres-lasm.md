# 03 - Real Postgres with LASM

This track is the primary alpha path for a non-trivial server with a real DB connection.

Use this in order:

1. full guided tutorial chapter:
   - `docs/book/1552-beginner-zed-plugin-postgres-lasm-tutorial.md`
2. focused Postgres e2e sample:
   - `examples/postgres-e2e/README.md`
3. full alpha app with middleware + users + DB routes:
   - `examples/lasm-alpha-full/README.md`

## Quick run (postgres-e2e)

```bash
infra/local-postgres/scripts/up.sh

set -a
if [ -f infra/local-postgres/.runtime.env ]; then
  source infra/local-postgres/.runtime.env
else
  source infra/local-postgres/.env
fi
set +a

cargo run -p sec4 -- check --path examples/postgres-e2e

cargo run -p sec4 -- run \
  --path examples/postgres-e2e \
  --backend lasm \
  --db-adapter postgres \
  --db-postgres-dsn "$SEC4_RT_LASM_DB_POSTGRES_DSN" \
  --port 18080
```

In another terminal:

```bash
curl -i -X POST 'http://127.0.0.1:18080/db/exec?template=CREATE%20TABLE%20IF%20NOT%20EXISTS%20demo_users%20%28id%20SERIAL%20PRIMARY%20KEY%2C%20email%20TEXT%20NOT%20NULL%20UNIQUE%29&params=0'
curl -i -X POST 'http://127.0.0.1:18080/db/exec-tx?template=INSERT%20INTO%20demo_users%20%28email%29%20VALUES%20%28%241%29%20ON%20CONFLICT%20%28email%29%20DO%20NOTHING&params=%5B%22alice%40example.com%22%5D'
curl -i 'http://127.0.0.1:18080/db/query-one?template=SELECT%20id%2C%20email%20FROM%20demo_users%20WHERE%20email%20%3D%20%241&params=%5B%22alice%40example.com%22%5D&row_schema=7'
curl -i 'http://127.0.0.1:18080/db/records'
```

Direct SQL verification:

```bash
psql "$SEC4_RT_LASM_DB_POSTGRES_DSN" -c "select id, email from demo_users order by id desc limit 10;"
```

## Adapter behavior summary

- `records`: writes runtime metadata to `<db-base>/records.log`
- `sqlite`: writes runtime metadata to `<db-base>/records.sqlite3`
- `postgres`: uses live Postgres tables (no local `records.log`)

## Common runtime flags

- `--backend lasm`
- `--db-adapter records|sqlite|postgres`
- `--db-base <path>`
- `--db-postgres-dsn <dsn>`
- `--db-postgres-dsn-file <path>`

## Next step

- Continue to `04-example-catalog.md`.
