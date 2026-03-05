# 1552 - Beginner Tutorial: Zed + Untrusted<T> Plugin + Real Postgres LASM Server

This tutorial is written for someone with zero context.

You will do all of this, step by step:

1. open the IDE (Zed)
2. install the Untrusted<T> plugin
3. create a project from scratch
4. replace starter code with a real multi-route server
5. run it on LASM with a real Postgres database (not text-file DB)
6. verify endpoints with `curl`

Everything below is copy/paste runnable.

---

## 0) Prerequisites

- macOS or Linux
- `git`, `curl`, `cargo` (Rust toolchain)
- Docker + Docker Compose plugin
- Zed editor installed

Optional but useful:

- `jq`
- `psql`

---

## 1) Clone and build tools

From a terminal:

```bash
git clone https://github.com/HubDev-AI/untrusted-compiler.git
cd untrusted-compiler

cargo build -p sec4 -p sec4audit-language-server
```

Verify binaries:

```bash
./target/debug/sec4 --help | head
./target/debug/sec4audit-language-server --help | head
```

---

## 2) Open the IDE (Zed)

1. Open Zed.
2. `File -> Open Folder...`
3. Select your cloned repo folder (`untrusted-compiler`).
4. Open Zed terminal panel (`Terminal -> New Terminal`) so all commands run from repo root.

---

## 3) Install the plugin (local install)

Install into your real Zed extension directory:

```bash
scripts/manage-zed-extension-local.sh status
scripts/manage-zed-extension-local.sh install
scripts/manage-zed-extension-local.sh status
```

If you want live local extension edits (symlink mode):

```bash
scripts/manage-zed-extension-local.sh install --mode symlink
```

Run smoke checks for plugin + language server:

```bash
scripts/run-zed-plugin-smoke.sh --fast
```

---

## 4) Create a brand-new project

Create the tutorial app in `/tmp` first:

```bash
APP_DIR=/tmp/sec4-book-tutorial
rm -rf "$APP_DIR"
./target/debug/sec4 init --path "$APP_DIR" --name sec4_book_tutorial
```

You now have:

- `sec4.toml`
- `sec4.policy`
- `src/main.ut`

---

## 5) Configure policy for real Postgres + full HTTP middleware behavior

Replace `sec4.policy`:

```bash
cat > "$APP_DIR/sec4.policy" <<'POLICY'
[policy]
name = "sec4-book-tutorial"
version = "0.1"
mode = "enforce"
env = "dev"

[json]
max_bytes = 4096
max_depth = 16

[http]
max_body_bytes = 262144
max_header_bytes = 32768
default_timeout_ms = 5000
max_runtime_steps = 65536
max_concurrency = 256

[net.public]
allowed_schemes = ["http", "https"]
allowed_domains = []
blocked_domains = []
allowed_ports = [80, 443]
allow_redirects = false
max_redirects = 0

[net.internal]
enabled = true
allowed_cidrs = []
allowed_domains = []

[fs]
enabled = true

[cors]
enabled = true
allowed_origins = ["*"]
allowed_methods = ["GET", "POST", "OPTIONS"]
allowed_headers = ["authorization", "content-type", "x-csrf-token", "x-request-id", "x-role"]
exposed_headers = ["x-trace-id", "x-showcase", "x-echo-trace", "x-flow"]
allow_credentials = false
max_age_seconds = 600
forbid_any_origin = false
forbid_reflect_origin = false
require_vary_origin = false

[security_headers]
enabled = true
x_content_type_options = true
x_frame_options = "SAMEORIGIN"
referrer_policy = "no-referrer"

[security_headers.hsts]
enabled = false
max_age_seconds = 0
include_subdomains = false
preload = false

[security_headers.csp]
enabled = true
report_only = false

[csrf]
enabled = false
mode = "off"
cookie_name = "csrf"
header_name = "x-csrf-token"
same_site = "Lax"
secure_cookie = false
http_only_cookie = false
protected_methods = ["POST", "PUT", "PATCH", "DELETE"]

[auth]
mode = "token"
cross_site_frontend = false
POLICY
```

---

## 6) Create a complex server (line-by-line ready code)

Replace `src/main.ut`:

```bash
cat > "$APP_DIR/src/main.ut" <<'UT'
// sec4-book-tutorial:
// Complex LASM server that demonstrates middleware, request placeholders,
// auth enforcement, dynamic users store, and DB intrinsic routes.

fn health() effects { net } -> Int {
  // Liveness check.
  res.text(200, "ok");
  0
}

fn metadata() effects { net } -> Int {
  // Demonstrates typed header/cookie sinks.
  res.setHeader(headers.name("X-Showcase"), headers.value("sec4-book"));
  res.addCookie(cookie.build("session", "demo"));
  res.text(200, "metadata ready");
  0
}

fn previewHtml() effects { net } -> Int {
  // Query input is untrusted; sanitize before HTML sink.
  let raw = req.query("raw");
  res.html(sanitize.html(raw));
  0
}

fn echoRequest() effects { net } -> Int {
  // Request-source placeholders + typed validation.
  let trace = validate.nonEmpty(req.header("x-request-id"));
  let dynamicHeaderName = headers.name(validate.nonEmpty(req.query("header_name")));
  let dynamicHeaderValue = headers.value(validate.nonEmpty(req.query("header_value")));
  let dynamicCookieName = validate.nonEmpty(req.query("cookie_name"));
  let dynamicCookieValue = validate.nonEmpty(req.query("cookie_value"));

  res.setHeader(headers.name("X-Echo-Trace"), headers.value(trace));
  res.setHeader(dynamicHeaderName, dynamicHeaderValue);
  res.addCookie(cookie.build(dynamicCookieName, dynamicCookieValue));
  res.text(
    200,
    "method={{req.method}} path={{req.path}} version={{req.httpVersion}} id={{req.pathParam:id}} q={{req.query:q}} trace={{req.header:x-request-id}} session={{req.cookie:session}}"
  );
  0
}

fn gateChecks() effects { net } -> Int {
  // Core validation/path/url gates.
  let publicRaw = req.query("public_url");
  let internalRaw = req.query("internal_url");
  let fileRaw = req.query("file");
  let countRaw = req.query("count");
  let uuidRaw = req.query("uuid");
  let headerRaw = req.query("header");

  let base = path.base("/tmp/sec4-book-tutorial");
  path.under(base, fileRaw);
  url.public(publicRaw);
  url.internal(internalRaw);
  validate.int64(countRaw);
  validate.uuid(uuidRaw);
  validate.headerValue(headerRaw);
  validate.nonEmpty(req.query("name"));

  res.text(200, "gate checks completed");
  0
}

fn authRequired() effects { net } -> Int {
  auth.require(ctx.current());
  res.text(200, "auth ok");
  0
}

fn adminRequired() effects { net } -> Int {
  auth.requireRole(ctx.current(), "admin");
  res.text(200, "admin ok");
  0
}

fn createUser() effects { net } -> Int {
  // Runtime materializes dynamic users store response.
  res.ok(201, "CreateUserResponse", 0);
  0
}

fn updateUser() effects { net } -> Int {
  res.json(200, "UpdateUserResponse", 0);
  0
}

fn getUser() effects { net } -> Int {
  res.json(200, "UserResponse", 0);
  0
}

fn getUserByEmail() effects { net } -> Int {
  res.json(200, "UserByEmailResponse", 0);
  0
}

fn deleteUser() effects { net } -> Int {
  res.json(200, "DeleteUserResponse", 0);
  0
}

fn listUsers() effects { net } -> Int {
  res.json(200, "ListUsersResponse", 0);
  0
}

fn dbExec() effects { net, db.write } -> Int {
  let db = DbCap();
  let template = validate.nonEmpty(req.query("template"));
  let params = validate.nonEmpty(req.query("params"));
  let query = sql.q(template, params);
  db.exec(db, query);
  res.json(200, "DbExecResponse", 0);
  0
}

fn dbExecTx() effects { net, db.write, db.tx } -> Int {
  let db = DbCap();
  let template = validate.nonEmpty(req.query("template"));
  let params = validate.nonEmpty(req.query("params"));
  let query = sql.q(template, params);
  let tx = db.tx(db);
  db.execTx(tx, query);
  res.json(200, "DbExecTxResponse", 0);
  0
}

fn dbWriteAndQuery() effects { net, db.write, db.read } -> Int {
  let db = DbCap();
  let writeTemplate = validate.nonEmpty(req.query("write_template"));
  let writeParams = validate.nonEmpty(req.query("write_params"));
  let queryTemplate = validate.nonEmpty(req.query("query_template"));
  let queryParams = validate.nonEmpty(req.query("query_params"));
  let rowSchema = schema.row(validate.int64(req.query("row_schema")));

  db.exec(db, sql.q(writeTemplate, writeParams));
  db.queryOne(db, sql.q(queryTemplate, queryParams), rowSchema);
  res.json(200, "DbWriteAndQueryResponse", 0);
  0
}

fn dbQueryOne() effects { net, db.read } -> Int {
  let db = DbCap();
  let template = validate.nonEmpty(req.query("template"));
  let params = validate.nonEmpty(req.query("params"));
  let rowSchema = schema.row(validate.int64(req.query("row_schema")));
  let query = sql.q(template, params);
  db.queryOne(db, query, rowSchema);
  res.json(200, "DbQueryOneResponse", 0);
  0
}

fn dbListRecords() effects { net } -> Int {
  res.setHeader(headers.name("X-Sec4-Internal-Db-Op"), headers.value("listRecords"));
  res.json(200, "DbListRecordsResponse", 0);
  0
}

fn main() effects { net } -> Int {
  // Middleware chain
  let securityHeaders = sec.defaultHeaders();
  let corsCfg = cors.fromPolicy();
  let csrfCfg = csrf.fromPolicy();
  let authCfg = auth.fromPolicy();

  let base = http.router();
  let withHeaders = sec.withSecurityHeaders(base, securityHeaders);
  let withCors = cors.withCors(withHeaders, corsCfg);
  let withCsrf = csrf.withCsrf(withCors, csrfCfg);
  let router = auth.withAuth(withCsrf, authCfg);

  // Basic routes
  http.get(router, "/health", health);
  http.get(router, "/metadata", metadata);
  http.get(router, "/preview", previewHtml);
  http.get(router, "/echo/:id", echoRequest);
  http.get(router, "/gates", gateChecks);

  // Auth routes
  http.get(router, "/auth/required", authRequired);
  http.get(router, "/auth/admin", adminRequired);

  // User store routes
  http.post(router, "/users", createUser);
  http.post(router, "/users/:id/update", updateUser);
  http.post(router, "/users/:id/delete", deleteUser);
  http.get(router, "/users/:id", getUser);
  http.get(router, "/users/by-email", getUserByEmail);
  http.get(router, "/users", listUsers);

  // DB intrinsic routes
  http.post(router, "/db/exec", dbExec);
  http.post(router, "/db/exec-tx", dbExecTx);
  http.post(router, "/db/write-and-query", dbWriteAndQuery);
  http.get(router, "/db/query-one", dbQueryOne);
  http.get(router, "/db/records", dbListRecords);

  http.serve(8080, router);
  0
}
UT
```

---

## 7) Validate code before running

```bash
./target/debug/sec4 check --path "$APP_DIR"
./target/debug/sec4 build --path "$APP_DIR" --emit lasm
```

---

## 8) Start real Postgres (local infra)

```bash
infra/local-postgres/scripts/up.sh
```

Load runtime DB env (prefer runtime-resolved values):

```bash
set -a
if [ -f infra/local-postgres/.runtime.env ]; then
  source infra/local-postgres/.runtime.env
else
  source infra/local-postgres/.env
fi
set +a
```

Sanity check DSN:

```bash
echo "$SEC4_RT_LASM_DB_POSTGRES_DSN"
```

---

## 9) Run the server with LASM + Postgres adapter

```bash
DB_BASE="$APP_DIR/.lasm-db"
mkdir -p "$DB_BASE"

./target/debug/sec4 run \
  --path "$APP_DIR" \
  --backend lasm \
  --db-base "$DB_BASE" \
  --db-adapter postgres \
  --db-postgres-dsn "$SEC4_RT_LASM_DB_POSTGRES_DSN" \
  --port 18080
```

Keep this terminal running.

---

## 10) Test endpoints (new terminal)

### 10.1 Health + metadata

```bash
curl -i -H 'Authorization: Bearer token123' http://127.0.0.1:18080/health
curl -i -H 'Authorization: Bearer token123' http://127.0.0.1:18080/metadata
```

### 10.2 Sanitized preview + request placeholders

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:18080/preview?raw=%3Cscript%3Ealert(1)%3C%2Fscript%3E'

curl -i \
  -H 'Authorization: Bearer token123' \
  -H 'X-Request-Id: req-42' \
  -H 'Cookie: session=s1' \
  'http://127.0.0.1:18080/echo/42?q=hello&header_name=X-Flow&header_value=active&cookie_name=flow-cookie&cookie_value=on'
```

### 10.3 Auth enforcement

```bash
curl -i http://127.0.0.1:18080/auth/required
curl -i -H 'Authorization: Bearer token123' http://127.0.0.1:18080/auth/required
curl -i -H 'Authorization: Bearer role=admin token123' http://127.0.0.1:18080/auth/admin
```

### 10.4 Users store flows

```bash
USER_ID='123e4567-e89b-42d3-a456-426614174000'

CREATE_PAYLOAD='{"id":"'"$USER_ID"'","email":"user@example.com","age":30,"tags":["core"],"address":{"zip":"12345"},"meta":{"flags":{"a":true,"b":false,"c":true}}}'

UPDATE_PAYLOAD='{"id":"'"$USER_ID"'","email":"updated@example.com","age":31,"tags":["core","updated"],"address":{"zip":"67890"},"meta":{"flags":{"a":false,"b":true,"c":false}}}'

curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  -H 'Content-Type: application/json' \
  --data "$CREATE_PAYLOAD" \
  http://127.0.0.1:18080/users

curl -i -H 'Authorization: Bearer token123' "http://127.0.0.1:18080/users/$USER_ID"
curl -i -H 'Authorization: Bearer token123' "http://127.0.0.1:18080/users/by-email?email=updated%40example.com"
curl -i -H 'Authorization: Bearer token123' "http://127.0.0.1:18080/users"

curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  -H 'Content-Type: application/json' \
  --data "$UPDATE_PAYLOAD" \
  "http://127.0.0.1:18080/users/$USER_ID/update"

curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  "http://127.0.0.1:18080/users/$USER_ID/delete"
```

### 10.5 Real Postgres DB routes

Create table:

```bash
curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:18080/db/exec?template=CREATE%20TABLE%20IF%20NOT%20EXISTS%20demo_users%20%28id%20SERIAL%20PRIMARY%20KEY%2C%20email%20TEXT%20NOT%20NULL%20UNIQUE%29&params=0'
```

Insert row in transaction:

```bash
curl -i -X POST \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:18080/db/exec-tx?template=INSERT%20INTO%20demo_users%20%28email%29%20VALUES%20%28%241%29%20ON%20CONFLICT%20%28email%29%20DO%20NOTHING&params=%5B%22alice%40example.com%22%5D'
```

Query row:

```bash
curl -i \
  -H 'Authorization: Bearer token123' \
  'http://127.0.0.1:18080/db/query-one?template=SELECT%20id%2C%20email%20FROM%20demo_users%20WHERE%20email%20%3D%20%241&params=%5B%22alice%40example.com%22%5D&row_schema=7'
```

Runtime DB telemetry:

```bash
curl -i -H 'Authorization: Bearer token123' http://127.0.0.1:18080/db/records
```

Optional direct DB verification via `psql`:

```bash
psql "$SEC4_RT_LASM_DB_POSTGRES_DSN" -c "select id, email from demo_users order by id desc limit 5;"
```

---

## 11) Line-by-line code walkthrough

Use this to understand exactly what each section does.

### `health`, `metadata`, `previewHtml`

- `health` is the liveness endpoint.
- `metadata` demonstrates typed header/cookie sinks.
- `previewHtml` demonstrates trust boundary: raw input -> `sanitize.html(...)` -> `res.html(...)`.

### `echoRequest`

- Reads request values from headers/query/cookies.
- Uses typed validators and sink constructors:
  - `validate.nonEmpty(...)`
  - `headers.name(...)`
  - `headers.value(...)`
- Writes dynamic header + cookie and returns placeholder-resolved response text.

### `gateChecks`

- Enforces path and URL gates before doing anything dangerous:
  - `path.base(...)`
  - `path.under(...)`
  - `url.public(...)`
  - `url.internal(...)`
- Enforces typed primitive guards (`int64`, `uuid`, `headerValue`).

### `authRequired` and `adminRequired`

- `auth.require(...)` rejects requests without valid auth context.
- `auth.requireRole(..., "admin")` enforces role-gated behavior.

### User routes (`createUser`, `getUser`, etc.)

- Keep handler logic minimal and return deterministic response schemas.
- Runtime materialization handles dynamic user store flows behind these schema tags.

### DB routes (`dbExec`, `dbExecTx`, `dbWriteAndQuery`, `dbQueryOne`, `dbListRecords`)

- Instantiate DB capability with `DbCap()`.
- Build parameterized SQL via `sql.q(template, params)`.
- Use explicit operations:
  - `db.exec(...)`
  - `db.execTx(...)`
  - `db.queryOne(...)`
- `dbListRecords` returns adapter/runtime telemetry for observability.

### `main`

- Builds middleware chain in explicit order:
  - security headers -> CORS -> CSRF -> auth
- Registers all routes.
- Starts server with `http.serve(8080, router)`.

---

## 12) Troubleshooting

1. `run failed: --db-adapter is only supported with --backend lasm`
- Use `--backend lasm` when using DB adapter flags.

2. `db adapter postgres requires ... DSN`
- Ensure `infra/local-postgres/.env` is sourced and DSN env is exported.

3. Port conflict on `18080`
- Change `--port` to another free port.

4. Zed plugin not loading LSP
- Run:
  - `scripts/manage-zed-extension-local.sh status`
  - `which sec4audit-language-server`
- Optionally set explicit Zed binary path in settings JSON.

---

## 13) Cleanup

Stop server with `Ctrl+C`, then stop local Postgres:

```bash
infra/local-postgres/scripts/down.sh
```
