# 43 AILang v0 Scope (One Page)

## Purpose

AILang v0 is a TypeScript-like, backend-focused compiled language whose compiler enforces security-by-construction and effect auditing. v0 is not sugar because it rejects insecure flows and missing effect declarations at compile time.

---

## 1) Non-negotiable compile-time guarantees

v0 must reject these at compile time:

1. Untrusted -> Sink

- Values of type `Untrusted<T>` cannot be used where a trusted value is required, especially in sinks:
  - SQL execution
  - HTML responses/templates
  - outbound HTTP URL
  - filesystem paths
  - logging (optional strict mode)

2. Typed sinks only

- SQL APIs accept only `SqlQuery`
- HTML responses accept only `HtmlSafe`
- HTTP client accepts only `UrlSafe`
- FS APIs accept only `PathSafe`

3. No secret leakage

- `Secret<T>` cannot be:
  - logged
  - JSON-encoded
  - string-formatted/interpolated
- Only allowed via `redact(secret)` (safe) or `reveal(secret)` (requires effect + policy allow)

4. Effects must be declared

- Any function that uses effects must declare them:
  - `used_effects` subset of `declared_effects`
- Runtime intrinsics and capability functions contribute effects.

These 4 guarantees define real language behavior.

---

## 2) What v0 is (and is not)

### v0 is:

- A small compiler front-end (parse -> typecheck -> effects/security checks -> MIR)
- A backend that reuses an existing compiler (Emit C + clang is default)
- A runtime library providing HTTP + JSON + logging + time (DB optional at first)

### v0 is not:

- A systems language (no lifetimes/borrow checker)
- A macro platform
- A trait/interface ecosystem

---

## 3) Language surface (TypeScript-like)

### Core declarations

- `import ...`
- `type Name = ...` (includes object literal types)
- `enum Name { ... }` (errors and tagged unions)
- `schema Name { ... }`
- `function name(args...): Ret [throws E] { ... }` (or `fn`)

### Values and control flow

- `const` and `let` (optionally allow `mut`, but TS-like defaults are fine)
- `if`, `match` (and optionally `switch` sugar)
- expression calls, member access, object literals
- `?` try operator for `Result`
- `T?` sugar for optional fields / `Option<T>` concept

### Generics (restricted)

Allowed only for:

- `Option<T>` / `Result<T, E>`
- `req.json<T>()` only if you also provide explicit schema mapping OR restrict to schema-only decoding
  - Recommended v0: schema-only decoding to reduce magic.

No user-defined generic algorithms in v0.

---

## 4) No-traits / no-interfaces architecture

### Structural shape types

- Object/record types define required fields.
- Width subtyping allowed (extra fields OK).
- Intersections `A & B` allowed.

### Capability objects

Abstractions are values containing functions:

```ailang
type UserRepo = {
  create: fn(NewUser) effects { db.write } -> Result<User, DbError>
}
```

No `implements` anywhere.

### Effects integration

- Function types inside shapes must declare effects.
- Calling a capability field counts those effects.

---

## 5) Schemas (replace derive, avoid magic)

### `schema` blocks are explicit and required at trust boundaries

```ailang
schema CreateUserRequest {
  email: Email = validate.email
  age?: Int64 = validate.int64
}
```

### Required rule: HTTP input must be decoded via schema

- `req.json(CreateUserRequest)` is the canonical gate:
  - Input starts as `Untrusted<Bytes>`
  - Output is trusted `CreateUserRequest` or structured error

### Encoding policy (configurable)

- strict mode: `Response.json(schema, value)` required
- relaxed mode: allow `json.encode(value)` only for non-secret, non-untrusted (still typechecked)

---

## 6) Security types

### `Untrusted<T>`

- produced by HTTP/env/raw JSON extraction
- cannot flow into sinks or trusted params without explicit gate:
  - schema decode
  - validate/sanitize functions

### `Secret<T>`

- produced by secret sources (env secrets, vault)
- forbidden in logs/JSON/string formatting
- `redact` allowed, `reveal` requires `effects { secrets.reveal }` + policy allow

---

## 7) Effects (v0 set)

Minimum effects:

- `log`
- `time.now`
- `net` (inbound/outbound)
- `secrets.read`, `secrets.reveal`

Optional early:

- `db.read`, `db.write`
- `fs.read`, `fs.write`

Forbidden by default (policy):

- `shell`
- `unsafe`

---

## 8) Minimal IR + backend strategy

### MIR (minimum)

- basic blocks, locals, calls, branches, return
- explicit `Try` lowering
- spans everywhere

### Backend

Default v0 backend:

- Emit C -> compile with clang
- runtime provides stable ABI functions (`ailang_runtime.h`)

Cranelift/WASM can come later; MIR stays backend-neutral.

---

## 9) Runtime v0 requirements

Minimum runtime:

- HTTP router + request/response
- JSON parse/build
- logging + time

Optional v0.1:

- DB adapter with typed `SqlQuery` only

---

## 10) Debuggability requirements (must-have)

- Every compiler error includes:
  - span, taint origin, sink name, suggested fix
- `--emit=mir` always available
- deterministic request replay hooks (optional but strongly recommended):
  - capture inbound request + seed/time -> replay

---

## 11) Explicit out-of-scope (v0)

- macros / derives
- traits/interfaces
- advanced generics
- operator overloading
- full async/await runtime (can be added later)
- custom allocators / manual memory model

---

## 12) Acceptance tests (prove not sugar)

v0 is considered real if it passes these compile-time tests:

1. Reject `db.exec("..." + userInput)`
2. Reject `Response.html("<h1>" + q + "</h1>")` unless `HtmlSafe`
3. Reject logging/encoding a `Secret<_>`
4. Reject missing declared effects in a function that uses them
5. Require schema gate for decoding request body into trusted types
