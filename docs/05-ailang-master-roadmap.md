# AILang Master Roadmap (From Line 1 to Minimal Real Working Compiler)

This roadmap is the execution plan for building a minimal, real, runnable AILang implementation.

It explicitly includes a parallel documentation workflow so `docs/` evolves into a book-quality project history and technical guide.

## Current Status (2026-02-11)

- M0 bootstrap completed and committed.
- M1 frontend bootstrap completed:
  - Lexer/token model with span-aware diagnostics.
  - Parser/AST for `fn`, `struct`, `enum`, blocks, expressions, `match`, and generic types (`Option`/`Result` forms).
  - `ailang check --emit ast` output wired into CLI.
  - Parser golden fixtures added in `compiler/ailang-core/tests/fixtures/parser`.
- M2 semantic bootstrap completed:
  - Name resolution for declared types/functions and local identifiers.
  - Minimal type checker for bindings, returns, calls, operators, and branch compatibility.
  - Match exhaustiveness checks for `Bool`, user enums, and `Option`/`Result` forms.
  - Semantic golden fixtures added in `compiler/ailang-core/tests/fixtures/semantic`.
- M3 effects slice completed:
  - Parser support for `effects { ... }` on function declarations.
  - Semantic validation of effect names and duplicate declarations.
  - Enforcement of `used_effects` subset of `declared_effects` for function bodies.
  - Effect golden fixtures added under semantic tests.
- Security-first baseline documentation has been expanded and locked as mandatory input for upcoming milestones.
- Roadmap is now realigned to insert a dedicated security-hardening milestone before MIR/backend work.
- M0 through M3 implementation is complete.
- M4 implementation is in progress:
  - Policy file loading/parsing is wired into semantic analysis entry flow.
  - Forbidden-effect checks are policy-driven.
  - Capability-required intrinsic calls are enforced with dedicated diagnostics (`E2003`, `E2004`).
  - Initial M4 policy and semantic golden tests are in place.
  - `security_map` metadata generation is implemented for baseline sensitive calls and middleware tags.
  - `@allow(...)` annotations are validated during analysis and emitted into `security_map.allows` for audit reporting.
  - `ailang sec audit` CLI command is implemented with deterministic text/json findings and threshold gating.
  - `sec.audit` includes deterministic allowlist hygiene findings (high-risk bypasses, expiry/soon-expiry, exception-count posture signal) plus expiry-window aging metrics and explicit severity-input thresholds.
  - `sec.audit` now supports optional baseline comparison (`--baseline <audit.json>`) and emits deterministic trend deltas:
    - baseline policy hash/risk score,
    - risk and finding-count deltas,
    - per-severity deltas,
    - added/resolved finding ID sets.
  - `sec.audit` now supports explicit report persistence (`--write-report <path>`) to support baseline capture and opt-in trend history workflows.
  - `sec.audit` now supports opt-in history capture (`--history-dir <path>`):
    - writes timestamped JSON reports per run,
    - auto-loads the latest history report as baseline when `--baseline` is not provided,
    - preserves JSON-only stdout contract in JSON mode while emitting history/baseline artifact hints to stderr.
  - Diagnostics now carry structured tag metadata (`security`, `taint`, `secret`, `policy`, `effects`, `capability`, `schema`, `sink`) for editor/LSP-oriented consumers while preserving current text rendering.
  - CLI now supports machine-readable diagnostics output via `ailang check --emit diagnostics-json`, exposing spans/codes/notes/tags as JSON for tooling integration.
  - Machine-readable CLI output contracts are now strict:
    - `ailang check --emit diagnostics-json` prints JSON-only payloads on stdout in both success and failure paths.
    - `ailang sec audit --format json` prints JSON-only report payload on stdout; human hint lines (for example `security map: ...`) are emitted to stderr.
  - CLI integration tests now verify parseable JSON stdout contracts for `check --emit diagnostics-json` and `sec audit --format json`.
  - New diagnostic-tag tests assert tags for representative sink, capability, policy, and schema violations.
  - Parser/semantic/security-map now support dotted stdlib call names (`db.exec`, `req.json`, `cors.withCors`, etc.) in addition to underscore intrinsic aliases.
  - Callable/member resolution now seeds namespace aliases from capability-typed parameters/bindings (`DbCap`, `TxCap`, `NetCap`, `InternalNetCap`, `FsCap`, `SecretsCap`):
    - direct member calls like `repo.exec(...)` resolve to canonical sink symbols (`db.exec`) without requiring `let repo = db`.
    - helper-forwarded callable members like `getExec(repo)` also resolve to canonical symbols for semantic checks and `security_map`.
  - Semantic flow checks now reject `Secret<_>`/`Untrusted<_>` values across log, JSON, SQL, URL/net, filesystem, and header/cookie sinks with explicit diagnostics (`E1002`, `E1003`, `E1004`, `E1005`).
  - Sink-flow origin notes in diagnostics are now callable-summary aware for call expressions, include compact forwarding-chain context for helper-forwarded sources, and propagate through `let`-bound identifier flows into sinks.
  - Request boundary trust-gate semantics are now enforced:
    - `req.body` is typed as `Untrusted<Bytes>`.
    - `req.query`, `req.pathParam`, and `req.header` are typed as `Untrusted<String>`.
    - `req.json(schema)` now requires an explicit schema argument (`E4001` when missing).
  - New semantic fixtures cover missing schema gate arguments and untrusted `req.query` to SQL sink rejection.
  - Core validator/sanitizer trust gates are now typed and contract-checked:
    - `validate.headerValue` -> `HeaderValue`, requires `Untrusted<String>`.
    - `sanitize.html` -> `HtmlSafe`, requires `Untrusted<String>`.
    - `path.under` / `validate.pathUnder` -> `PathSafe`, requires `(PathSafe, Untrusted<String>)`.
    - `url.public` -> `PublicUrl` and `url.internal` -> `InternalUrl`.
  - Additional semantic fixtures cover trust-gate contract violations and valid gate input flows.
  - `security_map` tag coverage was expanded for these trust gates and HTTP source boundaries:
    - call tags now include `gate.header.value`, `gate.sanitize.html`, `gate.path.under`, `gate.url.public`, `gate.url.internal`.
    - source tags now include dotted/underscore variants of `req.body`, `req.query`, `req.header`, and `req.pathParam`.
    - symbol registry coverage and tests were extended to keep `sec.audit` marker detection deterministic.
  - `[logging]` and `[sql]` policy sections are now ingested into the typed policy model.
  - `[net.public]` domain policy lists are now ingested into the typed policy model:
    - `allowed_domains`
    - `blocked_domains`
  - `[fs]` symlink posture is now ingested/validated in the typed policy model:
    - `forbid_symlinks` (`off|warn|enforce`)
  - `sec.audit` now emits deterministic logging/SQL posture findings:
    - `LOG_STRUCTURED_ONLY_DISABLED`
    - `LOG_REMOTE_IP_ENABLED`
    - `LOG_USER_AGENT_ENABLED`
    - `SQL_RAW_ALLOWED_BY_POLICY`
    - `SQL_LIMIT_RULE_DISABLED`
  - logging posture evidence is now uniformly callsite-backed:
    - `LOG_USER_AGENT_ENABLED` now includes bounded deterministic `sampleCalls` evidence, aligned with other logging findings.
  - CORS reflection posture is now explicit and auditable:
    - policy parser supports `cors.reflect_origin` and rejects forbidden reflection (`P6003`) when `cors.forbid_reflect_origin=true`.
    - `security_map` middleware attrs now include `reflectOrigin` for `middleware.cors`.
    - `sec.audit` now emits `CORS_REFLECT_ORIGIN_ENABLED` with deterministic `sampleCalls` evidence when origin reflection is enabled.
  - policy parser now validates `sql.require_limit_on_select` values (`off|warn|enforce`) with dedicated diagnostics.
  - `security_map` now adds callsite SQL hygiene tags (`sql.select_without_limit`) for SQL sink calls with unbounded `SELECT` literals.
  - `sec.audit` now emits `SQL_SELECT_WITHOUT_LIMIT` with policy-mapped severity:
    - `MEDIUM` for `sql.require_limit_on_select = "warn"`.
    - `HIGH` for `sql.require_limit_on_select = "enforce"`.
  - Tests now cover both SQL hygiene tag extraction and `sec.audit` severity mapping for warn/enforce modes.
  - SQL keyword scanning now ignores quoted SQL strings and SQL comments, reducing false positives/negatives in `sql.select_without_limit` tagging.
  - Trust-gate enforcement now covers broader validator/sanitizer families (`validate.*`, `sanitize.*`) for `Untrusted<String>` input contracts.
  - Intrinsic return typing now includes core validator outputs (`Email`, `Uuid`, `Int64`, and `String` for `validate.nonEmpty`).
  - `security_map` gate marker coverage now includes validator tags:
    - `gate.validate.email`
    - `gate.validate.uuid`
    - `gate.validate.int64`
    - `gate.validate.non_empty`
  - Semantic and security_map tests now cover these expanded validator gate paths.
  - Policy model now includes `json.require_schema_for_encode` (default strict mode enabled).
  - Semantic checks now enforce strict JSON encode schema requirements on `res.json(...)` calls (`E4004` when schema argument is missing).
  - Semantic fixtures now include strict-mode rejection for single-argument JSON responses and a valid schema-argument encoding case.
  - Policy tests now cover toggling strict JSON encode schema mode.
  - Strict JSON encode checks now validate full call signatures:
    - `res.json(schema, value)`
    - `res.json(status, schema, value)` with numeric status.
  - JSON strict-mode diagnostics now include:
    - invalid argument count,
    - non-numeric status,
    - invalid schema argument type/taint.
  - JSON sink flow checks now evaluate only the value argument, avoiding false positives on status/schema arguments.
  - Additional semantic fixtures now cover invalid status type, invalid schema type, invalid arity, and valid status+schema+value encoding.
  - Generic `Schema<T>` is now recognized in semantic type rules.
  - `res.json` now enforces typed schema-value pairing when schema argument is `Schema<T>`:
    - compatible payload types pass,
    - mismatches emit `E4004` with expected/actual type notes.
  - Semantic fixtures now cover both valid and invalid typed schema-value pairing cases.
  - `security_map` call records now include optional `arg_roles` metadata for explainability.
  - Role labels are emitted for core sensitive API calls (for example capability/query/url/schema/value/path).
  - JSON role labels adapt to the detected call signature form.
  - security_map tests now assert argument-role metadata presence for representative sink calls.
  - `security_map` call records now include optional `origin_edges` metadata for argument-level provenance.
  - Origin edges capture argument index, canonical origin labels, and source/gate tag context.
  - Local `let`-bound origin propagation is tracked for call/member/unary expression shapes.
  - Origin inference now also propagates through:
    - binary expressions with single-origin or same-origin operands,
    - `if`/`match` branches when origin is consistent,
    - block-tail expressions with local shadow bindings.
  - security_map now computes simple function origin summaries and propagates one-hop forwarding origins across function calls.
  - security_map tests now assert sink-argument origin tracing for `req.query -> db.exec` flow.
  - security_map tests now also assert deterministic origin tracing through composite expression wrappers.
  - security_map tests now also assert one-hop interprocedural forwarding flows (`queryParam -> passThrough -> db.exec`).
  - Function-origin summaries now converge iteratively, so source-origin tags propagate across deeper forwarding chains (`queryParam -> passThrough -> passthroughTwice -> wrap -> db.exec`).
  - `security_map.origin_edges` now include deterministic provenance trace chains (`trace`) showing multi-hop source-to-sink forwarding steps.
  - `sec.audit` findings now include bounded `sampleCalls` evidence for key finding families.
  - Sample call evidence includes callee, source location, argument roles, and origin edges when present.
  - Callsite evidence coverage currently includes:
    - `SQL_SELECT_WITHOUT_LIMIT`
    - `SECRETS_REVEAL_USED`
    - `LOG_STRUCTURED_ONLY_DISABLED`
    - `LOG_REMOTE_IP_ENABLED`
    - `XFO_DISABLED`
    - `NOSNIFF_DISABLED`
    - `REFERRER_POLICY_WEAK`
    - `SQL_RAW_ALLOWED_BY_POLICY`
    - `SQL_LIMIT_RULE_DISABLED`
    - `INTERNAL_NET_ENABLED_NO_ALLOWLIST`
    - `FS_ENABLED_NO_BASE_ALLOWLIST`
    - `SYMLINK_POLICY_WEAK`
    - `CSRF_PROTECTED_METHODS_INCOMPLETE`
    - `PUBLIC_REDIRECTS_ENABLED_WITHOUT_REVALIDATION`
    - `DNS_RESOLUTION_DISABLED`
    - `PUBLIC_EGRESS_NO_DOMAIN_POLICY` (when public-net sink calls exist and both domain lists are empty)
    - `CAPTURE_REDACTION_INCOMPLETE`
    - `CAPTURE_ALL_IN_PROD`
    - `REPLAY_EFFECTS_ALLOW`
    - CORS/security-headers/auth/CSRF posture families via middleware-tagged samples
    - allowlist bypass families via bypass-tag call sampling (`SECRETS_REVEAL_ALLOWLISTED`, `INTERNAL_NET_CALL_ALLOWLISTED`)
    - non-call exception posture snapshot evidence for `ALLOW_COUNT_HIGH` (`sampleExceptions`)
  - sec_audit tests now assert deterministic sample-call evidence presence for these findings.
  - sec_audit tests now also assert suppression of `PUBLIC_EGRESS_NO_DOMAIN_POLICY` when `net.public.allowed_domains` is configured.
  - `REPLAY_EFFECTS_ALLOW` severity is now environment-aware:
    - `HIGH` in `prod`
    - `MEDIUM` in non-production environments
  - `sec.audit` text rendering now includes sample-call previews with provenance trace snippets when available.
  - `sec.audit` now emits deterministic exception-expiry rollup findings (`ALLOW_EXPIRY_WINDOW_ROLLUP`) with sampled exception evidence.
  - Capability enforcement now supports both compact and context-first stdlib signatures for core sensitive families (db/net/fs/secrets).
  - `E2003`/`E2004` diagnostics now report the precise capability argument index for these call forms.
  - Sink-flow argument indexing now adapts to call shape so context/capability arguments are excluded from payload checks.
  - `security_map` SQL query extraction and argument roles now support context-first `db.exec`/`db.queryOne` forms.
  - Semantic/security_map fixtures now cover valid and invalid context-first capability paths.
  - Typed stdlib symbol resolution now supports local alias/value-call paths:
    - semantic analysis resolves local callable aliases to canonical intrinsic/function symbols before security checks,
    - `security_map` resolves alias-invoked callsites to canonical callee names for deterministic tags/roles/origin edges.
  - Semantic/security_map fixtures now cover alias-invoked intrinsic calls (`let exec = db.exec; exec(...)`).
  - Alias resolution now also supports member value-call paths rooted in typed stdlib/capability stems (`let repo = db; repo.exec(...)`), preserving canonical `db.exec` enforcement and metadata.
  - Callable-forwarding summaries now allow interprocedural alias resolution through helper functions (`let exec = getExec(); exec(...)` where `getExec` forwards `db.exec`).
- Security posture specs were expanded with:
  - typed security middleware baseline (`CORS + security headers + CSRF + auth`),
  - deterministic `sec.audit` contract,
  - compiler-emitted security metadata tags for robust audit tooling.
- M5 bootstrap has started:
  - backend-neutral MIR module is now implemented in `ailang-core` (`MirProgram`, `MirFunction`, `MirBlock`).
  - AST function bodies now lower into deterministic single-block MIR (`bb0`) with explicit `return` terminators.
  - tail `if` expressions now lower into explicit branch control-flow with multiple blocks (`bb0` -> `bb1` / `bb2`).
  - tail `match` expressions now lower into `switch` terminators with per-arm blocks.
  - explicit `return if` and `return match` expressions now lower into explicit branch/switch blocks (not inline expression returns).
  - statement-level `if`/`match` expression statements now lower into explicit continuation CFG blocks using `goto` join targets.
  - nested `if`/`match` control flow inside branch bodies/block tails now lowers recursively into explicit CFG blocks in both return and continuation contexts.
  - MIR output now applies deterministic canonical block-id remapping after lowering, including terminator target rewrites.
  - `ailang build --emit mir` now prints textual MIR for inspection.
  - `ailang build --emit mir-json` now emits machine-readable MIR JSON (JSON-only stdout mode).
  - MIR unit tests, MIR fixture-based golden tests, and CLI integration tests cover lowering and emit-path behavior.
- M6 bootstrap has started:
  - `ailang build --emit c` now emits C source from lowered MIR.
  - `ailang build --emit c-bin` now writes generated C and compiles a runnable binary via `clang`.
  - `ailang run` now executes binaries produced via the C compile pipeline.
  - C backend compile flow now emits explicit runtime ABI artifacts (`ailang_runtime.h` + `ailang_runtime.c`) and links them with generated C.
  - Generated C return paths now route scalar returns through runtime ABI identity intrinsics (`ailang_rt_identity_i64`, `ailang_rt_identity_bool`).
  - C emission now rewrites `time.now` intrinsic calls to runtime ABI symbol `ailang_rt_time_now`, with runtime header/source coverage.
  - CLI integration now includes a non-trivial `c-bin` fixture covering function calls + `if/else` control flow end-to-end.
  - Runtime ABI C assets are now externalized under `runtime/c/` and emitted via `include_str!` from canonical runtime files.
  - CLI integration now includes end-to-end `time.now` intrinsic coverage through `c-bin` builds and runnable binaries.
  - C emission now lowers `log.info/warn/error/emit` intrinsic calls to runtime symbol `ailang_rt_log_any`, with runtime stub and `c-bin` integration coverage.
  - C emission now lowers `req.json`, `res.json`, and `res.html` intrinsics to runtime symbols with stub implementations and `c-bin` integration coverage.
  - C emission now lowers `res.setHeader` and `res.addCookie` intrinsics to runtime symbols with stub implementations and `c-bin` integration coverage.
  - C emission now lowers core IO intrinsics (`db.*`, `fs.*`, `httpClient.get*`) to runtime symbols with stub implementations and `c-bin` integration coverage.
  - C emission now lowers `secrets.get` / `secrets.reveal` intrinsics to runtime symbols with stub implementations (plus `secrets.get` `c-bin` integration coverage).
  - C emission now lowers validator/sanitizer/url/path gate intrinsics (`validate.*`, `sanitize.html`, `url.*`, `path.under`) to runtime symbols with stub implementations and `c-bin` integration coverage.
  - Core and CLI tests cover the C emit path (`c_backend` + CLI build output checks).
- M7 bootstrap has started:
  - Semantic layer now recognizes router intrinsics (`http.router`, `http.get`, `http.post`, `http.serve`) with `http.serve` requiring `effects { net }`.
  - C emission now lowers router intrinsics to runtime symbols (`ailang_rt_http_router`, `ailang_rt_http_route_get`, `ailang_rt_http_route_post`, `ailang_rt_http_serve`).
  - Runtime ABI now includes router bridge stubs for these symbols, with clang-gated `c-bin` integration coverage.
  - Added `examples/hello-api` bootstrap sample and clang-gated `c-bin` integration coverage asserting router + req/res lowering in generated C.
  - Added clang-gated `ailang run` integration coverage for `examples/hello-api`.
  - Semantic + C runtime bridge now includes security middleware intrinsics (`withSecurityHeaders`, `withCors`, `withCsrf`, `withAuth`) with dotted namespace forms.
  - Semantic primitive type catalog now includes core HTTP surface names (`Router`, `Request`, `Response`, `HttpError`, `Handler`) for API-shaped signatures.
  - Semantic + C runtime bridge now includes standard error-model helper intrinsics (`err.validation`, `err.auth`, `err.notFound`, `err.conflict`, `err.rateLimit`, `err.internal`, `err.withPath`, `err.withDetail`, `err.withLimit`, `err.withDependency`, `err.withCause`) with runtime stubs and clang-gated `c-bin` coverage.
  - Semantic + C runtime bridge now includes `res.text` plain-text response intrinsic with runtime stubs and req/res `c-bin` integration coverage.
  - Semantic + C runtime bridge now includes structured log-event helper intrinsics (`log.attrRedacted`, `log.withAttr`, `log.withHttp`, `log.withError`) with runtime stubs and log-builder integration coverage.
  - Semantic + C runtime bridge now includes transaction helpers (`db.tx`, `db.execTx`) with capability/effect enforcement, runtime stubs, and db/fs/net integration coverage.
  - Semantic + C runtime bridge now includes query-construction helper `sql.q` with runtime stubs and security-map gate tagging (`gate.sql.parameterize`).
  - Semantic + C runtime bridge now includes `cookie.build` typed-cookie helper with runtime stubs and `security_map` gate tagging (`gate.cookie.build`).
  - Alias resolution now includes recursion guards for namespace-shadowing call aliases, preventing runaway expansion patterns like `cookie.build.build...` during semantic/security analysis.
  - Semantic analysis now accepts declared function symbols as value expressions for handler-style routing/wiring (for example `http.get(router, "/health", health)`).
  - `examples/hello-api` now uses explicit `/health` and `/users` route paths with function-symbol handlers (`health`, `createUser`) and policy-driven middleware wiring chain (`security headers -> CORS -> CSRF -> auth`).
  - Route registration semantics now enforce an initial handler contract for `http.get`/`http.post`:
    - path argument must be `String`,
    - handler argument must resolve to a declared function symbol,
    - handler must declare `effects { net }`.
  - Canonical router security bootstrap calls now have typed semantic contracts:
    - `sec.defaultHeaders()` -> `SecurityHeadersConfig`
    - `cors.fromPolicy()` -> `CorsConfig`
    - `csrf.fromPolicy()` -> `CsrfConfig`
    - `auth.fromPolicy()` -> `AuthConfig`
    - `sec.withSecurityHeaders(router, cfg)` / `cors.withCors(router, cfg)` / `csrf.withCsrf(router, cfg)` / `auth.withAuth(router, cfg)` require typed `(Router, Config)` arguments and return `Router`.
  - HTTP intrinsic call-shape checks now enforce:
    - `http.get/post(router, path, handler)` require first argument `Router`,
    - `http.serve(port, router)` requires numeric port and `Router` as second argument.
  - Route handler compatibility now requires zero-argument handlers in the current v0 runtime bridge (`http.get/post`), with compile-time diagnostics for parameterized handler functions.
  - `examples/hello-api` create-user route is now aligned to this bridge contract (`createUser()`), with schema-gate and response schema names resolved in-handler.
  - Route handler compatibility now also requires numeric return types (`Int`/`Int64`) for the current runtime bridge, with compile-time diagnostics for non-numeric handler returns.
  - `req.json(...)` schema gate now rejects invalid schema argument shapes (numeric/boolean/untrusted/secret) instead of only checking missing arguments, with security/schema tagged diagnostics.
  - `req.json(...)` now also enforces exact call arity (`req.json(schema)` only).
  - `res.text(...)` now enforces bridge signature shape:
    - exactly two arguments,
    - numeric status code,
    - string response body.
  - `res.html(...)` now enforces bridge signature shape:
    - exactly one argument,
    - argument must be `HtmlSafe`,
    - violations emit `E4001` with `security` + `sink` diagnostic tags.
  - req/res `c-bin` integration coverage now uses an explicit HTML gate flow (`req.query` -> `sanitize.html` -> `res.html`) to validate the hardened sink contract end-to-end.
  - Header/cookie sink signatures are now typed and enforced:
    - `res.setHeader(name, value)` requires `HeaderName` + `HeaderValue`,
    - `res.addCookie(cookie)` requires `Cookie`,
    - violations emit tagged `E4001` sink diagnostics.
  - Header/cookie `c-bin` integration coverage now uses typed constructor flow (`headers.name`/`headers.value`/`cookie.build`) before sink calls.
  - Header constructor helpers are now signature-checked:
    - `headers.name(value)` and `headers.value(value)` require exactly one `String` argument,
    - malformed arity/type calls emit `E4001` with security-tagged diagnostics.
  - Gate and header/cookie `c-bin` integration fixtures now exercise string-based header constructor inputs for parity with the tightened contracts.
  - Request source helpers are now signature-checked:
    - `req.query`, `req.pathParam`, and `req.header` require exactly one `String` key argument,
    - malformed arity/type calls emit `E4001` with `security` + `schema` tags.
  - req/res and cors-origin `c-bin` integration fixtures now use string-key request-source calls, matching the hardened trust-boundary contract.
  - `path.base(...)` is now signature-checked:
    - requires exactly one `String` argument,
    - malformed arity/type calls emit `E4001` with security-tagged diagnostics.
  - Gate intrinsic `c-bin` integration fixtures now use string base-path constructor inputs (`path.base("/tmp/base")`) to match the hardened constructor contract.
  - `req.body(...)` is now signature-checked:
    - requires exact call shape `req.body(ctx, request)`,
    - argument types must be `Ctx` and `Request`,
    - malformed arity/type calls emit `E4001` with `security` + `schema` tags.
  - req/res `c-bin` integration fixtures now thread explicit `ctx`/`req` symbols through `req.body(...)`, matching the hardened trust-boundary contract.
  - Trust-gate arity contracts are now tightened:
    - single-input trust gates (for example `validate.*`, `sanitize.*`, `url.*`, `cors.origin`) require exactly one argument,
    - `path.under(...)` requires exactly two arguments,
    - malformed arity calls emit tagged `E4001` diagnostics.
  - DB sink call-shape contracts are now hardened:
    - `db.exec` requires `(capability, query)` or `(ctx, capability, query)`,
    - `db.execTx` requires `(tx, query)` or `(ctx, tx, query)`,
    - `db.queryOne` requires `(capability, query, rowSchema)` or `(ctx, capability, query, rowSchema)`,
    - malformed shapes emit tagged `E4001` sink diagnostics.
  - DB sink context-first typing is now hardened:
    - for `db.exec`/`db.execTx`/`db.queryOne` context-first forms, argument 1 must be `Ctx`,
    - type violations emit tagged `E4001` sink diagnostics.
  - DB sink query typing is now hardened:
    - DB sink query arguments must be `SqlQuery` in both compact and context-first forms,
    - non-`SqlQuery` inputs emit tagged `E4001` sink diagnostics.
  - `db.tx` call-shape contract is now hardened:
    - `db.tx` requires `(dbCap)` or `(ctx, dbCap)`,
    - malformed shapes emit tagged `E4001` capability diagnostics.
  - `db.tx` context-first typing is now hardened:
    - for `(ctx, dbCap)` calls, argument 1 must be `Ctx`,
    - type violations emit tagged `E4001` capability diagnostics.
  - Net sink call-shape contracts are now hardened:
    - `httpClient.get` requires `(netCap, url)` or `(ctx, netCap, url)`,
    - `httpClient.getInternal` requires `(internalNetCap, url)` or `(ctx, internalNetCap, url)`,
    - malformed shapes emit tagged `E4001` sink diagnostics.
  - Net sink context-first typing is now hardened:
    - for context-first net sink forms, argument 1 must be `Ctx`,
    - type violations emit tagged `E4001` sink diagnostics.
  - Net sink URL typing is now hardened:
    - `httpClient.get` requires `PublicUrl` payloads,
    - `httpClient.getInternal` requires `InternalUrl` payloads,
    - non-typed URL payloads emit tagged `E4001` sink diagnostics.
  - FS sink call-shape contracts are now hardened:
    - `fs.read` requires `(fsCap, path)` or `(ctx, fsCap, path)`,
    - `fs.write` requires `(fsCap, path, value)` or `(ctx, fsCap, path, value)`,
    - malformed shapes emit tagged `E4001` sink diagnostics.
  - FS sink context-first typing is now hardened:
    - for context-first FS sink forms, argument 1 must be `Ctx`,
    - type violations emit tagged `E4001` sink diagnostics.
  - FS sink path typing is now hardened:
    - FS sink path arguments must be `PathSafe` in compact and context-first forms,
    - non-`PathSafe` path inputs emit tagged `E4001` sink diagnostics.
  - Secret redact payload typing is now hardened:
    - `secrets.redact` argument must be `Secret<_>`,
    - non-secret redact payloads emit tagged `E4001` secret diagnostics.
  - `db.queryOne` row-schema typing is now hardened:
    - row schema arguments reject numeric/boolean placeholder values,
    - row schema arguments must be `Schema<_>` descriptors,
    - invalid row-schema values emit tagged `E4001` schema diagnostics.
  - `sql.q` signature typing is now hardened:
    - `sql.q` requires `(template, params)` shape with a `String` template argument,
    - params argument rejects `Secret<_>` and `Untrusted<_>` payloads,
    - invalid template values emit tagged `E4001` schema diagnostics.
  - `cookie.build` signature typing is now hardened:
    - `cookie.build` requires `(name, value)` shape with string name/value arguments,
    - invalid cookie-constructor values emit tagged `E4001` security diagnostics.
  - `err.withDetail` detail-value hardening is now active:
    - first argument must be `StdError`,
    - key argument must be `String`,
    - detail values reject `Secret<_>` and `Untrusted<_>` payloads with tagged diagnostics.
  - `err.withPath` path typing is now hardened:
    - first argument must be `StdError`,
    - path argument must be `String`,
    - invalid path payloads emit tagged `E4001` security diagnostics.
  - `err.withLimit` argument typing is now hardened:
    - call shape is enforced as `(error, name, max, actual)`,
    - first argument must be `StdError`,
    - `name` must be `String` and `max`/`actual` must be numeric.
  - `err.withDependency` argument typing is now hardened:
    - call shape is enforced as `(error, name, operation, retryable)`,
    - first argument must be `StdError`,
    - dependency/operation are string-typed and retryable is boolean.
  - `err.withCause` argument typing is now hardened:
    - call shape is enforced as `(error, cause)`,
    - both arguments must be `StdError`.
  - `err.internal` message typing is now hardened:
    - call shape is enforced as a single message argument,
    - message argument must be `String`.
  - `err.validation` constructor typing is now hardened:
    - call shape is enforced as `(code, message)`,
    - code and message arguments must be `String`.
  - `err.auth` constructor typing is now hardened:
    - call shape is enforced as `(code, message, status)`,
    - code/message are string-typed and status is numeric.
  - `err.notFound` constructor typing is now hardened:
    - call shape is enforced as `(code, message)`,
    - code and message arguments must be `String`.
  - `err.conflict` constructor typing is now hardened:
    - call shape is enforced as `(code, message)`,
    - code and message arguments must be `String`.
  - `err.rateLimit` constructor typing is now hardened:
    - call shape is enforced as `(code, message, retryAfterMs)`,
    - code/message are string-typed and retry-after is numeric.
  - CSP builder signature typing is now hardened:
    - `sec.csp` is enforced as zero-argument constructor,
    - `sec.cspAdd` enforces `(CspPolicy, String, String)` in the current runtime bridge.
  - `json.encode` schema-argument typing is now hardened:
    - call shape is enforced as `(schema, value)`,
    - schema argument must be `Schema<_>`,
    - schema argument rejects numeric/boolean/untrusted/secret payloads.
  - `json.decode` schema-argument typing is now hardened:
    - call shape is enforced as `(ctx, schema, raw)`,
    - schema argument must be `Schema<_>`,
    - schema argument rejects numeric/boolean/untrusted/secret payloads.
  - `json.decode` context/payload typing is now hardened:
    - argument 1 must be `Ctx`,
    - argument 3 must be `Untrusted<Bytes>`.
  - Secret source call-shape contract is now hardened:
    - `secrets.get` requires `(secretsCap, name)` or `(ctx, secretsCap, name)`,
    - malformed shapes emit tagged `E4001` secret diagnostics.
  - Secret source context-first typing is now hardened:
    - for `secrets.get(ctx, secretsCap, name)`, argument 1 must be `Ctx`,
    - type violations emit tagged `E4001` secret diagnostics.
  - Secret source name typing is now hardened:
    - `secrets.get` name argument must be `String`,
    - non-string name inputs emit tagged `E4001` secret diagnostics.
  - Secret redact call-shape contract is now hardened:
    - `secrets.redact` requires exactly one argument,
    - malformed shapes emit tagged `E4001` secret diagnostics.
  - Secret reveal call-shape contract is now hardened:
    - `secrets.reveal` requires `(secretsCap, secret)` or `(ctx, secretsCap, secret)`,
    - malformed shapes emit tagged `E4001` secret diagnostics.
  - Secret reveal context-first typing is now hardened:
    - for `secrets.reveal(ctx, secretsCap, secret)`, argument 1 must be `Ctx`,
    - type violations emit tagged `E4001` secret diagnostics.
  - Secret reveal value typing is now hardened:
    - `secrets.reveal` value argument must be `Secret<_>`,
    - non-secret value inputs emit tagged `E4001` secret diagnostics.
  - Auth helper call-shape contracts are now hardened:
    - `auth.require` requires exactly one `Ctx` argument,
    - `auth.requireRole` requires exactly two arguments: `(Ctx, String)`,
    - malformed shapes emit tagged `E4001` security diagnostics.
- Post-stability benchmark and cross-language comparison spec is now defined as a roadmap milestone input.
- Editor tooling and Zed integration architecture is now defined (compiler-backed LSP + extension + tree-sitter).

## 0. Product Direction (Locked Constraints)

These constraints come from current AILang docs and the new files:
- `docs/06-ailang-typescript-like-profile.md`
- `docs/07-ailang-no-inheritance-composition-model.md`
- `docs/book/43-ailang-v0-scope.md`
- `docs/book/52-security-first-priorities.md`
- `docs/book/53-v0-security-baseline.md`
- `docs/book/54-v0-stdlib-security-surface.md`
- `docs/book/55-v0-typing-effects-security-rules.md`
- `docs/book/56-security-diagnostics-taxonomy.md`
- `docs/book/57-standard-runtime-error-model.md`
- `docs/book/58-success-envelope-and-log-event-schema.md`
- `docs/book/59-request-capture-and-deterministic-replay.md`
- `docs/book/60-v0-policy-keys-spec.md`
- `docs/book/61-cors-typed-security-spec.md`
- `docs/book/63-security-middleware-baseline.md`
- `docs/book/64-sec-audit-spec.md`
- `docs/book/65-sensitive-api-markers-and-security-map.md`
- `docs/book/66-deterministic-severity-mapping-for-sec-audit.md`
- `docs/book/67-sec-audit-examples-and-policy-profiles.md`
- `docs/book/68-auth-policy-keys-and-csrf-coupling.md`
- `docs/book/69-auth-middleware-api-v0.md`
- `docs/book/71-benchmarking-and-comparison-spec.md`
- `docs/book/72-ailang-editor-tooling-and-zed-lsp-spec.md`
- `docs/book/73-m5-mir-bootstrap-and-introspection.md`
- `docs/book/74-m5-return-control-flow-lowering.md`
- `docs/book/75-m5-statement-control-flow-continuations.md`
- `docs/book/76-m5-nested-cfg-lowering.md`
- `docs/book/77-m5-canonical-block-id-normalization.md`
- `docs/book/78-m6-c-emitter-bootstrap.md`
- `docs/book/79-m6-clang-compile-link-pipeline.md`
- `docs/book/80-m6-run-command-via-c-bin.md`
- `docs/book/81-m6-runtime-abi-scaffold-and-link-integration.md`
- `docs/book/82-m6-runtime-return-identity-intrinsics.md`
- `docs/book/83-m6-runtime-intrinsic-call-rewriting.md`
- `docs/book/84-m6-non-trivial-c-bin-control-flow-validation.md`
- `docs/book/85-m6-runtime-c-assets-externalized.md`
- `docs/book/86-m6-time-now-end-to-end-c-bin-validation.md`
- `docs/book/87-m6-log-intrinsic-runtime-lowering.md`
- `docs/book/88-m6-req-res-intrinsic-runtime-lowering.md`
- `docs/book/89-m6-header-cookie-intrinsic-runtime-lowering.md`
- `docs/book/90-m6-db-fs-net-intrinsic-runtime-lowering.md`
- `docs/book/91-m6-secrets-intrinsic-runtime-lowering.md`
- `docs/book/92-m6-validator-sanitizer-url-path-intrinsic-runtime-lowering.md`
- `docs/book/93-m7-http-router-intrinsic-bootstrap.md`
- `docs/book/94-m7-hello-api-bootstrap-example.md`
- `docs/book/95-m7-hello-api-run-flow-validation.md`
- `docs/book/96-m7-security-middleware-intrinsic-bootstrap.md`
- `docs/book/97-m7-http-surface-type-bootstrap.md`
- `docs/book/98-m7-policy-config-intrinsic-bootstrap.md`
- `docs/book/99-m7-success-envelope-intrinsic-bootstrap.md`
- `docs/book/100-m7-request-source-intrinsic-bootstrap.md`
- `docs/book/101-m7-auth-requirement-intrinsic-bootstrap.md`
- `docs/book/102-m7-cors-origin-gate-intrinsic-bootstrap.md`
- `docs/book/103-m7-csrf-issue-token-intrinsic-bootstrap.md`
- `docs/book/104-m7-csp-builder-and-security-config-surface-bootstrap.md`
- `docs/book/105-m7-log-builder-intrinsic-bootstrap.md`
- `docs/book/106-m7-path-header-redact-helper-bridge.md`
- `docs/book/107-m7-json-helper-intrinsic-bridge.md`
- `docs/book/108-m7-error-builder-intrinsic-bridge.md`
- `docs/book/109-m7-res-text-intrinsic-bridge.md`
- `docs/book/110-m7-log-event-helper-intrinsic-bridge.md`
- `docs/book/111-m7-db-transaction-intrinsic-bridge.md`
- `docs/book/112-m7-sql-query-builder-intrinsic-bridge.md`
- `docs/book/113-m7-cookie-builder-intrinsic-bridge.md`

### 0.1 v0.1-lite philosophy
- Prioritize TypeScript-like ergonomics over systems-language complexity.
- Keep strong safety defaults (Option/Result, typed sinks, explicit effects).
- Avoid Rust-like complexity in v0.1: no lifetimes, no macros, no heavy generics, no trait/interface webs.

### 0.2 Core language shape for v0.1
- Minimal syntax set: `type`, `struct`, `enum`, `fn/function`, `let`, `const`, `if`, `match`, `return`.
- Strong error model: `Result<T, E>` (with optional sugar).
- Explicit absence model: `Option<T>` (with optional `T?` sugar).
- Structural composition + capability objects (no nominal inheritance/traits).
- Schema-driven decode/encode at boundaries.
- Effects remain first-class and auditable.

### 0.3 Security baseline for v0.1
- `Untrusted<T>` and `Secret<T>` are opaque wrappers with no implicit unwrapping.
- Typed sinks are mandatory (`SqlQuery`, `HtmlSafe`, `PublicUrl`/`InternalUrl`, `PathSafe`, `HeaderValue`, `LogValue`).
- Security policy is compiler-enforced (hard errors), not lint-only guidance.
- Capabilities + effects are both required for sensitive operations.
- Security middleware (`cors`, `sec`, `csrf`, `auth`) is typed, policy-driven, and non-optional in router bootstrap for relevant services.

### 0.4 Non-negotiable compile-time guarantees (v0 scope alignment)
- Reject `Untrusted<T>` flowing into trusted sinks without explicit gates.
- Enforce typed sink-only APIs (`SqlQuery`, `HtmlSafe`, `PublicUrl`, `InternalUrl`, `PathSafe`, `HeaderValue`, `LogValue`).
- Reject `Secret<T>` leakage in logs/JSON/string formatting unless explicitly redacted/revealed under policy.
- Enforce effects declarations (`used_effects` subset of `declared_effects`).
- Enforce capability availability (`DbCap`, `NetCap`, `FsCap`, `SecretsCap`, `InternalNetCap`) for sensitive APIs.
- Enforce schema-gated trust boundaries for handler input decoding.
- Enforce typed CORS configuration and policy constraints (credentials/wildcard/reflection rules).
- Enforce deterministic auth/CSRF coupling via policy (`auth.mode`, cross-site cookie constraints).

### 0.5 Explicit out-of-scope for v0
- No traits/interfaces, macros/derives, advanced generics, operator overloading, or manual memory model.

### 0.6 Plan Alignment Check
Aligned:
- TypeScript-like surface with small, explicit compiler stages.
- C backend-first strategy (`emit C` + `clang`) with backend-neutral MIR.
- No trait/interface inheritance architecture for v0.
- Security-by-construction as a compile-time contract, not conventions.

Adjusted in this roadmap:
- M3 explicitly enforces declared-effect checking and capability effect propagation.
- New M4 (Security Foundation Hardening) is inserted before MIR.
- URL security model is split (`PublicUrl` vs `InternalUrl`) with runtime SSRF checks.
- Structured logging, typed headers/cookies, and request budgets are now mandatory security primitives.
- Policy-as-code and diagnostics taxonomy are explicitly scheduled as compiler features.
- Success envelope/log-event schema, capture/replay, policy-key schema, and typed CORS behavior are now explicit roadmap inputs.
- Deterministic security posture reporting (`sec.audit`) and metadata tags (`security_map`) are explicit implementation targets.

## 1. Definition of Done (Minimal Real Working AILang)

A minimal real working AILang (v0.1-alpha) means:
- Compiler CLI exists and builds a runnable binary from `.ai` source.
- Lexer + parser + name resolution + minimal type checking are operational.
- Core semantic checks include unknown-name detection, type mismatch detection, and non-exhaustive match detection.
- MIR lowering exists and can be emitted for inspection.
- One backend exists (C emission first), producing working executables.
- Runtime ABI supports at least: strings/bytes, JSON parse/encode, HTTP serve, simple logging.
- End-to-end app works: `GET /health` and one JSON `POST` endpoint.
- Security gates/sinks are enforced for at least one path (schema gate + typed sink check + capability + effect enforcement).
- Policy file enforcement is active and reflected in build metadata.
- Security diagnostics include origin trace + sink + fix path for core flow violations.
- Canonical router bootstrap supports typed `security_headers`, `cors`, `csrf`, and `auth` middleware from policy.
- `sec.audit` can emit text/json posture reports and CI-gate on severity thresholds.
- Tests are automated for parser, type checks, MIR, and E2E sample.
- Documentation is updated for every implemented subsystem and can be read as a coherent book.

## 2. Build Strategy (Execution Order)

Implementation order is intentionally linear to reduce thrash:
1. Freeze v0.1-lite language profile.
2. Bootstrap compiler workspace + CLI.
3. Implement frontend pipeline (lexer/parser/AST).
4. Add semantic layer (names/types/effects minimal subset).
5. Security-hardening foundation: capabilities, policies, typed boundaries, diagnostics taxonomy, runtime safety contracts.
6. Lower to MIR + add MIR text output.
7. Implement C backend + runtime ABI stubs.
8. Add HTTP/JSON vertical slice with strict schema/budget defaults.
9. Add advanced security-by-construction constraints and policy checks.
10. Add security posture tooling (`security_map`, `sec.audit`, deterministic severity mapping, CI gating).
11. Harden tests, diagnostics, packaging, and build integrity metadata.
12. Run post-stability benchmark suite and publish cross-language comparison results.
13. Implement official editor tooling stack (compiler service + LSP + Zed extension + tree-sitter grammar).

## 3. Milestone Plan

## M0 - Repo Foundation and Toolchain
### Build tasks
- Define canonical repository layout (`compiler/`, `runtime/`, `stdlib/`, `examples/`, `docs/`).
- Implement CLI skeleton: `ailang build|run|check|test|fmt|lint`.
- Add diagnostics framework with spans and structured error codes.
- Add golden test harness infrastructure.

### Exit criteria
- `ailang build` runs on a hello-world fixture.
- Error output includes file/line/span and stable error code.

### Docs/book outputs
- Chapter: "Project Setup and Architecture".
- Chapter: "CLI Commands and Build Lifecycle".
- Glossary page for core terms (AST, HIR, MIR, sink, gate, effect).

## M1 - Lexer + Parser + AST
### Build tasks
- Implement tokenizer for keywords/operators/literals.
- Implement parser for core declarations/statements/expressions.
- Parse `Option`, `Result`, `match`, and function declarations.
- Preserve source spans throughout AST.

### Exit criteria
- Parser passes golden tests for language fixtures.
- `ailang check --emit ast` works.

### Docs/book outputs
- Chapter: "Lexical Grammar and Tokens".
- Chapter: "Parser Design and AST".
- Chapter: "Language Syntax by Example".

## M2 - Name Resolution + Type Checking (v0.1-lite)
### Build tasks
- Implement module/import resolution (next M2 slice; current M2 bootstrap is single-file).
- Build symbol tables for types/functions/fields.
- Implement minimal type checker with Option/Result flow and match exhaustiveness.
- Add structural shape compatibility checks (no nominal trait system).

### Exit criteria
- Representative fixtures type-check.
- Clear diagnostics for unknown names, type mismatch, non-exhaustive match.

### Docs/book outputs
- Chapter: "Name Resolution and Symbols".
- Chapter: "Type System (v0.1-lite)".
- Chapter: "Structural Composition, Not Inheritance".

## M3 - Effects + Boundary Semantics
### Build tasks
- Parse and validate function `effects { ... }` declarations.
- Enforce effect usage against declarations.
- Enforce `used_effects` subset of `declared_effects` for function bodies.

### Exit criteria
- Compiler rejects undeclared effect use.
- Effect diagnostics are stable and covered by golden tests.

### Docs/book outputs
- Chapter: "Effect System and Auditable Side Effects".
- Chapter: "Trust Boundaries and Untrusted Data".

## M4 - Security Foundation Hardening (Pre-MIR)
### Build tasks
- Add capability model for sensitive operations (`DbCap`, `NetCap`, `FsCap`, `SecretsCap`, `InternalNetCap`, `TxCap`).
- Require capability + effect for sensitive API usage.
- Add policy-as-code compiler integration (`ailang.policy` / equivalent) with hard-error enforcement.
- Introduce typed security primitives and sinks:
  - `PublicUrl` / `InternalUrl`
  - `HeaderName` / `HeaderValue`
  - `LogValue`
  - `Budget`
- Define and enforce schema-gated input trust boundaries (`req.json(schema)` canonical path).
- Lock diagnostics taxonomy and standard error model contracts.
- Define standard success envelope and structured log event contracts.
- Keep machine-readable CLI output modes JSON-only on stdout (`check --emit diagnostics-json`, `sec audit --format json`) for deterministic tooling/editor integration.
- Define capture/replay format contracts and replay-policy integration points.
- Define typed CORS config rules and credentials-origin safety checks.
- Add policy schema support for `security_headers`, `csrf`, and `auth` keys with deterministic validation rules.
- Define security marker/tag registry and `security_map` metadata contract for audit tooling.

### Exit criteria
- Compiler can reject missing capability and forbidden effect/policy combinations.
- Compiler can reject untrusted/secret misuse in at least one representative path per sink family.
- Baseline security acceptance tests are implemented and passing for this milestone scope.
- Policy schema keys for effects/net/logging/cors/capture can be parsed and validated.
- Policy schema keys for `security_headers`/`csrf`/`auth` can be parsed and validated.
- Baseline metadata tag coverage exists for core sensitive APIs and middleware symbols.

### Docs/book outputs
- Chapter: "Security-First Priorities".
- Chapter: "v0 Security Baseline".
- Chapter: "v0 Security Stdlib Surface".
- Chapter: "Typing and Effects Security Rules".
- Chapter: "Security Diagnostics Taxonomy".
- Chapter: "Standard Runtime Error Model".
- Chapter: "Standard Success Envelope and Log Event Schema".
- Chapter: "Request Capture and Deterministic Replay".
- Chapter: "v0 Policy Keys Spec".
- Chapter: "CORS Typed Security Spec".
- Chapter: "M4 Security Foundation Implementation (Current Slice)".
- Chapter: "Security Middleware Baseline".
- Chapter: "sec.audit Spec".
- Chapter: "Sensitive API Markers and security_map".
- Chapter: "Deterministic Severity Mapping for sec.audit".
- Chapter: "Auth Policy Keys and CSRF Coupling".
- Chapter: "Auth Middleware API".
- Chapter: "M4 security_map and sec.audit Implementation".

## M5 - MIR Lowering + Introspection
### Build tasks
- Design and implement backend-neutral MIR structures.
- Lower typed AST/HIR into MIR.
- Emit MIR text for debugging and tests.

### Exit criteria
- `ailang build --emit mir` works on sample programs.
- MIR golden tests pass.

### Docs/book outputs
- Chapter: "MIR Design".
- Chapter: "Lowering Rules".
- Chapter: "How to Read MIR".
- Chapter: "M5 Bootstrap: MIR Lowering and Introspection".
- Chapter: "M5 Slice: Return-Site Control-Flow Lowering".
- Chapter: "M5 Slice: Statement Control-Flow Continuations".
- Chapter: "M5 Slice: Nested CFG Lowering".
- Chapter: "M5 Slice: Canonical Block ID Normalization".

## M6 - C Backend + Runtime ABI (First Runnable Target)
### Build tasks
- Implement ABI-lowering pass.
- Emit C code from MIR (single binary target first).
- Implement runtime header/source with required intrinsics.
- Add compile/link pipeline via `clang`.

### Exit criteria
- Compiled executable runs non-trivial program.
- Generated C backend supports control flow + function calls + Result handling.

### Docs/book outputs
- Chapter: "Backend Architecture and ABI".
- Chapter: "C Emission Strategy".
- Chapter: "Runtime Intrinsics".
- Chapter: "M6 Bootstrap: C Emitter from MIR".
- Chapter: "M6 Slice: Clang Compile/Link Pipeline".
- Chapter: "M6 Slice: run Command via C Backend".
- Chapter: "M6 Slice: Runtime ABI Scaffold and Link Integration".
- Chapter: "M6 Slice: Runtime Return Identity Intrinsics".
- Chapter: "M6 Slice: Runtime Intrinsic Call Rewriting".
- Chapter: "M6 Slice: Non-Trivial c-bin Control-Flow Validation".
- Chapter: "M6 Slice: Runtime C Assets Externalized".
- Chapter: "M6 Slice: time.now End-to-End c-bin Validation".
- Chapter: "M6 Slice: Log Intrinsic Runtime Lowering".
- Chapter: "M6 Slice: req/res Intrinsic Runtime Lowering".
- Chapter: "M6 Slice: Header/Cookie Intrinsic Runtime Lowering".
- Chapter: "M6 Slice: DB/FS/Net Intrinsic Runtime Lowering".
- Chapter: "M6 Slice: Secrets Intrinsic Runtime Lowering".
- Chapter: "M6 Slice: Validator/Sanitizer/URL/Path Intrinsic Runtime Lowering".

## M7 - HTTP/JSON Vertical Slice
### Build tasks
- Implement minimal HTTP router/runtime bridge.
- Implement JSON parse/encode primitives and schema decode path.
- Enforce schema-gated request decoding as the default trusted-input path.
- Enforce budgeted decode defaults (`maxBodyBytes`, `maxJsonBytes`, `maxJsonDepth`, deadlines).
- Implement optional standard success envelope support controlled by policy.
- Implement canonical router security bootstrap (`security_headers`, `cors`, `csrf`, `auth`) from policy.
- Build sample service with `/health` + one typed POST endpoint.

### Exit criteria
- `ailang run examples/hello-api` serves both endpoints.
- E2E tests verify status, decode behavior, and errors.
- Middleware behavior tests cover CORS preflight, header emission, and CSRF gate behavior.

### Docs/book outputs
- Chapter: "HTTP Runtime and Request Lifecycle".
- Chapter: "Schema-Driven JSON".
- Chapter: "Building Your First AILang API".
- Chapter: "M7 Bootstrap: HTTP Router Intrinsic Runtime Bridge".
- Chapter: "M7 Slice: hello-api Bootstrap Example".
- Chapter: "M7 Slice: hello-api run Flow Validation".
- Chapter: "M7 Slice: Security Middleware Intrinsic Bootstrap".
- Chapter: "M7 Slice: HTTP Surface Type Bootstrap".
- Chapter: "M7 Slice: Policy-Config Intrinsic Bootstrap".
- Chapter: "M7 Slice: Success Envelope Intrinsic Bootstrap".
- Chapter: "M7 Slice: Request Source Intrinsic Bootstrap".
- Chapter: "M7 Slice: Auth Requirement Intrinsic Bootstrap".
- Chapter: "M7 Slice: CORS Origin Gate Intrinsic Bootstrap".
- Chapter: "M7 Slice: CSRF IssueToken Intrinsic Bootstrap".
- Chapter: "M7 Slice: CSP Builder and Security Config Surface Bootstrap".
- Chapter: "M7 Slice: CSP Builder Signature Hardening".
- Chapter: "M7 Slice: Log Builder Intrinsic Bootstrap".
- Chapter: "M7 Slice: json.encode Schema-Argument Hardening".
- Chapter: "M7 Slice: json.encode Schema-Descriptor Hardening".
- Chapter: "M7 Slice: json.decode Schema-Argument Hardening".
- Chapter: "M7 Slice: json.decode Schema-Descriptor Hardening".
- Chapter: "M7 Slice: json.decode Context/Payload Argument-Type Hardening".
- Chapter: "M7 Slice: Path/Header/Redact Helper Bridge".
- Chapter: "M7 Slice: JSON Helper Intrinsic Bridge".
- Chapter: "M7 Slice: Error Builder Intrinsic Bridge".
- Chapter: "M7 Slice: err.withDetail Value-Safety Hardening".
- Chapter: "M7 Slice: err.withDetail Error-Argument Hardening".
- Chapter: "M7 Slice: err.withPath Path-Argument Hardening".
- Chapter: "M7 Slice: err.withPath Error-Argument Hardening".
- Chapter: "M7 Slice: err.withLimit Argument Hardening".
- Chapter: "M7 Slice: err.withLimit Error-Argument Hardening".
- Chapter: "M7 Slice: err.withDependency Argument Hardening".
- Chapter: "M7 Slice: err.withDependency Error-Argument Hardening".
- Chapter: "M7 Slice: err.withCause Argument Hardening".
- Chapter: "M7 Slice: err.internal Message-Argument Hardening".
- Chapter: "M7 Slice: err.validation Constructor-Argument Hardening".
- Chapter: "M7 Slice: err.auth Constructor-Argument Hardening".
- Chapter: "M7 Slice: err.notFound Constructor-Argument Hardening".
- Chapter: "M7 Slice: err.conflict Constructor-Argument Hardening".
- Chapter: "M7 Slice: err.rateLimit Constructor-Argument Hardening".
- Chapter: "M7 Slice: res.text Intrinsic Bridge".
- Chapter: "M7 Slice: Log Event Helper Intrinsic Bridge".
- Chapter: "M7 Slice: DB Transaction Intrinsic Bridge".
- Chapter: "M7 Slice: SQL Query Builder Intrinsic Bridge".
- Chapter: "M7 Slice: Cookie Builder Intrinsic Bridge".
- Chapter: "M7 Slice: Function Symbol Handler Wiring for hello-api".
- Chapter: "M7 Slice: Route Registration Contract Checks".
- Chapter: "M7 Slice: Typed Router Security Bootstrap Contracts".
- Chapter: "M7 Slice: HTTP Call-Shape Contract Checks".
- Chapter: "M7 Slice: Route Handler Compatibility (Zero-Arg Bridge Contract)".
- Chapter: "M7 Slice: Route Handler Return-Type Bridge Contract".
- Chapter: "M7 Slice: req.json Schema-Argument Contract Hardening".
- Chapter: "M7 Slice: req/res Call-Shape Signature Hardening".
- Chapter: "M7 Slice: res.html Signature Hardening".
- Chapter: "M7 Slice: Header/Cookie Sink Signature Hardening".
- Chapter: "M7 Slice: Header Constructor Signature Hardening".
- Chapter: "M7 Slice: cookie.build Signature Hardening".
- Chapter: "M7 Slice: Request Source Signature Hardening".
- Chapter: "M7 Slice: path.base Signature Hardening".
- Chapter: "M7 Slice: req.body Signature Hardening".
- Chapter: "M7 Slice: Trust-Gate Arity Hardening".
- Chapter: "M7 Slice: DB Sink Call-Shape Hardening".
- Chapter: "M7 Slice: DB Sink Context-Argument Type Hardening".
- Chapter: "M7 Slice: DB Sink Query-Argument Type Hardening".
- Chapter: "M7 Slice: db.queryOne Row-Schema Argument Hardening".
- Chapter: "M7 Slice: db.queryOne Row-Schema Descriptor Hardening".
- Chapter: "M7 Slice: sql.q Template-Argument Hardening".
- Chapter: "M7 Slice: sql.q Params Value-Safety Hardening".
- Chapter: "M7 Slice: db.tx Call-Shape Hardening".
- Chapter: "M7 Slice: db.tx Context-Argument Type Hardening".
- Chapter: "M7 Slice: Net Sink Call-Shape Hardening".
- Chapter: "M7 Slice: Net Sink Context-Argument Type Hardening".
- Chapter: "M7 Slice: Net Sink URL-Argument Type Hardening".
- Chapter: "M7 Slice: FS Sink Call-Shape Hardening".
- Chapter: "M7 Slice: FS Sink Context-Argument Type Hardening".
- Chapter: "M7 Slice: FS Sink Path-Argument Type Hardening".
- Chapter: "M7 Slice: Secret Source Call-Shape Hardening".
- Chapter: "M7 Slice: Secret Source Context-Argument Type Hardening".
- Chapter: "M7 Slice: Secret Source Name-Argument Type Hardening".
- Chapter: "M7 Slice: Secret Redact Call-Shape Hardening".
- Chapter: "M7 Slice: Secret Redact Value-Argument Type Hardening".
- Chapter: "M7 Slice: Secret Reveal Call-Shape Hardening".
- Chapter: "M7 Slice: Secret Reveal Context-Argument Type Hardening".
- Chapter: "M7 Slice: Secret Reveal Value-Argument Type Hardening".
- Chapter: "M7 Slice: Auth Helper Call-Shape Hardening".

## M8 - Security-by-Construction Enforcement
### Build tasks
- Enforce typed SQL sink path (`sql"..."` to `SqlQuery`).
- Enforce typed sink-only APIs for SQL/HTML/URL/Path/Header/Log sinks.
- Enforce secret handling policy for logging/encoding/string formatting/interpolation.
- Enforce URL tiering rules (`PublicUrl` vs `InternalUrl`) and redirect/internal-network policy constraints.
- Enforce structured logging and response/header safety rules.
- Enforce typed CORS policy constraints and safe preflight/header behavior.
- Enforce capture/replay policy constraints and replay-effect blocking defaults.
- Add policy config and allowlist annotation flow for strictly controlled exceptions.
- Emit `security_map` metadata with sensitive API/middleware tags and allowlist bypass records.
- Implement `ailang sec audit` text/json report based on tags + policy + deterministic severity mapping.

### Exit criteria
- Compiler/linter rejects representative insecure patterns.
- Security tests included in CI.
- Policy hash and compiler version are embedded in build metadata.
- `sec.audit` can fail CI by configured threshold (`--fail-on risk>=...`).

### Docs/book outputs
- Chapter: "Security Model".
- Chapter: "Typed Sinks and Safe Boundaries".
- Chapter: "Policy and Lint Rules".

## M9 - Release Hardening
### Build tasks
- Improve diagnostics quality and error explainability.
- Add deterministic build controls and lock strategy.
- Finalize minimal stdlib and sample apps.
- Add reproducible build metadata and optional SBOM generation.
- Finalize secure policy profiles (`default-secure-prod`, `permissive-dev`) and release audit baselines.
- Prepare alpha release checklist.

### Exit criteria
- Repeatable build across clean environments.
- Test suite stable with release tag candidate.

### Docs/book outputs
- Chapter: "Release Notes and Compatibility".
- Chapter: "Known Limits of v0.1-alpha".
- Chapter: "Road to v0.2".

## M10 - Post-Stability Benchmarking and Comparative Validation
### Trigger condition
- Start M10 only after AILang is stable and working end-to-end (M9 exit criteria met).

### Build tasks
- Create benchmark suite with identical service behavior across implementations:
  - Endpoint A: `GET /ping` (hello HTTP overhead)
  - Endpoint B: `POST /decode` (JSON + schema validation path)
  - Endpoint C: `POST /users` and `GET /users/:id` (real DB write/read workloads)
  - Optional Endpoint D (later): fanout path with outbound call (`/enrich/:id`)
- Implement comparison services for:
  - AILang (C + clang backend)
  - Go
  - Node.js TypeScript
  - Rust
  - Optional reference floor: minimal C server
- Standardize fairness controls:
  - same machine and runtime envelope
  - same DB schema/indexes/query text/pool size/timeouts
  - same payload shapes and validation rules
  - same load profile (constant-rate + step-load)
- Run benchmark matrix and collect:
  - throughput, p50/p95/p99, error rate
  - CPU and RSS memory
  - binary size and startup time (optional)
- Add AILang-specific validation tracks:
  - security-defaults-on cost (schema gates, typed sinks, URL safety)
  - deterministic replay demo (`capture -> replay -> same error code`)
  - `sec.audit` posture output with policy/build stamping
- Publish reproducible harness:
  - benchmark scripts, raw outputs, summaries, and plots
  - one-command runner for each endpoint profile

### Exit criteria
- Benchmark suite runs end-to-end with reproducible scripts and documented environment.
- Cross-language comparison report is generated from raw captured results.
- Tail-latency behavior (`p99`) and failure-mode behavior are explicitly analyzed.
- Security-cost and debugging-workflow benchmarks are included in final report.

### Docs/book outputs
- Chapter: "Benchmarking and Comparison Spec".
- Chapter: "Benchmark Harness and Reproducibility Guide".
- Chapter: "First Public Performance and Security Report".

## M11 - Editor Tooling and Zed Integration
### Trigger condition
- Start M11 after core language/runtime behavior is stable enough for deterministic editor semantics (at minimum: M8 complete, ideally after M9 stabilization).

### Build tasks
- Expose compiler frontend as a tooling service API (AST/HIR/symbols/diagnostics/completions metadata).
- Implement `ailang-language-server` using compiler APIs (no duplicated parser/typechecker).
- Implement required LSP features:
  - diagnostics, definition, references, hover, completion, rename, code actions
- Implement incremental analysis and bounded execution:
  - per-file caching, dependency invalidation, request time/memory budgets
- Implement security-aware editor UX:
  - tagged diagnostics for `security`/`taint`/`secret`
  - source-to-sink notes and safe quick-fix families
- Build `zed-ailang` extension:
  - language config for `.ai`
  - LSP wiring to `ailang-language-server --stdio`
  - grammar registration via `tree-sitter-ailang`
- Add tree-sitter grammar and baseline queries (`highlights`, optional `outline`/`indent`).

### Exit criteria
- Zed can open `.ai` files with working diagnostics, go-to-definition, hover, and completion.
- Rename and references behave deterministically on multi-file test fixtures.
- Security diagnostics in editor include stable codes, spans, tags, and actionable notes.
- Tooling budgets are enforced and tested (no hangs on large/invalid inputs).
- LSP server runs with safe defaults (no implicit code execution/network).

### Docs/book outputs
- Chapter: "AILang Editor Tooling and Zed LSP Spec".
- Chapter: "LSP Protocol Mapping and Compiler Service API".
- Chapter: "Zed Extension and Tree-sitter Integration Guide".

## 4. Documentation-as-Book Plan (Mandatory Workflow)

`docs/` should evolve into book structure, not ad-hoc notes.

## 4.1 Book structure target
- `docs/book/00-preface.md`
- `docs/book/01-language-overview.md`
- `docs/book/02-type-system.md`
- `docs/book/03-effects-and-security.md`
- `docs/book/04-compiler-front-end.md`
- `docs/book/05-mir-and-backend.md`
- `docs/book/06-runtime-and-stdlib.md`
- `docs/book/07-building-services.md`
- `docs/book/08-testing-and-debugging.md`
- `docs/book/09-release-and-roadmap.md`

## 4.2 Rule: every implementation PR/task must update docs
For every implemented part, include both:
- Technical implementation section (what changed in code).
- Reader-facing explanation section (what it is + how it works + why chosen).

## 4.3 Explanation template (required for each implemented part)
For each subsystem/change, write:
1. What it is.
2. Why it exists.
3. How it works internally.
4. Inputs/outputs and constraints.
5. Failure modes and diagnostics.
6. Example usage.
7. Trade-offs and future improvements.

## 4.4 Documentation checkpoints per milestone
Each milestone completion requires:
- Updated chapter(s).
- One end-to-end worked example.
- One troubleshooting subsection.
- One "AI implementation notes" subsection to improve future agent consistency.

## 4.5 v0 acceptance tests (must pass by M8)
1. Reject raw SQL concatenation.
2. Reject unescaped raw HTML output without `HtmlSafe`.
3. Reject logging/encoding of `Secret<_>`.
4. Reject missing declared effects.
5. Require schema gate for decoding request body into trusted types.
6. Reject internal URL access without internal capability/policy.
7. Reject unsafe header setting without `HeaderValue` validation.
8. Enforce budgeted request decode behavior and limit diagnostics.
9. Reject unsafe CORS configurations (credentials + wildcard/reflection when forbidden by policy).
10. Ensure replay deny mode blocks external effects without capabilities.
11. Enforce CORS preflight and `Vary: Origin` correctness for allowlist mode.
12. Enforce security headers presence on success and error paths.
13. Enforce CSRF token validation for protected methods in cookie-auth mode.
14. Enforce deterministic auth-mode to CSRF requirements via policy.
15. Produce deterministic `sec.audit` findings for equivalent code/policy inputs.

## 5. Day-to-Day Development Loop

For each task:
1. Select task from active milestone.
2. Implement smallest working vertical slice.
3. Add/adjust tests.
4. Add explanation using template above.
5. Update relevant book chapter.
6. Run check/test/lint.
7. Record decision in changelog/ADR section.

No task is considered complete without code + tests + docs + explanation.

## 6. Initial 14-Day Execution Sprint (Suggested)

Days 1-2:
- M0 bootstrap + chapter skeletons.

Days 3-5:
- M1 lexer/parser + syntax chapter.

Days 6-8:
- M2 names/types minimal + type chapter.

Days 9-10:
- M3 effects implementation + diagnostics stabilization.

Days 11-13:
- M4 security foundation hardening (capabilities, policy, boundary/sink typing).

Day 14:
- Consolidation, acceptance test delta review, and next sprint planning for MIR.

## 7. Immediate Next Actions (Start Here)

1. Extend callable/member canonicalization beyond capability namespace seeding into user-defined capability-object function fields and other non-stdlib callable-value shapes.
2. Extend provenance traces from compiler/audit outputs into editor-facing explainability surfaces (LSP/hover/code-action context).
3. Expand trend mode from file-based baseline comparison to persisted multi-run history and time-window trend summaries.
4. Prepare M10 benchmark harness scaffold once M9 stability gate is reached.
5. Prepare M11 editor tooling scaffold once semantic outputs are stabilized for LSP use.

---

This roadmap is the canonical execution path until v0.1-alpha is running and documented as a coherent book.
