# postgres-e2e

Durable local example that proves LASM runtime against a real Postgres connection.

## What this proves

- LASM backend runs with `SEC4_RT_LASM_DB_ADAPTER=postgres`.
- Intrinsics execute against Postgres through runtime adapter:
  - `db.exec`
  - `db.execTx`
  - `db.queryOne`
- `DbListRecordsResponse` exposes runtime adapter + DB telemetry.

## Layout

- Shared infra: `/Users/vladimirtrifonov/src/ai/AILang/infra/local-postgres`
- Example app: `/Users/vladimirtrifonov/src/ai/AILang/examples/postgres-e2e`

## 1) Start local Postgres

```bash
/Users/vladimirtrifonov/src/ai/AILang/infra/local-postgres/scripts/up.sh
```

If `.env` is missing, it is created from `.env.example`.

## 2) Validate example compiles

```bash
cargo run -p sec4 -- check --path /Users/vladimirtrifonov/src/ai/AILang/examples/postgres-e2e
```

## 3) Run automated smoke flow

```bash
/Users/vladimirtrifonov/src/ai/AILang/examples/postgres-e2e/scripts/smoke.sh
```

Expected output ends with:

```text
postgres e2e smoke passed
```

## 4) Manual route checks (optional)

Run server manually:

```bash
set -a
source /Users/vladimirtrifonov/src/ai/AILang/infra/local-postgres/.env
set +a
export SEC4_RT_LASM_DB_ADAPTER=postgres
cargo run -p sec4 -- run \
  --path /Users/vladimirtrifonov/src/ai/AILang/examples/postgres-e2e \
  --backend lasm \
  --port 18080
```

Then call routes:

```bash
curl -i -X POST 'http://127.0.0.1:18080/db/exec?template=CREATE%20TABLE%20IF%20NOT%20EXISTS%20demo_users%20%28id%20SERIAL%20PRIMARY%20KEY%2C%20email%20TEXT%20NOT%20NULL%20UNIQUE%29&params=0'
curl -i -X POST 'http://127.0.0.1:18080/db/exec-tx?template=INSERT%20INTO%20demo_users%20%28email%29%20VALUES%20%28%241%29%20ON%20CONFLICT%20%28email%29%20DO%20NOTHING&params=%5B%22alice%40example.com%22%5D'
curl -i 'http://127.0.0.1:18080/db/query-one?template=SELECT%20id%2C%20email%20FROM%20demo_users%20WHERE%20email%20%3D%20%241&params=%5B%22alice%40example.com%22%5D&row_schema=7'
curl -i 'http://127.0.0.1:18080/db/records'
```

## Troubleshooting

- `db adapter postgres requires SEC4_RT_LASM_DB_POSTGRES_DSN to be set`
  - Ensure `.env` exists and is sourced, or export DSN explicitly.
- Postgres connect/auth failures
  - Verify container health: `docker compose -f /Users/vladimirtrifonov/src/ai/AILang/infra/local-postgres/docker-compose.yml ps`
  - Verify DSN user/password/db in `.env`.
