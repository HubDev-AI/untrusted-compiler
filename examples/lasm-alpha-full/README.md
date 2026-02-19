# lasm-alpha-full

Full LASM alpha example app that combines:

- HTTP + middleware runtime behavior
- request placeholder materialization
- auth helper enforcement
- dynamic user-store flows (`users.json`)
- dynamic DB record flows (`records.log`)

Use this app to test LASM + DB behavior end-to-end.

## 1) Start

```bash
DB_BASE="$(pwd)/examples/lasm-alpha-full/.lasm-db"
cargo run -p sec4 -- run \
  --path examples/lasm-alpha-full \
  --backend lasm \
  --db-base "$DB_BASE" \
  --port 8080
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

## 6) Dynamic DB flows (`records.log`)

Write non-tx record:

```bash
curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/db/exec?template=SELECT%201&params=alpha&db=1'
```

Write tx record:

```bash
curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/db/exec-tx?template=SELECT%201&params=alpha&db=1&tx=9'
```

Query latest matching record:

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/db/query-one?template=SELECT%201&params=alpha&db=1&row_schema=7'
```

List records:

```bash
curl -i -H 'Authorization: Bearer token123' http://127.0.0.1:8080/db/records
```

Inspect persisted records file:

```bash
cat "$DB_BASE/records.log"
```

## 7) Validate/Build

```bash
cargo run -p sec4 -- check --path examples/lasm-alpha-full
cargo run -p sec4 -- build --path examples/lasm-alpha-full --emit lasm
```
