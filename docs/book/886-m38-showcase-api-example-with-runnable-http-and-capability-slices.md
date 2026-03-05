# 886 M38 Slice: `showcase-api` Example with Runnable HTTP + Capability Slices

This slice adds a larger example project under `examples/showcase-api` to make alpha usage easier to evaluate end-to-end.

## Added Files

- `examples/showcase-api/sec4.toml`
- `examples/showcase-api/sec4.policy`
- `examples/showcase-api/src/main.ut`
- `examples/showcase-api/build/.gitignore`
- `examples/showcase-api/README.md`

## What the Example Covers

- Runnable HTTP flow (`sec4 run`) with middleware chain and multiple routes.
- Request-source intrinsics:
  - `req.query`, `req.pathParam`, `req.header`, `req.json`
- Typed gates and sanitizers:
  - `sanitize.html`, `validate.email`, `validate.uuid`, `validate.int64`, `validate.headerValue`, `validate.nonEmpty`
  - `url.public`, `url.internal`, `path.base`, `path.under` (using absolute normalized base paths)
- Typed response sinks and constructors:
  - `res.text`, `res.html`, `res.ok`, `res.setHeader`, `res.addCookie`
  - `headers.name`, `headers.value`, `cookie.build`
- Live runtime capability probes:
  - `/db/write` via `DbCap()`
  - `/fs/write-read` via `FsCap()` with absolute `path.base(...)` input
  - `/net/public` via `NetCap()`
  - `/net/internal` via `InternalNetCap()`
- Compile-checked capability slice for DB/FS/NET intrinsics in `capabilityShowcase(...)`.

## Important Runtime Bridge Note

Current bridge contract requires router handlers to be zero-arg (`fn handler()`).

Typed handler-parameter injection for capabilities is still pending, but runtime capability constructors (`DbCap()`, `FsCap()`, `NetCap()`, `InternalNetCap()`) now unblock live route usage for common operations.

This keeps the example runnable now while preserving typed capability signatures for future handler-injection work.

## Validation

- `cargo run -p sec4 -- check --path examples/showcase-api`
- `cargo run -p sec4 -- build --path examples/showcase-api --emit c-bin`
- `cargo run -p sec4 -- run --path examples/showcase-api --port 8080`
- Manual route checks from `examples/showcase-api/README.md` for:
  - `/health`
  - `/headers`
  - `/preview`
  - `/users/:id`
  - `/gates`
  - `/db/write`
  - `/fs/write-read`
  - `/net/public`
  - `/net/internal`
  - `POST /users`
