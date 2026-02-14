# Untrusted<T>

Untrusted<T> is a backend-focused, AI-operable programming language project. The goal is a small but real compiler pipeline with strong safety defaults, explicit effects, and security-by-construction boundaries.

## Is Untrusted<T> a good idea, or just syntax sugar?

Untrusted<T> can be a good idea, but only if it enforces semantics that are not reliably enforceable with TypeScript + linting alone.

### If it is only syntax sugar

If Untrusted<T> is only a thin wrapper over TS/JS and does not enforce stronger guarantees, it is likely not worth the complexity. In that case, a practical stack is:

- TypeScript framework
- strict lint rules
- schema validation (for example, Zod)
- tagged SQL wrappers
- secure API wrappers
- AI coding guidelines

### When it becomes a real language

Untrusted<T> is not just sugar if the compiler/toolchain enforces properties such as:

- typed trust boundaries (`Untrusted<T>` cannot reach sensitive sinks without explicit gates)
- typed sinks (`SqlQuery`, `HtmlSafe`, `UrlSafe`, `PathSafe` accepted instead of raw strings)
- effects checking (functions must declare and satisfy side-effect usage)
- deterministic debugging/replay support in the toolchain

If those checks are compile-time guarantees, Untrusted<T> is a real language design, not a style guide.

### Does it need its own compiler?

It needs its own compiler front-end in the meaningful sense:

- parser
- type checker
- effects/security checker
- MIR (or equivalent IR)

The backend can initially target existing toolchains (for example, C + `clang`, or WASM runtime). A full optimizing backend is not required on day one.

### Practical litmus test

If Untrusted<T> can reject these at compile time, it is clearly more than sugar:

- raw SQL string concatenation
- unescaped user input sent to HTML sink
- logging secrets
- using DB write paths without declared `db.write` effects

If it cannot, it is mostly convention.

## Current implementation status

- Core compiler/runtime path is live end-to-end:
  - parser/type/effects/security checks,
  - MIR -> C emission,
  - `clang` build/run path via `sec4`.
- Replay/capture bootstrap and runtime mock execution contracts are implemented and closure-gated (`M14`/`M15`).
- Live HTTP runtime serving is implemented with deterministic e2e coverage (`M16`), including:
  - JSON request gate + standard error/success envelopes,
  - trace correlation,
  - CORS/security-headers/CSRF/auth middleware behavior.
- CI closure audit currently enforces milestone gates through `M16`.

## Run End-to-End Now

Fast deterministic smoke check:

```bash
scripts/smoke-sec4-run-hello-api.sh
```

Manual live run:

```bash
cargo run -p sec4 -- run --path examples/hello-api
```

Deterministic one-request run (helpful for scripted checks):

```bash
cargo run -p sec4 -- run --path examples/hello-api --oneshot
```

Optional runtime body-cap override:

```bash
cargo run -p sec4 -- run --path examples/hello-api --oneshot --max-body-bytes 16384
```

Optional oneshot serve-timeout override:

```bash
cargo run -p sec4 -- run --path examples/hello-api --oneshot --serve-timeout-ms 12000
```

Then in another terminal:

```bash
curl -i \\
  -H 'Authorization: Bearer token123' \\
  http://127.0.0.1:8080/health
```

```bash
curl -i -X POST \\
  -H 'Authorization: Bearer token123' \\
  -H 'Content-Type: application/json' \\
  -H 'X-CSRF-Token: token123' \\
  -H 'Cookie: csrf=token123' \\
  --data '{}' \\
  http://127.0.0.1:8080/users
```

Roadmap: `docs/05-sec4-master-roadmap.md`
Book docs: `docs/book/README.md`
v0 scope (one-page): `docs/book/43-sec4-v0-scope.md`
security baseline: `docs/book/53-v0-security-baseline.md`

## Ecosystem naming lock

- Language title: `Untrusted<T>`
- Tooling brand: `sec4Audit`
- CLI/tooling command: `sec4`
- Security policy workflow:
  - `sec4 audit`
  - `sec4 explain`
  - `sec4 gate`
- GitHub repo: `https://github.com/HubDev-AI/untrusted-compiler`
- Source file extension: `.ut`
- Package namespaces:
  - `ut/std`
  - `ut/http`
  - `ut/sec`
