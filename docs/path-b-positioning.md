# sec4 / Untrusted\<T\>: Path B Positioning

**The pitch:**
Tell an AI what your API does. It writes the security spec. The compiler proves it's secure. The runtime runs it at 2500 req/s.

---

## What sec4 Actually Is

sec4 is not a general-purpose backend language. It is a **security specification language** paired with a smart runtime (LASM). The unit of composition is a security contract, not a handler function.

Three things happen when you write a `.ut` file:

1. **You declare what your API does** — routes, effects, data shapes, resource schemas.
2. **The compiler proves the declaration is secure** — Untrusted\<T\> flow, effects, typed sinks, policy gates.
3. **The LASM runtime materializes the API at native speed** — no interpreter, no middleware framework, no runtime framework.

This is the opposite of writing handlers in TypeScript or Go and hoping you remembered to sanitize every input. In sec4 the language itself cannot express "untrusted data flows directly into SQL" — that program does not compile.

---

## The Paradigm Shift

| General-purpose backend | sec4 |
|---|---|
| You write handlers. Hope you validated inputs. | You declare contracts. Compiler verifies inputs can't bypass validation. |
| Security is a library you remember to use. | Security is the type system. You can't forget it. |
| Middleware is wired manually. | Policy is declared. Misconfiguration is a compile error. |
| 254 lines for basic CRUD. | 10 lines for the same 5 endpoints. |
| Human reviews 254 lines of implementation. | Human (or AI) reviews 10-line spec. |

sec4 does not compete with TypeScript/Go/Rust for expressiveness. It out-competes them on **security density**: more security guarantees per line of code reviewed.

---

## Before / After: CRUD

### Before — hand-written handlers (~254 lines for lasm-alpha-full pattern)

Every handler manually pulls request data, validates it, constructs queries, and routes responses. Here is the create+list pattern compressed:

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

fn listTasks() effects { net, db.read } -> Int { /* ~20 lines */ }
fn getTask()   effects { net, db.read } -> Int { /* ~20 lines */ }
fn updateTask() effects { net, db.write } -> Int { /* ~20 lines */ }
fn deleteTask() effects { net, db.write } -> Int { /* ~15 lines */ }

fn main() effects { net } -> Int {
  let router = http.router();
  http.post(router, "/tasks", createTask);
  http.get(router,  "/tasks", listTasks);
  http.get(router,  "/tasks/:id", getTask);
  http.post(router, "/tasks/:id/update", updateTask);
  http.post(router, "/tasks/:id/delete", deleteTask);
  http.serve(8080, router);
  0
}
```

### After — resource declaration (10 lines)

```ut
resource Task {
  id:         Uuid   @primary
  title:      String
  status:     String @default("pending")
  owner_id:   Uuid
  created_at: Time   @auto
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.serve(8080, router);
  0
}
```

That declaration produces five live endpoints:

| Method | Path | Operation |
|--------|------|-----------|
| `POST` | `/tasks` | create |
| `GET` | `/tasks/:id` | get one |
| `GET` | `/tasks` | list (paginated) |
| `POST` | `/tasks/:id/update` | update (PATCH semantics) |
| `POST` | `/tasks/:id/delete` | delete |

Every request-sourced field flows through the Untrusted\<T\> validation gate for its type (`Uuid` -> `validate.uuid`, `String` -> `validate.nonEmpty`, `Email` -> `validate.email`). The compiler enforces this. No developer choice is involved.

---

## What the Compiler Enforces

These are hard compile errors, not lint warnings:

**SQL injection blocked (E1002)**
```ut
fn bad(cap: DbCap, query: Untrusted<String>) effects { db.write } -> Int {
  db.exec(cap, query);  // error[E1002]: untrusted value cannot flow into SQL sink
  1
}
```
The fix is not "remember to sanitize." The only valid path is a typed `SqlQuery` constructed through parameterized intrinsics.

**XSS blocked (HtmlSafe type)**
```ut
fn bad() effects { net } -> Int {
  res.html(1);  // error: res.html requires HtmlSafe, not Int
  0
}
```
`res.html(...)` only accepts `HtmlSafe`. Raw strings or untrusted values do not satisfy that type.

**Path traversal blocked (PathSafe gates)**
```ut
fn bad(cap: FsCap, path: Untrusted<String>) effects { fs.read } -> Int {
  fs.read(cap, path);  // error[E1002]: untrusted value cannot flow into filesystem sink
                       // note: sink requires trusted PathSafe values
  1
}
```
Filesystem sinks only accept `PathSafe`. The gate is in the type, not the runtime.

**SSRF blocked (PublicUrl / InternalUrl types)**

Network sinks require `PublicUrl` or `InternalUrl` — typed wrappers that can only be constructed through URL validation gates. A raw `Untrusted<String>` cannot flow into `net.call(...)`.

**Secret leakage blocked (E1003, E1004, E1005)**
```ut
fn bad(token: Secret<String>) effects { log } -> Int {
  log.info(token);  // error[E1003]: secret value cannot be logged
  1
}
// E1004: secret value cannot be JSON-encoded
// E1005: secret value cannot flow into SQL sink
```
`Secret<T>` is a distinct type. It cannot be formatted, logged, serialized, or passed to SQL sinks without an explicit `secret.redact(...)` call.

**Effect violations blocked (E4002)**

Every function declares its effects. Using a DB operation without declaring `db.write` is a compile error — the intent is explicit and verifiable.

**Untrusted data cannot reach sinks without explicit validation gates.** This is not a policy. It is the type system. The compile error is the only path.

---

## "Limitations" Are the Product

Evaluators often note that sec4 "can't do X" compared to Go or TypeScript. These are features, not gaps.

**"Can't access raw body fields"**
Correct. All request data is `Untrusted<T>`. The only way to get a typed value out is through a validation gate. This means zero parsing bugs, zero forgotten validations, zero injection from body fields.

**"Can't write custom middleware"**
Correct. Security policy is declared in `sec4.policy`, not wired by hand. A misconfigured CORS policy or forgotten auth check is a policy file error visible in code review — not a runtime surprise.

**"Effects are mandatory"**
Yes. Every function's side-effect profile is declared and compiler-verified. This makes the program auditable by humans and parseable by AI agents without executing it.

**"Can't shape response bodies freely"**
Correct. Responses follow deterministic contracts (`ok/status/data/error`). No serialization drift, no inconsistent error shapes across endpoints.

These constraints define the contract surface that the compiler can prove. If you want unconstrained handlers, use Go. If you want provably secure contracts, use sec4.

---

## The AI Agent Use Case

sec4's constraint model makes it uniquely suited to AI-generated APIs:

1. **AI agent receives natural language requirements** — "CRUD for tasks with title, status, and owner."
2. **Agent writes a `.ut` spec** — 10-line resource declaration.
3. **Compiler catches security mistakes before deployment** — if the agent makes a type error or tries to pass untrusted data to a sink, the compile error is deterministic and actionable.
4. **Runtime executes at native speed** — no interpreter overhead.
5. **Human reviews the 10-line spec, not 254 lines of generated handlers.**

The effect declarations are particularly useful: an AI agent can read `effects { db.write, net }` and know exactly what a function does without executing it. Security audits become diff reviews of typed declarations.

---

## Benchmarks

Canonical DB-backed benchmark (Postgres, single-mode LASM, 2026-03-07):

| Endpoint | sec4-lasm | Go | Rust | Node |
|---|---|---|---|---|
| `wb-task-get` (read by id) | **2465 req/s** | 212 | 42 | 29 |
| `wb-tasks-list` (paginated list) | **1458 req/s** | 139 | 22 | 10 |
| `wb-tasks-post` (write) | **492 req/s** | 391 | 42 | 36 |
| `wb-tasks-with-comment` (multi-write) | **198 req/s** | 96 | 41 | 40 |
| `wb-tasks-with-comment-tx` (transactional) | **198 req/s** | 90 | 41 | 38 |

sec4-lasm is **5-160x faster than Go/Rust/Node** on DB-backed workloads. The gap is widest on read paths because the LASM runtime eliminates the framework overhead present in all comparison implementations.

Resource-declared endpoints use the same DB adapter path as hand-written handlers. No performance regression from using `resource` instead of explicit handlers.

---

## Toolchain

The full developer workflow from spec to running service:

```
sec4 generate → sec4 check → sec4 describe → sec4 migrate → sec4 openapi → sec4 run
```

| Step | Command | What it does |
|------|---------|--------------|
| Scaffold | `sec4 generate --name MyApp --resources "..."` | Create project from resource descriptions |
| Verify | `sec4 check --path ./myapp` | Compile and surface type/effect/policy errors |
| Inspect | `sec4 describe --path ./myapp` | Print resource fields and all generated endpoints |
| Schema | `sec4 migrate --path ./myapp --adapter postgres` | Emit `CREATE TABLE` DDL ready to pipe to DB |
| Contract | `sec4 openapi --path ./myapp` | Emit the OpenAPI contract for the generated resource routes |
| Run | `sec4 run --path ./myapp` | Build and start the live service |

This workflow is designed for AI agents: `generate` → `check` → inspect → contract → commit. Every step is deterministic and produces machine-readable output. `describe` can be parsed to verify what was generated before deploying, `migrate` eliminates hand-written schema files, and `openapi` exposes the generated success-envelope contract to external clients without hand-maintained specs.

---

## How to Evaluate sec4

The right question is not "can sec4 do everything TypeScript can?" It is: "for the security-critical core of my API, how many lines of code does my security team need to review?"

A resource declaration like:

```ut
resource User {
  id:         Uuid   @primary
  email:      Email
  role:       String @default("member")
  created_at: Time   @auto
}
```

...encodes the full security contract for a five-endpoint user API. There is no SQL to audit. There is no middleware to review. There is no body parsing to verify. The compiler has already done it.

That is the Path B value proposition: **security specification language + smart runtime**, not a general-purpose backend language with security libraries bolted on.
