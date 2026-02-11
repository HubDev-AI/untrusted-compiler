# AILang

AILang is a backend-focused, AI-operable programming language project. The goal is a small but real compiler pipeline with strong safety defaults, explicit effects, and security-by-construction boundaries.

## Is AILang a good idea, or just syntax sugar?

AILang can be a good idea, but only if it enforces semantics that are not reliably enforceable with TypeScript + linting alone.

### If it is only syntax sugar

If AILang is only a thin wrapper over TS/JS and does not enforce stronger guarantees, it is likely not worth the complexity. In that case, a practical stack is:

- TypeScript framework
- strict lint rules
- schema validation (for example, Zod)
- tagged SQL wrappers
- secure API wrappers
- AI coding guidelines

### When it becomes a real language

AILang is not just sugar if the compiler/toolchain enforces properties such as:

- typed trust boundaries (`Untrusted<T>` cannot reach sensitive sinks without explicit gates)
- typed sinks (`SqlQuery`, `HtmlSafe`, `UrlSafe`, `PathSafe` accepted instead of raw strings)
- effects checking (functions must declare and satisfy side-effect usage)
- deterministic debugging/replay support in the toolchain

If those checks are compile-time guarantees, AILang is a real language design, not a style guide.

### Does it need its own compiler?

It needs its own compiler front-end in the meaningful sense:

- parser
- type checker
- effects/security checker
- MIR (or equivalent IR)

The backend can initially target existing toolchains (for example, C + `clang`, or WASM runtime). A full optimizing backend is not required on day one.

### Practical litmus test

If AILang can reject these at compile time, it is clearly more than sugar:

- raw SQL string concatenation
- unescaped user input sent to HTML sink
- logging secrets
- using DB write paths without declared `db.write` effects

If it cannot, it is mostly convention.

## Current implementation status

- M0 completed: workspace + CLI skeleton + diagnostics + manifest/lockfile basics.
- M1 completed: lexer/parser/AST + `check --emit ast` + parser golden tests.

Roadmap: `/Users/vladimirtrifonov/src/ai/AILang/docs/05-ailang-master-roadmap.md`
Book docs: `/Users/vladimirtrifonov/src/ai/AILang/docs/book/README.md`
