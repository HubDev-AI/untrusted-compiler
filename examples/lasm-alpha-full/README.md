# lasm-alpha-full

Full LASM alpha example app that combines:

- HTTP + middleware runtime behavior
- request placeholder materialization
- auth helper enforcement
- dynamic user-store flows (`users.json`)
- dynamic DB record flows (`records.log`, `sqlite`, `postgres`)
- DB batch and read-after-write composition (`db.exec-batch`, `db.write-and-query`)

Use this app to test LASM + DB behavior end-to-end.

## 1) Start

Default adapter (`records.log`):

```bash
DB_BASE="$(pwd)/examples/lasm-alpha-full/.lasm-db"
cargo run -p sec4 -- run \
  --path examples/lasm-alpha-full \
  --backend lasm \
  --db-base "$DB_BASE" \
  --port 8080
```

Postgres adapter:

```bash
DB_BASE="$(pwd)/examples/lasm-alpha-full/.lasm-db"
POSTGRES_DSN='postgres://user:pass@127.0.0.1:5432/postgres'
cargo run -p sec4 -- run \
  --path examples/lasm-alpha-full \
  --backend lasm \
  --db-base "$DB_BASE" \
  --db-postgres-dsn "$POSTGRES_DSN" \
  --db-adapter postgres \
  --port 8080
```

If you prefer the convenience lanes, use the local Makefile:

```bash
cd examples/lasm-alpha-full
make check
make run-records      # default records adapter
make run-sqlite
make run-postgres     # requires DSN env
make smoke            # full DB alpha smoke + persistence check
make smoke-sqlite
make smoke-postgres   # requires DSN env
```

## 2) Quick smoke

```bash
curl -i -H 'Authorization: Bearer token123' http://127.0.0.1:8080/health
curl -i -H 'Authorization: Bearer token123' http://127.0.0.1:8080/metadata
curl -i -H 'Authorization: Bearer token123' 'http://127.0.0.1:8080/preview?raw=%3Cscript%3Ealert(1)%3C%2Fscript%3E'
```

## 3) Placeholder + typed sink flow

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  -H 'X-Request-Id: req-42' \
  -H 'Cookie: session=s1' \
  'http://127.0.0.1:8080/echo/42?q=hello&header_name=X-Flow&header_value=active&cookie_name=flow-cookie&cookie_value=on'
```

## 4) Auth helper enforcement

Unauthorized:

```bash
curl -i http://127.0.0.1:8080/auth/required
```

Authorized:

```bash
curl -i -H 'Authorization: Bearer token123' http://127.0.0.1:8080/auth/required
```

Role required:

```bash
curl -i -H 'Authorization: Bearer role=admin token123' http://127.0.0.1:8080/auth/admin
```

## 5) Dynamic users store (`users.json`)

Use one deterministic user id:

```bash
USER_ID='123e4567-e89b-42d3-a456-426614174000'
CREATE_PAYLOAD='{"id":"'"$USER_ID"'","email":"user@example.com","age":30,"tags":["core"],"address":{"zip":"12345"},"meta":{"flags":{"a":true,"b":false,"c":true}}}'
UPDATE_PAYLOAD='{"id":"'"$USER_ID"'","email":"updated@example.com","age":31,"tags":["core","updated"],"address":{"zip":"67890"},"meta":{"flags":{"a":false,"b":true,"c":false}}}'
```

Create:

```bash
curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  -H 'Content-Type: application/json' \
  --data "$CREATE_PAYLOAD" \
  http://127.0.0.1:8080/users
```

Read/list/by-email:

```bash
curl -i -H 'Authorization: Bearer token123' "http://127.0.0.1:8080/users/$USER_ID"
curl -i -H 'Authorization: Bearer token123' "http://127.0.0.1:8080/users/by-email?email=updated%40example.com"
curl -i -H 'Authorization: Bearer token123' "http://127.0.0.1:8080/users"
```

Update/delete:

```bash
curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  -H 'Content-Type: application/json' \
  --data "$UPDATE_PAYLOAD" \
  "http://127.0.0.1:8080/users/$USER_ID/update"

curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  "http://127.0.0.1:8080/users/$USER_ID/delete"
```

Inspect persisted users file:

```bash
cat "$DB_BASE/users.json"
```

## 6) Dynamic DB flows (`records.log` / `sqlite` / `postgres`)

Write non-tx record:

```bash
curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/db/exec?template=SELECT%201&params=%5B%5D'
```

Write tx record:

```bash
curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/db/exec-tx?template=SELECT%201&params=%5B%22alpha%22%5D'
```

Execute two tx writes in one transaction:

```bash
curl -i -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/db/exec-batch?template_a=SELECT%201&params_a=%5B%22alpha%22%5D&template_b=SELECT%202&params_b=%5B%22beta%22%5D'
```

Execute write+query in one request:

```bash
curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/db/write-and-query?write_template=SELECT%201&write_params=%5B1%5D&query_template=SELECT%201&query_params=%5B1%5D&row_schema=7'
```

Query latest matching record:

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/db/query-one?template=SELECT%201&params=%5B%5D&row_schema=7'
```

Postgres parameterized query demo (`$N` placeholders + JSON-array params):

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/db/query-one?template=SELECT%20%24%24%242-dollar%24%24%20AS%20dollar_literal,%20%27$1-literal%27%20AS%20literal,%20$1::int%20AS%20value,%20$2::boolean%20AS%20enabled,%20$3::double%20precision%20AS%20ratio&params=%5B42%2Ctrue%2C3.25%5D&row_schema=7'
```

Postgres deterministic placeholder-arity failure demo:

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/db/query-one?template=SELECT%20$2::int%20AS%20value&params=%5B42%5D&row_schema=7'
```

List records:

```bash
curl -i -H 'Authorization: Bearer token123' http://127.0.0.1:8080/db/records
```

Inspect persisted records file for the records adapter:

```bash
cat "$DB_BASE/records.log"
```

For sqlite:

```bash
ls -l "$DB_BASE/records.sqlite3"
```

## 7) Validate/Build

```bash
cargo run -p sec4 -- check --path examples/lasm-alpha-full
cargo run -p sec4 -- build --path examples/lasm-alpha-full --emit lasm
```
For `records` adapter, inspect local persistence artifacts:

```bash
cat "$DB_BASE/records.log"
```

When using SQLite or Postgres, inspect storage with adapter tools/CLI instead of `records.log`.

## 8) Full smoke check

Run one command smoke flow:

```bash
cd examples/lasm-alpha-full
./scripts/run-smoke.sh
```

Smoke options:

- `SEC4_ALPHA_FULL_PORT` (default `8088`)
- `SEC4_ALPHA_FULL_DB_ADAPTER` (`records`, `records.log`, `records-log`, `records_log`, `sqlite`, `postgres`)
- `SEC4_ALPHA_FULL_DB_BASE` (default `<project>/.lasm-db`)
- `SEC4_ALPHA_FULL_AUTH_HEADER` (default `Authorization: Bearer token123`)
- `SEC4_ALPHA_FULL_QUERY_TEMPLATE` (default `SELECT%201`)
- `SEC4_ALPHA_FULL_QUERY_PARAMS` (default `%5B%5D`)
- `SEC4_ALPHA_FULL_QUERY_ONE_ROW_SCHEMA` (default `7`)
- `SEC4_ALPHA_FULL_TIMEOUT_MS` (request timeout, default `5000`)
- `SEC4_ALPHA_FULL_POSTGRES_DSN` (or `SEC4_RT_LASM_DB_POSTGRES_DSN`)
- `SEC4_ALPHA_FULL_POSTGRES_DSN_FILE` / `SEC4_ALPHA_FULL_POSTGRES_DSN_FILE_PATH` (or `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE` / `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE_PATH`)
- `SEC4_ALPHA_FULL_POSTGRES_RUNTIME_ENV_FILE` (or `SEC4_RT_LASM_DB_POSTGRES_RUNTIME_ENV_FILE` / `SEC4_RT_LASM_DB_POSTGRES_RUNTIME_DSN_FILE`)

For postgres tests:

```bash
export SEC4_ALPHA_FULL_DB_ADAPTER=postgres
export SEC4_ALPHA_FULL_POSTGRES_DSN='postgres://user:pass@127.0.0.1:5432/postgres'
./scripts/run-smoke.sh
```
