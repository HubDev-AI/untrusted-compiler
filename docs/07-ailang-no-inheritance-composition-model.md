## Spec: Avoiding trait/interface spaghetti in AILang (TS-like, low-magic)

This section defines a **no-traits/no-interfaces** programming model that still supports abstraction, reuse, testing, security, and effects—without “implements what” graphs or derive-style codegen.

### Goals

* Avoid global implementation hierarchies (“X implements Y implements Z…”).
* Keep abstraction **local, explicit, and diff-friendly**.
* Keep **effects** and **security** (Untrusted/Secret + typed sinks) as first-class.
* Avoid “magic” code generation (no `derive`-style impl synthesis).
* Maintain TS-like ergonomics: simple objects, composition, schema validation.

---

## 1) No nominal traits/interfaces in user code

AILang v0.x does **not** expose user-defined nominal traits/interfaces.

Instead, abstraction is achieved via:

1. **Structural shape types** (record/object types)
2. **Capability objects** (values containing functions)
3. **Schema-based decoding/encoding** (explicit, non-magical)
4. Optional **law modules** (behavior contracts via tests)

> Rationale: nominal interfaces create large cross-module coupling and “implements hell”. Structural capabilities keep dependencies visible at the call site.

---

## 2) Structural shape types

A **shape type** is a record/object type literal describing required fields and their types.

```ailang
type HasClock = { nowMs: fn() -> Int64 }
type HasLog   = { info: fn(String) -> Unit }
```

### 2.1 Structural compatibility

A value `v` is compatible with shape `S` iff:

* `v` has **at least** the fields of `S`
* each field type is compatible

This is **width subtyping**:

* `{ a: Int, b: Int }` is usable as `{ a: Int }`

### 2.2 Intersection types for composition

`A & B` requires all fields from both shapes:

```ailang
function f(ctx: HasClock & HasLog) -> Unit { ... }
```

Conflict rule:

* If both define the same field name, the field types must unify exactly (or by a defined compatibility rule); otherwise it’s an error.

### 2.3 Closed vs open records

To keep debugging and AI-generation deterministic:

* Shapes are **closed by default**: only declared fields exist.
* An optional “open record” feature may be added later, but is not required for v0.x.

---

## 3) Capability objects (the replacement for “implements”)

A **capability object** is a value whose fields are operations (functions). This replaces interfaces.

Example: repository abstraction:

```ailang
type UserRepo = {
  findById: fn(UserId) effects { db.read }  -> Result<User, DbError>
  create:   fn(NewUser) effects { db.write } -> Result<User, DbError>
}
```

### 3.1 Construction is explicit

```ailang
function makePgRepo(db: Db) effects { db.read, db.write } -> UserRepo {
  return {
    findById: (id) => db.queryOne(sql`SELECT ... WHERE id=${id}`),
    create:   (u)  => db.exec(sql`INSERT ...`),
  }
}
```

### 3.2 Testing/mocking is trivial

```ailang
function makeFakeRepo() -> UserRepo {
  return {
    findById: (id) => Ok(fakeUser(id)),
    create:   (u)  => Ok(fakeUser(newId())),
  }
}
```

No `implements`, no inheritance, no trait objects.

---

## 4) Effects integration (required)

Effects remain the authoritative model for side effects.

### 4.1 Effects on function types (mandatory for capability fields)

In v0.x, **function types inside shapes must declare effects**:

```ailang
type HasLog = { info: fn(String) effects { log } -> Unit }
```

This avoids ambiguity and keeps diagnostics excellent.

### 4.2 Effect propagation rule

When calling a function-valued field, its declared effects are counted as used effects of the caller.

If:

* `Γ ⊢ repo : UserRepo`
* `repo.create : fn(NewUser) effects { db.write } -> Result<User, DbError>`

Then calling `repo.create(x)` requires the caller to declare `db.write`.

---

## 5) Replace “derive/implement protocols” with explicit schemas (no magic)

Most “implements” mess in typed ecosystems comes from serialization/validation glue.

AILang replaces it with **explicit `schema` blocks** and **generic library decode/encode**.

### 5.1 Schema definition

A `schema` is a *value-level* description of how to decode/validate (and optionally encode) a type.

```ailang
schema CreateUserRequest {
  email: Email = validate.email
  name: NonEmptyString = validate.non_empty
  age?: Int64 = validate.int64
}
```

Rules:

* Each field declares its target type and the validator/converter.
* `field?: T` means optional (maps to `Option<T>` conceptually).
* Schemas are explicit, readable, versionable, and testable.

### 5.2 Decoding boundary (trust gate)

`req.json(schema)` is the canonical trust boundary:

```ailang
const body = req.json(CreateUserRequest)?  // Result<CreateUserRequest, HttpError>
```

* Request body starts as `Untrusted<Bytes>`.
* Decoding via schema produces **trusted** structured data (or a structured error).
* This avoids the need for “JsonCodec implements …”.

### 5.3 Encoding is also explicit (optional but recommended)

To avoid accidental leaks (esp. `Secret<T>`), encoding can require an explicit schema as well:

```ailang
return Response.json(200, json.encode(CreateUserResponse, respValue))
```

Policy can enforce:

* “No JSON encode without schema” (strict mode), or
* “Secret fields forbidden in schemas unless explicitly redacted.”

### 5.4 No hidden code generation

Schemas do **not** generate per-type functions.
They are consumed by generic runtime/stdlib functions:

* `json.decode(schema, jsonValue) -> Result<T, JsonError>`
* `json.encode(schema, value) -> Json`

---

## 6) Optional: Law modules (behavior contracts without trait hierarchies)

When you truly need behavioral constraints (“this is a cache”), express them as **test suites**.

```ailang
module cache_laws {
  type Cache = { get: fn(String)->Option<String>, set: fn(String,String)->Unit }

  function laws(make: fn() -> Cache) -> Unit {
    test "get after set" {
      const c = make()
      c.set("k","v")
      assert(c.get("k") == Some("v"))
    }
  }
}
```

This avoids:

* trait inheritance webs
* complicated generic bounds
* “implements” graphs

And it produces better real-world correctness.

---

## 7) How this avoids spaghetti (design constraints)

To keep things from devolving into another form of hell, enforce:

1. **Small shapes**: prefer many small capabilities (`HasLog`, `HasClock`) over one giant `AppContext`.
2. **No deep nesting**: keep capability records shallow; compose with `A & B` rather than nesting layers.
3. **No implicit globals**: dependencies are explicit parameters.
4. **Canonical patterns**: one recommended way to structure backend code (router → handler → service).
5. **Effects are mandatory** on function fields: prevents hidden side effects.
6. **Schemas are the only trust gate** for HTTP input: prevents ad-hoc parsing.

---

## 8) Security stays first-class (still no interfaces)

This model works *with* security-by-construction:

* External data is `Untrusted<_>` until schema decode/validation.
* Secrets are `Secret<_>` and schemas/encoders can forbid them by default.
* Sinks remain typed:

  * DB exec accepts `SqlQuery` only
  * HTML response accepts `HtmlSafe` only
  * HTTP client accepts `UrlSafe` only
* Policies/lints check:

  * forbidden effects (shell)
  * required middleware patterns
  * unsafe escapes (if allowed, require annotated allowlist)

No traits needed.

---

## 9) Lowering/implementation note (non-spec, but clarifying)

Capability objects can compile in two ways (compiler choice, not user-visible):

1. **Record-of-function-pointers** (like a simple vtable), OR
2. **Monomorphized structs** when passed as generics

In both cases, effects are preserved via the function types in the record.

---

### Summary

AILang avoids trait/interface spaghetti by making abstraction:

* **structural** (shape types),
* **value-based** (capability objects),
* **schema-driven** (explicit decoding/encoding),
* and **effect-audited** (effects on function types).

It keeps the codebase understandable for humans and extremely predictable for AI.
