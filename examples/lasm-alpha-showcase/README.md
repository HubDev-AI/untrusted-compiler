# lasm-alpha-showcase

Full-size LASM alpha showcase with:

- DB intrinsics (`db.exec`, `db.execTx`, `db.queryOne`)
- FS intrinsics (`fs.write`, `fs.read` with `path.base`/`path.under`)
- Net intrinsics (`httpClient.get`, `httpClient.getInternal`)
- Cookies/headers/middleware metadata
- Multi-file module layout (`use showcase_db`, `use showcase_fs`, `use showcase_net`)

This project is intended for manual alpha verification of the real runtime logic.

## Files

- `src/main.ut`:
  - wires routes to feature modules
  - exports health/meta/db/fs/net endpoints
- `src/showcase_db.ut`:
  - write / tx / queryOne / write+query composition / list routes
- `src/showcase_fs.ut`:
  - deterministic `path.base` + `path.under` flow
  - write/read roundtrip and containment-inspection helper
- `src/showcase_net.ut`:
  - public and internal get routes
  - explicit internal network path
- `scripts/smoke.sh`:
  - one-command end-to-end smoke for DB + FS + net routes

## Quick start

1. Verify compile:

```bash
cargo run -p sec4 -- check --path examples/lasm-alpha-showcase
```

2. Run with records adapter (default):

```bash
SEC4_DB_ALPHA_DB_BASE="$(pwd)/examples/lasm-alpha-showcase/.lasm-db" \
SEC4_DB_ALPHA_TIMEOUT_MS=5000 \
cargo run -p sec4 -- run \
  --path examples/lasm-alpha-showcase \
  --backend lasm \
  --db-base examples/lasm-alpha-showcase/.lasm-db \
  --db-adapter records \
  --port 8090
```

3. Exercise routes (default server at `http://127.0.0.1:8090`):

```bash
curl -i http://127.0.0.1:8090/
curl -i http://127.0.0.1:8090/status
```

### DB checks (records/sqlite)

```bash
curl -i -X POST \
  'http://127.0.0.1:8090/db/exec?template=SELECT%201&params=%5B%22alpha%22%5D'

curl -i -X POST \
  'http://127.0.0.1:8090/db/exec-tx?template=SELECT%201&params=%5B%22beta%22%5D'

curl -i -X POST \
  'http://127.0.0.1:8090/db/write-and-query?write_template=SELECT%201&write_params=%5B1%5D&query_template=SELECT%201&query_params=%5B1%5D&row_schema=7'

curl -i \
  'http://127.0.0.1:8090/db/query-one?template=SELECT%201&params=%5B1%5D&row_schema=7'

curl -i http://127.0.0.1:8090/db/list

ls -l examples/lasm-alpha-showcase/.lasm-db
cat examples/lasm-alpha-showcase/.lasm-db/records.log
```

### FS checks

```bash
curl -i 'http://127.0.0.1:8090/fs/roundtrip?file=sample.txt&value=hello'
curl -i 'http://127.0.0.1:8090/fs/inspect?file=sample.txt'
```

### Net checks

Public:

```bash
curl -i 'http://127.0.0.1:8090/net/public?url=https://example.com'
```

Internal (requires explicit runtime permission):

```bash
SEC4_RT_ALLOW_INTERNAL_NET=1 \
  cargo run -p sec4 -- run \
  --path examples/lasm-alpha-showcase \
  --backend lasm \
  --db-base examples/lasm-alpha-showcase/.lasm-db \
  --db-adapter sqlite \
  --port 8090

curl -i 'http://127.0.0.1:8090/net/internal?url=http://127.0.0.1:8090/status'
```

## Postgres adapter smoke

Set DSN first:

```bash
export SEC4_DB_ALPHA_DB_POSTGRES_DSN='postgres://user:pass@127.0.0.1:5432/sec4'
```

Then run using the same route checks, adding `--db-adapter postgres`.  
In PostgreSQL mode, there is no `records.log` file; data persists in DB tables.

Optional route smoke:

```bash
SEC4_DB_ALPHA_DB_ADAPTER=postgres \
SEC4_DB_ALPHA_DB_BASE=examples/lasm-alpha-showcase/.lasm-db \
SEC4_DB_ALPHA_TIMEOUT_MS=5000 \
cargo run -p sec4 -- run \
  --path examples/lasm-alpha-showcase \
  --backend lasm \
  --db-base examples/lasm-alpha-showcase/.lasm-db \
  --db-adapter postgres \
  --port 8090
```

## Smoke runner

```bash
./examples/lasm-alpha-showcase/scripts/smoke.sh
```

Use env overrides:

- `SEC4_LASM_SHOWCASE_PORT` (default `8090`)
- `SEC4_LASM_SHOWCASE_DB_ADAPTER` (`records`, `sqlite`, `postgres`)
- `SEC4_LASM_SHOWCASE_DB_BASE` (default `<project>/.lasm-db`)
- `SEC4_DB_ALPHA_DB_POSTGRES_DSN` / `SEC4_DB_ALPHA_POSTGRES_DSN_FILE`
- `SEC4_LASM_SHOWCASE_TIMEOUT_MS` (request timeout, milliseconds)
