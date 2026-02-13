# Untrusted<T> Security Model

The key idea: make 'security surfaces' **typed, effect-checked, and policy-checked** so unsafe patterns are hard to express without explicitly opting in.

Below is a concrete design that fits the Untrusted<T> spec + MIR/intrinsics approach.

---

## 1) What 'attack vectors as first-class citizens' means in practice

For web backends, the most important attack surfaces are:

- **Untrusted input** (HTTP body/query/path/headers)
- **Injection sinks** (SQL, shell, template rendering, eval-like APIs)
- **XSS / output encoding** (HTML/JS contexts)
- **Authn/Authz** (who are you; what can you do)
- **Secrets handling** (tokens, passwords, keys)
- **Crypto usage** (dangerous footguns)
- **Filesystem and network** (SSRF, path traversal)
- **Deserialization and parsing** (JSON bombs, prototype pollution in some ecosystems)
- **Logging** (PII leakage, log injection)
- **Concurrency/time** (TOCTOU, race conditions)
- **Rate limits / abuse** (resource exhaustion)

Making these first-class means: the language forces developers (and AI) to be explicit about:

- where data is untrusted,
- where it can flow,
- and which sinks require sanitization/parameterization.

---

## 2) Language-level model: 'Taint' + 'Sinks' + 'Policies' (built in)

### 2.1 Introduce two core type wrappers

#### `Untrusted<T>`

Any data coming from outside is `Untrusted<T>` by default.

```ut
type Untrusted<T> // special/stdlib type with compiler support
type Trusted<T>   // optional; or just use plain T for trusted
```
- `HttpRequest.body()` returns `Untrusted<Bytes>`
- `req.query("q")` returns `Untrusted<Option<String>>`

You cannot directly use `Untrusted<String>` where `String` is expected.

#### `Secret<T>`

Secrets are tracked to prevent accidental logging/serialization.

```ut
type Secret<T>
```
- Env secrets: `env.secret("DB_PASSWORD") -> Secret<String>`
- Password fields decode into `Secret<String>`

Compiler/linter rules:

- `Secret<T>` cannot be formatted or logged unless explicitly redacted.
- JSON encoding of `Secret<T>` is forbidden unless explicitly allowed.

### 2.2 Sinks are typed, not stringly

Define 'sink types' that can only be constructed safely:

- `SqlQuery` cannot be constructed from arbitrary string concatenation.
- `HtmlSafe` cannot be created from raw strings without escaping.
- `ShellCmd` cannot be built from untrusted parts without quoting rules.

Examples:

```ut
type SqlQuery
type HtmlSafe
type ShellCmd
```

Construction is via safe APIs:

```ut
let q: SqlQuery = sql"SELECT * FROM users WHERE id = ${id}"
let page: HtmlSafe = html"<div>${escape(user_input)}</div>"
```

### 2.3 Security policies as compile-time checks

Add a policy section in `sec4.toml`:

```toml
[security]
forbid_raw_sql = true
forbid_shell = true
forbid_html_templates = false
require_csrf_on_cookie_auth = true
require_rate_limit = true
log_pii = "forbid"
```

And enforce it through:

- compiler diagnostics (hard errors for forbidden patterns),
- or a mandatory `aillint` pass in `sec4 build/test`.

This keeps it deterministic and AI-friendly.

---

## 3) How this integrates with effects and MIR

### 3.1 New 'security effects' (or effect tags)

Extend effect system with security-relevant categories:

- `input.http` (reading untrusted input)
- `sink.sql` (executing SQL)
- `sink.shell`
- `sink.html`
- `secrets.read`
- `crypto`
- `auth.verify`, `authz.check`

Handlers implicitly have `input.http`. DB calls have `sink.sql`, etc.

Then you can enforce policies like:

- 'All functions calling `sink.sql` must be in modules that import `db.safe`'
- 'No `sink.shell` in production build'
- 'Any function with `sink.html` must return `HtmlSafe` not `String`'

### 3.2 MIR-level: taint is a type property

You don't need to implement full dynamic taint tracking at runtime in v0.1.  
You can implement **static taint typing**:

- `Untrusted<T>` is a distinct type in the typechecker.
- There is no implicit conversion from `Untrusted<T>` to `T`.
- Only `validate(...)`, `sanitize(...)`, or safe constructors can unwrap.

In MIR it's just a type - lowering stays simple.

### 3.3 Intrinsics: security gates are runtime functions

Add a few runtime intrinsics:

- `ValidateEmail(Untrusted<String>) -> Result<Email, ValidationError>`
- `EscapeHtml(Untrusted<String>) -> HtmlSafe`
- `Redact(Secret<String>) -> String`

In MIR these are normal calls tagged with effects like `sanitize` / `validate`.

---

## 4) Are these external libs or standard library?

### Recommendation: **hybrid**

Make the 'attack surface model' **language + stdlib**, and keep actual algorithms partly in libraries.

#### In the language / compiler (must-have)

- `Untrusted<T>` and `Secret<T>` as special types (or stdlib types with compiler-known rules)
- Prohibit unsafe conversions by default
- Recognize sink APIs (SQL execution, templating) as special 'sinks'
- Policy enforcement hooks (compiler errors / lints)
- Canonical safe SQL literal `sql"..."` (no raw SQL concatenation)

These should not be optional libs, otherwise people (and AIs) skip them.

#### In stdlib/runtime (should ship with the language)

- JSON parsing/validation primitives
- SQL parameter binding and DB interface
- HTML escaping utilities
- Secure password hashing (Argon2/bcrypt) wrappers
- Constant-time compare
- Secret redaction tools
- CSRF helpers, cookie/session helpers

#### External libraries (optional ecosystem)

- Full WAF-like features, advanced rate limiters
- OAuth providers, SSO integrations
- Special protocol clients, custom auth frameworks
- Specialized sanitizers or parsers

But even when using external libs, they should plug into the first-class types:

- Accept `Untrusted<T>` for inputs
- Return refined/validated types
- Use safe sink types

---

## 5) Concrete language examples (what you'll hand to an AI)

### 5.1 Request decoding produces Untrusted data

```ut
fn create_user(req: HttpRequest)
  effects { input.http, sink.sql, authz.check, log }
  -> Result<HttpResponse, HttpError>
{
  let raw: Untrusted<Bytes> = req.body()
  let json: Untrusted<Json> = Json.parse(raw)?           // still untrusted

  let email_u: Untrusted<String> = json.get_string("email")?
  let email: Email = validate.email(email_u)?            // unwrap through validator

  let name_u: Untrusted<String> = json.get_string("name")?
  let name: NonEmptyString = validate.non_empty(name_u)?

  db.users.insert(email, name)?                          // db API only accepts validated types
  Ok(HttpResponse.text(201, "ok"))
}
```

### 5.2 SQL injection is structurally impossible

```ut
// forbidden by policy
let q = "SELECT * FROM users WHERE email = '" + email + "'"

// allowed: parameterized
let q: SqlQuery = sql"SELECT * FROM users WHERE email = ${email}"
db.query(q)
```

Make `db.query` accept only `SqlQuery`.

### 5.3 XSS prevention by types

```ut
fn profile_page(name: Untrusted<String>) -> HtmlSafe {
  html"<h1>${escape_html(name)}</h1>"
}
```

`HttpResponse.html(...)` accepts `HtmlSafe`, not `String`.

### 5.4 Secrets cannot be logged accidentally

```ut
let token: Secret<String> = env.secret("API_TOKEN")?
log.info(token)          // compile error
log.info(redact(token))  // ok
```

---

## 6) Implementation plan: add security as first-class without blowing up v0.1

### v0.1 Security Milestones (practical)

1.  **Introduce `Untrusted<T>` and `Secret<T>`** - Typechecker: no implicit unwrap - Simple helpers: `validate.*`, `redact`

2.  **Typed sinks** - `SqlQuery` + `sql"..."` template required - `HtmlSafe` + `escape_html` required for HTML responses

3.  **Policy config in `sec4.toml`** - Implement as mandatory lints at build time - Start with:

- forbid raw SQL (string concat or raw exec)
- forbid logging secrets
- require parameter binding

4.  **Static checks** - If a function calls `db.query` it must have effect `sink.sql` - If a function calls `env.secret` it must have effect `secrets.read` - If a function builds `HtmlSafe` it must have effect `sink.html` (optional)

This gives you a real 'attack vector aware' language quickly.

---

## 7) How this shows up in the compiler artifacts you already asked for

### 7.1 MIR additions (small)

- New builtin types: `Untrusted<T>`, `Secret<T>`, `SqlQuery`, `HtmlSafe`
- Add intrinsics:

- `SqlTemplateNew` / `DbExec` (only accepts SqlQuery)
- `EscapeHtml`
- `RedactSecret`
- Add effect tags to those calls.

No need for runtime taint tracking yet.

### 7.2 JSON derive changes

Derived decoders produce `Untrusted<...>` first, then call validators to unwrap.  
That prevents accidental usage of raw strings in SQL/HTML.

---

If you say 'yes' again, I'll write:

- the **formal type rules** for `Untrusted<T>` and `Secret<T>` (what conversions are allowed),
- the **exact policy/lint rules** (what's an error vs warning),
- and how to encode those rules into the compiler pipeline (typecheck phase vs effect phase vs lint phase).

---

Alright - here's (1) formal-ish **type rules** for `Untrusted<T>` and `Secret<T>`, (2) **policy/lint rules** you can encode, (3) where they live in the compiler pipeline, and (4) what to bake in so Untrusted<T> is *ridiculously easy to debug*.

---

## 1) Type rules for `Untrusted<T>` and `Secret<T>`

### 1.1 `Untrusted<T>` (taint-by-type, static)

**Intent:** anything from outside the trust boundary is `Untrusted<_>` until validated/sanitized.

#### Core typing rules

Let `Gamma |- e : T` mean: in context `Gamma`, expression `e` has type `T`.

**No implicit untaint**

- There is **no** rule that allows:
- `Gamma |- e : Untrusted<T>` to imply `Gamma |- e : T`
- So passing `Untrusted<String>` to a function expecting `String` is a compile error.

**Covariance (safe)**

- If `T <: U` (subtype or coercible) then `Untrusted<T> <: Untrusted<U>`.
- Practically: `Untrusted<T>` behaves like a wrapper; you can map inside but can't unwrap.

**Trusted constructors**  
The only way to convert `Untrusted<T> -> T` is via **explicit trust-establishing operations** that the compiler recognizes (stdlib 'gates'):

1.  **Validation**

```ut
fn validate.email(x: Untrusted<String>) -> Result<Email, ValidationError>
fn validate.non_empty(x: Untrusted<String>) -> Result<NonEmptyString, ValidationError>
fn validate.json<T: JsonCodec>(x: Untrusted<Bytes>) -> Result<T, ValidationError>
```

Rule:

- If `Gamma |- v : Untrusted<T>` and a validator returns `Result<U, E>`, then on the `Ok` branch you get `U` (trusted refined/typed).

2.  **Sanitization**

```ut
fn sanitize.html(x: Untrusted<String>) -> HtmlSafe
fn sanitize.sql_identifier(x: Untrusted<String>) -> SqlIdent  // optional, narrow and dangerous
```

Rule:

- Sanitizers produce types that are safe for specific sinks (`HtmlSafe`, `SqlIdent`), not raw `String`.

3.  **Parsing into refined types**  
    Construction APIs for refined types are gate functions:

```ut
Email.try_from(x: Untrusted<String>) -> Result<Email, ValidationError>
```

or two-step:

```ut
Email.try_from(x: String) -> Result<Email, ValidationError>   // but then you need a prior trust step
```

Recommended: accept `Untrusted<String>` directly for common refinements.

**Mapping / transforming Untrusted**  
Allow pure transformations without changing trust:

```ut
map(u: Untrusted<T>, f: fn(T)->U) -> Untrusted<U>     // compiler knows f must be pure
```

But v0.1 can avoid this: just don't provide general map; keep it explicit.

#### Where Untrusted originates (compiler-enforced)

All external boundary functions return `Untrusted`:

- `HttpRequest.body() -> Untrusted<Bytes>`
- `HttpRequest.query(name) -> Untrusted<Option<String>>`
- `Env.get(name) -> Untrusted<Option<String>>` (if env not considered trusted)
- `Json.get_string(k) -> Untrusted<String>`

This forces the 'taint front-end' without runtime tracking.

---

### 1.2 `Secret<T>` (non-leakable-by-default)

**Intent:** prevent accidental exfiltration (logs, traces, JSON, error strings).

#### Core typing rules

**No implicit reveal**

- No coercion:

- `Secret<String>` is not `String`
- formatting `Secret<T>` is forbidden
- JSON encoding `Secret<T>` is forbidden

**Two explicit reveal paths**

1.  **Redaction (safe reveal)**

```ut
fn redact(x: Secret<String>) -> String
fn redact_bytes(x: Secret<Bytes>) -> Bytes
```

2.  **Reveal (dangerous) - gated**

```ut
fn reveal(x: Secret<T>) effects { secrets.reveal } -> T
```
- Requires explicit effect `secrets.reveal` plus policy allowlist.
- This is how you force auditability.

**Propagation rules**

- If you combine secrets, result stays secret.
- `concat(Secret<String>, Secret<String>) -> Secret<String>` (or forbid unless needed)
- Storing a secret in a non-secret field is forbidden (no drop of wrapper).

#### Where Secrets originate

- `env.secret("KEY") -> Result<Secret<String>, EnvError>`
- password fields in JSON decode can produce `Secret<String>`.

---

## 2) Security policy + lint rules (compile-time gates)

### 2.1 Policy file (`sec4.toml`)

Example:

```toml
[security]
forbid_raw_sql = true
forbid_shell = true
forbid_html_unsafe = true
forbid_secret_reveal = true
require_param_sql = true
require_csrf_on_cookie_auth = true
require_rate_limit = true
log_pii = "forbid"  # forbid|warn|allow
```

### 2.2 Hard errors (compiler)

These should be *errors*, not warnings, to keep AI-generated code safe.

**A. Untrusted-to-sink errors**

- Passing `Untrusted<String>` to:

- SQL parameter position
- HTML response builder (unless escaped)
- filesystem path APIs
- URL fetch APIs (SSRF risk) unless validated as `SafeUrl`/`InternalUrl` etc.

Mechanism: sinks accept only safe types:

- `db.query(SqlQuery)` not `String`
- `HttpResponse.html(HtmlSafe)` not `String`
- `fs.read(PathSafe)` not `String`
- `http.client.get(UrlSafe)` not `String`

**B. Raw SQL forbidden**

- Reject:

- `db.exec(String)`
- any `sql` built by concatenation
- Allow only:
- `SqlQuery` produced via `sql"..."` template or `SqlQuery.builder(...)`

**C. Secrets leakage**

- Any attempt to:

- log `Secret<T>` (directly or via string interpolation)
- JSON encode `Secret<T>`
- include `Secret<T>` in error messages
- `reveal()` is forbidden if policy says so, or requires explicit allowlist.

**D. Forbidden effects**

- If `forbid_shell=true`, any call tagged `sink.shell` is a compile error.
- Same for `unsafe`.

**E. Missing declared effects**

- If a function uses sink intrinsics (db/net/fs), but doesn't declare effect -\> error. (You already had this.)

### 2.3 Warnings (linter)

Some checks are 'smell' not 'always wrong':

- logging unsanitized user input (log injection): warn unless normalized
- constructing URLs from untrusted components without `validate.url`
- missing rate limit middleware registration (if you can detect router composition)
- broad error mapping that returns internal messages to clients

### 2.4 Allowlist escape hatches (necessary)

You'll need a way to do unsafe things deliberately but auditable:

```ut
@allow(security="raw_sql", reason="legacy migration query, reviewed", ticket="SEC-123")
fn legacy_migration() effects { sink.sql } { ... }
```

Compiler rule:

- any forbidden construct can be allowed only inside an `@allow(...)` scope with required fields.
- CI can enforce 'no allows on main' or 'requires security reviewer'.

This keeps attack vectors 'first-class' while still letting teams ship.

---

## 3) Where this lives in the compiler pipeline

### Phase A

- Typechecking (primary enforcement)
- `Untrusted<T>` and `Secret<T>` are types.
- Most security happens automatically because sinks won't typecheck.

### Phase B

- Effect checking (audit trail)
- Enforce declared effects include used effects (`sink.sql`, `secrets.read`, `secrets.reveal`, etc).
- Enforce policy forbids some effects entirely.

### Phase C

- Security lint pass (policy + flow checks)
- Rules that require *context* beyond types:

- 'every cookie-auth route must include CSRF middleware'
- 'every POST route must have rate limiting'
- 'no internal error messages leaked'
- This pass runs after HIR typing (so it has full symbol/type info).

### Phase D

- MIR lowering
- By now, unsafe patterns should already be blocked.
- MIR includes effect tags on calls; security-lint annotations can be attached to MIR for debug/audit.

---

## 4) Make it the easiest language to debug (bake it into the spec)

Debuggability is mostly about **determinism, observability, and explanations**.

### 4.1 Language and runtime features (first-class)

**A. Deterministic stack traces**

- Every error (including `Result::Err`) carries optional:

- span (file/line/col)
- call-site chain (in debug builds)
- Runtime prints Untrusted<T> frames, not C frames.

**B. 'Explain mode' in the compiler**

- `sec4 explain <error-code>` prints:

- why it happened
- how to fix
- minimal examples
    This is huge for AI + humans.

**C. Structured logs built-in**

- Logging API is structured by default:

```ut
log.info({ event: "user_created", user_id: id })
```
- Prevent log injection by encoding fields.
- Automatically redacts `Secret<T>` fields.

**D. One-step 'repro bundle'**  
`sec4 bugreport` produces:

- MIR for relevant functions
- exact dependency lock + compiler version
- sanitized runtime config template
  So debugging is reproducible.

### 4.2 Compiler output requirements (debug mode)

- Always emit:

- HIR dump (`--emit=hir`)
- MIR dump (`--emit=mir`)
- lowered-ABI MIR dump (`--emit=mir-abi`)
- Each MIR instruction retains `span`.
- Deterministic IDs for locals/blocks (stable across builds when code unchanged).

### 4.3 Runtime 'debug hooks' (intrinsics)

Add intrinsics for inspection:

- `debug.print(value)` (pretty-print with types)
- `debug.trace(tag, value)` (structured trace event)
- `debug.assert(predicate, msg)` (runtime assert with source spans)

These are no-ops in release unless enabled.

### 4.4 'Time-travel-ish' debugging: deterministic replay (v0.1 feasible)

For web backends, easiest win:

- runtime can record incoming requests (headers/body) + timestamps + random seeds
- `sec4 replay <capture>` replays a request deterministically

To enable this, define:

- `random` and `time.now` as effects that can be overridden by a replay harness.
- In debug, runtime can supply recorded values.

### 4.5 Error types should be inspectable

Make `JsonError`, `HttpError`, `DbError` carry:

- `code` (stable string)
- `message` (safe for logs, not always safe for client)
- `debug` (optional, only in debug mode)
- `path` (for JSON decoding: `$.user.email`)

This makes bugs self-localizing.

---

## 5) Answering your 'will they be external libs?'

- The **model** (Untrusted/Secret + sink types + policies + effect tags) should be **language + stdlib**, otherwise it won't be consistently applied.
- The **implementations** (HTML escaper, password hash, URL parser) can be stdlib/runtime (recommended), and advanced features can be external libs *as long as they accept/return the first-class types*.

That gives you the best of both worlds: mandatory safety + extensibility.

---

If you want one more 'next step,' I can write the **formal typing judgments** for:

- safe sink functions (e.g., `db.query : SqlQuery -> ...`)
- validation/sanitization gates (`Untrusted<T> -> Result<Safe,U>`)
- and a minimal **dataflow lint** (taint flow) for cases types can't catch (like building URLs from parts).

---

## 1) Formal typing judgments for sinks + gates

I'll use these judgments:

- `Gamma |- e : T` - expression `e` has type `T`
- `Gamma |- f : (T1..Tn) -> U` - function type
- `Trusted(T)` means `T` is not wrapped in `Untrusted<_>`
- `Sink(S)` is a sink category: `SQL`, `HTML`, `SHELL`, `FS`, `NET`

### 1.1 Core wrapper types

- `Untrusted<T>` and `Secret<T>` are ordinary (nominal) types, but the compiler applies extra rules:

**No implicit unwrap**

- There is no coercion rule `Untrusted<T> <: T`
- There is no coercion rule `Secret<T> <: T`

So any use must go through explicit functions.

---

## 2) Typing rules for safe sinks (structural prevention)

### 2.1 SQL sink

Define:

- `SqlQuery` is the only type accepted by SQL execution primitives.
- `sql"..."` is the only *constructor* for `SqlQuery` in v0.1 (plus optional builder APIs).

**SQL execution**

```rust
(db.query) : (SqlQuery) -> Result<Rows, DbError>
(db.exec)  : (SqlQuery) -> Result<Unit, DbError>
```

Typing rule:

```markdown
'' ' q : SqlQuery
---------------------------
'' ' db.exec(q) : Result<Unit, DbError>
```

There is intentionally **no** overload `db.exec(String)`.

**SQL interpolation constraint**  
For `sql" ... ${ei} ... "` each interpolation expression must be of a 'DB-safe param type':

Define a predicate `DbParam('')`:

- True for: `Int`, `Int64`, `Bool`, `Uuid`, `Email`, `NonEmptyString`, `String` (optional), etc.
- False for: `Untrusted<_>`, `Secret<_>`, raw `Json`, `HtmlSafe`, etc.

Typing rule:

```bash
'' ' e1 : ''1  DbParam(''1)
...
'' ' en : ''n  DbParam(''n)
---------------------------------
'' ' sql"...${e1}...${en}..." : SqlQuery
```

And explicitly:

- `DbParam(Untrusted<T>)` is false
- `DbParam(Secret<T>)` is false

So SQL injection becomes type-impossible unless someone adds an escape hatch.

---

### 2.2 HTML sink (XSS prevention by type)

Define:

- `HtmlSafe` is the only type accepted by HTML response constructors.

```rust
HttpResponse.html : (HtmlSafe) -> HttpResponse
```

Typing rule:

```markdown
'' ' h : HtmlSafe
---------------------------
'' ' HttpResponse.html(h) : HttpResponse
```

There is no `HttpResponse.html(String)`.

**Escape gate**

```rust
sanitize.html : Untrusted<String> -> HtmlSafe
escape_html   : Untrusted<String> -> HtmlSafe
```

Optional: also allow escaping trusted string (still safe):

```rust
escape_html_trusted : String -> HtmlSafe
```

---

### 2.3 Shell sink (command injection)

Define:

- `ShellCmd` is required for shell execution:

```rust
shell.run : (ShellCmd) -> Result<ExitStatus, ShellError>
```

Only safe constructors exist:

- `shell.cmd("ls")` returns `ShellCmd` but **rejects** arguments unless provided as `ShellArgSafe`.
- `shell.arg_escape(Untrusted<String>) -> ShellArgSafe` (very restrictive; often forbidden by policy)
- Preferred: do not allow untrusted to shell at all in v0.1.

---

### 2.4 Filesystem sink (path traversal)

Define:

- `PathSafe` required for file operations.

```rust
fs.read  : (PathSafe) -> Result<Bytes, FsError>
fs.write : (PathSafe, Bytes) -> Result<Unit, FsError>
```

Gate:

- `validate.path_under(base: PathSafe, p: Untrusted<String>) -> Result<PathSafe, ValidationError>`
- Or 'path join' that normalizes and checks no traversal.

---

### 2.5 Network sink (SSRF)

Define:

- `UrlSafe` required for outbound requests.

```rust
http.client.get : (UrlSafe) -> Result<HttpClientResponse, NetError>
```

Gate:

- `validate.url_public(Untrusted<String>) -> Result<UrlSafe, ValidationError>`
- `validate.url_internal(Untrusted<String>) -> Result<InternalUrl, ValidationError>`

Policy can restrict internal URL usage.

---

## 3) Typing rules for validation/sanitization gates (Untrusted

- Safe)

### 3.1 General validation gate form

A 'validator' has the shape:

```rust
validate_X : Untrusted<A> -> Result<B, ValidationError>
```

and is the *only* way to produce `B` from untrusted input.

Rule (informally): you can only obtain `B` by pattern matching on `Ok(b)`.

Example:

```rust
'' ' u : Untrusted<String>
'' ' validate.email : Untrusted<String> -> Result<Email, ValidationError>
---
'' ' validate.email(u) : Result<Email, ValidationError>
```

Then:

```rust
'' ' r : Result<Email, E>
match r { Ok(x) => ... }   // inside Ok arm, x : Email
```

### 3.2 Sanitization gate form

Sanitizers typically return a *safe-for-context* type (not raw string):

```rust
sanitize.html : Untrusted<String> -> HtmlSafe
sanitize.log  : Untrusted<String> -> LogSafeString   // log injection mitigation
```

---

## 4) Secret typing rules (non-leakable-by-default)

### 4.1 Forbidden contexts

- No `ToString` / formatting trait for `Secret<T>`
- No `JsonCodec` derive for `Secret<T>`
- No implicit logging parameter accepts `Secret<T>`

So these expressions are rejected:

- `log.info(secret)`
- `Json.encode(secret)`
- `"{secret}"`

### 4.2 Allowed safe operations

- `redact : Secret<String> -> String` (safe)
- `reveal : Secret<T> effects { secrets.reveal } -> T` (dangerous, policy-gated)

Typing:

```markdown
'' ' s : Secret<T>
---------------------------------------
'' ' redact(s) : String   (if T = String)
```

And:

```markdown
'' ' s : Secret<T>
effects ' {secrets.reveal}
policy.allow_secret_reveal = true
---------------------------------------
'' ' reveal(s) : T
```

---

## 5) Minimal dataflow lint (for what types don't catch)

Types catch most cases if sinks are typed. The linter is for 'composed risk' and policy checks.

### 5.1 Dataflow model

Define a lattice:

- `TAINT = { Trusted, Untrusted }` (and optionally `Secret` as separate dimension)
- Each expression has a taint label computed by:
- literals/constants: `Trusted`
- request/env/json extraction: `Untrusted`
- combining ops: `join` (max)
- validators: `Trusted` output
- sanitizers: output is `Trusted` but of safe type (`HtmlSafe`, `PathSafe`, etc.)

### 5.2 Propagation rules (simple)

- `taint(x + y) = join(taint(x), taint(y))`
- `taint(f(a..)) =` depends:

- if `f` is a validator/sanitizer: output taint becomes Trusted
- else: default join of args (conservative)

### 5.3 Sink rules (lint)

Even if typing prevents direct use, lint can enforce stronger policies, e.g.:

**URL composition SSRF warning**  
If you do:

- `let u = "http://internal/" + input`
  even if you later validate, warn unless validation is immediately applied.

**Logging untrusted**  
Warn if `log.*` receives `Untrusted<String>` that has only been concatenated/trimmed (not `sanitize.log`).

**Authz-required routes**  
If a handler calls `db.write` and lacks `authz.check`, warn/error depending policy.

### 5.4 Implementation (practical)

Run after typecheck on HIR:

- Build CFG per function
- Track taint for locals
- Report when tainted value reaches:
- log calls
- URL constructors
- template builders
- header setters (response splitting risk)

This lint is intentionally shallow for v0.1. The type system does the heavy lifting.

---

## 6) Debuggability additions tied to security features (small but huge)

- Every 'gate' function (`validate.*`, `sanitize.*`) must return structured errors:

- `code`, `path`, `reason`, `span`
- The compiler's diagnostics for taint/sink errors should include:
- the *source of taint* (e.g., `req.query("id")`)
- the *sink* (e.g., SQL parameter)
- the *missing gate* suggestion (`validate.uuid`, `sanitize.html`, etc.)
    That makes debugging security flows painless.
