# lasm-workbench

A larger end-to-end LASM sample for manual verification and operator smoke.

This app is intentionally explicit and multi-file (`use workbench_*`) so you can
validate module resolution + runtime behavior together.

## What this sample proves

- LASM runtime execution path compiles and serves routes.
- `DbCap` + `sql.q` + `db.exec` / `db.execTx` / `db.queryOne` / `DbListRecordsResponse` materialization.
- `FsCap` + `path.base` / `path.under` + live `fs.write`/`fs.read` roundtrip.
- `NetCap` + `url.public` / `httpClient.get`.
- `InternalNetCap` + `url.internal` / `httpClient.getInternal`.
- Multi-file import path (`use workbench_db;`, `use workbench_fs;`, `use workbench_net;`).
- Typed gate/proxy checks in one route (`/gates`).

## Start

```bash
cd $REPO_ROOT/examples/lasm-workbench
sec4 check
sec4 build --emit c-bin
```

Run locally:

```bash
./scripts/run-workbench.sh
```

## Smoke checks

```bash
./scripts/run-smoke.sh --db-adapter records
./scripts/run-smoke.sh --skip-network --db-adapter records
```

For PostgreSQL-backed smoke, set one of:

```bash
SEC4_DB_ALPHA_DB_POSTGRES_DSN='postgres://user:pass@127.0.0.1:5432/sec4_lasm'
# or legacy alias SEC4_RT_LASM_DB_POSTGRES_DSN='postgres://user:pass@127.0.0.1:5432/sec4_lasm'
./scripts/run-smoke.sh --db-adapter postgres
```

Network endpoints in smoke mode are local to the script:

- public/internal routes target a local helper server on `SEC4_LASM_WORKBENCH_NETWORK_PORT` (default `19090`).
- set `--skip-network` if local HTTP helper is not available.

## DB adapters

The script sets adapter behavior via `SEC4_LASM_WORKBENCH_DB_ADAPTER`:

- `records` (default): file-backed adapter storing `records.log`
- `sqlite`: sqlite adapter storing `records.sqlite3`
- `postgres`: Postgres adapter using DSN in `SEC4_DB_ALPHA_DB_POSTGRES_DSN` (legacy `SEC4_RT_LASM_DB_POSTGRES_DSN`) or file variants

```bash
SEC4_LASM_WORKBENCH_DB_ADAPTER=records ./scripts/run-workbench.sh
SEC4_LASM_WORKBENCH_DB_ADAPTER=sqlite  SEC4_LASM_WORKBENCH_DB_BASE=/tmp/lasm-workbench-db ./scripts/run-workbench.sh
SEC4_LASM_WORKBENCH_DB_ADAPTER=postgres SEC4_DB_ALPHA_DB_POSTGRES_DSN='postgres://user:pass@127.0.0.1:5432/sec4_lasm' ./scripts/run-workbench.sh
```

Optional env:

- `SEC4_LASM_WORKBENCH_PORT` (default `8080`)
- `SEC4_LASM_WORKBENCH_DB_BASE` (default `$PROJECT/.lasm-workbench-db`)
- `SEC4_DB_ALPHA_POSTGRES_DSN_FILE` or `SEC4_DB_ALPHA_POSTGRES_DSN_FILE_PATH`
- `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE` or `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH`

## Example checks

Assuming server is running on `http://127.0.0.1:8080`:

```bash
curl -i 'http://127.0.0.1:8080/'
curl -i -H 'Authorization: Bearer token123' 'http://127.0.0.1:8080/meta'
curl -i -H 'X-Request-Id: req-42' 'http://127.0.0.1:8080/echo/jane?user=jane'
```

### DB checks (records adapter default)

```bash
curl -i 'http://127.0.0.1:8080/db/write?template=SELECT%201&params=[]'
curl -i 'http://127.0.0.1:8080/db/write-tx?template=SELECT%201&params=[]'
curl -i 'http://127.0.0.1:8080/db/write-tx-batch'
curl -i 'http://127.0.0.1:8080/db/list'
```

Query example:

```bash
curl -i "http://127.0.0.1:8080/db/query?template=SELECT%20id%2C%20email%20FROM%20users%20WHERE%20id%20%3D%20%241&params=%5B1%5D&row_schema=7"
```

### FS + gate checks

```bash
curl -i 'http://127.0.0.1:8080/fs/roundtrip?file=demo.txt&value=hello'

curl -i \
  --get 'http://127.0.0.1:8080/gates' \
  --data-urlencode 'file=demo.txt' \
  --data-urlencode 'public_url=https://example.com' \
  --data-urlencode 'internal_url=http://127.0.0.1/svc' \
  --data-urlencode 'port_hint=8080' \
  --data-urlencode 'request_id=123e4567-e89b-12d3-a456-426614174000'
```

### Network checks

```bash
curl -i 'http://127.0.0.1:8080/net/public?url=https://example.com'
```

Internal net requires `SEC4_RT_ALLOW_INTERNAL_NET=1` in runtime env:

```bash
SEC4_RT_ALLOW_INTERNAL_NET=1 ./scripts/run-workbench.sh

curl -i 'http://127.0.0.1:8080/net/internal?url=http://127.0.0.1:18080/internal'
```

## Notes

- Internal net calls may be blocked by policy unless `SEC4_RT_ALLOW_INTERNAL_NET=1`.
- If you change working data, restart is optional; adapters persist through the selected base directory.
- In `records` mode you should see `.lasm-workbench-db/records.log`.
- In `sqlite` mode you should see `.lasm-workbench-db/records.sqlite3`.
- In `postgres` mode there is no local persisted record file (metadata is in DB).
