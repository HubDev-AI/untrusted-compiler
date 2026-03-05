# showcase-api

A larger Untrusted<T> example that is both:

- runnable end-to-end over HTTP (`sec4 run`), and
- compile-checked for typed DB/FS/NET capability usage in `capabilityShowcase(...)`.

## What this example demonstrates

- Router + middleware chain:
  - `sec.defaultHeaders`, `cors.fromPolicy`, `csrf.fromPolicy`, `auth.fromPolicy`
  - `sec.withSecurityHeaders`, `cors.withCors`, `csrf.withCsrf`, `auth.withAuth`
- Request sources:
  - `req.query`, `req.pathParam`, `req.header`, `req.json`
  - query/path param extraction decodes standard percent-encoded values (`%XX`) and `+` spaces
- Typed gates/sanitizers:
  - `sanitize.html`, `validate.email`, `validate.uuid`, `validate.int64`, `validate.headerValue`, `validate.nonEmpty`
  - `url.public`, `url.internal`, `path.base`, `path.under`
- Typed response sinks:
  - `res.text`, `res.html`, `res.ok`, `res.setHeader`, `res.addCookie`
- Typed constructors:
  - `headers.name`, `headers.value`, `cookie.build`
- Runtime capability constructors:
  - `DbCap`, `FsCap`, `NetCap`, `InternalNetCap`, `SecretsCap`, `Ctx`
- Live capability-backed routes:
  - `/db/write` (`DbCap`)
  - `/fs/write-read` (`FsCap`)
  - `/net/public` (`NetCap`)
  - `/net/internal` (`InternalNetCap`)
- Compile-checked capability slice:
  - `sql.q`, `db.tx`, `db.exec`, `db.execTx`, `db.queryOne`, `fs.read`, `fs.write`, `httpClient.get`, `httpClient.getInternal`

## Quick start

1. Validate project semantics:

```bash
cargo run -p sec4 -- check --path examples/showcase-api
```

2. Build native binary through C backend:

```bash
cargo run -p sec4 -- build --path examples/showcase-api --emit c-bin
```

3. Run HTTP server:

```bash
cargo run -p sec4 -- run --path examples/showcase-api --port 8080
```

Use `--oneshot` when you want exactly one request then deterministic exit:

```bash
cargo run -p sec4 -- run --path examples/showcase-api --oneshot --port 8080
```

## Route checks (manual)

`GET /health`

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  http://127.0.0.1:8080/health
```

`GET /headers` (typed header + cookie sinks)

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  http://127.0.0.1:8080/headers
```

`GET /preview` (untrusted -> `sanitize.html` -> `res.html`)

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/preview?raw=%3Cscript%3Ealert(1)%3C%2Fscript%3E'
```

`GET /users/:id` (path param + header source)

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  -H 'X-Request-Id: req-42' \
  http://127.0.0.1:8080/users/42
```

`GET /gates` (typed gate coverage)

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/gates?public_url=http://example.com&internal_url=http://127.0.0.1/svc&file=note.txt&count=123&uuid=123e4567-e89b-12d3-a456-426614174000&header=x-demo&name=alice'
```

`GET /db/write` (live DB write path)

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  http://127.0.0.1:8080/db/write
```

`GET /fs/write-read` (live FS write + read path)

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/fs/write-read?file=note.txt&value=hello-showcase'
```

`GET /net/public` (live outbound public GET path)

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/net/public?url=http://example.com'
```

`GET /net/internal` (live outbound internal GET path)

Requires `SEC4_RT_ALLOW_INTERNAL_NET=1` and a local HTTP target:

```bash
tmp_dir="$(mktemp -d)"
printf 'ok' > "${tmp_dir}/ok.txt"
python3 -m http.server 18081 --bind 127.0.0.1 --directory "${tmp_dir}"
```

```bash
SEC4_RT_ALLOW_INTERNAL_NET=1 \
  cargo run -p sec4 -- run --path examples/showcase-api --port 8080
```

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:8080/net/internal?url=http://127.0.0.1:18081/ok.txt'
```

`POST /users` (`req.json` + `validate.email` + `res.ok`)

```bash
curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  -H 'Content-Type: application/json' \
  --data '{}' \
  'http://127.0.0.1:8080/users?email=user@example.com'
```

## Database note

`/db/write` executes real runtime DB writes.

To force deterministic output location for inspection:

```bash
SEC4_RT_DB_BASE=/tmp/sec4-showcase-db \
  cargo run -p sec4 -- run --path examples/showcase-api --port 8080
```

Then call `/db/write` and inspect:

```bash
cat /tmp/sec4-showcase-db/records.log
```

FS probe writes under `SEC4_RT_FS_BASE` too. The probe path is `/tmp/sec4-showcase/...`,
so set runtime base to `/` for cross-platform prefix checks:

```bash
SEC4_RT_FS_BASE=/ \
  cargo run -p sec4 -- run --path examples/showcase-api --port 8080
```

Then call `/fs/write-read?...` and inspect:

```bash
cat /tmp/sec4-showcase/note.txt
```

`capabilityShowcase(...)` still includes additional compile-checked DB/FS/NET intrinsic usage.

Current bridge limitation: live HTTP route handlers are zero-arg (`fn handler()`) and do not yet receive typed capability values (`DbCap`, `FsCap`, `NetCap`, `InternalNetCap`) as handler parameters.
Runtime capability constructors unblock live route usage (for example `/db/write`) while full typed handler injection is still pending.

## Optional runtime integration checks from this repo

These tests exercise DB/FS/NET runtime behavior directly:

```bash
cargo test -p sec4 --test json_output c_bin_runtime_db_exec_query_one_roundtrip_returns_tracked_body_when_clang_available
cargo test -p sec4 --test json_output c_bin_runtime_fs_write_read_roundtrip_when_clang_available
cargo test -p sec4 --test json_output c_bin_runtime_internal_get_roundtrip_succeeds_with_env_override_when_clang_available
```
