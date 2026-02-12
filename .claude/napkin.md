# Napkin

## Corrections
| Date | Source | What Went Wrong | What To Do Instead |
|------|--------|-----------------|--------------------|
| 2026-02-12 | self | Started session actions before confirming `.claude/napkin.md` existed and reading it. | Always check/create and read `.claude/napkin.md` first in-session. |
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
