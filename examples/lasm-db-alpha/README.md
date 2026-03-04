# lasm-db-alpha

Larger alpha example for validating LASM server mode with real disk-backed DB behavior.

## What this example proves

- `sec4 run --backend lasm` can persist DB records to disk (`records.log`).
- DB writes survive process restarts when `--db-base` (or `SEC4_RT_LASM_DB_BASE`) is set.
- `db.queryOne` style behavior returns the latest matching record.
- You can inspect full persisted state through `/db/records`.
- `db` adapter can be switched between `records`, `sqlite`, and `postgres` without changing handlers.
- Multi-statement transaction batching via `/db/exec-batch`.
- Write/read composition in one request via `/db/write-and-query`.

## Files

- `src/main.ut`:
  - intrinsic-backed DB routes (`db.exec`, `db.execTx`, `db.queryOne`)
  - `/db/exec-batch` for tx batching
  - `/db/write-and-query` for request-local write/read composition
  - comments explain each route and expected query parameters.
- `sec4.toml`: project manifest.
- `sec4.policy`: minimal policy for local alpha runs.
- `scripts/run-smoke.sh`: deterministic operator smoke that performs writes, tx writes, query-one, and list checks.

## Run

1. Validate:

```bash
cargo run -p sec4 -- check --path examples/lasm-db-alpha
```

2. Start LASM server with explicit DB base:

```bash
DB_BASE="$(pwd)/examples/lasm-db-alpha/.lasm-db"
export SEC4_DB_ALPHA_TIMEOUT_MS=5000
cargo run -p sec4 -- run \
  --path examples/lasm-db-alpha \
  --backend lasm \
  --db-base "$DB_BASE" \
  --port 8080
```

If your app uses a different DB base directory name, set `--db-base` (or `SEC4_RT_LASM_DB_BASE`) consistently.

Adapter example:

```bash
DB_BASE="$(pwd)/examples/lasm-db-alpha/.lasm-db"
export SEC4_DB_ALPHA_DB_ADAPTER=sqlite
export SEC4_DB_ALPHA_DB_BASE="$DB_BASE"
cargo run -p sec4 -- run \
  --path examples/lasm-db-alpha \
  --backend lasm \
  --db-base "$DB_BASE" \
  --db-adapter sqlite \
  --port 8080
```

Postgres:

```bash
DB_BASE="$(pwd)/examples/lasm-db-alpha/.lasm-db"
export SEC4_DB_ALPHA_DB_ADAPTER=postgres
export SEC4_DB_ALPHA_DB_POSTGRES_DSN='postgres://user:pass@127.0.0.1:5432/sec4'
cargo run -p sec4 -- run \
  --path examples/lasm-db-alpha \
  --backend lasm \
  --db-base "$DB_BASE" \
  --db-adapter postgres \
  --port 8080
```

## Automated smoke check

Run one-command smoke from this folder:

```bash
cd examples/lasm-db-alpha
make smoke
```

Available make lanes:

- `make check` – run `sec4 check`
- `make run-records` – run with `records` adapter
- `make run-sqlite` – run with `sqlite` adapter
- `make run-postgres` – run with `postgres` adapter (requires DSN env)
- `make smoke`, `make smoke-records`, `make smoke-sqlite`, `make smoke-postgres`
- `make smoke-help` – print full script options
- `make clean-artifacts` – remove local DB artifacts under `DB_BASE`

Run the new script for deterministic DB verification:

```bash
./examples/lasm-db-alpha/scripts/run-smoke.sh
```

Run with SQLite adapter:

```bash
SEC4_DB_ALPHA_DB_ADAPTER=sqlite ./examples/lasm-db-alpha/scripts/run-smoke.sh
```

For Postgres, set a DSN and run:

```bash
export SEC4_DB_ALPHA_DB_POSTGRES_DSN='postgres://user:pass@127.0.0.1:5432/sec4'
SEC4_DB_ALPHA_DB_ADAPTER=postgres ./examples/lasm-db-alpha/scripts/run-smoke.sh
```

If your DSN must stay out of shell history, prefer DSN file mode:

```bash
export SEC4_DB_ALPHA_POSTGRES_DSN_FILE=~/.config/sec4/lasm-postgres-dsn
SEC4_DB_ALPHA_DB_ADAPTER=postgres ./examples/lasm-db-alpha/scripts/run-smoke.sh
```

If you are using `infra/local-postgres`, you can let the smoke script pick up
`infra/local-postgres/.runtime.env` automatically:

```bash
cd /path/to/AILang
infra/local-postgres/scripts/up.sh
SEC4_DB_ALPHA_DB_ADAPTER=postgres ./examples/lasm-db-alpha/scripts/run-smoke.sh
```

Script knobs:

- `SEC4_DB_ALPHA_PORT` (default `8088`)
- `SEC4_DB_ALPHA_DB_ADAPTER` (`records`, `sqlite`, `postgres`)
- `SEC4_DB_ALPHA_DB_BASE` (default `<project>/.lasm-db`)
- `SEC4_DB_ALPHA_QUERY_TEMPLATE` (URL-encoded SQL template, default `SELECT%201`)
- `SEC4_DB_ALPHA_QUERY_PARAMS` (URL-encoded JSON params, default `%5B%5D`)
- `SEC4_DB_ALPHA_QUERY_ONE_ROW_SCHEMA` (row schema id for `query-one`, default `7`)
- `SEC4_DB_ALPHA_TIMEOUT_MS` (request timeout in milliseconds, default `5000`)
- `SEC4_DB_ALPHA_DB_POSTGRES_DSN`
- `SEC4_DB_ALPHA_POSTGRES_DSN_FILE`
- `SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH`
- `SEC4_DB_ALPHA_POSTGRES_RUNTIME_ENV_FILE`
- Script flag equivalents: `--port`, `--db-base`, `--db-adapter`, `--query-template`, `--query-params`, `--query-one-row-schema`, `--serve-timeout-ms`, `--request-timeout-ms`.
- Legacy compatibility knobs are also accepted by script:
  - `SEC4_RT_LASM_DB_ADAPTER`, `SEC4_RT_LASM_DB_BASE`, `SEC4_RT_LASM_DB_PORT`,
    `SEC4_RT_LASM_DB_SERVE_TIMEOUT_MS`, `SEC4_RT_LASM_DB_TIMEOUT_MS`,
    `SEC4_RT_LASM_DB_POSTGRES_DSN`, `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE`,
    `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH`.

## Test flow (manual)

Before you run manual checks that list persisted records, send at least one write route first.

1. Append non-transactional record:

```bash
curl -i -X POST \
  'http://127.0.0.1:8080/db/exec?template=SELECT%201&params=%5B%5D'
```

2. Append transactional record:

```bash
curl -i -X POST \
  'http://127.0.0.1:8080/db/exec-tx?template=SELECT%201&params=%5B%5D'
```

3. Query latest matching record (same template/params as writes):

```bash
curl -i \
  'http://127.0.0.1:8080/db/query-one?template=SELECT%201&params=%5B%5D&row_schema=7'
```

4. Execute a batched tx write:

```bash
curl -i \
  'http://127.0.0.1:8080/db/exec-batch?template_a=INSERT%201&params_a=%5B%22alpha%22%5D&template_b=INSERT%202&params_b=%5B%22beta%22%5D'
```

5. Execute write+query composition:

```bash
curl -i -X POST \
  'http://127.0.0.1:8080/db/write-and-query?write_template=SELECT%201&write_params=%5B1%5D&query_template=SELECT%201&query_params=%5B1%5D&row_schema=7'
```

6. List all persisted records:

```bash
curl -i 'http://127.0.0.1:8080/db/records'
```

7. Inspect persisted DB file:

```bash
cat "$DB_BASE/records.log"
``` 

If `records.log` does not exist yet, either DB writes were not executed, startup failed, or the adapter is not `records`.

## Expected shape

- `/db/exec` and `/db/exec-tx` return JSON with `recordId`, `op`, `db`, `template`, `params`, `tx`.
- `/db/exec-batch` returns plain text confirming batched tx writes.
- `/db/query-one` returns latest matching `record` plus deterministic `row` text.
- `/db/write-and-query` returns query result of the post-write lookup.
- `/db/records` returns `{ "ok": true, "count": N, "records": [...] }`.

## Notes

- `template`/`params` are required for intrinsic DB routes.
- `/db/query-one` also requires numeric `row_schema`.
- Without `--db-base`/`SEC4_RT_LASM_DB_BASE`, records stay in-process only.
- For `records` adapter, `records.log` is created on server startup when the adapter is active, so it can be inspected even before writes.
- `SEC4_DB_ALPHA_TIMEOUT_MS` is interpreted as milliseconds in script docs and converted to `curl --max-time` seconds internally.
