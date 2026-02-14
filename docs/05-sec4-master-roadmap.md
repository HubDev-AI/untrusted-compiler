# Untrusted<T> Master Roadmap (From Line 1 to Minimal Real Working Compiler)

This roadmap is the execution plan for building a minimal, real, runnable Untrusted<T> implementation.

It explicitly includes a parallel documentation workflow so `docs/` evolves into a book-quality project history and technical guide.

## Ecosystem Naming Lock (2026-02-13)

The naming and packaging surface is now locked and must be treated as a compatibility contract:

- Language (docs/marketing): `Untrusted<T>`
- Tooling brand: `sec4Audit`
- Compiler/CLI brand and command: `sec4`
- Policy/security tooling commands:
  - `sec4 audit`
  - `sec4 explain`
  - `sec4 gate`
  - `sec4 replay`
- GitHub repository: `https://github.com/HubDev-AI/untrusted-compiler`
- Source file extension: `.ut`
- Package/module namespace contract:
  - `ut/std`
  - `ut/http`
  - `ut/sec`

Roadmap impact:

- Add an explicit naming-alignment slice before further feature expansion.
- Every milestone must preserve these names in:
  - CLI UX/help/docs examples
  - policy filenames and policy docs
  - benchmark harness implementation IDs and report artifacts
  - editor tooling (LSP + Zed + tree-sitter)
  - runtime ABI/docs

## Current Status (2026-02-14)

- Milestone progression reached M14 bootstrap slices with active enforcement.
- M9 release hardening gate is operational both locally and in CI:
  - `scripts/release-alpha-gate.sh`
  - `.github/workflows/alpha-release-gate.yml`
- M11 editor tooling scope has no remaining tasks in this roadmap revision.
- M12 naming alignment scope has no remaining tasks in this roadmap revision.
- M13 operational confidence closure gates are green.
- M14 replay bootstrap is active with closure gating (`M14-A`, `M14-B`, `M14-C`, `M14-D`).
- M15 runtime replay-stubbing expansion scope is now defined and ready to execute.
- M16-S1 live HTTP runtime bootstrap is now implemented:
  - runtime router + route registration + socket serve loop are active in `runtime/c/sec4_runtime.c`,
  - `res.text` now materializes real HTTP response payloads for active request handlers,
  - `SEC4_RT_HTTP_SERVE_MODE=oneshot` is available for deterministic non-blocking test execution.
- M16-S2 JSON response materialization bootstrap is now implemented:
  - `res.ok`/`res.json` runtime paths materialize JSON HTTP responses,
  - `POST /users` runtime E2E serving is validated in oneshot mode.
- M16-S3 status-propagation hardening is now implemented:
  - `res.ok(status, ...)` and `res.okMeta(status, ...)` now propagate caller status into runtime HTTP response line.
- M16-S4 request-body JSON gate bootstrap is now implemented:
  - `req.json(...)` now validates live request body shape at runtime (bridge-level JSON gate),
  - invalid JSON requests deterministically return `400 Bad Request` with JSON error payload.
- M16-S5 content-type gate hardening is now implemented:
  - `req.json(...)` now enforces JSON media type (`application/json` or `application/*+json`),
  - non-JSON request media types deterministically return `415 Unsupported Media Type`.
- M16-S6 `sec4 run` live-serving e2e coverage is now implemented:
  - CLI `sec4 run` path is now explicitly validated with a real HTTP POST roundtrip in oneshot mode.
- M16-S7 request-size guard hardening is now implemented:
  - `req.json(...)` now rejects oversized request bodies with deterministic `413 Payload Too Large`.
- M16-S8 std-error envelope alignment is now implemented:
  - runtime `req.json(...)` gate failures now return deterministic standard JSON error envelopes with stable error codes.

## Formal Closure Audit (Strict, 2026-02-13)

Canonical closure should be evaluated with:

```bash
scripts/check-milestone-closure.sh
```

Current strict closure result:

| Gate | Status | Meaning | Evidence |
| --- | --- | --- | --- |
| `M9-A` | PASS | Release gate script exists | `scripts/release-alpha-gate.sh` |
| `M9-B` | PASS | Release gate workflow exists | `.github/workflows/alpha-release-gate.yml` |
| `M9-C` | PASS | Promotion verifier/manifest chain exists | `scripts/verify-release-promotion-inputs.sh`, publish-manifest scripts |
| `M9-D` | PASS | Release gate enforces strict milestone closure | `scripts/release-alpha-gate.sh` |
| `M9-E` | PASS | Release-contract-smoke workflow keeps release verifier/publish + guard checks | `.github/workflows/release-contract-smoke.yml` |
| `M9-F` | PASS | Naming-lock CI enforces release-contract-smoke contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M9-G` | PASS | Alpha-release workflow keeps release/promotion/publish/upload contract | `.github/workflows/alpha-release-gate.yml` |
| `M9-H` | PASS | Naming-lock CI enforces alpha-release workflow contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M10-A` | PASS | Live cross-impl matrix evidence includes `sec4/go/node/rust` for each endpoint | `benchmark-suite/results/summaries/compare-matrix.json` |
| `M10-B` | PASS | Live cross-impl matrix row contract is aligned (`endpoint`, `leader`, `compared`) | `benchmark-suite/results/summaries/compare-matrix.json` |
| `M10-C` | PASS | Cross-impl evidence workflow keeps scoped run, strict quality gate, and artifact upload contract | `.github/workflows/benchmark-cross-impl-evidence.yml` |
| `M10-D` | PASS | Naming-lock CI enforces cross-impl workflow contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M11-A` | PASS | Naming-lock CI enforces Zed grammar pin contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M12-A` | PASS | Naming-lock CI enforces `sec4` CLI command contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M13-A` | PASS | Trend note includes live `Trend Entry` block | `docs/book/322-m13-first-trend-run-results-note.md` |
| `M13-B` | PASS | Trend workflow keeps strict quality + regression guard steps | `.github/workflows/benchmark-trend.yml` |
| `M13-C` | PASS | Trend workflow uploads benchmark artifacts for trend-note ingestion | `.github/workflows/benchmark-trend.yml` |
| `M13-D` | PASS | Benchmark-smoke workflow enforces closure + cross-impl/trend contract guards and strict closure audit | `.github/workflows/benchmark-smoke.yml` |
| `M13-E` | PASS | Naming-lock CI enforces benchmark-trend workflow contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M13-F` | PASS | Naming-lock CI enforces sec4 explain audit-coverage contract test | `.github/workflows/naming-lock.yml` |
| `M14-A` | PASS | Naming-lock CI enforces replay capture contract test | `.github/workflows/naming-lock.yml` |
| `M14-B` | PASS | Naming-lock CI enforces replay capture compatibility test | `.github/workflows/naming-lock.yml` |
| `M14-C` | PASS | Naming-lock CI enforces replay stub registry contract test | `.github/workflows/naming-lock.yml` |
| `M14-D` | PASS | Naming-lock CI enforces replay CLI json contract + guard tests | `.github/workflows/naming-lock.yml` |

Strict closure interpretation:
- M11 and M12 are complete for current scope.
- M9 implementation is functionally complete but release-candidate evidence remains a verification activity.
- M10 and M13 closure evidence requirements are now satisfied.
- M14 bootstrap replay-contract enforcement is active (`M14-A`).
- M14 replay compatibility enforcement is active (`M14-B`).
- M14 replay stub-registry bootstrap enforcement is active (`M14-C`).
- M14 replay CLI json contract enforcement is active (`M14-D`).

Historical implementation bullets below are retained as build history; strict gate status above is the closure source of truth.

- M0 bootstrap completed and committed.
- M1 frontend bootstrap completed:
  - Lexer/token model with span-aware diagnostics.
  - Parser/AST for `fn`, `struct`, `enum`, blocks, expressions, `match`, and generic types (`Option`/`Result` forms).
  - `sec4 check --emit ast` output wired into CLI.
  - Parser golden fixtures added in `compiler/sec4-core/tests/fixtures/parser`.
- M2 semantic bootstrap completed:
  - Name resolution for declared types/functions and local identifiers.
  - Minimal type checker for bindings, returns, calls, operators, and branch compatibility.
  - Match exhaustiveness checks for `Bool`, user enums, and `Option`/`Result` forms.
  - Semantic golden fixtures added in `compiler/sec4-core/tests/fixtures/semantic`.
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
  - `sec4 audit` CLI command is implemented with deterministic text/json findings and threshold gating.
  - `sec4 audit` includes deterministic allowlist hygiene findings (high-risk bypasses, expiry/soon-expiry, exception-count posture signal) plus expiry-window aging metrics and explicit severity-input thresholds.
  - `sec4 audit` now supports optional baseline comparison (`--baseline <audit.json>`) and emits deterministic trend deltas:
    - baseline policy hash/risk score,
    - risk and finding-count deltas,
    - per-severity deltas,
    - added/resolved finding ID sets.
  - `sec4 audit` now supports explicit report persistence (`--write-report <path>`) to support baseline capture and opt-in trend history workflows.
  - `sec4 audit` now supports opt-in history capture (`--history-dir <path>`):
    - writes timestamped JSON reports per run,
    - auto-loads the latest history report as baseline when `--baseline` is not provided,
    - preserves JSON-only stdout contract in JSON mode while emitting history/baseline artifact hints to stderr.
  - Diagnostics now carry structured tag metadata (`security`, `taint`, `secret`, `policy`, `effects`, `capability`, `schema`, `sink`) for editor/LSP-oriented consumers while preserving current text rendering.
  - CLI now supports machine-readable diagnostics output via `sec4 check --emit diagnostics-json`, exposing spans/codes/notes/tags as JSON for tooling integration.
  - Machine-readable CLI output contracts are now strict:
    - `sec4 check --emit diagnostics-json` prints JSON-only payloads on stdout in both success and failure paths.
    - `sec4 audit --format json` prints JSON-only report payload on stdout; human hint lines (for example `security map: ...`) are emitted to stderr.
  - CLI integration tests now verify parseable JSON stdout contracts for `check --emit diagnostics-json` and `audit --format json`.
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
    - symbol registry coverage and tests were extended to keep `sec4 audit` marker detection deterministic.
  - `[logging]` and `[sql]` policy sections are now ingested into the typed policy model.
  - `[net.public]` domain policy lists are now ingested into the typed policy model:
    - `allowed_domains`
    - `blocked_domains`
  - `[fs]` symlink posture is now ingested/validated in the typed policy model:
    - `forbid_symlinks` (`off|warn|enforce`)
  - `sec4 audit` now emits deterministic logging/SQL posture findings:
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
    - `sec4 audit` now emits `CORS_REFLECT_ORIGIN_ENABLED` with deterministic `sampleCalls` evidence when origin reflection is enabled.
  - policy parser now validates `sql.require_limit_on_select` values (`off|warn|enforce`) with dedicated diagnostics.
  - `security_map` now adds callsite SQL hygiene tags (`sql.select_without_limit`) for SQL sink calls with unbounded `SELECT` literals.
  - `sec4 audit` now emits `SQL_SELECT_WITHOUT_LIMIT` with policy-mapped severity:
    - `MEDIUM` for `sql.require_limit_on_select = "warn"`.
    - `HIGH` for `sql.require_limit_on_select = "enforce"`.
  - Tests now cover both SQL hygiene tag extraction and `sec4 audit` severity mapping for warn/enforce modes.
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
  - JSON response schema descriptors are now narrowed to bridge-name `String` values or typed `Schema<_>` descriptors.
  - `res.okMeta` now rejects `Secret<_>` and `Untrusted<_>` metadata payloads with `E4004` diagnostics.
  - Log-event helper signatures are now hardened:
    - `log.attrRedacted(label)` requires string labels.
    - `log.withAttr(event, key, value)` requires `LogEvent`/`LogValue`, string keys, and `LogAttr` values.
    - `log.withHttp(event, method, path, status, latencyMs)` requires typed string/numeric payloads.
    - `log.withError(event, error)` requires typed `StdError` inputs.
  - Log sink signatures are now hardened:
    - `log.info`/`log.warn`/`log.error`/`log.emit` require exactly one `LogValue` payload argument.
    - secret/untrusted payloads remain reported via existing sink-flow diagnostics without duplicate signature errors.
  - Log value-builder label signatures are now hardened:
    - `log.event(name)` requires `String` event names.
    - `log.redacted(label)` requires `String` redaction labels.
  - Additional log value-builder signatures are now hardened:
    - `log.str(value)` requires `String`.
    - `log.i64(value)` requires numeric payloads.
    - `log.bool(value)` requires `Bool`.
    - `log.field(key, value)` requires string keys and `LogValue` payloads.
    - `log.obj(fields)` now enforces single-argument arity.
  - `log.obj` now also enforces typed payload input (`LogValue`) beyond arity-only checks.
  - `headers.value` now rejects CR/LF literal payloads at compile time to reduce response-splitting risk in constant header values.
  - `headers.name` now rejects invalid literal token characters (for example whitespace/colon) at compile time.
  - `cookie.build` now rejects CR/LF literal sequences in constant cookie values at compile time.
  - `cookie.build` now also rejects invalid literal cookie-name token characters at compile time.
  - Secret equality hardening is now active: direct `==`/`!=` on `Secret<_>` values is compile-time rejected with guidance toward constant-time comparison helpers.
  - Added `crypto.ctEq` intrinsic bridge:
    - typed semantic contract requires two compatible `Secret<_>` arguments,
    - returns `Bool`,
    - lowers to runtime symbol `sec4_rt_crypto_ct_eq`.
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
  - `sec4 audit` findings now include bounded `sampleCalls` evidence for key finding families.
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
  - `sec4 audit` text rendering now includes sample-call previews with provenance trace snippets when available.
  - `sec4 audit` now emits deterministic exception-expiry rollup findings (`ALLOW_EXPIRY_WINDOW_ROLLUP`) with sampled exception evidence.
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
  - deterministic `sec4 audit` contract,
  - compiler-emitted security metadata tags for robust audit tooling.
- M5 bootstrap has started:
  - backend-neutral MIR module is now implemented in `sec4-core` (`MirProgram`, `MirFunction`, `MirBlock`).
  - AST function bodies now lower into deterministic single-block MIR (`bb0`) with explicit `return` terminators.
  - tail `if` expressions now lower into explicit branch control-flow with multiple blocks (`bb0` -> `bb1` / `bb2`).
  - tail `match` expressions now lower into `switch` terminators with per-arm blocks.
  - explicit `return if` and `return match` expressions now lower into explicit branch/switch blocks (not inline expression returns).
  - statement-level `if`/`match` expression statements now lower into explicit continuation CFG blocks using `goto` join targets.
  - nested `if`/`match` control flow inside branch bodies/block tails now lowers recursively into explicit CFG blocks in both return and continuation contexts.
  - MIR output now applies deterministic canonical block-id remapping after lowering, including terminator target rewrites.
  - `sec4 build --emit mir` now prints textual MIR for inspection.
  - `sec4 build --emit mir-json` now emits machine-readable MIR JSON (JSON-only stdout mode).
  - MIR unit tests, MIR fixture-based golden tests, and CLI integration tests cover lowering and emit-path behavior.
- M6 bootstrap has started:
  - `sec4 build --emit c` now emits C source from lowered MIR.
  - `sec4 build --emit c-bin` now writes generated C and compiles a runnable binary via `clang`.
  - `sec4 run` now executes binaries produced via the C compile pipeline.
  - C backend compile flow now emits explicit runtime ABI artifacts (`sec4_runtime.h` + `sec4_runtime.c`) and links them with generated C.
  - Generated C return paths now route scalar returns through runtime ABI identity intrinsics (`sec4_rt_identity_i64`, `sec4_rt_identity_bool`).
  - C emission now rewrites `time.now` intrinsic calls to runtime ABI symbol `sec4_rt_time_now`, with runtime header/source coverage.
  - CLI integration now includes a non-trivial `c-bin` fixture covering function calls + `if/else` control flow end-to-end.
  - Runtime ABI C assets are now externalized under `runtime/c/` and emitted via `include_str!` from canonical runtime files.
  - CLI integration now includes end-to-end `time.now` intrinsic coverage through `c-bin` builds and runnable binaries.
  - C emission now lowers `log.info/warn/error/emit` intrinsic calls to runtime symbol `sec4_rt_log_any`, with runtime stub and `c-bin` integration coverage.
  - C emission now lowers `req.json`, `res.json`, and `res.html` intrinsics to runtime symbols with stub implementations and `c-bin` integration coverage.
  - C emission now lowers `res.setHeader` and `res.addCookie` intrinsics to runtime symbols with stub implementations and `c-bin` integration coverage.
  - C emission now lowers core IO intrinsics (`db.*`, `fs.*`, `httpClient.get*`) to runtime symbols with stub implementations and `c-bin` integration coverage.
  - C emission now lowers `secrets.get` / `secrets.reveal` intrinsics to runtime symbols with stub implementations (plus `secrets.get` `c-bin` integration coverage).
  - C emission now lowers validator/sanitizer/url/path gate intrinsics (`validate.*`, `sanitize.html`, `url.*`, `path.under`) to runtime symbols with stub implementations and `c-bin` integration coverage.
  - Core and CLI tests cover the C emit path (`c_backend` + CLI build output checks).
- M7 bootstrap has started:
  - Semantic layer now recognizes router intrinsics (`http.router`, `http.get`, `http.post`, `http.serve`) with `http.serve` requiring `effects { net }`.
  - C emission now lowers router intrinsics to runtime symbols (`sec4_rt_http_router`, `sec4_rt_http_route_get`, `sec4_rt_http_route_post`, `sec4_rt_http_serve`).
  - Runtime ABI now includes router bridge stubs for these symbols, with clang-gated `c-bin` integration coverage.
  - Added `examples/hello-api` bootstrap sample and clang-gated `c-bin` integration coverage asserting router + req/res lowering in generated C.
  - Added clang-gated `sec4 run` integration coverage for `examples/hello-api`.
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
  - `req.json(...)` now narrows accepted schema descriptors to bridge-name `String` values or typed `Schema<_>` descriptors.
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
- M9 release hardening now includes an executable alpha gate script:
  - `scripts/release-alpha-gate.sh` runs tests, locked builds, deterministic SBOM/metadata checks, policy-audited sample gates, and artifact capture in one command.
- M9 release hardening gate is now CI-wired:
  - `.github/workflows/alpha-release-gate.yml` executes the same gate on manual dispatch and alpha-tag pushes, and uploads captured artifacts.
- Release contract tests are now CI-smoked on PR/main:
  - `.github/workflows/release-contract-smoke.yml` runs release/promotion/publish contract tests continuously (`test-verify-release-promotion-inputs`, `test-generate-release-publish-manifest`, `test-verify-release-publish-manifest`, plus alpha workflow contract check).
  - naming-lock CI now enforces release-contract-smoke workflow step wiring via `scripts/test-release-contract-smoke-workflow-contract.sh`.
- Alpha release-gate artifacts now include explicit policy identity + naming-lock status:
  - captures the active policy profile file in `build/release-alpha-gate/`,
  - records policy profile SHA256 in `checksums.txt` and `summary.txt`,
  - stamps `naming lock: PASS` in release summary output.
- Alpha release-gate artifacts now stamp compiler/runtime binary identity:
  - records `sec4` binary SHA256,
  - records runtime ABI source/header SHA256,
  - captures runtime ABI files in release artifacts for traceable rebuild provenance.
- Alpha release-gate now verifies metadata identity hash consistency:
  - verifies `build_metadata.json` (`policyHash`, `compilerHash`, `runtimeHash`) matches `sec4 audit` JSON fields per sample,
  - verifies identity hashes are consistent across all gated release samples,
  - stamps verified identity hashes into release `checksums.txt` and `summary.txt`.
- Release-publish integration checks now verify gate artifact consistency before promotion:
  - new script `scripts/verify-release-promotion-inputs.sh` validates checksum/summary identity stamps against copied artifacts (`policy`, runtime files, per-sample metadata/sbom/audit),
  - `.github/workflows/alpha-release-gate.yml` now runs promotion-input verification after `release-alpha-gate.sh`.
- Alpha release gate now enforces milestone-closure readiness:
  - `scripts/release-alpha-gate.sh` now runs `scripts/check-milestone-closure.sh --fail-on-pending`,
  - release summary now stamps `milestone closure: PASS`,
  - promotion verifier now requires the `milestone closure` summary stamp to be `PASS`.
- Release automation now emits a publish-consumption manifest:
  - `scripts/generate-release-publish-manifest.sh` builds `publish-manifest.json` from verified release artifacts,
  - `.github/workflows/alpha-release-gate.yml` now generates publish manifest before artifact upload.
- Release automation now verifies downstream publish-manifest consumption:
  - `scripts/verify-release-publish-manifest.sh` validates manifest identity/artifact bindings against release checksums,
  - `.github/workflows/alpha-release-gate.yml` now verifies publish manifest before artifact upload.
- Release publish-manifest contract now carries readiness checks:
  - `generate-release-publish-manifest.sh` now exports `checks.namingLock` and `checks.milestoneClosure` from release summary into `publish-manifest.json`,
  - `verify-release-publish-manifest.sh` now requires those checks to match summary and remain `PASS`,
  - release publish-manifest tests now include missing-summary-entry and tampered-check negative coverage.
- Alpha release workflow wiring now has a CI contract test:
  - `scripts/test-alpha-release-workflow-contract.sh` validates required release/promotion/publish steps and artifact upload bindings in `.github/workflows/alpha-release-gate.yml`,
  - `.github/workflows/naming-lock.yml` now runs this contract test on PRs and `main` pushes.
- Release-contract-smoke workflow contract is now stricter:
  - `.github/workflows/release-contract-smoke.yml` now executes both workflow guard regression tests (`test-alpha-release-workflow-contract-guard.sh`, `test-release-contract-smoke-workflow-contract-guard.sh`) in addition to release/promotion/publish contract checks,
  - `scripts/test-release-contract-smoke-workflow-contract.sh` and `M9-E` closure gating now require those guard steps.
- M12 naming-alignment enforcement has started:
  - legacy nested `sec` security subcommand alias has been removed from CLI/tests in favor of canonical `sec4 audit`,
  - `scripts/check-naming-lock.sh` now enforces locked naming tokens and legacy-pattern absence in tracked source/docs,
  - alpha release gate now runs naming-lock validation before artifact checks,
  - `.github/workflows/naming-lock.yml` runs the naming-lock check on `main` pushes and pull requests,
  - naming-lock validation now also enforces benchmark implementation IDs (`sec4`, `go`, `node`, `rust`, `c`) and rejects legacy IDs in benchmark testdata.
- Benchmark script smoke CI is now in place:
  - `.github/workflows/benchmark-smoke.yml` runs key M10 harness smoke tests (`test_preflight`, `test_compare_matrix`, `test_publish_report`) on pull requests and `main` pushes.
- Benchmark harness now supports `wrk` fallback when `wrk2` is unavailable:
  - preflight accepts either `wrk2` (preferred) or `wrk` (fallback),
  - profile runner emits explicit warning when using `wrk` fallback and omits constant-rate `-R` flag.
  - benchmark Makefile profile targets (`bench-ping`, `bench-decode`, `bench-users`, `bench-users-get`) now route through `run_profile.sh` so fallback behavior is consistent.
  - summary parser now supports both `wrk2` (`50.000%`) and `wrk` (`50%`) percentile formats,
  - compare/trend artifacts now carry `loadGenerator` + `constantRate` metadata and mark coverage/guards as `n/a` for non-constant runs,
  - endpoint compare reports now emit `loadGenerator` + `constantRate` metadata with defaults for legacy report bundles,
  - compare-matrix leader ordering now prefers `constantRate=true` runs over higher-throughput non-constant fallback runs.
- M10 cross-impl evidence import path is now wired for closure readiness:
  - `.github/workflows/benchmark-cross-impl-evidence.yml` can be manually dispatched to run `sec4/node/go/rust` `ping+decode`, enforce strict evidence quality, and publish artifact `benchmark-cross-impl-evidence`.
  - `benchmark-suite/scripts/update_cross_impl_matrix_from_ci.sh` imports the latest successful artifact (or explicit matrix path), validates required per-endpoint impl coverage (`sec4/go/node/rust`), enforces strict quality by default, and updates `benchmark-suite/results/summaries/compare-matrix.json`.
  - benchmark smoke CI now validates the importer command contract via `benchmark-suite/scripts/test_update_cross_impl_matrix_from_ci.sh`.
- `sec4 explain` now has expanded exact-code mappings for high-frequency diagnostics (`E1002`, `E1003`, `E2001`, `E2002`, `E2003`, `E4001`, `E4004`, `E5001`, `E6001`) plus policy/audit finding IDs (`ALLOW_EXPIRED`, `ALLOW_EXPIRING_SOON`, `ALLOW_COUNT_HIGH`, `ALLOW_EXPIRY_WINDOW_ROLLUP`, `CORS_CREDENTIALS_WITH_WILDCARD`, `CORS_ANY_ORIGIN`, `CORS_REFLECT_ORIGIN_ENABLED`, `CORS_VARY_ORIGIN_MISSING`, `CSP_DISABLED`, `CSP_REPORT_ONLY`, `HSTS_DISABLED_IN_PROD`, `REFERRER_POLICY_WEAK`, `NOSNIFF_DISABLED`, `XFO_DISABLED`, `CSRF_REQUIRED_BUT_DISABLED`, `CSRF_PROTECTED_METHODS_INCOMPLETE`, `COOKIE_CROSS_SITE_WITHOUT_CORS_CREDS`, `COOKIE_CROSS_SITE_WITH_WILDCARD_ORIGIN`, `COOKIE_SAMESITE_NONE_WITHOUT_SECURE`, `INTERNAL_NET_ENABLED_NO_ALLOWLIST`, `INTERNAL_NET_CALL_ALLOWLISTED`, `PUBLIC_REDIRECTS_ENABLED_WITHOUT_REVALIDATION`, `DNS_RESOLUTION_DISABLED`, `PUBLIC_EGRESS_NO_DOMAIN_POLICY`, `FS_ENABLED_NO_BASE_ALLOWLIST`, `SYMLINK_POLICY_WEAK`, `CAPTURE_REDACTION_INCOMPLETE`, `CAPTURE_ALL_IN_PROD`, `REPLAY_EFFECTS_ALLOW`, `LOG_STRUCTURED_ONLY_DISABLED`, `LOG_REMOTE_IP_ENABLED`, `LOG_USER_AGENT_ENABLED`, `SQL_RAW_ALLOWED_BY_POLICY`, `SQL_LIMIT_RULE_DISABLED`, `SQL_SELECT_WITHOUT_LIMIT`, `SECRETS_REVEAL_USED`, `SECRETS_REVEAL_ALLOWLISTED`) with direct chapter pointers.
- `sec4 gate` CLI behavior is now directly regression-tested:
  - default threshold path (`risk>=HIGH`) fails on HIGH findings,
  - JSON mode with custom threshold (`risk>=CRITICAL`) remains parseable and preserves expected finding output.
  - CLI command-surface contract checker now has fixture-based guard coverage (`scripts/test-sec4-cli-command-contract-guard.sh`) and naming-lock CI enforcement.
- `sec4 explain` now supports machine-readable output mode:
  - `sec4 explain <CODE> --format json` emits structured payload (`code`, `topic`, `summary`, `likelyActions`, `relatedCommands`, `docsPath`).
- `sec4 explain` audit-finding coverage parity is now CI-enforced:
  - `scripts/check-sec4-explain-audit-coverage.sh` fails when `sec4-core` audit finding IDs are missing in CLI explain mappings,
  - `.github/workflows/naming-lock.yml` now runs the coverage guard on pull requests and `main` pushes.
- Benchmark artifact schema/version contract is now centralized:
  - `benchmark-suite/spec/artifact-contract-v0.1.md` defines canonical artifact filenames + key sets,
  - naming-lock validation now asserts this contract spec exists and includes required schema tokens.
- Benchmark artifact contract now includes machine-validated schema assets:
  - canonical schema files live under `benchmark-suite/spec/schemas/`,
  - single-endpoint compare artifact contract now has dedicated schema/sample coverage (`compare-report.schema.json` + `sample-compare-report-ping.json`),
  - naming-lock now executes the standalone benchmark contract validator (`benchmark-suite/scripts/validate_contract_schema.sh`) as part of benchmark artifact enforcement,
  - naming-lock validation now checks benchmark sample artifacts against schema-required keys and version constants,
  - compare-matrix sample validation now enforces row-level keys (`loadGenerator`, `constantRate`) for both `compared[]` and `leader`.
- Benchmark schema validation is now runnable as a standalone command:
  - `benchmark-suite/scripts/validate_contract_schema.sh` validates schema assets against benchmark sample artifacts,
  - standalone validator now checks compare-matrix and compare-report row contracts (`loadGenerator` + `constantRate` + endpoint alignment + leader membership),
  - trend compare-matrix fixtures are now schema-validated with full `compared[]` + `leader` row-shape checks,
  - benchmark smoke now includes `test_compare_reports_contract.sh` to enforce runtime `compare_reports.sh` output shape against compare-report contract expectations,
  - benchmark smoke now includes `test_compare_matrix_contract.sh` to enforce runtime `compare_matrix.sh` output shape/invariants,
  - `.github/workflows/benchmark-smoke.yml` now runs `test_validate_contract_schema.sh` before other smoke scripts.
- Benchmark CI coverage now includes deterministic dry-run orchestrator checks:
  - `.github/workflows/benchmark-smoke.yml` now runs `test_run_comparison_matrix.sh`, `test_run_step_matrix.sh`, and `test_run_full_benchmark_suite.sh`,
  - orchestrator dry-run contracts are now validated in CI without requiring live benchmark services.
- M13-S1 benchmark trend checks have started:
  - `.github/workflows/benchmark-trend.yml` schedules scoped live benchmark execution (`node + ping`) and supports manual dispatch,
  - `benchmark-suite/scripts/check_regression_thresholds.sh` enforces first threshold guard (`p99` and target coverage) on `compare-matrix.json`,
  - benchmark smoke CI now includes `test_check_regression_thresholds.sh`.
- M13 trend retention/baseline policy is now codified:
  - `benchmark-suite/baselines/node-ping-trend-baseline.json` defines baseline comparison guard values,
  - scheduled trend workflow now runs baseline-aware threshold checks,
  - trend artifact uploads are retained for 30 days in CI (`retention-days: 30`).
- Scoped live benchmark trend workflow now covers two endpoints:
  - `benchmark-trend.yml` runs `node` live checks for `ping,decode`,
  - per-endpoint threshold checks now apply with endpoint-specific baseline policies.
- M13 promotion workflow documentation now includes a dedicated operator runbook:
  - `docs/book/313-m13-release-promotion-playbook.md` defines release gate, verifier, naming-lock, and evidence capture steps.
- M13-S2 candidate scope is now locked:
  - trend-run result codification and threshold tuning workflow documentation,
  - release publish handoff contract notes for external tooling integration.
- M13 publish handoff contract notes are now documented:
  - `docs/book/313-m13-release-promotion-playbook.md` now defines required handoff files/fields for downstream tooling,
  - `docs/book/320-m13-release-publish-handoff-notes.md` captures the explicit external publish contract checklist.
- M13 decode threshold tuning rubric is now documented:
  - `docs/book/321-m13-decode-threshold-tuning-rubric.md` defines deterministic keep/tighten/relax rules and bounded update limits.
- M13 first trend-note scaffold is now added:
  - `docs/book/322-m13-first-trend-run-results-note.md` captures first local readiness observation and explicit live-artifact follow-up steps.
- M13 trend-note rendering helper is now implemented:
  - `benchmark-suite/scripts/render_trend_note_entry.sh` renders deterministic markdown entries from `compare-matrix.json`,
  - benchmark smoke CI validates renderer behavior via `test_render_trend_note_entry.sh`,
  - scheduled trend workflow now emits `trend-note-entry.md` artifact for direct chapter updates.
- M13 trend-note importer helper is now implemented:
  - `benchmark-suite/scripts/import_trend_note_entry.sh` appends rendered entries into chapter `322` with heading-based deduplication,
  - benchmark smoke CI validates idempotent import behavior via `test_import_trend_note_entry.sh`.
- M13 trend-artifact fetch helper is now implemented:
  - `benchmark-suite/scripts/fetch_trend_artifact.sh` pulls latest successful benchmark-trend artifact package via `gh`,
  - benchmark smoke CI validates dry-run fetch command generation via `test_fetch_trend_artifact.sh`.
- M13 trend-note one-command updater is now implemented:
  - `benchmark-suite/scripts/update_trend_note_from_ci.sh` chains fetch + import flows,
  - same-heading entries are now refreshed in place via importer replace mode (`--replace-existing`) to support same-day reruns,
  - benchmark smoke CI validates update command composition and local-entry import via `test_update_trend_note_from_ci.sh`.
- M10+M13 closure refresh now has a single operator command:
  - `scripts/refresh-closure-evidence-from-ci.sh` runs cross-impl matrix import + trend-note update + benchmark evidence quality check + strict closure check (`--fail-on-pending`) in one flow,
  - closure refresh now enforces benchmark quality warnings as failures by default (`--fail-on-warning`) with explicit local override (`--quality-allow-warning`),
  - supports dry-run and local fixture-backed execution (`--matrix`, `--entry`) for deterministic smoke validation.
- Milestone closure audit gate coverage is now stricter:
  - `scripts/check-milestone-closure.sh` now enforces per-endpoint M10 impl coverage (not union-only),
  - closure now validates compare-matrix row-contract alignment (`endpoint` + leader membership and endpoint consistency),
  - closure now verifies cross-impl evidence workflow contract (scoped `sec4,node,go,rust` `ping,decode` run + strict quality + artifact upload),
  - closure now verifies scheduled trend workflow keeps strict quality + regression guard steps,
  - closure now verifies scheduled trend workflow artifact upload contract (`benchmark-trend-*` + `benchmark-suite/results`).
  - benchmark smoke CI now executes strict closure audit against repository evidence (`scripts/check-milestone-closure.sh --fail-on-pending`) and includes a workflow contract test to prevent gate-step drift.
  - roadmap closure table gate IDs are now CI-aligned to executable closure gates via `scripts/test-roadmap-closure-gate-alignment.sh` (run in `naming-lock.yml`).
  - closure now verifies release-contract smoke wiring (`M9-E`), naming-lock CI enforcement of its contract + guard tests (`M9-F`), alpha-release workflow contract wiring (`M9-G`), and naming-lock CI enforcement of alpha contract + guard tests (`M9-H`).
  - closure now verifies naming-lock CI enforcement of cross-impl workflow contract + guard tests (`M10-D`).
  - closure now verifies naming-lock CI enforcement of zed grammar pin contract + guard tests (`M11-A`).
  - closure now verifies naming-lock CI enforcement of sec4 CLI command contract + guard tests (`M12-A`).
  - release-contract-smoke workflow contract test now also enforces trigger coverage (`pull_request` + `push` on `main`).
  - closure audit evidence paths are rendered repository-relative to avoid leaking local absolute workspace paths.
  - closure now verifies benchmark-smoke workflow keeps closure + cross-impl/trend contract guard tests plus strict closure audit wiring (`M13-D`).
  - closure now verifies naming-lock CI enforcement of benchmark-trend workflow contract + guard tests (`M13-E`).
  - closure now verifies naming-lock CI enforcement of sec4 explain audit-coverage contract test (`M13-F`).
  - closure now verifies naming-lock CI enforcement of replay capture contract test (`M14-A`).
  - closure now verifies naming-lock CI enforcement of replay capture compatibility test (`M14-B`).
  - closure now verifies naming-lock CI enforcement of replay stub registry contract test (`M14-C`).
  - closure now verifies naming-lock CI enforcement of replay CLI json contract + guard tests (`M14-D`).
  - closure audit now supports machine-readable output (`--format json`) with stable gate rows (`gate`, `status`, `check`, `evidence`) and deterministic `overall`/`pendingCount` fields (including `PENDING` + non-zero count on strict-fail paths).
- Benchmark evidence quality checker is now available:
  - `scripts/check-benchmark-evidence-quality.sh` audits compare-matrix endpoint/leader integrity plus leader quality posture (`p99` validity + `constantRate`),
  - malformed endpoint contract checks (`missing compared rows`, `leader endpoint mismatch`, `leader missing in compared`) now hard-fail with exit code `2`,
  - supports advisory mode and strict mode (`--fail-on-warning`) for promotion-gate tightening,
  - scheduled benchmark trend workflow now runs strict quality mode before regression-threshold checks,
  - cross-impl evidence workflow now runs strict quality mode before publishing artifact evidence,
  - benchmark smoke CI now enforces strict-quality workflow-step presence via `scripts/test-benchmark-workflow-quality-gates.sh`.

## 0. Product Direction (Locked Constraints)

These constraints come from current Untrusted<T> docs and the new files:
- `docs/06-sec4-typescript-like-profile.md`
- `docs/07-sec4-no-inheritance-composition-model.md`
- `docs/book/43-sec4-v0-scope.md`
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
- `docs/book/72-sec4-editor-tooling-and-zed-lsp-spec.md`
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
- Deterministic security posture reporting (`sec4 audit`) and metadata tags (`security_map`) are explicit implementation targets.

## 1. Definition of Done (Minimal Real Working Untrusted<T>)

A minimal real working Untrusted<T> (v0.1-alpha) means:
- Compiler CLI exists and builds a runnable binary from `.ut` source.
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
- `sec4 audit` can emit text/json posture reports and CI-gate on severity thresholds.
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
10. Add security posture tooling (`security_map`, `sec4 audit`, deterministic severity mapping, CI gating).
11. Harden tests, diagnostics, packaging, and build integrity metadata.
12. Run post-stability benchmark suite and publish cross-language comparison results.
13. Implement official editor tooling stack (compiler service + LSP + Zed extension + tree-sitter grammar).

## 3. Milestone Plan

## M0 - Repo Foundation and Toolchain
### Build tasks
- Define canonical repository layout (`compiler/`, `runtime/`, `stdlib/`, `examples/`, `docs/`).
- Implement CLI skeleton: `sec4 build|run|check|test|fmt|lint`.
- Add diagnostics framework with spans and structured error codes.
- Add golden test harness infrastructure.

### Exit criteria
- `sec4 build` runs on a hello-world fixture.
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
- `sec4 check --emit ast` works.

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
- Add policy-as-code compiler integration (`sec4.policy` / equivalent) with hard-error enforcement.
- Introduce typed security primitives and sinks:
  - `PublicUrl` / `InternalUrl`
  - `HeaderName` / `HeaderValue`
  - `LogValue`
  - `Budget`
- Define and enforce schema-gated input trust boundaries (`req.json(schema)` canonical path).
- Lock diagnostics taxonomy and standard error model contracts.
- Define standard success envelope and structured log event contracts.
- Keep machine-readable CLI output modes JSON-only on stdout (`check --emit diagnostics-json`, `audit --format json`) for deterministic tooling/editor integration.
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
- Chapter: "sec4 audit Spec".
- Chapter: "Sensitive API Markers and security_map".
- Chapter: "Deterministic Severity Mapping for sec4 audit".
- Chapter: "Auth Policy Keys and CSRF Coupling".
- Chapter: "Auth Middleware API".
- Chapter: "M4 security_map and sec4 audit Implementation".

## M5 - MIR Lowering + Introspection
### Build tasks
- Design and implement backend-neutral MIR structures.
- Lower typed AST/HIR into MIR.
- Emit MIR text for debugging and tests.

### Exit criteria
- `sec4 build --emit mir` works on sample programs.
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
- `sec4 run examples/hello-api` serves both endpoints.
- E2E tests verify status, decode behavior, and errors.
- Middleware behavior tests cover CORS preflight, header emission, and CSRF gate behavior.

### Docs/book outputs
- Chapter: "HTTP Runtime and Request Lifecycle".
- Chapter: "Schema-Driven JSON".
- Chapter: "Building Your First Untrusted<T> API".
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
- Chapter: "M7 Slice: JSON Response Schema-Descriptor Narrowing".
- Chapter: "M7 Slice: JSON Response Meta Secret/Taint Hardening".
- Chapter: "M7 Slice: Log Event Helper Signature Hardening".
- Chapter: "M7 Slice: Log Sink Payload Signature Hardening".
- Chapter: "M7 Slice: Log Value Builder Label Signature Hardening".
- Chapter: "M7 Slice: Log Value Builder Payload Signature Hardening".
- Chapter: "M7 Slice: Log Object Builder Type Hardening".
- Chapter: "M7 Slice: Header Value CRLF Literal Hardening".
- Chapter: "M7 Slice: Header Name Literal Token Hardening".
- Chapter: "M7 Slice: Cookie Value CRLF Literal Hardening".
- Chapter: "M7 Slice: Cookie Name Literal Token Hardening".
- Chapter: "M7 Slice: Secret Equality Comparison Hardening".
- Chapter: "M7 Slice: crypto.ctEq Intrinsic Bridge".
- Chapter: "M7 Slice: security_map crypto.ctEq Tagging".
- Chapter: "M7 Slice: security_map crypto Namespace Alias Resolution".
- Chapter: "M7 Slice: security_map Helper Namespace Alias Resolution".
- Chapter: "M7 Slice: security_map ctx.caps Capability Alias Resolution".
- Chapter: "M7 Slice: security_map Direct ctx.caps Member-Call Resolution".
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
- Chapter: "M7 Slice: req.json Schema-Descriptor Narrowing".
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
- Implement `sec4 audit` text/json report based on tags + policy + deterministic severity mapping.

### Exit criteria
- Compiler/linter rejects representative insecure patterns.
- Security tests included in CI.
- Policy hash and compiler version are embedded in build metadata.
- `sec4 audit` can fail CI by configured threshold (`--fail-on risk>=...`).

### Docs/book outputs
- Chapter: "Security Model".
- Chapter: "Typed Sinks and Safe Boundaries".
- Chapter: "Policy and Lint Rules".
- Chapter: "M8 Slice: sec4 audit History-Window Trend Summary".
- Chapter: "M8 Slice: sec4 audit History-Window Severity Rollups".
- Chapter: "M8 Slice: sec4 audit History-Summary Export".
- Chapter: "M8 Slice: sec4 audit History-Window Bounds Validation".
- Chapter: "M8 Slice: sec4 audit History-Window Range Metadata".
- Chapter: "M8 Slice: sec4 audit History-Window Output-Mode Validation".
- Chapter: "M8 Slice: Core History-Window Summary Model".
- Chapter: "M8 Slice: sec4 audit historyWindow Report Embedding".
- Chapter: "M8 Slice: historyWindow Text Renderer Coverage".

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
- Start M10 only after Untrusted<T> is stable and working end-to-end (M9 exit criteria met).

### Build tasks
- Create benchmark suite with identical service behavior across implementations:
  - Endpoint A: `GET /ping` (hello HTTP overhead)
  - Endpoint B: `POST /decode` (JSON + schema validation path)
  - Endpoint C: `POST /users` and `GET /users/:id` (real DB write/read workloads)
  - Optional Endpoint D (later): fanout path with outbound call (`/enrich/:id`)
- Implement comparison services for:
  - Untrusted<T> (C + clang backend)
  - Go
  - Node.js TypeScript
  - Rust
  - Optional reference floor: minimal C server
- Standardize fairness controls:
  - same machine and runtime envelope
  - same DB schema/indexes/query text/pool size/timeouts
  - same payload shapes and validation rules
  - same load profile (constant-rate + step-load)
  - benchmark preflight checks for required tooling with deterministic early-fail output
- Run benchmark matrix and collect:
  - throughput, p50/p95/p99, error rate
  - CPU and RSS memory
  - binary size and startup time (optional)
- Add Untrusted<T>-specific validation tracks:
  - security-defaults-on cost (schema gates, typed sinks, URL safety)
  - deterministic replay demo (`capture -> replay -> same error code`)
  - `sec4 audit` posture output with policy/build stamping
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
- Chapter: "M10 Slice: Benchmark Preflight and Early-Fail Checks".
- Chapter: "M10 Slice: Cross-Impl Evidence Importer (CI -> compare-matrix)".

## M11 - Editor Tooling and Zed Integration
### Trigger condition
- Start M11 after core language/runtime behavior is stable enough for deterministic editor semantics (at minimum: M8 complete, ideally after M9 stabilization).

### Build tasks
- Expose compiler frontend as a tooling service API (AST/HIR/symbols/diagnostics/completions metadata).
- Implement `sec4audit-language-server` using compiler APIs (no duplicated parser/typechecker).
- Implement required LSP features:
  - diagnostics, definition, references, hover, completion, rename, code actions
- Implement incremental analysis and bounded execution:
  - per-file caching, dependency invalidation, request time/memory budgets
- Implement security-aware editor UX:
  - tagged diagnostics for `security`/`taint`/`secret`
  - source-to-sink notes and safe quick-fix families
- Build `zed-sec4` extension:
  - language config for `.ut`
  - LSP wiring to `sec4audit-language-server --stdio`
  - grammar registration via `tree-sitter-untrusted`
- Add tree-sitter grammar and baseline queries (`highlights`, optional `outline`/`indent`).

### Progress so far
- Completed:
  - `sec4audit-language-server` stdio bootstrap.
  - compiler-backed diagnostics (`didOpen`/`didChange`/`didClose`).
  - LSP features: definition, hover, references, implementation, completion.
  - rename workflow: `prepareRename` + `rename` (document-local).
  - baseline quick-fix code actions keyed by security/effects diagnostic codes.
  - `zed-extension` scaffold (language config + LSP wiring contract).
  - `tree-sitter-untrusted` scaffold with baseline highlight queries.
  - diagnostics budget/cap guardrails with `I9001` overflow signaling.
  - references/rename now aggregate deterministically across currently open workspace documents.
  - open-document parse cache integrated into major LSP request paths.
  - request-deadline guardrails applied to multi-document references/rename scans.
  - code actions now support concrete redact/validate/effect-declaration auto-edits for core security/effects diagnostics.
  - references/rename now include unopened workspace `.ut` files discovered from project root.
  - tree-sitter query surface now includes highlights + outline + indent + textobjects baseline.
  - tree-sitter grammar/query coverage expanded for effects clauses, member-call syntax, and richer symbol/type captures.
  - definition/hover now resolve declarations/signatures via workspace lookup (including unopened files).
  - ambiguity-safe guardrails prevent definition/rename guesses when duplicate declarations exist.
  - unopened-file scan can now be toggled for performance-sensitive editor sessions.
  - request deadlines now propagate into semantic traversal (symbol/identifier walks) for preemptive short-circuiting in heavy LSP requests.
  - expired request budgets now short-circuit uncached parse loading paths, preventing late parse work on stale requests.
  - open-document symbol indexing now caches deterministic function symbol metadata (stable IDs) and reuses it in completion/declaration scans.
  - open-document import graph tracking now invalidates dependent parse/symbol caches on dependency refresh.
  - import-edge extraction and replacement behavior now has dedicated deterministic unit coverage.
  - references/rename hit collection now targets callsite callee identifiers, reducing non-reference name collisions.
  - diagnostics analysis now short-circuits at budget boundaries (zero-budget early return + skip-analyze when parse consumed budget).
  - Zed grammar pinning now has an explicit validation gate (`scripts/check-zed-grammar-pin.sh`) to block placeholder revisions in release flow.
  - definition/references payloads now propagate stable `symbolId` metadata for resolved symbols.
  - effect quick-fix anchoring now uses parser-derived function signature windows to avoid body-text `fn` false anchors.
  - symbolId metadata coverage now explicitly includes callsite-only reference responses.
  - call-target resolution is now scope-aware, preventing shadowed local names from being resolved as function symbols in navigation/rename flows.
  - callsite navigation/editing now resolves target symbols by `symbolId` (local-first declaration resolution) and filters references/rename edits by resolved symbol identity to avoid duplicate-name cross-binding.
  - interrupt-aware core analysis hooks now exist in `sec4-core` parser/semantic pipelines, and LSP diagnostics/requested parses now use those hooks for preemptive in-stage cancellation.
  - lexer-stage interrupt checks now use the same core interrupt signal path, so budget/deadline cancellation can stop tokenization early with `I9001` info diagnostics.
  - Zed grammar integration is now pinned to an immutable `tree-sitter-untrusted` revision SHA in extension metadata.
  - unopened workspace files loaded during navigation/rename are now cached in parse/symbol stores and register dependency edges for later invalidation.
  - dependency invalidation now evicts cached unopened dependents (not only open-document dependents) when upstream files refresh/fail parse.
  - import-edge extraction now uses token-aware scanning (with line-based fallback), so multiline import forms are tracked more reliably while ignoring string/comment noise.
  - effect-declaration quickfix rewrites now use AST-backed function/effect spans for insertion anchors in multiline effect blocks, keeping edits tied to signature structure instead of line-only heuristics.
- Remaining:
  - none for current M11 scope.

### Exit criteria
- Zed can open `.ut` files with working diagnostics, go-to-definition, hover, and completion.
- Rename and references behave deterministically on multi-file test fixtures.
- Security diagnostics in editor include stable codes, spans, tags, and actionable notes.
- Tooling budgets are enforced and tested (no hangs on large/invalid inputs).
- LSP server runs with safe defaults (no implicit code execution/network).

### Docs/book outputs
- Chapter: "Untrusted<T> Editor Tooling and Zed LSP Spec".
- Chapter: "LSP Protocol Mapping and Compiler Service API".
- Chapter: "Zed Extension and Tree-sitter Integration Guide".
- Chapter: "M11 Slice: Language Server Stdio Bootstrap".
- Chapter: "M11 Slice: Compiler-Backed LSP Diagnostics".
- Chapter: "M11 Slice: Definition and Hover Navigation".
- Chapter: "M11 Slice: References Provider".
- Chapter: "M11 Slice: Implementation Provider".
- Chapter: "M11 Slice: Completion Provider".
- Chapter: "M11 Slice: Prepare-Rename and Rename".
- Chapter: "M11 Slice: Code-Action Quickfix Baseline".
- Chapter: "M11 Slice: Zed Extension Scaffold".
- Chapter: "M11 Slice: Tree-Sitter Grammar Scaffold".
- Chapter: "M11 Slice: Diagnostics Analysis Budget Guardrails".
- Chapter: "M11 Slice: Workspace Open-Document References and Rename".
- Chapter: "M11 Slice: Open-Document Parse Cache".
- Chapter: "M11 Slice: Request-Deadline Guardrails".
- Chapter: "M11 Slice: Code-Action Redact Edit".
- Chapter: "M11 Slice: Unopened Workspace-File References and Rename".
- Chapter: "M11 Slice: Code-Action Validate Edit".
- Chapter: "M11 Slice: Tree-Sitter Outline/Indent/Textobjects".
- Chapter: "M11 Slice: Workspace-Aware Definition and Hover".
- Chapter: "M11 Slice: Ambiguous Declaration Safety Guard".
- Chapter: "M11 Slice: Language Server Operations Guide".
- Chapter: "M11 Slice: Code-Action Effect-Declaration Edit".
- Chapter: "M11 Slice: Unopened-Scan Toggle".
- Chapter: "M11 Slice: Effect Quickfix Append and De-duplicate".
- Chapter: "M11 Slice: Multiline Effect Quickfix Support".
- Chapter: "M11 Slice: Deadline-Aware Semantic Walk Cancellation".
- Chapter: "M11 Slice: Tree-sitter Grammar Coverage Expansion".
- Chapter: "M11 Slice: Parse-Stage Short-Circuit on Expired Deadlines".
- Chapter: "M11 Slice: Open-Document Symbol Index and Stable IDs".
- Chapter: "M11 Slice: Open-Document Import-Graph Invalidation".
- Chapter: "M11 Slice: Callsite-Focused Reference Hit Collection".
- Chapter: "M11 Slice: Analysis-Stage Budget Short-Circuit".
- Chapter: "M11 Slice: Zed Grammar Pin Validation Gate".
- Chapter: "M11 Slice: Import Edge Extraction and Replacement Tests".
- Chapter: "M11 Slice: Symbol-ID Metadata in Definition and References".
- Chapter: "M11 Slice: Signature-Window-Aware Effect Quickfix".
- Chapter: "M11 Slice: SymbolId Metadata Coverage for Callsite References".
- Chapter: "M11 Slice: Scope-Aware Call-Target Resolution".
- Chapter: "M11 Slice: Core Parser/Semantic Interrupt Hooks".
- Chapter: "M11 Slice: Lexer Interrupt Hooks and Zed Grammar Pin".
- Chapter: "M11 Slice: Symbol-ID Callsite Binding for References and Rename".
- Chapter: "M11 Slice: Workspace Dependency Invalidation for Cached Unopened Files".
- Chapter: "M11 Slice: AST-Backed Effect Quickfix Rewrites".
- Chapter: "M11 Slice: Zed Grammar Pin CI Closure Gate".

## M12 - Naming Alignment and Ecosystem Lock
### Trigger condition
- Run once core CLI/LSP/runtime paths are stable enough to do a broad rename safely (recommended after M9, can be executed incrementally during M10/M11 if needed).

### Build tasks
- Enforce locked external names everywhere:
  - language/docs title: `Untrusted<T>`
  - CLI command/tooling brand: `sec4`
  - policy workflow commands: `sec4 audit`, `sec4 explain`, `sec4 gate`, `sec4 replay` (under the `sec4Audit` tooling brand)
  - repository URL: `https://github.com/HubDev-AI/untrusted-compiler`
  - source extension: `.ut`
  - namespace contract: `ut/std`, `ut/http`, `ut/sec`
- Remove legacy pre-rename naming from:
  - CLI help/examples and docs snippets
  - manifest/policy filenames and examples
  - benchmark harness implementation IDs and artifact names
  - editor tooling identifiers (LSP + Zed + tree-sitter scaffolds)
  - runtime ABI docs/examples
- Remove transitional aliases once migration is complete; canonical command surface is `sec4 audit`, `sec4 explain`, `sec4 gate`, and `sec4 replay`.

### Progress so far
- Completed:
  - canonical command surface now routes through top-level `sec4 audit`, `sec4 explain`, `sec4 gate`, and `sec4 replay` only (legacy nested `sec` alias removed).
  - developer workflow skill naming is aligned with locked ecosystem naming:
    - `skills/ailang-dev-workflow/` is now `skills/sec4-dev-workflow/`.
  - automated naming-lock guard (`scripts/check-naming-lock.sh`) validates:
    - absence of legacy language/extension tokens and old editor/server names,
    - presence of locked contract tokens (`Untrusted<T>`, `.ut`, `ut/std|ut/http|ut/sec`, canonical command names).
  - release hardening gate (`scripts/release-alpha-gate.sh`) now executes naming-lock validation as part of alpha readiness.
  - dedicated CI workflow (`.github/workflows/naming-lock.yml`) now enforces naming lock on pull requests and `main` pushes.
  - benchmark naming guard now verifies canonical implementation directory IDs and benchmark testdata `impl` values.
  - benchmark naming guard now validates canonical benchmark artifact filename patterns and benchmark testdata schema keys for report/summary/step artifacts.
  - naming-lock CI now includes a static CLI command-surface contract test (`scripts/test-sec4-cli-command-contract.sh`) to enforce top-level `sec4 audit|gate|explain|replay` wiring, replay effects/output field presence, and guard against legacy nested `sec` alias reintroduction.
- Remaining:
  - none for current M12 scope.

### Exit criteria
- `rg` across tracked source/docs returns no legacy pre-rename tokens outside historical commit logs and third-party artifacts.
- All docs and examples use `.ut`.
- CLI supports and documents `sec4 audit`, `sec4 explain`, `sec4 gate`, and `sec4 replay`.
- Zed + LSP scaffolding references `Untrusted<T>` + `.ut` + current grammar IDs consistently.

### Docs/book outputs
- Chapter: "M12 Naming Alignment and Ecosystem Lock".
- Update impacted chapters to keep command/file/module names consistent with locked naming contract.

## M13 - Operational Confidence and Promotion Workflow
### Trigger condition
- Start after M9/M10/M11/M12 stabilization slices are green in CI and release-gate artifacts are deterministic.

### Scope decision (locked for M13)
- Primary scope: performance consistency hardening + promotion workflow enforceability.
- Included tracks:
  - release promotion playbook and verification flow,
  - scheduled lightweight live benchmark execution on scoped impl/endpoint set,
  - explain-map coverage expansion for remaining high-signal policy/audit finding IDs.
- Deferred out of M13:
  - deep replay IO stubbing architecture expansion (candidate for M14),
  - large editor UX feature expansions beyond current quick-fix coverage.

### Build tasks
- Formalize promotion workflow contract:
  - define required release artifacts/identity stamps,
  - bind `release-alpha-gate` + `verify-release-promotion-inputs` to promote-ready checklist.
- Add scheduled benchmark workflow:
  - run scoped live benchmark set on schedule (small impl/endpoint matrix),
  - publish artifacts and markdown summary for trend tracking.
- Add benchmark regression guardrails:
  - establish baseline thresholds for key endpoints (`p99`, achieved ratio),
  - emit deterministic pass/fail signals for threshold breaches.
- Extend `sec4 explain` policy/audit finding coverage:
  - map additional finding IDs used by `sec4 audit` severity output.
  - completed: low-frequency finding IDs used by current `sec4 audit` posture rules now have exact `sec4 explain` mappings and JSON-mode coverage tests.

### First M13 slice (M13-S1) acceptance criteria
- A documented promotion playbook exists and references concrete scripts/workflows.
- Scheduled benchmark workflow exists and runs scoped live checks with artifact upload.
- At least one deterministic regression threshold check is implemented for scheduled benchmark output.

### M13-S1 tracking (live status)
- [x] Promotion playbook chapter added (`docs/book/313-m13-release-promotion-playbook.md`).
- [x] Scheduled scoped live benchmark workflow added (`.github/workflows/benchmark-trend.yml`).
- [x] First regression threshold guard implemented (`benchmark-suite/scripts/check_regression_thresholds.sh`).
- [x] Scoped live benchmark workflow expanded to additional endpoint (`decode`) after initial stability window.
- [x] Trend retention/baseline comparison policy documented and enforced.

M13-S1 go/no-go note:
- Status: `GO` for ongoing scheduled execution and trend collection.
- Conditions to revisit: recurring threshold failures in two consecutive scheduled runs, or release promotion verifier drift in alpha workflow.

### M13-S2 candidate scope (locked)
- Trend result codification:
  - publish first trend-run result note with observed endpoint signals and threshold posture summary.
- Threshold tuning workflow:
  - define deterministic rules for when to tune decode threshold/baseline values.
- External publish handoff contract:
  - define required release-manifest fields and artifact bindings for downstream publish tooling consumers.

### M13-S2 acceptance criteria
- Chapter exists with first trend-run result note and explicit observations.
- Decode threshold tuning rubric is documented and linked from benchmark trend docs.
- Release publish handoff notes chapter defines required inputs/outputs and operator checklist.

### M13-S2 tracking (live status)
- [x] Candidate scope locked and documented.
- [x] Release publish handoff contract notes documented.
- [x] First trend-note chapter added with local readiness observation and follow-up actions.
- [x] Trend-note entry renderer added for deterministic artifact-to-markdown conversion.
- [x] Trend-note entry importer added for idempotent chapter updates.
- [x] Trend-artifact fetch helper added for deterministic CI artifact retrieval workflow.
- [x] Trend-note one-command updater added for fetch+import operator flow.
- [x] First live trend-run endpoint metrics captured and appended to trend note.
- [x] Decode threshold tuning rubric documented with deterministic decision rules.

### Exit criteria
- Promotion flow is documented and executable without manual interpretation gaps.
- Scheduled benchmark signals are available and trend-comparable over time.
- Regression threshold checks can fail workflow runs deterministically.
- Explain mappings cover priority policy/audit finding IDs used in current posture reports.

### Docs/book outputs
- Chapter: "M13 Operational Confidence Scope and Plan".
- Chapter: "Release Promotion Playbook".
- Chapter: "Scheduled Benchmark Trend Workflow".
- Chapter: "M13 Slice: Benchmark Regression Threshold Guard".
- Chapter: "M13-S2 Candidate Scope and Delivery Contract".
- Chapter: "M13 Slice: Release Publish Handoff Notes".
- Chapter: "M13 Slice: Decode Threshold Tuning Rubric".
- Chapter: "M13 Slice: First Trend-Run Results Note".
- Chapter: "M13 Slice: Trend Note Entry Renderer".
- Chapter: "M13 Slice: Trend Note Entry Importer".
- Chapter: "M13 Slice: Trend Artifact Fetch Helper".
- Chapter: "M13 Slice: Trend Note Update Command".
- Chapter: "M13 Slice: Closure Audit Gate Expansion".
- Chapter: "M13 Slice: Benchmark Smoke Closure Audit Gate".
- Chapter: "M13 Slice: sec4 Explain-Coverage Closure Gate".
- Chapter: "M10 Slice: Cross-Impl Workflow Closure Contract Gate".

## M14 - Replay Capture Contract and Stubbing Track Bootstrap
### Trigger condition
- Start after M13 closure gates are green and stable in CI.

### Scope decision (M14 bootstrap)
- Primary scope: executable replay-capture contract validation and CI guardrails before deeper runtime IO stubbing.
- Included tracks:
  - replay capture JSON contract checker + guard tests,
  - naming-lock CI enforcement of replay-capture contract,
  - closure-audit integration for replay-capture CI enforcement,
  - replay stub-registry contract checker for deterministic `mock`-mode bootstrap.
- Deferred to later M14 slices:
  - deterministic net/db/fs stub artifact ingestion during replay execution.

### Build tasks
- Add capture JSON contract checker that validates required top-level/request/determinism/redaction fields.
- Add fixture guard tests covering missing IDs, encoding-mode invariants, and invalid encoding values.
- Wire replay-capture contract test into naming-lock CI and closure-audit gate checks.
- Add replay-capture compatibility checker with deterministic hash mismatch handling and policy-mismatch override.
- Wire replay-capture compatibility test into naming-lock CI and closure-audit gate checks.
- Extend `sec4` CLI command-surface contract checks so top-level `replay` is locked in CI.
- Add replay stub-registry contract checker (net stubs + redaction metadata + deterministic request-signature uniqueness).
- Add replay stub-registry guard regression tests for malformed/missing/duplicate stubs.
- Wire replay stub-registry contract test into naming-lock CI and closure-audit gate checks.
- Extend `sec4 replay` command with optional `--stubs` contract validation for deterministic mock-mode bootstrap.
- Add replay effects-mode guard semantics (`--effects deny|mock|allow`) with deterministic mode-policy checks.
- Add replay JSON output mode (`--format json`) for machine-readable CI/operator integration.
- Add replay CLI JSON-contract guard scripts and closure gate enforcement in naming-lock CI.
- Enforce mandatory replay redaction baseline (`authorization`, `cookie`, `set-cookie`; `$.password`, `$.token`, `$.secret`, `$.apiKey`) in stub-contract validators.
- Surface replay stub inventory counts (`net/db/fs`) in CLI text/json outputs for mock-mode observability.
- Enforce deterministic mock-mode net-stub matching from capture request signatures and fail with `REPLAY.STUB_MISSING` when unmatched.
- Surface deterministic matched mock net-stub response summary in replay text/json output.
- Enforce deterministic DB/FS stub entry-shape contracts and surface DB/FS ingestion summary in replay output.
- Enforce deterministic uniqueness for DB/FS stub request signatures.
- Enforce deterministic capture DB/FS dependency-signature matching in mock mode with explicit `REPLAY.DB_STUB_MISSING` / `REPLAY.FS_STUB_MISSING` failures.
- Surface deterministic mock dependency-match counts in replay text/json output (`mockDependencyMatches`).
- Reject duplicate DB/FS dependency request signatures in capture artifacts to preserve deterministic replay matching.
- Surface deterministic matched DB/FS dependency-signature lists in replay text/json output (`mockDependencySignatures`) and lock contract presence.
- Surface deterministic matched DB/FS dependency response summaries in replay text/json output (`mockDependencyStubSummaries`) and lock contract presence.
- Surface deterministic per-dependency replay trace entries in replay text/json output (`mockDependencyTraces`) and lock contract presence.
- Include numeric per-family trace indices in replay dependency traces to make machine-order assertions deterministic.

### M14-S1 tracking (live status)
- [x] Added replay-capture sample fixture (`captures/sample-capture.json`).
- [x] Added replay-capture contract checker (`scripts/check-replay-capture-contract.sh`).
- [x] Added replay-capture guard regression script (`scripts/test-replay-capture-contract.sh`).
- [x] Wired naming-lock CI enforcement for replay-capture contract test.
- [x] Added strict closure gate `M14-A` for replay-capture CI enforcement.
- [x] Added replay-capture compatibility checker (`scripts/check-replay-capture-compat.sh`).
- [x] Added replay-capture compatibility regression script (`scripts/test-replay-capture-compat.sh`).
- [x] Wired naming-lock CI enforcement for replay-capture compatibility test.
- [x] Added strict closure gate `M14-B` for replay-capture compatibility CI enforcement.
- [x] Hardened `sec4` CLI command contract/guard scripts to require top-level `replay` command shape.
- [x] Added replay stub-registry sample fixture (`captures/sample-replay-stubs.json`).
- [x] Added replay stub-registry contract checker (`scripts/check-replay-stub-registry-contract.sh`).
- [x] Added replay stub-registry regression script (`scripts/test-replay-stub-registry-contract.sh`).
- [x] Wired naming-lock CI enforcement for replay stub-registry test.
- [x] Added strict closure gate `M14-C` for replay stub-registry CI enforcement.
- [x] Extended `sec4 replay` with optional `--stubs` contract validation and duplicate-signature detection.
- [x] Extended `sec4 replay` with `--effects deny|mock|allow` mode checks (`mock` requires stubs, `allow` warns).
- [x] Hardened static `sec4` CLI command contract tests to require replay effects/output field wiring.
- [x] Extended `sec4 replay` with `--format json` machine-readable success payload output.
- [x] Added replay CLI json contract + guard scripts and strict closure gate `M14-D`.
- [x] Enforced required replay redaction headers/jsonPaths in shell + CLI stub-contract validators with regression tests.
- [x] Added replay stub inventory summary output (`stubCounts` JSON + text summary line).
- [x] Added deterministic mock-mode capture-signature matching with explicit `REPLAY.STUB_MISSING` failures.
- [x] Added mock-mode matched net-stub response summary output (`status`, `truncated`, `bodyKind`) for deterministic observability.
- [x] Locked replay CLI json contract for mock fields (`mockRequestSignature`, `mockMatchedStub`) with guard fixtures.
- [x] Added DB/FS stub entry-shape validation and deterministic replay summary output (`stubDetails`) for DB template/FS op coverage.
- [x] Added deterministic duplicate-signature rejection for DB/FS stubs in both CLI and shell contract checks.
- [x] Added deterministic mock-mode capture dependency-signature matching for DB/FS with explicit `REPLAY.DB_STUB_MISSING` / `REPLAY.FS_STUB_MISSING` diagnostics.
- [x] Added replay mock dependency-match summary output in text/json (`mockDependencyMatches`) with CLI integration coverage.
- [x] Added duplicate-signature rejection for capture DB/FS dependency requests in replay contract checks (CLI + shell) with regression coverage.
- [x] Added replay mock dependency-signature summary output (`mockDependencySignatures`) and locked JSON contract coverage in replay guard scripts/tests.
- [x] Added replay mock dependency response-summary output (`mockDependencyStubSummaries`) and locked JSON contract coverage with text/json replay assertions.
- [x] Added replay mock dependency trace output (`mockDependencyTraces`) with deterministic `traceId` sequencing and JSON/text contract coverage.
- [x] Added numeric dependency trace indices (`index`) and multi-entry ordering coverage for `mockDependencyTraces` in replay JSON tests.
- [x] Enforced replay capture URL-derivation contract (`request.url` or non-empty `request.scheme`+`request.host`, with non-empty `request.path`) in CLI + shell validators with regression coverage.
- [x] Enforced replay base64 body integrity contract (`request.body.sha256`) for capture validation in CLI + shell validators with regression coverage.
- [x] Enforced optional replay request-query typing contract (`request.query` must be a string when present) in CLI + shell validators with regression coverage.
- [x] Enforced optional replay request URL-field typing contracts (`request.url`/`request.scheme`/`request.host` must be non-empty strings when present) in CLI + shell validators with regression coverage.
- [x] Enforced optional replay request-route typing contract (`request.route` must be a non-empty string when present) in CLI + shell validators with regression coverage.
- [x] Enforced replay body-encoding exclusivity contract (`encoding=none` must not carry `request.body.bytes`) in CLI + shell validators with regression coverage.
- [x] Enforced replay base64-body syntax contract (valid base64 required for `encoding=base64`) in CLI + shell validators with regression coverage.
- [x] Enforced replay HTTP-method shape contract (`request.method` must be non-empty uppercase token) in CLI + shell validators with regression coverage.

### Exit criteria
- Replay-capture contract checker is deterministic and fixture-tested.
- CI fails if replay-capture contract enforcement is removed from naming-lock workflow.
- Replay-capture compatibility checker enforces compiler/runtime hash parity and policy-hash mismatch handling.
- CI fails if replay-capture compatibility enforcement is removed from naming-lock workflow.
- Replay stub-registry contract checker enforces deterministic net-request signature uniqueness and redaction metadata shape.
- CI fails if replay stub-registry contract enforcement is removed from naming-lock workflow.
- Closure audit reports replay-capture gate status alongside existing milestone gates.
- CLI contract guard fails if top-level `sec4 replay` command wiring is removed.
- `sec4 replay --stubs <path>` fails deterministically on invalid stub-registry contract payloads.
- `sec4 replay --effects mock` fails deterministically without `--stubs`; `--effects allow` emits explicit risk warning.
- `sec4 replay --format json` emits parseable structured payload with mode/hash-match/warning + stub-count fields.
- Replay CLI JSON contract tests enforce presence of `mockRequestSignature` and `mockMatchedStub` fields.
- Replay CLI JSON contract tests enforce presence of `stubDetails` field.
- CI fails if replay CLI json contract + guard enforcement is removed from naming-lock workflow.
- Replay stub-contract validation fails when required redaction headers/jsonPaths are incomplete.
- Replay outputs include deterministic stub inventory counts when `--stubs` is supplied.
- Replay `mock` mode fails deterministically with `REPLAY.STUB_MISSING` when capture request signatures are not present in `stubs.net`.
- Replay `mock` mode success outputs include deterministic matched-stub response summary in both text/json formats.
- Replay stub contracts validate DB/FS entry shapes when present and replay outputs include deterministic `stubDetails` summaries.
- Replay stub contracts reject duplicate request signatures across net, DB, and FS stub families.
- Replay `mock` mode fails deterministically with `REPLAY.DB_STUB_MISSING` / `REPLAY.FS_STUB_MISSING` when capture dependency signatures are not present in `stubs.db` / `stubs.fs`.
- Replay `mock` mode JSON/text outputs include deterministic dependency-match summaries (`mockDependencyMatches`).
- Replay capture-contract validation rejects duplicate DB/FS dependency request signatures to prevent ambiguous dependency matching.
- Replay `mock` mode JSON/text outputs include deterministic dependency-signature lists (`mockDependencySignatures`) for matched DB/FS capture dependencies.
- Replay `mock` mode JSON/text outputs include deterministic dependency response summaries (`mockDependencyStubSummaries`) for matched DB/FS capture dependencies.
- Replay `mock` mode JSON/text outputs include deterministic per-dependency trace entries (`mockDependencyTraces`) for matched DB/FS capture dependencies.
- Replay dependency traces include deterministic per-family numeric `index` fields and preserve capture dependency order.
- Replay capture contract rejects requests that cannot derive deterministic net signatures (`request.url` or `request.scheme`/`request.host` + non-empty path required).
- Replay capture contract requires non-empty `request.body.sha256` for `encoding=base64` and `encoding=none`.
- Replay capture contract rejects non-string `request.query` values when query is present.
- Replay capture contract rejects non-string/empty optional URL fields (`request.url`, `request.scheme`, `request.host`) when present.
- Replay capture contract rejects non-string/empty optional `request.route` values when route is present.
- Replay capture contract rejects mixed body representation where `encoding=none` still includes `request.body.bytes`.
- Replay capture contract rejects invalid base64 payload text when `request.body.encoding=base64`.
- Replay capture contract rejects non-uppercase `request.method` values.

### Docs/book outputs
- Chapter: "M14 Slice: Replay Capture Contract Test Harness".
- Chapter: "M14 Slice: Replay Capture Compatibility Contract Gate".
- Chapter: "M14 Slice: sec4 Replay CLI Command Contract Hardening".
- Chapter: "M14 Slice: Replay Stub Registry Contract Bootstrap".
- Chapter: "M14 Slice: Replay CLI Stub Registry Validation".
- Chapter: "M14 Slice: Replay CLI Effects-Mode Guardrails".
- Chapter: "M14 Slice: Replay CLI JSON Output Mode".
- Chapter: "M14 Slice: Replay CLI JSON Contract Closure Gate".
- Chapter: "M14 Slice: Replay Redaction Header Baseline Enforcement".
- Chapter: "M14 Slice: Replay Redaction JSONPath Baseline Enforcement".
- Chapter: "M14 Slice: Replay Stub Inventory Output".
- Chapter: "M14 Slice: Replay Mock Stub Signature Match".
- Chapter: "M14 Slice: Replay Mock Stub Response Summary".
- Chapter: "M14 Slice: Replay Mock Stub JSON Contract Lock".
- Chapter: "M14 Slice: Replay DB/FS Stub Contract and Summary Ingestion".
- Chapter: "M14 Slice: Replay DB/FS Stub Signature Uniqueness Enforcement".
- Chapter: "M14 Slice: Replay Mock DB/FS Dependency Signature Match and Counts".
- Chapter: "M14 Slice: Replay Capture Dependency Signature Uniqueness Enforcement".
- Chapter: "M14 Slice: Replay Mock Dependency Signature Output Contract".
- Chapter: "M14 Slice: Replay Mock Dependency Stub Summary Output Contract".
- Chapter: "M14 Slice: Replay Mock Dependency Trace Output Contract".
- Chapter: "M14 Slice: Replay Mock Dependency Trace Index and Ordering Contract".
- Chapter: "M14 Slice: Replay Capture URL-Derivation Contract Enforcement".
- Chapter: "M14 Slice: Replay Capture Base64 Body Hash Contract Enforcement".
- Chapter: "M14 Slice: Replay Capture Query Type Contract Enforcement".
- Chapter: "M14 Slice: Replay Capture URL-Field Type Contract Enforcement".
- Chapter: "M14 Slice: Replay Capture Route Type Contract Enforcement".
- Chapter: "M14 Slice: Replay Capture Body-Encoding Exclusivity Contract Enforcement".
- Chapter: "M14 Slice: Replay Capture Base64 Syntax Contract Enforcement".
- Chapter: "M14 Slice: Replay Capture HTTP-Method Shape Contract Enforcement".

## M15 - Replay Runtime Stubbing and Deterministic Re-Execution
### Trigger condition
- Start after M14 bootstrap contract gates (`M14-A`/`M14-B`/`M14-C`/`M14-D`) are consistently green in CI.

### Scope decision (M15)
- Primary scope: move replay from contract-validation-only bootstrap into deterministic runtime execution paths for `mock` mode.
- Included tracks:
  - deterministic stub selection and response materialization for net/db/fs replay operations,
  - replay diagnostics and output contracts for runtime stub misses/mismatches/truncation behavior,
  - deterministic replay summary/auditability output that can be consumed by CI and incident workflows.
- Deferred out of M15:
  - full capture-time dependency recording expansion beyond current contract shape,
  - production-grade distributed replay orchestration,
  - cross-service multi-capture replay scheduling.

### Build tasks
- Implement deterministic runtime net stub resolution in replay execution path:
  - map request signature -> selected net stub response,
  - preserve deterministic selection behavior across duplicate-safe registries.
- Implement deterministic runtime DB stub materialization:
  - map DB dependency signature -> row payload summary/result surface,
  - expose deterministic row-count/truncation behavior to replay outputs.
- Implement deterministic runtime FS stub materialization:
  - map FS dependency signature -> operation result (`ok`, `bytes`, error surface when modeled),
  - preserve deterministic ordering when multiple FS dependencies are replayed.
- Add explicit replay runtime diagnostics contract:
  - runtime stub-miss diagnostics for net/db/fs with stable codes and signature evidence,
  - mismatch diagnostics when stub payloads violate expected runtime shapes.
- Expand replay JSON/text outputs:
  - include deterministic executed-stub traces (net/db/fs),
  - include deterministic replay execution summary fields suitable for CI contract checks.
- Add regression and contract tests:
  - CLI integration tests for runtime stub execution in `--effects mock`,
  - shell contract tests for output-field presence and deterministic execution invariants.
- Wire naming-lock CI + closure audit for new replay-runtime guard scripts.

### M15-S1 acceptance criteria
- `sec4 replay --effects mock --stubs <file>` executes deterministic net/db/fs stub materialization paths, not only contract prechecks.
- Runtime replay outputs include deterministic executed-stub summaries/traces in both text and JSON formats.
- Runtime replay failure diagnostics for unresolved/malformed stub execution are deterministic and signature-backed.
- CI guards fail deterministically if runtime replay execution contracts are removed or drift.

### M15-S1 tracking (live status)
- [ ] Scope locked and documented.
- [ ] Runtime net stub materialization wired in replay `mock` path.
- [ ] Runtime DB stub materialization wired in replay `mock` path.
- [ ] Runtime FS stub materialization wired in replay `mock` path.
- [ ] Replay runtime execution JSON/text contracts defined and tested.
- [ ] Runtime replay guard scripts added and wired into naming-lock CI.
- [ ] Closure-audit checks expanded with M15 runtime replay enforcement gates.

### Exit criteria
- Replay `mock` mode performs deterministic runtime stub execution across net/db/fs paths.
- Replay outputs expose deterministic executed-stub evidence sufficient for incident/debug workflows.
- Runtime replay guard scripts and CI wiring prevent silent contract regressions.
- Closure audit reports M15 replay-runtime gate status alongside existing milestone gates.

### Docs/book outputs
- Chapter: "M15 Runtime Replay Stubbing Scope and Plan".
- Chapter: "M15 Slice: Runtime Net Stub Materialization".
- Chapter: "M15 Slice: Runtime DB Stub Materialization".
- Chapter: "M15 Slice: Runtime FS Stub Materialization".
- Chapter: "M15 Slice: Replay Runtime Diagnostics Contract".
- Chapter: "M15 Slice: Replay Runtime Output Contract Guard".

## M16 - Live HTTP Runtime Bootstrap (`sec4 run` E2E Serving)
### Trigger condition
- Start after compile-path HTTP bridge slices are stable (`http.router/get/post/serve`, middleware routing chain, and handler bridge contracts already green in `c-bin` tests).

### Scope decision (M16-S1)
- Primary scope: replace no-op HTTP runtime stubs with a minimal executable HTTP server path so emitted binaries can serve real traffic.
- Included in M16-S1:
  - typed runtime ABI for HTTP route/serve/middleware pass-through and `res.text`,
  - in-process router table with deterministic exact method/path matching (`GET`/`POST`),
  - socket-based HTTP request loop with real response writes and deterministic 404/400 fallbacks,
  - deterministic `oneshot` runtime mode for test harnesses.
- Deferred out of M16-S1:
  - full typed request decoding/body plumbing for handler inputs,
  - JSON/HTML response materialization parity,
  - middleware-enforced security behavior beyond pass-through chaining.

### Build tasks
- Update runtime ABI signatures in `runtime/c/sec4_runtime.h`:
  - `sec4_rt_res_text(int64_t status, const char *body)`,
  - typed `sec4_rt_http_route_get/post`, `sec4_rt_http_serve`, and middleware pass-through functions.
- Implement minimal runtime HTTP engine in `runtime/c/sec4_runtime.c`:
  - router allocation and route registration,
  - request-line parsing + route dispatch,
  - response assembly and write (`HTTP/1.1`, `Content-Length`, `Connection: close`),
  - deterministic fallback responses (`400` bad request, `404` not found).
- Add deterministic serve mode switch for tests:
  - default loop mode for real `sec4 run` behavior,
  - `SEC4_RT_HTTP_SERVE_MODE=oneshot` with configurable timeout (`SEC4_RT_HTTP_SERVE_TIMEOUT_MS`) for CI/tests.
- Expand test coverage:
  - `sec4-core` runtime ABI assertion updates for new typed signatures,
  - CLI integration test that compiles and runs a temp HTTP project, executes `GET /health`, and validates `200 OK` + `ok` body.
- Keep existing `sec4 run` integration tests deterministic by running them with explicit oneshot env mode.

### M16-S1 acceptance criteria
- `sec4 run --path examples/hello-api` executes compiled binary against a live HTTP runtime path (no-op stubs removed for router/serve path).
- In default mode, runtime enters serving loop and can process requests.
- In oneshot mode, runtime accepts one request (or timeout) and exits deterministically.
- A compiled Untrusted<T> service can answer `GET /health` with `HTTP/1.1 200 OK` and body `ok`.
- Existing compile/run integration suites remain green.

### M16-S1 tracking (live status)
- [x] Typed runtime ABI introduced for route/serve/middleware + `res.text`.
- [x] Minimal HTTP router and serve loop implemented in C runtime.
- [x] Deterministic oneshot test mode implemented (`SEC4_RT_HTTP_SERVE_MODE=oneshot`).
- [x] `sec4-core` runtime ABI assertions aligned to typed signatures.
- [x] CLI integration coverage added for real `/health` request/response execution.
- [x] Existing `sec4 run` integration tests updated to use oneshot mode for deterministic completion.

### Exit criteria
- Runtime no longer treats HTTP routing/serving as placeholders for the core `hello-api` flow.
- Developers can run compiled services and observe end-to-end request handling behavior.
- Test harnesses can exercise HTTP runtime deterministically without hanging.

### M16-S2 follow-up slice (JSON response materialization)
#### Scope
- Bootstrap response materialization for JSON-oriented response intrinsics so API routes can return structured payloads, not only plain text.

#### Build tasks
- Implement minimal runtime JSON response wiring:
  - `sec4_rt_res_ok` returns deterministic `201` JSON payload,
  - `sec4_rt_res_json` returns deterministic `200` JSON payload,
  - `sec4_rt_res_ok_meta` returns deterministic JSON envelope with meta field.
- Keep `req.json` as bridge-stage decode placeholder while preserving successful handler flow.
- Add runtime E2E integration coverage for `POST /users`:
  - compile a temp fixture with `req.json + res.ok`,
  - send HTTP POST request,
  - assert `201 Created`, JSON content-type, and JSON response body.

#### Acceptance criteria
- Runtime can serve at least one JSON response route end-to-end (`POST /users`).
- Response status + content-type + body are deterministic across runs in oneshot mode.
- Existing `json_output` integration suite remains green.

#### Tracking (live status)
- [x] `res.ok` runtime materialization returns JSON `201` response.
- [x] `res.json` runtime materialization returns JSON `200` response.
- [x] `res.okMeta` runtime materialization returns JSON response with `meta`.
- [x] `POST /users` runtime E2E integration test added and green.

### M16-S3 follow-up slice (status propagation hardening)
#### Scope
- Ensure runtime JSON success helpers honor explicit status values from compiled handler calls.

#### Build tasks
- Update runtime helpers:
  - `sec4_rt_res_ok(status, ...)` uses provided status when positive.
  - `sec4_rt_res_ok_meta(status, ...)` uses provided status when positive.
- Preserve stable JSON body/content-type behavior from M16-S2.
- Add E2E integration coverage for custom status behavior (`202`) on `res.ok`.

#### Acceptance criteria
- Runtime response line reflects caller-provided status for `res.ok`.
- Existing response materialization behavior remains deterministic.
- Full `sec4` and `sec4-core` test suites remain green.

#### Tracking (live status)
- [x] `res.ok(status, ...)` propagates caller status.
- [x] `res.okMeta(status, ...)` propagates caller status.
- [x] Custom-status (`202`) runtime E2E integration test added and green.

### M16-S4 follow-up slice (runtime request JSON gate bootstrap)
#### Scope
- Connect runtime `req.json(...)` to actual inbound request body so handler paths can fail fast on malformed JSON payloads.

#### Build tasks
- Capture request body in runtime request context during HTTP dispatch.
- Implement bridge-level JSON shape check in `sec4_rt_req_json(...)`:
  - accept object/array-shaped JSON text,
  - reject empty/malformed payloads.
- Ensure success helpers (`res.ok`, `res.json`, `res.okMeta`) preserve JSON-gate failures and do not overwrite 400 error responses.
- Add E2E integration coverage for invalid JSON POST payload against `/users`.
- Keep runtime ABI compatibility for mixed schema token lowering paths (string literals + erased schema locals).

#### Acceptance criteria
- Invalid JSON request body on a `req.json(...)` route returns deterministic `400 Bad Request` response.
- Valid JSON body still follows existing success route path.
- Full `sec4` and `sec4-core` suites remain green after runtime gate integration.

#### Tracking (live status)
- [x] Runtime request body capture wired for route handlers.
- [x] `req.json(...)` bridge-level JSON validation implemented.
- [x] Invalid JSON route path returns deterministic `400` JSON error payload.
- [x] New invalid-JSON runtime E2E integration test added and green.

### M16-S5 follow-up slice (runtime `req.json` content-type enforcement)
#### Scope
- Enforce request media type constraints for `req.json(...)` so runtime JSON gate rejects non-JSON payload declarations before handler success paths run.

#### Build tasks
- Extend runtime request capture with parsed content-type classification.
- Accept only JSON media types for `req.json(...)`:
  - `application/json`,
  - `application/*+json`.
- Return deterministic `415 Unsupported Media Type` response when `req.json(...)` is invoked with missing/non-JSON content-type.
- Add E2E integration coverage for `POST /users` with `Content-Type: text/plain`.

#### Acceptance criteria
- Requests routed through `req.json(...)` fail with deterministic `415` on non-JSON content-type.
- Existing valid JSON route paths remain green.
- `sec4` and `sec4-core` suites remain green after content-type gate integration.

#### Tracking (live status)
- [x] Runtime content-type parsing is wired into request context.
- [x] `req.json(...)` enforces JSON media type and returns deterministic `415` otherwise.
- [x] New non-JSON content-type runtime E2E integration test added and green.

### M16-S6 follow-up slice (`sec4 run` live-serving command-path validation)
#### Scope
- Add explicit integration coverage that exercises the full CLI command path (`sec4 run`) while serving a real HTTP request via runtime oneshot mode.

#### Build tasks
- Add integration test fixture that:
  - creates a temporary HTTP project with `req.json(...)` + `res.ok(...)`,
  - launches `sec4 run --path <fixture>` with oneshot runtime env vars,
  - sends `POST /users`,
  - asserts deterministic JSON response contract.
- Ensure test waits deterministically and fails fast if the command exits before request handling.

#### Acceptance criteria
- `sec4 run` command path is proven to compile and execute a live HTTP runtime service.
- Request/response roundtrip through `sec4 run` yields deterministic `201` JSON response.
- Test suite remains stable with no hangs under oneshot mode.

#### Tracking (live status)
- [x] `sec4 run` command-path HTTP oneshot e2e test added.
- [x] Deterministic `POST /users` response contract asserted through CLI run path.

### M16-S7 follow-up slice (`req.json` request-size guard)
#### Scope
- Add baseline runtime request-body size enforcement for `req.json(...)` so oversized payloads fail deterministically instead of being silently truncated.

#### Build tasks
- Track request-body limit overflow during HTTP request capture.
- Extend `req.json(...)` runtime gate to return deterministic `413 Payload Too Large` on overflow.
- Add integration coverage with oversized JSON request body against `/users`.

#### Acceptance criteria
- Oversized request body on `req.json(...)` route returns deterministic `413` response.
- Runtime does not silently treat truncated oversized payload as valid route input.
- Existing request-gate and success-path tests remain green.

#### Tracking (live status)
- [x] Runtime request state tracks body-size limit overflow.
- [x] `req.json(...)` returns deterministic `413` on oversized request body.
- [x] Oversized-body runtime E2E integration test added and green.

### M16-S8 follow-up slice (`req.json` standard error envelope alignment)
#### Scope
- Align runtime `req.json(...)` gate failures with the standard error envelope shape so runtime failures match security/debuggability expectations.

#### Build tasks
- Add runtime helper to emit structured JSON error envelope:
  - `error.code`,
  - `error.kind`,
  - `error.message`,
  - `error.status`,
  - deterministic `traceId` and `timeMs`.
- Replace ad-hoc gate payloads for:
  - missing body (`400`),
  - invalid body (`400`),
  - non-JSON content-type (`415`),
  - oversized body (`413`).
- Update integration assertions to validate stable error codes and messages.

#### Acceptance criteria
- All `req.json(...)` runtime gate failures return structured standard error envelopes.
- Error payloads remain deterministic and parseable.
- Existing runtime e2e gate tests remain green with updated assertions.

#### Tracking (live status)
- [x] Runtime standard error-envelope helper implemented for gate failures.
- [x] `req.json(...)` gate failure paths migrated to stable error codes.
- [x] Runtime e2e tests updated to assert structured error-code payloads.

### Docs/book outputs
- Chapter: "M16 Slice: Live HTTP Runtime Serve Bootstrap".
- Chapter: "M16 Slice: JSON Response Materialization for Runtime Routes".
- Chapter: "M16 Slice: Runtime Status Propagation for Success Responses".
- Chapter: "M16 Slice: Runtime Request JSON Gate for Invalid Payload Handling".
- Chapter: "M16 Slice: Runtime req.json Content-Type Gate Enforcement".
- Chapter: "M16 Slice: sec4 run Live HTTP Command-Path Validation".
- Chapter: "M16 Slice: req.json Request-Size Guard Enforcement".
- Chapter: "M16 Slice: req.json Standard Error Envelope Alignment".

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
15. Produce deterministic `sec4 audit` findings for equivalent code/policy inputs.

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

1. Add first live trend-run endpoint metrics + threshold decision updates to `docs/book/322-m13-first-trend-run-results-note.md` using `benchmark-suite/scripts/update_trend_note_from_ci.sh`.

---

This roadmap is the canonical execution path until v0.1-alpha is running and documented as a coherent book.
