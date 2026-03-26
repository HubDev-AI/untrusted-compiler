# Resource Auto-CRUD: Smart Runtime for sec4 Path B

**Date:** 2026-03-19
**Status:** Implemented on `feature/resource-improvements` and verified on 2026-03-26
**Context:** Post-alpha strategic decision to position sec4 as a security specification language with smart runtime (Path B), rather than a general-purpose backend language.

## Problem

sec4's .ut programs are effectively security-typed route declarations. Developers write 254 lines (lasm-alpha-full) to declare what amounts to typed boilerplate around 6 DB operations. The language has 70+ intrinsics and comprehensive security enforcement (SQL injection, XSS, path traversal, SSRF, secret leakage all blocked at compile time), but no ergonomic way to express standard CRUD patterns without hand-writing every handler.

The gap: hello-api (28 lines) to lasm-alpha-full (254 lines) is a cliff, not a ramp. There's no intermediate abstraction.

## Solution

Introduce `resource` declarations in .ut files. The compiler validates the security properties. The LASM runtime auto-generates CRUD endpoints with full validation, typed SQL, and deterministic response envelopes.

## Syntax

```ut
resource Task {
  id: Uuid        @primary
  title: String
  status: String  @default("pending")
  owner_id: Uuid
  created_at: Time @auto
}
```

### Field annotations

- `@primary` — identity field, used for `:id` path parameter routing and WHERE clauses. Exactly one `@primary` field is required per resource (compiler rejects zero or multiple). Without `@auto`, the primary field is accepted from the request body. With `@primary @auto`, the runtime always generates the ID.
- `@auto` — runtime-filled field, not accepted from request bodies (e.g., timestamps, generated UUIDs). Auto-fill rules: `Time` fields use `now()`, `Uuid` fields use `gen_uuid()`.
- `@default(value)` — fallback value if field is absent from request body. Value must be a literal matching the field type: `@default("text")` for String, `@default(42)` for Int/Int64, `@default(true)` for Bool. On UPDATE, omitting a `@default` field means "no change" (PATCH semantics), NOT "reset to default".
- `@unique` — add a uniqueness constraint and map duplicate writes to deterministic `409 RESOURCE.CONFLICT`.
- `@optional` — allow the field to be omitted on create and set to SQL `NULL`.

### Table naming

Table name is derived by naive snake_case + append-`s` (e.g., `Task` -> `tasks`, `UserProfile` -> `user_profiles`). For irregular plurals, use explicit table name override:

```ut
resource Person table "people" {
  id: Uuid @primary
  name: String
}
```

### Field types and validation gates

Each field type determines the automatic validation gate applied to request-sourced values:

| Field Type | Validation Gate | SQL Type (Postgres) | SQL Type (SQLite) |
|------------|----------------|--------------------|--------------------|
| Uuid       | validate.uuid  | TEXT               | TEXT               |
| String     | validate.nonEmpty | TEXT            | TEXT               |
| Email      | validate.email | TEXT               | TEXT               |
| Int64      | validate.int64 | BIGINT             | INTEGER            |
| Int        | (numeric check)| INTEGER            | INTEGER            |
| Time       | (iso8601 check)| TEXT               | TEXT               |
| Bool       | (boolean check)| BOOLEAN            | INTEGER            |

The compiler enforces Untrusted<T> gates on all request-sourced fields automatically. No developer choice is involved in validation — the type determines the gate.

## Architecture

### Route method constraint

The current compiler only supports `http.get` and `http.post` as route registration intrinsics (no `http.put`, `http.patch`, `http.delete`). Resource CRUD routes use `POST /{prefix}/:id/update` and `POST /{prefix}/:id/delete` deliberately to match this constraint. Future RESTful method support (`PUT`, `DELETE`) is a follow-on, not a blocker.

### SQL identifier quoting

All generated SQL quotes column and table names to prevent collisions with SQL reserved words. Postgres uses `"column_name"`, SQLite uses `"column_name"` (both support ANSI quoting). This is unconditional — no allowlist of reserved words, just always quote.

### Untrusted\<T\> trust model for resources

The compiler does not trace Untrusted\<T\> flow for resource fields because the runtime CRUD implementation is trusted code that always applies the correct validation gate per field type. This is analogous to how built-in intrinsics (like `sanitize.html()`) are trusted to enforce their security contracts. The compiler validates that the resource declaration is well-formed; the runtime guarantees correct execution.

### Compiler responsibilities (sec4-core + sec4-cli)

1. **Parse** `resource` declarations as new syntax production
2. **Validate** field types are known built-in types (no user-defined types in resource fields). Reject resources with zero `@primary` fields, multiple `@primary` fields, zero non-auto fields (warning), or field names colliding with other resource names.
3. **Extract** resource metadata into `LasmResourcePlan`:
   ```
   LasmResourcePlan {
     name: String,              // "Task"
     table: String,             // "tasks" (snake_case pluralized)
     fields: Vec<ResourceField>,// [{name, type, primary, auto, default}]
     route_prefix: String,      // "/tasks"
   }
   ```
4. **Generate** route plans (same shape as existing `LasmRunRoutePlan`):
   - `GET /{prefix}` — list (effects: db.read, net)
   - `GET /{prefix}/:id` — get one (effects: db.read, net)
   - `POST /{prefix}` — create (effects: db.write, net)
   - `POST /{prefix}/:id/update` — update (effects: db.write, net)
   - `POST /{prefix}/:id/delete` — delete (effects: db.write, net)
5. **Detect conflicts** — if a developer registers an explicit handler for the same method+path, the explicit handler wins

### Resource dispatch architecture

Resource routes are dispatched through `lasm_resource_dispatch.rs` BEFORE `apply_lasm_dynamic_response_materialization()` runs. The resource dispatch function reads the request body, validates fields, executes SQL, and writes the response directly, bypassing template-based materialization. This is a separate dispatch path, not an extension of the existing header-based plan system.

Resource metadata is embedded in the route plan via a dedicated header:
- `X-Sec4-Internal-Resource-Op: create|get|list|update|delete`
- `X-Sec4-Internal-Resource-Plan: <JSON-encoded LasmResourcePlan>`

The JSON encoding carries the full resource metadata (name, table, fields with types/annotations). The runtime deserializes this once per route at startup and caches the parsed plan.

### Runtime responsibilities (LASM)

#### POST /{prefix} (create)

1. Parse request body as JSON
2. Validate each field using type-driven gate
3. Fill `@auto` fields (Time -> now(), Uuid @primary -> gen_uuid() if not provided)
4. Fill `@default` fields if absent
5. Generate: `INSERT INTO {table} ({columns}) VALUES ({placeholders})`
6. Execute via existing db.exec adapter path
7. Return: `{ ok: true, status: 201, data: { id: "...", ...fields } }`

#### GET /{prefix}/:id (read one)

1. Validate `:id` path param via primary field type gate
2. Generate: `SELECT {columns} FROM {table} WHERE {primary} = $1 LIMIT 1`
3. Execute via existing db.queryOne adapter path
4. Return: `{ ok: true, status: 200, data: { ...fields } }` or `{ ok: false, status: 404, error: { code: "NOT_FOUND" } }`

#### GET /{prefix} (list)

1. Accept query params: `?limit=N&offset=N`, plus field filters (`?email=x`) and `sort` / `order`
2. Validate filter values against their resource field types
3. Generate: `SELECT {columns} FROM {table} [WHERE ...] ORDER BY {field|primary} {ASC|DESC} LIMIT $N OFFSET $N`
4. Return success envelope data: `{ items: [...], count: N, limit: N, offset: N }`

#### POST /{prefix}/:id/update

1. Validate `:id` + body fields (only provided non-auto fields are updated)
2. Generate: `UPDATE {table} SET {field} = $N, ... WHERE {primary} = $1`
3. Return updated record or 404

#### POST /{prefix}/:id/delete

1. Validate `:id`
2. Generate: `DELETE FROM {table} WHERE {primary} = $1`
3. Return: `{ ok: true }` or 404

### DDL and contract generation

The implemented branch includes three readout/generation commands on top of the runtime path:

- `sec4 describe --path ...` — print fields, annotations, and generated routes
- `sec4 migrate --path ... --adapter sqlite|postgres` — emit `CREATE TABLE IF NOT EXISTS ...` DDL from resource definitions
- `sec4 openapi --path ...` — emit an OpenAPI 3.0 spec that matches the resource success-envelope contract

Important implementation note: Postgres DDL currently maps string-like resource fields (`Uuid`, `String`, `Email`, `Time`) to `TEXT`. This is deliberate so generated prepared statements stay compatible with the LASM runtime's parameter binding behavior.

## Policy integration

`sec4.policy` gains a `[resource]` section:

```toml
[resource]
max_list_limit = 100
default_list_limit = 20
allow_delete = true
require_auth = true
```

Controls:
- Pagination bounds (prevents unbounded queries)
- Whether DELETE endpoints are generated
- Whether auth middleware wraps resource routes
- Inherits existing body size limits from `[http]` section

## Custom handler override

Developers override auto-generated routes by registering explicit handlers:

```ut
resource Task {
  id: Uuid @primary
  title: String
  status: String
}

// Override the auto-generated create
fn createTask() effects { net, db.write } -> Int {
  let db = DbCap();
  db.exec(db, sql.q("INSERT INTO tasks ...", params));
  res.json(201, "TaskResponse", 0);
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  // Auto-CRUD routes for Task are registered implicitly
  // POST /tasks overridden by explicit registration:
  http.post(router, "/tasks", createTask);
  http.serve(8080, router);
  0
}
```

Rule: explicit `http.{method}(router, path, handler)` for a method+path that matches a resource route overrides the auto-generated handler for that single operation. All other CRUD routes remain auto-generated.

## What a real app looks like after this change

Before (lasm-alpha-full pattern, ~60 lines for basic task CRUD):

```ut
fn createTask() effects { net, db.write } -> Int {
  let db = DbCap();
  let template = validate.nonEmpty(req.query("template"));
  let params = validate.nonEmpty(req.query("params"));
  let query = sql.q(template, params);
  db.exec(db, query);
  res.json(201, "CreateTaskResponse", 0);
  0
}
fn getTask() effects { net, db.read } -> Int { ... }
fn listTasks() effects { net, db.read } -> Int { ... }
fn updateTask() effects { net, db.write } -> Int { ... }
fn deleteTask() effects { net, db.write } -> Int { ... }
fn main() effects { net } -> Int {
  // ... 9 lines middleware ...
  http.post(router, "/tasks", createTask);
  http.get(router, "/tasks/:id", getTask);
  http.get(router, "/tasks", listTasks);
  // ... etc
}
```

After (resource declaration, ~10 lines):

```ut
resource Task {
  id: Uuid        @primary
  title: String
  status: String  @default("pending")
  owner_id: Uuid
  created_at: Time @auto
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.serve(8080, router);
  0
}
```

5 CRUD endpoints with full validation, typed SQL, auth (if policy says so), pagination, and deterministic error envelopes. The compiler guarantees all Untrusted<T> flows through validation gates. The runtime materializes everything.

## Implementation scope

### Files modified

| File | Change |
|------|--------|
| `compiler/sec4-core/src/token.rs` | New `Keyword::Resource` variant |
| `compiler/sec4-core/src/lexer.rs` | Keyword recognition for `"resource"` |
| `compiler/sec4-core/src/ast.rs` | New `ItemKind::Resource(ResourceDecl)` variant, `ResourceDecl` struct with field annotations |
| `compiler/sec4-core/src/parser.rs` | New `parse_resource_item` production, branch in `parse_program` |
| `compiler/sec4-core/src/semantic.rs` | Resource field validation, auto-route plan generation, conflict detection, resource name uniqueness vs struct/enum names |
| `compiler/sec4-cli/src/main.rs` | Resource metadata extraction, route plan injection, resource dispatch intercept before `apply_lasm_dynamic_response_materialization()` |
| NEW: `compiler/sec4-cli/src/lasm_resource_dispatch.rs` | SQL template generation (with quoted identifiers), type-driven validation execution, CRUD operation handling, resource plan caching |

### Files unchanged

- DB adapters (records-log, sqlite, postgres) — already execute arbitrary SQL
- Response envelope format — same `ok/status/data/error` shape
- Middleware chain — resources use same auth/cors/csrf/sec pipeline
- Effects system — auto-generated routes declare effects like any handler
- sec4.toml — no changes
- Existing .ut files — backward compatible, `resource` is additive

### Testing plan

1. Parser tests: `resource` declarations parse correctly, reject invalid field types
2. Semantic tests: auto-generated route plans match expected shapes, conflict detection works
3. LASM integration tests: each CRUD operation against SQLite adapter (fast, no external deps)
4. Postgres integration test: full CRUD cycle against local Postgres
5. Policy tests: list pagination bounds, delete disable, auth requirement
6. Override test: explicit handler suppresses auto-generated route for that method+path
7. Negative tests: resource with unknown field type rejected, resource with no @primary rejected

## Edge cases

**Multiple resources:** Supported. Each resource generates 5 routes under its own prefix. The compiler rejects route prefix collisions between resources at compile time (e.g., two resources that both resolve to `/tasks`).

**Empty resource (only `@primary`):** Valid but the compiler emits a warning. CREATE inserts only the primary key. UPDATE has no updatable fields (returns the record unchanged). LIST and GET work normally.

**All `@auto` fields:** Valid. CREATE accepts an empty request body. The runtime fills all fields. No request-body validation step is needed (empty validated-field set is valid).

**Resource + struct name collision:** Rejected. `struct Task { ... }` and `resource Task { ... }` in the same compilation unit produces a compile error.

## Non-goals

- Schema diffing or migration history management beyond `CREATE TABLE IF NOT EXISTS ...` generation
- Foreign key / relationship declarations (future `@belongs_to(User)`)
- Computed fields or virtual columns
- Custom serialization formats (always JSON)
- Soft delete (future `@soft_delete`)
- Audit logging (already handled by existing records persistence)

## Success criteria

1. A developer can declare a resource and get 5 working CRUD endpoints with zero handler code
2. All request values flow through Untrusted<T> validation gates (compiler-enforced)
3. Custom handlers cleanly override individual operations
4. Performance matches hand-written handlers (same SQL execution path)
5. The workbench benchmark can be rewritten using resources with equivalent performance
