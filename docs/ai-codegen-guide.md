# AI Agent Codegen Guide for sec4 / Untrusted\<T\>

This guide is for AI agents (Claude, GPT, Copilot) generating `.ut` source files. No theory — just what you need to produce correct sec4 code.

---

## Language Quick Reference

### `resource` Declaration

```ut
resource <Name> {
  <field>: <Type> [annotations],
  ...
}
```

**Annotations**:
- `@primary` — marks the primary key field (exactly one per resource, required)
- `@auto` — runtime-filled; never set by the caller (use for `id`, `created_at`)
- `@default("value")` — compile-time-validated default value for the field

**Table name**: auto-derived as lowercase plural (`Task` → `tasks`). Override with:
```ut
resource Name table "custom_table" { ... }
```

### Field Types

| Type     | Notes                                      |
|----------|--------------------------------------------|
| `Uuid`   | UUID v4; typically `@primary @auto`        |
| `String` | Arbitrary text                             |
| `Email`  | Email address; validated at trust boundary |
| `Int64`  | 64-bit integer                             |
| `Int`    | 32-bit integer                             |
| `Time`   | Timestamp; use `@auto` for `created_at`    |
| `Bool`   | Boolean; `@default("false")` or `@default("true")` |

### Handler Syntax

```ut
fn <name>() effects { <effect>, ... } -> Int {
  ...
  0
}
```

Handlers always return `Int`. The trailing `0` is the exit value.

### Effects

Declare every capability the handler uses:

| Effect       | When to use                                       |
|--------------|---------------------------------------------------|
| `net`        | All HTTP handlers (always required)               |
| `db.read`    | Reading from the database                         |
| `db.write`   | Writing to the database                           |
| `db.tx`      | Explicit transaction control                      |
| `fs.read`    | Reading from the filesystem                       |
| `fs.write`   | Writing to the filesystem                         |
| `log`        | Emitting log output                               |
| `secrets.read` | Reading secret values                           |

Missing effects are a compile error (E4002).

### Middleware Chain Pattern

```ut
fn main() effects { net } -> Int {
  let router = http.router();
  http.use(router, sec);
  http.use(router, cors);
  http.use(router, csrf);
  http.use(router, auth);
  http.serve(8080, router);
  0
}
```

Apply middleware with `http.use` in order: `sec → cors → csrf → auth`.

### Validation Intrinsics

These operate on `Untrusted<T>` values and promote them to trusted types:

| Intrinsic              | Input type     | Result type |
|------------------------|----------------|-------------|
| `validate.email(v)`    | `Untrusted<String>` | `Email` |
| `validate.uuid(v)`     | `Untrusted<String>` | `Uuid`  |
| `validate.int64(v)`    | `Untrusted<String>` | `Int64` |
| `validate.nonEmpty(v)` | `Untrusted<String>` | `String` |
| `sanitize.html(v)`     | `Untrusted<String>` | `String` |

### Typed Sinks

Pass only trusted (validated) values to these sinks:

| Sink                         | Purpose                              |
|------------------------------|--------------------------------------|
| `sql.q(query, params...)`    | Parameterized SQL query              |
| `res.text(status, body)`     | Plain-text HTTP response             |
| `res.json(status, value)`    | JSON HTTP response                   |
| `res.html(status, body)`     | HTML HTTP response                   |
| `url.public(path)`           | External-facing URL construction     |
| `url.internal(path)`         | Internal service URL construction    |
| `path.base(p)`               | Canonicalize a base path             |
| `path.under(base, sub)`      | Safely join a sub-path under a base  |

---

## Examples

### Example 1 — Simple resource: Task management API

**Prompt**: "Create a task management API with title, status, and due date"

```ut
resource Task {
  id: Uuid @primary @auto,
  title: String,
  status: String @default("pending"),
  due_date: Time,
  created_at: Time @auto,
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.serve(8080, router);
  0
}
```

The runtime generates these endpoints automatically from the `Task` resource:
- `POST   /tasks`         — create
- `GET    /tasks`         — list
- `GET    /tasks/:id`     — get by id
- `PUT    /tasks/:id`     — update
- `DELETE /tasks/:id`     — delete

---

### Example 2 — Multiple resources: Blog API

**Prompt**: "Create a blog API with posts and comments"

```ut
resource Post {
  id: Uuid @primary @auto,
  title: String,
  body: String,
  author_email: Email,
  published: Bool @default("false"),
  created_at: Time @auto,
}

resource Comment {
  id: Uuid @primary @auto,
  post_id: Uuid,
  body: String,
  author: String,
  created_at: Time @auto,
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.serve(8080, router);
  0
}
```

Each resource gets its own five CRUD endpoints. `post_id` is a plain `Uuid` field — referential integrity is enforced at the application layer or via custom handlers.

---

### Example 3 — Resource + custom handler: User API with email validation

**Prompt**: "Create a user API where signup has email validation"

```ut
resource User {
  id: Uuid @primary @auto,
  email: Email,
  name: String,
  created_at: Time @auto,
}

fn signup() effects { net } -> Int {
  let emailRaw = req.query("email");
  validate.email(emailRaw);
  res.text(201, "user created");
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.post(router, "/users", signup);
  http.serve(8080, router);
  0
}
```

Custom handlers registered with the same method and path override the auto-generated route. Here `POST /users` is handled by `signup` instead of the resource default.

---

## Common Mistakes (and the Compiler Errors They Produce)

| Mistake | Error |
|---------|-------|
| Passing `Untrusted<T>` directly to `sql.q(...)` | E1002 — untrusted value flows to trusted sink |
| Logging a `Secret<T>` value | E1003 — secret value flows to log sink |
| Declaring a handler without `effects { ... }` | E4002 — missing effects declaration |
| Using `db.write` in a handler without declaring it | E4002 — undeclared effect |

**Rule of thumb**: Every value from `req.*` is `Untrusted<T>`. Run it through a `validate.*` or `sanitize.*` intrinsic before passing it anywhere typed.

---

## `sec4.policy` Template for Generated Projects

Place this file as `sec4.policy` in the project root alongside your `.ut` source files:

```toml
[policy]
name = "generated-api"
mode = "enforce"

[http]
max_body_bytes = 65536

[auth]
mode = "token"

[resource]
max_list_limit = 100
default_list_limit = 20
allow_delete = true
require_auth = true
```

- `mode = "enforce"` — policy violations are hard errors at runtime, not warnings.
- `require_auth = true` — all resource endpoints require a valid auth token.
- `allow_delete = true` — set to `false` to disable the auto-generated `DELETE` route for all resources.
