# Napkin

## Corrections
| Date | Source | What Went Wrong | What To Do Instead |
|------|--------|-----------------|--------------------|
| 2026-02-12 | self | Added strict DB query type checks and initially broke capability-mismatch fixtures because they passed raw string SQL literals. | When tightening sink payload types, immediately align existing capability/alias fixtures to pass typed payload symbols so each fixture still isolates a single failure mode. |
| 2026-02-12 | self | Estimated forbidden-effect diagnostic columns manually in a new semantic golden and missed by 2 chars after signature edits. | After adding security fixtures with policy diagnostics, run the golden test once and copy exact spans from failure output before proceeding. |
| 2026-02-12 | self | Added a semantic fixture for `secrets.reveal` shape hardening but forgot policy-forbidden `E2002` diagnostics also fire by default. | For `secrets.reveal` semantic fixtures, either include policy diagnostics in golden output or add a valid `@allow(...)` annotation when isolating shape-only behavior. |
| 2026-02-12 | self | Started session actions before confirming `.claude/napkin.md` existed and reading it. | Always check/create and read `.claude/napkin.md` first in-session. |
| 2026-02-12 | self | Reintroduced a moved-`Span` compile error while wiring a new semantic helper call (`enforce_res_html_signature`). | Default to passing `span.clone()` into helper calls inside semantic enforcement unless the span is consumed as final use. |
| 2026-02-12 | self | Asserted wrong runtime symbol names in a CLI integration test (`sec4_rt_header_*` instead of emitted `sec4_rt_headers_*`). | Verify intrinsic runtime symbol spelling from `c_backend.rs` rewrite table or existing integration assertions before adding new expectations. |
| 2026-02-12 | self | Assumed CLI/test hangs were caused by recent code changes; issue reproduced even on reverted state and simple local binaries. | Treat executable runtime as environment-level blocker first, but re-validate before locking the assumption. |
| 2026-02-12 | self | Treated runtime hang as persistent for the full session. | Re-run full runtime verification after environment hiccups; blockers can be transient. |
| 2026-02-12 | self | Deferred baseline verification too early due blocker assumption. | Keep full baseline verification (`cargo test`, `check`, `emit ast`, `build`) in the same slice once execution recovers. |
| 2026-02-12 | self | Inserted a new Rust test block inside an existing raw string, which produced cascading parser errors. | After patching large test files, immediately inspect the surrounding lines with `nl -ba` before running broad test suites. |
| 2026-02-12 | self | Ran `cargo test` with multiple bare test-name args, which Cargo treats as unexpected arguments. | Use one test filter, or run explicit targets (`cargo test --test <name>`). |
| 2026-02-12 | self | Assumed statement-level continuation block numbering would match previous branch-first ordering. | Reserve block IDs intentionally and update MIR tests/goldens to assert the actual deterministic numbering strategy. |
| 2026-02-12 | self | Generated C `main` with `int64_t` return type, which clang rejects. | Force emitted `main` signature to return `int` even when Untrusted<T> return type maps to `Int`/`Int64`. |
| 2026-02-12 | self | Used `status` as a temporary shell variable in `zsh`; it is readonly and broke a manual validation script. | Use a neutral temp variable name like `rc` for shell command exit codes. |
| 2026-02-12 | self | Added `sec4_rt_log_any` with mismatched header/source signatures (`();` vs `(void)`), causing runtime-content assertion drift. | Keep runtime ABI declarations/definitions identical and validate both via unit tests immediately after edits. |
| 2026-02-12 | self | Wrote a `res.json(schema, ...)` integration test using numeric schema placeholders, which violates strict schema-argument checks (`E4004`). | For compile-path req/res fixtures, pass real `Schema<T>`-typed symbols (for example function parameters) and keep net effects declared. |
| 2026-02-12 | self | Forgot that `url.public` / `url.internal` are modeled as `net` effects, so gate-only fixture failed with `E4002`. | When gate fixtures include URL validators, declare `effects { net }` explicitly. |
| 2026-02-12 | self | Used a function symbol (`handler`) as a value argument in CLI integration fixture; current semantic model does not resolve function names as first-class values there. | In compile-path fixtures, pass literal/place-holder values unless first-class function values are explicitly implemented. |
| 2026-02-12 | self | Replaced bare middleware names before dotted names in C intrinsic rewriting, producing invalid forms like `sec.sec4_rt_*`. | In string-based intrinsic rewrites, replace dotted forms before bare aliases to avoid partial-prefix corruption. |
| 2026-02-12 | self | Used `register` as a fixture function name; it maps directly to C and collides with the C keyword `register`. | Avoid C reserved keywords in Untrusted<T> integration fixtures until backend identifier mangling is implemented. |
| 2026-02-12 | self | Added an extra schema-note text to existing `E4004` diagnostics and broke semantic golden fixtures. | Preserve established diagnostic wording unless intentionally updating goldens as part of the slice. |
| 2026-02-12 | self | Asserted an exact emitted C local declaration shape (`int64_t csp = ...`) for a non-primitive typed binding, causing brittle test failure. | Assert intrinsic call lowering substrings rather than exact local declaration spelling unless declaration shape is the behavior under test. |
| 2026-02-12 | self | Ran two Cargo test commands in parallel again, hitting package-cache lock waits and noisy failures. | Run Cargo commands sequentially in this repo to avoid lock contention. |
| 2026-02-12 | self | Used `-p sec4-cli` in tests; actual package name is `sec4`. | Use `-p sec4` for CLI crate-specific test runs. |
| 2026-02-12 | self | Inserted a new Rust test block before closing an existing raw string literal, breaking test-file parsing. | After adding tests, inspect surrounding lines with `nl -ba` to confirm raw-string boundaries are intact. |
| 2026-02-12 | self | Repeated a brittle C-backend assertion using an exact local declaration shape for `sql.q` lowering. | Assert runtime-call substrings (`sec4_rt_*`) instead of exact declaration text unless declaration shape is explicitly under test. |
| 2026-02-12 | self | Introduced a local variable named `cookie` initialized from `cookie.build(...)`, which triggered alias expansion growth (`cookie.build.build...`) and made semantic analysis appear stuck. | Guard alias-name expansion against recursive suffix growth and cap alias-resolution steps; treat namespace-shadowing call aliases as a hot path for regressions. |
| 2026-02-12 | self | Used parallel tool calls for multiple Cargo test invocations again, causing lock waits/noisy output. | Run Cargo commands sequentially in this repo; parallelize reads/searches only. |
| 2026-02-12 | self | Moved `Span` into a helper call, then reused it in the same function and hit borrow-after-move compile failure. | Pass cloned spans (`span.clone()`) when the caller still needs the original for later diagnostics. |
| 2026-02-12 | self | Inserted a new Rust test into `diagnostic_tags.rs` before closing an existing raw string literal, causing parser errors that looked unrelated (`unknown prefix`, unterminated string). | After editing Rust tests with raw strings, immediately inspect surrounding lines with `nl -ba` to verify string boundaries before running tests. |
| 2026-02-12 | self | Repeated a span move regression by passing `span` by value into a new helper and then reusing it later in the same function. | When adding helper calls inside semantic enforcement, default to `span.clone()` unless the value is consumed as the final use site. |
| 2026-02-13 | self | Added stricter `db.queryOne` row-schema checks but initially missed updating the CLI `c-bin` db/fs/net integration fixture, which then failed. | When tightening semantic call contracts, immediately audit and update affected CLI integration fixtures (`json_output.rs`) before final validation. |
| 2026-02-13 | self | Advanced into M11 implementation while M9/M10 were still pending in roadmap order. | Treat roadmap milestone order as strict for active implementation: complete M9, then M10, then resume M11. |
| 2026-02-13 | self | Wrote new CLI diagnostic-fixture code with parser-invalid function syntax (`function ...`, `: Int`) and missing statement semicolon, which hid the intended semantic diagnostic under parser errors. | For CLI diagnostics integration tests, bootstrap fixtures from known-valid language examples (`examples/hello`) first, then introduce one targeted semantic misuse. |
| 2026-02-13 | self | Repeated `cargo test` misuse by passing multiple bare test-name filters in one invocation. | Use one `TESTNAME` filter per command, or run explicit target tests in separate sequential commands. |
| 2026-02-13 | self | Ran parallel Cargo tests again while lock-contention risks are known for this repo. | Keep Cargo invocations strictly sequential; parallelize only read/search commands. |
| 2026-02-13 | self | Repeated parallel Cargo invocation while running core test targets (`build_metadata` + `lockfile`). | Do not use parallel tool wrapper for any Cargo command in this repo; run each test target serially. |
| 2026-02-13 | self | Used unescaped backticks in shell heredoc while generating README placeholders; shell attempted command substitution and produced permission errors. | In shell-generated markdown, escape backticks or use quoted heredocs to prevent command substitution. |
| 2026-02-13 | self | Tried deleting a generated artifact with `rm`; command was blocked by policy. | Prefer ignore rules or non-destructive overwrites when cleanup commands are policy-restricted. |
| 2026-02-13 | self | Wrote initial benchmark Makefile commands with `benchmark-suite/`-prefixed paths, which broke under `make -C benchmark-suite`. | For Makefiles intended to run with `-C`, keep command paths relative to that directory. |
| 2026-02-13 | self | Initial Go smoke test used fixed sleep and failed intermittently due first-run compile startup delay. | Use readiness polling loops in smoke tests instead of fixed sleeps for service startup. |
| 2026-02-13 | self | Added a nested Rust benchmark crate under the repo and hit Cargo workspace ownership error. | For nested standalone crates, add local `[workspace]` in their `Cargo.toml` (or update parent workspace members/excludes) before running Cargo commands. |
| 2026-02-13 | self | Added `res.okMeta` metadata secret/taint checks on top of existing JSON sink checks, creating duplicate diagnostics in semantic goldens. | When adding specialized sink checks, narrow generic sink argument selection first (or suppress overlap) so each misuse emits one primary diagnostic. |
| 2026-02-13 | self | Inserted a call to a new semantic helper before actually defining that helper, causing a temporary unresolved method regression. | When introducing new enforcement helpers, either add the helper first or keep checks inside the existing function until extraction is complete. |
| 2026-02-13 | self | Log sink signature hardening initially risked duplicating existing secret/taint sink diagnostics. | For signature hardening on sensitive sinks, suppress type-mismatch diagnostics when payload is `Secret<_>`/`Untrusted<_>` so `E1003/E1002` remain primary. |
| 2026-02-13 | self | Tightening log-builder argument typing changed generated C literals and broke exact-string assertions in both core and CLI integration tests. | When hardening intrinsic signatures, immediately align C emission assertions to the new literal forms (`\"...\"` strings vs numeric placeholders). |
| 2026-02-13 | self | Inserted a new Rust test block before closing an existing raw string in `c_backend.rs`, causing parser errors (`unknown prefix`, unexpected delimiter). | After adding tests with embedded source strings, inspect the surrounding raw-string boundaries with `nl -ba` before running the suite. |
| 2026-02-13 | self | Added a new docs/book chapter file but initially missed linking it in `docs/book/README.md`. | For every new chapter, update the book index in the same slice before running final validation. |
| 2026-02-13 | self | Added a matrix test assertion expecting a Rust leader without including a Rust report fixture file. | Keep test fixtures synchronized with expected leaderboard assertions; add all comparator reports before asserting per-endpoint winners. |
| 2026-02-13 | self | Wrote a complex jq string interpolation with escaped quotes inside a single-quoted script block and produced a jq parse error. | For non-trivial jq conditionals, compute a temporary object/value first and concatenate plain strings instead of deeply nested escaped interpolation. |
| 2026-02-13 | self | Added orchestrator impl validation only inside runtime start path, so `--dry-run` accepted unsupported impl values. | Validate implementation lists before mode branching so dry-run and real execution enforce the same contract. |
| 2026-02-13 | self | Wrote files and directories in one parallel tool call; file writes raced directory creation and failed with \"no such file or directory\". | For file creation workflows, create directories first, then run dependent writes sequentially. |
| 2026-02-13 | self | Assumed `sec4 build` supported `--no-security-map` and `--no-build-metadata`; command failed with unexpected-argument errors. | Confirm CLI flags from `--help` (or existing tests) before scripting non-default build options. |
| 2026-02-13 | self | Captured `sec4 build --emit c` stdout directly and compiled it, but output included a leading status line that broke C compilation. | Filter emitted C output to start at the first `#include` line (or use a machine-output mode) before compiling. |
| 2026-02-13 | self | Captured service PID via `pid=\"$(start_service ...)\"` while background service inherited stdout; command substitution blocked until the service exited, making orchestrator look stuck. | Do not use command substitution for long-lived service startup; set PID via global/local variables and redirect service stdout/stderr to log files. |
| 2026-02-13 | self | Smoke readiness checks could pass/behave inconsistently due fixed shared ports and stale `/tmp/*` ping files. | Use dedicated default ports per smoke script, reset temp files, and require exact ping body (`ok`) plus process-alive checks before proceeding. |
| 2026-02-13 | self | Tried `run_comparison_matrix.sh --impls=...` while parser only accepted space-separated `--impls ...`, and missing `wrk2` then looked like startup stalling in matrix runs. | Support both `--impls ...` and `--impls=...` forms and run explicit preflight checks before orchestrator execution to fail early with dependency diagnostics. |
| 2026-02-13 | self | Benchmark Lua payload scripts relied on cwd-relative `spec/payloads/...`, which is fragile when orchestration scripts run from different working directories. | Pass payload/user-id context via env (`BENCH_PAYLOAD_FILE`, `BENCH_USER_ID`) from profile runner and keep Lua scripts deterministic across invocation contexts. |
| 2026-02-13 | self | Reached for the web tool twice while doing local file edits, which is unnecessary and noisy. | For local repo implementation work, stay on shell/apply_patch and reserve web tooling only for external verification. |
| 2026-02-13 | self | Matrix report bundling originally globbed all `impl-*.json`, so endpoint-filtered runs could include stale summaries from earlier runs. | Thread selected endpoint set into report bundling and require endpoint-scoped summary files for deterministic filtered runs. |
| 2026-02-13 | self | Endpoint-filtered reports looked structurally identical to full-suite reports, making downstream interpretation ambiguous. | Include explicit `selectedEndpoints` metadata in report bundles whenever endpoint scope is provided. |
| 2026-02-13 | self | Published markdown reports did not surface matrix endpoint scope, so partial runs could be misread as full-suite reports. | Render endpoint count/list in publish header from compare-matrix metadata to make run scope explicit. |
| 2026-02-13 | self | CLI scripts in benchmark-suite drifted on option parsing (`--impls` accepted equals-form in some scripts but not all). | Keep parser style parity (`--flag value` and `--flag=value`) across sibling scripts to reduce operator mistakes. |
| 2026-02-13 | self | Matrix comparison still scanned all report files, so stale non-selected implementations could leak into filtered-run analysis. | Scope compare-matrix input by selected implementations and fail if expected report files are missing. |
| 2026-02-13 | self | Report header clarity improved for endpoints, but implementation scope remained implicit for filtered IMPL runs. | Include explicit implementation count/list in publish header derived from matrix rows. |
| 2026-02-13 | self | Single fixed-target benchmark profiles are insufficient for identifying saturation knees and tail-latency inflection points. | Provide a step-load runner with endpoint-specific default rate ladders and aggregated per-step summary artifacts. |
| 2026-02-13 | self | Step-load outputs are hard to interpret manually when multiple rate points are captured. | Add deterministic step-analysis tooling that computes achieved-ratio trends and first knee-point below threshold. |
| 2026-02-13 | self | Step-analysis artifacts were per-impl and not directly comparable across implementations/endpoints. | Add a step-analysis matrix aggregator with scoped input support and deterministic endpoint-level leader ranking. |
| 2026-02-13 | self | Step-load artifacts existed but were not surfaced in final markdown reporting, reducing visibility of saturation behavior. | Let `publish_report.sh` accept optional `step_matrix.json` and render a dedicated step-load signals section. |
| 2026-02-13 | self | Step-load workflows still required manual multi-command orchestration and were error-prone across impl/endpoint scopes. | Add a dedicated step matrix orchestrator with dry-run, scoped execution, readiness checks, and deterministic artifact generation. |
| 2026-02-13 | self | Fixed-target and step-load orchestration became separate flows, making final combined report generation easy to forget. | Add a top-level full-suite runner that executes both phases and republishes one combined markdown report with step signals. |
| 2026-02-13 | self | Multi-artifact benchmark outputs lacked a deterministic manifest, making archival/verification harder. | Emit an artifact manifest with per-file sha256 and sizes at the end of full-suite execution. |
| 2026-02-13 | self | Even with full-suite output, missing per-impl/endpoint artifacts can go unnoticed until later analysis fails. | Add a dedicated bundle verifier that validates required artifact presence and JSON sanity for selected IMPLS/ENDPOINTS. |
| 2026-02-13 | self | Structural bundle verification can pass even when artifact contents drift from manifest. | Verify required artifact hashes against `artifact-manifest.json` by default, with explicit opt-out flag only when needed. |
| 2026-02-13 | self | LSP bootstrap negotiated initialize/shutdown but did not consume document events, so editors could not surface real diagnostics. | Wire `didOpen`/`didChange`/`didClose` to `sec4-core` parse+semantic analysis and publish/clear diagnostics through LSP notifications. |
| 2026-02-13 | self | New LSP references tests used a fixture shape (`let` call sites) that didn’t resolve to identifier hits as expected in current parser behavior, yielding empty references arrays. | Prefer minimal parser-proven call-site fixtures (`helper()` as expression statements/tails) when pinning navigation semantics. |

## User Preferences
- Keep strict milestone flow with docs updates and frequent commits (one commit per milestone slice).
- Add all design/spec changes into docs/book and keep roadmap aligned.
- Security-first language direction is mandatory.
- Avoid adding absolute local machine paths to committed docs.

## Patterns That Work
- Small vertical M4 slices with matching docs chapter updates reduce churn and keep progress reviewable.
- Deterministic `sec4 audit` findings with sample evidence improve traceability for policy/security posture.
- After transient execution issues, retrying full verification in the same session often recovers and avoids false blockers.

## Patterns That Don't Work
- Launching multiple `cargo run` commands in parallel can cause lock contention/timeouts and noisy diagnostics.

## Domain Notes
- Current focus is M4 security foundation hardening before MIR/backend milestones.
- `sec4 audit` and `security_map` are central artifacts for deterministic security posture and later editor tooling.

## Session Notes (2026-02-12)
- Repo currently has no `.trellis/` directory even though AGENTS references Trellis docs.
- Implemented and validated `sec audit --history-dir` with JSON-mode stdout contract preserved.
- Added `DNS_RESOLUTION_DISABLED` sec4 audit posture rule with sample-call evidence (prod + `net.ssrf.resolve_dns=false`).
- Added `PUBLIC_EGRESS_NO_DOMAIN_POLICY` sec4 audit posture rule (usage-gated on public-net sink calls) plus policy ingestion for `net.public.allowed_domains`/`blocked_domains`.
- Added `CSRF_PROTECTED_METHODS_INCOMPLETE` sec4 audit posture rule with deterministic missing-method evidence and middleware sample-call context.
- Added `SYMLINK_POLICY_WEAK` sec4 audit posture rule plus typed policy parsing/validation for `fs.forbid_symlinks`.
- Added `REFERRER_POLICY_WEAK` sec4 audit posture rule for weak security-header referrer-policy values.
- Added `XFO_DISABLED` and `NOSNIFF_DISABLED` sec4 audit posture findings for baseline security-header hardening gaps.
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
- Wired `sec4 run` to build via `c-bin` and execute the produced binary; added clang-gated integration coverage.
- Added M6 runtime ABI scaffolding: C backend now includes `sec4_runtime.h`, CLI writes runtime header/source into `build/`, and clang compiles generated + runtime translation units together.
- Routed scalar C return paths through runtime ABI identity helpers (`sec4_rt_identity_i64` / `sec4_rt_identity_bool`) so runtime linkage is exercised by emitted function bodies.
- Added intrinsic call rewriting in C emission for `time.now`/`time_now` -> `sec4_rt_time_now` with runtime stub coverage and C emitter tests.
- Added non-trivial `c-bin` integration coverage using a temp project fixture with helper-function call + `if/else` control flow.
- Moved runtime ABI C sources from Rust string literals into `runtime/c/` files and switched emitter helpers to `include_str!` those canonical runtime assets.
- Added clang-gated CLI integration coverage for `time.now` projects to validate semantic effects checks plus intrinsic rewrite/runtime linkage through full `c-bin` builds.
- Added log intrinsic C-lowering coverage (`log.info/warn/error/emit` -> `sec4_rt_log_any`) with runtime stubs and end-to-end `c-bin` integration tests.
- Added req/res intrinsic C-lowering coverage (`req.json`, `res.json`, `res.html`) with runtime stubs and end-to-end `c-bin` integration tests using valid `Schema<T>` parameters.
- Added header/cookie intrinsic C-lowering coverage (`res.setHeader`, `res.addCookie`) with runtime stubs and end-to-end `c-bin` integration tests.
- Added db/fs/net intrinsic C-lowering coverage (`db.*`, `fs.*`, `httpClient.get*`) with runtime stubs and end-to-end `c-bin` integration tests using capability-typed parameters.
- Added secrets intrinsic C-lowering coverage (`secrets.get`, `secrets.reveal`) with runtime stubs plus end-to-end `c-bin` coverage for `secrets.get` (non-forbidden default policy path).
- Added validator/sanitizer/url/path gate intrinsic C-lowering coverage (`validate.*`, `sanitize.html`, `url.*`, `path.under`) with runtime stubs and end-to-end `c-bin` integration tests.
- Added HTTP router intrinsic C-lowering coverage (`http.router/get/post/serve`) with runtime stubs and end-to-end `c-bin` integration tests.
- Added `examples/hello-api` as an M7 bootstrap sample plus clang-gated `c-bin` integration coverage asserting router + req/res lowering in generated C.
- Added `sec4 run` integration coverage for `examples/hello-api` so both build and run flows are pinned for the bootstrap API sample.
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
- Added `json.decode` schema-argument hardening so decoding requires `(ctx, schema, raw)` and rejects numeric/boolean/untrusted/secret schema placeholders, with schema-tagged diagnostics and aligned json-helper fixtures.
- Added `json.decode` context/payload argument-type hardening so decode calls require `Ctx` in slot 1 and `Untrusted<Bytes>` in slot 3, with schema-tagged diagnostics and aligned json-helper integration fixtures.
- Added `err.withCause` argument hardening so cause chaining enforces `(error, cause)` shape and requires `StdError` for both arguments, with security-tagged diagnostics and dedicated semantic/tag fixtures.
- Added `err.withPath` base-error argument hardening so the first argument must be `StdError`, with security-tagged diagnostics and dedicated semantic/tag fixture coverage.
- Added `err.withLimit` base-error argument hardening so the first argument must be `StdError`, with security-tagged diagnostics and dedicated semantic/tag fixture coverage.
- Added `err.withDependency` base-error argument hardening so the first argument must be `StdError`, with security-tagged diagnostics and dedicated semantic/tag fixture coverage.
- Added `err.withDetail` base-error argument hardening so the first argument must be `StdError`, while preserving key/value secret/taint checks and adding dedicated semantic/tag fixtures.
- Added `json.encode` schema-descriptor hardening so schema arguments must be explicit `Schema<_>` types (not generic placeholders like `String`), with schema-tagged diagnostics and dedicated fixture/tag coverage.
- Added `json.decode` schema-descriptor hardening so decode schema arguments must be explicit `Schema<_>` values (not placeholders like `String`), with schema-tagged diagnostics and dedicated fixture/tag coverage.
- Added `sql.q` params value-safety hardening so param payloads cannot be `Secret<_>` or `Untrusted<_>`, with tagged diagnostics and dedicated semantic/tag fixtures.
- Added `db.queryOne` row-schema descriptor hardening so row schema arguments must be `Schema<_>`; updated capability fixtures/integration samples away from string row names and added schema-tagged diagnostics.
- Narrowed `req.json` schema descriptor acceptance to `String` (bridge mode) or typed `Schema<_>` only, rejecting unrelated placeholder types with schema-tagged diagnostics.
- Narrowed JSON response sink schema descriptors (`res.json`/`res.ok`/`res.okMeta`) to `String` or `Schema<_>`, rejecting unrelated placeholders with `E4004` schema diagnostics.
- Added `security_map` coverage for `crypto.ctEq`/`crypto_ct_eq` (gate tag + arg roles + symbol registry), and validated via dedicated `security_map` tests.
- New intrinsic namespaces also need registration in `is_tagged_call_namespace`; otherwise forwarded namespace aliases lose call tagging in `security_map`.
- Forwarded namespace call tagging depends on `is_tagged_call_namespace`; when adding dotted helper APIs (`sql.*`, `json.*`, `headers.*`, `cookie.*`), keep namespace list in sync.
- Capability alias seeding from typed params is not enough; security-map canonicalization also needs member-path mapping for `ctx.caps.*` flows.
- For canonical call tagging, normalize direct capability-member call paths (`ctx.caps.db.exec`) inside alias resolution, not only alias-binding paths.
- Secret-type signature hardening cannot rely on local type inference from intrinsic calls in fixtures; use explicitly typed `Secret<_>` parameters/values in compile-path tests.
- For new CLI trend/export flags, enforce explicit flag dependencies early (`history-window` -> `history-dir`, `write-history-summary` -> `history-window`) and pin with integration tests.
- Avoid silent coercion for CLI security/reporting flags; reject invalid bounds explicitly and pin with integration tests.
- Window trend summaries are significantly more useful when anchored with oldest/latest policy hash + timestamp; include range metadata in exported payloads.
- For dual text/json CLI outputs, always add routing tests per mode so auxiliary lines never contaminate JSON stdout contracts.
- If CLI logic starts modeling audit semantics (not just I/O), move that model into `sec4-core` early to avoid parallel behavior drift.
- For trend features, attach computed window summaries to the primary report object before any stdout/file emission to avoid split-brain artifacts.
- When adding optional report sections, add renderer coverage immediately so text-mode contracts don’t regress unnoticed.
- For LSP bootstraps, start with strict stdio framing + initialize/shutdown/exit correctness and avoid advertising unsupported capabilities.
- E2001 quick-fix should not emit a guidance-only action when the effect already exists; omit the action entirely in source-backed codeAction flows to avoid misleading edits.
- Quick-fix signature scanning anchored only on the `fn` line misses multiline signatures; scan a bounded signature window up to `{` and resolve insertion anchors (`effects`, `->`, `{`) across lines.
- Loop-level request deadline guards are not enough for LSP responsiveness on large ASTs; propagate deadline checks through recursive identifier/symbol walkers to short-circuit semantic work.
- Tree-sitter scaffolds need early support for effects/member-call syntax; otherwise editor highlighting/navigation drifts from real language usage even when LSP is improving.
- Deadline checks should be applied before uncached parse fallbacks, otherwise expired requests still burn parser CPU despite downstream traversal guards.
- Treat open-document symbol data as a first-class cache alongside parsed programs; this avoids repeat symbol extraction and creates a stable base for symbol-ID-driven navigation.
- Incremental editor caches need reverse import edges; when a dependency changes, evict dependents proactively or completion/navigation will read stale symbol data.
- Function rename/reference matching should collect callsite callees rather than every same-name identifier; otherwise argument/variable identifiers create false-positive edits.
- Diagnostics pipelines should enforce budget checks at stage boundaries (pre-parse and pre-analyze); otherwise zero-budget requests still perform expensive parse/semantic work.
- Add explicit release gates for placeholder metadata (like Zed grammar rev); otherwise scaffolding values leak into production configs unnoticed.
- Dependency invalidation code needs direct edge-extraction and edge-replacement tests; otherwise reverse-map regressions hide behind broader LSP integration tests.
- Emit stable symbol IDs in navigation payloads early; it creates a migration path from name-based matching to true symbol-ID binding without breaking existing editor behavior.
- Constrain text-based quickfix anchors with parser-derived signature windows; this avoids accidental matches from body comments/strings that contain keyword-like text.
- Metadata contracts (like symbolId on reference locations) need explicit tests for each response mode (`includeDeclaration` true/false), not just one happy-path assertion.
- Cursor-target resolution for LSP should share the same scope rules as reference collection; otherwise shadowed locals still trigger wrong definition/rename operations.
- 2026-02-13 | self | Ran workspace-wide `cargo fmt` during a focused slice and unintentionally modified unrelated files. | Keep formatting scoped (or restore immediately) and commit only slice-owned files.
- 2026-02-13 | self | First interrupt wiring in semantic analysis used an unnecessary leaked handle to satisfy lifetimes. | Model interrupt signals with explicit borrowed lifetimes on `Analyzer<'a>` and avoid allocation/leak workarounds.
- 2026-02-13 | self | Parser/semantic interrupt hooks alone left lexing as a blind spot for deadline cancellation. | Propagate `InterruptSignal` into lexer loops and add a dedicated lex-stage interruption test so `I9001` can originate before parse traversal.
- 2026-02-13 | self | Initial naming-lock guard flagged its own roadmap/chapter examples because they contained literal banned tokens. | Describe banned patterns generically in docs (no literal tokens) so automated legacy scans stay signal-only.
- 2026-02-13 | self | Used a sed regex with `\s` while extracting benchmark impl IDs in bash guard script; BSD/GNU sed treated it literally and produced bad values. | Use POSIX-safe parsing for shell checks (`awk -F\"` or `[[:space:]]`), then validate with an immediate script run.
- 2026-02-13 | self | Release gate stamped compiler/runtime SHA256 values but did not verify semantic identity hashes against build metadata/audit outputs. | In release hardening slices, explicitly cross-check `build_metadata` identity fields with `sec4 audit` report fields and stamp verified identities in gate artifacts.
- 2026-02-13 | self | Naming-lock impl-ID extraction uses quote-splitting and misread compact inline JSON objects (`"leader": { "impl": ... }`) as value `impl`. | Keep benchmark fixture JSON formatted one field per line (or harden parser later) so guard scripts read actual impl IDs.
- 2026-02-13 | self | Tried building `wrk2` from source on this workstation and LuaJIT target detection failed with unsupported architecture errors. | Keep benchmark tooling resilient by supporting `wrk` fallback (with explicit warning about no `-R` constant-rate behavior) and test both code paths.
- 2026-02-13 | self | Used `jq` defaulting with `//` for boolean fields (`constantRate`) and accidentally coerced explicit `false` to `true`. | For boolean metadata defaults, use `if has("field") then .field else <default> end` instead of `//`.
- 2026-02-13 | self | Assumed renderer default generator should be `unknown`; actual desired default for missing metadata is `wrk2` (constant-rate baseline assumption). | Keep default generator + run-mode assumptions aligned with benchmark policy defaults and assert them in renderer tests.
- 2026-02-13 | self | Added strong matrix metadata (`constantRate`) but left report generation without surfacing that posture, which hid evidence-quality context in published markdown. | Whenever benchmark artifacts gain trust/quality metadata, thread it through publish_report output and tests immediately.
- 2026-02-13 | self | Changed compare-matrix leader ordering to quality-aware (`constantRate` first) but documentation still described throughput-only sorting. | When ranking semantics change in benchmark scripts, update all matrix/reporting chapters and roadmap bullets in the same slice before commit.
- 2026-02-13 | self | Evidence-quality checker existed but scheduled trend workflow still allowed running threshold checks on potentially low-quality leaders. | Wire strict evidence-quality gate before threshold checks in live trend CI so threshold analysis runs only on quality-qualified matrix inputs.
- 2026-02-13 | self | Strict evidence-quality checks were initially applied only to trend CI, leaving cross-impl evidence artifact generation less strict. | Apply the same strict quality gate consistently across all live benchmark evidence workflows to avoid mixed trust levels.
- 2026-02-13 | self | Workflow-level quality checks are easy to regress silently when only runtime scripts are tested. | Add explicit workflow contract smoke tests for required gate steps (quality checker + strict flags) so CI fails on YAML drift.
- 2026-02-13 | self | `wrk` fallback support can be bypassed if Make targets call `wrk2` directly. | Route user-facing benchmark Make targets through `run_profile.sh` and lock that behavior with a Makefile contract test.
- 2026-02-13 | self | compare-reports quality-ordering test initially reused the baseline fixture directory, so ordering assertions were contaminated by unrelated rows. | Keep targeted ordering scenarios in isolated fixture directories so assertions match the intended population.
- 2026-02-13 | self | Top-level schema-key checks can hide row-level contract drift (e.g., missing `constantRate` in compare leaders). | Add explicit row-shape validation for compare-matrix in both standalone validator and naming-lock checks.
- 2026-02-13 | self | Fixture matrices relying on implicit defaults can mask regressions in metadata propagation. | Keep benchmark fixture matrices explicit (`loadGenerator`, `constantRate`) so tests catch missing metadata wiring immediately.
- 2026-02-13 | self | Naming-lock artifact-pattern checks must match literal script text; abstract placeholders (like `compare-${endpoint}`) caused false failures. | Keep naming-lock regex tokens aligned with concrete script/config literals, and enforce generalized contracts via schema/sample validation instead.
- 2026-02-13 | self | Trend fixture matrices that omit `compared[]` diverge from canonical compare-matrix shape and weaken contract checks. | Keep trend fixtures structurally aligned with canonical matrix artifacts and validate them through the same schema/row-shape gates.
- 2026-02-13 | self | README still stated `wrk2` as mandatory after fallback support landed, which can mislead local operators. | Keep benchmark operator docs aligned with fallback semantics (`wrk2` preferred, `wrk` allowed with non-constant-rate warning).
- 2026-02-13 | self | Static fixture validation alone is not enough for compare-report contract safety; generator output can drift independently. | Add a runtime generator contract test (`test_compare_reports_contract.sh`) that validates actual `compare_reports.sh` output shape.
- 2026-02-13 | self | Benchmark-smoke initially exercised compare-matrix but not compare-report behavioral tests, leaving a CI gap. | Keep smoke coverage balanced: include both compare-report behavior tests and compare-report contract tests.
- 2026-02-13 | self | New jq invariant checks initially used ambiguous scope (`.leader` inside `any(.compared[])`) and tautological endpoint checks. | When asserting nested invariants, bind explicit variables (`. as $doc` / `. as $entry`) to avoid scope confusion and false-positive logic.
- 2026-02-13 | self | Evidence-quality checks were warning-focused and could miss malformed compare artifacts where leader/compared consistency broke. | Treat endpoint/leader contract violations as hard failures (exit 2) in quality checks, separate from advisory warning posture.
- 2026-02-13 | self | Invariant checks are easy to add but easy to under-test if only field-deletion failures are covered. | Add mismatch-based negative tests (endpoint-group mismatch, leader-not-in-compared) in addition to missing-field cases.
- 2026-02-13 | self | Naming-lock carried parallel benchmark schema checks that can drift from standalone validator behavior over time. | Make naming-lock call the standalone validator directly to keep benchmark contract enforcement single-sourced.
- 2026-02-13 | self | Keeping duplicate schema checks in naming-lock and standalone validator increases drift risk and maintenance cost. | Keep standalone contract validator as the canonical benchmark schema checker and have naming-lock call it instead of duplicating deep JSON checks.
- 2026-02-13 | self | Closure-refresh strict quality was optional by default, which allowed warning-grade evidence to slip through unless operators remembered a flag. | Make strict quality the default in closure-refresh; provide explicit `--quality-allow-warning` only for deliberate local fallback scenarios.
- 2026-02-13 | self | Cross-impl importer previously enforced impl coverage but could still import warning-grade evidence unless callers added separate quality checks. | Keep importer self-defensive: enforce strict quality by default and expose explicit `--quality-allow-warning` for deliberate local exceptions.
- 2026-02-13 | self | Added quality-allow override propagation but lacked explicit dry-run coverage, which could hide command-plan drift. | For orchestrator flags, assert dry-run command lines for both default and override paths.
- 2026-02-13 | self | Added importer quality override behavior without first-class dry-run assertions could let command-plan drift hide until runtime. | For every new CLI mode flag, add dry-run assertions for both default and override command composition.
- 2026-02-13 | self | After adding compare-report runtime contract checks, compare-matrix runtime generation still relied on behavior tests + fixture validation only. | Add dedicated generator contract tests for each artifact emitter (`compare_reports`, `compare_matrix`) and wire both into smoke CI.
