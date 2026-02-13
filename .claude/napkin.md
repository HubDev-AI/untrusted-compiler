# Napkin

## Corrections
| Date | Source | What Went Wrong | What To Do Instead |
|------|--------|-----------------|--------------------|
| 2026-02-12 | self | Added strict DB query type checks and initially broke capability-mismatch fixtures because they passed raw string SQL literals. | When tightening sink payload types, immediately align existing capability/alias fixtures to pass typed payload symbols so each fixture still isolates a single failure mode. |
| 2026-02-12 | self | Estimated forbidden-effect diagnostic columns manually in a new semantic golden and missed by 2 chars after signature edits. | After adding security fixtures with policy diagnostics, run the golden test once and copy exact spans from failure output before proceeding. |
| 2026-02-12 | self | Added a semantic fixture for `secrets.reveal` shape hardening but forgot policy-forbidden `E2002` diagnostics also fire by default. | For `secrets.reveal` semantic fixtures, either include policy diagnostics in golden output or add a valid `@allow(...)` annotation when isolating shape-only behavior. |
| 2026-02-12 | self | Started session actions before confirming `.claude/napkin.md` existed and reading it. | Always check/create and read `.claude/napkin.md` first in-session. |
| 2026-02-12 | self | Reintroduced a moved-`Span` compile error while wiring a new semantic helper call (`enforce_res_html_signature`). | Default to passing `span.clone()` into helper calls inside semantic enforcement unless the span is consumed as final use. |
| 2026-02-12 | self | Asserted wrong runtime symbol names in a CLI integration test (`ailang_rt_header_*` instead of emitted `ailang_rt_headers_*`). | Verify intrinsic runtime symbol spelling from `c_backend.rs` rewrite table or existing integration assertions before adding new expectations. |
| 2026-02-12 | self | Assumed CLI/test hangs were caused by recent code changes; issue reproduced even on reverted state and simple local binaries. | Treat executable runtime as environment-level blocker first, but re-validate before locking the assumption. |
| 2026-02-12 | self | Treated runtime hang as persistent for the full session. | Re-run full runtime verification after environment hiccups; blockers can be transient. |
| 2026-02-12 | self | Deferred baseline verification too early due blocker assumption. | Keep full baseline verification (`cargo test`, `check`, `emit ast`, `build`) in the same slice once execution recovers. |
| 2026-02-12 | self | Inserted a new Rust test block inside an existing raw string, which produced cascading parser errors. | After patching large test files, immediately inspect the surrounding lines with `nl -ba` before running broad test suites. |
| 2026-02-12 | self | Ran `cargo test` with multiple bare test-name args, which Cargo treats as unexpected arguments. | Use one test filter, or run explicit targets (`cargo test --test <name>`). |
| 2026-02-12 | self | Assumed statement-level continuation block numbering would match previous branch-first ordering. | Reserve block IDs intentionally and update MIR tests/goldens to assert the actual deterministic numbering strategy. |
| 2026-02-12 | self | Generated C `main` with `int64_t` return type, which clang rejects. | Force emitted `main` signature to return `int` even when AILang return type maps to `Int`/`Int64`. |
| 2026-02-12 | self | Used `status` as a temporary shell variable in `zsh`; it is readonly and broke a manual validation script. | Use a neutral temp variable name like `rc` for shell command exit codes. |
| 2026-02-12 | self | Added `ailang_rt_log_any` with mismatched header/source signatures (`();` vs `(void)`), causing runtime-content assertion drift. | Keep runtime ABI declarations/definitions identical and validate both via unit tests immediately after edits. |
| 2026-02-12 | self | Wrote a `res.json(schema, ...)` integration test using numeric schema placeholders, which violates strict schema-argument checks (`E4004`). | For compile-path req/res fixtures, pass real `Schema<T>`-typed symbols (for example function parameters) and keep net effects declared. |
| 2026-02-12 | self | Forgot that `url.public` / `url.internal` are modeled as `net` effects, so gate-only fixture failed with `E4002`. | When gate fixtures include URL validators, declare `effects { net }` explicitly. |
| 2026-02-12 | self | Used a function symbol (`handler`) as a value argument in CLI integration fixture; current semantic model does not resolve function names as first-class values there. | In compile-path fixtures, pass literal/place-holder values unless first-class function values are explicitly implemented. |
| 2026-02-12 | self | Replaced bare middleware names before dotted names in C intrinsic rewriting, producing invalid forms like `sec.ailang_rt_*`. | In string-based intrinsic rewrites, replace dotted forms before bare aliases to avoid partial-prefix corruption. |
| 2026-02-12 | self | Used `register` as a fixture function name; it maps directly to C and collides with the C keyword `register`. | Avoid C reserved keywords in AILang integration fixtures until backend identifier mangling is implemented. |
| 2026-02-12 | self | Added an extra schema-note text to existing `E4004` diagnostics and broke semantic golden fixtures. | Preserve established diagnostic wording unless intentionally updating goldens as part of the slice. |
| 2026-02-12 | self | Asserted an exact emitted C local declaration shape (`int64_t csp = ...`) for a non-primitive typed binding, causing brittle test failure. | Assert intrinsic call lowering substrings rather than exact local declaration spelling unless declaration shape is the behavior under test. |
| 2026-02-12 | self | Ran two Cargo test commands in parallel again, hitting package-cache lock waits and noisy failures. | Run Cargo commands sequentially in this repo to avoid lock contention. |
| 2026-02-12 | self | Used `-p ailang-cli` in tests; actual package name is `ailang`. | Use `-p ailang` for CLI crate-specific test runs. |
| 2026-02-12 | self | Inserted a new Rust test block before closing an existing raw string literal, breaking test-file parsing. | After adding tests, inspect surrounding lines with `nl -ba` to confirm raw-string boundaries are intact. |
| 2026-02-12 | self | Repeated a brittle C-backend assertion using an exact local declaration shape for `sql.q` lowering. | Assert runtime-call substrings (`ailang_rt_*`) instead of exact declaration text unless declaration shape is explicitly under test. |
| 2026-02-12 | self | Introduced a local variable named `cookie` initialized from `cookie.build(...)`, which triggered alias expansion growth (`cookie.build.build...`) and made semantic analysis appear stuck. | Guard alias-name expansion against recursive suffix growth and cap alias-resolution steps; treat namespace-shadowing call aliases as a hot path for regressions. |
| 2026-02-12 | self | Used parallel tool calls for multiple Cargo test invocations again, causing lock waits/noisy output. | Run Cargo commands sequentially in this repo; parallelize reads/searches only. |
| 2026-02-12 | self | Moved `Span` into a helper call, then reused it in the same function and hit borrow-after-move compile failure. | Pass cloned spans (`span.clone()`) when the caller still needs the original for later diagnostics. |
| 2026-02-12 | self | Inserted a new Rust test into `diagnostic_tags.rs` before closing an existing raw string literal, causing parser errors that looked unrelated (`unknown prefix`, unterminated string). | After editing Rust tests with raw strings, immediately inspect surrounding lines with `nl -ba` to verify string boundaries before running tests. |
| 2026-02-12 | self | Repeated a span move regression by passing `span` by value into a new helper and then reusing it later in the same function. | When adding helper calls inside semantic enforcement, default to `span.clone()` unless the value is consumed as the final use site. |
| 2026-02-13 | self | Added stricter `db.queryOne` row-schema checks but initially missed updating the CLI `c-bin` db/fs/net integration fixture, which then failed. | When tightening semantic call contracts, immediately audit and update affected CLI integration fixtures (`json_output.rs`) before final validation. |

## User Preferences
- Keep strict milestone flow with docs updates and frequent commits (one commit per milestone slice).
- Add all design/spec changes into docs/book and keep roadmap aligned.
- Security-first language direction is mandatory.
- Avoid adding absolute local machine paths to committed docs.

## Patterns That Work
- Small vertical M4 slices with matching docs chapter updates reduce churn and keep progress reviewable.
- Deterministic `sec.audit` findings with sample evidence improve traceability for policy/security posture.
- After transient execution issues, retrying full verification in the same session often recovers and avoids false blockers.

## Patterns That Don't Work
- Launching multiple `cargo run` commands in parallel can cause lock contention/timeouts and noisy diagnostics.

## Domain Notes
- Current focus is M4 security foundation hardening before MIR/backend milestones.
- `sec.audit` and `security_map` are central artifacts for deterministic security posture and later editor tooling.

## Session Notes (2026-02-12)
- Repo currently has no `.trellis/` directory even though AGENTS references Trellis docs.
- Implemented and validated `sec audit --history-dir` with JSON-mode stdout contract preserved.
- Added `DNS_RESOLUTION_DISABLED` sec.audit posture rule with sample-call evidence (prod + `net.ssrf.resolve_dns=false`).
- Added `PUBLIC_EGRESS_NO_DOMAIN_POLICY` sec.audit posture rule (usage-gated on public-net sink calls) plus policy ingestion for `net.public.allowed_domains`/`blocked_domains`.
- Added `CSRF_PROTECTED_METHODS_INCOMPLETE` sec.audit posture rule with deterministic missing-method evidence and middleware sample-call context.
- Added `SYMLINK_POLICY_WEAK` sec.audit posture rule plus typed policy parsing/validation for `fs.forbid_symlinks`.
- Added `REFERRER_POLICY_WEAK` sec.audit posture rule for weak security-header referrer-policy values.
- Added `XFO_DISABLED` and `NOSNIFF_DISABLED` sec.audit posture findings for baseline security-header hardening gaps.
- `REPLAY_EFFECTS_ALLOW` severity now maps by environment (`HIGH` in prod, `MEDIUM` otherwise).
- Started M5 bootstrap: added MIR module + lowering + `build --emit mir` CLI path with tests.
- Extended M5 MIR lowering with tail-`if` branch blocks and explicit `branch` terminators.
- Added fixture-based MIR golden tests to lock textual MIR output behavior.
- Extended M5 MIR control-flow lowering with tail-`match` to `switch` + per-arm blocks.
- Added `build --emit mir-json` JSON-only stdout mode for machine-readable MIR integration.
- Extended M5 MIR lowering beyond tail forms for explicit `return if` and `return match`, with shared lowering helpers and fixture coverage.
- Extended M5 MIR lowering for statement-level `if`/`match` expressions into explicit continuation CFG blocks using `goto` join targets.
- Refactored MIR lowering into recursive return/continuation CFG helpers to lower nested control flow in branch bodies and block tails.
- Added MIR block-id canonicalization pass with target remapping for branch/goto/switch terminators to stabilize textual and JSON output.
- Started M6 with MIR-to-C emission (`build --emit c`) and added core+CLI tests for generated C output.
- Extended M6 with `build --emit c-bin`: writes generated C and compiles a runnable binary via `clang`.
- Added `examples/hello/build/.gitignore` to keep generated C/binary artifacts out of git status while retaining `security_map.json`.
- Updated `examples/hello/build/.gitignore` to ignore all generated build artifacts (including security_map) for a clean working tree.
- Wired `ailang run` to build via `c-bin` and execute the produced binary; added clang-gated integration coverage.
- Added M6 runtime ABI scaffolding: C backend now includes `ailang_runtime.h`, CLI writes runtime header/source into `build/`, and clang compiles generated + runtime translation units together.
- Routed scalar C return paths through runtime ABI identity helpers (`ailang_rt_identity_i64` / `ailang_rt_identity_bool`) so runtime linkage is exercised by emitted function bodies.
- Added intrinsic call rewriting in C emission for `time.now`/`time_now` -> `ailang_rt_time_now` with runtime stub coverage and C emitter tests.
- Added non-trivial `c-bin` integration coverage using a temp project fixture with helper-function call + `if/else` control flow.
- Moved runtime ABI C sources from Rust string literals into `runtime/c/` files and switched emitter helpers to `include_str!` those canonical runtime assets.
- Added clang-gated CLI integration coverage for `time.now` projects to validate semantic effects checks plus intrinsic rewrite/runtime linkage through full `c-bin` builds.
- Added log intrinsic C-lowering coverage (`log.info/warn/error/emit` -> `ailang_rt_log_any`) with runtime stubs and end-to-end `c-bin` integration tests.
- Added req/res intrinsic C-lowering coverage (`req.json`, `res.json`, `res.html`) with runtime stubs and end-to-end `c-bin` integration tests using valid `Schema<T>` parameters.
- Added header/cookie intrinsic C-lowering coverage (`res.setHeader`, `res.addCookie`) with runtime stubs and end-to-end `c-bin` integration tests.
- Added db/fs/net intrinsic C-lowering coverage (`db.*`, `fs.*`, `httpClient.get*`) with runtime stubs and end-to-end `c-bin` integration tests using capability-typed parameters.
- Added secrets intrinsic C-lowering coverage (`secrets.get`, `secrets.reveal`) with runtime stubs plus end-to-end `c-bin` coverage for `secrets.get` (non-forbidden default policy path).
- Added validator/sanitizer/url/path gate intrinsic C-lowering coverage (`validate.*`, `sanitize.html`, `url.*`, `path.under`) with runtime stubs and end-to-end `c-bin` integration tests.
- Added HTTP router intrinsic C-lowering coverage (`http.router/get/post/serve`) with runtime stubs and end-to-end `c-bin` integration tests.
- Added `examples/hello-api` as an M7 bootstrap sample plus clang-gated `c-bin` integration coverage asserting router + req/res lowering in generated C.
- Added `ailang run` integration coverage for `examples/hello-api` so both build and run flows are pinned for the bootstrap API sample.
- Added security-middleware intrinsic C-lowering coverage (`withSecurityHeaders`, `withCors`, `withCsrf`, `withAuth`) with runtime stubs and end-to-end `c-bin` integration tests.
- Added HTTP surface primitive type support (`Router`, `Request`, `Response`, `HttpError`, `Handler`) so API-shaped signatures pass semantic type resolution in compile-path fixtures.
- Added policy-config intrinsic C-lowering coverage (`sec.defaultHeaders`, `cors.fromPolicy`, `csrf.fromPolicy`, `auth.fromPolicy`) with runtime stubs and end-to-end `c-bin` integration tests.
- Removed absolute local machine paths from M7 `hello-api` book chapters to keep docs repository-safe.
- Added success-envelope intrinsic bridge (`res.ok`, `res.okMeta`) across semantic checks, C lowering, runtime stubs, security-map tagging, and req/res `c-bin` integration coverage.
- Added request-source intrinsic bridge (`req.body`, `req.query`, `req.pathParam`, `req.header`) across C lowering/runtime ABI/tests and exercised them in `examples/hello-api`.
- Added auth helper intrinsic bridge (`auth.require`, `auth.requireRole`) across semantic registry, C lowering, runtime ABI, and `c-bin` integration coverage.
- Added typed CORS origin gate bridge (`cors.origin`) with semantic gate enforcement, runtime lowering, security-map tags, and `c-bin` integration coverage.
- Added CSRF token-issue helper bridge (`csrf.issueToken`) with `net` effect semantics, runtime lowering, security-map effect tag, and `c-bin` integration coverage.
- Added CSP builder bridge (`sec.csp`, `sec.cspAdd`) and security-config surface type-name catalog so spec-shaped config signatures compile through `c-bin`.
- Added structured log builder bridge (`log.event/field/obj/str/i64/bool/redacted`) so spec logging constructors compile through semantic + runtime bridge.
- Added helper bridge for `path.base`, `headers.name/value`, and `secrets.redact`, including runtime ABI stubs and `security_map` gate tags.
- Added JSON helper bridge for `json.decode` / `json.encode` plus `Json` primitive type support and `security_map` gate/sink tagging.
- Added std-error helper bridge for `err.*` constructors/enrichers across semantic intrinsics, C lowering, runtime ABI stubs, and clang-gated `c-bin` integration tests.
- Added `res.text` plain-text response bridge across semantic intrinsics, C lowering, runtime ABI stubs, and req/res integration coverage.
- Added structured log-event helper bridge for `log.attrRedacted`, `log.withAttr`, `log.withHttp`, and `log.withError` plus `LogAttr`/`LogEvent` primitive catalog support.
- Added DB transaction helper bridge for `db.tx`/`db.execTx`, including `db.tx` effect registration, capability enforcement, SQL sink coverage, and security-map tag/role extensions.
- Added SQL query-construction helper bridge for `sql.q`, including semantic namespace support, runtime lowering, and `security_map` `gate.sql.parameterize` tagging.
- Added cookie-builder helper bridge for `cookie.build`, including runtime lowering and `security_map` `gate.cookie.build` tagging, and hardened alias resolution against runaway suffix expansion.
- Route handler wiring needed function-symbol identifiers to be accepted as value expressions; semantic fallback to cataloged function names unblocks `http.get(..., health)` style samples without weakening unknown-name diagnostics for non-functions.
- Added initial semantic route contract checks for `http.get/http.post` (string path + function-symbol handler + handler `effects { net }`) and updated router fixtures to explicit path/handler forms.
- Reconfirmed that even two-test parallel cargo runs create lock-wait noise; keep all Cargo invocations strictly sequential during implementation/verification.
- Added typed semantic contracts for canonical router security bootstrap calls (`*.fromPolicy`, `*.with*`) so bootstrap chains are now shape-checked and return `Router`/typed configs.
- Added HTTP call-shape checks for router intrinsics (`http.get/post` require `Router`, `http.serve` requires numeric port + `Router`), reducing placeholder-style misuse in API wiring.
- Added temporary bridge-stage handler compatibility rule: route handlers must be zero-arg until runtime dispatch supports typed handler signatures; keep examples/tests aligned to that contract.
- Added bridge-stage route-handler return contract (`Int`/`Int64` only) so semantic checks align with current generated C handler expectations.
- Hardened `req.json` gate checks beyond missing-arg validation: schema argument now rejects numeric/bool/untrusted/secret shapes and emits tagged schema/security diagnostics.
- Added strict call-shape checks for `req.json` arity and `res.text(status, body)` typing, and updated CLI req/res integration fixtures to use string text bodies.
- Added `res.html` sink signature hardening (`HtmlSafe`-only + exact arity) with tagged diagnostics, semantic fixture coverage, and updated req/res `c-bin` integration fixture to use `sanitize.html(...)` gate flow.
- Added typed header/cookie sink signature hardening (`res.setHeader` with `HeaderName`/`HeaderValue`, `res.addCookie` with `Cookie`) with tagged diagnostics and updated header/cookie `c-bin` integration fixture to use `headers.name/value` constructors.
- Added signature hardening for `headers.name/value` constructors (exact one-arg `String` contract) and updated gate/header integration fixtures to use string inputs.
- Added request-source signature hardening for `req.query`/`req.pathParam`/`req.header` (single `String` key argument), with schema/security-tagged diagnostics and updated req/res + cors integration fixtures.
- Added `path.base` signature hardening (single `String` argument) with constructor diagnostics and updated gate integration fixtures to use explicit base-path literals.
- Added `req.body` signature hardening (`req.body(ctx, request)` with typed `Ctx`/`Request` arguments) and updated req/res integration fixtures accordingly.
- Tightened trust-gate arity contracts: single-input gates now require exactly one argument and `path.under` requires exactly two, with schema/security-tagged diagnostics and fixture coverage.
- Added DB sink arity hardening for `db.exec` / `db.execTx` / `db.queryOne` to require explicit query-bearing call shapes before typed-query enforcement, with sink-tagged diagnostics.
- Added net sink arity hardening for `httpClient.get` / `httpClient.getInternal` so URL arguments are required in compact and context-first forms, with sink-tagged diagnostics.
- Added FS sink arity hardening for `fs.read` / `fs.write` so path/payload arguments are required in compact and context-first forms, with sink-tagged diagnostics.
- Added `secrets.get` arity hardening so secret-name arguments are required in compact and context-first forms, with secret-tagged diagnostics.
- Added `secrets.redact` arity hardening so exactly one secret argument is required, with secret-tagged diagnostics.
- Added auth-helper call-shape hardening: `auth.require` now requires one `Ctx` argument and `auth.requireRole` now requires `(Ctx, String)`, with `E4001` security-tagged diagnostics and aligned c-bin integration fixtures.
- Added `secrets.reveal` call-shape hardening so only `(secretsCap, secret)` and `(ctx, secretsCap, secret)` are accepted, with secret-tagged diagnostics and aligned security_map/sec_audit fixtures.
- Added `db.tx` call-shape hardening so only `(dbCap)` and `(ctx, dbCap)` forms are accepted, with capability-tagged diagnostics and matching semantic/tag fixture coverage.
- Added `db.tx` context-first type hardening: two-argument form now requires `Ctx` in argument 1, with capability-tagged diagnostics and dedicated semantic/tag fixture coverage.
- Added DB sink context-first type hardening for `db.exec`, `db.execTx`, and `db.queryOne`, so slot-1 context must be `Ctx` in context-first forms, with sink-tagged diagnostics and dedicated semantic fixtures.
- Added net sink context-first type hardening for `httpClient.get` and `httpClient.getInternal`, so slot-1 context must be `Ctx` in context-first forms, with sink-tagged diagnostics and dedicated semantic fixtures.
- Added FS sink context-first type hardening for `fs.read` and `fs.write`, so slot-1 context must be `Ctx` in context-first forms, with sink-tagged diagnostics and dedicated semantic fixtures.
- Added secret-source context-first type hardening for `secrets.get`, so slot-1 context must be `Ctx` in context-first form, with secret-tagged diagnostics and dedicated semantic fixture coverage.
- Added secret-reveal context-first type hardening for `secrets.reveal`, so slot-1 context must be `Ctx` in context-first form, with secret-tagged diagnostics and dedicated semantic fixture coverage.
- Added secret-reveal payload typing hardening so `secrets.reveal` value argument must be `Secret<_>`, with secret-tagged diagnostics and dedicated semantic fixture coverage.
- Added secret-source name typing hardening so `secrets.get` name argument must be `String`, with secret-tagged diagnostics and dedicated semantic fixture coverage.
- Added DB sink query typing hardening so `db.exec`, `db.execTx`, and `db.queryOne` require `SqlQuery` payloads in compact/context-first forms, with sink-tagged diagnostics and fixture alignment across alias/capability tests.
- Added net sink URL typing hardening so `httpClient.get` requires `PublicUrl` and `httpClient.getInternal` requires `InternalUrl`, with sink-tagged diagnostics and typed-URL fixture alignment.
- Added FS sink path typing hardening so `fs.read`/`fs.write` require `PathSafe` path arguments in compact/context-first forms, with sink-tagged diagnostics and typed-path fixture alignment.
- Added secret-redact payload typing hardening so `secrets.redact` requires `Secret<_>`, with secret-tagged diagnostics and fixture/tag coverage.
- Added `db.queryOne` row-schema argument hardening so numeric/boolean placeholders are rejected with schema-tagged diagnostics, and aligned the db/fs/net `c-bin` integration fixture to use a schema-descriptor row argument.
- Added `sql.q` signature hardening so the helper enforces `(template, params)` with a `String` template argument, and aligned CLI/core integration fixtures to use string SQL templates.
- Added `cookie.build` signature hardening so the helper enforces `(name, value)` with string arguments, and aligned header/cookie integration fixtures and docs to use typed cookie-constructor values.
- Added `err.withDetail` value-safety hardening so detail keys must be strings and detail values reject `Secret<_>`/`Untrusted<_>` payloads, with tagged diagnostics and aligned error-builder integration fixtures.
- Added `err.withPath` path typing hardening so error-path payloads must be strings, with tagged diagnostics and aligned error-builder integration fixtures.
- Added `err.withLimit` argument hardening so limit names are strings and max/actual values are numeric, with tagged diagnostics and aligned error-builder integration fixtures.
- Added `err.withDependency` argument hardening so dependency/operation fields are strings and retryable is boolean, with tagged diagnostics and aligned error-builder integration fixtures.
- Added `err.internal` message hardening so internal-error constructors require a single string message argument, with tagged diagnostics and aligned error-builder integration fixtures.
- Added `err.validation` constructor hardening so validation error code/message arguments are string-typed, with tagged diagnostics and aligned error-builder integration fixtures.
- Added `err.auth` constructor hardening so auth error code/message are string-typed and status is numeric, with tagged diagnostics and aligned error-builder integration fixtures.
- Added `err.notFound` constructor hardening so not-found error code/message arguments are string-typed, with tagged diagnostics and aligned error-builder integration fixtures.
- Added `err.conflict` constructor hardening so conflict error code/message arguments are string-typed, with tagged diagnostics and aligned error-builder integration fixtures.
- Added `err.rateLimit` constructor hardening so rate-limit code/message are string-typed and retry-after values are numeric, with tagged diagnostics and aligned error-builder integration fixtures.
- Added CSP builder signature hardening so `sec.csp` is zero-arg and `sec.cspAdd` enforces `(CspPolicy, String, String)`, with tagged diagnostics and aligned policy-config integration fixtures.
- Added `json.encode` schema-argument hardening so encoding requires `(schema, value)` with schema-shape validation (and typed value checks when schema is `Schema<T>`), with schema-tagged diagnostics and aligned json-helper fixtures.
