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
- Repository hygiene lock:
  - forbid leaking local absolute workspace prefix tokens (for example the local `/Users/.../src/ai/` root) in tracked files,
  - enforced via CI guard script (`scripts/check-no-local-path-leaks.sh` + naming-lock workflow test).

Roadmap impact:

- Add an explicit naming-alignment slice before further feature expansion.
- Every milestone must preserve these names in:
  - CLI UX/help/docs examples
  - policy filenames and policy docs
  - benchmark harness implementation IDs and report artifacts
  - editor tooling (LSP + Zed + tree-sitter)
  - runtime ABI/docs

## Current Status (2026-02-14)

- Milestone progression reached M16 follow-up slices with active enforcement.
- M9 release hardening gate is operational both locally and in CI:
  - `scripts/release-alpha-gate.sh`
  - `.github/workflows/alpha-release-gate.yml`
- M11 editor tooling scope has no remaining tasks in this roadmap revision.
- M12 naming alignment scope has no remaining tasks in this roadmap revision.
- M4 security-foundation scope is complete for this roadmap revision (historical slice bullets retained below).
- Naming-lock workflow now includes a local-path leak guard test:
  - `scripts/test-check-no-local-path-leaks.sh`.
- M12 local-path leak closure enforcement is active (`M12-B`).
- M13 operational confidence closure gates are green.
- M13 trend-note update flow now supports local compare-matrix fallback:
  - `benchmark-suite/scripts/update_trend_note_from_ci.sh` supports `--prefer-local` and auto-fallback when remote artifact fetch fails.
- Alpha smoke is currently green (`cargo test -p sec4 --test alpha_smoke`).
- Runtime + CLI have moved beyond placeholder behavior for HTTP serving, request validation, FS/DB/NET intrinsics, and core command flows (`init/check/build/run/test/fmt/lint`), but strict no-stub alpha criteria are not fully satisfied yet.
- `c-bin` compile path now prefers canonical runtime sources from `runtime/c/` with deterministic build-folder fallback when canonical files are unavailable.

## Execution Mode Lock (2026-02-17)

Implementation work is now explicitly prioritized over repeated governance loops.

Operating rules:

1. Normal PRs must include real runtime/compiler/CLI behavior movement.
2. During coding loops, run focused implementation checks first:
   - `sec4 check/build/run` for touched scope,
   - targeted Rust tests around changed behavior.
3. Full governance suites (naming-lock/closure bundles) run once near PR completion (or in CI), not after every incremental edit.
4. Gate-only/test-only churn without logic movement is out-of-policy unless fixing a concrete broken contract.

Reference chapter:

- `docs/book/887-m38-implementation-first-execution-mode-lock.md`

## Alpha Scope Reset Addendum (2026-02-17)

This addendum is the canonical delivery lock for current execution. It exists to avoid repeated scope re-negotiation in chat threads.

Operating rules:

1. Window lock (through March 17, 2026): backend no-stub alpha completion only.
2. Implementation priority: real runtime/compiler logic and authoring usability, with multi-file module support prioritized over non-critical analyzers.
3. `M39-S2` Composition Contract Analyzer is deferred until after the alpha checkpoint unless browser-to-server promotion work becomes immediately blocking.
4. Test cadence remains implementation-first:
   - run targeted behavior tests for changed logic during coding,
   - run broad governance/closure suites near PR completion or in CI.
5. Scope decisions must be recorded in both this roadmap and a book chapter in the same PR.

Checkpoint criteria for continuing this execution mode:

1. Fresh clone `sec4 init -> sec4 check -> sec4 build --emit c-bin -> sec4 run` works locally and in CI.
2. Showcase app path validates DB/FS/NET behavior without branch-local patches.
3. Main branch CI stays green for at least 10 consecutive days.
4. No alpha-critical runtime/compiler placeholder paths remain.

## Backend Engine Transition Lock (2026-02-17)

This section resolves backend-engine direction explicitly.

1. Current C runtime is the alpha reference engine for correctness/conformance, not the final high-concurrency server engine.
2. C runtime work stays limited to:
   - alpha-critical correctness fixes,
   - deterministic behavior guarantees,
   - no broad new architecture investment beyond what is required for no-stub alpha.
3. LASM async backend bootstrap starts on February 18, 2026 as a parallel track.
4. Through the March 17, 2026 checkpoint window, execution split is:
   - 70% alpha closure + multi-file module delivery (`M39-S2A`),
   - 30% LASM async bootstrap (`M39-S2B`).
5. Browser execution does not depend on socket-based C runtime semantics:
   - browser target uses WASM + browser host ABI profile,
   - server concurrency evolution is owned by LASM async runtime track.

Checkpoint decision rule (March 17, 2026):

1. If alpha criteria are green, continue release hardening and increase LASM allocation.
2. If LASM bootstrap shows viable concurrent request handling, begin migration planning from C reference runtime to LASM server runtime.
3. If LASM bootstrap misses viability criteria, keep C as alpha reference runtime while continuing LASM in bounded slices.

## No-Stub Alpha Readiness (2026-02-15)

This section is the canonical plan for determining when v0.1-alpha is ready as a "no-stub" runtime/compiler release.

### Definition of done (no-stub alpha)

No-stub alpha is considered ready only when all items below are true:

1. `sec4 init/check/build/run/test/fmt/lint/audit/replay` are implemented with real deterministic behavior (no placeholder return-paths for supported flows).
2. Runtime HTTP path is end-to-end real for:
   - request parsing + routing + response emission,
   - JSON request/response envelopes,
   - middleware enforcement branches (cors/security headers/csrf/auth),
   - stable standard error envelope behavior.
3. Runtime security primitives are real for supported v0.1 scope:
   - URL gates (`url.public`/`url.internal`) with policy-driven checks,
   - FS base containment enforcement,
   - secret get/redact/reveal policy behavior,
   - constant-time compare path for secrets.
4. Runtime outbound net behavior covers policy-driven scheme/domain/port checks, timeout/body limits, and redirect handling contract.
5. Policy parser/model/runtime bridge parity exists for active keys used by compiler/runtime.
6. Alpha smoke + targeted runtime harness tests pass on `main` without requiring branch-local patches.

### Post-alpha priority lock (WASM/browser)

WASM/browser execution is now an explicit roadmap priority, but it is hard-gated behind no-stub alpha closure so core runtime completion is not delayed.

`WASM_START_GATE` opens only when all conditions are true:

1. no-stub alpha readiness criteria above are fully satisfied.
2. alpha decision record is `GO` (`build/m37-alpha-tag-decision-record.json`).
3. benchmark evidence is current and published for the release candidate baseline (cross-impl matrix + trend artifacts).
4. alpha artifacts are published and externally consumable (release notes + publish manifest chain).

### `WASM_START_GATE` status (2026-02-19)

- `OPEN` based on verified closure evidence:
  - alpha decision record `GO`:
    - `build/m37-alpha-tag-decision-record.json`
  - alpha tag created and pushed:
    - `v0.1.0-alpha.2`
  - post-tag publish-manifest chain verified:
    - `scripts/verify-release-promotion-inputs.sh`
    - `scripts/generate-release-publish-manifest.sh`
    - `scripts/verify-release-publish-manifest.sh`
    - `build/release-alpha-gate/publish-manifest.json`

When `WASM_START_GATE` is open, WASM backend + browser runtime profile becomes the highest-priority new feature track, executed first through the browser-to-server promotion milestone plan (M39 below).

### Post-alpha two-phase backend promotion track (browser -> server)

After `WASM_START_GATE` opens, the first execution track should prioritize browser-first prototyping with deterministic server promotion.

Target product loop:

1. Prototype instantly in browser profile (`no deploy`).
2. Validate UX/domain flows locally with deterministic state.
3. Promote same domain modules to server target with mechanical adapter swap + generated deployment scaffolding.

Post-alpha track acceptance anchors:

- Browser profile enforces strict capability boundary:
  - forbid `db.*`, `secrets.*`, `net.listen`, internal-net sinks.
  - allow browser-local persistence adapter (`localdb.*`) and constrained outbound fetch gates.
- Composition model is explicit:
  - target-agnostic domain module
  - repository interface
  - target-specific adapters (`LocalRepo` / `ServerRepo`).
- Promotion flow is compiler-driven and deterministic:
  - `sec4 promote --from browser --to server`
  - binding rewrite at composition root (`LocalRepo` -> `ServerRepo`)
  - generated server scaffold (schema/migrations, runtime wiring, deployment manifest baseline)
  - deterministic diagnostics when promotion preconditions fail.
- Promotion contract includes data portability path:
  - browser-local export artifact
  - generated server import scaffold/command path.

## M39 - Browser-First to Server Promotion MVP (Queued; gated by `WASM_START_GATE`)

### Goal

- Deliver a deterministic two-phase backend loop:
  - browser-first prototype profile (`zero deploy`)
  - mechanical promotion to server target (`sec4 promote --from browser --to server`).

### M39-S1 browser profile capability fence acceptance criteria

- Compiler/profile enforcement blocks server-only capabilities in browser mode:
  - `db.*`
  - `secrets.*`
  - `net.listen`
  - internal-net sinks.
- Browser profile allows only approved local/browser runtime capabilities (for MVP: `localdb.*`, constrained public fetch gates).
- Deterministic diagnostics include profile context and fix guidance.

### M39-S1 tracking (live status)

- [x] Profile capability fence diagnostics implemented in semantic layer.
- [x] Browser profile now also blocks server-only capability types/constructors:
  - type positions (`Ctx`, `DbCap`, `FsCap`, `NetCap`, `InternalNetCap`, `SecretsCap`)
  - constructor calls (`Ctx()`, `DbCap()`, `FsCap()`, `NetCap()`, `InternalNetCap()`, `SecretsCap()`)
- [x] CLI coverage added for allow/deny profile behavior:
  - `check_fails_when_browser_profile_uses_server_only_intrinsics`
  - `check_succeeds_when_browser_profile_uses_allowed_public_net_intrinsics`
  - `check_fails_when_browser_profile_uses_server_capability_types_and_constructors`
- [x] Book chapter documenting S1 implementation added:
  - `docs/book/891-m39-browser-profile-capability-fence-diagnostics.md`
  - `docs/book/892-m39-browser-profile-capability-type-and-constructor-fences.md`

### M39-S2 composition contract analyzer acceptance criteria (deferred by scope reset)

- Analyzer enforces promotion-ready architecture contract:
  - target-agnostic domain module
  - repository interface
  - target-specific adapters (`LocalRepo` / `ServerRepo`).
- Analyzer rejects direct domain coupling to browser/server-only adapters.
- Analyzer verifies repository method parity between browser/server adapters.

### M39-S2 tracking (live status)

- [ ] Deferred until post-alpha scope window unless promotion pipeline is blocked by missing analyzer guarantees.
- [ ] Composition contract analyzer implemented.
- [ ] Fixture coverage added for pass/fail composition graphs.
- [ ] Book chapter documenting S2 implementation added.

### M39-S2A multi-file module system acceptance criteria (alpha-priority replacement slice)

- Project code can be split across multiple `.ut` files with deterministic module resolution from project root.
- Module references remain explicit (`use`/imports) with stable path rules that survive folder/file renames via deterministic diagnostics.
- Resolver emits deterministic errors for:
  - missing modules/files,
  - ambiguous module targets,
  - cyclic module dependencies.
- Single-file projects remain valid with unchanged behavior.

### M39-S2A tracking (live status)

- [x] Module graph resolver implemented for project-local modules.
- [x] Deterministic cycle detection + diagnostics implemented.
- [x] CLI/integration fixture coverage added for multi-file pass/fail cases.
- [x] Book chapter documenting scope-reset and multi-file priority added (`docs/book/893-m39-alpha-scope-reset-and-multi-file-priority.md`).
- [x] Book chapter documenting resolver implementation added (`docs/book/896-m39-multi-file-module-resolver-implementation.md`).
- [x] Build/run integration coverage locked for multi-file modules (`docs/book/900-m39-multi-file-build-and-run-pipeline-lock.md`).
- [x] `sec4 test` now resolves module graphs per test entry (from `tests/` root), supports helper modules without standalone execution, fails deterministically when no runnable `fn main` entries exist, emits close-match suggestions for missing module imports, and reports deterministic discovery/skip counts (`docs/book/911-m39-test-command-module-graph-entrypoint-filtering.md`).

### M39-S2B LASM async backend bootstrap acceptance criteria (parallel lane)

- New backend target surface is introduced without breaking existing C target flows.
- Runtime execution model supports evented/concurrent request handling (not one-thread-per-request).
- Deterministic envelope parity is preserved for core response/error contracts.
- Benchmark smoke includes a LASM lane for baseline concurrency sanity.

### M39-S2B tracking (live status)

- [x] Backend target skeleton + compile-path wiring added for LASM bootstrap.
- [x] Async runtime core loop prototype added (reactor/scheduler baseline).
- [x] Minimal HTTP request/response path running on LASM runtime baseline.
- [x] Book chapter documenting C-runtime role and LASM transition plan added (`docs/book/894-m39-c-runtime-role-and-lasm-async-transition-plan.md`).
- [x] sec4 capacity-probe benchmark tooling added for early scale evidence (`benchmark-suite/scripts/run_sec4_capacity_probe.sh`, chapter `docs/book/895-m39-sec4-capacity-probe-tooling.md`).
- [x] LASM deterministic entrypoint contract metadata added for runtime bootstrap (`docs/book/897-m39-lasm-entrypoint-contract-baseline.md`).
- [x] LASM async scheduler core loop baseline added (`docs/book/898-m39-lasm-async-runtime-core-loop-baseline.md`).
- [x] LASM async scheduler now supports explicit task cancellation (`cancel_task`) + live task counting baseline for timeout/abort evolution (`docs/book/915-m39-lasm-runtime-task-cancellation-baseline.md`).
- [x] LASM in-memory HTTP baseline added on scheduler runtime (`docs/book/899-m39-lasm-http-runtime-baseline.md`).
- [x] CLI `lasm-smoke` command executes compiled entrypoint through LASM runtime baseline (`docs/book/901-m39-cli-lasm-smoke-command.md`).
- [x] CLI `lasm-smoke` route/response extraction now resolves through helper call graphs (route registration + `res.text/res.html/res.json/res.ok/res.okMeta` helpers), and LASM run response extraction now applies latest-response-write precedence plus caller-argument parameter binding for helper response wrappers (`docs/book/903-m39-lasm-smoke-helper-call-graph-route-resolution.md`, `docs/book/982-m39-lasm-run-latest-response-write-parity.md`, `docs/book/983-m39-lasm-run-response-helper-parameter-binding.md`).
- [x] CLI `lasm-smoke` now supports machine-readable `--format json` summaries alongside text output (`docs/book/904-m39-lasm-smoke-json-summary-output.md`).
- [x] LASM HTTP runtime now supports parameterized route matching (`/users/:id`) with exact-route-first resolution, captured path-parameter propagation into exchanges, query/fragment-insensitive request matching, and deterministic `HEAD -> GET` fallback when HEAD is not explicitly registered (`docs/book/905-m39-lasm-http-runtime-parameterized-route-matching.md`, `docs/book/907-m39-lasm-http-runtime-request-path-normalization.md`, `docs/book/908-m39-lasm-http-runtime-head-fallback.md`).
- [x] LASM HTTP runtime pattern route overrides now follow deterministic latest-registration-wins semantics (aligned with exact-route overwrite behavior), and `run --backend lasm` route-plan consolidation now preserves the same latest-registration-wins parity for duplicate method/path registrations (`docs/book/910-m39-lasm-http-runtime-pattern-override-order.md`, `docs/book/981-m39-lasm-run-duplicate-route-latest-wins-parity.md`).
- [x] LASM HTTP runtime now supports deterministic max in-flight concurrency gating with FIFO pending-request queue drain semantics (`set_max_in_flight` / `clear_max_in_flight`) (`docs/book/912-m39-lasm-http-runtime-max-in-flight-queue.md`).
- [x] LASM HTTP runtime now supports deterministic max pending queue backpressure (`set_max_pending` / `clear_max_pending`) with immediate queue-full `503` overflow responses (`docs/book/914-m39-lasm-http-runtime-max-pending-backpressure.md`).
- [x] LASM HTTP runtime now supports deterministic per-request timeout enforcement (`set_max_request_duration_ms`) with timeout `504` response mapping, live-task cancellation for timed-out handlers, and timeout-age evaluation from original request submit time (including pending queue wait) (`docs/book/916-m39-lasm-http-runtime-request-timeout-enforcement.md`, `docs/book/918-m39-lasm-http-runtime-timeout-from-submit-time.md`).
- [x] CLI `lasm-smoke` now supports `--request-path` and emits captured `pathParams` in text/json summaries (`docs/book/906-m39-lasm-smoke-request-path-and-path-param-summary.md`).
- [x] CLI `lasm-smoke` now supports `--max-in-flight`, `--max-pending`, and `--max-request-ms` for exercising LASM runtime queue/backpressure/timeout controls from command line, with deterministic zero-value rejection, explicit `maxInFlight`/`maxPending`/`maxRequestMs` + `statusCounts` summary visibility, deterministic latency metrics (`durationMs.{min,max,avg}` + `firstDurationMs`), and `--fail-on-errors` promotion of observed runtime error responses into deterministic smoke-command failure; `--runtime-script` now enables deterministic scripted action sequences (`yield`/`sleep:<ms>`/`complete:<code>`) for runtime behavior probes (`docs/book/913-m39-lasm-smoke-max-in-flight-flag.md`, `docs/book/914-m39-lasm-http-runtime-max-pending-backpressure.md`, `docs/book/917-m39-lasm-smoke-runtime-script-and-timeout-flag.md`, `docs/book/919-m39-lasm-smoke-duration-metrics.md`).
- [x] CLI `lasm-smoke` now extracts static response headers from handler call graphs (`res.setHeader`) including typed header gate wrappers and local let-bindings, and LASM run header extraction now binds helper parameters from caller literals so wrapper helpers emit deterministic headers (`docs/book/909-m39-lasm-smoke-response-header-extraction.md`, `docs/book/984-m39-lasm-run-response-header-helper-parameter-binding.md`).
- [x] LASM run backend now materializes benchmark contract JSON bodies from request payloads/path params (`DecodeResponse`, `CreateUserResponse`, `UserResponse`) with shared in-process user state for deterministic read-after-write behavior in the LASM benchmark lane (`docs/book/922-m39-lasm-run-benchmark-contract-response-materialization.md`).
- [x] LASM benchmark response materialization now enforces deterministic JSON/UUID validation (`400` `JSON.INVALID_SYNTAX` / `VALIDATION.UUID_INVALID`) for JSON-expected decode/create paths while preserving compatibility fallback for legacy empty-body fixture requests (`docs/book/923-m39-lasm-run-benchmark-response-validation-hardening.md`).
- [x] LASM decode/create materialization now enforces full benchmark user payload validation parity (`email`, `age`, `tags`, `address.zip`, `meta.flags`) with deterministic `VALIDATION.INVALID` responses for malformed request bodies (`docs/book/924-m39-lasm-benchmark-payload-validation-parity.md`).
- [x] Benchmark default implementation sets now include `sec4-lasm` across preflight/comparison/step/full-suite orchestration and Makefile defaults, so LASM benchmark coverage is exercised by default (`docs/book/925-m39-benchmark-default-impl-set-includes-sec4-lasm.md`).
- [x] LASM dynamic validation errors now include benchmark-style envelope metadata parity (`traceId` + `timeMs`) using one request-scoped trace id shared across header/body paths (`docs/book/926-m39-lasm-trace-and-error-envelope-parity.md`).
- [x] LASM decode/create materialization now rejects non-JSON content-type payloads deterministically (`400 HTTP.BAD_REQUEST`, `content-type must be application/json`) while preserving empty-body fallback compatibility (`docs/book/927-m39-lasm-json-content-type-enforcement.md`).
- [x] LASM non-oneshot overload path now returns deterministic JSON error envelopes (`HTTP.SERVICE_UNAVAILABLE`) with trace metadata instead of plain-text overload bodies (`docs/book/928-m39-lasm-overload-json-error-envelope.md`).
- [x] LASM response writer now enforces HTTP HEAD body omission across runtime and overload paths while preserving deterministic overload JSON headers (`docs/book/929-m39-lasm-head-body-omission-parity.md`).
- [x] LASM request-line parsing now enforces explicit HTTP version tokens, enforces `max_header_bytes` on the request line before parsing headers, and emits deterministic unsupported-version rejection (`505 HTTP Version Not Supported`) for non-HTTP/1.0/1.1 requests (`docs/book/930-m39-lasm-request-line-version-hardening.md`).
- [x] LASM worker runtime now supports HTTP keep-alive request reuse on persistent sockets (with HTTP/1.0 + `Connection: close` deterministic close behavior) while preserving existing queue/backpressure controls (`docs/book/931-m39-lasm-worker-keep-alive-support.md`).
- [x] LASM worker runtime now reuses a persistent buffered request reader per connection (supporting pipelined request parsing), interprets comma-delimited `Connection` tokens deterministically, and rejects unsupported `Transfer-Encoding` with deterministic `501 Not Implemented` responses (`docs/book/932-m39-lasm-persistent-reader-and-transfer-encoding-rejection.md`).
- [x] LASM chunked request parsing now validates chunk-extension syntax (token name + token/quoted value forms) and rejects malformed extensions deterministically (`400 Bad Request`, `invalid transfer-encoding chunk extension`) while preserving chunked success behavior (`docs/book/985-m39-lasm-chunk-extension-validation.md`).
- [x] LASM chunked trailer parsing now rejects forbidden framing/routing trailer headers (`Host`, `Content-Length`, `Transfer-Encoding`) with deterministic `400 Bad Request` diagnostics (`invalid chunk trailer: forbidden trailer header`) while preserving valid trailer acceptance (`docs/book/986-m39-lasm-chunk-trailer-forbidden-header-validation.md`).
- [x] LASM chunked trailer parsing now merges accepted trailer headers into request headers (case-insensitive merge semantics) so handlers can consume trailer values via `req.header(...)` (`docs/book/987-m39-lasm-chunk-trailer-header-merge-for-req-header.md`).
- [x] LASM chunked trailer parsing now enforces `max_header_bytes` against trailer-section bytes, rejecting oversized trailers deterministically with `431 Request Header Fields Too Large`; request-head header-limit diagnostics now also interpolate the configured byte value deterministically (`docs/book/988-m39-lasm-chunk-trailer-header-limit-enforcement.md`).
- [x] LASM chunked parser now also enforces `max_header_bytes` on chunk-size lines (including long extension lines), returning deterministic `431 Request Header Fields Too Large` on oversized framing metadata (`docs/book/989-m39-lasm-chunk-size-line-header-limit-enforcement.md`).
- [x] LASM request-head parser now rejects incomplete header sections (EOF before blank-line terminator) with deterministic `400 Bad Request` diagnostics (`incomplete request while reading header line`) to prevent truncated-header acceptance (`docs/book/990-m39-lasm-incomplete-header-section-rejection.md`).
- [x] LASM chunked-body parser now rejects incomplete chunk-size and chunk-trailer lines (EOF before newline terminator) with deterministic `400 Bad Request` diagnostics (`incomplete request while reading chunk size` / `incomplete request while reading chunk trailer`) to prevent truncated chunk framing acceptance (`docs/book/991-m39-lasm-incomplete-chunk-line-rejection.md`).
- [x] LASM request-head parser now rejects request/header lines missing newline terminators (EOF-truncated lines) with deterministic stage-specific `400 Bad Request` diagnostics (`incomplete request while reading request line` / `incomplete request while reading header line`) to prevent accepting partially framed start-lines and headers (`docs/book/992-m39-lasm-incomplete-request-and-header-line-terminator-rejection.md`).
- [x] LASM connection keep-alive request accounting now increments on runtime step-budget failure responses, so `max_keep_alive_requests` limits remain enforced even when requests terminate via deterministic runtime-step `500` paths (`docs/book/993-m39-lasm-keep-alive-accounting-on-runtime-step-failure.md`).
- [x] LASM absolute-form request-target parsing now accepts case-insensitive `http`/`https` schemes (`HTTP://`, `HTTPS://`) while preserving host-authority parity enforcement and deterministic mismatch diagnostics (`docs/book/994-m39-lasm-absolute-form-scheme-case-insensitive-parsing.md`).
- [x] LASM runtime now resets transient execution state after step-budget exhaustion responses so unfinished tasks/pending queues/ready-response carryover are cleared before the next request on the same connection (`docs/book/995-m39-lasm-runtime-reset-after-step-budget-exhaustion.md`).
- [x] LASM absolute-form normalization now preserves query-only targets without explicit path (`http://host?x=1` -> `/?x=1`) so `req.query(...)` extraction remains available for absolute-form requests (`docs/book/996-m39-lasm-absolute-form-query-only-normalization.md`).
- [x] LASM step-budget exhaustion responses now emit deterministic JSON error envelopes (`LASM.STEP_BUDGET_EXCEEDED`, `kind=internal`) with shared trace metadata parity instead of plain-text-only payloads (`docs/book/997-m39-lasm-step-budget-json-error-envelope.md`).
- [x] LASM parser now rejects request lines with leading whitespace before method tokens, returning deterministic `400 Bad Request` diagnostics (`invalid request line: leading whitespace is not allowed`) with parser-envelope parity (`docs/book/998-m39-lasm-request-line-leading-whitespace-rejection.md`).
- [x] LASM parser now rejects request lines with trailing whitespace after the HTTP version token, returning deterministic `400 Bad Request` diagnostics (`invalid request line: trailing whitespace is not allowed`) with parser-envelope parity (`docs/book/999-m39-lasm-request-line-trailing-whitespace-rejection.md`).
- [x] LASM parser now rejects tab-separated request-line tokens, returning deterministic `400 Bad Request` diagnostics (`invalid request line: tab separators are not allowed`) with parser-envelope parity (`docs/book/1000-m39-lasm-request-line-tab-separator-rejection.md`).
- [x] LASM parser now enforces single-space request-line token separation (rejecting double-space separators) with deterministic `400 Bad Request` diagnostics (`invalid request line: expected single-space separators`) and parser-envelope parity (`docs/book/1001-m39-lasm-request-line-single-space-separator-enforcement.md`).
- [x] LASM dynamic benchmark-user state now supports optional disk persistence through `SEC4_RT_LASM_DB_BASE` (users store file load on startup + write-through on create), enabling user read-after-restart behavior for `CreateUserResponse`/`UserResponse` flows (`docs/book/1002-m39-lasm-dynamic-user-store-persistence.md`).
- [x] LASM route extraction now materializes `sql.q` + `db.exec`/`db.execTx` intrinsic call plans into internal runtime markers, and LASM run executes those DB operations through deterministic tx-handle registration plus persisted `records.log` writes (instead of schema-name-only DB write bridges for these paths) (`docs/book/1004-m39-lasm-db-intrinsic-execution-baseline.md`).
- [x] `examples/lasm-alpha-full` DB write routes now execute real intrinsic flows (`DbCap` + `sql.q` + `db.exec`/`db.execTx`) so the canonical LASM operator sample demonstrates intrinsic-backed `records.log` writes end-to-end (`docs/book/1005-m39-lasm-alpha-full-db-intrinsic-routes.md`).
- [x] Removed LASM schema-switch DB write fallback arms (`DbExecResponse` / `DbExecTxResponse`) from response materialization; DB writes now require intrinsic extraction markers and no longer execute through response-schema-only branches (`docs/book/1006-m39-lasm-remove-schema-fallback-db-write-arms.md`).
- [x] Added `schema.row(...)` row-schema bridge intrinsic across semantic typing + C backend/runtime, moved LASM query-one routes/examples to real `db.queryOne` intrinsic flows, and removed `DbQueryOneResponse` schema-switch fallback materialization branch (`docs/book/1007-m39-lasm-db-query-one-intrinsic-parity-and-schema-row-bridge.md`).
- [x] `sec4 run` now defaults to LASM backend (`--backend lasm` implicit), while C backend remains explicit via `--backend c`; command coverage now locks help/default contract, default-backend acceptance of LASM-only flags, and C-only guard behavior under explicit C selection (`docs/book/1008-m39-lasm-default-run-backend-and-explicit-c-fallback.md`).
- [x] Benchmark profile summaries now capture service RSS memory samples (`memory.rssKb`) and propagate `rssKb` through compare-report/matrix leader rows plus published benchmark markdown output, establishing throughput/latency/memory baseline visibility for LASM stability hardening (`docs/book/1009-m39-benchmark-rss-memory-baseline-visibility.md`).
- [x] Benchmark regression threshold guard now supports RSS-based checks (`--max-rss-kb`) plus baseline RSS regression limits (`baselineRssKb`, `maxRssRegressionPct`) alongside existing p99/coverage checks, with deterministic failure modes for missing/invalid leader RSS data when RSS guards are active (`docs/book/1010-m39-benchmark-rss-regression-threshold-guard.md`).
- [x] Trend-note rendering/import flow now surfaces leader RSS (`rssKb`) as a first-class table column, so comparative trend notes keep throughput/latency/memory visibility aligned with matrix artifacts (`docs/book/1011-m39-trend-note-rss-column-visibility.md`).
- [x] Scheduled benchmark-trend CI workflow now applies explicit RSS absolute guards (`--max-rss-kb`) for ping/decode threshold steps, and workflow contract tests lock presence of RSS threshold flags to prevent drift (`docs/book/1012-m39-benchmark-trend-ci-rss-threshold-enforcement.md`).
- [x] Benchmark evidence-quality checker now treats missing/invalid leader RSS as a first-class warning signal (with `--fail-on-warning` escalation), and compare-row identity checks include `rssKb` parity to prevent hidden memory-signal drift across leader/compared rows (`docs/book/1013-m39-benchmark-evidence-quality-rss-warning-enforcement.md`).
- [x] Trend-note baseline guard evaluation now applies optional RSS regression checks when baseline files provide `baselineRssKb`/`maxRssRegressionPct`, so trend pass/fail status can reflect memory drift alongside p99/coverage in the same baseline verdict (`docs/book/1014-m39-trend-note-baseline-rss-guard-evaluation.md`).
- [x] LASM DB intrinsic persistence now supports adapter selection via `SEC4_RT_LASM_DB_ADAPTER` (`records.log` default, `sqlite` optional), with SQLite-backed load/persist parity for `db.exec`/`db.execTx`/`db.queryOne` while preserving default file-adapter behavior (`docs/book/1015-m39-lasm-db-sqlite-adapter-baseline.md`).
- [x] LASM sqlite DB adapter now runs real runtime intrinsic execution for `db.exec`, `db.execTx`, and `db.queryOne` against `records.sqlite3` (including parameter binding and typed row-value JSON materialization) instead of metadata-only query fallback responses (`docs/book/1077-m39-lasm-sqlite-runtime-exec-query-one-materialization.md`).
- [x] LASM sqlite runtime now enforces deterministic sql-parameter lower-bound validation and queryOne SQL-shape guards (`SELECT`/`WITH`/`VALUES`/`TABLE` only), including trailing-semicolon normalization for queryOne wrappers, so invalid sqlite queryOne flows return explicit deterministic runtime diagnostics (`docs/book/1078-m39-lasm-sqlite-query-one-shape-and-arity-guard.md`).
- [x] LASM DB intrinsic runtime error mapping now classifies known query-validation failures (`requires at least`, queryOne SQL-shape/non-empty, single-statement requirement) as deterministic `400 ..._INVALID` envelopes for sqlite/postgres exec/queryOne paths instead of generic `500 ..._FAILED` responses, while keeping adapter-config errors on deterministic internal classification (`docs/book/1079-m39-lasm-db-runtime-validation-error-classification.md`).
- [x] LASM sqlite runtime now rejects parameterized multi-statement SQL in `db.exec`/`db.execTx`/`db.queryOne` with deterministic validation diagnostics (`sqlite parameterized execution requires a single SQL statement`), preserving non-parameterized multi-statement compatibility while blocking ambiguous parameterized execution shapes (`docs/book/1080-m39-lasm-sqlite-parameterized-single-statement-guard.md`).
- [x] LASM postgres runtime now pre-validates parameterized multi-statement SQL before adapter execution in `db.exec`/`db.execTx`/`db.queryOne`, returning deterministic single-statement validation diagnostics (`postgres parameterized execution requires a single SQL statement`) without relying on adapter-native parse failures (`docs/book/1081-m39-lasm-postgres-parameterized-single-statement-precheck.md`).
- [x] LASM `DbListRecordsResponse` now exposes the active DB adapter label (`records.log` or `sqlite`) so operator introspection confirms which persistence backend is active during runtime validation flows (`docs/book/1016-m39-lasm-db-list-response-adapter-visibility.md`).
- [x] Benchmark-trend workflow contract checks now enforce endpoint-set coherence across benchmark run scope, threshold checks, and trend-note rendering, and require one `--max-rss-kb` guard per threshold endpoint invocation to prevent silent CI drift (`docs/book/1017-m39-benchmark-trend-workflow-endpoint-contract-coherence.md`).
- [x] `sec4 run` now supports explicit `--db-adapter <records-log|sqlite>` selection for LASM mode (single-instance and cluster-worker paths), with deterministic C-backend guard diagnostics for lasm-only usage; SQLite command coverage now exercises adapter selection through CLI flag surface while preserving `SEC4_RT_LASM_DB_ADAPTER` fallback compatibility (`docs/book/1019-m39-run-db-adapter-flag.md`).
- [x] `sec4 run` now supports explicit `--db-postgres-dsn <dsn>` for LASM Postgres adapter mode (single-instance and cluster-worker forwarding), with deterministic LASM-only guard + empty-value validation diagnostics and fallback compatibility with `SEC4_RT_LASM_DB_POSTGRES_DSN` when flag is omitted (`docs/book/1082-m39-run-db-postgres-dsn-flag.md`).
- [x] `sec4 run` now supports `--db-postgres-dsn-file <path>` for LASM Postgres adapter mode, with deterministic LASM-only guard diagnostics, explicit conflict guard against simultaneous `--db-postgres-dsn`, and non-empty file-content validation before runtime start (`docs/book/1084-m39-run-db-postgres-dsn-file-flag.md`).
- [x] LASM cluster worker Postgres DSN forwarding now uses env propagation (`SEC4_RT_LASM_DB_POSTGRES_DSN`) instead of CLI argument forwarding, reducing DSN exposure through worker command-line arguments while preserving explicit `--db-postgres-dsn` operator behavior at the parent run entrypoint (`docs/book/1083-m39-lasm-cluster-postgres-dsn-env-forwarding.md`).
- [x] LASM Postgres DSN resolver now supports `SEC4_RT_LASM_DB_POSTGRES_DSN_FILE` env fallback (when explicit DSN flags and `SEC4_RT_LASM_DB_POSTGRES_DSN` are unset), with deterministic empty-path/content diagnostics and command coverage for invalid file-content env cases (`docs/book/1085-m39-lasm-postgres-dsn-file-env-fallback.md`).
- [x] `sec4 run` now resolves explicit Postgres DSN sources (`--db-postgres-dsn` / `--db-postgres-dsn-file`) once in `cmd_run` before project validation and forwards only the resolved DSN downstream, eliminating duplicate file reads and keeping deterministic invalid-config exit semantics (`docs/book/1086-m39-run-postgres-dsn-single-resolution.md`).
- [x] `sec4 run` now auto-selects Postgres adapter when explicit Postgres DSN flags are provided without `--db-adapter`, and rejects DSN/DSN-file flags when a non-Postgres adapter is explicitly set, preventing silent DSN-flag ignore paths (`docs/book/1088-m39-run-postgres-dsn-adapter-coherence.md`).
- [x] LASM sqlite/postgres `db.queryOne` success paths now append persisted runtime records (`op=queryOne`) with deterministic IDs and adapter record-list visibility, instead of reusing prior `exec`/`execTx` metadata in queryOne responses (`docs/book/1089-m39-lasm-query-one-record-persistence-parity.md`).
- [x] LASM `db.queryOne` responses now expose structured `rowObject` payloads for sqlite/postgres adapters (while preserving legacy serialized `row` string and keeping records-log fallback on `rowObject: null`), improving runtime DB-client payload usability without breaking compatibility (`docs/book/1090-m39-lasm-query-one-row-object-response.md`).
- [x] LASM records-log fallback now also appends deterministic `queryOne` runtime records on successful queryOne matches, bringing queryOne record-history parity across records-log/sqlite/postgres adapters (`docs/book/1091-m39-lasm-query-one-records-log-parity.md`).
- [x] LASM runtime record persistence now uses incremental per-operation append/upsert writes across adapters (sqlite/postgres append-upsert, records-log append), each with deterministic full-sync fallback on append failure, instead of per-request full-history rewrite, reducing metadata write-path overhead while preserving deterministic persistence semantics (`docs/book/1117-m39-lasm-db-incremental-record-persistence.md`).
- [x] LASM sqlite adapter append persistence now reuses one cached sqlite connection per runtime instance (initialized at state bootstrap) instead of reopening sqlite on each append write, while preserving deterministic connection-drop + full-sync fallback on append failure (`docs/book/1118-m39-lasm-sqlite-connection-reuse-for-append-persistence.md`).
- [x] LASM tx handles are now runtime-ephemeral: startup no longer rehydrates tx-handle bindings from persisted record history, so stale `db.tx` handles are rejected after restart with deterministic `DB.EXEC_TX_HANDLE_INVALID` semantics (`docs/book/1092-m39-lasm-ephemeral-tx-handle-runtime-scope.md`).
- [x] LASM sqlite/postgres `db.exec` and `db.execTx` responses now include real `affectedRows` metadata from runtime execution paths (with deterministic `0` fallback for records-log adapter paths), improving DB-client observability for write execution outcomes (`docs/book/1093-m39-lasm-exec-affected-rows-runtime-metadata.md`).
- [x] LASM DB runtime error classification now maps sqlite/postgres unique-constraint failures to deterministic conflict envelopes (`409` + operation-specific `DB.*_CONFLICT` codes) instead of generic `500` failure codes (`docs/book/1094-m39-lasm-db-unique-conflict-error-classification.md`).
- [x] LASM DB runtime error classification now also maps sqlite/postgres not-null/check/input-syntax constraint failures to deterministic validation envelopes (`400` + operation-specific `DB.*_INVALID` codes), reducing generic runtime-failure ambiguity for common write-shape errors (`docs/book/1095-m39-lasm-db-constraint-validation-error-classification.md`).
- [x] LASM `db.execTx` runtime now validates existing tx-handle bindings before adapter execution, preventing SQL side effects from executing on invalid tx handles; sqlite command coverage now verifies invalid tx-handle flows return deterministic `DB.EXEC_TX_HANDLE_INVALID` without mutating persisted DB state (`docs/book/1087-m39-lasm-exec-tx-invalid-handle-preexecution-guard.md`).
- [x] LASM DB adapters now include a real `postgres` runtime client path (`--db-adapter postgres` / `SEC4_RT_LASM_DB_ADAPTER=postgres`) with deterministic DSN contract (`SEC4_RT_LASM_DB_POSTGRES_DSN`), metadata persistence in `sec4_lasm_db_records`, real `db.exec`/`db.execTx` execution, and real `db.queryOne` first-row materialization (`docs/book/1062-m39-lasm-db-postgres-adapter-runtime-client.md`).
- [x] LASM postgres adapter runtime now reuses one in-process Postgres client per worker/process (bootstrap connect + schema + metadata load, then shared execution/persist path) instead of reconnecting on each DB intrinsic call (`docs/book/1063-m39-lasm-postgres-client-reuse-in-runtime-state.md`).
- [x] LASM postgres adapter runtime now retries once with deterministic reconnect on closed Postgres connections for `db.exec`, `db.execTx`, `db.queryOne`, and metadata persistence sync paths (`docs/book/1064-m39-lasm-postgres-client-auto-reconnect.md`).
- [x] LASM postgres `db.queryOne` row materialization now preserves typed JSON values (`number`, `bool`, parsed JSON objects/arrays, and null) instead of string-only cells, improving DB client response fidelity for runtime consumers (`docs/book/1065-m39-lasm-postgres-query-one-typed-row-values.md`).
- [x] LASM postgres DB adapter now applies deterministic `$N` parameter rendering from `sql.q(..., params)` for `db.exec`, `db.execTx`, and `db.queryOne` (including JSON-array params and SQL-literal escaping), enabling real parameterized query execution paths in LASM runtime routes (`docs/book/1066-m39-lasm-postgres-parameter-rendering-for-db-intrinsics.md`).
- [x] LASM postgres DB intrinsic execution now validates SQL placeholder arity before execution (`$N` vs provided params) and returns deterministic runtime diagnostics on mismatch, preventing opaque adapter errors for under-specified parameter sets (`docs/book/1067-m39-lasm-postgres-placeholder-arity-validation.md`).
- [x] LASM postgres placeholder rendering now ignores `$N` tokens inside SQL string literals (and placeholder-arity scanning follows the same literal-awareness), preventing accidental replacement of literal text while keeping runtime parameter binding deterministic (`docs/book/1068-m39-lasm-postgres-literal-aware-placeholder-rendering.md`).
- [x] LASM postgres placeholder scanning/rendering now also ignores `$N` tokens inside dollar-quoted SQL literals (`$$...$$` / `$tag$...$tag$`) so parameter binding does not mutate literal dollar-quoted content while preserving deterministic arity checks (`docs/book/1070-m39-lasm-postgres-dollar-quoted-placeholder-preservation.md`).
- [x] LASM postgres `db.exec` / `db.execTx` now run parameterized queries through real prepared execution (`client.execute` / `transaction.execute`) when placeholders/params are present, with deterministic single-statement guidance when parameterized multi-statement SQL is provided (`docs/book/1071-m39-lasm-postgres-prepared-exec-and-exec-tx.md`).
- [x] LASM postgres `db.queryOne` now also runs through prepared parameter execution (no SQL literal substitution), wrapping result materialization with `row_to_json(... )::text` for deterministic JSON row payload decode, and preserving placeholder-arity validation + deterministic single-statement guidance (`docs/book/1072-m39-lasm-postgres-prepared-query-one-row-to-json.md`).
- [x] LASM postgres command integration flow now exercises prepared `db.execTx` end-to-end (request -> runtime -> persisted record metadata), locking deterministic `execTx` record sequencing/tx-handle visibility in the canonical Postgres runtime test path (`docs/book/1073-m39-lasm-postgres-exec-tx-prepared-flow-coverage.md`).
- [x] LASM postgres placeholder-arity scanning is now comment-aware (`-- ...` and `/* ... */`, including nested block comments), so placeholder-like tokens in SQL comments do not trigger false arity failures (`docs/book/1074-m39-lasm-postgres-comment-aware-placeholder-arity.md`).
- [x] LASM postgres `db.queryOne` now enforces deterministic SQL-shape guarding (`SELECT`/`WITH`/`VALUES`/`TABLE` only) before execution, returning explicit runtime diagnostics for non-row-returning statements in queryOne paths (`docs/book/1075-m39-lasm-postgres-query-one-select-shape-guard.md`).
- [x] LASM postgres `db.queryOne` now normalizes trailing semicolons before subquery wrapping, so valid row-returning SQL copied with terminal `;` executes deterministically instead of failing wrapper parse (`docs/book/1076-m39-lasm-postgres-query-one-trailing-semicolon-normalization.md`).
- [x] `sec4 run` runtime-flag script contracts now lock `--db-adapter` wiring invariants (run field presence, LASM-only guard diagnostic, LASM dynamic-state adapter bridge, and cluster worker forwarding bridge) with deterministic guard-fixture drift coverage in naming-lock script tests (`docs/book/1020-m39-run-db-adapter-runtime-flag-contract-lock.md`).
- [x] Added canonical LASM DB adapter operator smoke script (`scripts/smoke-sec4-run-lasm-db-adapter.sh`) that executes intrinsic-backed `/db/*` flows under explicit `--db-adapter` selection and validates adapter-specific persistence outputs (`records.log` vs `records.sqlite3`), plus script token-contract + guard coverage (`docs/book/1021-m39-lasm-db-adapter-operator-smoke-script.md`).
- [x] Naming-lock contract suite now executes LASM DB adapter smoke script contract + guard checks so adapter smoke script drift fails CI through the standard script-contract path (`docs/book/1022-m39-naming-lock-lasm-db-adapter-smoke-contract-coverage.md`).
- [x] Runtime-smoke CI workflow now executes LASM DB adapter smoke lanes (`records-log` and `sqlite`) in addition to hello-api lanes, with runtime-smoke workflow contract + guard coverage updated to lock both adapter-step tokens and preserve bundle-check enforcement (`docs/book/1023-m39-runtime-smoke-workflow-lasm-db-adapter-lanes.md`).
- [x] Milestone closure gate `M16-C` now enforces runtime-smoke DB adapter lane tokens (records-log + sqlite) alongside hello-api lanes, and closure fixture coverage was updated to keep strict gate evaluation deterministic (`docs/book/1030-m39-closure-gate-runtime-smoke-db-adapter-lane-enforcement.md`).
- [x] Runtime-smoke bundle validation now enforces LASM DB adapter artifact branch contracts (`lasm-db-records-log`, `lasm-db-sqlite`) via a dedicated db-adapter checker, and closure gate `M16-D` now requires naming-lock coverage for that checker (`docs/book/1033-m39-runtime-smoke-bundle-db-adapter-artifact-validation.md`).
- [x] LASM request parser now rejects conflicting duplicate `Content-Length` headers with deterministic `400 Bad Request` diagnostics (`conflicting content-length headers`) to harden request framing behavior (`docs/book/933-m39-lasm-conflicting-content-length-rejection.md`).
- [x] LASM parser now enforces HTTP/1.1 `Host` header presence (non-empty) with deterministic `400 Bad Request` diagnostics (`missing host header`) for malformed inbound requests (`docs/book/934-m39-lasm-http11-host-header-enforcement.md`).
- [x] LASM parser now normalizes absolute-form request targets (`http://host/path`) for route matching, rejects invalid request-target forms deterministically, and preserves existing query stripping semantics on normalized paths (`docs/book/935-m39-lasm-request-target-normalization.md`).
- [x] LASM parser now rejects conflicting duplicate `Host` headers deterministically (`400 Bad Request`, `conflicting host headers`) to prevent ambiguous HTTP/1.1 host resolution (`docs/book/936-m39-lasm-conflicting-host-header-rejection.md`).
- [x] LASM parser now rejects unsupported `Expect` request headers deterministically (`417 Expectation Failed`, `expect header is not supported`) to keep request-body negotiation semantics explicit (`docs/book/937-m39-lasm-expect-header-rejection.md`).
- [x] LASM parser now enforces absolute-form authority and `Host` header parity (`400 Bad Request` on mismatch) to prevent ambiguous host resolution when proxy-style request targets are used (`docs/book/938-m39-lasm-absolute-form-host-parity-enforcement.md`).
- [x] LASM parser now validates and normalizes host/authority syntax (rejecting malformed host lists/userinfo/invalid ports) and treats default absolute-form authority ports as parity-equivalent with `Host` (`http:80`, `https:443`) for deterministic routing-safe host checks (`docs/book/939-m39-lasm-authority-normalization-and-host-validation.md`).
- [x] LASM parser now rejects invalid HTTP method/header-name tokens, forbids whitespace around header names, and rejects request-target URI fragments with deterministic `400` diagnostics before route execution (`docs/book/940-m39-lasm-http-token-and-fragment-hardening.md`).
- [x] LASM parser now maps request-read failures into deterministic error classes (timeout, invalid encoding, incomplete request, generic read failure) to avoid host-dependent OS error text variance (`docs/book/941-m39-lasm-deterministic-read-error-mapping.md`).
- [x] LASM parser failures now emit deterministic JSON error envelopes (status-mapped code/kind + trace metadata) for contract parity with other runtime failure paths (`docs/book/942-m39-lasm-parser-error-envelope-parity.md`).
- [x] LASM parser failure responses now suppress default CORS headers, keeping malformed-request failure paths deterministic and non-permissive while preserving trace + structured error envelope metadata (`docs/book/943-m39-lasm-parser-error-cors-suppression.md`).
- [x] LASM overload-path request handling now reuses strict request-head parsing so malformed overflow requests return deterministic parser envelopes (while no-data timeout probes still fall back to deterministic busy `503`) (`docs/book/944-m39-lasm-overflow-parser-hardening.md`).
- [x] LASM shared request-head parsing now rejects control characters in header values with deterministic `400` diagnostics (`invalid header line: invalid header value character`), keeping normal and overload parser behavior aligned (`docs/book/945-m39-lasm-header-value-character-validation.md`).
- [x] LASM overload probing now uses bounded short read timeouts to avoid long accept-loop stalls under queue saturation, and `res.text(...)` runtime materialization now supports request-derived placeholders from `req.pathParam`/`req.header` (including `validate.nonEmpty(...)` wrappers) for deterministic dynamic response bodies (`docs/book/946-m39-lasm-overload-probe-timeout-and-req-placeholder-materialization.md`).
- [x] LASM `res.text(...)` dynamic materialization now includes `req.query("...")` placeholders (including `validate.nonEmpty(...)` wrappers) using parsed request query maps for deterministic query-driven response bodies (`docs/book/947-m39-lasm-req-query-placeholder-materialization.md`).
- [x] LASM query parsing now decodes `%XX` escapes and `+` (with deterministic raw fallback for invalid percent escapes), aligning `req.query(...)` placeholder materialization with existing runtime query-decoding behavior (`docs/book/948-m39-lasm-query-percent-decoding-parity.md`).
- [x] LASM placeholder materialization now also applies to response headers, including typed sink forms like `headers.value(validate.nonEmpty(req.query("trace")))`, keeping dynamic body/header response behavior aligned for request-derived route/query/header values (`docs/book/949-m39-lasm-response-header-placeholder-materialization.md`).
- [x] LASM `res.addCookie(cookie.build(...))` now materializes request-derived placeholders in cookie values (for example `cookie.build("session", validate.nonEmpty(req.query("session")))`), so dynamic Set-Cookie behavior matches LASM dynamic header/body materialization semantics (`docs/book/950-m39-lasm-dynamic-set-cookie-placeholder-materialization.md`).
- [x] LASM now emits multiple `Set-Cookie` header lines when handlers call `res.addCookie(...)` multiple times in one response, preserving deterministic cookie ordering while keeping dynamic placeholder materialization intact (`docs/book/951-m39-lasm-multi-set-cookie-emission.md`).
- [x] LASM `cookie.build(name, value)` extraction now materializes request-derived placeholders for both cookie names and values, enabling deterministic dynamic cookie-name flows (for example `cookie.build(validate.nonEmpty(req.query("cookie_name")), validate.nonEmpty(req.query("session")))`) in LASM run mode (`docs/book/952-m39-lasm-dynamic-cookie-name-placeholder-materialization.md`).
- [x] LASM response-header placeholder materialization now covers dynamic header keys as well as values (for example `res.setHeader(headers.name(validate.nonEmpty(req.query("header_name"))), headers.value(...))`), preserving deterministic dynamic-header behavior under typed sink wrappers (`docs/book/953-m39-lasm-dynamic-response-header-name-materialization.md`).
- [x] LASM response writer now normalizes/merges headers case-insensitively before default injection and trace/header upserts, preventing duplicate semantic headers like `Content-Type`/`content-type` and preserving deterministic override behavior (`docs/book/954-m39-lasm-case-insensitive-response-header-merge.md`).
- [x] LASM response writer now always upserts `Content-Length` from actual response body size (case-insensitive), overriding user-supplied `content-length` header values to prevent invalid framing while preserving deterministic header output (`docs/book/955-m39-lasm-content-length-upsert-correctness.md`).
- [x] LASM dynamic response header materialization now validates emitted header names/values using HTTP token/value constraints and drops invalid materialized headers deterministically (preventing control-character header injection via query/path/header-derived placeholders) (`docs/book/956-m39-lasm-dynamic-response-header-validation-hardening.md`).
- [x] LASM response-header extraction now unwraps `validate.headerValue(...)` wrappers (including request-derived arguments) when materializing dynamic headers, so direct typed-header flows (`res.setHeader(..., validate.headerValue(req.query(...)))`) execute with deterministic placeholder behavior (`docs/book/957-m39-lasm-validate-header-value-placeholder-extraction.md`).
- [x] LASM dynamic response-header name validation now matches header-name gate/runtime grammar (`[A-Za-z0-9-]+`), rejecting broader HTTP-token-only names (for example `_`) so dynamic placeholder-driven header names stay parity-aligned with C runtime sink constraints (`docs/book/958-m39-lasm-dynamic-header-name-grammar-parity.md`).
- [x] LASM dynamic response-header materialization now rejects empty materialized header values (including split dynamic `Set-Cookie` lines), preserving deterministic non-empty sink parity while keeping existing control-character validation intact (`docs/book/959-m39-lasm-dynamic-response-header-non-empty-value-parity.md`).
- [x] LASM request parsing now merges duplicate request headers case-insensitively in deterministic arrival order (`Cookie` via `; `, other headers via `, `) while preserving strict `Host`/`Content-Length` conflict checks (`docs/book/960-m39-lasm-request-header-case-insensitive-merge.md`).
- [x] Added first-class `req.cookie` intrinsic support across semantic typing, C backend/runtime ABI, and LASM placeholder extraction/materialization (`req.cookie(...)` + `{{req.cookie:...}}`) for deterministic cookie-driven handler and response behavior (`docs/book/961-m39-req-cookie-intrinsic-and-lasm-materialization.md`).
- [x] Added first-class `req.method()` / `req.path()` intrinsic support across semantic typing, C backend/runtime ABI, and LASM placeholder extraction/materialization (`{{req.method}}`, `{{req.path}}`) for deterministic method/path-driven handler and response behavior (`docs/book/962-m39-req-method-path-intrinsics-and-lasm-placeholders.md`).
- [x] Added first-class `req.httpVersion()` intrinsic support across semantic typing, C backend/runtime ABI, and LASM placeholder extraction/materialization (`{{req.httpVersion}}`) for deterministic request-version-aware handler and response behavior (`docs/book/963-m39-req-http-version-intrinsic-and-lasm-placeholder-parity.md`).
- [x] LASM query parsing now keeps first-seen values for duplicate keys (matching C runtime map behavior) so `req.query("...")` remains backend-parity deterministic under duplicate-key requests (`docs/book/964-m39-lasm-query-duplicate-first-value-parity.md`).
- [x] LASM `req.cookie(...)` lookup now matches cookie names case-insensitively (trimmed) so cookie-driven handlers/placeholders stay parity-aligned with C runtime behavior under mixed-case cookie keys (`docs/book/965-m39-lasm-req-cookie-case-insensitive-lookup-parity.md`).
- [x] C runtime `req.header(...)` / `req.cookie(...)` now merge duplicate request headers case-insensitively in arrival order (`Cookie` via `; `, generic headers via `, `, with first-value preservation for `Host`/`Content-Length`) to align request-header duplicate handling with LASM behavior (`docs/book/966-m39-c-runtime-req-header-cookie-duplicate-merge-parity.md`).
- [x] C runtime string-bridge ABI now accepts tracked handles for `res.text`, `headers.name/value`, and `cookie.build` (with literal fallback), fixing dynamic request-derived string flows in native oneshot/runtime paths (`docs/book/967-m39-c-runtime-string-handle-literal-abi-bridge-for-res-headers-cookies.md`).
- [x] C runtime `err.*` top-level helpers now accept tracked string handles (with literal fallback) for error `code`/`message` arguments, enabling deterministic dynamic error-message flows in native oneshot/runtime paths (`docs/book/968-m39-c-runtime-string-handle-literal-abi-bridge-for-err-helpers.md`).
- [x] C runtime `log.*` string-accepting helpers now accept tracked handles (with literal fallback) across event/key/value/method/path logging surfaces, closing string ABI gaps for structured logging flows in native execution (`docs/book/969-m39-c-runtime-string-handle-literal-abi-bridge-for-log-helpers.md`).
- [x] C runtime `err.with*` enrichment helpers now accept tracked string handles (with literal fallback + deterministic default field names), enabling dynamic error-path/detail/limit/dependency enrichment in native runtime paths (`docs/book/970-m39-c-runtime-string-handle-literal-abi-bridge-for-err-with-helpers.md`).
- [x] C runtime `sql.q`, `sec.cspAdd`, and `auth.requireRole` now accept tracked string handles (with literal fallback), closing remaining high-use string ABI mismatch points across DB/security/auth helper surfaces (`docs/book/971-m39-c-runtime-string-handle-literal-abi-bridge-for-auth-csp-sql-helpers.md`).
- [x] Added first-class `ctx.current()` intrinsic bridge across semantic typing/call-shape enforcement, C backend lowering, and runtime ABI (`sec4_rt_ctx_current`), enabling route handlers to obtain `Ctx` directly for `auth.require(...)` / `auth.requireRole(...)` flows in native run paths (`docs/book/972-m39-ctx-current-intrinsic-runtime-bridge.md`).
- [x] LASM run backend now enforces handler auth-helper contracts (`auth.require`, `auth.requireRole`) with policy-aware token/cookie subject checks and deterministic `401/403` envelopes, including dynamic required-role materialization via request placeholders (`docs/book/973-m39-lasm-auth-helper-enforcement-with-ctx-current.md`).
- [x] LASM run backend now enforces router middleware auth/csrf contracts (`auth.withAuth`, `csrf.withCsrf`) by extracting middleware usage from the route composition call graph and applying policy-aware deterministic `401 AUTH.UNAUTHORIZED` / `403 AUTH.CSRF_TOKEN_INVALID` response envelopes at request time (`docs/book/974-m39-lasm-auth-csrf-middleware-enforcement.md`).
- [x] LASM auth middleware enforcement now respects `SEC4_RT_AUTH_MODE` env override (`off` disables middleware auth checks) while preserving handler-level auth-helper enforcement semantics, matching C-runtime policy/env precedence more closely (`docs/book/975-m39-lasm-auth-middleware-env-off-parity.md`).
- [x] LASM csrf middleware enforcement now respects `SEC4_RT_CSRF_*` env overrides (including `SEC4_RT_CSRF_MODE=off`) for enabled-mode, cookie/header names, and protected-method sets, preserving deterministic `403 AUTH.CSRF_TOKEN_INVALID` only when effective csrf protection remains active (`docs/book/976-m39-lasm-csrf-middleware-env-off-parity.md`).
- [x] LASM route registration now derives middleware requirements from the specific router expression bound to each route (including local router-wrapper helper returns, helper side-effect calls on router bindings, non-first router-argument wrappers/side-effect helpers resolved through caller alias + typed router-parameter matching, and middleware wrappers applied later at `http.serve(...)` time), preventing unrelated/unbound `auth.withAuth` or `csrf.withCsrf` chains from leaking middleware enforcement onto other routes (`docs/book/977-m39-lasm-route-scoped-middleware-extraction.md`, `docs/book/978-m39-lasm-helper-wrapper-router-binding-resolution.md`, `docs/book/979-m39-lasm-serve-wrapper-middleware-propagation.md`, `docs/book/980-m39-lasm-helper-side-effect-router-argument-resolution.md`).

### M39-S3 `sec4 promote` dry-run planner acceptance criteria

- New command surface:
  - `sec4 promote --from browser --to server --dry-run [--path <project>]`.
- Dry-run emits deterministic transformation plan artifact:
  - changed bindings
  - generated files list
  - precondition diagnostics (if any).
- Re-running dry-run on unchanged tree yields byte-identical plan output.

### M39-S3 tracking (live status)

- [x] CLI command scaffolding + planner implementation added (`sec4 promote --from browser --to server --dry-run [--out <path>]`).
- [x] Determinism tests for plan artifact added (byte-identical dry-run output across repeated runs on unchanged tree + blocking-precondition coverage + semantic-diagnostic non-blocking warning coverage).
- [x] Book chapter documenting S3 implementation added (`docs/book/920-m39-promote-dry-run-planner-baseline.md`).

### M39-S4 promotion apply + generated scaffold acceptance criteria

- `sec4 promote --from browser --to server` applies composition-root binding rewrite only.
- Domain modules remain unchanged after promotion.
- Generator emits deterministic server scaffolding:
  - server repo adapter
  - schema/migration baseline
  - deploy profile baseline
  - promotion report artifact.
- Added browser-export -> server-import scaffold path with deterministic validation envelope.
- End-to-end fixture proves:
  - browser profile prototype builds/runs,
  - promotion succeeds,
  - server target builds/runs on generated scaffold.

### M39-S4 tracking (live status)

- [x] Apply rewrite engine implemented with composition-root-only rewrite guard (`sec4 promote --from browser --to server` rewrites only `src/main.ut`).
- [x] Scaffold generator + deterministic report implemented (generated `server/*` baseline files + `server/reports/promote-plan.json` apply report).
- [x] End-to-end promotion fixture (browser -> server) added and green (`promote_e2e_apply_generates_server_project_that_checks_and_runs`).
- [x] Book chapter documenting S4 implementation baseline added (`docs/book/921-m39-promote-apply-composition-root-guard.md`).

### Readiness estimate (live)

- Runnable alpha (end-to-end): ~97-99%
- Strict no-stub alpha: ~95-97%

### Remaining implementation slices (priority order)

1. [x] Completed LASM DB parity for active DB intrinsics (`sql.q`, `db.exec`, `db.execTx`, `db.queryOne`, `db.tx`) and removed schema-switch fallback branches for write/query-one paths while keeping deterministic envelopes on `records.log` adapter v1.
2. [x] Made LASM the default server runtime path for `sec4 run` (C backend remains explicit fallback), with clean-machine `init -> check -> build -> run` validated on LASM-first flow.
3. Harden LASM runtime under sustained load (queue/backpressure/timeout tuning and regression baseline for throughput/latency/memory).
   - [x] Added built-in LASM horizontal front-layer orchestration in `sec4 run`:
     - `--instances <N>` launches multi-instance LASM worker pool behind a built-in TCP front proxy.
     - `--autoscale-max-instances`, `--autoscale-target-connections`, `--autoscale-check-ms` add adaptive worker scaling controls.
     - documented in `docs/book/1018-m39-lasm-run-cluster-front-proxy-and-autoscale.md`.
   - [x] Added command coverage for cluster guardrails and cluster serving path.
   - [x] Added fast fixed-cluster shared-port mode (`SO_REUSEPORT`) for LASM:
     - when `--instances == --autoscale-max-instances`, workers bind one shared port without front-proxy relay.
     - benchmark probes show improved throughput vs proxy-cluster path (for example ~96k req/s on `/ping` in local wrk profile).
     - documented in `docs/book/1024-m39-lasm-fixed-cluster-reuseport-fast-path.md`.
   - [x] Replaced front-proxy thread-per-connection relay with bounded relay worker pool:
     - proxy now uses a fixed worker-count + bounded queue instead of spawning one relay thread per accepted connection,
     - queue saturation now returns deterministic `503` (`cluster relay saturated`) instead of unbounded relay thread growth,
     - local load probes after this change reached about `~60k req/s` in autoscale proxy mode and `~121k req/s` in fixed reuse-port mode (`/health`, auth header, local wrk profile).
     - documented in `docs/book/1025-m39-lasm-cluster-relay-worker-pool.md`.
   - [x] Added explicit cluster-proxy relay tuning flags:
     - `--cluster-relay-workers` and `--cluster-relay-queue` now tune front-proxy relay concurrency/queue depth without code edits,
     - both flags are LASM-only and require cluster proxy mode (`--instances > 1` with autoscale range); fixed reuse-port mode rejects them as not applicable.
     - documented in `docs/book/1026-m39-lasm-cluster-relay-tuning-flags.md`.
   - [x] Removed receiver-lock contention from LASM worker dispatch:
     - replaced `Arc<Mutex<Receiver<TcpStream>>>` fan-out with bounded multi-consumer channels (`crossbeam-channel`) for both proxy relay workers and LASM backend workers,
     - keeps bounded backpressure semantics while avoiding serialized `recv()` lock sections under load,
     - local proxy-mode probe remained stable at about `~60k req/s` after the channel swap.
     - documented in `docs/book/1027-m39-lasm-worker-dispatch-crossbeam-channel.md`.
   - [x] Added autoscale hysteresis controls for cluster worker scaling:
     - new LASM flags: `--autoscale-scale-up-cooldown-ms`, `--autoscale-scale-down-cooldown-ms`, `--autoscale-scale-up-step`, and `--autoscale-scale-down-step`,
     - autoscaler now enforces separate up/down cooldown windows and bounded per-check scale steps to reduce worker-count thrash on bursty traffic,
     - flags stay LASM-only with deterministic validation and guard diagnostics.
     - LASM cluster capacity probe script now supports these step controls for benchmark runs.
     - documented in `docs/book/1031-m39-lasm-autoscale-step-window-controls.md`.
   - [x] Added dedicated LASM cluster capacity probe tooling:
     - new `benchmark-suite/scripts/run_lasm_cluster_capacity_probe.sh` launches `sec4 run --backend lasm` with cluster/tuning flags, runs `wrk`, samples peak RSS, and writes deterministic JSON pass/fail output against target request count,
     - integrated make target: `make -C benchmark-suite lasm-cluster-capacity-probe`,
     - dry-run contract coverage added in benchmark suite script tests.
     - first 1M-threshold run evidence: `1,278,004` requests in `20s` (`~63.6k req/s`, peak RSS `~10,464 KB`) with tuned relay settings (`workers=32`, `queue=4096`).
     - documented in `docs/book/1029-m39-lasm-cluster-capacity-probe-tooling.md`.
   - [x] Added saturation-triggered autoscale boost in cluster proxy mode:
     - relay queue full events are counted and consumed by autoscaler each check window,
     - saturation pressure can raise `desired` worker count immediately (bounded by autoscale step and max instance caps) instead of relying only on active-connection heuristic,
     - cooldown and per-check step windows remain enforced.
     - documented in `docs/book/1032-m39-lasm-autoscale-saturation-boost.md`.
   - [x] Added dedicated saturation boost-step control for cluster autoscaling:
     - new LASM flag: `--autoscale-saturation-boost-step` (default `4`),
     - when relay saturation events are observed, autoscale now boosts desired workers and per-check scale-up budget using this dedicated step (independent from normal `--autoscale-scale-up-step`),
     - benchmark cluster capacity probe tooling now accepts/forwards the same flag for deterministic tuning workflows.
     - documented in `docs/book/1034-m39-lasm-autoscale-saturation-boost-step-control.md`.
   - [x] Added saturation boost matrix runner tooling for deterministic tuning sweeps:
     - new benchmark script runs multiple cluster capacity probes across `--boost-steps <csv>` and records per-step summaries,
     - dry-run output prints the full per-step execution plan; run mode writes a matrix summary artifact with pass/requests/throughput/RSS per boost step,
     - Makefile target added: `lasm-cluster-saturation-boost-matrix`.
     - documented in `docs/book/1035-m39-lasm-saturation-boost-matrix-tooling.md`.
   - [x] Added saturation boost matrix analysis tooling for deterministic best-step selection:
     - new benchmark script `benchmark-suite/scripts/analyze_lasm_cluster_saturation_boost_matrix.sh` consumes matrix output and emits ranked runs plus a recommended `saturationBoostStep`,
     - ranking is deterministic (`pass` first, then throughput, then request count, then lower RSS, then lower step),
     - Makefile target added: `lasm-cluster-saturation-boost-analyze`.
     - documented in `docs/book/1037-m39-lasm-saturation-boost-analysis-tooling.md`.
   - [x] Matrix runner now supports integrated recommendation emission:
     - `run_lasm_cluster_saturation_boost_matrix.sh` now accepts `--analysis-out` and runs analysis automatically by default after matrix generation,
     - run mode now prints `recommendedSaturationBoostStep=<n>` from the generated analysis artifact,
     - `--skip-analysis` keeps matrix-only behavior for explicit operator control.
     - documented in `docs/book/1038-m39-lasm-saturation-boost-matrix-integrated-analysis.md`.
   - [x] Matrix runner now supports recommended-step follow-up verification:
     - new flag `--verify-recommended` runs one additional capacity probe using the analyzed recommended boost step,
     - `--verify-out` controls follow-up artifact location, and the runner now rejects invalid `--verify-recommended --skip-analysis` combinations deterministically,
     - run mode prints `recommendedVerificationOut=<path>` after follow-up execution.
     - documented in `docs/book/1039-m39-lasm-saturation-boost-recommended-followup-verification.md`.
   - [x] Added saturation boost markdown summary renderer for operator handoff:
     - new script `benchmark-suite/scripts/render_lasm_cluster_saturation_boost_summary.sh` renders deterministic markdown from matrix + analysis artifacts (and optional verify artifact),
     - renderer validates recommended-step consistency between analysis and optional verification artifact,
     - Makefile target added: `lasm-cluster-saturation-boost-summary`.
     - documented in `docs/book/1040-m39-lasm-saturation-boost-summary-renderer.md`.
   - [x] Benchmark publish report now supports saturation summary integration:
     - `benchmark-suite/scripts/publish_report.sh` now accepts optional saturation-summary input and emits a dedicated `LASM Saturation Boost Tuning` section,
     - publish flow validates required saturation summary lines (`Selection mode`, `Recommended boost step`) when artifact is provided to catch summary-shape drift,
     - Makefile `publish-report` target now forwards optional `SATURATION_SUMMARY` input without changing default behavior when unset.
     - documented in `docs/book/1041-m39-publish-report-saturation-summary-integration.md`.
   - [x] Added one-command saturation tuning bundle orchestration:
     - new script `benchmark-suite/scripts/run_lasm_cluster_saturation_boost_bundle.sh` executes matrix + analysis + recommended-step verify + summary rendering as one deterministic flow,
     - supports `--dry-run` plan output and `--skip-verify` mode for matrix/analysis/summary-only execution,
     - Makefile target added: `lasm-cluster-saturation-boost-bundle`.
     - documented in `docs/book/1042-m39-lasm-saturation-boost-bundle-orchestration.md`.
   - [x] Full benchmark suite orchestration now supports optional LASM saturation lane:
     - `run_full_benchmark_suite.sh --include-lasm-saturation` now executes the saturation bundle as an explicit phase and forwards generated summary into final report publishing,
     - deterministic guard rejects saturation-lane requests when `sec4-lasm` is not included in `--impls`,
     - Makefile targets added: `bench-full-saturation` and `bench-full-saturation-dry`.
     - documented in `docs/book/1043-m39-full-suite-optional-saturation-lane.md`.
   - [x] Full-suite saturation lane now forwards performance-tuning knobs for iterative runs:
     - `run_full_benchmark_suite.sh` accepts explicit saturation lane pass-through flags (`--saturation-project-path`, `--saturation-duration`, `--saturation-threads`, `--saturation-connections`, `--saturation-target-requests`, relay worker/queue overrides),
     - `bench-full-saturation` and `bench-full-saturation-dry` now forward `LASM_CAPACITY_*` tuning values into the saturation lane so operators can tune from one Make invocation,
     - dry-run contract coverage now asserts delegated tuning values and step fanout in full-suite output.
     - documented in `docs/book/1047-m39-full-suite-saturation-tuning-knob-forwarding.md`.
   - [x] Added throughput-oriented saturation-enabled full-suite preset targets:
     - new Make targets `bench-full-saturation-throughput` and `bench-full-saturation-throughput-dry` apply tuned defaults for quick iterative throughput probes (`sec4-lasm`, `ping`, skip verify, boosted load/relay knobs),
     - preset values are overridable through `LASM_SATURATION_THROUGHPUT_*` variables while still delegating through full-suite saturation lane wiring,
     - added runtime dry-run contract test ensuring preset defaults reach delegated saturation plan output.
     - documented in `docs/book/1048-m39-full-suite-saturation-throughput-preset-targets.md`.
   - [x] Added latency-oriented saturation-enabled full-suite preset targets:
     - new Make targets `bench-full-saturation-latency` and `bench-full-saturation-latency-dry` apply lighter default load/relay settings for quicker latency-focused tuning passes,
     - preset values are overridable through `LASM_SATURATION_LATENCY_*` variables and still flow through full-suite saturation lane delegation,
     - added runtime dry-run contract test ensuring latency preset defaults reach delegated saturation plan output.
     - documented in `docs/book/1049-m39-full-suite-saturation-latency-preset-targets.md`.
   - [x] Added combined saturation preset orchestrator targets for one-command dual-profile sweeps:
     - new Make targets `bench-full-saturation-presets` and `bench-full-saturation-presets-dry` run throughput and latency presets sequentially via existing preset targets,
     - this keeps one-command exploratory tuning flows deterministic while preserving existing preset override behavior,
     - added dry-run contract test validating both preset profiles appear in combined target output.
     - documented in `docs/book/1050-m39-full-suite-saturation-combined-preset-targets.md`.
   - [x] Cluster proxy hot-path lock contention reduced:
     - relay worker request path no longer performs worker pruning/recovery/spawn under state lock for each incoming connection,
     - background scaler/maintenance loop now owns dead-worker pruning + min-instance recovery in one place,
     - relay backend-port selection now uses an atomic selection counter over current worker set to keep per-connection critical section minimal.
     - documented in `docs/book/1051-m39-lasm-cluster-hot-path-lock-contention-reduction.md`.
   - [x] Cluster state lock model optimized for relay-heavy read paths:
     - cluster worker state now uses `RwLock` so relay workers take shared read locks for backend selection,
     - background maintenance/autoscale loop remains the only writer for prune/recovery/scale mutations,
     - this removes unnecessary writer lock serialization between parallel relay workers during steady-state dispatch.
     - documented in `docs/book/1052-m39-lasm-cluster-read-write-lock-state-optimization.md`.
   - [x] Relay backend-port selection now uses lock-free worker-port snapshots:
     - autoscale/maintenance loop publishes current worker-port vectors through atomic snapshot updates,
     - relay workers read the latest snapshot and select backend ports via atomic round-robin counter without touching cluster state locks,
     - state locks remain on maintenance/autoscale mutation paths only.
     - documented in `docs/book/1053-m39-lasm-cluster-lock-free-worker-port-snapshots.md`.
   - [x] Relay failure handling now feeds autoscale pressure and bounds upstream connect stalls:
     - relay path increments saturation events on `no healthy workers` and backend connect failures,
     - backend worker connect now uses deterministic short timeout (`250ms`) instead of unbounded connect waits,
     - this improves failure-mode responsiveness and helps saturation-based scaling react faster to worker churn.
     - documented in `docs/book/1054-m39-lasm-cluster-relay-connect-timeout-and-saturation-signal.md`.
   - [x] Worker-port snapshot publishing now avoids steady-state churn:
     - autoscale loop republishes worker-port snapshots only when worker-port set actually changes,
     - steady-state maintenance ticks no longer allocate/store redundant snapshot vectors,
     - relay lock-free selection path remains unchanged while background publication overhead is reduced.
     - documented in `docs/book/1055-m39-lasm-cluster-snapshot-publish-change-detection.md`.
   - [x] Cluster relay path no longer spawns per-connection helper threads:
     - replaced `relay_lasm_cluster_connection` thread-spawn copy bridge with a single-thread nonblocking bidirectional pump loop (buffered both directions with deterministic half-close behavior),
     - removes per-connection relay thread creation/cloning overhead while preserving existing overload/autoscale/error-envelope semantics,
     - short capacity probe under unchanged profile improved from `~64.9k req/s` to `~70.5k req/s` (`20s`, `8` threads, `256` connections, `instances=4`, `autoscale-max=8`, `relay-workers=32`, `relay-queue=4096`).
     - documented in `docs/book/1056-m39-lasm-cluster-relay-no-spawn-pump-loop.md`.
   - [x] Relay worker auto-sizing now follows configured connection pressure:
     - default `--cluster-relay-workers` (auto mode) now uses `target_connections_per_instance` as the sizing floor (bounded by an adaptive cap from `max_instances * target_connections_per_instance`, clamped `128..512`),
     - this reduces under-provisioned relay pools in keep-alive-heavy cluster proxy workloads without requiring manual `--cluster-relay-workers` tuning,
     - short capacity probe with relay settings left in auto mode observed `~71.9k req/s` (`1,446,283` requests in `20s`) under the same baseline profile.
     - documented in `docs/book/1057-m39-lasm-cluster-relay-auto-worker-pressure-sizing.md`.
   - [x] Cluster relay workers now multiplex many active connection pairs per worker thread:
     - replaced one-connection-at-a-time relay handling with per-worker nonblocking relay pump sets (`Vec<LasmClusterRelayPump>`), so each relay worker drives multiple client<->backend pairs concurrently,
     - preserves deterministic overload/availability envelopes and autoscale saturation signaling while reducing relay worker head-of-line blocking under keep-alive pressure,
     - short auto-mode probe moved from `~71.9k req/s` to `~73.3k req/s` (`1,474,217` requests in `20s`) with lower observed peak RSS (`~21,952 KB` vs `~46,224 KB`) in the same profile.
     - documented in `docs/book/1058-m39-lasm-cluster-relay-worker-multiplex-pump-loop.md`.
   - [x] Relay workers now apply local backend-port connect-failure cooldown with healthy fast path:
     - on backend connect failure, relay worker marks the failed worker port unhealthy for a short cooldown window (`500ms`) and skips it during backend selection,
     - when no unhealthy ports are tracked, relay selection remains a direct round-robin fast path (no cooldown-map scan),
     - this reduces repeated immediate retries against transiently unavailable worker ports during churn while preserving existing deterministic `503` availability envelopes.
     - documented in `docs/book/1059-m39-lasm-cluster-relay-port-connect-failure-cooldown.md`.
   - [x] Relay pump read path now drains available socket bytes per cycle:
     - changed both client->upstream and upstream->client read phases from single-read-per-cycle to read-until-`WouldBlock` loops while preserving bounded buffer behavior and existing half-close semantics,
     - this reduces relay scheduler churn under active traffic by allowing each pump cycle to consume contiguous readable socket bursts,
     - short probe in the current post-cooldown baseline moved from `~72.49k req/s` to `~72.76k req/s` with comparable memory/tail-latency profile.
     - documented in `docs/book/1060-m39-lasm-cluster-relay-read-drain-loop.md`.
   - [x] Added optional LASM cluster status JSON telemetry output:
     - new flag: `sec4 run --cluster-status-json <path>` (LASM cluster mode only),
     - cluster front process now emits periodic JSON snapshots with worker counts/ports, active connections, and relay saturation counters (`pending` + `total`) for operator tuning/debug workflows,
     - healthy hot path remains unchanged when the flag is not set (status writer thread is not started).
     - documented in `docs/book/1061-m39-lasm-cluster-status-json-telemetry.md`.
   - [x] Tuned relay backend-connect failover behavior with precomputed configurable timing defaults:
     - moved relay backend connect timeout/cooldown off fixed constants into cluster config resolution,
     - added env-tunable controls (`SEC4_RT_LASM_CLUSTER_BACKEND_CONNECT_TIMEOUT_MS`, `SEC4_RT_LASM_CLUSTER_BACKEND_CONNECT_COOLDOWN_MS`) with deterministic bounded parsing and lower-latency default timeout path (`100ms`),
     - relay worker threads now consume these precomputed durations directly for backend connect attempts and unhealthy-port cooldown windows.
     - documented in `docs/book/1124-m39-lasm-cluster-backend-connect-timeout-tuning.md`.
   - [x] Added relay warning throttling in cluster failure paths:
     - backend connect-failure warnings are now rate-limited per worker port,
     - relay init/pump failure warnings are rate-limited per relay worker loop,
     - keeps diagnostics while preventing stderr log spam from dominating runtime under sustained failure churn.
     - documented in `docs/book/1125-m39-lasm-cluster-relay-warning-throttle.md`.
   - [x] Added bounded relay accept batching per worker loop iteration:
     - relay workers now cap the number of accepted client sockets per cycle before returning to pump existing active relays, preventing accept-path starvation under sustained intake pressure,
     - new env override `SEC4_RT_LASM_CLUSTER_RELAY_ACCEPT_BATCH_MAX` controls the per-cycle cap (default `64`, clamped `1..4096`) without changing CLI surface.
     - documented in `docs/book/1126-m39-lasm-cluster-relay-accept-batch-fairness.md`.
   - [x] Reduced unhealthy-port cooldown-map maintenance overhead in relay accept path:
     - relay workers now prune expired unhealthy-port entries once per worker cycle (before accept batching) instead of re-running map retention for every accepted connection,
     - backend selection in the unhealthy-path now uses the already-pruned map, keeping the fast path unchanged and reducing per-connection bookkeeping churn under sustained traffic.
     - documented in `docs/book/1127-m39-lasm-cluster-unhealthy-map-fastpath.md`.
   - [x] Reduced per-accept worker-port snapshot overhead in relay loop:
     - relay workers now capture worker-port snapshot once per worker cycle and reuse it across that cycle’s accept batch,
     - backend selection no longer performs a fresh `ArcSwap` load per accepted connection, reducing hot-path atomic/snapshot churn while keeping behavior deterministic.
     - documented in `docs/book/1128-m39-lasm-cluster-worker-snapshot-per-cycle.md`.
   - [x] Added worker-membership-aware unhealthy-port pruning and full-unhealthy fast reject:
     - relay workers now drop unhealthy-port cooldown entries that no longer belong to the current worker snapshot while pruning expirations,
     - when all current workers are marked unhealthy in a cycle, backend selection now short-circuits directly to unavailable handling without scanning the full worker list.
     - documented in `docs/book/1129-m39-lasm-cluster-unhealthy-membership-prune.md`.
   - [x] Replaced formatted 503 relay-envelope construction with static response payloads:
     - cluster relay unavailable envelopes now use prebuilt static response bytes for deterministic failure reasons (`no healthy workers`, `worker unavailable`, `cluster relay saturated`, `cluster relay unavailable`),
     - removes per-request string formatting/allocation on relay failure paths while preserving status/body contract behavior.
     - documented in `docs/book/1130-m39-lasm-cluster-static-unavailable-response-bytes.md`.
   - [x] Batched relay saturation counter atomics per worker cycle:
     - relay worker threads now accumulate saturation pending/total increments locally during accept/connect failure handling,
     - counters are flushed to shared atomics once per worker cycle, reducing repeated atomic writes in failure-heavy intervals while preserving counter semantics.
     - documented in `docs/book/1131-m39-lasm-cluster-saturation-counter-batch.md`.
   - [x] Optimized unhealthy membership pruning with per-cycle active-port set:
     - relay workers now build a per-cycle active worker-port set for unhealthy-map membership checks instead of repeated linear `Vec::contains` scans during retain,
     - when worker snapshot is empty, unhealthy cooldown map now clears immediately to avoid stale carry-over.
     - documented in `docs/book/1132-m39-lasm-cluster-unhealthy-membership-set-prune.md`.
   - [x] Removed status-writer state-lock reads for worker count telemetry:
     - cluster status writer now derives `workerCount` directly from the lock-free worker-port snapshot (`worker_ports.len()`),
     - removes periodic `shared_state` read-lock usage from telemetry path while preserving status payload semantics.
     - documented in `docs/book/1133-m39-lasm-cluster-status-lockfree-worker-count.md`.
   - [x] Batched listener-side saturation counter atomics under overload:
      - accept loop now accumulates saturation pending/total increments locally on queue-full events and flushes atomics in small batches (`LASM_CLUSTER_SATURATION_COUNTER_FLUSH_BATCH`),
      - shared helper now standardizes saturation-counter flush behavior across relay worker and listener paths.
      - documented in `docs/book/1134-m39-lasm-cluster-listener-saturation-counter-batch.md`.
   - [x] Reduced enqueue-counter atomics and unhealthy-set allocations in relay hot paths:
      - listener path now increments `active_connections` only after successful relay queue enqueue (no add/sub pair on rejected connections),
      - relay workers now reuse a mutable active-port set for unhealthy membership pruning instead of allocating a new set each cycle.
      - documented in `docs/book/1135-m39-lasm-cluster-enqueue-counter-and-set-reuse.md`.
   - [x] Cached active-port membership set by worker snapshot identity:
      - relay workers now keep the active-port set cache keyed by current worker snapshot pointer identity and rebuild only when snapshot changes,
      - unhealthy membership pruning no longer repopulates the set on every cycle when worker snapshot is unchanged.
      - documented in `docs/book/1136-m39-lasm-cluster-active-set-snapshot-cache.md`.
   - [x] Removed explicit flush syscall from cluster unavailable response writes:
      - relay overload/unavailable paths now write prebuilt response bytes and rely on connection close semantics, without `TcpStream::flush()` per response,
      - preserves deterministic response payload behavior while reducing failure-path syscall overhead.
      - documented in `docs/book/1137-m39-lasm-cluster-unavailable-write-no-flush.md`.
   - [x] Switched cluster status telemetry emission to compact JSON encoding:
      - status writer now uses compact `serde_json::to_vec(...)` output instead of pretty-printed payload encoding,
      - keeps JSON payload fields/semantics unchanged while reducing periodic telemetry serialization and write size overhead.
      - documented in `docs/book/1138-m39-lasm-cluster-status-compact-json-encoding.md`.
   - [x] Precomputed status writer temp-file path outside write loop:
     - status thread now derives the `<status>.tmp` path once before entering the periodic loop and passes it into status writer calls,
     - removes repeated path-extension formatting/allocations from every status-write iteration.
     - documented in `docs/book/1139-m39-lasm-cluster-status-precompute-tmp-path.md`.
   - [x] Replaced single relay queue with sharded queues + listener accept/dispatch batching:
     - cluster proxy now allocates one bounded relay queue per relay worker shard instead of one globally contended queue (`crossbeam` fan-in hot spot removed),
     - listener accept path now runs nonblocking batched intake (`cluster_relay_accept_batch_max`) and round-robins each accepted socket across queue shards with deterministic saturated/unavailable fallbacks.
     - short LASM cluster probe after this slice recorded `1,389,512` requests in `20s` (`~69.45k req/s`, peak RSS `~26,976 KB`, `p99 12.38ms`) under auto relay settings.
     - documented in `docs/book/1140-m39-lasm-cluster-sharded-relay-queue-and-batched-listener-dispatch.md`.
   - [x] Decoupled queue-shard dispatch cursor from backend-port selection counter:
     - listener queue-shard dispatch now uses a local batched round-robin cursor instead of sharing the relay backend-selection atomic counter,
     - this removes an avoidable cross-path atomic contention point between listener enqueue and relay worker backend-port selection.
     - repeated short probes (`20s`, auto relay settings) after this change reported `~63.59k`, `~68.34k`, and `~71.52k req/s` (`peak RSS ~27,008..27,072 KB`), with top-run throughput above the prior short-run baseline.
     - documented in `docs/book/1141-m39-lasm-cluster-dispatch-cursor-decoupling.md`.
   - [x] Batched active-connection atomics in listener and relay worker loops:
     - listener now accumulates successful enqueue counts per accept batch and flushes one `active_connections` add per batch,
     - relay workers now accumulate completion/failure decrements locally and flush one `active_connections` subtract per cycle,
     - keeps overload/availability semantics unchanged while reducing per-connection atomic churn in hot paths.
     - short LASM cluster probe after this slice recorded `1,440,370` requests in `20s` (`~71.65k req/s`, `p99 10.16ms`, peak RSS `~27,120 KB`) under auto relay settings.
     - documented in `docs/book/1142-m39-lasm-cluster-active-connection-atomic-batching.md`.
   - [x] Added parallel accept-worker pool for cluster proxy intake/dispatch:
     - proxy listener now supports a bounded accept-worker pool (`SEC4_RT_LASM_CLUSTER_ACCEPT_WORKERS`, default `min(4, relay_workers)`) using cloned nonblocking listener sockets,
     - accept workers run batched intake/dispatch loops in parallel and share deterministic saturation/unavailable response handling,
     - cluster status JSON now includes `relayAcceptWorkers` for operator visibility of active accept-loop parallelism.
     - short LASM cluster probe after this slice recorded `1,437,939` requests in `20s` (`~71.53k req/s`, `p99 11.53ms`, peak RSS `~27,216 KB`) under default settings.
     - documented in `docs/book/1143-m39-lasm-cluster-accept-worker-pool.md`.
   - [x] Lazy-loaded relay worker-port snapshots in accept processing cycles:
     - relay worker loops no longer load worker-port snapshots unconditionally every cycle; snapshot loads now happen only when needed (unhealthy-port pruning or actual accepted connection handling),
     - preserves existing backend selection semantics while reducing steady-state hot-loop snapshot churn.
     - short LASM cluster probe after this slice recorded `1,450,295` requests in `20s` (`~72.14k req/s`, `p99 11.41ms`, peak RSS `~27,312 KB`) under default settings.
     - documented in `docs/book/1144-m39-lasm-cluster-relay-lazy-worker-snapshot-load.md`.
   - [x] Added explicit CLI tuning flag for cluster accept-worker pool:
     - new LASM-only cluster flag: `--cluster-accept-workers <n>` (`n >= 1`) to override accept-loop parallelism without env-only tuning,
     - guardrails enforce LASM backend + cluster mode only, and reject the flag in fixed reuse-port mode where proxy relay is bypassed.
     - documented in `docs/book/1145-m39-lasm-cluster-accept-workers-cli-flag.md`.
   - [x] Switched cluster queue-shard dispatch cursor to a shared atomic across accept workers:
     - accept-loop queue dispatch now uses one global `AtomicUsize` cursor shared by all accept worker threads instead of per-thread local round-robin state,
     - keeps dispatch and relay backend selection decoupled while removing per-accept-worker cursor skew under parallel intake,
     - short LASM cluster probe after this slice recorded `1,431,766` requests in `20s` (`~71.58k req/s`, `p99 9.08ms`, peak RSS `~27,296 KB`) under default settings.
     - documented in `docs/book/1146-m39-lasm-cluster-shared-accept-dispatch-cursor.md`.
   - [x] Batched relay backend-selection counter reservations in worker accept loops:
     - relay workers now reserve backend-selection counter slots once per accepted batch (`relay_accept_batch_max`) instead of one atomic `fetch_add(1)` per accepted socket when healthy-port fast path is active,
     - preserves backend-selection semantics while lowering cross-worker atomic contention on the hot path.
     - short LASM cluster probe after this slice recorded `1,450,919` requests in `20s` (`~72.18k req/s`, `p99 9.15ms`, peak RSS `~27,184 KB`) under default settings.
     - documented in `docs/book/1147-m39-lasm-cluster-relay-selection-counter-batch-reservation.md`.
   - [x] Removed per-dispatch modulo/index arithmetic from relay queue-sender iteration:
     - `dispatch_lasm_cluster_relay_stream` now uses split-slice sender traversal (`tail` then `head`) from `start_index` instead of modulo/index calculation in every loop step,
     - preserves dispatch ordering and existing saturated/unavailable fallback behavior.
     - short LASM cluster probe after this slice recorded `1,429,683` requests in `20s` (`~71.12k req/s`, `p99 9.91ms`, peak RSS `~27,152 KB`) under default settings.
     - documented in `docs/book/1148-m39-lasm-cluster-relay-dispatch-slice-iteration.md`.
   - [x] Reused relay backend-selection batch reservation for unhealthy-worker fallback path:
     - relay worker backend selection now draws `start_index` from the same per-batch reserved selection window used by healthy-path selection (instead of fallback per-connection `fetch_add(1)` atomics),
     - keeps unhealthy-port skip semantics unchanged while removing another high-frequency shared atomic in mixed-health runs.
     - short LASM cluster probe after this slice recorded `1,467,019` requests in `20s` (`~72.98k req/s`, `p99 9.19ms`, peak RSS `~27,040 KB`) under default settings.
     - documented in `docs/book/1149-m39-lasm-cluster-relay-selection-batch-fallback-reuse.md`.
   - [x] Simplified unhealthy-port prune membership check to snapshot-backed lookup:
     - relay worker unhealthy-port prune now checks membership directly against the current worker-port snapshot (`snapshot.contains(port)`) instead of maintaining a mirrored hash-set cache,
     - removes hash-set cache refresh churn while preserving unhealthy-port retention semantics.
     - short LASM cluster probe after this slice recorded `1,462,382` requests in `20s` (`~72.75k req/s`, `p99 8.84ms`, peak RSS `~27,200 KB`) under default settings.
     - documented in `docs/book/1150-m39-lasm-cluster-unhealthy-prune-snapshot-membership.md`.
   - [x] Throttled unhealthy-port prune cadence in relay worker hot loop:
     - relay workers now prune `unhealthy_ports_until` on a short bounded interval (`LASM_CLUSTER_UNHEALTHY_PRUNE_INTERVAL_MS=2`) instead of every loop cycle while unhealthy entries exist,
     - backend-selection snapshot loading remains on-demand for accepted sockets, preserving routing behavior while reducing prune-side snapshot churn.
     - short LASM cluster probe after this slice recorded `1,438,720` requests in `20s` (`~71.57k req/s`, `p99 10.08ms`, peak RSS `~27,280 KB`) under default settings.
     - documented in `docs/book/1151-m39-lasm-cluster-unhealthy-prune-interval-throttle.md`.
   - [x] Extended cluster status JSON telemetry with relay worker count:
     - status payload now includes `relayWorkerCount` (resolved queue relay worker pool size) alongside `workerCount` and `relayAcceptWorkers`,
     - improves operator tuning visibility for relay worker sizing decisions without changing runtime routing semantics.
     - documented in `docs/book/1152-m39-lasm-cluster-status-relay-worker-count.md`.
   - [x] Extended cluster status JSON telemetry with relay queue sizing fields:
     - status payload now includes `relayQueueCapacity` (global relay queue target capacity) and `relayQueueShardCapacity` (per-shard bounded queue capacity),
     - makes relay queue-pressure tuning (`--cluster-relay-queue`, relay worker counts) observable from one status artifact.
     - documented in `docs/book/1153-m39-lasm-cluster-status-relay-queue-capacity-fields.md`.
   - [x] Extended cluster status JSON telemetry with saturation-rate signal:
     - status payload now includes `relaySaturationEventsPerSec` derived from `relaySaturationEventsTotal` delta over status emission interval,
     - keeps existing pending/total counters and adds direct rate visibility for autoscale/tuning diagnosis.
     - documented in `docs/book/1154-m39-lasm-cluster-status-saturation-rate-field.md`.
   - [x] Made saturation-triggered autoscale boost step dynamic by event volume:
     - autoscale loop now derives `dynamic_boost_step` from saturation-event batch count (`relaySaturationEvents` per eval) instead of applying a fixed one-step saturation boost,
     - keeps existing cooldown and max-instance bounds while allowing faster scale-up reaction when saturation bursts are larger.
     - short LASM cluster probe after this slice recorded `1,433,593` requests in `20s` (`~71.32k req/s`, `p99 11.03ms`, peak RSS `~27,120 KB`) under default settings.
     - documented in `docs/book/1155-m39-lasm-cluster-dynamic-saturation-boost-step.md`.
   - [x] Added autoscale-decision telemetry fields to cluster status JSON:
     - status payload now includes `autoscaleDesiredInstances`, `autoscaleLastSaturationEvents`, and `autoscaleLastDynamicBoostStep`,
     - autoscale loop publishes these values from each evaluation cycle via shared atomics, so status snapshots show latest autoscale decision context directly.
     - documented in `docs/book/1156-m39-lasm-cluster-status-autoscale-decision-fields.md`.
   - [x] Prioritized autoscale evaluation when saturation is pending:
     - autoscale loop now bypasses the normal `autoscale_check_ms` gate when saturation events are pending, so saturation-driven scale decisions can run at maintenance-loop cadence,
     - preserves existing autoscale cadence when saturation is absent.
     - short LASM cluster probe after this slice recorded `1,470,366` requests in `20s` (`~73.15k req/s`, `p99 9.64ms`, peak RSS `~27,216 KB`) under default settings.
     - documented in `docs/book/1157-m39-lasm-cluster-saturation-priority-autoscale-eval.md`.
   - [x] Added saturation-priority autoscale sleep interval:
     - autoscale loop now shortens sleep cadence to `min(maintenance_interval_ms, 100ms)` while saturation is pending, while keeping baseline maintenance cadence when saturation is absent,
     - this reduces autoscale reaction latency during sustained saturation without changing non-saturation loop cadence.
     - short LASM cluster probe after this slice recorded `1,476,018` requests in `20s` (`~73.43k req/s`, `p99 9.39ms`, peak RSS `~27,104 KB`) under default settings.
     - documented in `docs/book/1158-m39-lasm-cluster-saturation-priority-sleep-interval.md`.
   - [x] Added autoscale cooldown-remaining telemetry fields to cluster status:
     - status payload now includes `autoscaleScaleUpCooldownRemainingMs` and `autoscaleScaleDownCooldownRemainingMs`,
     - autoscale loop publishes these values each cycle from current cooldown state so operators can see when next scale-up/scale-down windows open.
     - documented in `docs/book/1159-m39-lasm-cluster-status-autoscale-cooldown-remaining-fields.md`.
   - [x] Added active-connection density telemetry to cluster status:
     - status payload now includes `activeConnectionsPerWorker` derived from live `activeConnections / workerCount`,
     - improves visibility of per-worker load density during tuning and autoscale diagnosis.
     - documented in `docs/book/1160-m39-lasm-cluster-status-active-connections-per-worker.md`.
   - [x] Reduced relay-loop arithmetic and membership-check overhead on hot paths:
     - accept-loop relay dispatch now computes modulo once per batch and advances dispatch start index via wrap increment per stream (instead of modulo per stream),
     - unhealthy-port prune membership now uses `binary_search` on the worker-port snapshot (snapshot order guarded before publish) instead of linear `contains` checks.
     - documented in `docs/book/1161-m39-lasm-cluster-hotpath-dispatch-wrap-and-prune-binary-search.md`.
   - [x] Reduced relay backend-selection per-connection index/modulo overhead:
     - relay worker backend-selection reservation now tracks wrapped `next_index` per worker-port snapshot and refreshes modulo only when reservation or worker-port cardinality changes,
     - unhealthy-port fallback candidate scan now uses split-slice iteration (`tail` then `head`) from the selected start index instead of modulo/index arithmetic in each scan step.
     - documented in `docs/book/1162-m39-lasm-cluster-relay-selection-wrap-and-split-scan.md`.
   - [x] Added relay per-connection buffer reuse in worker pumps:
     - relay workers now keep a bounded local buffer pool (`relay_accept_batch_max * 4`, minimum `64`) and reuse relay pump buffers across connection lifecycles,
     - avoids fresh dual-buffer allocation on every connection when reusable buffers are available while keeping bounded-memory behavior.
     - documented in `docs/book/1163-m39-lasm-cluster-relay-buffer-pool-reuse.md`.
   - [x] Tuned LASM accept/relay idle backoff cadence for lower burst wake latency:
     - replaced hardcoded idle thresholds/sleeps with shared constants (`LASM_CLUSTER_IDLE_SPIN_THRESHOLD`, `LASM_CLUSTER_IDLE_SLEEP_MICROS`),
     - accept and relay loops now use bounded short micro-sleep (`250us`) after spin threshold instead of `1ms` sleeps, reducing wake-up latency while preserving busy-loop protection.
     - documented in `docs/book/1164-m39-lasm-cluster-idle-backoff-microsleep-tuning.md`.
   - [x] Added relay-dispatch single-sender fast path:
     - `dispatch_lasm_cluster_relay_stream` now short-circuits to direct `try_send` handling when relay sender count is `1`,
     - avoids split-slice dispatch iteration overhead for single-relay-worker deployments while preserving saturated/unavailable error semantics.
     - documented in `docs/book/1165-m39-lasm-cluster-relay-dispatch-single-sender-fast-path.md`.
   - [x] Precomputed per-start-index backend-selection lookup for relay workers:
     - relay worker loop now rebuilds a `start_index -> selected backend index` lookup table only when worker snapshot or unhealthy-port set changes,
     - per-connection backend selection now uses O(1) lookup with existing selection-counter sequencing, preserving unhealthy-skip semantics while removing per-connection fallback scans.
     - documented in `docs/book/1166-m39-lasm-cluster-relay-selection-precomputed-lookup.md`.
   - [x] Cached worker backend socket addresses per snapshot in relay workers:
     - relay workers now rebuild backend `SocketAddr` vector only when worker-port snapshot changes and reuse cached addresses for connect calls,
     - removes per-connection `SocketAddr` construction on the relay connect hot path while preserving existing backend-port/error semantics.
     - documented in `docs/book/1167-m39-lasm-cluster-relay-backend-socketaddr-cache.md`.
   - [x] Reduced per-connection snapshot/lookup plumbing in relay batch processing:
     - relay worker batch loop now refreshes worker snapshot/lookup state only on batch setup (`worker_ports_snapshot` missing) or when lookup is marked dirty,
     - removes repeated pointer/lookup checks on every accepted connection within the same batch while preserving selection and unhealthy-prune semantics.
     - documented in `docs/book/1168-m39-lasm-cluster-relay-batch-snapshot-lookup-setup.md`.
   - [x] Pruned stale relay connect-warning entries during topology refresh:
     - relay unhealthy-prune path now drops `connect_warning_next_allowed` entries for ports no longer present in worker snapshot and clears map when no workers remain,
     - keeps warning-throttle map bounded to active worker ports during autoscale/topology churn.
     - documented in `docs/book/1169-m39-lasm-cluster-connect-warning-map-prune.md`.
   - [x] Reduced backend-selection lookup rebuild complexity to linear time:
     - `rebuild_lasm_cluster_backend_selection_lookup` now computes a per-index healthy mask once and fills lookup via reverse next-healthy propagation (`O(n)`),
     - replaces previous scan-per-start-index fallback rebuild (`O(n^2)`) while preserving unhealthy-skip selection semantics.
     - documented in `docs/book/1170-m39-lasm-cluster-selection-lookup-linear-rebuild.md`.
   - [x] Removed rebuild-time healthy-mask allocation churn in relay selection lookup:
     - relay workers now keep a reusable `selection_healthy_mask` scratch buffer and pass it into lookup rebuild helper,
     - avoids fresh `Vec<bool>` allocation during each topology/health lookup rebuild while preserving linear rebuild logic and selection behavior.
     - documented in `docs/book/1171-m39-lasm-cluster-selection-lookup-scratch-mask-reuse.md`.
   - [x] Added no-healthy fast gate for relay backend selection:
     - lookup rebuild now returns `selection_has_healthy_backends`, and relay selection short-circuits when no healthy backend exists,
     - avoids selection-counter reservation/index arithmetic in known no-healthy states while preserving existing `503 no healthy workers` behavior.
     - documented in `docs/book/1172-m39-lasm-cluster-no-healthy-selection-fast-gate.md`.
   - [x] Tightened relay dispatch sender-iteration hot path:
     - `dispatch_lasm_cluster_relay_stream` now uses direct slice loops (`start_index == 0` fast path, then tail/head loops) instead of iterator-chain traversal and disconnected-count tracking,
     - preserves saturated/unavailable routing semantics while reducing dispatch-path iterator/counter overhead.
     - documented in `docs/book/1173-m39-lasm-cluster-relay-dispatch-direct-slice-loops.md`.
   - [x] Added identity-selection fast path for healthy steady state:
     - lookup rebuild now reports whether selection mapping is identity (`selected_index = start_index`),
     - relay selection path uses direct start-index selection when mapping is identity, skipping lookup table reads in healthy steady-state traffic.
     - documented in `docs/book/1174-m39-lasm-cluster-selection-identity-fast-path.md`.
   - [x] Removed identity-state lookup table materialization:
     - lookup rebuild now skips `lookup.resize/fill` entirely when mapping is identity (all healthy backends),
     - rebuild gating now treats lookup length checks as non-required in identity mode, avoiding redundant table writes in healthy steady-state rebuilds.
     - documented in `docs/book/1175-m39-lasm-cluster-selection-identity-no-table-materialization.md`.
   - [x] Added worker-port membership set cache for prune paths:
     - relay workers now rebuild a `HashSet<u16>` membership cache on snapshot changes,
     - unhealthy-port and warning-map prune paths now use membership-set contains checks instead of per-entry snapshot binary searches.
     - documented in `docs/book/1176-m39-lasm-cluster-worker-port-membership-cache.md`.
   - [x] Bypassed accept-loop dispatch counter in single-relay mode:
     - `run_lasm_cluster_accept_loop` now uses dispatch-counter reservation only when relay sender count is greater than `1`,
     - single-relay/zero-relay paths now keep dispatch start fixed at `0`, avoiding unnecessary shared atomic `fetch_add` churn.
     - documented in `docs/book/1177-m39-lasm-cluster-single-relay-dispatch-counter-bypass.md`.
   - [x] Inlined single-sender dispatch in accept loop:
     - when relay sender count is exactly `1`, accept loop now dispatches directly via that sender (`try_send`) without calling the generic dispatch helper,
     - preserves saturated/unavailable mapping semantics while removing one hot-path function call layer in single-relay deployments.
     - documented in `docs/book/1178-m39-lasm-cluster-single-sender-accept-loop-inline-dispatch.md`.
   - [x] Switched selection-lookup storage to sentinel-index vector:
     - relay selection lookup now uses `Vec<usize>` with sentinel `usize::MAX` for “no backend” entries instead of `Vec<Option<usize>>`,
     - reduces lookup-entry option wrapping overhead while preserving unhealthy remap semantics and identity fast path behavior.
     - documented in `docs/book/1179-m39-lasm-cluster-selection-lookup-sentinel-storage.md`.
   - [x] Removed selection healthy-mask scratch vector from lookup rebuild:
     - selection lookup rebuild now derives `first_healthy_index` directly from `worker_ports` and writes sentinel lookup entries in reverse index order without building a parallel `Vec<bool>`,
     - avoids per-refresh healthy-mask clear/resize/fill work while preserving healthy/unhealthy remap behavior and identity fast path semantics.
     - documented in `docs/book/1180-m39-lasm-cluster-selection-lookup-maskless-rebuild.md`.
   - [x] Added explicit run-flag override for relay accept batch size:
     - `sec4 run` now accepts `--cluster-relay-accept-batch-max <n>` (LASM cluster-only) with deterministic zero-value and backend/cluster guard diagnostics,
     - runtime resolution now applies CLI override first, then `SEC4_RT_LASM_CLUSTER_RELAY_ACCEPT_BATCH_MAX`, then default.
     - documented in `docs/book/1181-m39-run-cluster-relay-accept-batch-max-flag.md`.
   - [x] Added explicit run-flag overrides for proxy worker connect timing:
     - `sec4 run` now accepts `--cluster-backend-connect-timeout-ms <n>` and `--cluster-backend-connect-cooldown-ms <n>` (LASM cluster-only) with deterministic zero-value and backend/cluster guard diagnostics,
     - runtime resolution now applies CLI overrides first, then `SEC4_RT_LASM_CLUSTER_BACKEND_CONNECT_TIMEOUT_MS` / `SEC4_RT_LASM_CLUSTER_BACKEND_CONNECT_COOLDOWN_MS`, then defaults.
     - documented in `docs/book/1182-m39-run-cluster-backend-connect-timing-flags.md`.
   - [x] Replaced relay unhealthy/warning port maps with index-aligned state vectors:
     - relay workers now track backend unhealthy/connect-warning cooldown state by worker index (`Vec<Option<Instant>>`) and remap that state on worker-topology snapshot changes,
     - selection lookup rebuild now consumes index-aligned unhealthy state directly, and snapshot remap now reuses vector ownership (`mem::take`) instead of cloning cooldown state.
     - documented in `docs/book/1183-m39-lasm-cluster-relay-index-aligned-unhealthy-state.md`.
   - [x] Exposed relay backend connect timing in cluster status telemetry:
     - cluster status JSON now includes `relayBackendConnectTimeoutMs` and `relayBackendConnectCooldownMs`,
     - these fields report the effective runtime connect-timeout/cooldown values used by relay workers.
     - documented in `docs/book/1184-m39-lasm-cluster-status-relay-backend-connect-timing-fields.md`.
   - [x] Added preferred-shard direct dispatch fast path for multi-relay accept:
     - accept loop now tries the preferred relay sender directly first, and only scans remaining relay shards on first-attempt failure,
     - fallback scan now iterates only remaining senders and preserves saturated/unavailable semantics.
     - documented in `docs/book/1185-m39-lasm-cluster-preferred-shard-direct-dispatch-fast-path.md`.
   - [x] Added relay dispatch fallback telemetry counter:
     - accept loop now increments a shared `relayDispatchFallbackTotal` counter whenever preferred-shard direct dispatch misses and fallback scanning is used,
     - cluster status JSON now includes this counter for operator-visible direct-hit vs fallback behavior tracking.
     - documented in `docs/book/1186-m39-lasm-cluster-relay-dispatch-fallback-telemetry.md`.
   - [x] Removed shared accept-loop dispatch atomic in cluster mode:
     - cluster accept loops now use per-accept-worker local dispatch cursors (seeded by accept worker index) instead of a shared `AtomicUsize` reservation counter,
     - preserves deterministic round-robin-per-worker dispatch behavior while removing shared atomic contention from accept hot path.
     - documented in `docs/book/1187-m39-lasm-cluster-per-accept-worker-local-dispatch-cursors.md`.
   - [x] Batched relay fallback telemetry counter flush in accept loops:
     - accept loops now accumulate fallback dispatch hits locally and flush aggregated totals to `relayDispatchFallbackTotal`,
     - avoids per-fallback atomic increments in fallback-heavy traffic while preserving monotonic telemetry semantics.
     - documented in `docs/book/1188-m39-lasm-cluster-batched-relay-fallback-telemetry-flush.md`.
   - [x] Added relay fallback rate telemetry in cluster status:
     - cluster status JSON now includes `relayDispatchFallbackPerSec` computed per status sample interval from fallback-total deltas,
     - keeps fallback miss pressure visible as both cumulative total and current rate.
     - documented in `docs/book/1189-m39-lasm-cluster-relay-dispatch-fallback-rate-telemetry.md`.
   - [x] Removed temporary accept-batch stream buffer from cluster accept hot path:
     - `run_lasm_cluster_accept_loop` now dispatches accepted streams immediately within the accept loop instead of push/clear/drain over a temporary `Vec<TcpStream>`,
     - preserves dispatch cursor ordering, fallback/saturation/unavailable handling, and batched counter flush semantics while removing per-iteration batch-buffer churn.
     - documented in `docs/book/1190-m39-lasm-cluster-streaming-accept-dispatch-no-batch-buffer.md`.
   - [x] Split single-relay and multi-relay accept dispatch paths:
     - `run_lasm_cluster_accept_loop` now chooses single-sender vs multi-sender dispatch loop once per iteration and avoids per-stream sender-count branching in the inner accept hot path,
     - preserved dispatch-result semantics by routing both paths through shared accept-dispatch result handling.
     - documented in `docs/book/1191-m39-lasm-cluster-split-single-vs-multi-relay-accept-dispatch.md`.
   - [x] Reduced accept-loop counter/flush overhead on steady-state path:
     - accept-loop per-batch counters (`listener_accepted_in_batch`, `listener_enqueued_local`, fallback-local total) now use direct `+= 1` updates on bounded hot-path counters,
     - end-of-iteration atomic flush helpers are now called only when local counters are non-zero, avoiding unconditional helper calls on the no-fallback/no-idle fast path.
     - documented in `docs/book/1192-m39-lasm-cluster-accept-loop-counter-and-conditional-flush-hotpath.md`.
   - [x] Tightened fallback relay sender scan to split-slice traversal:
     - `dispatch_lasm_cluster_relay_stream_fallback` now scans fallback relay senders with two linear slice passes (`start..end`, then `0..start`) bounded by remaining attempts,
     - removes per-iteration modulo/wrap index arithmetic while preserving fallback saturated/unavailable semantics and sender-attempt ordering.
     - documented in `docs/book/1193-m39-lasm-cluster-fallback-relay-scan-split-slice.md`.
   - [x] Removed fallback-start modulo from relay fallback dispatch path:
     - multi-relay accept dispatch now computes wrapped fallback start index once (`next shard or 0`) and passes it directly into fallback scan,
     - fallback scan now iterates fixed-size split spans (`start..start+len`, then `0..remaining`) without per-iteration remaining guards or modulo wrapping.
     - documented in `docs/book/1194-m39-lasm-cluster-fallback-start-prewrap-and-fixed-span-scan.md`.
   - [x] Reduced relay-worker loop counter/flush overhead:
     - relay-worker per-iteration local counters (`saturation_events_*_local`, `active_connection_decrements_local`) now use direct `+= 1` updates on bounded loop-local counters,
     - relay-worker end-of-iteration flush helpers now run only when local counters are non-zero.
     - documented in `docs/book/1195-m39-lasm-cluster-relay-worker-counter-and-conditional-flush-hotpath.md`.
   - [x] Removed remaining bounded saturating increments in accept/relay hot loops:
     - bounded local counters (`listener_idle_spins`, relay `accepted_in_batch`, relay `idle_spins`, and local unhealthy/prune counters) now use direct `+= 1` updates where values are bounded by loop controls/snapshot cardinality,
     - keeps semantics unchanged while trimming saturating arithmetic in the ingress/relay hot path.
     - documented in `docs/book/1196-m39-lasm-cluster-bounded-counter-direct-increments-hotpath.md`.
   - [x] Reduced relay selection-reservation arithmetic overhead:
     - relay selection-reservation counters (`relay_selection_reservation_offset`, `relay_selection_reservation_next_index`) now use direct bounded `+= 1` updates in the reservation hot path,
     - keeps reservation wrap behavior unchanged while removing saturating arithmetic from per-connection backend selection sequencing.
     - documented in `docs/book/1197-m39-lasm-cluster-relay-selection-reservation-direct-increments.md`.
   - [x] Removed per-connection closure from relay selection-reservation start-index path:
     - relay backend selection now computes reservation refill and start-index progression inline (no closure allocation/call in per-connection loop),
     - keeps reservation refill/wrap and identity-lookup behavior unchanged while simplifying the hot path.
     - documented in `docs/book/1198-m39-lasm-cluster-inline-relay-selection-reservation-start-index.md`.
   - [x] Simplified non-identity relay selection lookup read path:
     - relay backend selection now reads precomputed lookup entries via direct sentinel-index access instead of `get().copied().filter(...)`,
     - keeps sentinel semantics unchanged while removing option-chain overhead on the non-identity selection path.
     - documented in `docs/book/1199-m39-lasm-cluster-non-identity-selection-direct-lookup-read.md`.
   - [x] Removed success-path backend-port lookup in relay connect path:
     - relay worker now reads `selected_backend_port` only inside connect-failure warning path instead of unconditionally before each connect attempt,
     - keeps warning output unchanged while removing one success-path array read from the relay hot path.
     - documented in `docs/book/1200-m39-lasm-cluster-lazy-backend-port-read-on-connect-failure.md`.
   - [x] Switched relay connect-failure state access to direct index reads:
     - relay connect-failure path now updates unhealthy cooldown and warning-throttle vectors via direct index access (`[selected_backend_index]`) instead of repeated `get()/get_mut()` option chains,
     - preserves cooldown/throttle behavior while reducing option-layer overhead on the failure-path hot loop.
     - documented in `docs/book/1201-m39-lasm-cluster-direct-index-connect-failure-state-access.md`.
   - [x] Removed redundant fallback-start normalization branch:
     - fallback dispatch now consumes pre-wrapped start indexes directly (`scan_start_index = start_index_wrapped`) after existing debug invariant check,
     - removes one runtime normalization branch from relay fallback dispatch while preserving fallback scan behavior.
     - documented in `docs/book/1202-m39-lasm-cluster-fallback-scan-direct-prewrapped-start.md`.
   - [x] Tightened relay connect-warning throttle slot path:
     - relay connect-failure path now loads warning-throttle slot once into a mutable entry reference and performs direct match-based allowance check (`Some(next) => now >= next`),
     - keeps warning throttle semantics unchanged while removing repeated indexed option-chain reads/writes in the failure path.
     - documented in `docs/book/1203-m39-lasm-cluster-connect-warning-slot-direct-match-check.md`.
   - [x] Removed per-connection `Option` wrapping from relay backend selection result:
     - relay backend selection now uses `LASM_CLUSTER_SELECTION_LOOKUP_NONE` sentinel directly across identity and non-identity selection paths,
     - removes `Option` construction/unwrapping in per-connection selection while preserving no-healthy behavior and selection semantics.
     - documented in `docs/book/1204-m39-lasm-cluster-selection-sentinel-result-path.md`.
   - [x] Simplified relay init/pump warning-throttle checks:
     - relay init-failure and relay pump-failure warning gates now use direct `match`-based checks on `pump_warning_next_allowed` (`Some(next) => now >= next`, `None => true`) instead of map/unwrap option chains,
     - preserves warning-throttle behavior while reducing option-chain overhead in warning paths.
     - documented in `docs/book/1205-m39-lasm-cluster-relay-warning-throttle-direct-match-checks.md`.
   - [x] Removed remaining map/unwrap option-chain checks from relay/autoscale paths:
     - relay unhealthy-prune gate and unhealthy-slot marking now use direct `match` checks on option values instead of map/unwrap chains,
     - autoscale up/down cooldown checks now use direct `match`-based elapsed guards.
     - documented in `docs/book/1206-m39-lasm-cluster-relay-autoscale-direct-match-checks.md`.
   - [x] Reused precomputed duration values across relay/autoscale loops:
     - relay warning-throttle and unhealthy-prune intervals are now precomputed once per relay worker and reused on warning/cooldown updates,
     - status writer and autoscale loop now precompute interval/cooldown durations and reuse them for loop sleeps + cooldown checks.
     - documented in `docs/book/1207-m39-lasm-cluster-duration-reuse-in-relay-autoscale-loops.md`.
   - [x] Bypassed accept-dispatch helper on single-step success fast paths:
     - single-relay and multi-relay accept loops now update `listener_enqueued_local` directly on immediate `try_send` success,
     - helper-based dispatch handling remains on saturation/unavailable fallback paths only.
     - documented in `docs/book/1208-m39-lasm-cluster-accept-fast-path-direct-enqueue.md`.
   - [x] Added relay worker vector preallocation for steady-state capacities:
     - relay connection and relay buffer-pool vectors are now initialized with capacity derived from `relay_accept_batch_max` / `relay_buffer_pool_max`,
     - selected backend address vector now preallocates to current worker-port snapshot length before initial rebuild.
     - documented in `docs/book/1209-m39-lasm-cluster-relay-vector-preallocation.md`.
   - [x] Added precomputed relay next-index lookup for multi-relay accept dispatch:
     - multi-relay accept loop now precomputes sender-next indexes once and reuses them for dispatch-cursor advancement and fallback start selection,
     - removes per-request wrap branches for cursor/fallback index progression.
     - documented in `docs/book/1210-m39-lasm-cluster-accept-next-index-lookup.md`.
   - [x] Added precomputed relay selection next-index lookup for backend reservation progression:
     - relay worker backend selection now precomputes worker-next indexes when worker cardinality changes and reuses them to advance reservation indexes,
     - removes per-selection wrap branches from reservation progression while preserving selection order semantics.
     - documented in `docs/book/1211-m39-lasm-cluster-relay-selection-next-index-lookup.md`.
   - [x] Reused one computed next-dispatch index per accepted stream in multi-relay accept path:
     - accept dispatch now computes `next_dispatch_index` once and reuses it for both cursor advancement and fallback-start selection,
     - removes duplicate next-index lookup work from the multi-relay accept hot path.
     - documented in `docs/book/1212-m39-lasm-cluster-accept-next-index-single-read.md`.
   - [x] Restricted accept-dispatch helper handling to error outcomes only:
     - fallback dispatch now increments enqueue-local counters directly on `Ok(())` without routing through the dispatch helper,
     - accept dispatch helper now handles only saturated/unavailable error outcomes.
     - documented in `docs/book/1213-m39-lasm-cluster-accept-fallback-success-direct-enqueue.md`.
   - [x] Added explicit inline hints on tight cluster helper paths:
     - counter flush helpers, fallback dispatch scan helper, and accept-dispatch error helper now carry `#[inline(always)]`,
     - keeps helper decomposition while reducing call overhead risk in release hot paths.
     - documented in `docs/book/1214-m39-lasm-cluster-inline-hints-on-hot-helpers.md`.
   - [x] Switched fallback relay scan traversal to next-index lookup iteration:
     - fallback dispatch now consumes the precomputed sender next-index lookup and iterates by fixed attempt count,
     - removes split-slice scan arithmetic from fallback traversal while preserving sender-attempt ordering.
     - documented in `docs/book/1215-m39-lasm-cluster-fallback-next-index-iteration.md`.
   - [x] Precomputed idle-sleep durations in accept/relay loops:
     - accept and relay loops now allocate idle sleep durations once and reuse them instead of rebuilding microsecond durations per idle cycle,
     - keeps idle backoff behavior unchanged while trimming repeated duration construction in hot loops.
     - documented in `docs/book/1216-m39-lasm-cluster-idle-sleep-duration-reuse.md`.
   - [x] Specialized fallback scan helper for multi-relay mode:
     - fallback dispatch helper is now explicitly multi-relay (`sender_count > 1`) and no longer carries single-sender guard branching,
     - accept multi-relay path now calls the specialized helper directly.
     - documented in `docs/book/1217-m39-lasm-cluster-multi-relay-fallback-helper-specialization.md`.
   - [x] Reduced unchanged status-writer disk churn in cluster mode:
     - status writer now skips atomic status JSON rewrites when all status fields are unchanged (excluding `updatedAtMs`),
     - status file updates now occur only on meaningful status-snapshot changes.
     - documented in `docs/book/1218-m39-lasm-cluster-status-writer-unchanged-snapshot-skip.md`.
   - [x] Added cluster status-writer unchanged-snapshot contract coverage:
     - command integration test `run_command_lasm_cluster_status_json_skips_unchanged_snapshots` now locks stable `updatedAtMs` behavior for unchanged status snapshots,
     - prevents regressions that would reintroduce periodic unchanged status-file rewrites.
     - documented in `docs/book/1219-m39-lasm-cluster-status-writer-unchanged-snapshot-test-lock.md`.
   - [x] Replaced status-writer baseline-byte comparison with typed snapshot comparison:
     - status writer now compares a typed in-memory `LasmClusterStatusSnapshot` (including worker port set + autoscale/relay metrics) to detect unchanged state,
     - removes per-interval baseline JSON encoding from unchanged-snapshot detection while preserving unchanged-write skip semantics.
     - documented in `docs/book/1220-m39-lasm-cluster-status-writer-typed-snapshot-compare.md`.
   - [x] Removed status-snapshot worker-port cloning via shared worker-port arcs:
     - `LasmClusterStatusSnapshot` now stores worker ports as `Arc<Vec<u16>>` and status writer passes `load_full()` worker-port snapshots directly,
     - removes per-interval worker-port vector cloning from unchanged-snapshot comparison.
     - documented in `docs/book/1221-m39-lasm-cluster-status-snapshot-worker-port-arc-reuse.md`.
   - [x] Switched status JSON encoding path to typed payload serialization:
     - status writer now serializes a typed `LasmClusterStatusPayload` struct (serde rename rules) instead of building dynamic JSON maps via `serde_json::json!`,
     - preserves status JSON field contract while reducing dynamic payload construction overhead.
     - documented in `docs/book/1222-m39-lasm-cluster-status-writer-typed-payload-serialization.md`.
   - [x] Unified relay accept `try_send` error handling branches:
     - single-relay accept path now maps `TrySendError::{Full,Disconnected}` through one shared error-mapping branch before dispatch-error handling,
     - multi-relay accept path now uses one shared `TrySendError` branch to drive fallback dispatch with derived `saw_live_sender`.
     - documented in `docs/book/1223-m39-lasm-cluster-accept-try-send-error-branch-unification.md`.
   - [x] Switched changed-snapshot status writes to streamed temp-file serialization:
     - status writer now writes changed payloads via `serde_json::to_writer` into a buffered temp file and flushes before atomic rename,
     - removes changed-write `to_vec` allocation/copy path while preserving atomic file replacement semantics.
     - documented in `docs/book/1224-m39-lasm-cluster-status-writer-streamed-tempfile-serialization.md`.
   - [x] Added cached status-parent directory readiness with `NotFound` recovery:
     - status writer now caches successful parent-directory readiness and avoids repeated `create_dir_all` on each changed write,
     - tempfile creation now retries parent-dir initialization when `NotFound` occurs (for parent-dir removal recovery).
     - documented in `docs/book/1225-m39-lasm-cluster-status-parent-readiness-cache.md`.
   - [x] Removed changed-write status snapshot clone in writer handoff:
     - status writer now accepts `LasmClusterStatusSnapshot` by value and moves it into `last_snapshot` after a successful write,
     - removes per-write snapshot clone overhead while preserving unchanged-snapshot skip and status-file contract behavior.
     - documented in `docs/book/1226-m39-lasm-cluster-status-snapshot-move-handoff.md`.
   - [x] Added single-backend relay dispatch fast path:
     - relay worker selection now short-circuits to backend index `0` when only one worker port is available,
     - skips reservation-counter `fetch_add` and next-index rotation bookkeeping in the single-backend case while preserving selection semantics for multi-backend clusters.
     - documented in `docs/book/1227-m39-lasm-cluster-single-backend-selection-fast-path.md`.
   - [x] Removed relay next-index lookup vector from round-robin dispatch:
     - relay worker selection now advances round-robin start index with direct wrapped arithmetic (`start + 1` with wrap) instead of prebuilding/reading a per-worker next-index vector,
     - removes worker-port-change vector rebuild overhead and per-dispatch lookup reads while preserving selection order semantics.
     - documented in `docs/book/1228-m39-lasm-cluster-direct-round-robin-next-index.md`.
   - [x] Removed accept-loop sender next-index lookup vector and unified wrapped index progression helper:
     - accept loop multi-relay dispatch now computes next relay sender index via `lasm_cluster_next_index_wrapped(...)` instead of a prebuilt sender next-index vector,
     - fallback relay scan now also advances with the same wrapped helper, removing per-accept-loop sender-vector setup and per-step lookup reads while preserving relay scan order.
     - documented in `docs/book/1229-m39-lasm-cluster-accept-fallback-direct-next-index-helper.md`.
   - [x] Cached selected worker-port count in relay dispatch loop:
     - relay worker loop now maintains `selected_worker_port_count` alongside the selected worker-port snapshot and updates it only when snapshot identity changes,
     - selection rebuild and backend-index selection now use the cached count instead of repeated `selected_worker_ports_snapshot.as_ref().len()` reads in the hot path.
     - documented in `docs/book/1230-m39-lasm-cluster-cached-selected-worker-port-count.md`.
   - [x] Reduced relay worker-port snapshot replacement clone churn:
     - relay worker snapshot-update branches now use `std::mem::replace` to move out the previous selected snapshot while installing the new snapshot, instead of cloning both old/new arcs,
     - keeps unhealthy-port remap semantics unchanged while reducing reference-count update overhead on worker-port snapshot refreshes.
     - documented in `docs/book/1231-m39-lasm-cluster-worker-port-snapshot-replace-handoff.md`.
   - [x] Removed redundant connect-failure warning index read and unreachable reservation branch:
     - relay worker connect-failure warning now uses already-selected `backend_addr.port()` instead of re-indexing selected worker-port snapshots,
     - relay reservation refill in multi-backend path now assigns `base % worker_port_count` directly (removing an unreachable `worker_port_count <= 1` branch).
     - documented in `docs/book/1232-m39-lasm-cluster-connect-warning-port-and-reservation-branch-simplification.md`.
   - [x] Added trivial-worker-count selection-lookup bypass in relay loop:
     - relay loop now skips `rebuild_lasm_cluster_backend_selection_lookup(...)` when selected worker-port count is `0` or `1`,
     - for those trivial counts, it sets deterministic healthy/identity flags directly and clears lookup storage, avoiding unnecessary lookup rebuild calls in the hot path.
     - documented in `docs/book/1233-m39-lasm-cluster-trivial-worker-count-selection-lookup-bypass.md`.
   - [x] Added healthy-state selection-lookup rebuild bypass:
     - relay loop now short-circuits selection state when `unhealthy_port_count == 0` (for any non-zero worker count), setting healthy/identity flags directly and clearing lookup storage,
     - `rebuild_lasm_cluster_backend_selection_lookup(...)` now runs only for non-trivial unhealthy multi-backend states.
     - documented in `docs/book/1234-m39-lasm-cluster-healthy-state-selection-lookup-bypass.md`.
   - [x] Removed duplicate outer flush guards in accept/relay loops:
     - accept loop and relay worker loop now call inline flush helpers unconditionally each cycle,
     - relies on existing helper-local zero checks to skip atomic updates, removing duplicate outer branch checks from hot loop tails.
     - documented in `docs/book/1235-m39-lasm-cluster-flush-guard-dedup.md`.
   - [x] Added all-unhealthy fast-fail in backend selection lookup rebuild:
     - `rebuild_lasm_cluster_backend_selection_lookup` now short-circuits when `unhealthy_port_count >= worker_port_count`,
     - avoids unnecessary unhealthy-index scan work when no healthy backend exists.
     - documented in `docs/book/1236-m39-lasm-cluster-selection-lookup-all-unhealthy-fast-fail.md`.
   - [x] Removed per-iteration optional-index reads from selection lookup rebuild:
     - lookup rebuild now reads unhealthy markers with direct indexed access (`unhealthy_ports_until_by_index[index]`) and a debug precondition on slice length,
     - removes repeated bounds/option combinator overhead in selection lookup scan loops.
     - documented in `docs/book/1237-m39-lasm-cluster-selection-lookup-direct-index-read.md`.
   - [x] Added pointer-aware status snapshot equality for worker-port sets:
     - `LasmClusterStatusSnapshot` now uses manual `PartialEq` with `Arc::ptr_eq` short-circuit for `worker_ports` before slice fallback comparison,
     - preserves unchanged-snapshot semantics while reducing repeated deep vector comparisons when worker-port snapshot pointer is unchanged.
     - documented in `docs/book/1238-m39-lasm-cluster-status-snapshot-pointer-aware-equality.md`.
   - [x] Replaced over-scaled auto relay-worker sizing with bounded instance-based heuristic:
     - default relay worker auto-sizing no longer scales with `target_connections_per_instance` (which produced excessive defaults like `256` workers for `4..8` instance configs),
     - new auto heuristic uses bounded sublinear instance hint (`ceil(sqrt(max(min_instances, max_instances)))`, min `2` for multi-instance, clamped by host parallelism and `1..16` hard bound).
     - short capacity probe (`20s`, `8t/256c`, `/health`) improved from `~68.6k req/s` (`p99 11.99ms`) to `~75.5k req/s` (`p99 6.41ms`) under default auto relay settings with the same config envelope.
     - documented in `docs/book/1239-m39-lasm-cluster-auto-relay-worker-sizing-fix.md`.
   - [x] Tuned auto relay-worker default from ceil-sqrt to floor-sqrt for multi-instance mode:
     - auto relay sizing now uses `max(2, floor(sqrt(max(min_instances, max_instances))))` for multi-instance configs (single-instance remains `1`),
     - keeps existing host-parallelism and `1..16` clamps unchanged while biasing lower default relay thread counts to reduce hot-path contention.
     - short capacity probe (`20s`, `8t/256c`, `/health`) improved from `~75.5k req/s` (`p99 6.41ms`) to `~77.8k req/s` (`p99 5.35ms`) under default auto relay settings with the same config envelope.
     - documented in `docs/book/1240-m39-lasm-cluster-auto-relay-worker-floor-sqrt-tuning.md`.
   - [x] Extended LASM cluster capacity probe with accept-path override controls:
     - `run_lasm_cluster_capacity_probe.sh` now supports explicit `--cluster-accept-workers` and `--cluster-relay-accept-batch-max` flags (plus env counterparts),
     - probe dry-run output + summary JSON now include both fields for deterministic tuning artifact capture.
     - documented in `docs/book/1241-m39-lasm-cluster-capacity-probe-accept-worker-batch-overrides.md`.
   - [x] Forwarded accept-path tuning overrides through saturation/full-suite orchestration:
     - saturation matrix + bundle scripts now accept/pass `--cluster-accept-workers` and `--cluster-relay-accept-batch-max`,
     - full benchmark suite and Makefile saturation targets now thread matching saturation flags/variables end-to-end.
     - documented in `docs/book/1242-m39-lasm-saturation-suite-accept-path-override-forwarding.md`.
   - [x] Added resolved cluster-status telemetry capture in capacity probe artifacts:
     - `run_lasm_cluster_capacity_probe.sh` now writes a per-run `--cluster-status-json` artifact and records resolved relay/accept/queue runtime fields in probe summary JSON (`*Resolved` run fields),
     - keeps requested vs resolved relay settings visible in one deterministic probe artifact for tuning loops.
     - documented in `docs/book/1243-m39-lasm-capacity-probe-cluster-status-resolved-fields.md`.
   - [x] Upgraded saturation matrix/analyzer output with latency tie-break metadata:
     - matrix run items now propagate probe `p99` plus resolved relay/accept/queue fields from per-step probe summaries,
     - analyzer now normalizes `p99` durations (`us`/`ms`/`s`) into numeric `p99Ms` and uses it as a ranking tie-break after pass/throughput ordering.
     - documented in `docs/book/1244-m39-lasm-saturation-analysis-p99-and-resolved-metadata.md`.
   - [x] Added capacity-probe status artifact cleanup by default:
     - `run_lasm_cluster_capacity_probe.sh` now cleans per-run status-json artifacts unless `--keep-cluster-status-json` is set,
     - dry-run plan/test flow now makes keep-mode explicit for deterministic operator usage.
     - documented in `docs/book/1245-m39-lasm-capacity-probe-status-artifact-cleanup-default.md`.
   - [x] Upgraded saturation summary markdown with p99 + resolved runtime columns:
     - ranked runs table, recommendation line, and verification section now expose p99 and resolved relay/accept/queue fields directly from analysis/probe artifacts,
     - keeps top-level operator report actionable without manual raw JSON inspection.
     - documented in `docs/book/1246-m39-lasm-saturation-summary-p99-resolved-columns.md`.
   - [x] Added hybrid relay-pump scheduling for high-connection proxy workloads:
     - relay workers now keep the existing full-scan pump loop when active relay connections are within a bounded threshold (`<= relay_pump_batch_max`), preserving the fast path for common steady-state loads,
     - when active relay connections exceed that threshold, workers switch to a cursor-based capped pump budget to bound per-loop scan cost and reduce O(n) hot-loop pressure under high keep-alive fan-in,
     - short capacity probes (`20s`, `8t/256c`, `/health`) remained stable in the current range (`~75.8k req/s`, repeat `~75.4k req/s`) while keeping deterministic pass behavior,
     - documented in `docs/book/1247-m39-lasm-cluster-relay-hybrid-pump-scheduling.md`.
   - [x] Added explicit relay-pump batch override control for LASM cluster tuning:
     - `sec4 run --backend lasm` now accepts `--cluster-relay-pump-batch-max <n>` (cluster-proxy mode only) with deterministic LASM-only / lower-bound / cluster-mode / fixed-reuse-port guard diagnostics,
     - runtime status telemetry now reports `relayPumpBatchMax` so resolved runtime behavior is visible in status snapshots,
     - capacity probe script now supports `--cluster-relay-pump-batch-max` (and env counterpart), includes requested/resolved pump-batch values in summary artifacts, and keeps dry-run contract visibility.
     - documented in `docs/book/1248-m39-lasm-cluster-relay-pump-batch-override-and-telemetry.md`.
   - [x] Forwarded relay pump-batch override through saturation/full-suite orchestration:
     - saturation matrix and bundle scripts now accept/pass `--cluster-relay-pump-batch-max`,
     - full benchmark suite + Makefile saturation targets now forward `--saturation-cluster-relay-pump-batch-max` / `LASM_CAPACITY_CLUSTER_RELAY_PUMP_BATCH_MAX`,
     - dry-run contract tests for matrix/bundle/full-suite/make presets now assert relay pump-batch visibility end-to-end.
     - documented in `docs/book/1249-m39-lasm-saturation-suite-relay-pump-batch-forwarding.md`.
   - [x] Exposed relay pump-batch resolved telemetry in saturation analysis/summary artifacts:
     - analyzer now carries `clusterRelayPumpBatchMaxResolved` through ranked runs,
     - markdown summary probe profile/ranked table/recommendation/verify sections now report relay pump-batch requested/resolved values.
     - documented in `docs/book/1250-m39-lasm-saturation-summary-relay-pump-batch-columns.md`.
   - [x] Added fixed reuse-port mode support across LASM capacity/saturation orchestration:
     - `run_lasm_cluster_capacity_probe.sh` now supports `--fixed-reuse-port-mode` with deterministic guardrails (forces `autoscaleMaxInstances=instances`, rejects relay-proxy-only flags, suppresses cluster-status artifact wiring),
     - saturation matrix + bundle + full-suite + Makefile forwarding now thread fixed-mode toggles end-to-end,
     - short local sample (`20s`, `8t/256c`, `/health`) measured `~120.5k req/s` in fixed reuse-port mode vs `~77.6k req/s` proxy-relay mode in this environment.
     - documented in `docs/book/1251-m39-lasm-fixed-reuse-port-probe-and-saturation-forwarding.md`.
   - [x] Added deterministic proxy-vs-fixed LASM mode comparison runner:
     - new `run_lasm_cluster_mode_compare.sh` runs paired proxy-relay and fixed-reuse-port probes under one workload profile and emits a combined comparison artifact (`recommendedMode`, throughput delta/gain, latency + memory snapshot),
     - Makefile now includes `lasm-cluster-mode-compare` target and script contracts for dry-run comparison planning.
     - documented in `docs/book/1252-m39-lasm-cluster-mode-compare-runner.md`.
   - [x] Wired optional LASM mode-compare lane into full-suite orchestration:
     - `run_full_benchmark_suite.sh` now supports `--include-lasm-mode-compare` and runs `run_lasm_cluster_mode_compare.sh` as a dedicated phase with existing LASM saturation tuning overrides forwarded (`duration`, `threads`, `connections`, target requests, relay/accept/batch knobs),
     - deterministic guardrails now enforce `sec4-lasm` presence in `--impls` for mode-compare lane activation,
     - Makefile full-suite targets (`bench-full*`, `bench-full-saturation*`) now forward `FULL_LASM_INCLUDE_MODE_COMPARE=true` via `--include-lasm-mode-compare`.
     - documented in `docs/book/1253-m39-lasm-full-suite-optional-mode-compare-lane.md`.
   - [x] Added mode-compare section rendering in benchmark markdown reports:
     - `publish_report.sh` now accepts optional `mode_compare.json` and renders `LASM Mode Comparison` summary lines (recommended mode, pass statuses, req/s delta/gain, p99 split, peak RSS delta),
     - `run_full_benchmark_suite.sh` now forwards the mode-compare artifact into `publish_report.sh` when `--include-lasm-mode-compare` is enabled, so one top-level full-suite run emits both artifacts and report summary.
     - documented in `docs/book/1256-m39-benchmark-report-mode-compare-section.md`.
   - [x] Added relay accept-loop saturation short-circuit for fallback dispatch scans:
     - in multi-sender proxy mode, when one fallback scan confirms all relay sender shards are saturated for the current accept batch, subsequent `TrySendError::Full` dispatches in that same batch skip redundant full fallback scans and immediately return deterministic saturation handling,
     - successful dispatches reset the batch-saturation hint so recovery back to normal fallback behavior stays immediate once capacity frees up.
     - documented in `docs/book/1254-m39-lasm-cluster-accept-fallback-saturation-short-circuit.md`.
   - [x] Added status-json telemetry for relay saturation short-circuit events:
     - cluster status snapshots now expose `relayDispatchSaturationShortCircuitTotal` and `relayDispatchSaturationShortCircuitPerSec` alongside existing dispatch fallback telemetry,
     - accept-loop short-circuit handling now increments and flushes a dedicated counter so saturation shortcut behavior is observable during probe tuning.
     - documented in `docs/book/1255-m39-lasm-cluster-short-circuit-status-telemetry.md`.
   - [x] Propagated relay short-circuit telemetry through benchmark artifacts and report summaries:
     - LASM capacity probe summaries now persist `clusterRelayDispatchSaturationShortCircuitTotal` and `clusterRelayDispatchSaturationShortCircuitPerSec` in `run.*` fields when cluster status telemetry is available,
     - saturation matrix/analyzer/summary scripts now carry and render short-circuit totals/per-sec values in ranked rows and verification sections,
     - mode-compare comparison JSON + markdown report section now include short-circuit total signals alongside throughput/latency deltas.
     - documented in `docs/book/1257-m39-lasm-short-circuit-telemetry-artifact-propagation.md`.
   - [x] Added relay-sender liveness tracking in accept-dispatch fallback path:
     - LASM cluster accept loop now tracks disconnected relay sender shards and skips known-dead shards when choosing primary dispatch targets and fallback scans,
     - fallback dispatch now marks disconnected shards as dead and short-circuits unavailable results when all shards are disconnected, reducing repeated `TrySendError::Disconnected` churn in degraded states.
     - documented in `docs/book/1258-m39-lasm-relay-shard-liveness-tracking.md`.
   - [x] Reduced backend-selection round-robin counter atomic contention in relay workers:
     - relay worker selection reservation now allocates a larger minimum chunk (`LASM_CLUSTER_SELECTION_RESERVATION_MIN_CHUNK`, 64) instead of always reserving only `relay_accept_batch_max`,
     - keeps deterministic round-robin reservation semantics while reducing `fetch_add` frequency on the shared `relay_selection_counter` hot path under load.
     - documented in `docs/book/1259-m39-lasm-relay-selection-reservation-chunk-sizing.md`.
   - [x] Tightened fallback scan budget to live relay shards:
     - multi-sender fallback dispatch now tracks scanned live shards and stops after it has attempted all remaining live non-primary shards (`relay_live_sender_count - 1`),
     - avoids wasting fallback-loop iterations on known-dead shards while preserving saturated vs unavailable error semantics.
     - documented in `docs/book/1260-m39-lasm-fallback-live-shard-scan-budget.md`.
   - [x] Added all-live relay pool fast path for primary dispatch selection:
     - multi-sender accept dispatch now bypasses live-shard lookup scans while all relay sender shards are known live and uses direct cursor dispatch,
     - runtime flips to live-scan mode only after first observed relay sender disconnection (`TrySendError::Disconnected`), preserving degraded-state correctness while trimming healthy-path dispatch overhead.
     - documented in `docs/book/1261-m39-lasm-all-live-relay-fast-path.md`.
   - [x] Added two-sender fallback fast path in relay dispatch:
     - `dispatch_lasm_cluster_relay_stream_fallback_multi` now handles the common `sender_count == 2` case with a direct alternate-shard `try_send` path,
     - preserves disconnected-shard liveness updates and saturated/unavailable mapping while avoiding generic fallback scan-loop overhead in two-shard clusters.
     - documented in `docs/book/1262-m39-lasm-two-sender-fallback-fast-path.md`.
   - [x] Added degraded-mode live-cursor realignment and zero-live fast-fail in accept dispatch:
     - multi-sender accept dispatch now short-circuits to deterministic unavailable handling when `relay_live_sender_count == 0` instead of running repeated live-scan lookups,
     - after fallback/primary disconnect events, dispatch cursor now realigns to a live shard once (when possible) so subsequent degraded-path dispatch avoids repeated full live-index scans.
     - documented in `docs/book/1263-m39-lasm-degraded-live-cursor-realignment.md`.
   - [x] Added single-live-shard degraded dispatch fast path:
     - when degraded mode has exactly one live relay shard (`relay_live_sender_count == 1`), accept dispatch now reuses cached `relay_single_live_sender_index` and pins dispatch cursor directly to it,
     - avoids repeated generic live-shard scan lookups on each request in single-live-shard degraded states while preserving deterministic unavailable behavior if no live shard remains.
     - documented in `docs/book/1264-m39-lasm-single-live-shard-dispatch-fast-path.md`.
   - [x] Replaced relay-shard liveness bitset with byte flags in dispatch hot path:
     - multi-sender liveness tracking now uses `Vec<u8>` (`LIVE=1` / `DEAD=0`) instead of `Vec<bool>` specialized bitset storage for relay dispatch and fallback checks,
     - keeps liveness semantics unchanged while avoiding `Vec<bool>` proxy/bitset overhead in tight accept/fallback loops.
     - documented in `docs/book/1265-m39-lasm-relay-live-byte-flag-tracking.md`.
   - [x] Added relay live-shard telemetry to LASM cluster status snapshots:
     - status writer now includes `relayLiveSenderCount` in cluster status JSON payloads so relay-shard degradation is directly observable during load/probe runs,
     - value is sourced from shared accept-loop liveness counters and exposed alongside existing relay dispatch fallback/short-circuit telemetry.
     - documented in `docs/book/1266-m39-lasm-cluster-status-relay-live-shard-telemetry.md`.
   - [x] Propagated relay live-shard telemetry through LASM benchmark artifacts:
     - capacity probe summaries now include `run.clusterRelayLiveSenderCountResolved` from cluster status snapshots,
     - saturation matrix/analyzer/summary and mode-compare/report outputs now thread and render relay live-shard metrics for operator comparison.
     - documented in `docs/book/1267-m39-lasm-live-shard-telemetry-artifact-propagation.md`.
   - [x] Added all-live fast path to fallback dispatch scan loop:
     - `dispatch_lasm_cluster_relay_stream_fallback_multi` now bypasses dead-shard checks while relay pool is known healthy and iterates direct fallback sends for `sender_count > 2`,
     - degraded dead-shard-aware scan path remains active once any disconnection is observed.
     - documented in `docs/book/1268-m39-lasm-fallback-all-live-scan-fast-path.md`.
   - [x] Elided redundant live-hint refresh and relay-live telemetry atomics after fallback dispatch:
     - accept-loop fallback handling now tracks live-shard count before fallback and refreshes single/dual-live hints plus `relay_live_sender_count` telemetry only when fallback actually changes live-shard count,
     - removes repeated live-hint scans and `fetch_min` atomics on saturated-but-live fallback paths while preserving degraded/disconnect correctness.
     - documented in `docs/book/1269-m39-lasm-accept-fallback-live-hint-refresh-elision.md`.
   - [x] Reused cached degraded-mode single/dual-live sender hints in accept dispatch:
     - when degraded mode has one or two live relay shards, accept dispatch now reuses cached `relay_single_live_sender_index` / `relay_dual_live_sender_indices` and refreshes only when missing,
     - avoids repeated per-request hint rescans in degraded steady state while preserving deterministic unavailable fallback behavior if cached hints are absent.
     - documented in `docs/book/1270-m39-lasm-degraded-single-dual-hint-cache-reuse.md`.
   - [x] Fixed single-live degraded fallback dispatch in multi-sender relay pools:
     - `dispatch_lasm_cluster_relay_stream_fallback_multi` now handles `scan_live_target == 0` by attempting the remaining live shard once before returning deterministic saturated/unavailable errors,
     - prevents false immediate unavailable outcomes when exactly one live shard remains in pools with more than two relay senders.
     - documented in `docs/book/1271-m39-lasm-single-live-degraded-fallback-dispatch-fix.md`.
   - [x] Added cached-hint fallback dispatch fast paths for degraded single/dual live states:
     - accept-loop fallback selection now refreshes missing single/dual live hints once and prefers dedicated `dispatch_lasm_cluster_relay_stream_fallback_single_live` / `dispatch_lasm_cluster_relay_stream_fallback_dual_live` paths before generic fallback scans,
     - reduces degraded fallback scan overhead while preserving deterministic saturated/unavailable envelopes and liveness updates.
     - documented in `docs/book/1272-m39-lasm-degraded-fallback-cached-hint-fast-paths.md`.
   - [x] Elided redundant no-live telemetry atomic updates in accept loop:
     - when `relay_live_sender_count == 0`, accept-loop dispatch now skips repeated per-request `relay_live_sender_count_observed.fetch_min(0)` writes,
     - live-count telemetry remains correct because transition-to-zero is already recorded when liveness changes.
     - documented in `docs/book/1273-m39-lasm-no-live-telemetry-atomic-elision.md`.
   - [x] Deduplicated fallback live-count telemetry atomics on primary disconnect path:
     - accept-loop fallback now records primary relay-sender disconnect live-count changes through the existing end-of-fallback live-count delta branch, instead of issuing an immediate extra `fetch_min` before fallback completion,
     - keeps live telemetry semantics unchanged while removing duplicate disconnect-path atomic updates.
     - documented in `docs/book/1274-m39-lasm-fallback-live-count-atomic-dedup.md`.
   - [x] Added env-tunable relay selection reservation chunk with status visibility:
     - relay worker backend-selection reservation chunk is now configurable via `SEC4_RT_LASM_CLUSTER_SELECTION_RESERVATION_MIN_CHUNK` (bounded and clamped against accept batch size),
     - cluster status JSON now emits resolved `relaySelectionReservationMinChunk` for operator observability during load tuning.
     - documented in `docs/book/1275-m39-lasm-selection-reservation-min-chunk-env-and-status.md`.
   - [x] Added degraded dispatch cursor live-slot skipping for multi-live pools:
     - in degraded mode with more than two live relay shards, accept-loop cursor advancement now skips dead next-slot indices and advances directly to the next known live shard,
     - single-live degraded mode now keeps the dispatch cursor pinned to the current live shard instead of wrapping through dead slots between requests.
     - documented in `docs/book/1276-m39-lasm-degraded-cursor-live-slot-skipping.md`.
   - [x] Added cached next-live index lookup for degraded relay dispatch:
     - accept-loop degraded cursor realignment and next-index advancement now reuse a precomputed `next live relay index` lookup table instead of per-request scan lookups when dead slots exist,
     - lookup cache refreshes only when relay liveness changes (disconnect path), preserving deterministic degraded dispatch behavior while reducing repeated live-scan overhead.
     - documented in `docs/book/1277-m39-lasm-degraded-next-live-lookup-cache.md`.
   - [x] Optimized degraded next-live lookup refresh to linear-time:
     - `refresh_lasm_cluster_next_live_sender_lookup(...)` now builds per-slot next-live indices in one reverse sweep anchored at the first live shard instead of calling a full scan per slot,
     - preserves deterministic lookup semantics while reducing liveness-change refresh overhead from repeated scan lookups.
     - documented in `docs/book/1278-m39-lasm-next-live-lookup-linear-refresh.md`.
   - [x] Scoped next-live lookup cache to multi-live degraded pools only:
     - accept-loop now allocates/maintains `relay_next_live_sender_lookup` only when relay pool size is greater than two,
     - liveness-change refresh now skips next-live lookup rebuild once degraded live-shard count drops to `<=2`, where dedicated single/dual paths already handle dispatch.
     - documented in `docs/book/1279-m39-lasm-next-live-lookup-sparse-activation.md`.
   - [x] Simplified primary-disconnect hint handling in accept-loop fallback:
     - on primary relay-sender disconnect, accept-loop fallback now invalidates cached single/dual live-hint slots immediately instead of refreshing them in-place,
     - hint refresh remains centralized in existing post-fallback live-count-change path, avoiding duplicate disconnect-path hint scans while preserving deterministic fallback selection.
     - documented in `docs/book/1280-m39-lasm-disconnect-hint-invalidation.md`.
   - [x] Routed fallback-multi live-index selection through cached next-live lookup when available:
     - `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` now accepts optional next-live lookup state and uses cached live-index resolution in degraded scan branches (`scan_live_target` 0/1/2),
     - keeps scan fallback behavior for stale/missing cache entries while reducing repeated degraded live-index scans during fallback dispatch.
     - documented in `docs/book/1281-m39-lasm-fallback-multi-next-live-cache-usage.md`.
   - [x] Added explicit cache-eligibility gating for fallback lookup usage:
     - accept-loop now precomputes whether next-live lookup storage exists and passes cached lookup state into fallback-multi only while degraded live-shard count remains above two,
     - avoids per-fallback lookup-option churn and skips unnecessary cache refresh/use paths once runtime transitions into dedicated single/dual live fast paths.
     - documented in `docs/book/1282-m39-lasm-fallback-cache-eligibility-gating.md`.
   - [x] Removed redundant runtime cache-shape guard in next-live resolver hot path:
     - `resolve_lasm_cluster_next_live_sender_index(...)` now relies on debug assertions for cache-shape invariants and avoids per-call runtime length comparison before cache lookup dispatch,
     - preserves existing fallback-to-scan behavior when cache is absent while reducing branch work in cached degraded lookup paths.
     - documented in `docs/book/1283-m39-lasm-next-live-resolver-runtime-check-elision.md`.
   - [x] Added dead-slot jump-ahead in fallback-multi degraded scan loop:
     - in fallback-multi `scan_live_target > 2` path, dead-slot encounters now jump to next live candidate using the existing next-live resolver instead of stepping one dead slot at a time,
     - reduces repeated dead-slot checks in degraded sparse-live pools while preserving deterministic fallback bounds and saturated/unavailable outcomes.
     - documented in `docs/book/1284-m39-lasm-fallback-dead-slot-jump-ahead.md`.
   - [x] Refreshed next-live cache immediately on primary-disconnect before fallback dispatch:
     - when primary relay send fails with `Disconnected`, accept-loop now refreshes next-live lookup cache before entering fallback dispatch (for eligible multi-live cache states),
     - reduces stale-cache fallback scans within the same request’s fallback path while preserving post-fallback live-count refresh behavior.
     - documented in `docs/book/1285-m39-lasm-primary-disconnect-immediate-next-live-refresh.md`.
   - [x] Avoided duplicate next-live cache refresh after primary-disconnect fallback flows:
     - accept-loop now tracks live-count immediately after primary dispatch and skips post-fallback next-live cache refresh when fallback did not change live-count beyond already-refreshed primary-disconnect state,
     - keeps post-fallback refresh active when fallback introduces additional disconnect-driven live-count changes.
     - documented in `docs/book/1286-m39-lasm-duplicate-post-fallback-cache-refresh-elision.md`.
   - [x] Added jump-distance accounting for degraded fallback scan progression:
     - fallback-multi degraded scan loop now advances `scanned_slots` by wrapped jump distance when cursor jumps to next live candidate (dead-slot and post-attempt advancement paths),
     - preserves bounded traversal semantics while reducing repeated loop iterations in sparse-live degraded pools.
     - documented in `docs/book/1287-m39-lasm-fallback-jump-distance-accounting.md`.
   - [x] Added next-slot-live fast paths in degraded fallback jump advancement:
     - fallback-multi dead-slot and post-attempt jump paths now check wrapped `next_scan_start` liveness directly before invoking next-live resolver,
     - avoids resolver/cache dispatch overhead when the immediate wrapped next slot is already live.
     - documented in `docs/book/1288-m39-lasm-fallback-next-slot-live-fast-path.md`.
   - [x] Made degraded fallback general scan target adaptive to disconnects:
     - fallback-multi general degraded scan now tracks dynamic `scan_live_target` and updates it after disconnect-driven live-count drops inside the loop,
     - avoids iterating against stale pre-disconnect live-target counts while preserving bounded scan semantics.
     - documented in `docs/book/1289-m39-lasm-fallback-dynamic-scan-live-target.md`.
   - [x] Removed optional lookup plumbing from fallback-multi resolver path:
     - `resolve_lasm_cluster_next_live_sender_index(...)` and `dispatch_lasm_cluster_relay_stream_fallback_multi(...)` now use direct lookup-slice input (`&[usize]`) with empty-slice fallback semantics instead of option-wrapped lookup references,
     - simplifies hot-path branch shape and avoids repeated option wrapping at fallback call sites.
     - documented in `docs/book/1290-m39-lasm-fallback-lookup-slice-plumbing.md`.
   - [x] Added start-index-live short-circuit in next-live resolver:
     - `resolve_lasm_cluster_next_live_sender_index(...)` now returns immediately when `start_index_wrapped` is already live before cache lookup or scan fallback,
     - reduces resolver overhead in degraded dispatch paths that already point at a live relay shard.
     - documented in `docs/book/1291-m39-lasm-next-live-resolver-start-index-fast-path.md`.
   - [x] Added early-break on dynamic live-target shrink in fallback general scan:
     - fallback-multi general degraded scan now exits before post-attempt jump work when disconnect-driven live-target reduction makes current `scanned_live` already sufficient,
     - avoids unnecessary next-index resolution/jump bookkeeping after target shrink while preserving deterministic fallback outcomes.
     - documented in `docs/book/1292-m39-lasm-fallback-early-break-on-target-shrink.md`.
   - [x] Extracted shared fallback scan-index jump advancement helper:
     - fallback-multi now centralizes dead-slot and post-attempt jump progression into `advance_lasm_cluster_fallback_scan_index(...)`,
     - preserves existing jump-distance accounting and next-live resolver semantics while reducing duplicated hot-path branch logic.
     - documented in `docs/book/1293-m39-lasm-fallback-scan-index-advance-helper.md`.
   - [x] Added direct lookup/scan path in fallback scan-advance helper:
     - `advance_lasm_cluster_fallback_scan_index(...)` now uses direct lookup-slice (`lookup_lasm_cluster_next_live_sender_index`) or direct scan (`lasm_cluster_next_live_sender_index`) paths after dead wrapped-next detection, instead of routing back through the generic resolver helper,
     - removes redundant resolver branching in helper-level hot-path advancement while preserving fallback semantics.
     - documented in `docs/book/1294-m39-lasm-fallback-scan-helper-direct-lookup-path.md`.
   - [x] Added immediate no-live exit in fallback general degraded scan:
     - fallback-multi general degraded scan now returns deterministic unavailable as soon as disconnect handling drops `relay_live_sender_count` to zero inside the scan loop,
     - avoids extra loop-control/jump bookkeeping after terminal no-live transitions while preserving saturated/unavailable envelope semantics.
     - documented in `docs/book/1295-m39-lasm-fallback-immediate-no-live-exit.md`.
   - [x] Skipped final scan-index advancement when fallback scan budget is exhausted:
     - fallback-multi degraded general scan now exits before calling scan-index advancement helper when `scanned_slots + 1` already reaches `scan_slot_limit`,
     - applies to both dead-slot and post-attempt advancement branches to avoid terminal helper lookups that cannot feed another loop iteration.
     - documented in `docs/book/1296-m39-lasm-fallback-skip-final-scan-advance.md`.
   - [x] Merged duplicated single-attempt fallback branches:
     - fallback-multi now handles `scan_live_target <= 1` with one shared single-attempt branch instead of duplicated `== 0` and `== 1` blocks,
     - preserves deterministic saturated/unavailable outcomes while reducing hot-path branch depth and duplicate send/disconnect handling logic.
     - documented in `docs/book/1297-m39-lasm-fallback-single-attempt-branch-merge.md`.
   - [x] Extracted shared fallback terminal-result helper across single/dual/multi paths:
     - added inline helper `lasm_cluster_fallback_terminal_dispatch_error(...)` and routed repeated saturated/unavailable return branches through it,
     - reduces duplicate terminal-result branch logic in fallback dispatch paths while preserving deterministic error envelopes.
     - documented in `docs/book/1298-m39-lasm-fallback-terminal-result-helper.md`.
   - [x] Added immediate no-live guard in dual-attempt fallback branch:
     - fallback-multi `scan_live_target == 2` path now returns immediately before second-live resolution when first-attempt disconnect handling drops `relay_live_sender_count` to zero,
     - avoids unnecessary resolver/lookup work on terminal no-live transitions while preserving deterministic fallback outcomes.
     - documented in `docs/book/1299-m39-lasm-fallback-dual-attempt-no-live-guard.md`.
   - [x] Reused single-live helper for two-sender fallback path:
     - fallback-multi now routes `sender_count == 2` dispatch through `dispatch_lasm_cluster_relay_stream_fallback_single_live(...)` instead of maintaining an inline duplicate two-sender branch body,
     - preserves deterministic send/disconnect semantics while reducing duplicate fallback logic.
     - documented in `docs/book/1300-m39-lasm-fallback-two-sender-single-helper-reuse.md`.
   - [x] Reused single-live helper in `scan_live_target <= 1` fallback branch:
     - fallback-multi now delegates resolved single-target dispatch to `dispatch_lasm_cluster_relay_stream_fallback_single_live(...)` instead of keeping a duplicated local send/disconnect block,
     - preserves deterministic saturated/unavailable behavior while reducing hot-path duplication.
     - documented in `docs/book/1301-m39-lasm-fallback-single-target-helper-reuse.md`.
   - [x] Added second-attempt next-slot-live fast path in dual fallback branch:
     - fallback-multi `scan_live_target == 2` branch now checks the wrapped second-start slot directly before resolver lookup, and dispatches via the shared single-live helper,
     - avoids redundant second-attempt resolver work on adjacent-live paths while preserving deterministic fallback semantics.
     - documented in `docs/book/1302-m39-lasm-fallback-dual-second-live-fastpath.md`.
   - [x] Extracted shared relay send-attempt helper in fallback multi hot path:
     - added `attempt_lasm_cluster_relay_send(...)` and reused it in all-live fallback scan, dual-branch first attempt, and degraded general scan attempt path,
     - keeps disconnect/full handling centralized while preserving disconnect-driven dynamic target updates in degraded scans.
     - documented in `docs/book/1303-m39-lasm-fallback-shared-send-attempt-helper.md`.
   - [x] Routed single/dual live fallback helpers through shared send-attempt helper:
     - `dispatch_lasm_cluster_relay_stream_fallback_single_live(...)` and `dispatch_lasm_cluster_relay_stream_fallback_dual_live(...)` now delegate send/full/disconnect handling to `attempt_lasm_cluster_relay_send(...)`,
     - preserves deterministic terminal-result semantics while centralizing fallback send handling across single/dual/multi paths.
     - documented in `docs/book/1304-m39-lasm-fallback-single-dual-shared-send-helper.md`.
   - [x] Switched accept-loop primary relay dispatch to shared send-attempt helper:
     - multi-sender accept path now routes primary relay send through `attempt_lasm_cluster_relay_send(...)`, keeping full/disconnect transitions centralized before saturation short-circuit and fallback dispatch routing,
     - preserves saturated short-circuit behavior and disconnect-driven hint/lookup refresh semantics.
     - documented in `docs/book/1305-m39-lasm-accept-primary-shared-send-helper.md`.
   - [x] Added single-sender accept dispatch helper:
     - extracted `attempt_lasm_cluster_relay_send_single(...)` to centralize single-sender `try_send` error mapping (`Saturated`/`Unavailable`),
     - single-sender accept loop now delegates dispatch mapping through the helper before existing accept-dispatch error handling.
     - documented in `docs/book/1306-m39-lasm-single-sender-dispatch-helper.md`.
   - [x] Extracted relay send-attempt helpers into dedicated module:
      - moved `attempt_lasm_cluster_relay_send(...)` and `attempt_lasm_cluster_relay_send_single(...)` from `main.rs` into `compiler/sec4-cli/src/lasm_cluster_relay_send.rs`,
      - `main.rs` now imports relay send helpers through module boundaries, continuing multi-file LASM runtime decomposition without semantic changes.
      - documented in `docs/book/1307-m39-lasm-relay-send-module-extraction.md`.
   - [x] Extracted relay topology helpers into dedicated module:
      - moved `lasm_cluster_next_index_wrapped`, `lasm_cluster_next_live_sender_index`, `lookup_lasm_cluster_next_live_sender_index`, `resolve_lasm_cluster_next_live_sender_index`, and `realign_lasm_cluster_dispatch_cursor_to_live` into `compiler/sec4-cli/src/lasm_cluster_relay_topology.rs`,
      - `main.rs` now imports topology helpers through module boundaries while preserving LASM cluster dispatch semantics.
      - documented in `docs/book/1308-m39-lasm-relay-topology-module-extraction.md`.
   - [x] Moved relay live-hint refresh helpers into topology module:
      - extracted `refresh_lasm_cluster_next_live_sender_lookup`, `refresh_lasm_cluster_single_live_sender_index`, `refresh_lasm_cluster_dual_live_sender_indices`, and `refresh_lasm_cluster_live_sender_hints` from `main.rs` into `compiler/sec4-cli/src/lasm_cluster_relay_topology.rs`,
      - accept-loop/fallback call-sites now consume these helpers from module imports, continuing LASM cluster decomposition without runtime behavior changes.
      - documented in `docs/book/1309-m39-lasm-relay-live-hint-module-extraction.md`.
   - [x] Extracted fallback dispatch mechanics into dedicated module:
      - moved fallback dispatch enum + helpers (`dispatch_lasm_cluster_relay_stream_fallback_*` and scan-advance internals) from `main.rs` into `compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`,
      - `main.rs` now imports fallback dispatch functions from module boundaries while retaining accept-loop orchestration and error handling.
      - documented in `docs/book/1310-m39-lasm-fallback-dispatch-module-extraction.md`.
   - [x] Moved fallback dispatch error enum ownership into fallback module:
      - `LasmClusterRelayDispatchError` is now defined in `compiler/sec4-cli/src/lasm_cluster_fallback_dispatch.rs`,
      - `main.rs` and relay send helpers import the enum through module boundaries, reducing fallback type ownership in the CLI entry file.
      - documented in `docs/book/1311-m39-lasm-fallback-error-enum-module-ownership.md`.
   - [x] Extracted accept-dispatch helpers into dedicated module:
      - moved cluster unavailable response helpers, saturation/dispatch/active counter flush helpers, and `handle_lasm_cluster_accept_dispatch_error(...)` from `main.rs` into `compiler/sec4-cli/src/lasm_cluster_accept_dispatch.rs`,
      - accept-loop and relay-worker call sites now import this helper surface through module boundaries without changing saturation/unavailable response semantics.
      - documented in `docs/book/1312-m39-lasm-accept-dispatch-module-extraction.md`.
   - [x] Extracted cluster status snapshot/json writer into dedicated module:
      - moved `LasmClusterStatusSnapshot` and `write_lasm_cluster_status_json(...)` from `main.rs` into `compiler/sec4-cli/src/lasm_cluster_status_json.rs`,
      - status-writer thread now depends on a dedicated status-json module while preserving unchanged-snapshot skip behavior and deterministic payload shape.
      - documented in `docs/book/1313-m39-lasm-cluster-status-json-module-extraction.md`.
   - [x] Extracted LASM cluster accept loop into dedicated module:
      - moved `run_lasm_cluster_accept_loop(...)` from `main.rs` into `compiler/sec4-cli/src/lasm_cluster_accept_loop.rs`,
      - accept-loop orchestration now imports dispatch/fallback/topology helpers from module boundaries while preserving queue saturation, fallback dispatch, and counter flush behavior.
      - documented in `docs/book/1314-m39-lasm-cluster-accept-loop-module-extraction.md`.
   - [x] Extracted LASM relay pump types into dedicated module:
      - moved `LasmClusterRelayPumpStep` and `LasmClusterRelayPump` (+ impl) from `main.rs` into `compiler/sec4-cli/src/lasm_cluster_relay_pump.rs`,
      - relay worker loop now consumes relay pump types through module boundaries while preserving proxy relay buffer/pump semantics.
      - documented in `docs/book/1315-m39-lasm-cluster-relay-pump-module-extraction.md`.
   - [x] Extracted backend-selection/remap helpers into dedicated module:
      - moved `LASM_CLUSTER_SELECTION_LOOKUP_NONE`, `rebuild_lasm_cluster_backend_selection_lookup(...)`, `rebuild_lasm_cluster_worker_backend_addrs(...)`, and `remap_lasm_cluster_relay_port_state_by_index(...)` from `main.rs` into `compiler/sec4-cli/src/lasm_cluster_backend_selection.rs`,
      - relay worker loop now imports backend-selection/remap helpers from module boundaries while preserving deterministic healthy/unhealthy backend mapping behavior.
      - documented in `docs/book/1316-m39-lasm-cluster-backend-selection-module-extraction.md`.
   - [x] Extracted LASM cluster runtime-config helpers into dedicated module:
      - moved cluster sizing/timing/env resolver helpers from `main.rs` into `compiler/sec4-cli/src/lasm_cluster_runtime_config.rs` (proxy worker/queue sizing, accept-worker sizing, relay accept/pump batch helpers, selection min-chunk helper, backend connect timeout/cooldown, autoscale cooldown helpers),
      - `main.rs` now imports runtime-config helper surface from module boundaries while preserving existing runtime behavior and diagnostics.
      - documented in `docs/book/1317-m39-lasm-cluster-runtime-config-module-extraction.md`.
   - [x] Extracted LASM cluster worker lifecycle helpers into dedicated module:
      - moved cluster worker lifecycle helpers from `main.rs` into `compiler/sec4-cli/src/lasm_cluster_lifecycle.rs` (`compute_lasm_cluster_base_port`, worker spawn/wait helpers, prune/recover/stop helpers, reuse-port listener bind, reuse-port cluster runner),
      - cluster orchestration now imports lifecycle helpers through module boundaries while preserving worker startup/recovery semantics and reuse-port behavior.
      - documented in `docs/book/1318-m39-lasm-cluster-lifecycle-module-extraction.md`.
   - [x] Extracted LASM cluster relay worker loop into dedicated module:
      - moved relay worker thread loop from `cmd_run_lasm_cluster(...)` in `main.rs` into `compiler/sec4-cli/src/lasm_cluster_relay_worker_loop.rs` (`spawn_lasm_cluster_relay_worker_loop`),
      - cluster orchestration now spawns relay workers via module boundary while preserving backend selection, unhealthy cooldown mapping, relay pump scheduling, and deterministic unavailable-response behavior.
      - documented in `docs/book/1319-m39-lasm-cluster-relay-worker-loop-module-extraction.md`.
   - [x] Extracted LASM cluster status-writer thread loop into dedicated module:
      - moved status-writer thread loop from `cmd_run_lasm_cluster(...)` in `main.rs` into `compiler/sec4-cli/src/lasm_cluster_status_writer.rs` (`spawn_lasm_cluster_status_writer`, `LasmClusterStatusWriterConfig`),
      - cluster orchestration now wires status-writer dependencies through a typed module config while preserving unchanged-snapshot skip behavior and deterministic status payload emission cadence.
      - documented in `docs/book/1320-m39-lasm-cluster-status-writer-module-extraction.md`.
   - [x] Extracted LASM cluster autoscale thread loop into dedicated module:
      - moved autoscale thread loop from `cmd_run_lasm_cluster(...)` in `main.rs` into `compiler/sec4-cli/src/lasm_cluster_autoscale_loop.rs` (`spawn_lasm_cluster_autoscale_loop`, `LasmClusterAutoscaleLoopConfig`),
      - cluster orchestration now wires autoscale dependencies through module config while preserving dynamic boost behavior, cooldown tracking, and worker lifecycle update semantics.
      - documented in `docs/book/1321-m39-lasm-cluster-autoscale-loop-module-extraction.md`.
   - [x] Extracted LASM cluster accept-worker orchestration into dedicated module:
      - moved accept-worker thread orchestration (listener clones, worker thread spawn/join, primary accept-loop run) from `cmd_run_lasm_cluster(...)` into `compiler/sec4-cli/src/lasm_cluster_accept_workers.rs` (`run_lasm_cluster_accept_workers`, `LasmClusterAcceptWorkersConfig`),
      - cluster orchestration now delegates accept-worker lifecycle through module boundary while preserving deterministic accept-loop error reporting and stop-flag shutdown behavior.
      - documented in `docs/book/1322-m39-lasm-cluster-accept-workers-module-extraction.md`.
   - [x] Extracted LASM cluster shutdown finalization into dedicated module:
      - moved repeated cluster shutdown/join sequence from `cmd_run_lasm_cluster(...)` into `compiler/sec4-cli/src/lasm_cluster_shutdown.rs` (`finalize_lasm_cluster_runtime`),
      - nonblocking failure, accept-worker failure, and normal completion paths now share one deterministic finalizer for stop-flag set, relay/aux thread joins, and worker-stop lifecycle cleanup.
      - documented in `docs/book/1323-m39-lasm-cluster-shutdown-module-extraction.md`.
   - [x] Moved LASM cluster listener nonblocking setup to preflight:
      - `cmd_run_lasm_cluster(...)` now sets proxy listener nonblocking immediately after bind and exits before worker/thread startup on failure,
      - removes late nonblocking failure path from post-bootstrap orchestration while preserving existing failure diagnostics.
      - documented in `docs/book/1324-m39-lasm-cluster-listener-nonblocking-preflight.md`.
   - [x] Hardened LASM cluster thread lifecycle with panic-aware joins:
      - accept-worker orchestration now detects accept worker thread panics during join and returns deterministic runtime failure instead of silently ignoring join failures,
      - shared cluster finalizer now returns panic summary across relay/autoscale/status threads and `cmd_run_lasm_cluster(...)` fails deterministically when shutdown observes worker-thread panic conditions.
      - documented in `docs/book/1325-m39-lasm-cluster-thread-panic-handling.md`.
   - [x] Hardened LASM cluster accept-loop failure propagation:
      - accept-worker orchestration now records and returns deterministic first accept-loop failure message instead of printing and returning success,
      - `cmd_run_lasm_cluster(...)` now fails non-zero when any accept-loop worker/main accept path reports an error, preserving stop-flag shutdown behavior and deterministic error surface.
      - documented in `docs/book/1326-m39-lasm-cluster-accept-error-propagation.md`.
   - [x] Hardened LASM cluster shutdown lock-poison handling:
      - shutdown finalizer summary now tracks `state_lock_poisoned` when cluster state write lock is poisoned during finalization,
      - cluster run path now treats shutdown lock-poison as deterministic runtime failure alongside background-thread panic conditions.
      - documented in `docs/book/1327-m39-lasm-cluster-shutdown-lock-poison-handling.md`.
   - [x] Hardened LASM status-writer failure logging with throttled dedupe:
      - status-writer loop now throttles repeated JSON write warnings and suppresses duplicate error spam between intervals when failure text is unchanged,
      - successful status writes reset warning throttle state so new failures are surfaced immediately.
      - documented in `docs/book/1328-m39-lasm-cluster-status-writer-warning-throttle.md`.
   - [x] Tightened relay fallback scan slot accounting in multi-relay dispatch:
      - `dispatch_lasm_cluster_relay_stream_fallback_multi` now uses a remaining-slot budget instead of per-iteration saturating slot counters for dead/live sender advancement,
      - removes saturating arithmetic from fallback scan hot-path iteration while preserving fallback dispatch terminal semantics (`saturated` vs `unavailable`) and existing live-target bounds.
      - documented in `docs/book/1329-m39-lasm-cluster-fallback-scan-remaining-slot-accounting.md`.
   - [x] Tightened single/dual-live fallback membership checks to direct index reads:
      - `dispatch_lasm_cluster_relay_stream_fallback_single_live` and `_dual_live` now use direct indexed live-state checks with explicit debug shape assertions instead of `get().copied().unwrap_or(...)` chains,
      - keeps fallback terminal semantics unchanged while trimming option-chain overhead from hot fallback fast paths.
      - documented in `docs/book/1330-m39-lasm-cluster-fallback-direct-live-index-checks.md`.
   - [x] Removed repeated fallback lookup-presence checks from scan-advance hot path:
      - `dispatch_lasm_cluster_relay_stream_fallback_multi` now resolves lookup availability once (`relay_has_next_live_sender_lookup`) and threads that into scan-advance helper calls,
      - `advance_lasm_cluster_fallback_scan_index` no longer checks `relay_next_live_sender_lookup.is_empty()` on every scan step, preserving scan semantics while reducing per-step branch work.
      - documented in `docs/book/1331-m39-lasm-cluster-fallback-scan-lookup-presence-hoist.md`.
   - [x] Tightened relay topology next-live resolution to direct index reads:
      - `resolve_lasm_cluster_next_live_sender_index` now performs explicit empty-slice guard + direct `relay_sender_live[start_index_wrapped]` check instead of option-chain access (`get().copied().unwrap_or(...)`),
      - preserves next-live resolution semantics while removing option-chain overhead from a shared relay topology helper used in degraded dispatch paths.
      - documented in `docs/book/1332-m39-lasm-cluster-relay-topology-direct-start-index-check.md`.
   - [x] Tightened relay-send disconnected path to avoid repeated live-count decrements:
      - `attempt_lasm_cluster_relay_send` now marks sender dead/count-down only when sender state transitions from live to dead, replacing unconditional saturating decrement on every disconnected attempt,
      - preserves disconnected fallback behavior while avoiding duplicate counter decrements under repeated disconnected sends and removing saturating arithmetic from this path.
      - documented in `docs/book/1333-m39-lasm-cluster-relay-send-dead-transition-guard.md`.
   - [x] Tightened accept-dispatch overload response hot path:
      - `handle_lasm_cluster_accept_dispatch_error` now writes relay-saturated/relay-unavailable static response buffers directly in the dispatch-error handler instead of routing through reason enum + helper dispatch,
      - preserves overload response payloads and terminal semantics while removing per-error reason dispatch overhead in the accept hot path.
      - documented in `docs/book/1334-m39-lasm-cluster-accept-dispatch-direct-overload-buffer-writes.md`.
   - [x] Replaced enum-based unavailable-response dispatch with dedicated helper entry points:
      - `lasm_cluster_accept_dispatch` now exposes dedicated no-healthy-worker / worker-unavailable response writers instead of reason-enum routing,
      - relay worker loop now calls these dedicated helpers directly, preserving response payloads while removing reason-enum dispatch plumbing from unavailable response paths.
      - documented in `docs/book/1335-m39-lasm-cluster-unavailable-response-helper-split.md`.
   - [x] Tightened relay lookup helper to invariant fast path:
      - `lookup_lasm_cluster_next_live_sender_index` now relies on established topology invariants (`non-empty`, `cached index in bounds`) via debug assertions and direct indexed reads,
      - removed redundant runtime empty/bounds checks on the hot lookup path while preserving fallback live-scan behavior when cached entry is no longer live.
      - documented in `docs/book/1336-m39-lasm-cluster-relay-lookup-invariant-fast-path.md`.
   - [x] Added lookup-state-aware next-live resolver for degraded fallback dispatch:
      - relay topology now exposes `resolve_lasm_cluster_next_live_sender_index_with_lookup_state(...)` so hot callers can pass precomputed lookup availability,
      - fallback multi-dispatch now threads one `relay_has_next_live_sender_lookup` flag through next-live resolution calls, removing repeated lookup-availability branching from degraded dispatch resolution.
      - documented in `docs/book/1337-m39-lasm-cluster-resolve-next-live-with-lookup-state.md`.
   - [x] Simplified accept-loop fallback multi-dispatch lookup plumbing:
      - added `dispatch_lasm_cluster_relay_stream_fallback_multi_with_lookup_state(...)` so accept-loop code passes one precomputed lookup-state flag instead of rebuilding lookup slices at each fallback branch,
      - accept-loop fallback path now computes `relay_use_next_live_lookup_for_fallback` once per failed primary dispatch and reuses it across single/dual/default fallback paths.
      - documented in `docs/book/1338-m39-lasm-cluster-accept-loop-fallback-lookup-state-plumbing.md`.
   - [x] Tightened all-live fallback scan wrapped-index progression:
      - `dispatch_lasm_cluster_relay_stream_fallback_multi` now advances scan index with direct increment + wrap reset (`scan_index += 1; if == sender_count {0}`) in the all-live branch,
      - preserves sender traversal order and fallback semantics while removing per-iteration wrapped-index helper call overhead from the all-live fallback scan loop.
      - documented in `docs/book/1339-m39-lasm-cluster-fallback-all-live-direct-wrap-increment.md`.
   - [x] Removed remaining saturating arithmetic from fallback live-target setup:
      - fallback scan slot limit now uses direct `sender_count - 1` under existing `sender_count > 1` invariant,
      - fallback live-target setup now uses direct bounded decrement guards (`live_count - 1`) instead of `saturating_sub` for both initial and dynamic live-target tracking.
      - documented in `docs/book/1340-m39-lasm-cluster-fallback-live-target-direct-guards.md`.
   - [x] Removed redundant live-count guard in accept-loop dispatch-cursor realignment:
      - accept-loop fallback completion path now uses only `relay_live_sender_count > 1` for dispatch-cursor live realignment, dropping redundant `relay_live_sender_count > 0` conjunction,
      - preserves realignment behavior while trimming one redundant branch check on this hot path.
      - documented in `docs/book/1341-m39-lasm-cluster-accept-loop-redundant-live-guard-removal.md`.
   - [x] Tightened resolver invariants and saturated-counter flush path:
      - lookup-state resolver now relies on non-empty sender-slice invariant (`sender_count > 0`) via debug assertion in fallback-callers-only hot path, removing runtime empty-slice branch,
      - accept-dispatch saturated branch now flushes local saturation counters directly (atomic adds + reset) when batch threshold is reached, avoiding helper-call indirection on saturated-error hot path.
      - documented in `docs/book/1342-m39-lasm-cluster-resolver-invariant-and-saturation-inline-flush.md`.
   - [x] Tightened accept-loop fallback branch tree with live-state match:
      - accept-loop fallback dispatch now branches `relay_all_senders_live` first, then uses `match relay_live_sender_count` for degraded paths (`2`, `1`, default) instead of repeated `!relay_all_senders_live && count==...` checks,
      - preserves fallback behavior while reducing repeated boolean conjunction checks in failed-primary-dispatch handling.
      - documented in `docs/book/1343-m39-lasm-cluster-accept-loop-fallback-branch-tree-match.md`.
   - [x] Tightened shared next-live sender scan helper loop progression:
      - `lasm_cluster_next_live_sender_index` now advances scan cursor with direct increment + wrap reset instead of calling wrapped-index helper each iteration,
      - preserves scan semantics while reducing helper-call overhead in a shared degraded-routing utility used across accept/fallback paths.
      - documented in `docs/book/1344-m39-lasm-cluster-next-live-scan-direct-wrap-progression.md`.
   - [x] Inlined fallback scan-advance wrapped-step arithmetic:
      - `advance_lasm_cluster_fallback_scan_index` now computes next-scan wrapped index and forward-distance slots directly in-function,
      - removed helper indirection for wrapped-forward-distance calculation while preserving fallback scan step semantics and slot advancement behavior.
      - documented in `docs/book/1345-m39-lasm-cluster-fallback-scan-advance-inline-wrap-distance.md`.
   - [x] Removed fallback-multi wrapper and threaded lookup-state directly into core fallback function:
      - `dispatch_lasm_cluster_relay_stream_fallback_multi` now receives precomputed `relay_has_next_live_sender_lookup` directly and no longer derives lookup availability with internal `is_empty` checks,
      - accept-loop now calls core fallback-multi function directly (wrapper removed), reducing one call layer and one per-call lookup-availability branch in fallback dispatch plumbing.
      - documented in `docs/book/1346-m39-lasm-cluster-fallback-multi-direct-lookup-state-signature.md`.
   - [x] Tightened autoscale arithmetic to direct bounded math paths:
      - saturation batch count now uses direct quotient+remainder rounding (under non-zero flush-batch invariant), and scale-up/down bounds now use direct remaining-capacity / bounded-subtraction math instead of saturating add/sub helper chains,
      - preserves autoscale semantics while trimming saturating arithmetic overhead in the periodic LASM cluster autoscale loop.
      - documented in `docs/book/1347-m39-lasm-cluster-autoscale-direct-bounded-arithmetic.md`.
   - [x] Tightened relay worker selection/pump cursor progression to direct wrap increments:
      - relay selection reservation next-index progression now uses direct increment+wrap instead of wrapped-index helper calls,
      - relay pump cursor advancement in the batched pump loop now uses direct increment+wrap for `Progressed`/`Idle` steps.
      - documented in `docs/book/1348-m39-lasm-cluster-relay-worker-direct-wrap-cursor-progression.md`.
   - [x] Reduced autoscale-loop worker-port snapshot churn on no-op ticks:
      - autoscale loop now tracks whether workers changed after the initial maintenance refresh and skips the second status-port snapshot refresh when no scale-up/down occurred,
      - preserves snapshot publication semantics while avoiding repeated worker-port comparison/publish work in steady-state no-op ticks.
      - documented in `docs/book/1349-m39-lasm-cluster-autoscale-noop-second-refresh-elision.md`.
   - [x] Batched relay selection/runtime-config direct arithmetic simplifications:
      - relay worker loop now hoists selection reservation chunk size (`max(accept_batch, reservation_min_chunk)`) out of the per-request selection path,
      - backend selection lookup tail wrap-fill now uses direct slice fill for post-first-healthy indices,
      - desired autoscale instance calculation now uses direct ceil-division math on active connections (`((active-1)/target)+1`) with explicit `target>=1` guard.
      - documented in `docs/book/1350-m39-lasm-cluster-relay-selection-and-runtime-config-direct-arithmetic-batch.md`.
   - [x] Elided redundant autoscale cooldown atomic stores on unchanged anchors:
      - autoscale loop now tracks whether scale-up/down cooldown anchors changed in the current tick and only rewrites cooldown-remaining atomics at loop tail when anchors changed,
      - avoids duplicate cooldown atomic writes on steady-state no-op ticks while preserving immediate cooldown reset semantics after scale actions.
      - documented in `docs/book/1353-m39-lasm-cluster-autoscale-cooldown-tail-store-elision.md`.
   - [x] Rebalanced degraded relay selection lookup to cyclic healthy distribution:
      - backend selection lookup rebuild now materializes ordered healthy backend indices and maps selection slots cyclically across healthy backends when some workers are unhealthy,
      - removes gap-weighted degraded routing bias tied to dead-worker index spans and keeps degraded load spread deterministic across remaining healthy backends.
      - documented in `docs/book/1354-m39-lasm-cluster-cyclic-healthy-lookup-selection.md`.
   - [x] Routed degraded reservation cadence through healthy-cycle span:
      - backend lookup rebuild now returns explicit degraded healthy cycle span, and relay selection reservation now uses that span for modulo/cursor progression when not in identity mode,
      - removes remaining dead-slot influence from degraded reservation cadence and keeps reservation churn scoped to healthy backend count.
      - documented in `docs/book/1355-m39-lasm-cluster-healthy-cycle-span-reservation-selection.md`.
   - [x] Inlined remaining dispatch wrapped-step helpers in accept/fallback hot paths:
      - accept loop now advances `next_dispatch_wrapped` with direct increment+wrap arithmetic instead of relay-topology wrapped-index helper call,
      - fallback dual-live branch now computes second-attempt start index with direct increment+wrap arithmetic,
      - removed now-unused relay-topology wrapped-index helper symbol.
      - documented in `docs/book/1356-m39-lasm-cluster-accept-fallback-direct-wrap-step-elision.md`.
   - [x] Added relay worker connect-failure single healthy-backend fallback attempt:
      - when the primary selected backend connect fails, relay worker now marks it unhealthy and immediately tries one alternate currently-healthy backend before returning worker-unavailable response,
      - keeps deterministic unhealthy/warning tracking while reducing request failures under transient single-backend connect faults.
      - documented in `docs/book/1357-m39-lasm-cluster-relay-connect-failure-single-fallback-attempt.md`.
   - [x] Switched fallback connect candidate selection to next-healthy cyclic scan:
      - fallback connect candidate is now chosen by cyclic scan starting from the failed backend’s next index, instead of always picking the first healthy backend,
      - reduces fallback hot-spot bias under repeated connect failures and preserves deterministic degraded routing order.
      - documented in `docs/book/1358-m39-lasm-cluster-relay-connect-fallback-next-healthy-cyclic-scan.md`.
   - [x] Saturation counters now increment only on final connect failure outcome:
      - relay worker connect failure path now records saturation only when request ends with worker-unavailable response after fallback handling,
      - recovered requests (primary connect failure + successful fallback connect) no longer inflate saturation telemetry/autoscale signals.
      - documented in `docs/book/1359-m39-lasm-cluster-relay-saturation-count-final-failure-only.md`.
   - [x] Extended connect-failure fallback to bounded multi-alternate healthy attempts:
      - relay worker connect failure path now scans all remaining healthy backends in cyclic order (after the failed backend) and attempts fallback connect per candidate before final worker-unavailable response,
      - preserves deterministic unhealthy cooldown + warning tracking for each failed candidate while reducing final request failures when multiple healthy backends remain.
      - documented in `docs/book/1360-m39-lasm-cluster-relay-connect-fallback-multi-alternate-attempts.md`.
   - [x] Tightened multi-alternate fallback scan to direct wrapped cursor progression:
      - connect-failure fallback candidate traversal now uses direct wrapped index progression (`index += 1; wrap to 0`) plus explicit remaining-scan counter, removing per-iteration offset arithmetic,
      - fallback scan now exits early when all backends become unhealthy during candidate failures.
      - documented in `docs/book/1361-m39-lasm-cluster-relay-fallback-scan-direct-wrap-and-terminal-break.md`.
   - [x] Unified relay connect-success setup into one helper path:
      - primary and fallback connect-success branches now share one relay-initialization helper (`set_nodelay`, pooled-buffer pump construction, throttled init-failure warning handling),
      - removes duplicated connect-success setup branches in relay worker loop and keeps warning/decrement behavior aligned.
      - documented in `docs/book/1362-m39-lasm-cluster-relay-connect-success-single-setup-helper.md`.
   - [x] Added non-zero guard gates before relay counter-flush helper calls:
      - relay worker loop now calls saturation/active-decrement flush helpers only when local pending counters are non-zero,
      - removes per-cycle helper-call overhead on idle/steady-state cycles with no pending counter deltas.
      - documented in `docs/book/1363-m39-lasm-cluster-relay-flush-helper-nonzero-gates.md`.
   - [x] Reused one selection-state recompute helper across steady and failure paths:
      - relay worker selection-state derivation (`has_healthy`, identity, cycle-span, single-healthy index) now comes from one shared recompute helper,
      - connect-failure fallback traversal now reuses freshly recomputed healthy lookup state instead of scanning all worker slots for healthy candidates.
      - documented in `docs/book/1364-m39-lasm-cluster-relay-shared-selection-recompute-and-fallback-lookup-traversal.md`.
   - [x] Unified backend connect-failure unhealthy/warning bookkeeping in one helper:
      - primary and fallback connect-failure branches now share one helper for unhealthy cooldown marking, prune scheduling, warning throttling, and selection-dirty signaling,
      - removes duplicated failure bookkeeping branches in relay worker loop while preserving deterministic warning and cooldown behavior.
      - documented in `docs/book/1365-m39-lasm-cluster-relay-connect-failure-shared-unhealthy-warning-helper.md`.
   - [ ] Continue performance tuning: current cluster load measurements are well below 1M req/s target, so proxy/runtime hot-path optimization remains open.
4. [x] Progress DB adapters behind the same intrinsic surface:
   - keep file adapter (`records.log`) for alpha path,
   - add SQLite adapter as first real embedded DB target,
   - keep external DB adapters post-alpha.
   - [x] Persisted `affected_rows` metadata in LASM DB record history across records-log/sqlite/postgres adapters (including schema migration/back-compat defaulting for pre-field artifacts) so `DbListRecordsResponse` exposes stable per-record write impact metadata (`docs/book/1096-m39-lasm-db-record-affected-rows-persistence.md`).
   - [x] Records-log `db.queryOne` fallback now returns structured `rowObject` metadata (including `affected_rows`) instead of null, aligning response shape with sqlite/postgres query-one materialization while preserving deterministic fallback semantics (`docs/book/1097-m39-lasm-records-log-query-one-row-object-parity.md`).
   - [x] LASM DB runtime now rejects invalid DB capability handles (`db != 1`) for `db.exec`, `db.queryOne`, and inline `db.tx(...)` sources used by `db.execTx`, preventing manual-handle SQL execution on sqlite/postgres adapters and preserving deterministic `DB.*_INVALID` validation envelopes (`docs/book/1098-m39-lasm-db-capability-handle-validation-guard.md`).
   - [x] `DbListRecordsResponse` now includes deterministic aggregate `affectedRowsTotal` across persisted records for all adapters, improving operator visibility into accumulated write impact without changing intrinsic contracts (`docs/book/1099-m39-lasm-db-list-records-affected-rows-total.md`).
   - [x] LASM runtime now enforces bounded tx-handle capacity (`SEC4_RT_LASM_DB_MAX_TX_HANDLES`, default `256`) and returns deterministic `DB.TX_INTERNAL` when capacity is exhausted, preventing unbounded tx-handle growth in long-lived processes (`docs/book/1100-m39-lasm-db-tx-handle-capacity-guard.md`).
   - [x] `DbListRecordsResponse` now exposes live tx-handle telemetry (`txHandleCount`, `txHandleCapacity`) alongside persisted-record metadata for all adapters, giving deterministic runtime-state visibility during DB execution tuning/debug flows (`docs/book/1101-m39-lasm-db-list-tx-handle-telemetry.md`).
   - [x] `sec4 run` now supports explicit LASM tx-handle capacity override via `--db-max-tx-handles`, with deterministic LASM-only/zero-value guard diagnostics, cluster worker forwarding, and CLI-over-env precedence (`--db-max-tx-handles` over `SEC4_RT_LASM_DB_MAX_TX_HANDLES`) for runtime state initialization (`docs/book/1102-m39-run-db-max-tx-handles-flag.md`).
   - [x] Inline `db.tx(dbCap)` handles used by `db.execTx(...)` are now cleaned up after successful execution, preventing capacity leakage from one-shot inline tx allocations while preserving deterministic tx-handle validation on failure/restart paths (`docs/book/1351-m39-lasm-db-exectx-inline-tx-handle-cleanup.md`).
   - [x] Tx-handle allocator now performs wrap-safe/collision-safe allocation:
      - allocation now probes for vacant positive tx handles starting from `next_db_tx_handle` and wraps from `i64::MAX` back to `1`,
      - avoids handle overwrite risk under long-lived runtimes where handle counter approaches bounds or sparse handle sets are reused.
      - documented in `docs/book/1352-m39-lasm-db-tx-handle-wrap-and-collision-safe-allocation.md`.
5. Extract runtime adapter layers into packages/modules without changing language semantics (priority immediately after DB implementation completion).
   - preserve existing intrinsic contracts and diagnostics (`db.exec`, `db.execTx`, `db.queryOne`, `db.tx`) as-is,
   - move adapter-specific wiring behind package boundaries so runtime backends can evolve independently,
   - keep CLI/operator behavior unchanged while refactoring boundaries.
   - [x] Extracted LASM DB config/adapter resolution helpers from `compiler/sec4-cli/src/main.rs` into dedicated module `compiler/sec4-cli/src/lasm_db_config.rs` (store-base resolution, adapter selection, tx-handle capacity resolution, postgres DSN resolution, adapter label), keeping command/runtime semantics unchanged while establishing the first explicit adapter-boundary seam (`docs/book/1103-m39-lasm-db-config-module-extraction.md`).
   - [x] Extracted LASM records-log persistence/serialization helpers into dedicated module `compiler/sec4-cli/src/lasm_db_records_log.rs` (record JSON conversion, records-log load, records-log persist), reducing DB adapter logic in `main.rs` while preserving `DbListRecordsResponse`/intrinsic runtime behavior (`docs/book/1104-m39-lasm-db-records-log-module-extraction.md`).
   - [x] Extracted sqlite/postgres DB-state load/bootstrap helpers into dedicated module `compiler/sec4-cli/src/lasm_db_adapter_state.rs` (`load_lasm_dynamic_db_records_from_sqlite`, `connect_lasm_dynamic_db_records_postgres`, postgres schema bootstrap, postgres load), further isolating adapter-specific state initialization from CLI/runtime orchestration (`docs/book/1105-m39-lasm-db-adapter-state-module-extraction.md`).
   - [x] Extracted sqlite/postgres DB-state persist helpers into `compiler/sec4-cli/src/lasm_db_adapter_state.rs` (`persist_lasm_dynamic_db_records_to_sqlite`, `persist_lasm_dynamic_db_records_to_postgres`) and kept reconnect/schema semantics intact, further reducing adapter write-path coupling in `main.rs` (`docs/book/1106-m39-lasm-db-adapter-persist-module-extraction.md`).
   - [x] Extracted postgres query-param parsing + exec/queryOne runtime helpers into dedicated module `compiler/sec4-cli/src/lasm_db_runtime_postgres.rs` (`parse_lasm_postgres_query_params`, prepared-placeholder analysis, select-shape/normalization helpers, and `run_lasm_postgres_exec` / `run_lasm_postgres_exec_tx` / `run_lasm_postgres_query_one`), while keeping sqlite and runtime call-path behavior unchanged (`docs/book/1107-m39-lasm-db-runtime-postgres-module-extraction.md`).
   - [x] Extracted sqlite query-param parsing + exec/queryOne runtime helpers into dedicated module `compiler/sec4-cli/src/lasm_db_runtime_sqlite.rs` (`run_lasm_sqlite_exec`, `run_lasm_sqlite_exec_tx`, `run_lasm_sqlite_query_one` plus sqlite parameter/row helpers), while preserving existing sqlite runtime semantics and single-statement validation behavior (`docs/book/1108-m39-lasm-db-runtime-sqlite-module-extraction.md`).
   - [x] Extracted shared LASM DB runtime utilities into dedicated module `compiler/sec4-cli/src/lasm_db_runtime_common.rs` (`classify_lasm_db_runtime_error`, tx-handle allocator, postgres-client getter/reconnect), reducing DB runtime control-path coupling inside `main.rs` while preserving deterministic error mapping and reconnect behavior (`docs/book/1109-m39-lasm-db-runtime-common-module-extraction.md`).
   - [x] Moved sqlite schema bootstrap helper (`ensure_lasm_dynamic_db_records_sqlite_schema`) from `main.rs` into `compiler/sec4-cli/src/lasm_db_adapter_state.rs`, so sqlite adapter/runtime modules no longer depend on schema bootstrap defined in CLI orchestration file (`docs/book/1110-m39-lasm-db-sqlite-schema-helper-module-extraction.md`).
   - [x] Moved remaining DB handle/parameter normalization helpers (`parse_lasm_positive_i64`, `is_lasm_valid_db_cap_handle`, `normalize_lasm_db_params`) from `main.rs` into `compiler/sec4-cli/src/lasm_db_runtime_common.rs`, completing shared DB runtime utility extraction for adapter/runtime modules (`docs/book/1111-m39-lasm-db-runtime-common-helper-extraction.md`).
   - [x] Extracted the internal DB operation materialization dispatcher (`apply_lasm_internal_db_operation_materialization`) into dedicated module `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`, leaving `main.rs` with a thin delegator and preserving runtime semantics for `exec` / `execTx` / `queryOne` paths (`docs/book/1112-m39-lasm-db-runtime-dispatch-module-extraction.md`).
   - [x] Moved dispatch-only internal-header helpers (`take_lasm_internal_header_value`, `materialize_lasm_internal_header_value`) from `main.rs` into `compiler/sec4-cli/src/lasm_db_runtime_dispatch.rs`, completing co-location of DB internal-header materialization logic with DB dispatch execution paths (`docs/book/1113-m39-lasm-db-runtime-dispatch-helper-extraction.md`).
   - [x] Moved DB-record persistence dispatch helper (`persist_lasm_dynamic_db_records_to_disk`) from `main.rs` into `compiler/sec4-cli/src/lasm_db_adapter_state.rs`, so adapter selection and persistence routing are fully owned by adapter-state module boundaries (`docs/book/1114-m39-lasm-db-persist-dispatch-move-to-adapter-state.md`).
   - [x] Extracted DB operation-plan analysis/materialization helpers from `main.rs` into dedicated module `compiler/sec4-cli/src/lasm_db_plan.rs` (route-call DB operation extraction walkers + DB call classifiers + DB operation header-plan mapping), reducing orchestration-file DB AST-planning coupling while preserving intrinsic route-plan semantics (`docs/book/1115-m39-lasm-db-plan-module-extraction.md`).
   - [x] Moved DB operation-plan model types (`LasmSqlQueryPlan`, `LasmDbTxPlan`, `LasmDbOperationPlan`) from `main.rs` into `compiler/sec4-cli/src/lasm_db_plan.rs`, completing DB plan type+logic colocation and further reducing DB-only type surface in CLI orchestration (`docs/book/1116-m39-lasm-db-plan-type-extraction.md`).
   - [x] Added deterministic incremental DB record persistence for sqlite/postgres adapters and kept full-sync fallback on append failure, reducing full-store rewrite churn on hot DB operation paths (`docs/book/1117-m39-lasm-db-incremental-record-persistence.md`).
   - [x] Reused cached sqlite connection for append persistence (`persist_lasm_dynamic_db_record_append_to_sqlite`) with reconnect-on-failure fallback, removing per-append sqlite open/close churn (`docs/book/1118-m39-lasm-sqlite-connection-reuse-for-append-persistence.md`).
   - [x] Reused cached sqlite connection for runtime sqlite execution/query paths (`run_lasm_sqlite_exec`, `run_lasm_sqlite_exec_tx`, `run_lasm_sqlite_query_one`) with deterministic reconnect retry fallback, removing per-request sqlite open/close churn from LASM DB runtime materialization (`docs/book/1119-m39-lasm-sqlite-connection-reuse-for-runtime-exec-query.md`).
   - [x] Unified sqlite runtime reconnect behavior for both `exec*` and `queryOne` through a shared retry helper, so stale sqlite runtime connections now recover deterministically across read/write runtime operations while lock/parameter failures remain strict no-retry validation paths (`docs/book/1120-m39-lasm-sqlite-query-one-reconnect-retry.md`).
   - [x] Added sqlite runtime connection defaults for lock resilience and relational correctness (`SEC4_RT_LASM_SQLITE_BUSY_TIMEOUT_MS`, default `2000`, plus `PRAGMA foreign_keys = ON`) in shared sqlite connect bootstrap used by runtime and persistence paths (`docs/book/1121-m39-lasm-sqlite-runtime-busy-timeout-and-foreign-keys.md`).
   - [x] Added Postgres runtime session timeout defaults in shared connect bootstrap (`SEC4_RT_LASM_DB_POSTGRES_STATEMENT_TIMEOUT_MS`, default `5000`; `SEC4_RT_LASM_DB_POSTGRES_LOCK_TIMEOUT_MS`, default `2000`) so LASM Postgres adapter sessions are bounded for long-running/blocked operations (`docs/book/1122-m39-lasm-postgres-runtime-timeout-defaults.md`).
   - [x] Improved DB runtime timeout/lock error envelopes with deterministic timeout/conflict mapping codes (`DB.*_TIMEOUT`, `DB.*_LOCK_TIMEOUT`) for Postgres statement/lock timeouts and sqlite lock contention (`docs/book/1123-m39-lasm-db-timeout-lock-error-classification.md`).
6. Close alpha usability readiness with LASM+DB canonical example flow and reproducible operator docs.
   - [x] `examples/lasm-alpha-full` operator guide now documents real Postgres adapter run flow plus parameterized `db.queryOne` demos (typed values, literal-preserving placeholders, deterministic placeholder-arity failure), so DB-client behavior can be validated end-to-end from one canonical example (`docs/book/1069-m39-lasm-alpha-full-postgres-operator-flow.md`).
7. Start/expand WASM/browser execution track only after LASM+DB alpha path is stable and benchmarked.

### Post-DB execution lock (authoritative order)

When LASM DB intrinsic parity is complete, execution order is fixed:

1. LASM default backend.
2. LASM stability/load hardening.
3. DB adapter progression (SQLite first).
4. Adapter-layer extraction into packages/modules without semantic changes.
5. Alpha usability/reproducibility closure.
6. WASM/browser track.

## M37 - No-Stub Alpha Sprint (Execution-Complete; Tag Pending)

### Goal

- Finish implementation-first de-stubbing required for no-stub alpha readiness.

### M37-S1 redirect + policy parity hardening acceptance criteria

- Policy model persists active runtime keys: `json.max_bytes`, `json.max_depth`, `net.public.max_redirects`.
- Runtime outbound net path supports deterministic redirect policy behavior with explicit failure codes for:
  - redirect forbidden,
  - redirect limit exceeded,
  - invalid redirect location.
- Runtime harness coverage validates redirect denied-by-default, allowed-with-env, and redirect-limit failure paths.

### M37-S1 tracking (live status)

- [x] Policy model/parsing persistence for `json.max_bytes`, `json.max_depth`, `net.public.max_redirects` implemented.
- [x] Runtime redirect handling implemented for outbound public/internal GET intrinsics.
- [x] Clang-gated redirect runtime tests added and passing.
- [x] Book chapter documenting M37-S1 implementation added.

### M37-S2 runtime JSON semantic hardening acceptance criteria

- `req.json` and `json.decode` enforce non-zero schema descriptors with deterministic diagnostics.
- `json.decode` on active request bodies requires a successful `req.json` gate and blocks invalid-gate flows deterministically.
- Tracked schema descriptor mismatch between `req.json` and `json.decode` emits deterministic `JSON.SCHEMA_MISMATCH`.
- `json.encode` and JSON responders (`res.json`/`res.ok`/`res.okMeta`) materialize real envelope payload fragments instead of placeholder `{}` payloads.
- c-bin compile path remains deterministic for descriptor-literal schema callsites used by current examples.

### M37-S2 tracking (live status)

- [x] Runtime request state now tracks active JSON schema handle for gate sequencing.
- [x] Runtime decode path now enforces `JSON.GATE_REQUIRED` and `JSON.GATE_FAILED` for request-body decode order.
- [x] Runtime decode path now enforces tracked schema mismatch diagnostics (`JSON.SCHEMA_MISMATCH`).
- [x] Runtime encode + JSON responders now materialize deterministic `data` and `meta` payloads.
- [x] Added runtime harness coverage for gate-required/gate-failed/schema-mismatch decode paths.
- [x] Added compile compatibility bridge flag (`-Wno-int-conversion`) to keep schema descriptor literal callsites compiling in current ABI stage.
- [x] Book chapter documenting M37-S2 implementation added.

### M37-S3 middleware policy materialization hardening acceptance criteria

- `sec4 run` materializes active policy into runtime env for middleware/security paths:
  - CORS (`enabled`, `allowed_origins`, `allow_credentials`, `require_vary_origin`),
  - security headers (`enabled`, `x_content_type_options`, `x_frame_options`, `referrer_policy`),
  - CSRF (`enabled`, `mode`, `protected_methods`),
  - auth (`mode`).
- Runtime `cors.fromPolicy` / `sec.defaultHeaders` / `csrf.fromPolicy` / `auth.fromPolicy` handles apply those policy values deterministically via `with*` middleware hooks.
- Runtime response emission uses dynamic middleware-derived headers (not fixed constants) for normal and preflight responses.
- `sec4 run` integration coverage proves policy materialization behavior for:
  - CORS response headers,
  - disabled security headers,
  - disabled CSRF protection on protected methods.

### M37-S3 tracking (live status)

- [x] Runtime router middleware state expanded for policy-materialized CORS/security/CSRF/auth fields.
- [x] Runtime middleware `with*` implementations now honor policy handles and disabled-path behavior.
- [x] Runtime response path now emits policy-driven CORS and security headers dynamically.
- [x] `sec4 run` now exports middleware policy env keys (plus redirect/json runtime policy bridge keys) into c-bin execution.
- [x] Added run-command integration tests:
  - `run_command_oneshot_applies_cors_from_policy`
  - `run_command_oneshot_disables_security_headers_from_policy`
  - `run_command_oneshot_disables_csrf_from_policy`
- [x] Book chapter documenting M37-S3 implementation added.

### M37-S4 structured runtime log emission path acceptance criteria

- Runtime log intrinsics emit structured JSON events instead of handle-only placeholders.
- Emitted log events include deterministic baseline fields:
  - `timeMs`
  - `level`
  - `traceId`
  - `event`
- Runtime log builder helpers (`withAttr`, `withHttp`, `withError`) materialize into emitted JSON payload fragments.
- Runtime log emission remains deterministic and policy-safe by default:
  - redaction helpers emit explicit redaction markers,
  - output can be disabled via `SEC4_RT_LOG_OUTPUT=off`.

### M37-S4 tracking (live status)

- [x] Runtime log functions now maintain structured event state and emit JSON lines on `log_any`.
- [x] Runtime log value helpers now materialize JSON-safe values (`str`, `i64`, `bool`, `redacted`, `attrRedacted`).
- [x] Runtime log builder helpers now attach attrs/http/error metadata into emitted log events.
- [x] Added c-bin log intrinsic test assertions for emitted structured log fields.
- [x] Added dedicated clang-gated runtime harness coverage:
  - `c_bin_runtime_log_builders_emit_structured_json_when_clang_available`.
- [x] Book chapter documenting M37-S4 implementation added.

### M37-S5 final no-stub verification pass acceptance criteria

- Full runtime/compiler verification matrix runs green on `main`:
  - `cargo test -p sec4 --test json_output`
  - `cargo test -p sec4 --test commands`
  - `cargo test -p sec4 --test alpha_smoke`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- Verification evidence is summarized as a residual-gap note for alpha decision.

### M37-S5 tracking (live status)

- [x] `cargo test -p sec4 --test json_output` passed (`161 passed; 0 failed`).
- [x] `cargo test -p sec4 --test commands` passed (`36 passed; 0 failed`).
- [x] `cargo test -p sec4 --test alpha_smoke` passed (`2 passed; 0 failed`).
- [x] `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source` passed.
- [x] Residual gaps documented for alpha decision:
  - full HTTPS runtime behavior still depends on OpenSSL toolchain availability; `--tls-backend auto` falls back to non-TLS when OpenSSL linkage is unavailable.
- [x] Book chapter documenting M37-S5 verification pass added.

### M37-S6 alpha publish checklist delta + tag decision package acceptance criteria

- Publish-checklist delta script consumes `build/release-alpha-gate/` artifacts and emits deterministic JSON/markdown summary.
- Delta enforces identity-hash consistency (`policyHash`, `compilerHash`, `runtimeHash`) across release summaries, metadata, and audit reports for canonical samples.
- Delta emits deterministic alpha tag decision (`GO`/`HOLD`) with explicit next action guidance.

### M37-S6 tracking (live status)

- [x] Added `scripts/build-m37-alpha-publish-checklist-delta.sh`.
- [x] Added contract coverage `scripts/test-build-m37-alpha-publish-checklist-delta.sh`.
- [x] Added book chapter documenting M37-S6 publish-checklist delta artifact.
- [x] Updated CLI build/run TLS mode defaults to `--tls-backend auto` with deterministic OpenSSL fallback behavior.

### M37-S7 alpha release-note package acceptance criteria

- Release-note package script consumes M37-S6 checklist artifact and emits deterministic JSON/markdown outputs.
- Package computes deterministic `releaseState` (`alpha-ready` or `alpha-hold`) from checklist `overall` + `tagDecision`.
- Package includes canonical no-stub baseline statement, identity hashes, sample summary, known limits, and next action guidance.

### M37-S7 tracking (live status)

- [x] Added `scripts/build-m37-alpha-release-notes.sh`.
- [x] Added contract coverage `scripts/test-build-m37-alpha-release-notes.sh`.
- [x] Added book chapter documenting M37-S7 alpha release-note package.

### M37-S8 alpha tag decision execution record acceptance criteria

- Tag-decision record script consumes both M37-S6 and M37-S7 artifacts and emits deterministic JSON/markdown decision evidence.
- GO decision is blocked unless prerequisites are `PASS/GO` + `alpha-ready`.
- Record includes closure outcome, identity stamps, and explicit next action.

### M37-S8 tracking (live status)

- [x] Added `scripts/build-m37-alpha-tag-decision-record.sh`.
- [x] Added contract coverage `scripts/test-build-m37-alpha-tag-decision-record.sh`.
- [x] Added book chapter documenting M37-S8 alpha tag decision record.

### M37-S9 alpha tag gate execution evidence refresh acceptance criteria

- Live release gate execution evidence is refreshed from current `dev`.
- M37-S6/S7/S8 artifacts are rebuilt from refreshed gate artifacts and remain `GO`-compatible.
- Roadmap readiness/next actions are aligned to refreshed evidence state.

### M37-S9 tracking (live status)

- [x] Executed `scripts/release-alpha-gate.sh --skip-tests` with passing closure + sample determinism/audit chain.
- [x] Rebuilt M37-S6/S7/S8 artifacts and confirmed `PASS/GO`, `alpha-ready`, `GO/PASS`.
- [x] Added book chapter documenting M37-S9 evidence refresh (`docs/book/883-m37-alpha-tag-gate-execution-evidence-refresh.md`).

### M37-S10 full alpha gate execution with tests acceptance criteria

- Full release gate execution path (`scripts/release-alpha-gate.sh`) passes with complete test phase enabled.
- Rebuilt M37-S6/S7/S8 artifacts from full-gate outputs preserve `PASS/GO` continuity.
- Alpha closure actions are advanced to tag execution and post-tag verification.

### M37-S10 tracking (live status)

- [x] Executed `scripts/release-alpha-gate.sh` (with tests) and observed final gate `PASS`.
- [x] Rebuilt M37-S6/S7/S8 artifacts from full-gate evidence and revalidated `PASS/GO`, `alpha-ready GO`, `GO PASS`.
- [x] Added book chapter documenting M37-S10 full-gate execution evidence (`docs/book/884-m37-full-alpha-gate-execution-with-tests.md`).

### M37-S11 alpha tag execution + post-tag verification checklist closure acceptance criteria

- Alpha tag is created from the latest `M37-S8` `GO` decision baseline using workflow-compatible naming (`v0.1.0-alpha*`).
- Post-tag verification chain passes:
  - release gate evidence refresh,
  - release promotion input verification,
  - publish-manifest generation and verification.
- Alpha closure status is promoted to `WASM_START_GATE` `OPEN` with explicit evidence pointers.

### M37-S11 tracking (live status)

- [x] Created and pushed alpha tag `v0.1.0-alpha.1` from `dev` `HEAD`.
- [x] Refreshed release-gate stability for closure execution:
  - one-shot C harness readers force accepted sockets back to blocking mode to remove `WouldBlock` flakes.
  - `scripts/release-alpha-gate.sh` now runs `cargo test` in deterministic serialized mode by default (`CARGO_BUILD_JOBS=1`, `RUST_TEST_THREADS=1`, overrideable via env).
- [x] Created and pushed alpha tag `v0.1.0-alpha.2` from `dev` `HEAD` after closure refresh.
- [x] Re-executed post-tag verifier chain for `v0.1.0-alpha.2`:
  - `scripts/release-alpha-gate.sh --skip-tests`
  - `scripts/verify-release-promotion-inputs.sh`
  - `scripts/generate-release-publish-manifest.sh`
  - `scripts/verify-release-publish-manifest.sh`
- [x] Executed post-tag verifier chain:
  - `scripts/release-alpha-gate.sh --skip-tests`
  - `scripts/verify-release-promotion-inputs.sh`
  - `scripts/generate-release-publish-manifest.sh`
  - `scripts/verify-release-publish-manifest.sh`
- [x] Added book chapter documenting M37-S11 tag/post-tag closure (`docs/book/885-m37-alpha-tag-execution-and-post-tag-verification-closure.md`).
- [x] Added book chapter documenting alpha.2 closure refresh (`docs/book/1003-m37-alpha2-release-closure-refresh.md`).

### M38-S1 outbound HTTP chunked-body decoding hardening acceptance criteria

- Runtime outbound HTTP extraction decodes `Transfer-Encoding: chunked` payloads into plain body bytes for both HTTP and HTTPS response paths.
- Chunked decoding enforces deterministic validation errors for malformed chunk framing and resource limits.
- Existing content-length response handling and redirect behavior remain green.
- Clang-gated runtime harness coverage proves decoded chunked body roundtrip through `sec4_rt_http_get_internal`.

### M38-S1 tracking (live status)

- [x] Runtime outbound extractor now parses `Transfer-Encoding` + `Content-Length` headers and decodes chunked bodies.
- [x] Added chunked runtime harness helper + test:
  - `c_bin_runtime_internal_get_chunked_body_is_decoded_when_clang_available`.
- [x] Verified no regression on existing internal GET roundtrip + redirect allow paths.

### M38-S2 chunked edge-case diagnostics hardening acceptance criteria

- Malformed chunked framing maps to deterministic runtime error code `NET.CHUNK_INVALID`.
- Chunked valid-path decoding remains green for internal-net runtime harness.
- Redirect/internal-net existing runtime tests remain green after diagnostic mapping update.

### M38-S2 tracking (live status)

- [x] Runtime outbound read error mapper now handles `read_status=-7` as `NET.CHUNK_INVALID`.
- [x] Added malformed chunked harness helper + test:
  - `c_bin_runtime_internal_get_malformed_chunked_returns_chunk_invalid_when_clang_available`.
- [x] Revalidated chunked success path and redirect-allow regression path.

### M38-S3 chunk trailer/extension parser strictness acceptance criteria

- Chunked responses with chunk extensions and trailer headers decode successfully on outbound runtime path.
- Missing trailer terminator after zero-size chunk is rejected with deterministic `NET.CHUNK_INVALID`.
- Existing malformed-chunk and redirect-allow harness tests stay green.

### M38-S3 tracking (live status)

- [x] Added runtime strictness: zero-chunk trailer parsing now requires explicit trailer termination.
- [x] Added harness helpers + tests:
  - `c_bin_runtime_internal_get_chunked_trailers_roundtrip_when_clang_available`
  - `c_bin_runtime_internal_get_chunked_missing_trailer_terminator_returns_chunk_invalid_when_clang_available`
- [x] Revalidated malformed-chunk invalid-code and redirect-allow regression paths.

### M38-S4 outbound HTTP oversized-header diagnostics hardening acceptance criteria

- Runtime outbound parser distinguishes response-header overflow from generic response-invalid parsing failures.
- Oversized response headers map to deterministic runtime error code `NET.RESPONSE_HEADERS_TOO_LARGE`.
- Existing chunked/redirect outbound runtime paths remain green after diagnostic split.

### M38-S4 tracking (live status)

- [x] Runtime outbound extractor now returns dedicated read status for oversized headers and maps it to `NET.RESPONSE_HEADERS_TOO_LARGE`.
- [x] Added oversized-header harness helper + test:
  - `c_bin_runtime_internal_get_oversized_headers_returns_deterministic_code_when_clang_available`.
- [x] Revalidated c-backend emit contract and existing internal-net outbound regression paths.

### M38-S5 outbound HTTP framing-conflict diagnostics hardening acceptance criteria

- Runtime outbound parser rejects conflicting response framing signals (`Transfer-Encoding: chunked` combined with `Content-Length`).
- Duplicate `Content-Length` headers with inconsistent values are rejected deterministically.
- Conflicting framing failures map to deterministic runtime code `NET.RESPONSE_FRAMING_CONFLICT`.

### M38-S5 tracking (live status)

- [x] Runtime outbound parser now returns dedicated read status for framing conflicts.
- [x] Outbound read error mapping now emits deterministic `NET.RESPONSE_FRAMING_CONFLICT`.
- [x] Added conflicting-framing harness helper + test:
  - `c_bin_runtime_internal_get_conflicting_framing_headers_returns_deterministic_code_when_clang_available`.
- [x] Revalidated chunked success, redirect allow path, and c-backend emit contract.

### M38-S6 outbound HTTP parser token/status strictness diagnostics acceptance criteria

- Runtime rejects invalid HTTP status-line shape with deterministic `NET.STATUS_LINE_INVALID`.
- Runtime rejects invalid transfer-encoding token ordering with deterministic `NET.TRANSFER_ENCODING_INVALID`.
- Existing chunked decode and redirect runtime behavior stays green after strictness checks.

### M38-S6 tracking (live status)

- [x] Runtime status-line parser now enforces `HTTP/1.0` / `HTTP/1.1` prefix and strict 3-digit status parsing.
- [x] Runtime transfer-encoding parser now rejects token payload after `chunked`.
- [x] Outbound read error mapping now emits:
  - `NET.STATUS_LINE_INVALID`
  - `NET.TRANSFER_ENCODING_INVALID`
- [x] Added strictness harness helpers + tests:
  - `c_bin_runtime_internal_get_invalid_status_line_returns_deterministic_code_when_clang_available`
  - `c_bin_runtime_internal_get_invalid_transfer_encoding_returns_deterministic_code_when_clang_available`
- [x] Revalidated chunked success path, redirect allow path, and c-backend emit contract.

### M38-S7 outbound HTTP response-version/header-line diagnostics hardening acceptance criteria

- Unsupported outbound response HTTP versions are rejected deterministically with `NET.RESPONSE_VERSION_UNSUPPORTED`.
- Malformed outbound response header lines are rejected deterministically with `NET.HEADER_LINE_INVALID`.
- Existing status-line/transfer-encoding strictness and chunked/redirect runtime paths remain green.

### M38-S7 tracking (live status)

- [x] Runtime outbound parser now emits `NET.RESPONSE_VERSION_UNSUPPORTED` for non-`HTTP/1.0|1.1` response prefixes.
- [x] Runtime header-line validation now rejects invalid header shape/name tokens and maps to `NET.HEADER_LINE_INVALID`.
- [x] Added strictness harness helpers + tests:
  - `c_bin_runtime_internal_get_unsupported_version_returns_deterministic_code_when_clang_available`
  - `c_bin_runtime_internal_get_invalid_header_line_returns_deterministic_code_when_clang_available`
- [x] Revalidated existing `NET.STATUS_LINE_INVALID` + `NET.TRANSFER_ENCODING_INVALID` paths and c-backend emit contract.

### M38-S8 outbound HTTP duplicate-header/retryability diagnostics hardening acceptance criteria

- Runtime normalizes duplicate `Location` headers and rejects conflicting values deterministically.
- Runtime validates `Retry-After` numeric values and rejects malformed/conflicting duplicates deterministically.
- Duplicate-header/retryability diagnostics map to stable runtime error codes.

### M38-S8 tracking (live status)

- [x] Runtime now rejects conflicting duplicate `Location` header values with deterministic `NET.REDIRECT_LOCATION_CONFLICT`.
- [x] Runtime now rejects malformed or conflicting duplicate `Retry-After` values with deterministic `NET.RETRY_AFTER_INVALID`.
- [x] Added strictness harness helpers + tests:
  - `c_bin_runtime_internal_get_conflicting_location_headers_returns_deterministic_code_when_clang_available`
  - `c_bin_runtime_internal_get_invalid_retry_after_returns_deterministic_code_when_clang_available`
- [x] Revalidated response-version/header-line strictness, redirect-allow path, and c-backend emit contract.

### M38-S9 outbound HTTP duplicate-content-length diagnostics hardening acceptance criteria

- Runtime normalizes comma-delimited duplicate `Content-Length` values when all entries match.
- Malformed or conflicting duplicate `Content-Length` values are rejected deterministically with dedicated diagnostics.
- Existing retryability/redirect/status-line/header-line hardening paths remain green.

### M38-S9 tracking (live status)

- [x] Runtime now accepts normalized duplicate `Content-Length` list values (for example `Content-Length: 4, 4`).
- [x] Runtime now rejects malformed/conflicting duplicate `Content-Length` values with deterministic `NET.CONTENT_LENGTH_INVALID`.
- [x] Added strictness harness helpers + tests:
  - `c_bin_runtime_internal_get_duplicate_content_length_equal_is_accepted_when_clang_available`
  - `c_bin_runtime_internal_get_duplicate_content_length_conflict_returns_deterministic_code_when_clang_available`
- [x] Revalidated retry-after diagnostics, redirect-allow path, and c-backend emit contract.

### M38-S10 outbound HTTP transfer-encoding whitelist diagnostics hardening acceptance criteria

- Runtime accepts only `Transfer-Encoding: chunked` on outbound response parsing.
- Unsupported transfer-encoding tokens map to deterministic `NET.TRANSFER_ENCODING_UNSUPPORTED`.
- Mixed-token invalid ordering diagnostics (`NET.TRANSFER_ENCODING_INVALID`) remain deterministic.

### M38-S10 tracking (live status)

- [x] Runtime outbound parser now treats non-`chunked` transfer-encoding tokens as deterministic unsupported failures.
- [x] Outbound read error mapping now emits deterministic `NET.TRANSFER_ENCODING_UNSUPPORTED`.
- [x] Added strictness harness helper + test:
  - `c_bin_runtime_internal_get_unsupported_transfer_encoding_returns_deterministic_code_when_clang_available`
- [x] Revalidated mixed-token invalid ordering, chunked decode success, redirect-allow path, and c-backend emit contract.

### M38-S11 outbound HTTP header-whitespace/obs-fold diagnostics hardening acceptance criteria

- Runtime rejects obs-fold response headers (continuation lines beginning with SP/HTAB) with deterministic diagnostics.
- Runtime rejects header-name whitespace before colon with deterministic diagnostics.
- Existing header-line invalid and transfer-encoding strictness behavior remains green.

### M38-S11 tracking (live status)

- [x] Runtime now maps obs-fold header continuation lines to deterministic `NET.HEADER_OBS_FOLD_INVALID`.
- [x] Runtime now maps header-name whitespace before colon to deterministic `NET.HEADER_WHITESPACE_INVALID`.
- [x] Added strictness harness helpers + tests:
  - `c_bin_runtime_internal_get_obs_fold_header_returns_deterministic_code_when_clang_available`
  - `c_bin_runtime_internal_get_header_whitespace_before_colon_returns_deterministic_code_when_clang_available`
- [x] Revalidated transfer-encoding unsupported/invalid diagnostics and c-backend emit contract.

### M38-S12 outbound HTTP header-value/control-char diagnostics hardening acceptance criteria

- Runtime rejects malformed header sections (missing/invalid terminator framing) with deterministic diagnostics.
- Runtime rejects response header values containing invalid control characters with deterministic diagnostics.
- Existing obs-fold/whitespace/transfer-encoding parser hardening paths remain green.

### M38-S12 tracking (live status)

- [x] Runtime now maps malformed header sections to deterministic `NET.HEADER_SECTION_INVALID`.
- [x] Runtime now maps invalid header-value control characters to deterministic `NET.HEADER_VALUE_CONTROL_INVALID`.
- [x] Added strictness harness helpers + tests:
  - `c_bin_runtime_internal_get_invalid_header_section_returns_deterministic_code_when_clang_available`
  - `c_bin_runtime_internal_get_header_value_control_char_returns_deterministic_code_when_clang_available`
- [x] Revalidated obs-fold and transfer-encoding diagnostics plus c-backend emit contract.

### M38-S13 outbound HTTP content-type token diagnostics hardening acceptance criteria

- Runtime validates outbound response `Content-Type` media-type tokens with deterministic diagnostics.
- Invalid/malformed `Content-Type` values map to stable runtime code.
- Existing header-section/control-char/obs-fold diagnostics remain green.

### M38-S13 tracking (live status)

- [x] Runtime now validates `Content-Type` type/subtype token structure during outbound response parsing.
- [x] Invalid `Content-Type` values now map to deterministic `NET.CONTENT_TYPE_INVALID`.
- [x] Added strictness harness helper + test:
  - `c_bin_runtime_internal_get_invalid_content_type_returns_deterministic_code_when_clang_available`
- [x] Revalidated header-section and obs-fold diagnostics plus c-backend emit contract.

### M38-S14 outbound HTTP redirect-target normalization diagnostics hardening acceptance criteria

- Runtime normalizes relative redirect targets deterministically before follow-up request construction.
- Relative redirect targets that escape root or contain invalid target payload are rejected with deterministic diagnostics.
- Existing redirect allow/limit/conflict diagnostics remain green.

### M38-S14 tracking (live status)

- [x] Runtime redirect resolver now returns structured outcomes (ok/invalid/target-invalid) and normalizes relative path segments.
- [x] Added deterministic `NET.REDIRECT_TARGET_INVALID` mapping for invalid relative redirect targets.
- [x] Added redirect-target harness helpers + tests:
  - `c_bin_runtime_internal_get_relative_redirect_is_normalized_when_clang_available`
  - `c_bin_runtime_internal_get_relative_redirect_invalid_target_returns_deterministic_code_when_clang_available`
- [x] Revalidated redirect allow path, conflicting-location diagnostics, and c-backend emit contract.

### M38-S15 outbound HTTP redirect-authority host-token diagnostics hardening acceptance criteria

- Runtime validates redirect authority host tokens with deterministic diagnostics.
- Redirect authority normalization canonicalizes host casing and default-port rendering after parse.
- Invalid redirect hosts are surfaced with dedicated deterministic runtime code.

### M38-S15 tracking (live status)

- [x] Redirect resolver now validates host tokens via strict label-character checks.
- [x] Redirect resolver now canonicalizes resolved redirect authority (`host` lowercase, default port elided).
- [x] Added deterministic `NET.REDIRECT_HOST_INVALID` mapping for invalid redirect hosts.
- [x] Added redirect-authority harness helpers + tests:
  - `c_bin_runtime_internal_get_absolute_redirect_upper_host_succeeds_when_clang_available`
  - `c_bin_runtime_internal_get_redirect_host_invalid_returns_deterministic_code_when_clang_available`
- [x] Revalidated relative redirect normalization, redirect allow path, and c-backend emit contract.

### M38-S16 outbound HTTP redirect fragment/target-character diagnostics hardening acceptance criteria

- Runtime redirect resolver distinguishes fragment-invalid targets from generic/character-invalid targets.
- Redirect targets containing fragments are rejected with deterministic diagnostics.
- Redirect targets containing invalid characters are rejected with deterministic diagnostics.

### M38-S16 tracking (live status)

- [x] Redirect resolver now returns dedicated statuses for fragment-invalid and target-character-invalid outcomes.
- [x] Added deterministic `NET.REDIRECT_FRAGMENT_INVALID` mapping for fragment-bearing redirect targets.
- [x] Added deterministic `NET.REDIRECT_TARGET_CHAR_INVALID` mapping for redirect targets containing invalid characters.
- [x] Added redirect-target harness helpers + tests:
  - `c_bin_runtime_internal_get_redirect_fragment_invalid_returns_deterministic_code_when_clang_available`
  - `c_bin_runtime_internal_get_redirect_target_char_invalid_returns_deterministic_code_when_clang_available`
- [x] Revalidated host-invalid and normalized-relative redirect paths plus c-backend emit contract.

### M38-S17 outbound HTTP redirect query-component diagnostics hardening acceptance criteria

- Runtime validates redirect query components deterministically and rejects malformed query payload.
- Invalid percent-encoding sequences in redirect query components are rejected with dedicated diagnostics.
- Repeated query separators in redirect targets are rejected with dedicated diagnostics.

### M38-S17 tracking (live status)

- [x] Redirect resolver now validates query component shape and percent escapes deterministically.
- [x] Added deterministic `NET.REDIRECT_QUERY_INVALID` mapping for malformed redirect query components.
- [x] Added redirect-query harness helpers + tests:
  - `c_bin_runtime_internal_get_redirect_query_percent_invalid_returns_deterministic_code_when_clang_available`
  - `c_bin_runtime_internal_get_redirect_query_separator_invalid_returns_deterministic_code_when_clang_available`
- [x] Revalidated fragment/target-char diagnostics and c-backend emit contract.

### M38-S18 outbound HTTP absolute-redirect scope diagnostics hardening acceptance criteria

- Runtime distinguishes redirect parse failures from redirect scope-revalidation failures.
- Cross-scope redirect targets are rejected with dedicated deterministic diagnostics.
- Existing redirect allow/limit and query/fragment/target diagnostics remain green.

### M38-S18 tracking (live status)

- [x] Redirect follow-up runtime path now emits deterministic `NET.REDIRECT_SCOPE_INVALID` when resolved redirect target violates scope policy.
- [x] Added cross-scope redirect harness helper + test:
  - `c_bin_runtime_internal_get_redirect_scope_invalid_returns_deterministic_code_when_clang_available`
- [x] Revalidated redirect allow path and c-backend emit contract.

### M38-S19 outbound HTTP redirect revalidation-policy diagnostics hardening acceptance criteria

- Runtime fails closed for cross-scope redirect targets when explicit revalidation bypass is requested.
- Cross-scope redirect with revalidation disabled emits dedicated deterministic diagnostics.
- Existing scope-invalid and redirect-allow paths remain green.

### M38-S19 tracking (live status)

- [x] Redirect follow-up path now evaluates scope deterministically before follow-up even when revalidation toggle is disabled.
- [x] Added deterministic `NET.REDIRECT_SCOPE_REVALIDATION_DISABLED` mapping for cross-scope redirect targets under disabled revalidation.
- [x] Added dedicated harness test:
  - `c_bin_runtime_internal_get_redirect_scope_revalidation_disabled_returns_deterministic_code_when_clang_available`
- [x] Revalidated existing scope-invalid/redirect-allow flows and c-backend emit contract.

### M38-S20 outbound HTTP redirect missing-location diagnostics hardening acceptance criteria

- Runtime rejects redirect status responses that omit `Location` with a dedicated deterministic diagnostic.
- Existing redirect location conflict diagnostics stay unchanged.
- Existing scope/query/fragment/target redirect diagnostics remain green.

### M38-S20 tracking (live status)

- [x] Redirect follow-up path now emits deterministic `NET.REDIRECT_LOCATION_MISSING` when a redirect response omits `Location`.
- [x] Added missing-location harness helper + test:
  - `c_bin_runtime_internal_get_redirect_location_missing_returns_deterministic_code_when_clang_available`
- [x] Revalidated scope-invalid diagnostics and c-backend emit contract.

### M38-S21 outbound HTTP redirect scheme-downgrade diagnostics hardening acceptance criteria

- Redirect resolver rejects `https -> http` redirect downgrade targets with deterministic status.
- Runtime follow-up flow maps downgrade status to dedicated deterministic diagnostics.
- Existing location-missing and scope diagnostics remain green.

### M38-S21 tracking (live status)

- [x] Redirect resolver now emits `SEC4_RT_REDIRECT_RESOLVE_DOWNGRADE_INVALID` for `https -> http` downgrade targets.
- [x] Runtime follow-up flow now maps downgrade status to deterministic `NET.REDIRECT_DOWNGRADE_FORBIDDEN`.
- [x] Added resolver harness test:
  - `c_bin_runtime_redirect_resolver_rejects_https_to_http_downgrade_when_clang_available`
- [x] Revalidated location-missing diagnostics and c-backend emit contract.

### M38-S22 outbound HTTP redirect absolute-scheme diagnostics hardening acceptance criteria

- Runtime detects absolute redirect targets with unsupported schemes and emits dedicated deterministic diagnostics.
- Absolute-scheme diagnostics are distinct from generic redirect-invalid and target-invalid classes.
- Existing downgrade and location-missing diagnostics remain green.

### M38-S22 tracking (live status)

- [x] Redirect resolver now emits `SEC4_RT_REDIRECT_RESOLVE_SCHEME_INVALID` for unsupported absolute redirect schemes.
- [x] Runtime follow-up flow now maps scheme status to deterministic `NET.REDIRECT_SCHEME_INVALID`.
- [x] Added invalid-scheme harness helper + test:
  - `c_bin_runtime_internal_get_redirect_scheme_invalid_returns_deterministic_code_when_clang_available`
- [x] Revalidated downgrade/location-missing diagnostics and c-backend emit contract.

### M38-S23 outbound HTTP redirect configurable-downgrade-policy diagnostics hardening acceptance criteria

- Redirect resolver supports explicit configurable allow/deny for `https -> http` downgrade transitions.
- Secure default remains deny with deterministic downgrade diagnostic.
- Allow-mode resolves downgrade targets deterministically for controlled environments.

### M38-S23 tracking (live status)

- [x] Redirect resolver signature now accepts explicit `allow_https_downgrade` policy input.
- [x] Runtime follow-up flow now reads `SEC4_RT_NET_ALLOW_HTTPS_DOWNGRADE` (default deny).
- [x] Added resolver allow/deny harness coverage:
  - `c_bin_runtime_redirect_resolver_rejects_https_to_http_downgrade_when_clang_available`
  - `c_bin_runtime_redirect_resolver_allows_https_to_http_downgrade_when_enabled_when_clang_available`
- [x] Revalidated invalid-scheme diagnostics and c-backend emit contract.

### M38-S24 outbound HTTP redirect policy-surface validation hardening acceptance criteria

- Redirect policy env values are validated strictly with deterministic failure diagnostics.
- Invalid boolean or integer redirect policy values no longer silently fall back.
- Existing downgrade and invalid-scheme diagnostics remain green.

### M38-S24 tracking (live status)

- [x] Added strict redirect-policy env parsers for bool and bounded non-negative integer values.
- [x] Redirect flow now emits deterministic `NET.REDIRECT_POLICY_INVALID` for invalid redirect policy env values.
- [x] Added invalid-policy harness test:
  - `c_bin_runtime_internal_get_redirect_allow_redirects_policy_invalid_returns_deterministic_code_when_clang_available`
- [x] Revalidated configurable downgrade and invalid-scheme diagnostics plus c-backend emit contract.

### M38-S25 outbound HTTP redirect hop-budget diagnostics hardening acceptance criteria

- Runtime detects redirect cycles deterministically and rejects cyclic redirect chains.
- Redirect hop accounting remains deterministic under configured max-redirect budget.
- Existing policy-invalid and invalid-scheme diagnostics remain green.

### M38-S25 tracking (live status)

- [x] Redirect follow-up path now tracks visited redirect URLs and detects cycles deterministically.
- [x] Runtime now emits deterministic `NET.REDIRECT_CYCLE_DETECTED` on cyclic redirect chains.
- [x] Added redirect-cycle harness helper + test:
  - `c_bin_runtime_internal_get_redirect_cycle_returns_deterministic_code_when_clang_available`
- [x] Revalidated redirect-policy-invalid and invalid-scheme diagnostics plus c-backend emit contract.

### M38-S26 outbound HTTP redirect-hop budget diagnostics hardening acceptance criteria

- Runtime splits redirect hop-limit exhaustion from runtime hard-cap exhaustion with deterministic diagnostics.
- Operator-configured redirect limit and runtime safety cap remain deterministic and testable.
- Existing cycle detection and policy-invalid diagnostics remain green.

### M38-S26 tracking (live status)

- [x] Added runtime redirect policy cap (`SEC4_RT_MAX_OUTBOUND_HTTP_REDIRECTS`) separate from configurable policy max.
- [x] Runtime now emits deterministic `NET.REDIRECT_CAP_LIMIT` when runtime safety cap is exhausted.
- [x] Added cap-limit harness helper + test:
  - `c_bin_runtime_internal_get_redirect_cap_limit_returns_deterministic_code_when_clang_available`
- [x] Revalidated regular `NET.REDIRECT_LIMIT`, cycle diagnostics, and c-backend emit contract.

### M38-S27 outbound HTTP redirect-policy value diagnostics hardening acceptance criteria

- Redirect policy parsing emits field-specific deterministic diagnostics per invalid policy key.
- Shared generic policy-invalid diagnostics are replaced for redirect policy surface.
- Existing cap-limit and cycle diagnostics remain green.

### M38-S27 tracking (live status)

- [x] Redirect policy parser now emits field-specific diagnostics:
  - `NET.REDIRECT_POLICY_ALLOW_REDIRECTS_INVALID`
  - `NET.REDIRECT_POLICY_MAX_REDIRECTS_INVALID`
  - `NET.REDIRECT_POLICY_REVALIDATE_REDIRECTS_INVALID`
  - `NET.REDIRECT_POLICY_ALLOW_DOWNGRADE_INVALID`
- [x] Updated existing policy-invalid harness to assert allow-redirects-specific code.
- [x] Added max-redirects-invalid harness test:
  - `c_bin_runtime_internal_get_redirect_max_redirects_policy_invalid_returns_deterministic_code_when_clang_available`
- [x] Revalidated cap-limit diagnostics and c-backend emit contract.

### M38-S28 outbound HTTP redirect-policy revalidate/downgrade diagnostics harness expansion acceptance criteria

- Field-specific redirect policy diagnostics for revalidate/downgrade keys are covered by dedicated harness tests.
- Harness coverage includes allow-redirects/max-redirects/revalidate/downgrade invalid-value paths.
- Existing cap-limit and cycle diagnostics remain green.

### M38-S28 tracking (live status)

- [x] Added revalidate-policy-invalid harness test:
  - `c_bin_runtime_internal_get_redirect_revalidate_policy_invalid_returns_deterministic_code_when_clang_available`
- [x] Added allow-downgrade-policy-invalid harness test:
  - `c_bin_runtime_internal_get_redirect_allow_downgrade_policy_invalid_returns_deterministic_code_when_clang_available`
- [x] Revalidated max-redirects-policy-invalid diagnostics and c-backend emit contract.

### M38-S29 outbound HTTP redirect policy error-envelope detail hardening acceptance criteria

- Redirect policy invalid diagnostics include deterministic structured `details` entries identifying invalid policy key.
- Structured details are stable across allow-redirects/max-redirects/revalidate/downgrade invalid paths.
- Existing cap-limit and cycle diagnostics remain green.

### M38-S29 tracking (live status)

- [x] Added structured error helper path for policy-invalid diagnostics with deterministic `details`.
- [x] Redirect policy invalid responses now include `details:[{"key":"policyKey","value":"..."}]`.
- [x] Extended policy-invalid harness tests to assert structured details for all redirect policy keys.
- [x] Revalidated c-backend emit contract.

### M38-S30 outbound HTTP max-redirects format/range diagnostics hardening acceptance criteria

- `SEC4_RT_NET_PUBLIC_MAX_REDIRECTS` invalid format and invalid range are split into distinct deterministic diagnostics.
- Structured `policyKey` details remain present in both format and range failure paths.
- Existing revalidate/downgrade policy diagnostics remain green.

### M38-S30 tracking (live status)

- [x] Split max-redirects policy invalid diagnostics into:
  - `NET.REDIRECT_POLICY_MAX_REDIRECTS_FORMAT_INVALID`
  - `NET.REDIRECT_POLICY_MAX_REDIRECTS_RANGE_INVALID`
- [x] Added dedicated format-invalid harness test:
  - `c_bin_runtime_internal_get_redirect_max_redirects_policy_format_invalid_returns_deterministic_code_when_clang_available`
- [x] Updated range-invalid harness assertion to new range-specific code.
- [x] Revalidated revalidate-policy-invalid diagnostics and c-backend emit contract.

### M38-S31 outbound HTTP policy diagnostics cleanup acceptance criteria

- Redirect policy parser helper surface removes unused generic strict-int parser path.
- Redirect policy diagnostics behavior remains unchanged after cleanup.
- Existing format/range and revalidate policy diagnostics remain green.

### M38-S31 tracking (live status)

- [x] Removed unused `sec4_rt_parse_env_non_negative_i64_strict` helper + declaration.
- [x] Kept active redirect policy parsing path explicit and deterministic.
- [x] Revalidated max-redirects range-invalid, revalidate-invalid, and c-backend emit contract.

### M38-S32 outbound HTTP policy diagnostics stabilization acceptance criteria

- Redirect policy diagnostic naming remains consistent across runtime harness tests, roadmap references, and book chapters.
- Final redirect-oriented runtime matrix sweep is executed and remains green after naming-stabilization edits.
- Existing redirect policy diagnostics behavior is unchanged by naming-only stabilization updates.

### M38-S32 tracking (live status)

- [x] Renamed policy-invalid harness surface to explicit allow-redirects variant:
  - `c_bin_runtime_internal_get_redirect_allow_redirects_policy_invalid_returns_deterministic_code_when_clang_available`
- [x] Updated roadmap + chapter references to aligned policy-invalid harness naming.
- [x] Ran final redirect matrix sweep buckets:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_https_`
  - `cargo test -p sec4 --test json_output c_bin_runtime_redirect_resolver_`

### M38-S33 outbound HTTP SSRF block-toggle policy/runtime parity acceptance criteria

- Policy model persists `net.ssrf` block toggles:
  - `block_private_ranges`
  - `block_loopback`
  - `block_link_local`
  - `block_metadata_ips`
- `sec4 run` bridges these policy values into runtime env keys.
- Runtime `url.public(...)` validation enforces these toggles deterministically for direct host checks and DNS-resolved scope checks.

### M38-S33 tracking (live status)

- [x] Extended `NetSsrfPolicyConfig` persisted fields + parsing with secure defaults (`true`) for all four block toggles.
- [x] `sec4 run` now exports:
  - `SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES`
  - `SEC4_RT_NET_SSRF_BLOCK_LOOPBACK`
  - `SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL`
  - `SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS`
- [x] Runtime public-url gate now applies toggle-aware classification for:
  - direct IPv4 host parsing,
  - DNS-resolved IPv4/IPv6 addresses,
  - loopback/link-local/metadata/private-range separation.
- [x] Added/updated tests:
  - `cargo test -p sec4-core --test policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_allows_public_loopback_when_ssrf_block_toggles_are_disabled_by_policy`
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_public_respects_ssrf_block_toggles_when_clang_available`
  - `cargo test -p sec4 --test commands`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S34 outbound HTTP SSRF block-toggle env-validation diagnostics acceptance criteria

- Runtime `url.public(...)` path validates `SEC4_RT_NET_SSRF_BLOCK_*` env toggles with strict boolean parsing.
- Invalid values emit deterministic field-specific policy diagnostics instead of silently falling back:
  - `NET.SSRF_POLICY_BLOCK_PRIVATE_RANGES_INVALID`
  - `NET.SSRF_POLICY_BLOCK_LOOPBACK_INVALID`
  - `NET.SSRF_POLICY_BLOCK_LINK_LOCAL_INVALID`
  - `NET.SSRF_POLICY_BLOCK_METADATA_IPS_INVALID`
- Error envelopes include deterministic `details:[{\"key\":\"policyKey\",\"value\":\"...\"}]`.

### M38-S34 tracking (live status)

- [x] Added strict boolean parsing for SSRF block-toggle env keys in runtime public-url validation path.
- [x] Added deterministic field-specific policy diagnostics with `policyKey` detail entries.
- [x] Updated `url.public(...)` to preserve active detailed runtime policy diagnostics (no generic overwrite).
- [x] Expanded runtime harness coverage in:
  - `c_bin_runtime_url_public_respects_ssrf_block_toggles_when_clang_available`
  to assert invalid-value diagnostics/details for all four SSRF block-toggle keys.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_public_respects_ssrf_block_toggles_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_public_dns_resolution_toggle_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S35 outbound HTTP SSRF resolve-dns env-validation diagnostics acceptance criteria

- Runtime `url.public(...)` path validates `SEC4_RT_NET_SSRF_RESOLVE_DNS` with strict boolean parsing.
- Invalid `resolve_dns` values emit deterministic policy diagnostics instead of silent fallback:
  - `NET.SSRF_POLICY_RESOLVE_DNS_INVALID`
- Error envelope includes deterministic policy detail:
  - `details:[{\"key\":\"policyKey\",\"value\":\"SEC4_RT_NET_SSRF_RESOLVE_DNS\"}]`

### M38-S35 tracking (live status)

- [x] Added strict env parsing for `SEC4_RT_NET_SSRF_RESOLVE_DNS` in runtime public-url validation path.
- [x] Added deterministic policy-invalid diagnostic:
  - `NET.SSRF_POLICY_RESOLVE_DNS_INVALID`
- [x] Added dedicated runtime harness test:
  - `c_bin_runtime_url_public_resolve_dns_policy_invalid_returns_deterministic_code_when_clang_available`
- [x] Revalidated resolve-dns toggle behavior and c-backend emit contract:
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_public_dns_resolution_toggle_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S36 outbound HTTP public-url policy-list env-validation diagnostics acceptance criteria

- Runtime `url.public(...)` path validates malformed list/token policy env inputs deterministically for:
  - `SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES`
  - `SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS`
  - `SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS`
  - `SEC4_RT_NET_PUBLIC_ALLOWED_PORTS`
- Invalid values emit field-specific deterministic diagnostics with `policyKey` details:
  - `NET.URL_PUBLIC_POLICY_ALLOWED_SCHEMES_INVALID`
  - `NET.URL_PUBLIC_POLICY_ALLOWED_DOMAINS_INVALID`
  - `NET.URL_PUBLIC_POLICY_BLOCKED_DOMAINS_INVALID`
  - `NET.URL_PUBLIC_POLICY_ALLOWED_PORTS_INVALID`
- Existing allow/deny behavior for valid list values remains unchanged.

### M38-S36 tracking (live status)

- [x] Added strict CSV/token validators for public scheme/domain/port list env keys in runtime URL policy path.
- [x] Added field-specific deterministic policy diagnostics with structured `policyKey` details.
- [x] Expanded `c_bin_runtime_url_public_respects_env_policy_lists_when_clang_available` harness to assert all malformed-list diagnostic branches.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_public_respects_env_policy_lists_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_public_enforces_allowed_ports_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S37 outbound HTTP internal-url allowlist env-validation diagnostics acceptance criteria

- Runtime `url.internal(...)` path validates malformed internal allowlist env inputs deterministically for:
  - `SEC4_RT_NET_INTERNAL_ALLOWED_DOMAINS`
  - `SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS`
- Invalid values emit field-specific deterministic diagnostics with `policyKey` details:
  - `NET.URL_INTERNAL_POLICY_ALLOWED_DOMAINS_INVALID`
  - `NET.URL_INTERNAL_POLICY_ALLOWED_CIDRS_INVALID`
- Existing internal-url allowlist behavior for valid domains/CIDRs remains unchanged.

### M38-S37 tracking (live status)

- [x] Added strict domain/CIDR list validators for internal allowlist env keys.
- [x] Added deterministic internal policy diagnostics with structured `policyKey` details.
- [x] Updated `url.internal(...)` to preserve active detailed policy diagnostics (no generic overwrite).
- [x] Expanded internal allowlist runtime harness coverage in:
  - `c_bin_runtime_url_internal_respects_allowed_cidrs_when_clang_available`
  to assert malformed domain/CIDR diagnostics and details.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_internal_respects_allowed_cidrs_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_public_respects_env_policy_lists_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S38 outbound HTTP internal-url allowlist IPv6-CIDR support acceptance criteria

- Runtime internal allowlist CIDR validation accepts both IPv4 and IPv6 CIDR tokens.
- Runtime internal allowlist CIDR matching supports IPv6 literal hosts (bracketed URL host form).
- Internal URL host parsing supports IPv6 literals for policy checks and port parsing compatibility.

### M38-S38 tracking (live status)

- [x] Added IPv6 CIDR parser + matcher for internal allowlist checks.
- [x] Upgraded internal CIDR list validation from IPv4-only to dual-stack (IPv4/IPv6).
- [x] Added IPv6 literal host parsing support in URL host extraction and port resolution helper paths.
- [x] Expanded internal allowlist harness coverage to assert IPv6 CIDR allow/mismatch behavior:
  - `http://[fd00::1]/...` allowed by `fd00::/8`
  - `http://[fd00:2::1]/...` rejected by mismatched `fd00:1::/64`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_internal_respects_allowed_cidrs_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_public_respects_env_policy_lists_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S39 outbound HTTP client IPv6-literal parser support acceptance criteria

- Runtime outbound HTTP URL parser accepts bracketed IPv6 literal host form for net client calls.
- URL parser keeps deterministic rejection behavior for malformed bracketed IPv6 hosts.
- Existing IPv4/hostname parsing behavior remains unchanged.

### M38-S39 tracking (live status)

- [x] Added bracketed IPv6 host parsing branch in `sec4_rt_parse_outbound_http_url`.
- [x] Added deterministic invalid-path rejection for malformed bracketed hosts (missing `]`, empty bracket host).
- [x] Added dedicated runtime harness coverage:
  - `c_bin_runtime_outbound_url_parser_supports_ipv6_literals_when_clang_available`
- [x] Revalidated existing outbound/internal GET path and c-backend emit contract:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_roundtrip_succeeds_with_env_override_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S40 outbound HTTP client IPv6-literal transport validation acceptance criteria

- Runtime internal GET path supports real IPv6 loopback URL form (`http://[::1]:port/...`) when environment supports IPv6 loopback socket bind/connect.
- Test harness retains deterministic behavior by skipping explicitly when IPv6 loopback is unavailable.
- Existing IPv4/internal GET behavior remains green.

### M38-S40 tracking (live status)

- [x] Added runtime integration test:
  - `c_bin_runtime_internal_get_ipv6_loopback_roundtrip_when_supported_when_clang_available`
- [x] Test now performs real one-shot IPv6 loopback roundtrip against `sec4_rt_http_get_internal(...)`.
- [x] Added deterministic environment fallback:
  - explicit skip message when `TcpListener::bind(\"[::1]:0\")` is unavailable.
- [x] Revalidated parser + c-backend contract:
  - `cargo test -p sec4 --test json_output c_bin_runtime_outbound_url_parser_supports_ipv6_literals_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S41 outbound HTTP IPv6-literal malformed-host diagnostics hardening acceptance criteria

- `url.public(...)` emits deterministic malformed IPv6-literal diagnostics for:
  - missing closing bracket
  - empty bracket literal
  - invalid IPv6 literal token
- `url.internal(...)` emits the same class of deterministic malformed IPv6-literal diagnostics.
- Existing valid IPv6-literal parser/transport paths remain green.

### M38-S41 tracking (live status)

- [x] Added explicit malformed bracketed-IPv6 classifier in runtime URL gate path.
- [x] Added deterministic public URL diagnostics:
  - `NET.URL_PUBLIC_IPV6_BRACKET_MISSING`
  - `NET.URL_PUBLIC_IPV6_EMPTY_LITERAL`
  - `NET.URL_PUBLIC_IPV6_LITERAL_INVALID`
- [x] Added deterministic internal URL diagnostics:
  - `NET.URL_INTERNAL_IPV6_BRACKET_MISSING`
  - `NET.URL_INTERNAL_IPV6_EMPTY_LITERAL`
  - `NET.URL_INTERNAL_IPV6_LITERAL_INVALID`
- [x] Added dedicated runtime harness coverage:
  - `c_bin_runtime_url_ipv6_literal_diagnostics_when_clang_available`
- [x] Revalidated IPv6-literal transport and c-backend contract:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_ipv6_loopback_roundtrip_when_supported_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S42 outbound HTTP URL-gate diagnostic propagation hardening acceptance criteria

- `httpClient.get(...)` preserves detailed URL gate diagnostics from `url.public(...)` validation failures.
- `httpClient.getInternal(...)` preserves detailed URL gate diagnostics from `url.internal(...)` validation failures.
- Generic sink diagnostics are only emitted when no detailed gate diagnostic is active.

### M38-S42 tracking (live status)

- [x] Updated `sec4_rt_http_get` to preserve active `url.public(...)` diagnostics instead of always overwriting with generic `NET.URL_PUBLIC_INVALID`.
- [x] Updated `sec4_rt_http_get_internal` to preserve active `url.internal(...)` diagnostics instead of always overwriting with generic `NET.URL_INTERNAL_INVALID`.
- [x] Added runtime harness coverage:
  - `c_bin_runtime_internal_get_ipv6_url_gate_diagnostics_when_clang_available`
  validating propagation for:
  - `NET.URL_INTERNAL_IPV6_BRACKET_MISSING`
  - `NET.URL_INTERNAL_IPV6_EMPTY_LITERAL`
  - `NET.URL_INTERNAL_IPV6_LITERAL_INVALID`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_ipv6_loopback_roundtrip_when_supported_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_url_ipv6_literal_diagnostics_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S43 outbound HTTP request-parser IPv6 malformed diagnostics hardening acceptance criteria

- Request parser failures in outbound `http_get*` flow emit split deterministic diagnostics for malformed bracketed IPv6 URLs:
  - missing closing bracket
  - empty literal
  - invalid literal token
- Redirect resolution path preserves the same split parser diagnostics when malformed absolute redirect targets fail parser validation.
- Runtime harness asserts both request-path and redirect-path coverage for `NET.REQUEST_IPV6_*` codes.

### M38-S43 tracking (live status)

- [x] Extended redirect-resolver status model with parser-derived IPv6 failure classes:
  - `SEC4_RT_REDIRECT_RESOLVE_IPV6_BRACKET_MISSING`
  - `SEC4_RT_REDIRECT_RESOLVE_IPV6_EMPTY_LITERAL`
  - `SEC4_RT_REDIRECT_RESOLVE_IPV6_LITERAL_INVALID`
- [x] Wired `sec4_rt_resolve_redirect_url(...)` parser failures to preserve split IPv6 parse reason instead of collapsing to generic redirect invalid.
- [x] Updated outbound redirect handling to emit deterministic request-parser diagnostics:
  - `NET.REQUEST_IPV6_BRACKET_MISSING`
  - `NET.REQUEST_IPV6_EMPTY_LITERAL`
  - `NET.REQUEST_IPV6_LITERAL_INVALID`
- [x] Added dedicated runtime harness coverage:
  - `c_bin_runtime_outbound_request_parser_ipv6_diagnostics_when_clang_available`
  - `c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
- [x] Revalidated core runtime emission contract:
  - `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_ipv6_diagnostics_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S44 outbound HTTP request-parser fallback diagnostics hardening acceptance criteria

- Non-IPv6 outbound request-parser failures in `http_get*` flow emit deterministic split diagnostics for:
  - scheme missing
  - scheme invalid
  - host token invalid
  - port token invalid
  - target token invalid
- Redirect-resolution parser failures preserve the same non-IPv6 split request-parser diagnostics (instead of collapsing to generic redirect-invalid).
- Runtime harness includes deterministic assertions for both direct request-parser path and redirect-parser path for new non-IPv6 `NET.REQUEST_*` codes.

### M38-S44 tracking (live status)

- [x] Extended outbound parser status model with non-IPv6 failure classes:
  - `SEC4_RT_OUTBOUND_URL_PARSE_SCHEME_MISSING`
  - `SEC4_RT_OUTBOUND_URL_PARSE_SCHEME_INVALID`
  - `SEC4_RT_OUTBOUND_URL_PARSE_HOST_INVALID`
  - `SEC4_RT_OUTBOUND_URL_PARSE_PORT_INVALID`
  - `SEC4_RT_OUTBOUND_URL_PARSE_TARGET_INVALID`
- [x] Added deterministic direct request-path diagnostics:
  - `NET.REQUEST_SCHEME_MISSING`
  - `NET.REQUEST_SCHEME_INVALID`
  - `NET.REQUEST_HOST_INVALID`
  - `NET.REQUEST_PORT_INVALID`
  - `NET.REQUEST_TARGET_INVALID`
- [x] Extended redirect-resolver status propagation with non-IPv6 parser classes and mapped redirect-path failures to the same `NET.REQUEST_*` diagnostics.
- [x] Added runtime harness coverage:
  - `c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available`
  - `c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available` (expanded with non-IPv6 redirect-parser cases)
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S45 outbound HTTP request-parser diagnostics detail enrichment acceptance criteria

- All split `NET.REQUEST_*` parser diagnostics include deterministic structured details:
  - `{"key":"phase","value":"parse"}`
  - `{"key":"component","value":"..."}` where component is one of `scheme|host|port|target|ipv6`.
- Detail enrichment applies consistently in both direct request-parser path and redirect parser-failure path.
- Runtime harness assertions validate both split diagnostic code and deterministic detail entries.

### M38-S45 tracking (live status)

- [x] Added runtime standard-error helper for deterministic two-detail payloads.
- [x] Updated outbound request-parser split diagnostic emission to include structured details:
  - `phase=parse`
  - `component=<scheme|host|port|target|ipv6>`
- [x] Updated redirect parse-failure mapping path to reuse the same detail-enriched request-parser diagnostics.
- [x] Extended runtime harness assertions for deterministic detail payload checks:
  - `c_bin_runtime_outbound_request_parser_ipv6_diagnostics_when_clang_available`
  - `c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available`
  - `c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_ipv6_diagnostics_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S46 outbound HTTP request-parser generic fallback normalization acceptance criteria

- Non-classified parser-status fallback no longer emits generic `NET.URL_INVALID` in parser error path.
- Deterministic fallback code `NET.REQUEST_PARSE_INVALID` is used for unknown parser-status fallback with structured details:
  - `phase=parse`
  - `component=unknown`
- Redirect parser-status mapping normalizes unknown parser fallback to the same deterministic request-parser fallback code.
- Runtime harness includes dedicated assertions for unknown parser-status fallback in both direct and redirect mapping paths.

### M38-S46 tracking (live status)

- [x] Added deterministic parser fallback code:
  - `NET.REQUEST_PARSE_INVALID`
  with details:
  - `{"key":"phase","value":"parse"}`
  - `{"key":"component","value":"unknown"}`
- [x] Updated redirect parser-status mapping fallback to explicit `SEC4_RT_REDIRECT_RESOLVE_REQUEST_PARSE_INVALID`.
- [x] Added runtime harness coverage:
  - `c_bin_runtime_outbound_request_parser_unknown_fallback_is_deterministic_when_clang_available`
  - `c_bin_runtime_redirect_request_parser_unknown_fallback_is_deterministic_when_clang_available`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_unknown_fallback_is_deterministic_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_redirect_request_parser_unknown_fallback_is_deterministic_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S47 outbound HTTP request-parser public/internal sink parity acceptance criteria

- Public sink path (`sec4_rt_http_get`) emits the same split parser diagnostics as internal sink path (`sec4_rt_http_get_internal`) for equivalent malformed redirect-target parser failures.
- Structured parser details parity is preserved across both sinks:
  - `phase=parse`
  - matching `component` classification.
- Runtime harness includes explicit public-sink redirect parser diagnostics parity assertions.

### M38-S47 tracking (live status)

- [x] Added public-sink redirect parser diagnostics parity harness:
  - `c_bin_runtime_public_get_redirect_request_parser_diagnostics_match_internal_when_clang_available`
- [x] Parity harness validates split parser diagnostics and structured detail parity for:
  - `NET.REQUEST_IPV6_BRACKET_MISSING` (`component=ipv6`)
  - `NET.REQUEST_HOST_INVALID` (`component=host`)
  - `NET.REQUEST_PORT_INVALID` (`component=port`)
  - `NET.REQUEST_TARGET_INVALID` (`component=target`)
- [x] Revalidated internal-sink parity baseline:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_redirect_ipv6_request_parser_diagnostics_when_clang_available`
- [x] Revalidated public-sink parity path:
  - `cargo test -p sec4 --test json_output c_bin_runtime_public_get_redirect_request_parser_diagnostics_match_internal_when_clang_available`

### M38-S48 outbound HTTP parser diagnostics envelope consistency acceptance criteria

- All `NET.REQUEST_*` parser diagnostics are rendered through one unified runtime envelope path.
- Deterministic details ordering is preserved for parser diagnostics:
  - first detail: `phase=parse`
  - second detail: `component=<...>`
- Runtime harness includes dedicated coverage that validates detail-order contract across unknown fallback, non-IPv6 classes, and IPv6 class.

### M38-S48 tracking (live status)

- [x] Refactored parser diagnostic emission to a table-driven unified renderer path in runtime.
- [x] Added dedicated runtime harness:
  - `c_bin_runtime_request_parser_diagnostics_use_unified_details_order_when_clang_available`
  validating deterministic detail-order contract across:
  - `NET.REQUEST_PARSE_INVALID`
  - `NET.REQUEST_SCHEME_INVALID`
  - `NET.REQUEST_HOST_INVALID`
  - `NET.REQUEST_PORT_INVALID`
  - `NET.REQUEST_TARGET_INVALID`
  - `NET.REQUEST_IPV6_LITERAL_INVALID`
- [x] Revalidated parser diagnostics regression paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_outbound_request_parser_fallback_diagnostics_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_public_get_redirect_request_parser_diagnostics_match_internal_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S49 outbound HTTP parser diagnostics sink-bridge contract coverage acceptance criteria

- Direct malformed-input wrapper calls (`sec4_rt_http_get` and `sec4_rt_http_get_internal`) emit the same parser-class diagnostic contract when malformed target tokens bypass URL gate checks.
- Public/internal sink bridge paths preserve deterministic parser detail payloads:
  - `phase=parse`
  - `component=target`
- Runtime harness includes one direct-wrapper parity test that asserts parser diagnostics for multiple malformed target shapes without redirect indirection.

### M38-S49 tracking (live status)

- [x] Added direct wrapper malformed-target parity harness:
  - `c_bin_runtime_direct_wrapper_malformed_target_parser_diagnostics_match_between_public_and_internal_when_clang_available`
- [x] Harness validates deterministic parser diagnostics parity on both sinks for malformed target inputs:
  - fragment target (`#`)
  - CRLF target contamination
  - query + fragment target
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_direct_wrapper_malformed_target_parser_diagnostics_match_between_public_and_internal_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_get_rejects_invalid_url_or_untracked_handles_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S50 outbound HTTP wrapper invalid-url pre-parser diagnostics parity coverage acceptance criteria

- Wrapper-level invalid-url failures are asserted directly at `sec4_rt_http_get` and `sec4_rt_http_get_internal` boundaries before outbound parser execution.
- Public/internal wrapper envelopes remain deterministic for equivalent malformed URL handles:
  - public wrapper: `NET.URL_PUBLIC_INVALID`
  - internal wrapper: `NET.URL_INTERNAL_INVALID`
- Wrapper invalid-url envelopes do not regress into parser-class diagnostics (`NET.REQUEST_*`) for the same malformed URL handle classes.

### M38-S50 tracking (live status)

- [x] Added direct wrapper invalid-url pre-parser parity harness:
  - `c_bin_runtime_wrapper_invalid_url_preparser_envelope_parity_between_public_and_internal_when_clang_available`
- [x] Harness validates deterministic wrapper envelopes for malformed URL classes:
  - invalid scheme (`ftp://...`)
  - missing scheme (`example.com/...`)
  - missing host (`http:///...`)
  - userinfo authority token (`http://user@example.com/...`)
- [x] Harness validates untracked-handle parity:
  - public wrapper untracked handle -> `NET.URL_PUBLIC_INVALID`
  - internal wrapper untracked handle -> `NET.URL_INTERNAL_INVALID`
  with parser diagnostics excluded (`NET.REQUEST_*` absent).
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_wrapper_invalid_url_preparser_envelope_parity_between_public_and_internal_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_direct_wrapper_malformed_target_parser_diagnostics_match_between_public_and_internal_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S51 outbound HTTP wrapper invalid-capability handle diagnostics parity coverage acceptance criteria

- Missing/zero wrapper capability or URL handles are asserted directly at wrapper boundaries for both sinks:
  - `sec4_rt_http_get` -> `NET.GET_INVALID`
  - `sec4_rt_http_get_internal` -> `NET.GET_INTERNAL_INVALID`
- Wrapper invalid-handle diagnostics remain deterministic `validation` envelopes and do not regress to:
  - internal-policy denial (`NET.INTERNAL_DENIED`)
  - parser-class diagnostics (`NET.REQUEST_*`).
- Runtime harness includes direct assertions for both missing-net and missing-url handle classes across public/internal wrappers.

### M38-S51 tracking (live status)

- [x] Added direct wrapper invalid-handle parity harness:
  - `c_bin_runtime_wrapper_invalid_handle_diagnostics_parity_between_public_and_internal_when_clang_available`
- [x] Harness validates deterministic wrapper invalid-handle contracts for:
  - missing net capability handle (`net=0`)
  - missing URL handle (`url=0`)
  across both public/internal wrappers.
- [x] Harness validates wrapper precedence:
  - invalid-handle diagnostics emitted before internal-net policy denial checks.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_wrapper_invalid_handle_diagnostics_parity_between_public_and_internal_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_wrapper_invalid_url_preparser_envelope_parity_between_public_and_internal_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S52 outbound HTTP internal-policy denial precedence coverage acceptance criteria

- Internal wrapper calls with valid internal URL handles emit deterministic policy denial when internal-net policy is disabled:
  - `NET.INTERNAL_DENIED`
  - `kind=authorization`
- Policy denial precedence is asserted before URL/parser failure classes for the same valid internal URL handles.
- Runtime harness includes direct coverage for both default-deny and explicit-deny (`SEC4_RT_ALLOW_INTERNAL_NET=0`) policy states.

### M38-S52 tracking (live status)

- [x] Added internal-policy denial precedence harness:
  - `c_bin_runtime_internal_policy_denial_precedence_for_valid_internal_urls_when_clang_available`
- [x] Harness validates deterministic denial envelopes for valid internal URLs:
  - loopback IPv4 URL (`http://127.0.0.1/...`)
  - localhost URL (`http://localhost/...`)
- [x] Harness validates precedence exclusions:
  - no `NET.URL_INTERNAL_INVALID`
  - no `NET.REQUEST_*`
  - no `NET.GET_INTERNAL_INVALID`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_denial_precedence_for_valid_internal_urls_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_wrapper_invalid_handle_diagnostics_parity_between_public_and_internal_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S53 outbound HTTP internal-policy allow-token coverage acceptance criteria

- Truthy internal-policy tokens for `SEC4_RT_ALLOW_INTERNAL_NET` bypass policy denial deterministically:
  - `1`, `true`, `yes`, `on`, `allow` (case-insensitive)
- After denial bypass, wrapper flow reaches downstream deterministic stages for valid internal URL handles:
  - parser-class failure path (`NET.REQUEST_TARGET_INVALID`) for malformed target
  - transport-class failure path (`NET.TLS_UNSUPPORTED`) for HTTPS in non-TLS runtime build
- Allow-token path does not regress into:
  - `NET.INTERNAL_DENIED`
  - `NET.GET_INTERNAL_INVALID`

### M38-S53 tracking (live status)

- [x] Added internal-policy allow-token truthy coverage harness:
  - `c_bin_runtime_internal_policy_allow_truthy_tokens_bypass_denial_when_clang_available`
- [x] Harness validates truthy-token allow behavior across tokens:
  - `"1"`, `"true"`, `"yes"`, `"on"`, `"allow"`, plus mixed/upper-case variants.
- [x] Harness validates deterministic downstream outcomes after denial bypass:
  - `NET.REQUEST_TARGET_INVALID` (`kind=validation`)
  - `NET.TLS_UNSUPPORTED` (`kind=runtime`)
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_allow_truthy_tokens_bypass_denial_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_denial_precedence_for_valid_internal_urls_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S54 outbound HTTP internal-policy invalid-token fallback coverage acceptance criteria

- Unknown/invalid `SEC4_RT_ALLOW_INTERNAL_NET` tokens deterministically preserve deny-by-default behavior.
- For valid internal URL handles under invalid policy tokens, runtime emits:
  - `NET.INTERNAL_DENIED`
  - `kind=authorization`
- Invalid-token fallback does not regress to:
  - `NET.GET_INTERNAL_INVALID`
  - `NET.URL_INTERNAL_INVALID`
  - `NET.REQUEST_*`

### M38-S54 tracking (live status)

- [x] Added internal-policy invalid-token fallback harness:
  - `c_bin_runtime_internal_policy_invalid_tokens_fallback_to_deny_when_clang_available`
- [x] Harness validates deny-by-default fallback across invalid token classes:
  - semantic noise (`"maybe"`, `"enabled"`, `"true-ish"`)
  - numeric noise (`"2"`, `"-1"`)
  - near-miss tokens/spacing (`"t"`, `"y"`, `" allow "`, `"YES!"`)
- [x] Harness validates deterministic deny envelope on valid internal URLs:
  - `http://127.0.0.1/...`
  - `http://localhost/...`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_invalid_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_allow_truthy_tokens_bypass_denial_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S55 outbound HTTP internal-policy explicit deny-token matrix coverage acceptance criteria

- Explicit deny tokens for `SEC4_RT_ALLOW_INTERNAL_NET` deterministically enforce internal-net denial:
  - `0`, `false`, `no`, `off`, `deny` (case-insensitive)
- For valid internal URL handles under explicit deny tokens, runtime emits:
  - `NET.INTERNAL_DENIED`
  - `kind=authorization`
- Explicit deny-token matrix does not regress to:
  - `NET.GET_INTERNAL_INVALID`
  - `NET.URL_INTERNAL_INVALID`
  - `NET.REQUEST_*`

### M38-S55 tracking (live status)

- [x] Added internal-policy explicit deny-token matrix harness:
  - `c_bin_runtime_internal_policy_explicit_deny_tokens_enforce_denial_when_clang_available`
- [x] Harness validates explicit deny-token enforcement across tokens:
  - `"0"`, `"false"`, `"no"`, `"off"`, `"deny"`, plus mixed/upper-case variants.
- [x] Harness validates deterministic deny envelope on valid internal URLs:
  - `http://127.0.0.1/...`
  - `http://localhost/...`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_explicit_deny_tokens_enforce_denial_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_invalid_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S56 outbound HTTP internal-policy empty/whitespace-token fallback coverage acceptance criteria

- Unset, empty, and whitespace-only `SEC4_RT_ALLOW_INTERNAL_NET` values deterministically preserve deny-by-default behavior.
- For valid internal URL handles under unset/blank token states, runtime emits:
  - `NET.INTERNAL_DENIED`
  - `kind=authorization`
- Empty/whitespace-token fallback does not regress to:
  - `NET.GET_INTERNAL_INVALID`
  - `NET.URL_INTERNAL_INVALID`
  - `NET.REQUEST_*`

### M38-S56 tracking (live status)

- [x] Added internal-policy empty/whitespace-token fallback harness:
  - `c_bin_runtime_internal_policy_empty_or_whitespace_tokens_fallback_to_deny_when_clang_available`
- [x] Harness validates deny-by-default fallback for token states:
  - unset (`env_remove`)
  - empty (`""`)
  - whitespace-only (`" "`, `"   "`, `"\t"`, `"\n"`, `" \t "`)
- [x] Harness validates deterministic deny envelope on valid internal URLs:
  - `http://127.0.0.1/...`
  - `http://localhost/...`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_empty_or_whitespace_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_explicit_deny_tokens_enforce_denial_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S57 outbound HTTP internal-policy quoted-token fallback coverage acceptance criteria

- Quoted/escaped token spellings for `SEC4_RT_ALLOW_INTERNAL_NET` deterministically preserve deny-by-default behavior.
- For valid internal URL handles under quoted token values, runtime emits:
  - `NET.INTERNAL_DENIED`
  - `kind=authorization`
- Quoted-token fallback does not regress to:
  - `NET.GET_INTERNAL_INVALID`
  - `NET.URL_INTERNAL_INVALID`
  - `NET.REQUEST_*`

### M38-S57 tracking (live status)

- [x] Added internal-policy quoted-token fallback harness:
  - `c_bin_runtime_internal_policy_quoted_tokens_fallback_to_deny_when_clang_available`
- [x] Harness validates quoted-token deny-by-default fallback across tokens:
  - double-quoted (`"\"1\""`, `"\"true\""`, `"\"allow\""`)
  - single-quoted (`"'yes'"`, `"'on'"`, `"'ALLOW'"`)
  - quoted + inner whitespace (`"\" TRUE \""`, `"' yes '"`)
- [x] Harness validates deterministic deny envelope on valid internal URLs:
  - `http://127.0.0.1/...`
  - `http://localhost/...`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_quoted_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_empty_or_whitespace_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S58 outbound HTTP internal-policy delimited-token fallback coverage acceptance criteria

- Delimited token spellings for `SEC4_RT_ALLOW_INTERNAL_NET` deterministically preserve deny-by-default behavior.
- For valid internal URL handles under delimited token values, runtime emits:
  - `NET.INTERNAL_DENIED`
  - `kind=authorization`
- Delimited-token fallback does not regress to:
  - `NET.GET_INTERNAL_INVALID`
  - `NET.URL_INTERNAL_INVALID`
  - `NET.REQUEST_*`

### M38-S58 tracking (live status)

- [x] Added internal-policy delimited-token fallback harness:
  - `c_bin_runtime_internal_policy_delimited_tokens_fallback_to_deny_when_clang_available`
- [x] Harness validates deny-by-default fallback across delimiter classes:
  - comma (`"true,allow"`, `"allow,true"`)
  - pipe/slash (`"yes|on"`, `"allow/1"`)
  - semicolon/colon (`"true;allow"`, `"on:yes"`)
  - delimiter + spacing (`" yes|allow "`)
- [x] Harness validates deterministic deny envelope on valid internal URLs:
  - `http://127.0.0.1/...`
  - `http://localhost/...`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_delimited_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_quoted_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S59 outbound HTTP internal-policy prefixed-token fallback coverage acceptance criteria

- Prefixed token spellings for `SEC4_RT_ALLOW_INTERNAL_NET` deterministically preserve deny-by-default behavior.
- For valid internal URL handles under prefixed token values, runtime emits:
  - `NET.INTERNAL_DENIED`
  - `kind=authorization`
- Prefixed-token fallback does not regress to:
  - `NET.GET_INTERNAL_INVALID`
  - `NET.URL_INTERNAL_INVALID`
  - `NET.REQUEST_*`

### M38-S59 tracking (live status)

- [x] Added internal-policy prefixed-token fallback harness:
  - `c_bin_runtime_internal_policy_prefixed_tokens_fallback_to_deny_when_clang_available`
- [x] Harness validates deny-by-default fallback across prefixed token spellings:
  - key/value style (`"allow=true"`, `"token=1"`, `"internal-net=on"`)
  - namespace/prefix style (`"mode:allow"`, `"value:true"`, `"policy.allow=yes"`, `"allow:true"`)
- [x] Harness validates deterministic deny envelope on valid internal URLs:
  - `http://127.0.0.1/...`
  - `http://localhost/...`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_prefixed_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_delimited_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S60 security-headers CSP runtime materialization coverage acceptance criteria

- Runtime security headers block includes deterministic CSP emission when security headers are enabled.
- Default security-header policy emits enforce-mode CSP header:
  - `Content-Security-Policy: default-src 'self'; frame-ancestors 'none'; base-uri 'self'`
- Env policy toggles support report-only CSP header mode:
  - `SEC4_RT_SECURITY_HEADERS_CSP_REPORT_ONLY=1`
  - `SEC4_RT_SECURITY_HEADERS_CSP_POLICY=<policy>`
- CSP runtime materialization does not regress existing security-header behavior (`nosniff`, `x-frame-options`, `referrer-policy`).

### M38-S60 tracking (live status)

- [x] Extended runtime security-header policy/router state with CSP fields:
  - `csp_enabled`
  - `csp_report_only`
  - `csp_policy`
- [x] Added CSP header emission in `sec4_rt_security_headers_block` with deterministic enforce/report-only header selection.
- [x] Added env-policy loading for CSP controls and safe fallback to default CSP policy for invalid header values.
- [x] Added/updated HTTP runtime security-header e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_on_success_when_enabled` (assert default CSP enforce header)
  - `c_bin_http_runtime_applies_security_headers_csp_report_only_when_enabled` (assert report-only header path)
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_csp_report_only_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S61 security-headers HSTS runtime materialization coverage acceptance criteria

- Runtime security headers block includes deterministic HSTS emission when HSTS is enabled by policy.
- Env policy controls materialize HSTS header value:
  - `SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED`
  - `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS`
  - `SEC4_RT_SECURITY_HEADERS_HSTS_INCLUDE_SUBDOMAINS`
  - `SEC4_RT_SECURITY_HEADERS_HSTS_PRELOAD`
- HSTS materialization does not regress existing security-header behavior (`nosniff`, `x-frame-options`, `referrer-policy`, `csp`).

### M38-S61 tracking (live status)

- [x] Extended runtime security-header policy/router state with HSTS fields:
  - `hsts_enabled`
  - `hsts_max_age_seconds`
  - `hsts_include_subdomains`
  - `hsts_preload`
- [x] Added HSTS header emission in `sec4_rt_security_headers_block` with deterministic value rendering.
- [x] Added env-policy loading/materialization for HSTS controls in runtime security-header path.
- [x] Added HTTP runtime security-header HSTS e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S62 outbound HTTP internal-policy suffixed-token fallback coverage acceptance criteria

- Suffixed token spellings for `SEC4_RT_ALLOW_INTERNAL_NET` deterministically preserve deny-by-default behavior.
- For valid internal URL handles under suffixed token values, runtime emits:
  - `NET.INTERNAL_DENIED`
  - `kind=authorization`
- Suffixed-token fallback does not regress to:
  - `NET.GET_INTERNAL_INVALID`
  - `NET.URL_INTERNAL_INVALID`
  - `NET.REQUEST_*`

### M38-S62 tracking (live status)

- [x] Added internal-policy suffixed-token fallback harness:
  - `c_bin_runtime_internal_policy_suffixed_tokens_fallback_to_deny_when_clang_available`
- [x] Harness validates deny-by-default fallback across suffixed token spellings:
  - hyphen/underscore suffixes (`"true-value"`, `"allow_mode"`, `"yes-end"`)
  - alphanumeric/punctuation suffixes (`"on1"`, `"allow+"`, `"1ok"`, `"true."`)
- [x] Harness validates deterministic deny envelope on valid internal URLs:
  - `http://127.0.0.1/...`
  - `http://localhost/...`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_suffixed_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_runtime_internal_policy_prefixed_tokens_fallback_to_deny_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S63 security-headers HSTS invalid-env fallback coverage acceptance criteria

- Invalid/non-numeric `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS` values deterministically fall back to safe default `max-age=15552000`.
- Negative `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS` values deterministically fall back to safe default `max-age=15552000`.
- Fallback behavior preserves existing HSTS header composition semantics (includeSubDomains/preload toggles).
- Invalid raw env values are never reflected into emitted HSTS header strings.

### M38-S63 tracking (live status)

- [x] Added HTTP runtime security-header HSTS fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_hsts_invalid_max_age_falls_back_to_default_when_enabled`
- [x] Harness validates both malformed and negative max-age env inputs:
  - malformed numeric token (`"not-a-number"`)
  - negative numeric token (`"-7"`)
- [x] Harness validates deterministic safe fallback header:
  - `Strict-Transport-Security: max-age=15552000; includeSubDomains`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_invalid_max_age_falls_back_to_default_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S64 security-headers x-frame-options invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_SECURITY_HEADERS_X_FRAME_OPTIONS` env values deterministically fall back to `DENY`.
- Invalid raw env values are never reflected in emitted `X-Frame-Options` response headers.
- Fallback coverage does not regress existing security-header success-path behavior.

### M38-S64 tracking (live status)

- [x] Added HTTP runtime security-header x-frame-options fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_x_frame_options_invalid_env_falls_back_to_deny_when_enabled`
- [x] Harness validates invalid-token fallback behavior:
  - invalid env token (`"ALLOW-FROM"`) falls back to `X-Frame-Options: DENY`
- [x] Harness validates response output never reflects invalid x-frame-options token values.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_x_frame_options_invalid_env_falls_back_to_deny_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S65 security-headers referrer-policy invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_SECURITY_HEADERS_REFERRER_POLICY` env values deterministically fall back to `strict-origin-when-cross-origin`.
- Invalid raw env values are never reflected in emitted `Referrer-Policy` response headers.
- Fallback coverage does not regress existing security-header success-path behavior.

### M38-S65 tracking (live status)

- [x] Added HTTP runtime security-header referrer-policy fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_referrer_policy_invalid_env_falls_back_to_default_when_enabled`
- [x] Runtime policy loader now validates referrer-policy values against an explicit allowlist and clamps invalid values to `strict-origin-when-cross-origin`.
- [x] Harness validates invalid-token fallback behavior:
  - invalid env token (`"INVALID-POLICY"`) falls back to `Referrer-Policy: strict-origin-when-cross-origin`
- [x] Harness validates response output never reflects invalid referrer-policy token values.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_referrer_policy_invalid_env_falls_back_to_default_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S66 security-headers x-content-type-options invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_SECURITY_HEADERS_X_CONTENT_TYPE_OPTIONS` env values deterministically fall back to enabled `nosniff` behavior.
- Fallback coverage does not regress existing security-header success-path behavior.

### M38-S66 tracking (live status)

- [x] Added HTTP runtime security-header x-content-type-options fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_x_content_type_options_invalid_env_falls_back_to_nosniff_when_enabled`
- [x] Harness validates invalid boolean token fallback behavior:
  - invalid env token (`"MAYBE"`) still emits `X-Content-Type-Options: nosniff`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_x_content_type_options_invalid_env_falls_back_to_nosniff_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S67 security-headers CSP report-only invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_SECURITY_HEADERS_CSP_REPORT_ONLY` env values deterministically fall back to enforce-mode CSP header behavior.
- Invalid raw env values are never reflected as report-only mode in emitted CSP header names.
- Fallback coverage does not regress existing security-header success-path behavior.

### M38-S67 tracking (live status)

- [x] Added HTTP runtime security-header CSP report-only fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_csp_report_only_invalid_env_falls_back_to_enforce_when_enabled`
- [x] Harness validates invalid boolean token fallback behavior:
  - invalid env token (`"MAYBE"`) emits enforce header (`Content-Security-Policy`) and does not emit report-only header.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_csp_report_only_invalid_env_falls_back_to_enforce_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_csp_report_only_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S68 security-headers CSP enabled invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_SECURITY_HEADERS_CSP_ENABLED` env values deterministically fall back to keeping CSP enabled.
- Invalid raw env values are never reflected as disabled CSP behavior in emitted response headers.
- Fallback coverage does not regress existing security-header success-path behavior.

### M38-S68 tracking (live status)

- [x] Added HTTP runtime security-header CSP enabled fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_csp_enabled_invalid_env_falls_back_to_enabled_when_security_headers_enabled`
- [x] Harness validates invalid boolean token fallback behavior:
  - invalid env token (`"MAYBE"`) keeps enforce-mode `Content-Security-Policy` header active.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_csp_enabled_invalid_env_falls_back_to_enabled_when_security_headers_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S69 security-headers middleware-enabled invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_SECURITY_HEADERS_ENABLED` env values deterministically fall back to keeping security headers enabled.
- Invalid raw env values are never reflected as disabled security-header behavior in emitted responses.
- Fallback coverage does not regress existing security-header success-path behavior.

### M38-S69 tracking (live status)

- [x] Added HTTP runtime security-header enabled-flag fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_enabled_invalid_env_falls_back_to_enabled_when_enabled_by_default`
- [x] Harness validates invalid boolean token fallback behavior:
  - invalid env token (`"MAYBE"`) keeps baseline security headers active.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_enabled_invalid_env_falls_back_to_enabled_when_enabled_by_default`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S70 security-headers HSTS-enabled invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED` env values deterministically fall back to default HSTS-disabled behavior.
- Invalid raw env values are never reflected as enabled HSTS behavior in emitted responses.
- Fallback coverage does not regress existing security-header success-path behavior.

### M38-S70 tracking (live status)

- [x] Added HTTP runtime security-header HSTS-enabled fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_hsts_enabled_invalid_env_falls_back_to_disabled_by_default`
- [x] Harness validates invalid boolean token fallback behavior:
  - invalid env token (`"MAYBE"`) keeps default no-HSTS-header output.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_enabled_invalid_env_falls_back_to_disabled_by_default`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S71 security-headers HSTS includeSubDomains invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_SECURITY_HEADERS_HSTS_INCLUDE_SUBDOMAINS` env values deterministically fall back to default includeSubDomains-enabled behavior when HSTS is enabled.
- Invalid raw env values are never reflected as includeSubDomains-disabled behavior in emitted HSTS headers.
- Fallback coverage does not regress existing HSTS/header success-path behavior.

### M38-S71 tracking (live status)

- [x] Added HTTP runtime security-header HSTS includeSubDomains fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_hsts_include_subdomains_invalid_env_falls_back_to_enabled_when_hsts_enabled`
- [x] Harness validates invalid boolean token fallback behavior:
  - invalid env token (`"MAYBE"`) preserves `; includeSubDomains` in emitted HSTS header.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_include_subdomains_invalid_env_falls_back_to_enabled_when_hsts_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S72 security-headers HSTS preload invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_SECURITY_HEADERS_HSTS_PRELOAD` env values deterministically fall back to default preload-disabled behavior when HSTS is enabled.
- Invalid raw env values are never reflected as preload-enabled behavior in emitted HSTS headers.
- Fallback coverage does not regress existing HSTS/header success-path behavior.

### M38-S72 tracking (live status)

- [x] Added HTTP runtime security-header HSTS preload fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_hsts_preload_invalid_env_falls_back_to_disabled_when_hsts_enabled`
- [x] Harness validates invalid boolean token fallback behavior:
  - invalid env token (`"MAYBE"`) keeps HSTS header without `; preload`.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_preload_invalid_env_falls_back_to_disabled_when_hsts_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_hsts_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S73 security-headers CSP policy invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_SECURITY_HEADERS_CSP_POLICY` env header values deterministically fall back to default CSP policy.
- Invalid raw env values are never reflected in emitted CSP headers.
- Fallback coverage does not regress existing CSP/security-header success-path behavior.

### M38-S73 tracking (live status)

- [x] Added HTTP runtime security-header CSP policy fallback e2e coverage:
  - `c_bin_http_runtime_applies_security_headers_csp_policy_invalid_env_falls_back_to_default_policy_when_enabled`
- [x] Harness validates invalid header-value token fallback behavior:
  - invalid env value containing newline falls back to default CSP policy.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_csp_policy_invalid_env_falls_back_to_default_policy_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_security_headers_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S74 CORS allowed-origins invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_CORS_ALLOWED_ORIGINS` env values deterministically fall back to wildcard-origin baseline (`*`).
- Invalid raw env values are never reflected into emitted `Access-Control-Allow-Origin` response headers.
- Fallback coverage does not regress existing CORS success-path behavior.

### M38-S74 tracking (live status)

- [x] Runtime CORS policy loading now validates copied allowed-origin token as safe header value and falls back to wildcard when invalid.
- [x] Added HTTP runtime CORS allowed-origins fallback e2e coverage:
  - `c_bin_http_runtime_applies_cors_allowed_origins_invalid_env_falls_back_to_wildcard_when_enabled`
- [x] Harness validates invalid header-value token fallback behavior:
  - invalid env value containing newline falls back to wildcard origin header.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_invalid_env_falls_back_to_wildcard_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S75 CORS allow-credentials invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_CORS_ALLOW_CREDENTIALS` env values deterministically fall back to default credentials-disabled behavior.
- Invalid raw env values are never reflected as enabled credentials behavior in emitted responses.
- Fallback coverage does not regress existing CORS success-path behavior.

### M38-S75 tracking (live status)

- [x] Added HTTP runtime CORS allow-credentials fallback e2e coverage:
  - `c_bin_http_runtime_applies_cors_allow_credentials_invalid_env_falls_back_to_disabled_when_enabled`
- [x] Harness validates invalid boolean token fallback behavior:
  - invalid env token (`"MAYBE"`) keeps `Access-Control-Allow-Credentials` header absent.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allow_credentials_invalid_env_falls_back_to_disabled_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S76 CORS require-vary-origin invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_CORS_REQUIRE_VARY_ORIGIN` env values deterministically fall back to default vary-origin-disabled behavior.
- Invalid raw env values are never reflected as enabled vary-origin behavior in emitted responses.
- Fallback coverage does not regress existing CORS success-path behavior.

### M38-S76 tracking (live status)

- [x] Added HTTP runtime CORS require-vary-origin fallback e2e coverage:
  - `c_bin_http_runtime_applies_cors_require_vary_origin_invalid_env_falls_back_to_disabled_when_origin_is_non_wildcard`
- [x] Harness validates invalid boolean token fallback behavior:
  - invalid env token (`"MAYBE"`) keeps `Vary: Origin` header absent under non-wildcard allow-origin configuration.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_require_vary_origin_invalid_env_falls_back_to_disabled_when_origin_is_non_wildcard`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S77 CORS enabled invalid-env fallback coverage acceptance criteria

- Invalid `SEC4_RT_CORS_ENABLED` env values deterministically fall back to default CORS-enabled behavior.
- Invalid raw env values are never reflected as disabled CORS behavior in emitted responses.
- Fallback coverage does not regress existing CORS success-path behavior.

### M38-S77 tracking (live status)

- [x] Added HTTP runtime CORS enabled fallback e2e coverage:
  - `c_bin_http_runtime_applies_cors_enabled_invalid_env_falls_back_to_enabled_by_default`
- [x] Harness validates invalid boolean token fallback behavior:
  - invalid env token (`"MAYBE"`) preserves baseline `Access-Control-Allow-Origin` emission.
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_enabled_invalid_env_falls_back_to_enabled_by_default`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S78 auth mode invalid-env fallback hardening acceptance criteria

- Invalid `SEC4_RT_AUTH_MODE` env values deterministically fall back to `token` mode.
- Invalid raw env values are never allowed to force an unsupported auth mode state.
- Fallback hardening does not regress existing token/cookie auth success-path behavior.

### M38-S78 tracking (live status)

- [x] Runtime auth policy loading now normalizes unsupported auth mode env values to `token`.
- [x] Runtime effective-auth resolution now clamps unsupported mode values to `token` before auth checks.
- [x] Added HTTP runtime auth mode fallback e2e coverage:
  - `c_bin_http_runtime_auth_mode_invalid_env_falls_back_to_token_mode`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_auth_mode_invalid_env_falls_back_to_token_mode`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_request_with_auth_header_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_request_with_session_cookie_when_cookie_auth_mode_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S79 CSRF protected-methods invalid-env fallback hardening acceptance criteria

- Invalid `SEC4_RT_CSRF_PROTECTED_METHODS` env values deterministically fall back to default protected methods (`POST,PUT,PATCH,DELETE`).
- Invalid raw env values are never allowed to weaken CSRF protection coverage for unsafe methods.
- Fallback hardening does not regress existing CSRF reject/allow behavior.

### M38-S79 tracking (live status)

- [x] Runtime CSRF policy loading now validates protected-method tokens and clamps invalid lists to default unsafe-method set.
- [x] Added HTTP runtime CSRF protected-methods fallback e2e coverage:
  - `c_bin_http_runtime_csrf_protected_methods_invalid_env_falls_back_to_default_set`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_csrf_protected_methods_invalid_env_falls_back_to_default_set`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_post_without_csrf_tokens_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_post_with_matching_csrf_tokens_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S80 auth cookie-name invalid-env fallback hardening acceptance criteria

- Invalid `SEC4_RT_AUTH_COOKIE_NAME` env values deterministically fall back to default cookie name (`session`).
- Invalid raw env values are never allowed to force impossible/unsafe cookie-name matching behavior.
- Fallback hardening does not regress existing cookie-auth success/role-check behavior.

### M38-S80 tracking (live status)

- [x] Runtime auth cookie-name resolution now validates env token syntax and clamps invalid values to `session`.
- [x] Added HTTP runtime auth cookie-name fallback e2e coverage:
  - `c_bin_http_runtime_auth_cookie_name_invalid_env_falls_back_to_session_cookie`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_auth_cookie_name_invalid_env_falls_back_to_session_cookie`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_request_with_session_cookie_when_cookie_auth_mode_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_auth_require_role_rejects_cookie_without_required_role_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S81 CORS max-age env policy materialization hardening acceptance criteria

- Runtime CORS policy loader materializes `SEC4_RT_CORS_MAX_AGE_SECONDS` into preflight responses when valid.
- Invalid/non-positive `SEC4_RT_CORS_MAX_AGE_SECONDS` env values deterministically fall back to default `600`.
- Max-age policy materialization does not regress existing preflight CORS behavior.

### M38-S81 tracking (live status)

- [x] Runtime CORS policy state now includes max-age field loaded from env with deterministic clamp (`>0`, else `600`).
- [x] `withCors(..., cors.fromPolicy())` now applies materialized max-age value into router preflight config.
- [x] Added HTTP runtime CORS max-age e2e coverage:
  - `c_bin_http_runtime_applies_cors_max_age_env_value_on_preflight_when_enabled`
  - `c_bin_http_runtime_applies_cors_max_age_invalid_env_falls_back_to_default_on_preflight`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_max_age_env_value_on_preflight_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_max_age_invalid_env_falls_back_to_default_on_preflight`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S82 CSRF header/cookie name env materialization hardening acceptance criteria

- Runtime CSRF checks honor `SEC4_RT_CSRF_HEADER_NAME` and `SEC4_RT_CSRF_COOKIE_NAME` when values are valid.
- Invalid CSRF header/cookie-name env values deterministically fall back to defaults (`X-CSRF-Token`, `csrf`).
- CSRF name materialization hardening does not regress existing reject/allow and `csrf.issueToken` behavior.

### M38-S82 tracking (live status)

- [x] Runtime CSRF policy state and router state now carry header/cookie names materialized from env policy.
- [x] CSRF enforcement path now resolves configured names instead of hardcoded values.
- [x] `csrf.issueToken` response headers now emit configured/fallback CSRF names deterministically.
- [x] Added HTTP runtime CSRF name-materialization e2e coverage:
  - `c_bin_http_runtime_allows_post_with_matching_custom_csrf_names_from_env`
  - `c_bin_http_runtime_csrf_names_invalid_env_fall_back_to_defaults`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_post_with_matching_custom_csrf_names_from_env`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_csrf_names_invalid_env_fall_back_to_defaults`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_post_with_matching_csrf_tokens_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_post_without_csrf_tokens_when_enabled`
  - `cargo test -p sec4 --test json_output build_emit_c_bin_handles_csrf_issue_token_intrinsic_when_clang_available`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S83 CORS methods/headers env policy materialization hardening acceptance criteria

- Runtime preflight CORS responses honor env policy lists:
  - `SEC4_RT_CORS_ALLOWED_METHODS`
  - `SEC4_RT_CORS_ALLOWED_HEADERS`
- Invalid methods/header-list env values deterministically fall back to defaults.
- Methods/headers materialization hardening does not regress existing preflight behavior.

### M38-S83 tracking (live status)

- [x] Runtime CORS policy state now carries allowed-methods and allowed-headers lists loaded from env policy.
- [x] Added list validators for CORS methods and CORS header-name CSV tokens.
- [x] `withCors(..., cors.fromPolicy())` now applies validated env-provided method/header lists.
- [x] Added HTTP runtime CORS methods/headers e2e coverage:
  - `c_bin_http_runtime_applies_cors_methods_and_headers_from_env_on_preflight`
  - `c_bin_http_runtime_cors_methods_and_headers_invalid_env_fall_back_to_defaults`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_methods_and_headers_from_env_on_preflight`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_methods_and_headers_invalid_env_fall_back_to_defaults`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S84 CORS exposed-headers env policy materialization hardening acceptance criteria

- Runtime CORS success responses honor `SEC4_RT_CORS_EXPOSED_HEADERS` when values are valid.
- Invalid exposed-headers env values deterministically fall back to absent `Access-Control-Expose-Headers`.
- Exposed-headers materialization hardening does not regress existing CORS success/preflight behavior.

### M38-S84 tracking (live status)

- [x] Runtime CORS policy state now carries `expose_headers` loaded from env policy.
- [x] Added exposed-headers validation/fallback path using header-name CSV validation.
- [x] Runtime CORS success-header block now emits `Access-Control-Expose-Headers` when configured.
- [x] Added HTTP runtime CORS exposed-headers e2e coverage:
  - `c_bin_http_runtime_applies_cors_exposed_headers_from_env_on_success_when_enabled`
  - `c_bin_http_runtime_cors_exposed_headers_invalid_env_fall_back_to_absent_when_enabled`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_exposed_headers_from_env_on_success_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_exposed_headers_invalid_env_fall_back_to_absent_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S85 CORS allowed-origins allowlist request-origin materialization hardening acceptance criteria

- Runtime CORS success/preflight responses honor multi-origin `SEC4_RT_CORS_ALLOWED_ORIGINS` lists by reflecting the request `Origin` when it matches an allowlist token.
- Invalid allowlist env values deterministically fall back to wildcard (`*`) behavior.
- Non-matching request `Origin` values do not get reflected and deterministically fall back to the first configured allow-origin token.

### M38-S85 tracking (live status)

- [x] Runtime CORS allowed-origins env loading now validates full CSV allowlist tokens instead of only the first token.
- [x] Runtime CORS success/preflight header assembly now resolves effective allow-origin from request `Origin` when allowlist matching applies.
- [x] Added HTTP runtime CORS allowlist e2e coverage:
  - `c_bin_http_runtime_applies_cors_allowed_origins_allowlist_matching_request_origin_when_enabled`
  - `c_bin_http_runtime_cors_allowed_origins_allowlist_non_matching_origin_falls_back_to_first_token_when_enabled`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_allowlist_matching_request_origin_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_allowed_origins_allowlist_non_matching_origin_falls_back_to_first_token_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_invalid_env_falls_back_to_wildcard_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S86 CORS wildcard+credentials runtime safety guard hardening acceptance criteria

- Runtime CORS header emission never outputs `Access-Control-Allow-Credentials: true` when effective allow-origin is wildcard (`*`).
- Non-wildcard allow-origin flows continue to emit credentials headers when explicitly enabled.
- Wildcard+credentials guard applies consistently to success and preflight CORS responses.

### M38-S86 tracking (live status)

- [x] Runtime CORS success/preflight header assembly now suppresses credentials headers whenever effective allow-origin resolves to wildcard.
- [x] Wildcard token detection now works for wildcard present anywhere in the configured allow-origin CSV list.
- [x] Added HTTP runtime CORS wildcard+credentials guard e2e coverage:
  - `c_bin_http_runtime_cors_wildcard_with_allow_credentials_env_suppresses_credentials_header`
  - `c_bin_http_runtime_cors_non_wildcard_with_allow_credentials_env_emits_credentials_header`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_wildcard_with_allow_credentials_env_suppresses_credentials_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_non_wildcard_with_allow_credentials_env_emits_credentials_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_allowlist_matching_request_origin_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S87 CORS allow-origin token-shape validation hardening acceptance criteria

- Runtime CORS allow-origin policy list accepts only wildcard (`*`) or strict origin tokens (`http://...` / `https://...` with valid host and optional valid port).
- Malformed allow-origin tokens (for example missing scheme) deterministically trigger wildcard fallback.
- Strict token-shape validation does not regress valid allowlist request-origin matching behavior.

### M38-S87 tracking (live status)

- [x] Added strict CORS origin-token validator used by CORS allow-origin CSV policy validation.
- [x] Origin-token validation now rejects malformed tokens lacking required scheme/authority shape before runtime header emission.
- [x] Added HTTP runtime CORS invalid-token-shape e2e coverage:
  - `c_bin_http_runtime_cors_allowed_origins_missing_scheme_token_falls_back_to_wildcard_when_enabled`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_cors_allowed_origins_missing_scheme_token_falls_back_to_wildcard_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_allowlist_matching_request_origin_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_allowed_origins_invalid_env_falls_back_to_wildcard_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S88 CORS preflight requested-method enforcement hardening acceptance criteria

- Runtime CORS preflight handling validates `Access-Control-Request-Method` against configured allow-methods policy.
- Preflight requests for disallowed methods deterministically return `403` with explicit rejection diagnostics.
- Allowed-method preflight behavior (`204` with CORS preflight headers) remains unchanged.

### M38-S88 tracking (live status)

- [x] Runtime preflight path now checks requested method membership in CORS allow-methods CSV before emitting success preflight headers.
- [x] Added deterministic rejection response for disallowed preflight methods (`403` + fixed message body).
- [x] Added HTTP runtime CORS preflight disallowed-method e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_with_auth_and_csrf_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S89 CORS preflight requested-headers enforcement hardening acceptance criteria

- Runtime CORS preflight handling validates `Access-Control-Request-Headers` tokens against configured CORS allow-headers policy.
- Preflight requests containing disallowed/invalid requested headers deterministically return `403` with explicit rejection diagnostics.
- Allowed requested-header preflight behavior remains `204` with normal CORS preflight headers.

### M38-S89 tracking (live status)

- [x] Added runtime requested-headers membership validation for CORS preflight path.
- [x] Added deterministic rejection response for disallowed requested headers (`403` + fixed message body).
- [x] Added HTTP runtime CORS preflight requested-headers e2e coverage:
  - `c_bin_http_runtime_allows_cors_preflight_when_requested_headers_are_allowed`
  - `c_bin_http_runtime_rejects_cors_preflight_when_requested_header_is_not_allowed`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_when_requested_headers_are_allowed`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_header_is_not_allowed`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_with_auth_and_csrf_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S90 CORS preflight missing requested-method rejection hardening acceptance criteria

- Runtime CORS preflight handling rejects requests that omit `Access-Control-Request-Method`.
- Missing requested-method preflight requests deterministically return `400` with explicit diagnostics.
- Valid preflight requests containing requested method continue to use existing allow/deny method enforcement behavior.

### M38-S90 tracking (live status)

- [x] Added explicit runtime preflight guard for missing `Access-Control-Request-Method`.
- [x] Added deterministic missing-method rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight missing-method e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_without_requested_method_header`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_without_requested_method_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_with_auth_and_csrf_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S91 CORS preflight origin-header presence/shape enforcement hardening acceptance criteria

- Runtime CORS preflight handling requires `Origin` header presence.
- Missing or invalid `Origin` headers deterministically return `400` with explicit diagnostics.
- Valid preflight requests with valid origin continue through method/header policy validation.

### M38-S91 tracking (live status)

- [x] Added runtime preflight guard requiring `Origin` header on CORS preflight requests.
- [x] Added strict origin-shape validation on preflight `Origin` values before method/header checks.
- [x] Added deterministic rejection responses for missing/invalid preflight origin (`400` + fixed message bodies).
- [x] Added HTTP runtime CORS preflight origin-guard e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_without_origin_header`
  - `c_bin_http_runtime_rejects_cors_preflight_with_invalid_origin_header`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_without_origin_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_invalid_origin_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_without_requested_method_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_when_requested_headers_are_allowed`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S92 CORS preflight requested-method token-shape validation hardening acceptance criteria

- Runtime CORS preflight handling validates `Access-Control-Request-Method` token shape before allow-method policy matching.
- Invalid requested-method tokens deterministically return `400` with explicit diagnostics.
- Valid requested methods continue through existing allow-method membership checks (`403` for disallowed methods remains unchanged).

### M38-S92 tracking (live status)

- [x] Added runtime token-shape validation for preflight `Access-Control-Request-Method`.
- [x] Added deterministic invalid-token rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight invalid requested-method-token e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_when_requested_method_token_is_invalid`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_token_is_invalid`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_without_requested_method_header`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S93 CORS preflight requested-headers token-shape validation hardening acceptance criteria

- Runtime CORS preflight handling distinguishes malformed `Access-Control-Request-Headers` tokens from policy-denied header names.
- Invalid requested-header tokens deterministically return `400` with explicit diagnostics.
- Well-formed but disallowed requested headers continue to return deterministic `403`.

### M38-S93 tracking (live status)

- [x] Added runtime invalid-token tracking in preflight requested-headers validation.
- [x] Added deterministic invalid requested-headers rejection response (`400` + fixed message body).
- [x] Preserved deterministic disallowed-header rejection path (`403` + fixed message body).
- [x] Added HTTP runtime CORS preflight invalid requested-headers-token e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_when_requested_headers_token_is_invalid`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_headers_token_is_invalid`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_header_is_not_allowed`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_when_requested_headers_are_allowed`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S94 CORS preflight empty requested-headers rejection hardening acceptance criteria

- Runtime CORS preflight handling rejects explicitly empty `Access-Control-Request-Headers` values as malformed input.
- Empty requested-headers preflight requests deterministically return `400` with explicit diagnostics.
- Non-empty requested-headers behavior remains split between `400` malformed-token and `403` disallowed-policy outcomes.

### M38-S94 tracking (live status)

- [x] Tightened runtime requested-headers validator to classify empty values as invalid tokens.
- [x] Added deterministic empty requested-headers rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight empty requested-headers e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_empty_requested_headers_value`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_empty_requested_headers_value`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_headers_token_is_invalid`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_header_is_not_allowed`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S95 CORS preflight duplicate requested-headers rejection hardening acceptance criteria

- Runtime CORS preflight handling rejects duplicate `Access-Control-Request-Headers` tokens as malformed input.
- Duplicate requested-headers preflight requests deterministically return `400` with explicit diagnostics.
- Distinct requested-headers tokens continue through existing malformed/disallowed/allowed validation paths.

### M38-S95 tracking (live status)

- [x] Added duplicate-token detection for requested-headers validation (case-insensitive).
- [x] Added deterministic duplicate requested-headers rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight duplicate requested-headers e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_headers_tokens`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_headers_tokens`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_empty_requested_headers_value`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_header_is_not_allowed`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S96 CORS preflight duplicate requested-method header rejection hardening acceptance criteria

- Runtime CORS preflight handling rejects duplicate `Access-Control-Request-Method` header lines as malformed input.
- Duplicate requested-method preflight requests deterministically return `400` with explicit diagnostics.
- Single requested-method preflight requests continue through existing missing/invalid/disallowed/allowed branches.

### M38-S96 tracking (live status)

- [x] Added runtime header-occurrence counter for deterministic duplicate-header detection.
- [x] Added duplicate `Access-Control-Request-Method` rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight duplicate requested-method e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_method_headers`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_method_headers`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_token_is_invalid`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S97 CORS preflight duplicate requested-headers header-line rejection hardening acceptance criteria

- Runtime CORS preflight handling rejects duplicate `Access-Control-Request-Headers` header lines as malformed input.
- Duplicate requested-headers header lines deterministically return `400` with explicit diagnostics.
- Single requested-headers header-line requests continue through existing malformed/disallowed/allowed token checks.

### M38-S97 tracking (live status)

- [x] Added duplicate header-line detection for `Access-Control-Request-Headers` in preflight path.
- [x] Added deterministic duplicate requested-headers header-line rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight duplicate requested-headers header-line e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_headers_header_lines`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_headers_header_lines`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_requested_headers_tokens`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_headers_token_is_invalid`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S98 CORS preflight duplicate origin header-line rejection hardening acceptance criteria

- Runtime CORS preflight handling rejects duplicate `Origin` header lines as malformed input.
- Duplicate preflight origin headers deterministically return `400` with explicit diagnostics.
- Single origin-header preflight requests continue through existing missing/invalid/valid origin checks.

### M38-S98 tracking (live status)

- [x] Added duplicate `Origin` header-line detection in preflight path.
- [x] Added deterministic duplicate-origin rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight duplicate-origin e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_duplicate_origin_headers`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_origin_headers`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_without_origin_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_invalid_origin_header`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S99 CORS non-preflight duplicate origin header-line rejection hardening acceptance criteria

- Runtime CORS handling rejects duplicate `Origin` header lines on non-preflight requests as malformed input.
- Duplicate-origin non-preflight requests deterministically return `400` with explicit diagnostics.
- Rejected duplicate-origin non-preflight responses do not emit `Access-Control-Allow-Origin`.

### M38-S99 tracking (live status)

- [x] Added non-preflight duplicate-origin guard branch in runtime CORS handling.
- [x] Added deterministic duplicate-origin non-preflight rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS duplicate-origin non-preflight e2e coverage:
  - `c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_origin_headers`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_origin_headers`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_origin_headers`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S100 CORS preflight body rejection hardening acceptance criteria

- Runtime CORS preflight handling rejects preflight requests with non-empty request bodies.
- Preflight requests carrying request bodies deterministically return `400` with explicit diagnostics.
- Rejected preflight-body requests do not emit CORS allow-methods preflight headers.

### M38-S100 tracking (live status)

- [x] Added runtime preflight guard for non-empty request body payloads.
- [x] Added deterministic preflight-body rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight body-rejection e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_request_body`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_request_body`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_when_requested_method_is_not_allowed`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S101 CORS non-preflight invalid-origin rejection hardening acceptance criteria

- Runtime CORS handling rejects non-preflight requests that carry invalid `Origin` values.
- Invalid non-preflight origin requests deterministically return `400` with explicit diagnostics.
- Rejected invalid-origin non-preflight responses do not emit `Access-Control-Allow-Origin`.

### M38-S101 tracking (live status)

- [x] Added non-preflight invalid-origin guard branch in runtime CORS handling.
- [x] Added deterministic invalid-origin non-preflight rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS invalid-origin non-preflight e2e coverage:
  - `c_bin_http_runtime_rejects_cors_non_preflight_with_invalid_origin_header`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_invalid_origin_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_origin_headers`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S102 CORS non-preflight requested-method-header rejection hardening acceptance criteria

- Runtime CORS handling rejects non-preflight requests that include `Access-Control-Request-Method`.
- Non-preflight requests carrying preflight-only requested-method header deterministically return `400` with explicit diagnostics.
- Rejected responses do not emit `Access-Control-Allow-Origin`.

### M38-S102 tracking (live status)

- [x] Added non-preflight guard for `Access-Control-Request-Method` header presence.
- [x] Added deterministic non-preflight requested-method-header rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS requested-method non-preflight e2e coverage:
  - `c_bin_http_runtime_rejects_cors_non_preflight_with_requested_method_header`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_method_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_invalid_origin_header`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S103 CORS non-preflight requested-headers-header rejection hardening acceptance criteria

- Runtime CORS handling rejects non-preflight requests that include `Access-Control-Request-Headers`.
- Non-preflight requests carrying preflight-only requested-headers header deterministically return `400` with explicit diagnostics.
- Rejected responses do not emit `Access-Control-Allow-Origin`.

### M38-S103 tracking (live status)

- [x] Added non-preflight guard for `Access-Control-Request-Headers` header presence.
- [x] Added deterministic non-preflight requested-headers-header rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS requested-headers non-preflight e2e coverage:
  - `c_bin_http_runtime_rejects_cors_non_preflight_with_requested_headers_header`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_headers_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_method_header`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S104 CORS non-preflight duplicate requested-method-header rejection hardening acceptance criteria

- Runtime CORS handling rejects non-preflight requests containing duplicate `Access-Control-Request-Method` header lines.
- Duplicate requested-method non-preflight requests deterministically return `400` with explicit diagnostics.
- Rejected duplicate requested-method responses do not emit `Access-Control-Allow-Origin`.

### M38-S104 tracking (live status)

- [x] Added non-preflight duplicate-header-line guard for `Access-Control-Request-Method`.
- [x] Added deterministic duplicate requested-method non-preflight rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS duplicate requested-method non-preflight e2e coverage:
  - `c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_requested_method_headers`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_requested_method_headers`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_method_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S105 CORS non-preflight duplicate requested-headers-header rejection hardening acceptance criteria

- Runtime CORS handling rejects non-preflight requests containing duplicate `Access-Control-Request-Headers` header lines.
- Duplicate requested-headers non-preflight requests deterministically return `400` with explicit diagnostics.
- Rejected duplicate requested-headers responses do not emit `Access-Control-Allow-Origin`.

### M38-S105 tracking (live status)

- [x] Added non-preflight duplicate-header-line guard for `Access-Control-Request-Headers`.
- [x] Added deterministic duplicate requested-headers non-preflight rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS duplicate requested-headers non-preflight e2e coverage:
  - `c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_requested_headers_headers`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_requested_headers_headers`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_headers_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S106 CORS preflight cookie-header rejection hardening acceptance criteria

- Runtime CORS preflight handling rejects requests that include `Cookie` header.
- Preflight requests carrying cookies deterministically return `400` with explicit diagnostics.
- Rejected preflight cookie-header requests do not emit preflight allow-methods header block.

### M38-S106 tracking (live status)

- [x] Added preflight guard rejecting `Cookie` header presence.
- [x] Added deterministic preflight cookie-header rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight cookie-header e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_cookie_header`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_cookie_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_request_body`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S107 CORS preflight authorization-header rejection hardening acceptance criteria

- Runtime CORS preflight handling rejects requests that include `Authorization` header.
- Preflight requests carrying authorization header deterministically return `400` with explicit diagnostics.
- Rejected preflight authorization-header requests do not emit preflight allow-methods header block.

### M38-S107 tracking (live status)

- [x] Added preflight guard rejecting `Authorization` header presence.
- [x] Added deterministic preflight authorization-header rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight authorization-header e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_authorization_header`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_authorization_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_cookie_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S108 CORS non-preflight private-network-header rejection hardening acceptance criteria

- Runtime CORS handling rejects non-preflight requests that include `Access-Control-Request-Private-Network`.
- Non-preflight requests carrying private-network preflight header deterministically return `400` with explicit diagnostics.
- Rejected responses do not emit `Access-Control-Allow-Origin`.

### M38-S108 tracking (live status)

- [x] Added non-preflight guard for `Access-Control-Request-Private-Network` header presence.
- [x] Added deterministic non-preflight private-network-header rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS private-network non-preflight e2e coverage:
  - `c_bin_http_runtime_rejects_cors_non_preflight_with_requested_private_network_header`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_private_network_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_headers_header`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S109 CORS preflight private-network-header value validation hardening acceptance criteria

- Runtime CORS preflight handling validates `Access-Control-Request-Private-Network` value shape when the header is present.
- Invalid private-network preflight header values deterministically return `400` with explicit diagnostics.
- Rejected preflight invalid private-network requests do not emit preflight allow-methods header block.

### M38-S109 tracking (live status)

- [x] Added preflight validation for `Access-Control-Request-Private-Network` value (`true` only).
- [x] Added deterministic preflight private-network-header invalid-value rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight invalid private-network-header e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_invalid_private_network_header`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_invalid_private_network_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_authorization_header`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S110 CORS preflight duplicate private-network-header rejection hardening acceptance criteria

- Runtime CORS preflight handling rejects requests containing duplicate `Access-Control-Request-Private-Network` header lines.
- Duplicate private-network preflight header lines deterministically return `400` with explicit diagnostics.
- Rejected preflight duplicate private-network requests do not emit preflight allow-methods header block.

### M38-S110 tracking (live status)

- [x] Added preflight duplicate-header-line guard for `Access-Control-Request-Private-Network`.
- [x] Added deterministic duplicate private-network preflight rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS preflight duplicate private-network-header e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_duplicate_private_network_headers`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_private_network_headers`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_invalid_private_network_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S111 CORS non-preflight duplicate private-network-header rejection hardening acceptance criteria

- Runtime CORS handling rejects non-preflight requests containing duplicate `Access-Control-Request-Private-Network` header lines.
- Duplicate private-network non-preflight requests deterministically return `400` with explicit diagnostics.
- Rejected duplicate private-network non-preflight responses do not emit `Access-Control-Allow-Origin`.

### M38-S111 tracking (live status)

- [x] Added non-preflight duplicate-header-line guard for `Access-Control-Request-Private-Network`.
- [x] Added deterministic duplicate private-network non-preflight rejection response (`400` + fixed message body).
- [x] Added HTTP runtime CORS duplicate private-network non-preflight e2e coverage:
  - `c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_private_network_headers`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_duplicate_private_network_headers`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_non_preflight_with_requested_private_network_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_applies_cors_origin_header_on_success_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S112 CORS preflight private-network default-deny enforcement hardening acceptance criteria

- Runtime CORS preflight handling rejects `Access-Control-Request-Private-Network: true` by default (no implicit private-network opt-in).
- Private-network preflight requests deterministically return `403` with explicit diagnostics.
- Rejected private-network preflight responses do not emit preflight allow-methods header block.

### M38-S112 tracking (live status)

- [x] Added explicit default-deny branch for preflight `Access-Control-Request-Private-Network`.
- [x] Added deterministic private-network default-deny response (`403` + fixed message body).
- [x] Added HTTP runtime CORS preflight private-network default-deny e2e coverage:
  - `c_bin_http_runtime_rejects_cors_preflight_with_private_network_header_when_not_allowed`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_private_network_header_when_not_allowed`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_invalid_private_network_header`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_handles_cors_preflight_when_enabled`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S113 CORS preflight private-network explicit opt-in allow-path hardening acceptance criteria

- Runtime CORS preflight handling allows `Access-Control-Request-Private-Network: true` only when explicit policy/env opt-in is enabled.
- Allowed private-network preflight responses include `Access-Control-Allow-Private-Network: true`.
- Default behavior remains deny (`403`) when opt-in is not enabled.

### M38-S113 tracking (live status)

- [x] Added CORS policy/env bridge key `SEC4_RT_CORS_ALLOW_PRIVATE_NETWORK` (default `false`).
- [x] Added preflight allow path for private-network requests when opt-in is enabled.
- [x] Added `Access-Control-Allow-Private-Network: true` materialization for allowed preflight responses.
- [x] Added HTTP runtime CORS private-network preflight allow-path e2e coverage:
  - `c_bin_http_runtime_allows_cors_preflight_with_private_network_header_when_policy_enabled`
- [x] Revalidated related runtime paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_allows_cors_preflight_with_private_network_header_when_policy_enabled`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_private_network_header_when_not_allowed`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_rejects_cors_preflight_with_duplicate_private_network_headers`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S114 CORS private-network policy materialization bridge acceptance criteria

- `sec4.policy` supports explicit CORS toggle `cors.allow_private_network`.
- `sec4 run` materializes `cors.allow_private_network` into runtime env (`SEC4_RT_CORS_ALLOW_PRIVATE_NETWORK`) deterministically.
- Policy-enabled private-network preflight requests succeed with deterministic allow headers via the normal run command path.

### M38-S114 tracking (live status)

- [x] Added `cors.allow_private_network` to policy model/defaults and parser ingestion.
- [x] Wired `cmd_run` runtime env bridge for `SEC4_RT_CORS_ALLOW_PRIVATE_NETWORK`.
- [x] Added run-command e2e coverage for policy-driven private-network preflight allow path:
  - `run_command_oneshot_allows_private_network_preflight_when_cors_policy_enables_it`
- [x] Revalidated related policy/runtime bridging paths:
  - `cargo test -p sec4-core --test policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_applies_cors_from_policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_allows_private_network_preflight_when_cors_policy_enables_it`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S115 CORS preflight envelope policy materialization bridge acceptance criteria

- `sec4.policy` CORS preflight envelope fields are first-class and persisted:
  - `cors.allowed_methods`
  - `cors.allowed_headers`
  - `cors.exposed_headers`
  - `cors.max_age_seconds`
- `sec4 run` materializes those policy values into runtime env bridge keys deterministically.
- Runtime responses through `sec4 run` reflect policy-driven preflight envelope and expose-headers behavior.

### M38-S115 tracking (live status)

- [x] Added persisted CORS preflight envelope fields to policy model/defaults.
- [x] Added parser ingestion and validation for `cors.max_age_seconds` (`>= 1`).
- [x] Wired `sec4 run` env bridge keys:
  - `SEC4_RT_CORS_ALLOWED_METHODS`
  - `SEC4_RT_CORS_ALLOWED_HEADERS`
  - `SEC4_RT_CORS_EXPOSED_HEADERS`
  - `SEC4_RT_CORS_MAX_AGE_SECONDS`
- [x] Added/extended run-command e2e coverage:
  - `run_command_oneshot_applies_cors_from_policy` (includes expose-headers check)
  - `run_command_oneshot_applies_cors_preflight_methods_headers_and_max_age_from_policy`
- [x] Revalidated related policy/runtime bridging paths:
  - `cargo test -p sec4-core --test policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_applies_cors_from_policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_applies_cors_preflight_methods_headers_and_max_age_from_policy`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S116 security-headers HSTS/CSP policy materialization bridge acceptance criteria

- `sec4.policy` security-headers policy persists HSTS/CSP runtime-shape fields:
  - `security_headers.hsts.max_age_seconds`
  - `security_headers.hsts.include_subdomains`
  - `security_headers.hsts.preload`
  - `security_headers.csp.policy`
- `sec4 run` deterministically materializes those policy values into runtime env bridge keys.
- Runtime response headers emitted through `sec4 run` reflect HSTS/CSP policy settings on success path.

### M38-S116 tracking (live status)

- [x] Extended `SecurityHeadersPolicyConfig` with persisted HSTS/CSP fields and secure defaults.
- [x] Added parser ingestion/validation for:
  - `security_headers.hsts.max_age_seconds` (>= 0, and >= 1 when HSTS enabled)
  - `security_headers.hsts.include_subdomains`
  - `security_headers.hsts.preload`
  - `security_headers.csp.policy` (non-empty string when provided)
- [x] Wired `sec4 run` env bridge keys:
  - `SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED`
  - `SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS`
  - `SEC4_RT_SECURITY_HEADERS_HSTS_INCLUDE_SUBDOMAINS`
  - `SEC4_RT_SECURITY_HEADERS_HSTS_PRELOAD`
  - `SEC4_RT_SECURITY_HEADERS_CSP_ENABLED`
  - `SEC4_RT_SECURITY_HEADERS_CSP_REPORT_ONLY`
  - `SEC4_RT_SECURITY_HEADERS_CSP_POLICY`
- [x] Added run-command e2e coverage for policy-driven HSTS/CSP materialization:
  - `run_command_oneshot_applies_security_headers_hsts_and_csp_from_policy`
- [x] Revalidated related policy/runtime bridge paths:
  - `cargo test -p sec4-core --test policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_applies_security_headers_hsts_and_csp_from_policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_disables_security_headers_from_policy`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S117 CSRF cookie/header-name policy materialization bridge acceptance criteria

- `sec4.policy` persists CSRF token-name controls:
  - `csrf.cookie_name`
  - `csrf.header_name`
- `sec4 run` deterministically materializes CSRF name policy into runtime env bridge keys.
- Runtime CSRF middleware behavior through `sec4 run` respects policy-configured cookie/header token names.

### M38-S117 tracking (live status)

- [x] Extended `CsrfPolicyConfig` with persisted token-name fields and secure defaults.
- [x] Added parser ingestion/validation for:
  - `csrf.cookie_name` (non-empty string)
  - `csrf.header_name` (non-empty string)
- [x] Wired `sec4 run` env bridge keys:
  - `SEC4_RT_CSRF_COOKIE_NAME`
  - `SEC4_RT_CSRF_HEADER_NAME`
- [x] Added run-command e2e coverage for policy-driven CSRF token-name materialization:
  - `run_command_oneshot_applies_csrf_cookie_and_header_names_from_policy`
- [x] Revalidated related policy/runtime bridge paths:
  - `cargo test -p sec4-core --test policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_applies_csrf_cookie_and_header_names_from_policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_disables_csrf_from_policy`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S118 HTTP body/timeout policy materialization bridge acceptance criteria

- `sec4.policy` persists HTTP ingress runtime-shape controls:
  - `http.max_body_bytes`
  - `http.default_timeout_ms`
- `sec4 run` deterministically materializes those policy values into runtime env bridge keys.
- Runtime request-body enforcement through `sec4 run` reflects policy-configured `http.max_body_bytes` without requiring CLI override flags.

### M38-S118 tracking (live status)

- [x] Added `HttpPolicyConfig` persisted fields with defaults aligned to runtime baseline:
  - `max_body_bytes = 4096`
  - `default_timeout_ms = 200`
- [x] Added parser ingestion/validation for:
  - `http.max_body_bytes` (>= 1)
  - `http.default_timeout_ms` (>= 1)
- [x] Wired `sec4 run` env bridge keys:
  - `SEC4_RT_HTTP_MAX_BODY_BYTES`
  - `SEC4_RT_HTTP_SERVE_TIMEOUT_MS`
- [x] Added run-command e2e coverage for policy-driven ingress body-limit materialization:
  - `run_command_oneshot_applies_http_body_limit_from_policy`
- [x] Revalidated related policy/runtime bridge paths:
  - `cargo test -p sec4-core --test policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_applies_http_body_limit_from_policy`
  - `cargo test -p sec4 --test commands`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S119 HTTP policy-vs-CLI precedence hardening acceptance criteria

- `sec4 run` keeps policy-driven HTTP ingress defaults active when CLI override flags are absent.
- CLI flags deterministically override policy-provided HTTP ingress values:
  - `--max-body-bytes`
  - `--serve-timeout-ms`
- Run-command integration coverage locks both precedence branches (policy-default path and CLI-override path).

### M38-S119 tracking (live status)

- [x] Added run-command e2e coverage for CLI override precedence over policy body limit:
  - `run_command_oneshot_cli_max_body_bytes_overrides_policy_limit`
- [x] Added run-command e2e coverage for CLI timeout override precedence over policy timeout:
  - `run_command_oneshot_cli_serve_timeout_overrides_policy_timeout`
- [x] Revalidated command runtime bridge behavior:
  - `cargo test -p sec4 --test commands run_command_oneshot_cli_max_body_bytes_overrides_policy_limit`
  - `cargo test -p sec4 --test commands run_command_oneshot_cli_serve_timeout_overrides_policy_timeout`
  - `cargo test -p sec4 --test commands`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S120 HTTP ingress override input hardening acceptance criteria

- `sec4 run` rejects zero-value HTTP ingress override flags deterministically before build/runtime launch:
  - `--max-body-bytes 0`
  - `--serve-timeout-ms 0`
- Invalid zero overrides fail fast with stable CLI diagnostics and non-zero exit status.
- Existing policy-default and non-zero override behavior remains unchanged.

### M38-S120 tracking (live status)

- [x] Added fail-fast CLI validation in `cmd_run` for:
  - `--max-body-bytes` (must be >= 1)
  - `--serve-timeout-ms` (must be >= 1)
- [x] Added integration tests for deterministic invalid-zero diagnostics:
  - `run_command_rejects_zero_max_body_bytes_override`
  - `run_command_rejects_zero_serve_timeout_ms_override`
- [x] Revalidated command runtime bridge behavior:
  - `cargo test -p sec4 --test commands run_command_rejects_zero_max_body_bytes_override`
  - `cargo test -p sec4 --test commands run_command_rejects_zero_serve_timeout_ms_override`
  - `cargo test -p sec4 --test commands`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S121 HTTP max-header-bytes policy/runtime materialization bridge acceptance criteria

- `sec4.policy` persists HTTP ingress header-size control:
  - `http.max_header_bytes`
- `sec4 run` deterministically materializes `http.max_header_bytes` into runtime env bridge key:
  - `SEC4_RT_HTTP_MAX_HEADER_BYTES`
- Runtime HTTP ingress enforces configured header-size limit and emits deterministic rejection response when exceeded.

### M38-S121 tracking (live status)

- [x] Extended `HttpPolicyConfig` with persisted `max_header_bytes` field and default baseline.
- [x] Added parser ingestion/validation for:
  - `http.max_header_bytes` (>= 1)
- [x] Wired `sec4 run` env bridge key:
  - `SEC4_RT_HTTP_MAX_HEADER_BYTES`
- [x] Added runtime ingress enforcement for configured header-size cap with deterministic `431` response path.
- [x] Added run-command e2e coverage for policy-driven ingress header-limit materialization:
  - `run_command_oneshot_applies_http_header_limit_from_policy`
- [x] Revalidated related policy/runtime bridge paths:
  - `cargo test -p sec4-core --test policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_applies_http_header_limit_from_policy`
  - `cargo test -p sec4 --test commands`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S122 HTTP max-multipart-bytes policy/runtime materialization bridge acceptance criteria

- `sec4.policy` persists HTTP multipart ingress-size control:
  - `http.max_multipart_bytes`
- `sec4 run` deterministically materializes `http.max_multipart_bytes` into runtime env bridge key:
  - `SEC4_RT_HTTP_MAX_MULTIPART_BYTES`
- Runtime HTTP ingress enforces configured multipart-size limit and emits deterministic `413` rejection when exceeded.

### M38-S122 tracking (live status)

- [x] Extended `HttpPolicyConfig` with persisted `max_multipart_bytes` field and default baseline.
- [x] Added parser ingestion/validation for:
  - `http.max_multipart_bytes` (>= 1)
- [x] Wired `sec4 run` env bridge key:
  - `SEC4_RT_HTTP_MAX_MULTIPART_BYTES`
- [x] Added runtime ingress enforcement for configured multipart-size cap with deterministic `413` response path.
- [x] Added run-command e2e coverage for policy-driven ingress multipart-limit materialization:
  - `run_command_oneshot_applies_http_multipart_limit_from_policy`
- [x] Revalidated related policy/runtime bridge paths:
  - `cargo test -p sec4-core --test policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_applies_http_multipart_limit_from_policy`
  - `cargo test -p sec4 --test commands`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S123 HTTP ingress generic body-limit enforcement acceptance criteria

- Runtime ingress enforces configured body-size cap for non-JSON handler paths that do not call `req.json(...)`.
- Oversized non-JSON request bodies are rejected deterministically with `413` before successful route responses are emitted.
- Existing JSON-specific body-limit behavior (`req.json` + `LIMIT.BODY_BYTES`) remains unchanged.

### M38-S123 tracking (live status)

- [x] Added runtime generic body-limit enforcement branch for non-JSON paths (`body_limit_exceeded && !json_checked`).
- [x] Added run-command e2e coverage:
  - `run_command_oneshot_enforces_body_limit_for_non_json_handler_paths`
- [x] Revalidated non-regression for JSON limit behavior:
  - `run_command_oneshot_applies_http_body_limit_from_policy`
- [x] Revalidated runtime command behavior:
  - `cargo test -p sec4 --test commands run_command_oneshot_enforces_body_limit_for_non_json_handler_paths`
  - `cargo test -p sec4 --test commands run_command_oneshot_applies_http_body_limit_from_policy`
  - `cargo test -p sec4 --test commands`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S124 HTTP max-concurrency policy/runtime materialization bridge acceptance criteria

- `sec4.policy` persists HTTP ingress concurrency control:
  - `http.max_concurrency`
- `sec4 run` deterministically materializes `http.max_concurrency` into runtime env bridge key:
  - `SEC4_RT_HTTP_MAX_CONCURRENCY`
- Runtime HTTP ingress enforces configured concurrency cap and emits deterministic rejection response when over-cap connections are accepted.

### M38-S124 tracking (live status)

- [x] Extended `HttpPolicyConfig` with persisted `max_concurrency` field and default baseline.
- [x] Added parser ingestion/validation for:
  - `http.max_concurrency` (>= 1)
- [x] Wired `sec4 run` env bridge key:
  - `SEC4_RT_HTTP_MAX_CONCURRENCY`
- [x] Added runtime ingress enforcement for configured concurrency cap with deterministic `503` throttle response path.
- [x] Added run-command e2e coverage for policy-driven ingress max-concurrency materialization:
  - `run_command_oneshot_applies_http_max_concurrency_from_policy`
- [x] Revalidated related policy/runtime bridge paths:
  - `cargo test -p sec4-core --test policy`
  - `cargo test -p sec4 --test commands run_command_oneshot_applies_http_max_concurrency_from_policy`
  - `cargo test -p sec4 --test commands`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S125 HTTP max-concurrency CLI override bridge + precedence hardening acceptance criteria

- `sec4 run` accepts explicit max-concurrency override flag:
  - `--max-concurrency <n>`
- CLI max-concurrency override deterministically materializes to runtime env bridge key:
  - `SEC4_RT_HTTP_MAX_CONCURRENCY`
- CLI override takes precedence over policy `http.max_concurrency` and suppresses policy-driven throttling when override is higher.
- CLI rejects invalid override values (`0`) with deterministic diagnostics and non-zero exit.

### M38-S125 tracking (live status)

- [x] Added `sec4 run` flag:
  - `--max-concurrency`
- [x] Wired CLI override into runtime env bridge key:
  - `SEC4_RT_HTTP_MAX_CONCURRENCY`
- [x] Added deterministic run-command precedence coverage:
  - `run_command_cli_max_concurrency_overrides_policy_limit`
- [x] Added deterministic CLI invalid-input coverage:
  - `run_command_rejects_zero_max_concurrency_override`
- [x] Revalidated related command/runtime behavior:
  - `cargo test -p sec4 --test commands run_command_cli_max_concurrency_overrides_policy_limit`
  - `cargo test -p sec4 --test commands run_command_rejects_zero_max_concurrency_override`
  - `cargo test -p sec4 --test commands`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S126 HTTP max-concurrency runtime fallback/clamp hardening acceptance criteria

- Runtime deterministically falls back to safe concurrency defaults when `SEC4_RT_HTTP_MAX_CONCURRENCY` is invalid or empty.
- Runtime deterministically clamps over-cap `SEC4_RT_HTTP_MAX_CONCURRENCY` values to bounded safe limits.
- CLI help surface explicitly documents `--max-concurrency` runtime bridge flag.

### M38-S126 tracking (live status)

- [x] Added runtime e2e fallback coverage for invalid max-concurrency env values:
  - `c_bin_http_runtime_max_concurrency_invalid_env_falls_back_to_default_when_clang_available`
- [x] Added runtime e2e fallback coverage for empty max-concurrency env values:
  - `c_bin_http_runtime_max_concurrency_empty_env_falls_back_to_default_when_clang_available`
- [x] Added runtime e2e clamp coverage for over-cap max-concurrency env values:
  - `c_bin_http_runtime_max_concurrency_over_cap_env_is_clamped_when_clang_available`
- [x] Updated CLI run help contract coverage:
  - `run_command_help_lists_runtime_bridge_flags`
- [x] Revalidated related runtime/CLI behavior:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `cargo test -p sec4 --test json_output run_command_help_lists_runtime_bridge_flags`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S127 HTTP max-concurrency queue-boundary deterministic throttle coverage acceptance criteria

- Runtime queue-boundary pressure with `SEC4_RT_HTTP_MAX_CONCURRENCY=1` produces exactly one successful route response and one deterministic `503` throttle response for a two-client concurrent ingress attempt.
- Throttle response contract is deterministic under queue pressure:
  - status: `503 Service Unavailable`
  - trace header: `X-Trace-Id: rt-*`
  - body: `server busy: max concurrency exceeded`
  - stable response framing headers (`Content-Type`, `Content-Length`, `Connection`).
- Coverage is runtime e2e (`c-bin`), not only CLI argument/path validation.

### M38-S127 tracking (live status)

- [x] Added queue-boundary runtime e2e throttle coverage:
  - `c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
- [x] Locked deterministic success+throttle pair contract for two-client contention:
  - exactly one `200 OK`
  - exactly one `503 Service Unavailable`
  - deterministic throttle envelope assertions (status/body/trace/framing headers)
- [x] Revalidated related runtime behavior:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S128 HTTP max-concurrency throttle-response security-header parity hardening acceptance criteria

- When security-headers middleware is enabled, runtime `503` max-concurrency throttle responses include the same baseline security headers as successful route responses.
- In oneshot mode, once a request is served, pending and newly-accepted backlog clients are deterministically drained through throttle responses (instead of silent close paths), preserving throttle envelope parity under contention.
- Coverage proves security-header parity and deterministic success/throttle split in a two-client contention scenario.

### M38-S128 tracking (live status)

- [x] Added runtime oneshot backlog drain helper for deterministic post-success throttle handling:
  - `sec4_rt_drain_oneshot_backlog_with_throttle(...)`
- [x] Hardened oneshot serve loop to drain pending/new backlog with throttle after first served request.
- [x] Added security-header parity e2e test under contention:
  - `c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
- [x] Revalidated related runtime/backend behavior:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S129 HTTP max-concurrency oneshot late-connection deterministic drain coverage acceptance criteria

- In oneshot mode with `SEC4_RT_HTTP_MAX_CONCURRENCY=1`, a late second client connection established after the first client is accepted is deterministically drained with a `503` throttle response.
- Response ordering is deterministic for the late-connection path:
  - first served client receives `200` with trace `rt-1`,
  - late backlog client receives `503` with trace `rt-2`.
- Late-drain throttle envelope remains deterministic:
  - `Content-Type: text/plain; charset=utf-8`
  - `Connection: close`
  - body `server busy: max concurrency exceeded`.

### M38-S129 tracking (live status)

- [x] Added runtime e2e late-connection drain coverage:
  - `c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
- [x] Locked deterministic ordering for first-served and late-drained clients (`rt-1` success, `rt-2` throttle).
- [x] Revalidated max-concurrency suite coverage:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S130 HTTP max-concurrency burst-ingress trace/order deterministic coverage acceptance criteria

- In oneshot mode with `SEC4_RT_HTTP_MAX_CONCURRENCY=1`, a three-client burst (one accepted + two late backlog connections) preserves deterministic trace/order contract:
  - first served response is `200` with `X-Trace-Id: rt-1`,
  - second and third responses are `503` throttle with `X-Trace-Id: rt-2` and `rt-3` respectively.
- Burst-ingress throttle responses preserve deterministic framing envelope under contention:
  - `Content-Type: text/plain; charset=utf-8`
  - `Content-Length: 37`
  - `Connection: close`.
- Coverage is runtime e2e (`c-bin`) and validates burst-path order invariants beyond two-client queue-boundary scenarios.

### M38-S130 tracking (live status)

- [x] Added runtime e2e burst-ingress trace/order coverage:
  - `c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- [x] Hardened late/burst contention contract assertions around deterministic status/trace/framing invariants under socket timing variance.
- [x] Revalidated max-concurrency suite coverage:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S131 HTTP max-concurrency throttle body-delivery socket-close determinism hardening acceptance criteria

- Runtime throttle close path under queue/backlog contention deterministically preserves advertised payload delivery (`Content-Length: 37` plus body bytes) instead of timing-sensitive header-only outcomes.
- Socket-close sequence for throttle responses is hardened to use bounded graceful write-shutdown + peer-input drain before final close, avoiding close-time reset races in late/burst oneshot contention paths.
- Runtime e2e late/burst contention coverage requires full throttle body presence alongside existing status/trace/framing assertions.

### M38-S131 tracking (live status)

- [x] Added runtime throttle close hardening helper:
  - `sec4_rt_finalize_throttle_socket_close(...)`
- [x] Wired throttle close hardening into all max-concurrency throttle close sites:
  - queue-boundary overflow path
  - oneshot pending backlog drain path
  - oneshot newly-accepted backlog drain path
- [x] Restored strict payload assertions for race-prone contention tests:
  - `c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
  - `c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
- [x] Revalidated max-concurrency/runtime contract coverage:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_queue_boundary_emits_deterministic_throttle_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_throttle_response_preserves_security_headers_when_enabled_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S132 HTTP max-concurrency throttle close-drain timeout-budget coverage hardening acceptance criteria

- Throttle close-drain loop enforces deterministic bounded timeout budget instead of fixed attempt count.
- Timeout budget is configurable via runtime env bridge `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS` with safe defaults.
- Late/burst contention coverage validates throttle body delivery under backlog trailing-input pressure while keeping bounded tail-latency contract.

### M38-S132 tracking (live status)

- [x] Added runtime throttle drain timeout-budget bridge:
  - `sec4_rt_http_throttle_drain_timeout_ms(...)`
  - `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS` (`default=20ms`, bounded max clamp)
- [x] Reworked throttle close helper to deadline-driven bounded drain:
  - `sec4_rt_finalize_throttle_socket_close(...)` now uses time budget + remaining-time select waits.
- [x] Hardened late/burst contention runtime e2e coverage with backlog trailing-input pressure and bounded-tail assertions:
  - `c_bin_http_runtime_max_concurrency_oneshot_late_connection_is_drain_throttled_when_clang_available`
  - `c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_preserves_trace_order_when_clang_available`

### M38-S133 HTTP max-concurrency throttle drain-timeout env fallback/clamp hardening acceptance criteria

- Invalid `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS` values deterministically fall back to safe defaults.
- Over-cap timeout values are deterministically clamped to bounded runtime maximum.
- Coverage pins fallback/clamp behavior without regressing max-concurrency route success contracts.

### M38-S133 tracking (live status)

- [x] Added runtime e2e fallback coverage for invalid throttle-drain-timeout env values:
  - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_invalid_env_falls_back_to_default_when_clang_available`
- [x] Added runtime e2e clamp coverage for over-cap throttle-drain-timeout env values:
  - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_over_cap_env_is_clamped_when_clang_available`
- [x] Revalidated max-concurrency/runtime contract coverage:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S134 HTTP max-concurrency throttle close-drain low-timeout deterministic body contract coverage acceptance criteria

- With `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS` configured to a low value, oneshot burst contention still preserves deterministic throttle body delivery (`Content-Length: 37` + full payload bytes).
- Low-timeout contention path preserves deterministic trace/order invariants (`rt-1` success, `rt-2`/`rt-3` throttle).
- Low-timeout contention path preserves bounded tail-latency contract under trailing-input backlog pressure.

### M38-S134 tracking (live status)

- [x] Added runtime e2e low-timeout burst coverage:
  - `c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
- [x] Locked deterministic low-timeout contention contract:
  - throttle body delivery remains present for both throttled clients,
  - trace/order contract remains deterministic (`rt-1`, `rt-2`, `rt-3`),
  - bounded tail-latency assertion remains enforced.
- [x] Revalidated max-concurrency/runtime contract coverage:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_oneshot_burst_ingress_low_drain_timeout_preserves_throttle_body_when_clang_available`
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S135 HTTP max-concurrency throttle close-drain zero/near-zero timeout fallback behavior hardening acceptance criteria

- `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS=0` follows deterministic safe fallback semantics (default timeout budget), not zero-budget close behavior.
- Zero-value fallback preserves existing route-success contracts in oneshot runtime paths.
- Coverage explicitly locks zero-timeout fallback behavior alongside existing invalid/over-cap timeout tests.

### M38-S135 tracking (live status)

- [x] Added runtime e2e zero-value fallback coverage:
  - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_zero_env_falls_back_to_default_when_clang_available`
- [x] Revalidated max-concurrency runtime contract suite including timeout fallback/clamp matrix:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S136 HTTP max-concurrency throttle close-drain minimum-budget deterministic latency/contract hardening acceptance criteria

- With minimum configured timeout budget (`SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS=1`), late-connection oneshot contention preserves deterministic throttle body-delivery contract.
- Minimum-budget late-connection path preserves deterministic trace/order invariants (`rt-1` success, `rt-2` throttle).
- Minimum-budget late-connection path remains bounded in tail-latency under trailing-input backlog pressure.

### M38-S136 tracking (live status)

- [x] Added runtime e2e low-timeout late-connection coverage:
  - `c_bin_http_runtime_max_concurrency_oneshot_late_connection_low_drain_timeout_preserves_throttle_body_when_clang_available`
- [x] Revalidated max-concurrency runtime contract suite including minimum-budget burst+late contention paths:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S137 HTTP max-concurrency throttle drain-timeout negative-value fallback behavior hardening acceptance criteria

- Negative `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS` values deterministically fall back to safe default timeout budget.
- Negative-value fallback preserves oneshot runtime route-success contracts.
- Coverage explicitly locks negative-value fallback alongside zero/invalid/over-cap timeout matrix.

### M38-S137 tracking (live status)

- [x] Added runtime e2e negative-value fallback coverage:
  - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_negative_env_falls_back_to_default_when_clang_available`
- [x] Revalidated max-concurrency runtime contract suite including timeout fallback/clamp matrix:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S138 HTTP max-concurrency throttle drain-timeout whitespace-token fallback behavior hardening acceptance criteria

- Whitespace-padded `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS` values deterministically fall back to safe default timeout budget.
- Whitespace-token fallback preserves oneshot runtime route-success contracts.
- Coverage explicitly locks whitespace-token fallback semantics in the timeout fallback matrix.

### M38-S138 tracking (live status)

- [x] Added runtime e2e whitespace-token fallback coverage:
  - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_whitespace_env_falls_back_to_default_when_clang_available`
- [x] Revalidated max-concurrency timeout fallback matrix:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S139 HTTP max-concurrency throttle drain-timeout empty-token fallback behavior hardening acceptance criteria

- Empty `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS` values deterministically fall back to safe default timeout budget.
- Empty-token fallback preserves oneshot runtime route-success contracts.
- Coverage explicitly locks empty-token fallback semantics in the timeout fallback matrix.

### M38-S139 tracking (live status)

- [x] Added runtime e2e empty-token fallback coverage:
  - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_empty_env_falls_back_to_default_when_clang_available`
- [x] Revalidated max-concurrency timeout fallback matrix:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S140 HTTP max-concurrency throttle drain-timeout malformed-token fallback behavior hardening acceptance criteria

- Malformed `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS` values (non-numeric suffix/prefix) deterministically fall back to safe default timeout budget.
- Malformed-token fallback preserves oneshot runtime route-success contracts.
- Coverage explicitly locks malformed-token fallback semantics in the timeout fallback matrix.

### M38-S140 tracking (live status)

- [x] Added runtime e2e malformed-token fallback coverage:
  - `c_bin_http_runtime_max_concurrency_throttle_drain_timeout_malformed_env_falls_back_to_default_when_clang_available`
- [x] Revalidated max-concurrency timeout fallback matrix:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S141 HTTP max-concurrency queue-boundary low-timeout deterministic throttle contract hardening acceptance criteria

- Queue-boundary contention with minimum timeout budget (`SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS=1`) preserves deterministic one-success/one-throttle contract.
- Low-timeout queue-boundary path preserves deterministic throttle body delivery (`Content-Length: 37` + full payload bytes).
- Low-timeout queue-boundary path preserves bounded tail-latency under trailing-input pressure.

### M38-S141 tracking (live status)

- [x] Added runtime e2e low-timeout queue-boundary coverage:
  - `c_bin_http_runtime_max_concurrency_queue_boundary_low_drain_timeout_preserves_throttle_body_when_clang_available`
- [x] Revalidated max-concurrency/runtime contract suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S142 HTTP max-concurrency throttle drain-timeout runtime-cache hardening acceptance criteria

- Runtime parses `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS` once per `http.serve` lifecycle and reuses it for all throttle close paths.
- Oneshot backlog drain and queue-overflow throttle close paths receive the same serve-scoped timeout value, removing repeated per-socket env parsing.
- Existing max-concurrency contention contracts remain unchanged.

### M38-S142 tracking (live status)

- [x] Added serve-scoped throttle drain-timeout caching in runtime:
  - `sec4_rt_http_serve(...)` now computes `throttle_drain_timeout_ms` once.
- [x] Propagated cached timeout through throttle close path wiring:
  - `sec4_rt_drain_oneshot_backlog_with_throttle(...)`
  - queue-overflow close path
  - `sec4_rt_finalize_throttle_socket_close(...)`
- [x] Revalidated runtime contracts:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### M38-S143 HTTP max-concurrency timeout fallback-matrix assertion-helper dedup acceptance criteria

- Timeout fallback matrix tests share one deterministic route-success assertion helper instead of repeated inline assert blocks.
- Helper keeps failure diagnostics case-specific via explicit case labels.
- Fallback/clamp matrix behavior remains unchanged.

### M38-S143 tracking (live status)

- [x] Added matrix assertion helper:
  - `assert_throttle_drain_timeout_env_route_success(...)`
- [x] Migrated timeout fallback/clamp tests to helper-backed assertions.
- [x] Revalidated max-concurrency suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S144 HTTP max-concurrency oneshot-spawn helper dedup acceptance criteria

- Max-concurrency contention tests use one canonical oneshot runtime spawn helper.
- Helper centralizes default runtime env wiring (`serve mode`, `serve timeout`, `max concurrency`, `port`) with optional timeout override.
- Contention tests preserve existing contracts while reducing setup drift risk.

### M38-S144 tracking (live status)

- [x] Added canonical spawn helper:
  - `spawn_max_concurrency_oneshot_binary(...)`
- [x] Enabled optional `SEC4_RT_HTTP_THROTTLE_DRAIN_TIMEOUT_MS` injection through helper.
- [x] Revalidated max-concurrency suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S145 HTTP max-concurrency queue/security spawn-helper adoption acceptance criteria

- Queue-boundary contention tests use canonical oneshot-spawn helper.
- Security-header parity contention test uses canonical oneshot-spawn helper.
- Queue/security contracts remain deterministic after helper migration.

### M38-S145 tracking (live status)

- [x] Migrated queue-boundary contention spawn path to `spawn_max_concurrency_oneshot_binary(...)`.
- [x] Migrated security-header parity contention spawn path to `spawn_max_concurrency_oneshot_binary(...)`.
- [x] Revalidated max-concurrency suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S146 HTTP max-concurrency late-connection spawn-helper adoption acceptance criteria

- Late-connection contention tests (default + low-timeout) use canonical oneshot-spawn helper.
- Late-connection deterministic status/trace/body contracts remain unchanged after helper migration.
- Low-timeout bounded-tail assertions remain intact.

### M38-S146 tracking (live status)

- [x] Migrated late-connection default timeout test spawn path to helper.
- [x] Migrated late-connection low-timeout test spawn path to helper.
- [x] Revalidated max-concurrency suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S147 HTTP max-concurrency burst-ingress spawn-helper adoption acceptance criteria

- Burst-ingress contention tests (default + low-timeout) use canonical oneshot-spawn helper.
- Burst deterministic trace/order and throttle body contracts remain unchanged after helper migration.
- Low-timeout bounded-tail assertions remain intact.

### M38-S147 tracking (live status)

- [x] Migrated burst-ingress default timeout test spawn path to helper.
- [x] Migrated burst-ingress low-timeout test spawn path to helper.
- [x] Revalidated max-concurrency suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S148 HTTP max-concurrency c-bin fixture-build helper introduction acceptance criteria

- Max-concurrency runtime e2e tests share a canonical c-bin fixture builder.
- Helper centralizes project scaffolding, manifest/source writes, build invocation, and binary existence assertions.
- Fixture-build failure messages remain case-specific through explicit labels.

### M38-S148 tracking (live status)

- [x] Added canonical fixture helper:
  - `build_c_bin_fixture(...)`
- [x] Helper now owns deterministic fixture-build assertions for max-concurrency runtime e2e tests.
- [x] Revalidated max-concurrency suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S149 HTTP max-concurrency queue fixture-helper adoption acceptance criteria

- Queue-boundary tests (default + low-timeout) use canonical c-bin fixture helper.
- Queue fixture setup duplication is removed without changing contention contracts.
- Queue deterministic one-success/one-throttle invariants remain intact.

### M38-S149 tracking (live status)

- [x] Migrated queue-boundary default fixture setup to `build_c_bin_fixture(...)`.
- [x] Migrated queue-boundary low-timeout fixture setup to `build_c_bin_fixture(...)`.
- [x] Revalidated max-concurrency suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S150 HTTP max-concurrency late/burst fixture-helper adoption acceptance criteria

- Late-connection and burst-ingress tests (default + low-timeout variants) use canonical c-bin fixture helper.
- Late/burst fixture setup duplication is removed without changing contention contracts.
- Existing status/trace/body/bounded-tail assertions remain intact.

### M38-S150 tracking (live status)

- [x] Migrated late-connection default + low-timeout fixture setup to helper.
- [x] Migrated burst-ingress default + low-timeout fixture setup to helper.
- [x] Revalidated max-concurrency suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S151 HTTP max-concurrency env/security fixture-helper adoption acceptance criteria

- Timeout env-matrix helper path and security-header parity path use canonical c-bin fixture helper where applicable.
- Max-concurrency test harness setup is consistently helper-driven across env fallback, queue, late, burst, and security branches.
- Consolidated helper structure reduces fixture drift risk while preserving deterministic runtime contracts.

### M38-S151 tracking (live status)

- [x] Migrated env fallback path fixture build in `run_http_runtime_health_with_max_concurrency_env(...)` to helper.
- [x] Migrated security-header parity fixture build path to helper.
- [x] Revalidated consolidated contract suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
  - `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
  - `scripts/test-roadmap-closure-gate-alignment.sh`

### M38-S152 HTTP max-concurrency helper-layer determinism regression guard expansion acceptance criteria

- Max-concurrency contention tests centralize deterministic guard logic behind explicit helper functions.
- Guard helper adoption keeps queue/late/burst/security runtime contracts unchanged.
- Regression diagnostics remain attempt-scoped and deterministic.

### M38-S152 tracking (live status)

- [x] Added deterministic helper-layer guard surface across max-concurrency contention tests.
- [x] Preserved one-success/one-throttle and ordered-trace contracts after helper adoption.
- [x] Revalidated max-concurrency contract suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S153 HTTP max-concurrency connector retry helper introduction acceptance criteria

- Connection retry loops are centralized in one helper for deterministic retry cadence.
- Connector failure handling is centralized with child terminate+wait fallback.
- Queue/late/burst tests no longer duplicate raw retry loops.

### M38-S153 tracking (live status)

- [x] Added `connect_with_retry(...)`.
- [x] Added `connect_with_retry_or_terminate(...)`.
- [x] Adopted helper in late/burst setup paths.

### M38-S154 HTTP max-concurrency socket I/O helper layer adoption acceptance criteria

- Read-timeout setup uses one canonical helper.
- Request writes and trailing-noise writes use canonical helper wrappers with deterministic diagnostics.
- Response reads use one helper surface across contention tests.

### M38-S154 tracking (live status)

- [x] Added `set_stream_read_timeout(...)`.
- [x] Added `write_http_request(...)` and `write_http_trailing_noise(...)`.
- [x] Added `read_http_response(...)` and migrated queue/late/burst/security tests.

### M38-S155 HTTP max-concurrency child-exit deterministic wait helper acceptance criteria

- Child wait loops are centralized in one bounded helper with kill+wait timeout fallback.
- All contention tests use the same wait window and polling cadence contracts.
- Exit timeout failure diagnostics remain test-specific.

### M38-S155 tracking (live status)

- [x] Added `wait_for_child_exit_or_terminate(...)`.
- [x] Migrated queue/late/burst/security contention paths to helper wait surface.

### M38-S156 HTTP max-concurrency response contract predicate helper introduction acceptance criteria

- Success/throttle contract assertions are centralized in canonical predicate helpers.
- Trace-specific contract checks for ordered paths use explicit helper entrypoints.
- Security-header parity checks use one canonical predicate.

### M38-S156 tracking (live status)

- [x] Added canonical response predicates:
  - `response_has_success_contract(...)`
  - `response_has_success_contract_with_trace(...)`
  - `response_has_throttle_contract(...)`
  - `response_has_throttle_contract_with_trace(...)`
  - `response_has_default_security_headers(...)`

### M38-S157 HTTP max-concurrency pair-selection and observation formatter helper introduction acceptance criteria

- Pair success/throttle selection logic is centralized to prevent branch drift.
- Attempt observation rendering uses canonical formatters for two-response and three-response paths.
- Contention test failure envelopes remain deterministic and readable.

### M38-S157 tracking (live status)

- [x] Added `select_success_and_throttle(...)`.
- [x] Added `format_two_response_observation(...)` and `format_three_response_observation(...)`.
- [x] Migrated queue/security/late/burst attempt observation strings to helper formatters.

### M38-S158 HTTP max-concurrency queue contention helper adoption acceptance criteria

- Queue-boundary default and low-timeout tests consume canonical connect/I/O/wait/predicate helpers.
- Queue contention invariants remain deterministic after helper migration.
- Low-timeout bounded-tail contract remains intact.

### M38-S158 tracking (live status)

- [x] Migrated queue-boundary default contention test to helper layer.
- [x] Migrated queue-boundary low-timeout contention test to helper layer.

### M38-S159 HTTP max-concurrency security-header parity helper adoption acceptance criteria

- Security-header parity contention test consumes canonical connect/I/O/wait and predicate helpers.
- Success/throttle header parity is asserted through canonical header predicate helper.
- Security parity contract remains unchanged.

### M38-S159 tracking (live status)

- [x] Migrated security-header parity contention test to helper layer.
- [x] Preserved deterministic parity assertions for success and throttle envelopes.

### M38-S160 HTTP max-concurrency late-connection helper adoption acceptance criteria

- Late-connection default and low-timeout tests consume canonical connect/I/O/wait/predicate helpers.
- Ordered `rt-1 success -> rt-2 throttle` contract remains deterministic.
- Bounded-tail contract remains intact for both timeout branches.

### M38-S160 tracking (live status)

- [x] Migrated late-connection default contention path to helper layer.
- [x] Migrated late-connection low-timeout contention path to helper layer.

### M38-S161 HTTP max-concurrency burst-ingress helper adoption acceptance criteria

- Burst-ingress default and low-timeout tests consume canonical connect/I/O/wait/predicate helpers.
- Ordered `rt-1 success -> rt-2/rt-3 throttle` contract remains deterministic.
- Bounded-tail contract remains intact for both timeout branches.

### M38-S161 tracking (live status)

- [x] Migrated burst-ingress default contention path to helper layer.
- [x] Migrated burst-ingress low-timeout contention path to helper layer.
- [x] Revalidated helper-layer contract suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S162 HTTP max-concurrency connector-thread helper normalization for queue/security contention paths acceptance criteria

- Queue/security contention tests no longer inline connector thread spawn/join blocks.
- Connector thread failure handling is centralized behind one helper path.
- Queue/security contention contracts remain unchanged.

### M38-S162 tracking (live status)

- [x] Added shared queue/security connector-thread helper surface.
- [x] Migrated queue default/low-timeout and security parity tests to helper-based connector setup.

### M38-S163 HTTP max-concurrency parallel connector-pair helper introduction acceptance criteria

- Canonical helper builds two parallel connector attempts with deterministic retry cadence.
- Helper emits case-specific first/second connection failure diagnostics.
- Child terminate+wait fallback is preserved on connector failure.

### M38-S163 tracking (live status)

- [x] Added `connect_pair_in_parallel_or_terminate(...)`.
- [x] Preserved first/second connector-specific failure envelopes in migrated tests.

### M38-S164 HTTP max-concurrency two/three response read helper introduction acceptance criteria

- Two-stream and three-stream response read collection are centralized.
- Contention tests use canonical multi-response readers instead of repeated inline calls.
- Failure envelopes remain deterministic.

### M38-S164 tracking (live status)

- [x] Added `read_two_http_responses(...)`.
- [x] Added `read_three_http_responses(...)`.
- [x] Migrated queue/late/burst contention paths to helper response readers.

### M38-S165 HTTP max-concurrency pair contract predicate helper introduction acceptance criteria

- Pair contention success/throttle contract validation is centralized.
- Pair helper validates process success and envelope contracts in one deterministic predicate.
- Queue pair assertions no longer duplicate contract branch logic.

### M38-S165 tracking (live status)

- [x] Added `pair_success_throttle_contract_holds(...)`.
- [x] Migrated queue default + low-timeout assertion paths to helper predicate.

### M38-S166 HTTP max-concurrency security parity pair predicate helper adoption acceptance criteria

- Security-header parity pair assertions are centralized in one deterministic predicate helper.
- Success/throttle envelope and default security header parity checks share one path.
- Security parity contention contract remains unchanged.

### M38-S166 tracking (live status)

- [x] Added `pair_success_throttle_with_security_header_parity_holds(...)`.
- [x] Migrated security parity contention assertion path to helper predicate.

### M38-S167 HTTP max-concurrency oneshot accept-barrier helper introduction acceptance criteria

- Accepted-socket synchronization delay uses one helper instead of raw sleep literals.
- Late/burst contention tests express accept-barrier intent consistently.
- Accept-barrier timing contract remains unchanged.

### M38-S167 tracking (live status)

- [x] Added `wait_for_oneshot_accept_barrier(...)`.
- [x] Migrated late + burst contention tests to helper barrier call.

### M38-S168 HTTP max-concurrency late-backlog staging helper adoption acceptance criteria

- Late-connection backlog request/noise staging is centralized in one helper.
- Default and low-timeout late contention tests share one staging path.
- Ordered late contention contract remains unchanged.

### M38-S168 tracking (live status)

- [x] Added `stage_late_backlog_requests(...)`.
- [x] Migrated late default + low-timeout backlog staging to helper.

### M38-S169 HTTP max-concurrency burst-backlog staging helper adoption acceptance criteria

- Burst-ingress backlog request/noise staging is centralized in one helper.
- Default and low-timeout burst contention tests share one staging path.
- Ordered burst contention contract remains unchanged.

### M38-S169 tracking (live status)

- [x] Added `stage_burst_backlog_requests(...)`.
- [x] Migrated burst default + low-timeout backlog staging to helper.

### M38-S170 HTTP max-concurrency ordered-trace predicate helper adoption acceptance criteria

- Late and burst ordered trace assertions use canonical trace-aware predicate helpers.
- Ordered-trace contract logic is no longer duplicated inline.
- `rt-1 success` and throttle trace sequencing remains deterministic.

### M38-S170 tracking (live status)

- [x] Added `late_ordered_trace_contract_holds(...)`.
- [x] Added `burst_ordered_trace_contract_holds(...)`.
- [x] Migrated late/burst ordered trace assertions to helper predicates.

### M38-S171 HTTP max-concurrency bounded-tail latency helper normalization acceptance criteria

- Bounded-tail latency checks use one helper with explicit millisecond window input.
- Queue/late/burst low-timeout and default bounded-tail branches share one check path.
- Bounded-tail contract behavior remains unchanged.

### M38-S171 tracking (live status)

- [x] Added `has_bounded_tail_latency(...)`.
- [x] Migrated bounded-tail assertions across queue/late/burst contention paths to helper.
- [x] Revalidated helper-layer contract suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S172 HTTP max-concurrency staged-connection helper foundation acceptance criteria

- Late/burst contention setup paths share one staged-connection helper foundation.
- Helper foundation keeps connection-retry, timeout wiring, and failure envelopes deterministic.
- Existing contention behavior remains unchanged.

### M38-S172 tracking (live status)

- [x] Added staged-connection helper foundation:
  - `connect_staged_stream_or_terminate(...)`
- [x] Revalidated max-concurrency suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`

### M38-S173 HTTP max-concurrency staged first-connection helper introduction acceptance criteria

- First staged connection setup in late/burst paths uses a dedicated helper.
- First-connection retry budget and diagnostics are centralized.
- First-connection contract behavior remains unchanged.

### M38-S173 tracking (live status)

- [x] Added `connect_staged_first_stream_or_terminate(...)`.
- [x] Migrated late/burst first-connection setup to helper.

### M38-S174 HTTP max-concurrency staged second-connection helper introduction acceptance criteria

- Second staged connection setup in late/burst paths uses a dedicated helper.
- Second-connection retry budget and diagnostics are centralized.
- Second-connection contract behavior remains unchanged.

### M38-S174 tracking (live status)

- [x] Added `connect_staged_second_stream_or_terminate(...)`.
- [x] Migrated late/burst second-connection setup to helper.

### M38-S175 HTTP max-concurrency staged third-connection helper introduction acceptance criteria

- Third staged connection setup in burst paths uses a dedicated helper.
- Third-connection retry budget and diagnostics are centralized.
- Third-connection contract behavior remains unchanged.

### M38-S175 tracking (live status)

- [x] Added `connect_staged_third_stream_or_terminate(...)`.
- [x] Migrated burst third-connection setup to helper.

### M38-S176 HTTP max-concurrency late staged-connection helper adoption acceptance criteria

- Late-connection default and low-timeout paths use staged first/second connection helpers.
- Late staged setup no longer duplicates retry + read-timeout wiring inline.
- Ordered late contention contracts remain deterministic.

### M38-S176 tracking (live status)

- [x] Migrated late default contention setup to staged connection helpers.
- [x] Migrated late low-timeout contention setup to staged connection helpers.

### M38-S177 HTTP max-concurrency burst staged-connection helper adoption acceptance criteria

- Burst-ingress default and low-timeout paths use staged first/second/third connection helpers.
- Burst staged setup no longer duplicates retry + read-timeout wiring inline.
- Ordered burst contention contracts remain deterministic.

### M38-S177 tracking (live status)

- [x] Migrated burst default contention setup to staged connection helpers.
- [x] Migrated burst low-timeout contention setup to staged connection helpers.

### M38-S178 HTTP max-concurrency staged-attempt spawn helper introduction acceptance criteria

- Late/burst contention paths use one helper that returns attempt start time, port, and spawned child.
- Attempt bootstrap no longer duplicates `Instant::now + find port + spawn` blocks inline.
- Spawn contract behavior remains unchanged.

### M38-S178 tracking (live status)

- [x] Added `spawn_staged_contention_attempt(...)`.
- [x] Helper preserves deterministic oneshot spawn wiring.

### M38-S179 HTTP max-concurrency late staged-attempt spawn helper adoption acceptance criteria

- Late default and low-timeout contention paths use the staged-attempt spawn helper.
- Late attempt bootstrap duplication is removed.
- Late contention contracts remain deterministic.

### M38-S179 tracking (live status)

- [x] Migrated late default attempt bootstrap to helper.
- [x] Migrated late low-timeout attempt bootstrap to helper.

### M38-S180 HTTP max-concurrency burst staged-attempt spawn helper adoption acceptance criteria

- Burst default and low-timeout contention paths use the staged-attempt spawn helper.
- Burst attempt bootstrap duplication is removed.
- Burst contention contracts remain deterministic.

### M38-S180 tracking (live status)

- [x] Migrated burst default attempt bootstrap to helper.
- [x] Migrated burst low-timeout attempt bootstrap to helper.

### M38-S181 HTTP max-concurrency staged-connection helper consolidation revalidation acceptance criteria

- Consolidated staged-connection helper layer keeps all max-concurrency contention contracts green.
- No behavioral regression in queue, security, late, or burst branches after staged helper adoption.
- Roadmap/book tracking is refreshed for `M38-S172..M38-S181`.

### M38-S181 tracking (live status)

- [x] Revalidated helper-layer contract suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- [x] Published book chapters `799..808`.

### M38-S182 HTTP max-concurrency queue/security pair-attempt bootstrap helper normalization acceptance criteria

- Queue/security contention tests share normalized pair-attempt bootstrap helper paths.
- Pair-attempt bootstrap helper adoption removes repeated attempt setup boilerplate.
- Queue/security contention behavior remains unchanged.

### M38-S182 tracking (live status)

- [x] Added normalized pair-attempt bootstrap helper coverage for queue/security branches.
- [x] Revalidated queue/security contention contracts after helper adoption.

### M38-S183 HTTP max-concurrency pair-attempt spawn helper introduction acceptance criteria

- Pair contention branches use a dedicated attempt spawn helper.
- Attempt helper centralizes `attempt_started + port + oneshot spawn` bootstrap.
- Spawn diagnostics remain case-specific.

### M38-S183 tracking (live status)

- [x] Added `spawn_pair_contention_attempt(...)`.
- [x] Helper reuses staged contention bootstrap semantics.

### M38-S184 HTTP max-concurrency queue default pair-attempt spawn helper adoption acceptance criteria

- Queue default contention path uses pair-attempt spawn helper.
- Queue default test no longer duplicates attempt bootstrap.
- Queue default deterministic contract remains unchanged.

### M38-S184 tracking (live status)

- [x] Migrated queue default attempt bootstrap to `spawn_pair_contention_attempt(...)`.
- [x] Preserved one-success/one-throttle queue default contract.

### M38-S185 HTTP max-concurrency queue low-timeout pair-attempt spawn helper adoption acceptance criteria

- Queue low-timeout contention path uses pair-attempt spawn helper.
- Queue low-timeout test no longer duplicates attempt bootstrap.
- Queue low-timeout bounded-tail contract remains unchanged.

### M38-S185 tracking (live status)

- [x] Migrated queue low-timeout attempt bootstrap to `spawn_pair_contention_attempt(...)`.
- [x] Preserved queue low-timeout bounded-tail contract.

### M38-S186 HTTP max-concurrency security parity pair-attempt spawn helper adoption acceptance criteria

- Security parity contention path uses pair-attempt spawn helper.
- Security parity test no longer duplicates attempt bootstrap.
- Security parity contract remains unchanged.

### M38-S186 tracking (live status)

- [x] Migrated security parity attempt bootstrap to `spawn_pair_contention_attempt(...)`.
- [x] Preserved deterministic success/throttle security parity contract.

### M38-S187 HTTP max-concurrency pair stream connect-timeout helper introduction acceptance criteria

- Pair contention branches use one helper for pair connect + stream timeout setup.
- Pair helper centralizes first/second connection failure envelopes.
- Connect-timeout behavior remains deterministic.

### M38-S187 tracking (live status)

- [x] Added `connect_pair_streams_or_terminate(...)`.
- [x] Helper centralizes pair connect retry envelope + timeout wiring.

### M38-S188 HTTP max-concurrency queue pair-connect helper adoption acceptance criteria

- Queue default and low-timeout branches use pair connect-timeout helper.
- Queue branches no longer duplicate pair connect + timeout setup.
- Queue contracts remain deterministic.

### M38-S188 tracking (live status)

- [x] Migrated queue default pair connect setup to helper.
- [x] Migrated queue low-timeout pair connect setup to helper.

### M38-S189 HTTP max-concurrency security parity pair-connect helper adoption acceptance criteria

- Security parity branch uses pair connect-timeout helper.
- Security parity branch no longer duplicates pair connect + timeout setup.
- Security parity contract remains deterministic.

### M38-S189 tracking (live status)

- [x] Migrated security parity pair connect setup to helper.
- [x] Preserved deterministic security parity contention contract.

### M38-S190 HTTP max-concurrency pair request-exchange helper introduction acceptance criteria

- Pair contention branches use one helper for paired request writes and response collection.
- Helper centralizes request exchange envelope in queue/security branches.
- Request exchange behavior remains deterministic.

### M38-S190 tracking (live status)

- [x] Added `exchange_pair_http_requests_and_collect(...)`.
- [x] Migrated queue/security pair request exchange to helper.

### M38-S191 HTTP max-concurrency queue/security pair-helper consolidation revalidation acceptance criteria

- Queue/security pair helper consolidation keeps contention contracts green.
- No regression in queue default, queue low-timeout, or security parity branches.
- Roadmap/book tracking is refreshed for `M38-S182..M38-S191`.

### M38-S191 tracking (live status)

- [x] Revalidated helper-layer contract suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- [x] Published book chapters `809..818`.

### M38-S192 HTTP max-concurrency queue/security pair-attempt outcome helper normalization acceptance criteria

- Queue/security contention branches share normalized pair-attempt outcome helper paths.
- Outcome helper adoption removes repeated status/response handling boilerplate.
- Queue/security contention behavior remains unchanged.

### M38-S192 tracking (live status)

- [x] Added normalized pair-attempt outcome helper coverage for queue/security branches.
- [x] Revalidated queue/security contention contracts after helper adoption.

### M38-S193 HTTP max-concurrency pair-attempt outcome envelope introduction acceptance criteria

- Pair attempt outcome uses a canonical envelope type.
- Envelope stores process status plus first/second responses for contract evaluation.
- Outcome envelope semantics remain deterministic.

### M38-S193 tracking (live status)

- [x] Added `PairAttemptOutcome`.
- [x] Added canonical outcome contract alias `PairOutcomeContract`.

### M38-S194 HTTP max-concurrency pair-attempt outcome collector helper introduction acceptance criteria

- Pair attempts use one helper to exchange requests and collect status/responses.
- Outcome collector centralizes child wait timeout behavior with case-specific diagnostics.
- Outcome collection behavior remains deterministic.

### M38-S194 tracking (live status)

- [x] Added `collect_pair_attempt_outcome(...)`.
- [x] Migrated queue/security pair branches to outcome collector helper.

### M38-S195 HTTP max-concurrency pair outcome contract-dispatch helper introduction acceptance criteria

- Pair outcome contract evaluation uses one contract-dispatch helper.
- Queue/security branches no longer invoke contract functions with inline response tuple plumbing.
- Contract-dispatch behavior remains deterministic.

### M38-S195 tracking (live status)

- [x] Added `pair_outcome_matches_contract(...)`.
- [x] Migrated queue default/security parity contract checks to helper.

### M38-S196 HTTP max-concurrency pair outcome bounded-tail contract helper introduction acceptance criteria

- Pair outcome plus bounded-tail checks use one helper.
- Queue low-timeout branch no longer duplicates bounded-tail conjunction logic.
- Bounded-tail behavior remains deterministic.

### M38-S196 tracking (live status)

- [x] Added `pair_outcome_matches_contract_with_bounded_tail(...)`.
- [x] Migrated queue low-timeout contract path to helper.

### M38-S197 HTTP max-concurrency pair outcome observation formatter helper introduction acceptance criteria

- Pair attempt observation rendering uses one helper path.
- Queue/security branches no longer format observation strings with direct tuple plumbing.
- Observation envelopes remain deterministic.

### M38-S197 tracking (live status)

- [x] Added `format_pair_outcome_observation(...)`.
- [x] Migrated queue/security observation updates to helper.

### M38-S198 HTTP max-concurrency queue default outcome helper adoption acceptance criteria

- Queue default branch uses canonical pair outcome helpers for collection/contract/observation.
- Queue default branch no longer duplicates outcome handling boilerplate.
- Queue default deterministic contract remains unchanged.

### M38-S198 tracking (live status)

- [x] Migrated queue default outcome handling to pair outcome helpers.
- [x] Preserved queue default one-success/one-throttle contract.

### M38-S199 HTTP max-concurrency queue low-timeout outcome helper adoption acceptance criteria

- Queue low-timeout branch uses canonical pair outcome helpers for collection/contract/observation.
- Queue low-timeout branch no longer duplicates outcome+bounded-tail boilerplate.
- Queue low-timeout deterministic contract remains unchanged.

### M38-S199 tracking (live status)

- [x] Migrated queue low-timeout outcome handling to pair outcome helpers.
- [x] Preserved queue low-timeout bounded-tail contract.

### M38-S200 HTTP max-concurrency security parity outcome helper adoption acceptance criteria

- Security parity branch uses canonical pair outcome helpers for collection/contract/observation.
- Security parity branch no longer duplicates outcome handling boilerplate.
- Security parity deterministic contract remains unchanged.

### M38-S200 tracking (live status)

- [x] Migrated security parity outcome handling to pair outcome helpers.
- [x] Preserved security parity contract behavior.

### M38-S201 HTTP max-concurrency pair-outcome helper consolidation revalidation acceptance criteria

- Queue/security pair-outcome helper consolidation keeps contention contracts green.
- No regression in queue default, queue low-timeout, or security parity branches.
- Roadmap/book tracking is refreshed for `M38-S192..M38-S201`.

### M38-S201 tracking (live status)

- [x] Revalidated helper-layer contract suite:
  - `cargo test -p sec4 --test json_output c_bin_http_runtime_max_concurrency_`
- [x] Published book chapters `819..828`.

### M38-S202 HTTP max-concurrency queue/security pair-attempt assertion-loop helper normalization acceptance criteria

- Queue/security contention tests share one pair-attempt assertion loop helper path.
- Pair assertion loop helper centralizes per-attempt spawn/connect/collect/contract envelope logic.
- Queue/security contention behavior remains deterministic.

### M38-S202 tracking (live status)

- [x] Added `run_pair_contention_attempt_loop(...)`.
- [x] Migrated queue/security pair-attempt assertions to normalized helper loop.

### M38-S203 HTTP max-concurrency ordered pair-attempt outcome envelope introduction acceptance criteria

- Late staged contention path uses canonical ordered pair-attempt outcome envelope type.
- Ordered pair envelope stores status plus first/second responses for contract checks.
- Ordered pair outcome behavior remains deterministic.

### M38-S203 tracking (live status)

- [x] Added `OrderedPairAttemptOutcome`.
- [x] Added ordered pair contract alias `OrderedPairContract`.

### M38-S204 HTTP max-concurrency ordered triple-attempt outcome envelope introduction acceptance criteria

- Burst staged contention path uses canonical ordered triple-attempt outcome envelope type.
- Ordered triple envelope stores status plus first/second/third responses for contract checks.
- Ordered triple outcome behavior remains deterministic.

### M38-S204 tracking (live status)

- [x] Added `OrderedTripleAttemptOutcome`.
- [x] Added ordered triple contract alias `OrderedTripleContract`.

### M38-S205 HTTP max-concurrency late ordered pair outcome collector helper introduction acceptance criteria

- Late staged contention uses one collector helper to stage backlog and collect ordered pair responses.
- Late collector helper centralizes child exit wait envelope for staged pair attempts.
- Late collector behavior remains deterministic.

### M38-S205 tracking (live status)

- [x] Added `collect_late_ordered_pair_attempt_outcome(...)`.
- [x] Late staged pair attempts now share collector helper path.

### M38-S206 HTTP max-concurrency burst ordered triple outcome collector helper introduction acceptance criteria

- Burst staged contention uses one collector helper to stage backlog and collect ordered triple responses.
- Burst collector helper centralizes child exit wait envelope for staged triple attempts.
- Burst collector behavior remains deterministic.

### M38-S206 tracking (live status)

- [x] Added `collect_burst_ordered_triple_attempt_outcome(...)`.
- [x] Burst staged triple attempts now share collector helper path.

### M38-S207 HTTP max-concurrency ordered pair outcome contract helper introduction acceptance criteria

- Ordered pair contract evaluation uses one helper path.
- Ordered pair bounded-tail contract conjunction uses one helper path.
- Late staged pair assertions no longer duplicate inline status/contract conjunction plumbing.

### M38-S207 tracking (live status)

- [x] Added `ordered_pair_outcome_matches_contract(...)`.
- [x] Added `ordered_pair_outcome_matches_contract_with_bounded_tail(...)`.

### M38-S208 HTTP max-concurrency ordered triple outcome contract helper introduction acceptance criteria

- Ordered triple contract evaluation uses one helper path.
- Ordered triple bounded-tail contract conjunction uses one helper path.
- Burst staged triple assertions no longer duplicate inline status/contract conjunction plumbing.

### M38-S208 tracking (live status)

- [x] Added `ordered_triple_outcome_matches_contract(...)`.
- [x] Added `ordered_triple_outcome_matches_contract_with_bounded_tail(...)`.

### M38-S209 HTTP max-concurrency ordered pair outcome observation formatter helper introduction acceptance criteria

- Ordered pair observation rendering uses one helper path.
- Late staged pair assertions no longer format observations with inline response tuple plumbing.
- Ordered pair observation envelopes remain deterministic.

### M38-S209 tracking (live status)

- [x] Added `format_ordered_pair_outcome_observation(...)`.
- [x] Late staged pair observation updates now use helper formatter.

### M38-S210 HTTP max-concurrency ordered triple outcome observation formatter helper introduction acceptance criteria

- Ordered triple observation rendering uses one helper path.
- Burst staged triple assertions no longer format observations with inline response tuple plumbing.
- Ordered triple observation envelopes remain deterministic.

### M38-S210 tracking (live status)

- [x] Added `format_ordered_triple_outcome_observation(...)`.
- [x] Burst staged triple observation updates now use helper formatter.

### M38-S211 HTTP max-concurrency late/burst assertion-loop helper adoption consolidation revalidation acceptance criteria

- Late/burst staged contention branches use normalized ordered assertion-loop helpers.
- No regression in late default, late low-timeout, burst default, or burst low-timeout branches.
- Roadmap/book tracking is refreshed for `M38-S202..M38-S211`.

### M38-S211 tracking (live status)

- [x] Added `run_late_contention_attempt_loop(...)` and `run_burst_contention_attempt_loop(...)`.
- [x] Migrated late/burst staged assertions to ordered loop helpers and revalidated contracts.

### M38-S212 HTTP max-concurrency ordered-attempt helper branch-scope contract matrix expansion acceptance criteria

- Ordered queue/late/burst branch assertion arguments are centralized through explicit branch-case structs.
- Queue boundary, security parity, late, and burst branches call canonical branch-case assertion helpers instead of inline loop invocation blocks.
- Deterministic failure envelopes (`last_observation` diagnostics) remain unchanged across all migrated branches.

### M38-S212 tracking (live status)

- [x] Added `PairContentionBranchCase`, `LateContentionBranchCase`, and `BurstContentionBranchCase`.
- [x] Added `assert_pair_contention_branch_case(...)`, `assert_late_contention_branch_case(...)`, and `assert_burst_contention_branch_case(...)`.
- [x] Migrated seven ordered max-concurrency branch assertions to branch-scope matrix helper calls.

### M38-S213 HTTP max-concurrency ordered-attempt helper branch-case fixture-constructor normalization acceptance criteria

- Branch-case fixture construction for pair/late/burst paths uses canonical constructor helpers.
- Queue boundary, security parity, late, and burst branches no longer inline branch-case field wiring.
- Ordered helper failure contracts remain deterministic across all constructor-backed branches.

### M38-S213 tracking (live status)

- [x] Added `pair_contention_branch_case(...)`, `late_contention_branch_case(...)`, and `burst_contention_branch_case(...)`.
- [x] Migrated seven ordered branch assertions to constructor-backed branch-case fixtures.

### M38-S214 HTTP max-concurrency ordered-attempt helper fixture-catalog dispatch normalization acceptance criteria

- Ordered branch fixtures are selected through canonical fixture catalogs instead of per-test literal constructor arguments.
- Queue/security/late/burst branches dispatch by explicit fixture identifiers.
- Deterministic contract diagnostics and assertion semantics remain unchanged.

### M38-S214 tracking (live status)

- [x] Added fixture enums: `PairContentionBranchFixture`, `LateContentionBranchFixture`, `BurstContentionBranchFixture`.
- [x] Added fixture dispatch helpers: `pair_contention_branch_fixture(...)`, `late_contention_branch_fixture(...)`, `burst_contention_branch_fixture(...)`.
- [x] Migrated seven ordered branch assertions to fixture-catalog dispatch calls.

### M38-S215 HTTP max-concurrency ordered-attempt fixture catalog assertion-pack revalidation acceptance criteria

- Fixture-catalog dispatch is consumed through one-step assertion-pack helpers for pair/late/burst paths.
- Ordered queue/security/late/burst tests no longer nest fixture lookup inside assertion call sites.
- Contract behavior and deterministic failure envelopes remain unchanged.

### M38-S215 tracking (live status)

- [x] Added assertion-pack helpers: `assert_pair_contention_fixture(...)`, `assert_late_contention_fixture(...)`, `assert_burst_contention_fixture(...)`.
- [x] Migrated seven ordered branch assertions to one-step fixture assertion-pack calls.

### M38-S216 HTTP max-concurrency ordered-attempt fixture-catalog grouped runner normalization acceptance criteria

- Pair/late/burst fixture execution paths use canonical grouped runner helpers.
- Shared HTTP route fixture source strings are centralized instead of repeated inline test fixtures.
- Ordered branch assertions keep deterministic behavior while test-local build/assert boilerplate is reduced.

### M38-S216 tracking (live status)

- [x] Added grouped runner helpers: `run_pair_contention_fixture_case(...)`, `run_late_contention_fixture_case(...)`, `run_burst_contention_fixture_case(...)`.
- [x] Added shared fixture source constants for simple router and security-header router variants.
- [x] Migrated seven ordered branch tests to grouped runner helper calls.

### M38-S217 HTTP max-concurrency ordered-attempt fixture descriptor catalog normalization acceptance criteria

- Fixture name/module/label/source selection is centralized behind descriptor catalogs.
- Pair/late/burst grouped runner call sites dispatch by descriptor enums instead of literal fixture metadata.
- Deterministic contention contracts remain unchanged after descriptor-catalog adoption.

### M38-S217 tracking (live status)

- [x] Added descriptor enums: `PairContentionFixtureDescriptor`, `LateContentionFixtureDescriptor`, `BurstContentionFixtureDescriptor`.
- [x] Added descriptor catalog helpers for pair/late/burst grouped runners.
- [x] Migrated seven ordered branch tests to descriptor-driven grouped runner calls.

### M38-S218 HTTP max-concurrency ordered-attempt descriptor-runner assertion-pack unification acceptance criteria

- Ordered pair/late/burst descriptor execution is reachable through one canonical dispatcher helper.
- Queue/security/late/burst test call sites invoke one unified ordered descriptor runner path.
- Deterministic contention assertions and diagnostics remain unchanged after dispatcher unification.

### M38-S218 tracking (live status)

- [x] Added `OrderedContentionFixtureDescriptor` dispatcher enum.
- [x] Added `run_ordered_contention_fixture_descriptor_case(...)` unified dispatcher helper.
- [x] Migrated seven ordered branch tests to the unified descriptor-runner assertion-pack path.

### M38-S219 HTTP max-concurrency ordered-attempt descriptor metadata table extraction acceptance criteria

- Pair/late/burst descriptor metadata is backed by canonical static metadata tables.
- Descriptor lookup helpers resolve metadata through deterministic table indexing rather than repeated match tuple literals.
- Ordered descriptor-runner behavior and assertion contracts remain unchanged.

### M38-S219 tracking (live status)

- [x] Added pair/late/burst descriptor metadata structs and static metadata tables.
- [x] Added deterministic descriptor index mapping helpers (`as_index`) for table lookup.
- [x] Migrated descriptor lookup helpers to metadata-table extraction paths.

### M38-S220 HTTP max-concurrency ordered-attempt descriptor table-runner envelope simplification acceptance criteria

- Ordered descriptor execution resolves to one runner metadata envelope before fixture build/assertion.
- Pair/late/burst runner branches share one canonical execute path.
- Deterministic contention contracts remain unchanged after envelope unification.

### M38-S220 tracking (live status)

- [x] Added `OrderedContentionRunnerMetadata` and `OrderedContentionBranchFixture`.
- [x] Added `ordered_contention_runner_metadata(...)` envelope resolver.
- [x] Unified ordered fixture execution into one `run_ordered_contention_fixture_descriptor_case(...)` path.

### M38-S221 HTTP max-concurrency ordered-attempt descriptor contract catalog sanity coverage acceptance criteria

- Descriptor catalog contract invariants are verified by explicit sanity coverage tests.
- Pair/late/burst descriptor metadata expectations (fixture id, module, label, source, branch mapping) are asserted deterministically.
- Sanity coverage is independent of clang/runtime availability.

### M38-S221 tracking (live status)

- [x] Added `assert_ordered_contention_runner_metadata(...)`.
- [x] Added `c_bin_http_runtime_max_concurrency_ordered_descriptor_catalog_sanity_coverage`.
- [x] Revalidated ordered queue/late/burst contention contract pack after sanity coverage wiring.

### M38-S222 HTTP max-concurrency ordered-attempt descriptor catalog assertion-table compaction acceptance criteria

- Descriptor catalog sanity expectations are expressed as one canonical assertion table.
- Sanity coverage iterates the assertion table instead of repeating seven manual assertion blocks.
- Pair/late/burst descriptor contract checks remain unchanged.

### M38-S222 tracking (live status)

- [x] Added `OrderedContentionRunnerMetadataExpectation`.
- [x] Added `ORDERED_CONTENTION_RUNNER_METADATA_EXPECTATIONS` assertion table.
- [x] Migrated descriptor catalog sanity test to table-driven iteration.

### M38-S223 HTTP max-concurrency ordered-attempt descriptor table-driven fixture smoke harness compaction acceptance criteria

- Ordered descriptor smoke coverage is executed from one canonical descriptor table instead of seven repeated fixture test bodies.
- Pair/late/burst fixture smoke assertions continue to run unchanged through the existing ordered descriptor runner path.
- Clang-gated ordered contention smoke validation remains deterministic after harness compaction.

### M38-S223 tracking (live status)

- [x] Added `ORDERED_CONTENTION_FIXTURE_SMOKE_DESCRIPTORS`.
- [x] Added `c_bin_http_runtime_max_concurrency_ordered_descriptor_fixture_smoke_harness_when_clang_available`.
- [x] Removed repeated per-descriptor smoke test bodies in favor of table-driven iteration.

### M38-S224 HTTP max-concurrency ordered-attempt descriptor contract/smoke dual-table unification acceptance criteria

- Ordered descriptor smoke harness reuses the canonical descriptor contract expectation catalog directly.
- Separate smoke-only descriptor table is removed to eliminate drift between smoke and sanity coverage inputs.
- Pair/late/burst smoke behavior remains unchanged after table unification.

### M38-S224 tracking (live status)

- [x] Removed `ORDERED_CONTENTION_FIXTURE_SMOKE_DESCRIPTORS`.
- [x] Migrated ordered smoke harness iteration to `ORDERED_CONTENTION_RUNNER_METADATA_EXPECTATIONS`.
- [x] Revalidated descriptor sanity + smoke harness coverage after table unification.

### M38-S225 HTTP max-concurrency ordered-attempt descriptor contract/smoke iterator-helper extraction acceptance criteria

- Ordered descriptor expectation traversal is centralized behind shared iterator helpers.
- Sanity and smoke harness tests consume the shared iterator helpers instead of manual `for` loops.
- Descriptor contract and smoke behavior remain deterministic after iterator-helper extraction.

### M38-S225 tracking (live status)

- [x] Added `for_each_ordered_contention_runner_metadata_expectation(...)`.
- [x] Added shared fixture-descriptor traversal path derived from runner metadata expectations.
- [x] Migrated sanity + smoke harness tests to iterator-helper traversal.

### M38-S226 HTTP max-concurrency ordered-attempt descriptor iterator-helper failure-context enrichment acceptance criteria

- Ordered descriptor smoke harness failures include deterministic case context (label/module/fixture) for table-driven traversal diagnostics.
- Panic payload extraction is normalized so wrapped smoke failures preserve useful original panic details.
- Clang-gated ordered smoke harness behavior remains unchanged except enriched failure diagnostics.

### M38-S226 tracking (live status)

- [x] Added `panic_payload_message(...)` normalization helper for wrapped panic payloads.
- [x] Added `run_ordered_contention_smoke_expectation_with_failure_context(...)`.
- [x] Migrated ordered smoke harness loop to failure-context wrapped expectation execution.

### M38-S227 HTTP max-concurrency ordered-attempt descriptor smoke harness case-banner tracing acceptance criteria

- Ordered descriptor smoke harness prints deterministic per-case banners before running each table-driven fixture.
- Case-banner format includes case label, module name, and fixture name for direct correlation with failure context.
- Existing failure-context enrichment behavior remains unchanged after banner tracing is added.

### M38-S227 tracking (live status)

- [x] Added `ordered_contention_smoke_case_banner(...)`.
- [x] Added per-expectation case-banner tracing in smoke harness execution.
- [x] Reused case-banner content in wrapped failure diagnostics.

### M38-S228 HTTP max-concurrency ordered-attempt descriptor smoke harness banner-contract coverage acceptance criteria

- Ordered descriptor case-banner rendering contract is covered by a dedicated deterministic test.
- Banner contract assertions run without clang/runtime dependencies.
- Case-banner shape (`case=... module=... fixture=...`) remains synchronized with smoke harness tracing/failure output.

### M38-S228 tracking (live status)

- [x] Added `c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_case_banner_contract_coverage`.
- [x] Added deterministic exact banner-shape assertions for every ordered descriptor expectation.
- [x] Revalidated sanity + banner-contract + smoke harness coverage pack.

### M38-S229 HTTP max-concurrency ordered-attempt descriptor smoke banner-prefix constant extraction acceptance criteria

- Smoke banner prefix literal is extracted into one canonical constant used by banner rendering and banner-contract coverage assertions.
- Banner output contract remains unchanged after constant extraction.
- Table-driven smoke harness logging/failure context continues to emit deterministic banner strings.

### M38-S229 tracking (live status)

- [x] Added `ORDERED_CONTENTION_SMOKE_BANNER_PREFIX`.
- [x] Migrated `ordered_contention_smoke_case_banner(...)` to use the shared banner-prefix constant.
- [x] Migrated banner-contract coverage expected strings to use the shared banner-prefix constant.

### M38-S230 HTTP max-concurrency ordered-attempt descriptor smoke banner formatter helper contract extraction acceptance criteria

- Ordered descriptor smoke banner rendering logic is centralized in a dedicated formatter helper.
- Banner contract coverage validates the dedicated formatter helper output and case-banner wrapper parity.
- Smoke harness tracing/failure output keeps the same deterministic banner shape after helper extraction.

### M38-S230 tracking (live status)

- [x] Added `ordered_contention_smoke_banner_formatter(...)`.
- [x] Migrated `ordered_contention_smoke_case_banner(...)` to delegate to formatter helper.
- [x] Extended banner-contract coverage to assert formatter output plus wrapper parity.

### M38-S231 HTTP max-concurrency ordered-attempt descriptor smoke failure-banner formatter extraction acceptance criteria

- Ordered descriptor smoke failure message rendering is centralized in a dedicated formatter helper.
- Wrapped smoke failure-path rendering from panic payloads delegates to the same formatter helper.
- Dedicated deterministic contract coverage validates both direct failure-banner formatting and payload-derived failure-banner formatting.

### M38-S231 tracking (live status)

- [x] Added `ordered_contention_smoke_failure_banner(...)`.
- [x] Added `ordered_contention_smoke_failure_banner_from_payload(...)`.
- [x] Added `c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_failure_banner_contract_coverage`.

### M38-S232 HTTP max-concurrency ordered-attempt descriptor smoke banner field-key constant extraction acceptance criteria

- Ordered descriptor smoke banner field keys (`case`, `module`, `fixture`) are extracted into canonical constants.
- Banner formatter and banner-contract coverage expected rendering both use shared field-key constants.
- Banner output contract remains unchanged after field-key constant extraction.

### M38-S232 tracking (live status)

- [x] Added `ORDERED_CONTENTION_SMOKE_BANNER_FIELD_CASE`.
- [x] Added `ORDERED_CONTENTION_SMOKE_BANNER_FIELD_MODULE`.
- [x] Added `ORDERED_CONTENTION_SMOKE_BANNER_FIELD_FIXTURE`.

### M38-S233 HTTP max-concurrency ordered-attempt descriptor smoke failure-suffix constant extraction acceptance criteria

- Ordered descriptor smoke failure suffix token (`failed:`) is extracted into one canonical constant.
- Failure-banner formatter and failure-banner contract expected rendering both use the shared failure-suffix constant.
- Failure-banner output contract remains unchanged after suffix constant extraction.

### M38-S233 tracking (live status)

- [x] Added `ORDERED_CONTENTION_SMOKE_FAILURE_SUFFIX`.
- [x] Migrated `ordered_contention_smoke_failure_banner(...)` to use the shared failure-suffix constant.
- [x] Migrated failure-banner contract coverage expected rendering to the shared failure-suffix constant.

### M38-S234 HTTP max-concurrency ordered-attempt descriptor smoke failure-suffix contract helper extraction acceptance criteria

- Failure-banner contract expected rendering is centralized in a dedicated helper.
- Failure-banner formatter delegates to the same contract helper to keep output expectations and runtime path synchronized.
- Failure-banner contract coverage uses the dedicated helper instead of in-test expected string assembly.

### M38-S234 tracking (live status)

- [x] Added `ordered_contention_smoke_failure_banner_contract_expected(...)`.
- [x] Migrated `ordered_contention_smoke_failure_banner(...)` to delegate to the contract helper.
- [x] Migrated failure-banner contract test expected rendering to helper-based assembly.

### M38-S235 HTTP max-concurrency ordered-attempt descriptor smoke banner-contract assertion helper extraction acceptance criteria

- Ordered descriptor case-banner contract assertions are centralized in a dedicated helper.
- Ordered descriptor failure-banner contract assertions are centralized in a dedicated helper.
- Case/failure banner contract coverage tests run table iteration via assertion helpers rather than in-test duplicated assertion blocks.

### M38-S235 tracking (live status)

- [x] Added `assert_ordered_contention_smoke_case_banner_contract_expectation(...)`.
- [x] Added `assert_ordered_contention_smoke_failure_banner_contract_expectation(...)`.
- [x] Migrated case/failure banner contract coverage tests to helper-based assertion dispatch.

### M38-S236 HTTP max-concurrency ordered-attempt descriptor smoke contract helper naming parity cleanup acceptance criteria

- Ordered descriptor smoke contract helper names follow consistent expectation-oriented naming (`*_contract_expectation`).
- Case and failure contract coverage tests dispatch helpers with naming parity.
- Banner contract behavior remains unchanged after helper naming cleanup.

### M38-S236 tracking (live status)

- [x] Renamed case-banner contract helper to `assert_ordered_contention_smoke_case_banner_contract_expectation(...)`.
- [x] Renamed failure-banner contract helper to `assert_ordered_contention_smoke_failure_banner_contract_expectation(...)`.
- [x] Updated contract coverage test dispatchers to the renamed parity helpers.

### M38-S237 HTTP max-concurrency ordered-attempt descriptor smoke contract helper dispatch consolidator extraction acceptance criteria

- Ordered descriptor smoke contract coverage tests use one shared assertion-dispatch helper for expectation iteration.
- Case and failure banner contract coverage tests dispatch their expectation-specific assertion helpers through the shared dispatcher.
- Contract behavior remains unchanged after dispatch-consolidator extraction.

### M38-S237 tracking (live status)

- [x] Added `run_ordered_contention_smoke_contract_assertion_dispatch(...)`.
- [x] Migrated case-banner contract coverage test to the shared contract assertion dispatcher.
- [x] Migrated failure-banner contract coverage test to the shared contract assertion dispatcher.

### M38-S238 HTTP max-concurrency ordered-attempt descriptor smoke contract assertion matrix helper extraction acceptance criteria

- Ordered descriptor smoke contract assertion cases are represented by a canonical matrix helper with case-label + assertion function binding.
- Contract coverage tests resolve and execute matrix cases through dedicated matrix-case helpers.
- Contract behavior remains unchanged after assertion-matrix helper extraction.

### M38-S238 tracking (live status)

- [x] Added `OrderedContentionSmokeContractAssertionMatrixCase`.
- [x] Added `ORDERED_CONTENTION_SMOKE_CONTRACT_ASSERTION_MATRIX` + case-label resolver helper.
- [x] Migrated case/failure banner contract coverage tests to matrix-case helper dispatch.

### M38-S239 HTTP max-concurrency ordered-attempt descriptor smoke contract assertion matrix case-label constant extraction acceptance criteria

- Assertion-matrix unknown-case panic prefix is extracted into a dedicated constant.
- Matrix case-label contract coverage validates both known case-label resolution and unknown-case panic message contract.
- Matrix case-label resolver contract remains deterministic after constant extraction.

### M38-S239 tracking (live status)

- [x] Added `ORDERED_CONTENTION_SMOKE_CONTRACT_ASSERTION_UNKNOWN_MATRIX_CASE_LABEL_PREFIX`.
- [x] Added `c_bin_http_runtime_max_concurrency_ordered_descriptor_smoke_contract_assertion_matrix_case_label_contract_coverage`.
- [x] Migrated unknown-case panic rendering to shared prefix constant output.

### M38-S240 HTTP max-concurrency ordered-attempt descriptor smoke contract assertion matrix case-label helper extraction acceptance criteria

- Matrix case-label assertion/dispatch behavior is centralized in dedicated helper functions.
- Case/failure banner contract coverage tests run matrix case labels through the shared case-label execution helper.
- Matrix case-label contract coverage reuses a shared label-resolution assertion helper for known labels.

### M38-S240 tracking (live status)

- [x] Added `run_ordered_contention_smoke_contract_assertion_matrix_case_label(...)`.
- [x] Added `assert_ordered_contention_smoke_contract_assertion_matrix_case_label_resolution(...)`.
- [x] Migrated case/failure banner contract coverage tests to helper-based case-label dispatch.

### M38-S241 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label assertion helper extraction acceptance criteria

- Unknown matrix case-label panic contract assertions are centralized in a dedicated helper.
- Matrix case-label contract coverage uses the unknown-label assertion helper instead of inline panic assertion blocks.
- Unknown-label panic contract behavior remains unchanged after helper extraction.

### M38-S241 tracking (live status)

- [x] Added `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic(...)`.
- [x] Migrated matrix case-label contract coverage unknown-label assertions to helper dispatch.
- [x] Revalidated matrix case-label unknown-label panic contract after helper extraction.

### M38-S242 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label panic-message helper extraction acceptance criteria

- Unknown matrix case-label panic message rendering is centralized in a dedicated helper.
- Matrix case-label resolver panic path delegates to the panic-message helper.
- Unknown-label panic assertion helper validates exact deterministic message equality from the panic-message helper.

### M38-S242 tracking (live status)

- [x] Added `ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_message(...)`.
- [x] Migrated resolver panic rendering to use the unknown-label panic-message helper.
- [x] Tightened unknown-label panic assertion helper to exact deterministic message equality.

### M38-S243 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix case-label resolver assertion helper extraction acceptance criteria

- Matrix case-label resolver assertions are centralized in a dedicated resolver-contract helper.
- Matrix case-label contract coverage iterates a canonical known-label set and dispatches the resolver-contract helper.
- Unknown-label panic contract coverage remains unchanged and continues to run alongside known-label resolver-contract checks.

### M38-S243 tracking (live status)

- [x] Added `assert_ordered_contention_smoke_contract_assertion_matrix_case_label_resolver_contract(...)`.
- [x] Added `ORDERED_CONTENTION_SMOKE_CONTRACT_ASSERTION_KNOWN_MATRIX_CASE_LABELS`.
- [x] Migrated matrix case-label contract coverage known-label checks to helper-based loop dispatch.

### M38-S244 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label constant extraction acceptance criteria

- Matrix unknown-label sentinel token is extracted into a shared constant.
- Unknown-label panic assertion coverage consumes the shared unknown-label constant.
- Unknown-label panic contract behavior remains unchanged after constant extraction.

### M38-S244 tracking (live status)

- [x] Added `ORDERED_CONTENTION_SMOKE_CONTRACT_ASSERTION_UNKNOWN_MATRIX_CASE_LABEL`.
- [x] Migrated matrix unknown-label contract coverage to shared unknown-label constant usage.
- [x] Revalidated unknown-label panic contract after constant extraction.

### M38-S245 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label contract helper extraction acceptance criteria

- Unknown-label panic contract invocation is centralized behind a dedicated no-arg contract helper.
- Matrix case-label contract coverage dispatches unknown-label assertions through the helper.
- Unknown-label contract assertion behavior remains deterministic after helper extraction.

### M38-S245 tracking (live status)

- [x] Added `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_contract()`.
- [x] Migrated matrix case-label contract coverage unknown-label assertion call to helper dispatch.
- [x] Revalidated unknown-label contract helper path against deterministic panic-message behavior.

### M38-S246 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix case-label contract-coverage consolidator extraction acceptance criteria

- Matrix case-label contract coverage orchestration is centralized in one dedicated helper.
- Known-label resolver checks and unknown-label contract checks are dispatched via the consolidator helper.
- Matrix case-label contract test body delegates fully to consolidator helper execution.

### M38-S246 tracking (live status)

- [x] Added `assert_ordered_contention_smoke_contract_assertion_matrix_case_label_contract_coverage()`.
- [x] Migrated matrix case-label contract coverage test body to consolidator helper delegation.
- [x] Revalidated case-label contract coverage after consolidator extraction.

### M38-S247 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label panic payload assertion helper extraction acceptance criteria

- Unknown-label panic payload capture/assertion flow is centralized in a dedicated helper.
- Unknown-label panic contract assertions delegate payload extraction through the helper.
- Unknown-label panic contract behavior remains unchanged and deterministic after payload helper extraction.

### M38-S247 tracking (live status)

- [x] Added `ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_payload(...)`.
- [x] Migrated unknown-label panic contract assertions to payload helper dispatch.
- [x] Revalidated matrix unknown-label panic behavior after payload helper extraction.

### M38-S248 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label panic message assertion helper extraction acceptance criteria

- Unknown-label panic message equality assertions are centralized in a dedicated helper.
- Unknown-label panic contract assertions delegate message checks through the helper.
- Unknown-label panic message contract remains exact and deterministic after helper extraction.

### M38-S248 tracking (live status)

- [x] Added `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_message(...)`.
- [x] Migrated unknown-label panic assertion flow to message-helper dispatch.
- [x] Revalidated deterministic panic-message equality after helper extraction.

### M38-S249 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label panic contract helper extraction acceptance criteria

- Unknown-label panic contract invocation is centralized behind a dedicated contract helper.
- Unknown-label no-arg contract helper delegates to the panic contract helper with the shared unknown-label constant.
- Matrix case-label contract coverage behavior remains unchanged after unknown-label panic contract helper extraction.

### M38-S249 tracking (live status)

- [x] Added `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_contract(...)`.
- [x] Migrated unknown-label no-arg contract helper to panic-contract helper delegation.
- [x] Revalidated matrix case-label contract coverage after panic-contract helper extraction.

### M38-S250 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label panic-message expected-value helper extraction acceptance criteria

- Unknown-label panic-message expected-value rendering is centralized in a dedicated helper.
- Unknown-label panic-message assertion flow delegates expected-message computation to the helper.
- Panic-message expected-value contract remains deterministic and unchanged after helper extraction.

### M38-S250 tracking (live status)

- [x] Added `ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_expected_panic_message(...)`.
- [x] Migrated unknown-label panic-message assertion flow to expected-message helper dispatch.
- [x] Revalidated deterministic expected panic-message contract behavior after helper extraction.

### M38-S251 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label panic-message actual-value helper extraction acceptance criteria

- Unknown-label panic-message actual-value decoding is centralized in a dedicated helper.
- Unknown-label panic-message assertion flow delegates panic payload decoding to the helper.
- Panic-message actual-value decoding remains deterministic after helper extraction.

### M38-S251 tracking (live status)

- [x] Added `ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_actual_panic_message(...)`.
- [x] Migrated unknown-label panic-message assertion flow to actual-message helper dispatch.
- [x] Revalidated panic payload message decoding behavior after helper extraction.

### M38-S252 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label panic-message equality helper extraction acceptance criteria

- Unknown-label panic-message equality assertion is centralized in a dedicated helper.
- Unknown-label panic-message assertion flow delegates equality checks to the helper.
- Panic-message equality assertion text and deterministic behavior remain unchanged after helper extraction.

### M38-S252 tracking (live status)

- [x] Added `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_message_equality(...)`.
- [x] Migrated unknown-label panic-message assertion helper to equality-helper dispatch.
- [x] Revalidated unknown-label panic-message contract coverage after equality helper extraction.

### M38-S253 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label panic-message contract helper extraction acceptance criteria

- Unknown-label panic-message contract assertions are centralized in a contract-named helper.
- Unknown-label panic contract orchestration delegates panic-message validation through the contract helper.
- Panic-message contract behavior remains deterministic and unchanged after helper extraction.

### M38-S253 tracking (live status)

- [x] Added `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_message_contract(...)`.
- [x] Migrated unknown-label panic contract flow to panic-message contract helper dispatch.
- [x] Revalidated unknown-label panic-message contract behavior after helper extraction.

### M38-S254 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label panic payload contract helper extraction acceptance criteria

- Unknown-label panic payload contract invocation is centralized in a dedicated contract helper.
- Unknown-label panic contract orchestration delegates panic payload capture/assertion through the contract helper.
- Panic payload contract behavior remains deterministic after helper extraction.

### M38-S254 tracking (live status)

- [x] Added `assert_ordered_contention_smoke_contract_assertion_matrix_unknown_case_label_panic_payload_contract(...)`.
- [x] Migrated unknown-label panic contract flow to panic payload contract helper dispatch.
- [x] Revalidated unknown-label panic payload contract behavior after helper extraction.

### M38-S255 HTTP max-concurrency ordered-attempt descriptor smoke contract matrix unknown-label panic orchestration helper simplification acceptance criteria

- Unknown-label panic contract orchestration no longer routes through an intermediate panic assertion helper.
- Unknown-label panic contract helper now dispatches payload and message contract helpers directly.
- Matrix case-label contract coverage behavior remains unchanged after orchestration simplification.

### M38-S255 tracking (live status)

- [x] Removed intermediate unknown-label panic assertion helper from matrix contract orchestration path.
- [x] Migrated unknown-label panic contract helper to direct payload/message contract helper dispatch.
- [x] Revalidated matrix case-label contract coverage after orchestration simplification.

- M17-S1 operator handoff checklist + readiness verifier is now implemented:
  - `docs/book/464-m17-operator-handoff-checklist-and-readiness-verifier.md`
  - `scripts/check-m17-operator-handoff-readiness.sh`
  - `scripts/test-check-m17-operator-handoff-readiness.sh`
- M17-S2 operator bootstrap profile helper is now implemented:
  - `scripts/run-m17-operator-bootstrap.sh`
  - `scripts/test-run-m17-operator-bootstrap.sh`
  - `docs/book/465-m17-operator-bootstrap-profile-helper.md`
- M17-S3 operator troubleshooting matrix is now implemented:
  - `scripts/print-m17-operator-troubleshooting-matrix.sh`
  - `scripts/test-print-m17-operator-troubleshooting-matrix.sh`
  - `docs/book/466-m17-operator-troubleshooting-matrix.md`
- M17-S4 operator handoff quickstart orchestrator is now implemented:
  - `scripts/run-m17-operator-handoff-quickstart.sh`
  - `scripts/test-run-m17-operator-handoff-quickstart.sh`
  - `docs/book/467-m17-operator-handoff-quickstart.md`
- M17-S5 operator handoff CI smoke wrapper is now implemented:
  - `scripts/run-m17-operator-handoff-ci-smoke.sh`
  - `scripts/test-run-m17-operator-handoff-ci-smoke.sh`
  - `docs/book/468-m17-operator-handoff-ci-smoke-wrapper.md`
- M17-S6 operator-handoff workflow contract + CI wiring is now implemented:
  - `.github/workflows/operator-handoff-smoke.yml`
  - `scripts/test-operator-handoff-workflow-contract.sh`
  - `scripts/test-operator-handoff-workflow-contract-guard.sh`
  - `docs/book/469-m17-operator-handoff-workflow-contracts.md`
- M17-S7 operator handoff artifact inspector is now implemented:
  - `scripts/inspect-m17-operator-handoff-artifacts.sh`
  - `scripts/test-inspect-m17-operator-handoff-artifacts.sh`
  - `docs/book/470-m17-operator-handoff-artifact-inspector.md`
- M17-S8 operator readiness summary script is now implemented:
  - `scripts/summarize-m17-operator-handoff-readiness.sh`
  - `scripts/test-summarize-m17-operator-handoff-readiness.sh`
  - `docs/book/471-m17-operator-readiness-summary.md`
- M17-S9 operator release-packet builder is now implemented:
  - `scripts/build-m17-operator-release-packet.sh`
  - `scripts/test-build-m17-operator-release-packet.sh`
  - `docs/book/472-m17-operator-release-packet-builder.md`
- M17-S10 final operator handoff playbook chapter checker is now implemented:
  - `scripts/check-m17-operator-handoff-playbook.sh`
  - `scripts/test-check-m17-operator-handoff-playbook.sh`
  - `docs/book/473-m17-operator-handoff-final-playbook.md`
- M17-S11 clean-clone rehearsal runner is now implemented:
  - `scripts/run-m17-operator-clean-clone-rehearsal.sh`
  - `scripts/test-run-m17-operator-clean-clone-rehearsal.sh`
  - `docs/book/474-m17-operator-clean-clone-rehearsal.md`
- M17-S12 live clean-clone rehearsal note is now documented:
  - `build/operator-clean-clone-rehearsal-live/rehearsal-report.json` (latest local evidence)
  - `docs/book/475-m17-clean-clone-rehearsal-results-note.md`
- M18-S1 kickoff brief generator is now implemented:
  - `scripts/generate-m18-kickoff-brief.sh`
  - `scripts/test-generate-m18-kickoff-brief.sh`
  - `docs/book/476-m18-kickoff-brief-generator.md`
- M18-S2 priority matrix artifact is now implemented:
  - `scripts/build-m18-priority-matrix.sh`
  - `scripts/test-build-m18-priority-matrix.sh`
  - `docs/book/477-m18-priority-matrix-artifact.md`
- M18-S3 next-slice selector is now implemented:
  - `scripts/select-m18-next-slice.sh`
  - `scripts/test-select-m18-next-slice.sh`
  - `docs/book/478-m18-next-slice-selector.md`
- M18-S4 editor contract expansion path is now implemented:
  - `compiler/sec4-lsp/src/main.rs` quickfix actions now emit stable `data.id` values.
  - `scripts/test-m18-editor-contract-expansion.sh`
  - `docs/book/479-m18-editor-contract-expansion.md`
- M18-S5 release publish-integrity contract expansion is now implemented:
  - `scripts/verify-release-publish-manifest.sh` now verifies artifact file hashes against release checksum entries.
  - `scripts/test-verify-release-publish-manifest.sh` now includes tampered-sbom regression coverage.
  - `scripts/test-m18-release-publish-integrity.sh`
  - `docs/book/480-m18-release-publish-integrity-contract-expansion.md`
- M18-S6 runtime-track execution runner is now implemented:
  - `scripts/run-m18-runtime-track.sh`
  - `scripts/test-run-m18-runtime-track.sh`
  - `docs/book/481-m18-runtime-track-execution-runner.md`
- M18-S7 track-convergence summary is now implemented:
  - `scripts/build-m18-track-convergence-summary.sh`
  - `scripts/test-build-m18-track-convergence-summary.sh`
  - `docs/book/482-m18-track-convergence-summary.md`
- M18-S8 transition handoff packet is now implemented:
  - `scripts/build-m18-transition-handoff-packet.sh`
  - `scripts/test-build-m18-transition-handoff-packet.sh`
  - `docs/book/483-m18-transition-handoff-packet.md`
- M18-S9 closure report is now implemented:
  - `scripts/build-m18-closure-report.sh`
  - `scripts/test-build-m18-closure-report.sh`
  - `docs/book/484-m18-closure-report.md`
- M19-S1 kickoff brief is now implemented:
  - `scripts/generate-m19-kickoff-brief.sh`
  - `scripts/test-generate-m19-kickoff-brief.sh`
  - `docs/book/485-m19-kickoff-brief.md`
- M19-S2 priority matrix is now implemented:
  - `scripts/build-m19-priority-matrix.sh`
  - `scripts/test-build-m19-priority-matrix.sh`
  - `docs/book/486-m19-priority-matrix.md`
- M19-S3 next-slice selector is now implemented:
  - `scripts/select-m19-next-slice.sh`
  - `scripts/test-select-m19-next-slice.sh`
  - `docs/book/487-m19-next-slice-selector.md`
- M14 replay bootstrap is active with closure gating (`M14-A`, `M14-B`, `M14-C`, `M14-D`).
- M15-S1 replay runtime stubbing bootstrap is now implemented:
  - `sec4 replay --effects mock` emits deterministic executed-stub counts/traces for net/db/fs in text and JSON output modes,
  - replay JSON contract scripts lock execution fields and missing-key guard cases,
  - closure audit now tracks replay execution-contract enforcement via `M15-A`.
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
- M16-S9 trace-correlation bootstrap is now implemented:
  - runtime now emits deterministic `X-Trace-Id` response header and aligns `req.json(...)` error-envelope `traceId` with request-scoped runtime trace ids.
- M16-S10 standard success-envelope bootstrap is now implemented:
  - runtime JSON success responders now emit deterministic structured success envelopes (`ok/status/traceId/timeMs/data[/meta]`).
- M16-S11 `res.okMeta` runtime e2e coverage is now implemented:
  - live HTTP runtime path for `res.okMeta(...)` is now validated with deterministic envelope+meta assertions.
- M16-S12 HTTP method-mismatch semantics are now implemented:
  - runtime now returns deterministic `405 Method Not Allowed` when path matches but method does not.
- M16-S13 `405` Allow-header enrichment is now implemented:
  - runtime `405` responses now include deterministic `Allow` header built from matching route path methods.
- M16-S14 CORS preflight runtime path is now implemented:
  - when `cors.withCors(...)` is active, runtime handles `OPTIONS` preflight with deterministic CORS allow headers and `204 No Content`.
- M16-S15 security-headers runtime path is now implemented:
  - when `sec.withSecurityHeaders(...)` is active, runtime injects deterministic security headers on both success and not-found error responses.
- M16-S16 CSRF runtime gate is now implemented:
  - when `csrf.withCsrf(...)` is active, runtime enforces double-submit token checks on protected methods and returns deterministic `403` on missing/mismatched tokens.
- M16-S17 auth runtime gate is now implemented:
  - when `auth.withAuth(...)` is active, runtime enforces `Authorization: Bearer ...` checks and returns deterministic `401` for missing/invalid auth headers.
- M16-S18 CORS response-header propagation is now implemented:
  - when `cors.withCors(...)` is active, runtime injects `Access-Control-Allow-Origin: *` into non-preflight responses.
- M16-S19 runtime request-body limit configurability is now implemented:
  - `req.json(...)` body-size guard now honors `SEC4_RT_HTTP_MAX_BODY_BYTES` for stricter deployment-time limits.
- M16-S20 CORS error-path propagation coverage is now implemented:
  - runtime e2e now locks `Access-Control-Allow-Origin` propagation on auth (`401`) and csrf (`403`) middleware rejection responses.
- M16-S21 CORS header coverage is now expanded for generic error branches:
  - runtime e2e now locks `Access-Control-Allow-Origin` propagation on `404` not-found and `405` method-mismatch responses when CORS middleware is enabled.
- M16-S22 combined-middleware preflight interoperability is now implemented:
  - runtime e2e now locks that CORS `OPTIONS` preflight remains `204` and bypasses auth/csrf rejection branches when `cors + auth + csrf` middleware are composed.
- M16-S23 security-header coverage is now expanded for middleware rejection branches:
  - runtime e2e now locks security-header injection on auth (`401`) and csrf (`403`) rejection responses when `sec.withSecurityHeaders(...)` is enabled.
- M16-S24 security-header coverage is now expanded for dispatch/preflight branches:
  - runtime e2e now locks security-header injection on `405` method-mismatch and CORS preflight (`204`) responses.
- M16-S25 operator smoke coverage is now implemented:
  - `scripts/smoke-sec4-run-hello-api.sh` now validates `sec4 run` end-to-end request handling (`GET /health`, `POST /users`) against a temporary project copy in deterministic oneshot mode.
- M16-S27 run-command runtime bridge flags are now implemented:
  - `sec4 run` now supports `--oneshot` and `--max-body-bytes <N>` as explicit operator flags instead of requiring direct environment setup.
  - runtime e2e coverage now validates oneshot serving and body-limit enforcement through these CLI flags.
- M16-S28 operator smoke script now uses run-command flags:
  - `scripts/smoke-sec4-run-hello-api.sh` now launches `sec4 run` with `--oneshot` directly.
  - smoke-script contract tests and runtime-smoke artifact checks remain green after migration.
- M16-S29 run-command timeout bridge flag is now implemented:
  - `sec4 run` now supports `--serve-timeout-ms <N>` for deterministic oneshot runtime windows without direct timeout env setup.
  - run-command and operator smoke coverage now exercise this flag path.
- M16-S30 run-command port override is now implemented:
  - `sec4 run` now supports `--port <N>`, wired through runtime env bridge (`SEC4_RT_HTTP_PORT`).
  - operator smoke script now runs on free ports via `--port` without patching source files.
- M16-S31 run-command flag contract CI coverage is now implemented:
  - dedicated contract + guard scripts lock runtime flag bridge surface for `sec4 run`.
  - naming-lock workflow now executes these scripts on pull requests and main pushes.
- M16-S32 runtime-smoke metadata contract is now expanded:
  - smoke artifacts now include deterministic run-flag metadata (`oneshot`, `serveTimeoutMs`, `runFlags`).
  - artifact checker and checker tests now enforce this metadata shape.
- M16-S33 smoke-script surface contract now locks metadata emit tokens:
  - smoke script contract checks now require deterministic metadata fields emitted by `run-metadata.txt` generation block.
- M16-S34 smoke-script guard coverage now includes metadata token drift:
  - guard script now verifies contract failure for removed `runFlags` metadata token in addition to request-token drift.
- M16-S35 runtime-smoke checker regression coverage now includes missing runFlags metadata:
  - checker test now has dedicated failing fixture for absent `runFlags` key.
- M16-S37 runtime-smoke metadata checker now validates source/work project fields:
  - checker enforces non-empty `sourceProject` and `workProject` metadata keys.
  - regression coverage includes dedicated missing-sourceProject failure fixture.
- M16-S38 runtime-smoke checker regression coverage now validates missing workProject metadata:
  - checker tests include dedicated failing fixture for absent `workProject`.
  - deterministic missing-workProject diagnostic is now contract-locked.
- M16-S39 smoke-script contract now locks source/work metadata emit tokens:
  - smoke-script contract checker now requires `sourceProject=${source_project}` and `workProject=${work_project}` tokens.
  - guard coverage now validates deterministic missing-token diagnostics for both metadata tokens.
- M16-S40 runtime-smoke checker regression coverage now validates empty source/work metadata values:
  - checker tests include dedicated fixtures for `sourceProject=` and `workProject=` empty-value failures.
  - deterministic diagnostics remain identical to missing-field branches.
- M16-S41 runtime-smoke checker now enforces source/work provenance divergence:
  - checker rejects metadata where `sourceProject` equals `workProject`.
  - regression coverage now pins deterministic divergence diagnostic.
- M16-S42 runtime-smoke success-envelope checker now enforces `timeMs` field typing:
  - `users.body` contract now requires numeric `timeMs` alongside `ok/status/traceId/data`.
  - regression coverage includes malformed-envelope fixture missing `timeMs`.
- M16-S43 runtime-smoke success-envelope checker now enforces traceId format:
  - `users.body` contract now requires trace ids matching `rt-[0-9]+`.
  - regression coverage includes malformed traceId fixture.
- M16-S44 runtime-smoke checker now enforces users trace header/body correlation:
  - `users.headers` must include `X-Trace-Id`.
  - header trace id must match `users.body.traceId`.
- M16-S45 runtime-smoke checker now enforces health trace-header contract:
  - `health.headers` must include `X-Trace-Id`.
  - health trace header value must match `rt-[0-9]+` format.
- M16-S46 runtime-smoke checker now enforces run-log invocation flag contracts:
  - `health.run.log` and `users.run.log` must contain `--port`, `--oneshot`, and `--serve-timeout-ms 12000` tokens.
  - regression coverage includes deterministic missing-token failures for both logs.
- M16-S47 runtime-smoke checker now enforces run-log/metadata port correlation:
  - invocation logs must include exact `--port <run-metadata port>` token.
  - regression coverage includes deterministic port-mismatch failure fixture.
- M16-S48 runtime-smoke checker now enforces run-log/metadata timeout correlation:
  - invocation logs must include exact `--serve-timeout-ms <run-metadata serveTimeoutMs>` token.
  - regression coverage includes deterministic timeout-mismatch failure fixture.
- M16-S49 operator smoke script now supports timeout override:
  - `scripts/smoke-sec4-run-hello-api.sh` accepts `--serve-timeout-ms <ms>` with default `12000`.
  - smoke-script contract/guard checks now lock variable-based timeout tokens.
- M16-S50 runtime-smoke checker now enforces metadata numeric bounds:
  - metadata `port` must be in range `1..65535`.
  - metadata `serveTimeoutMs` must be positive (`> 0`).
- M16-S51 operator smoke script now supports max-body override with metadata/log correlation checks:
  - smoke script accepts `--max-body-bytes <bytes>` and emits `maxBodyBytes` metadata (`unset` or numeric).
  - checker enforces max-body metadata shape and optional run-log token correlation when set.
- M16-S52 runtime-smoke checker regression coverage now pins users-branch max-body correlation failures:
  - dedicated fixture ensures health log passes max-body correlation while users log fails.
  - deterministic users-branch max-body mismatch diagnostic is contract-locked.
- M16-S53 runtime-smoke checker regression coverage now includes invalid-shape maxBody metadata:
  - dedicated fixture asserts failure for `maxBodyBytes=abc`.
  - deterministic invalid-shape diagnostic is contract-locked.
- M16-S54 runtime-smoke metadata now enforces shape-aware `runFlags` for optional max-body wiring:
  - smoke script emits `runFlags` with `--max-body-bytes` only when `maxBodyBytes` is set.
  - checker enforces deterministic `runFlags` shape against the max-body metadata branch.
  - regression coverage now includes both unset/set max-body `runFlags` shape-drift fixtures.
- M16-S55 runtime-smoke CI now exercises both metadata-shape branches:
  - workflow runs smoke+checker for default branch and `--max-body-bytes` branch.
  - workflow contract/guard + closure fixtures now lock both branch invocations.
- M16-S56 runtime-smoke artifact bundle now includes aggregated branch indexing:
  - workflow builds `runtime-smoke-branch-index.json` from `default` + `max-body` artifact sets.
  - naming-lock CI and closure fixtures now lock the branch-index builder contract.
- M16-S57 runtime-smoke workflow now uses one deterministic bundle checker pass:
  - workflow validates both branch artifacts + index generation via `scripts/check-runtime-smoke-bundle.sh`.
  - naming-lock CI and closure fixtures now lock bundle-checker workflow contract coverage.
- M16 closure enforcement is active (`M16-A`) for runtime HTTP coverage contract + guard tests in naming-lock CI.
- M16 operator smoke-script closure enforcement is active (`M16-B`) for naming-lock CI contract + guard checks.
- M16 runtime-smoke workflow closure enforcement is active:
  - `M16-C` validates workflow contract (`.github/workflows/runtime-smoke.yml`, including dual-branch smoke execution + bundle-check validation/index generation + artifact upload),
  - `M16-D` validates naming-lock CI enforcement of runtime-smoke contract + guard tests + artifact checker/index/bundle tests.
- M16 run-command runtime-flag CI-guard enforcement is active (`M16-E`):
  - validates naming-lock CI enforcement of `sec4 run` runtime-flag contract + guard tests.

## Formal Closure Audit (Strict, 2026-02-14)

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
| `M12-B` | PASS | Naming-lock CI enforces local path leak guard test | `.github/workflows/naming-lock.yml` |
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
| `M15-A` | PASS | Replay CLI json contract scripts enforce execution fields + missing-key guard cases | `scripts/test-replay-cli-json-contract.sh`, `scripts/test-replay-cli-json-contract-guard.sh` |
| `M16-A` | PASS | Naming-lock CI enforces M16 runtime HTTP coverage contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M16-B` | PASS | Naming-lock CI enforces sec4 run hello-api smoke script contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M16-C` | PASS | Runtime-smoke workflow runs dual-branch sec4 run smoke on main push, validates artifacts, builds branch index, and uploads runtime artifacts | `.github/workflows/runtime-smoke.yml` |
| `M16-D` | PASS | Naming-lock CI enforces runtime-smoke workflow contract + guard tests + artifact checker/index/bundle tests | `.github/workflows/naming-lock.yml` |
| `M16-E` | PASS | Naming-lock CI enforces sec4 run runtime-flag contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M17-A` | PASS | Naming-lock CI enforces M17 operator handoff readiness checker | `.github/workflows/naming-lock.yml` |
| `M17-B` | PASS | Naming-lock CI enforces M17 operator bootstrap profile helper | `.github/workflows/naming-lock.yml` |
| `M17-C` | PASS | Naming-lock CI enforces M17 operator troubleshooting matrix | `.github/workflows/naming-lock.yml` |
| `M17-D` | PASS | Naming-lock CI enforces M17 operator handoff quickstart | `.github/workflows/naming-lock.yml` |
| `M17-E` | PASS | Naming-lock CI enforces M17 operator handoff CI smoke wrapper | `.github/workflows/naming-lock.yml` |
| `M17-F` | PASS | Operator-handoff workflow executes CI smoke wrapper + artifact upload on main push | `.github/workflows/operator-handoff-smoke.yml` |
| `M17-G` | PASS | Naming-lock CI enforces operator-handoff workflow contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M17-H` | PASS | Naming-lock CI enforces M17 operator handoff artifact inspector | `.github/workflows/naming-lock.yml` |
| `M17-I` | PASS | Naming-lock CI enforces M17 operator handoff readiness summary | `.github/workflows/naming-lock.yml` |
| `M17-J` | PASS | Naming-lock CI enforces M17 operator release-packet builder | `.github/workflows/naming-lock.yml` |
| `M17-K` | PASS | Naming-lock CI enforces M17 final handoff playbook checker | `.github/workflows/naming-lock.yml` |
| `M17-L` | PASS | Naming-lock CI enforces M17 clean-clone rehearsal runner | `.github/workflows/naming-lock.yml` |
| `M18-A` | PASS | Naming-lock CI enforces M18 kickoff brief generator | `.github/workflows/naming-lock.yml` |
| `M18-B` | PASS | Naming-lock CI enforces M18 priority matrix artifact | `.github/workflows/naming-lock.yml` |
| `M18-C` | PASS | Naming-lock CI enforces M18 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M18-D` | PASS | Naming-lock CI enforces M18 editor contract expansion | `.github/workflows/naming-lock.yml` |
| `M18-E` | PASS | Naming-lock CI enforces M18 release publish-integrity contract expansion | `.github/workflows/naming-lock.yml` |
| `M18-F` | PASS | Naming-lock CI enforces M18 runtime-track execution runner | `.github/workflows/naming-lock.yml` |
| `M18-G` | PASS | Naming-lock CI enforces M18 track-convergence summary | `.github/workflows/naming-lock.yml` |
| `M18-H` | PASS | Naming-lock CI enforces M18 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M18-I` | PASS | Naming-lock CI enforces M18 closure report | `.github/workflows/naming-lock.yml` |
| `M19-A` | PASS | Naming-lock CI enforces M19 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M19-B` | PASS | Naming-lock CI enforces M19 priority matrix | `.github/workflows/naming-lock.yml` |
| `M19-C` | PASS | Naming-lock CI enforces M19 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M19-D` | PASS | Naming-lock CI enforces M19 runtime hardening runner | `.github/workflows/naming-lock.yml` |
| `M19-E` | PASS | Naming-lock CI enforces M19 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M19-F` | PASS | Naming-lock CI enforces M19 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M19-G` | PASS | Naming-lock CI enforces M19 closure report | `.github/workflows/naming-lock.yml` |
| `M20-A` | PASS | Naming-lock CI enforces M20 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M20-B` | PASS | Naming-lock CI enforces M20 priority matrix | `.github/workflows/naming-lock.yml` |
| `M20-C` | PASS | Naming-lock CI enforces M20 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M20-D` | PASS | Naming-lock CI enforces M20 runtime hardening runner | `.github/workflows/naming-lock.yml` |
| `M20-E` | PASS | Naming-lock CI enforces M20 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M20-F` | PASS | Naming-lock CI enforces M20 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M20-G` | PASS | Naming-lock CI enforces M20 closure report | `.github/workflows/naming-lock.yml` |
| `M21-A` | PASS | Naming-lock CI enforces M21 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M21-B` | PASS | Naming-lock CI enforces M21 priority matrix | `.github/workflows/naming-lock.yml` |
| `M21-C` | PASS | Naming-lock CI enforces M21 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M21-D` | PASS | Naming-lock CI enforces M21 runtime hardening runner | `.github/workflows/naming-lock.yml` |
| `M21-E` | PASS | Naming-lock CI enforces M21 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M21-F` | PASS | Naming-lock CI enforces M21 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M21-G` | PASS | Naming-lock CI enforces M21 closure report | `.github/workflows/naming-lock.yml` |
| `M22-A` | PASS | Naming-lock CI enforces M22 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M22-B` | PASS | Naming-lock CI enforces M22 priority matrix | `.github/workflows/naming-lock.yml` |
| `M22-C` | PASS | Naming-lock CI enforces M22 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M22-D` | PASS | Naming-lock CI enforces M22 runtime hardening runner | `.github/workflows/naming-lock.yml` |
| `M22-E` | PASS | Naming-lock CI enforces M22 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M22-F` | PASS | Naming-lock CI enforces M22 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M22-G` | PASS | Naming-lock CI enforces M22 closure report | `.github/workflows/naming-lock.yml` |
| `M23-A` | PASS | Naming-lock CI enforces M23 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M23-B` | PASS | Naming-lock CI enforces M23 priority matrix | `.github/workflows/naming-lock.yml` |
| `M23-C` | PASS | Naming-lock CI enforces M23 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M23-D` | PASS | Naming-lock CI enforces M23 runtime hardening runner | `.github/workflows/naming-lock.yml` |
| `M23-E` | PASS | Naming-lock CI enforces M23 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M23-F` | PASS | Naming-lock CI enforces M23 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M23-G` | PASS | Naming-lock CI enforces M23 closure report | `.github/workflows/naming-lock.yml` |
| `M24-A` | PASS | Naming-lock CI enforces M24 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M24-B` | PASS | Naming-lock CI enforces M24 priority matrix | `.github/workflows/naming-lock.yml` |
| `M24-C` | PASS | Naming-lock CI enforces M24 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M24-D` | PASS | Naming-lock CI enforces M24 runtime hardening runner | `.github/workflows/naming-lock.yml` |
| `M24-E` | PASS | Naming-lock CI enforces M24 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M24-F` | PASS | Naming-lock CI enforces M24 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M24-G` | PASS | Naming-lock CI enforces M24 closure report | `.github/workflows/naming-lock.yml` |
| `M25-A` | PASS | Naming-lock CI enforces M25 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M25-B` | PASS | Naming-lock CI enforces M25 priority matrix | `.github/workflows/naming-lock.yml` |
| `M25-C` | PASS | Naming-lock CI enforces M25 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M25-D` | PASS | Naming-lock CI enforces M25 runtime hardening runner | `.github/workflows/naming-lock.yml` |
| `M25-E` | PASS | Naming-lock CI enforces M25 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M25-F` | PASS | Naming-lock CI enforces M25 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M25-G` | PASS | Naming-lock CI enforces M25 closure report | `.github/workflows/naming-lock.yml` |
| `M26-A` | PASS | Naming-lock CI enforces M26 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M26-B` | PASS | Naming-lock CI enforces M26 priority matrix | `.github/workflows/naming-lock.yml` |
| `M26-C` | PASS | Naming-lock CI enforces M26 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M26-D` | PASS | Naming-lock CI enforces M26 runtime hardening runner | `.github/workflows/naming-lock.yml` |
| `M26-E` | PASS | Naming-lock CI enforces M26 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M26-F` | PASS | Naming-lock CI enforces M26 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M26-G` | PASS | Naming-lock CI enforces M26 closure report | `.github/workflows/naming-lock.yml` |
| `M27-A` | PASS | Naming-lock CI enforces M27 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M27-B` | PASS | Naming-lock CI enforces M27 priority matrix | `.github/workflows/naming-lock.yml` |
| `M27-C` | PASS | Naming-lock CI enforces M27 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M27-D` | PASS | Naming-lock CI enforces M27 runtime hardening runner | `.github/workflows/naming-lock.yml` |
| `M27-E` | PASS | Naming-lock CI enforces M27 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M27-F` | PASS | Naming-lock CI enforces M27 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M27-G` | PASS | Naming-lock CI enforces M27 closure report | `.github/workflows/naming-lock.yml` |
| `M28-A` | PASS | Naming-lock CI enforces M28 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M28-B` | PASS | Naming-lock CI enforces M28 priority matrix | `.github/workflows/naming-lock.yml` |
| `M28-C` | PASS | Naming-lock CI enforces M28 next-slice selector | `.github/workflows/naming-lock.yml` |
| `M28-D` | PASS | Naming-lock CI enforces M28 runtime hardening runner | `.github/workflows/naming-lock.yml` |
| `M28-E` | PASS | Naming-lock CI enforces M28 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M28-F` | PASS | Naming-lock CI enforces M28 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M28-G` | PASS | Naming-lock CI enforces M28 closure report | `.github/workflows/naming-lock.yml` |
| `M29-A` | PASS | Naming-lock CI enforces M29 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M29-B` | PASS | Naming-lock CI enforces M29 priority matrix | `.github/workflows/naming-lock.yml` |
| `M29-C` | PASS | Naming-lock CI enforces M29 runtime de-stub planner | `.github/workflows/naming-lock.yml` |
| `M29-D` | PASS | Naming-lock CI enforces M29 runtime de-stub runner | `.github/workflows/naming-lock.yml` |
| `M29-E` | PASS | Naming-lock CI enforces M29 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M29-F` | PASS | Naming-lock CI enforces M29 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M29-G` | PASS | Naming-lock CI enforces M29 closure report | `.github/workflows/naming-lock.yml` |
| `M30-A` | PASS | Naming-lock CI enforces M30 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M30-B` | PASS | Naming-lock CI enforces M30 priority matrix | `.github/workflows/naming-lock.yml` |
| `M30-C` | PASS | Naming-lock CI enforces M30 runtime de-stub planner | `.github/workflows/naming-lock.yml` |
| `M30-D` | PASS | Naming-lock CI enforces M30 runtime de-stub runner | `.github/workflows/naming-lock.yml` |
| `M30-E` | PASS | Naming-lock CI enforces M30 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M30-F` | PASS | Naming-lock CI enforces M30 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M30-G` | PASS | Naming-lock CI enforces M30 closure report | `.github/workflows/naming-lock.yml` |
| `M31-A` | PASS | Naming-lock CI enforces M31 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M31-B` | PASS | Naming-lock CI enforces M31 priority matrix | `.github/workflows/naming-lock.yml` |
| `M31-C` | PASS | Naming-lock CI enforces M31 runtime de-stub planner | `.github/workflows/naming-lock.yml` |
| `M31-D` | PASS | Naming-lock CI enforces M31 runtime de-stub runner | `.github/workflows/naming-lock.yml` |
| `M31-E` | PASS | Naming-lock CI enforces M31 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M31-F` | PASS | Naming-lock CI enforces M31 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M31-G` | PASS | Naming-lock CI enforces M31 closure report | `.github/workflows/naming-lock.yml` |
| `M32-A` | PASS | Naming-lock CI enforces M32 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M32-B` | PASS | Naming-lock CI enforces M32 priority matrix | `.github/workflows/naming-lock.yml` |
| `M32-C` | PASS | Naming-lock CI enforces M32 runtime de-stub planner | `.github/workflows/naming-lock.yml` |
| `M32-D` | PASS | Naming-lock CI enforces M32 runtime de-stub runner | `.github/workflows/naming-lock.yml` |
| `M32-E` | PASS | Naming-lock CI enforces M32 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M32-F` | PASS | Naming-lock CI enforces M32 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M32-G` | PASS | Naming-lock CI enforces M32 closure report | `.github/workflows/naming-lock.yml` |
| `M33-A` | PASS | Naming-lock CI enforces M33 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M33-B` | PASS | Naming-lock CI enforces M33 priority matrix | `.github/workflows/naming-lock.yml` |
| `M33-C` | PASS | Naming-lock CI enforces M33 runtime de-stub planner | `.github/workflows/naming-lock.yml` |
| `M33-D` | PASS | Naming-lock CI enforces M33 runtime de-stub runner | `.github/workflows/naming-lock.yml` |
| `M33-E` | PASS | Naming-lock CI enforces M33 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M33-F` | PASS | Naming-lock CI enforces M33 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M33-G` | PASS | Naming-lock CI enforces M33 closure report | `.github/workflows/naming-lock.yml` |
| `M34-A` | PASS | Naming-lock CI enforces M34 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M34-B` | PASS | Naming-lock CI enforces M34 priority matrix | `.github/workflows/naming-lock.yml` |
| `M34-C` | PASS | Naming-lock CI enforces M34 runtime de-stub planner | `.github/workflows/naming-lock.yml` |
| `M34-D` | PASS | Naming-lock CI enforces M34 runtime de-stub runner | `.github/workflows/naming-lock.yml` |
| `M34-E` | PASS | Naming-lock CI enforces M34 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M34-F` | PASS | Naming-lock CI enforces M34 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M34-G` | PASS | Naming-lock CI enforces M34 closure report | `.github/workflows/naming-lock.yml` |
| `M35-A` | PASS | Naming-lock CI enforces M35 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M35-B` | PASS | Naming-lock CI enforces M35 priority matrix | `.github/workflows/naming-lock.yml` |
| `M35-C` | PASS | Naming-lock CI enforces M35 runtime de-stub planner | `.github/workflows/naming-lock.yml` |
| `M35-D` | PASS | Naming-lock CI enforces M35 runtime de-stub runner | `.github/workflows/naming-lock.yml` |
| `M35-E` | PASS | Naming-lock CI enforces M35 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M35-F` | PASS | Naming-lock CI enforces M35 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M35-G` | PASS | Naming-lock CI enforces M35 closure report | `.github/workflows/naming-lock.yml` |
| `M36-A` | PASS | Naming-lock CI enforces M36 kickoff brief | `.github/workflows/naming-lock.yml` |
| `M36-B` | PASS | Naming-lock CI enforces M36 priority matrix | `.github/workflows/naming-lock.yml` |
| `M36-C` | PASS | Naming-lock CI enforces M36 runtime de-stub planner | `.github/workflows/naming-lock.yml` |
| `M36-D` | PASS | Naming-lock CI enforces M36 runtime de-stub runner | `.github/workflows/naming-lock.yml` |
| `M36-E` | PASS | Naming-lock CI enforces M36 executed-slice convergence summary | `.github/workflows/naming-lock.yml` |
| `M36-F` | PASS | Naming-lock CI enforces M36 transition handoff packet | `.github/workflows/naming-lock.yml` |
| `M36-G` | PASS | Naming-lock CI enforces M36 closure report | `.github/workflows/naming-lock.yml` |

Strict closure interpretation:
- M11 and M12 are complete for current scope.
- M9 implementation is functionally complete but release-candidate evidence remains a verification activity.
- M10 and M13 closure evidence requirements are now satisfied.
- M12 local-path leak guard enforcement is active (`M12-B`).
- M14 bootstrap replay-contract enforcement is active (`M14-A`).
- M14 replay compatibility enforcement is active (`M14-B`).
- M14 replay stub-registry bootstrap enforcement is active (`M14-C`).
- M14 replay CLI json contract enforcement is active (`M14-D`).
- M15 replay execution contract guard enforcement is active (`M15-A`).
- M16 runtime HTTP coverage closure enforcement is active (`M16-A`).
- M16 operator smoke-script closure enforcement is active (`M16-B`).
- M16 runtime-smoke workflow contract enforcement is active (`M16-C`).
- M16 runtime-smoke CI-guard enforcement is active (`M16-D`).
- M16 run-command runtime-flag CI-guard enforcement is active (`M16-E`).
- M17 operator handoff readiness checker enforcement is active (`M17-A`).
- M17 operator bootstrap profile helper enforcement is active (`M17-B`).
- M17 operator troubleshooting matrix enforcement is active (`M17-C`).
- M17 operator handoff quickstart enforcement is active (`M17-D`).
- M17 operator handoff CI smoke wrapper enforcement is active (`M17-E`).
- M17 operator-handoff workflow contract enforcement is active (`M17-F`).
- M17 operator-handoff workflow CI-guard enforcement is active (`M17-G`).
- M17 operator handoff artifact-inspector enforcement is active (`M17-H`).
- M17 operator handoff readiness-summary enforcement is active (`M17-I`).
- M17 operator release-packet builder enforcement is active (`M17-J`).
- M17 final handoff playbook checker enforcement is active (`M17-K`).
- M17 clean-clone rehearsal runner enforcement is active (`M17-L`).
- M18 kickoff brief generator enforcement is active (`M18-A`).
- M18 priority matrix artifact enforcement is active (`M18-B`).
- M18 next-slice selector enforcement is active (`M18-C`).
- M18 editor contract expansion enforcement is active (`M18-D`).
- M18 release publish-integrity contract expansion enforcement is active (`M18-E`).
- M18 runtime-track execution runner enforcement is active (`M18-F`).
- M18 track-convergence summary enforcement is active (`M18-G`).
- M18 transition handoff packet enforcement is active (`M18-H`).
- M18 closure report enforcement is active (`M18-I`).
- M19 kickoff brief enforcement is active (`M19-A`).
- M19 priority matrix enforcement is active (`M19-B`).
- M19 next-slice selector enforcement is active (`M19-C`).
- M19 runtime hardening runner enforcement is active (`M19-D`).
- M19 executed-slice convergence summary enforcement is active (`M19-E`).
- M19 transition handoff packet enforcement is active (`M19-F`).
- M19 closure report enforcement is active (`M19-G`).
- M20 kickoff brief enforcement is active (`M20-A`).
- M20 priority matrix enforcement is active (`M20-B`).
- M20 next-slice selector enforcement is active (`M20-C`).
- M20 runtime hardening runner enforcement is active (`M20-D`).
- M20 executed-slice convergence summary enforcement is active (`M20-E`).
- M20 transition handoff packet enforcement is active (`M20-F`).
- M20 closure report enforcement is active (`M20-G`).
- M21 kickoff brief enforcement is active (`M21-A`).
- M21 priority matrix enforcement is active (`M21-B`).
- M21 next-slice selector enforcement is active (`M21-C`).
- M21 runtime hardening runner enforcement is active (`M21-D`).
- M21 executed-slice convergence summary enforcement is active (`M21-E`).
- M21 transition handoff packet enforcement is active (`M21-F`).
- M21 closure report enforcement is active (`M21-G`).
- M22 kickoff brief enforcement is active (`M22-A`).
- M22 priority matrix enforcement is active (`M22-B`).
- M22 next-slice selector enforcement is active (`M22-C`).
- M22 runtime hardening runner enforcement is active (`M22-D`).
- M22 executed-slice convergence summary enforcement is active (`M22-E`).
- M22 transition handoff packet enforcement is active (`M22-F`).
- M22 closure report enforcement is active (`M22-G`).
- M23 kickoff brief enforcement is active (`M23-A`).
- M23 priority matrix enforcement is active (`M23-B`).
- M23 next-slice selector enforcement is active (`M23-C`).
- M23 runtime hardening runner enforcement is active (`M23-D`).
- M23 executed-slice convergence summary enforcement is active (`M23-E`).
- M23 transition handoff packet enforcement is active (`M23-F`).
- M23 closure report enforcement is active (`M23-G`).
- M24 kickoff brief enforcement is active (`M24-A`).
- M24 priority matrix enforcement is active (`M24-B`).
- M24 next-slice selector enforcement is active (`M24-C`).
- M24 runtime hardening runner enforcement is active (`M24-D`).
- M24 executed-slice convergence summary enforcement is active (`M24-E`).
- M24 transition handoff packet enforcement is active (`M24-F`).
- M24 closure report enforcement is active (`M24-G`).
- M25 kickoff brief enforcement is active (`M25-A`).
- M25 priority matrix enforcement is active (`M25-B`).
- M25 next-slice selector enforcement is active (`M25-C`).
- M25 runtime hardening runner enforcement is active (`M25-D`).
- M25 executed-slice convergence summary enforcement is active (`M25-E`).
- M25 transition handoff packet enforcement is active (`M25-F`).
- M25 closure report enforcement is active (`M25-G`).
- M26 kickoff brief enforcement is active (`M26-A`).
- M26 priority matrix enforcement is active (`M26-B`).
- M26 next-slice selector enforcement is active (`M26-C`).
- M26 runtime hardening runner enforcement is active (`M26-D`).
- M26 executed-slice convergence summary enforcement is active (`M26-E`).
- M26 transition handoff packet enforcement is active (`M26-F`).
- M26 closure report enforcement is active (`M26-G`).
- M27 kickoff brief enforcement is active (`M27-A`).
- M27 priority matrix enforcement is active (`M27-B`).
- M27 next-slice selector enforcement is active (`M27-C`).
- M27 runtime hardening runner enforcement is active (`M27-D`).
- M27 executed-slice convergence summary enforcement is active (`M27-E`).
- M27 transition handoff packet enforcement is active (`M27-F`).
- M27 closure report enforcement is active (`M27-G`).
- M28 kickoff brief enforcement is active (`M28-A`).
- M28 priority matrix enforcement is active (`M28-B`).
- M28 next-slice selector enforcement is active (`M28-C`).
- M28 runtime hardening runner enforcement is active (`M28-D`).
- M28 executed-slice convergence summary enforcement is active (`M28-E`).
- M28 transition handoff packet enforcement is active (`M28-F`).
- M28 closure report enforcement is active (`M28-G`).
- M29 kickoff brief enforcement is active (`M29-A`).
- M29 priority matrix enforcement is active (`M29-B`).
- M29 runtime de-stub planner enforcement is active (`M29-C`).
- M29 runtime de-stub runner enforcement is active (`M29-D`).
- M29 executed-slice convergence summary enforcement is active (`M29-E`).
- M29 transition handoff packet enforcement is active (`M29-F`).
- M29 closure report enforcement is active (`M29-G`).
- M30 kickoff brief enforcement is active (`M30-A`).
- M30 priority matrix enforcement is active (`M30-B`).
- M30 runtime de-stub planner enforcement is active (`M30-C`).
- M30 runtime de-stub runner enforcement is active (`M30-D`).
- M30 executed-slice convergence summary enforcement is active (`M30-E`).
- M30 transition handoff packet enforcement is active (`M30-F`).
- M30 closure report enforcement is active (`M30-G`).
- M31 kickoff brief enforcement is active (`M31-A`).
- M31 priority matrix enforcement is active (`M31-B`).
- M31 runtime de-stub planner enforcement is active (`M31-C`).
- M31 runtime de-stub runner enforcement is active (`M31-D`).
- M31 executed-slice convergence summary enforcement is active (`M31-E`).
- M31 transition handoff packet enforcement is active (`M31-F`).
- M31 closure report enforcement is active (`M31-G`).
- M32 kickoff brief enforcement is active (`M32-A`).
- M32 priority matrix enforcement is active (`M32-B`).
- M32 runtime de-stub planner enforcement is active (`M32-C`).
- M32 runtime de-stub runner enforcement is active (`M32-D`).
- M32 executed-slice convergence summary enforcement is active (`M32-E`).
- M32 transition handoff packet enforcement is active (`M32-F`).
- M32 closure report enforcement is active (`M32-G`).
- M33 kickoff brief enforcement is active (`M33-A`).
- M33 priority matrix enforcement is active (`M33-B`).
- M33 runtime de-stub planner enforcement is active (`M33-C`).
- M33 runtime de-stub runner enforcement is active (`M33-D`).
- M33 executed-slice convergence summary enforcement is active (`M33-E`).
- M33 transition handoff packet enforcement is active (`M33-F`).
- M33 closure report enforcement is active (`M33-G`).
- M34 kickoff brief enforcement is active (`M34-A`).
- M34 priority matrix enforcement is active (`M34-B`).
- M34 runtime de-stub planner enforcement is active (`M34-C`).
- M34 runtime de-stub runner enforcement is active (`M34-D`).
- M34 executed-slice convergence summary enforcement is active (`M34-E`).
- M34 transition handoff packet enforcement is active (`M34-F`).
- M34 closure report enforcement is active (`M34-G`).
- M35 kickoff brief enforcement is active (`M35-A`).
- M35 priority matrix enforcement is active (`M35-B`).
- M35 runtime de-stub planner enforcement is active (`M35-C`).
- M35 runtime de-stub runner enforcement is active (`M35-D`).
- M35 executed-slice convergence summary enforcement is active (`M35-E`).
- M35 transition handoff packet enforcement is active (`M35-F`).
- M35 closure report enforcement is active (`M35-G`).
- M36 kickoff brief enforcement is active (`M36-A`).
- M36 priority matrix enforcement is active (`M36-B`).
- M36 runtime de-stub planner enforcement is active (`M36-C`).
- M36 runtime de-stub runner enforcement is active (`M36-D`).
- M36 executed-slice convergence summary enforcement is active (`M36-E`).
- M36 transition handoff packet enforcement is active (`M36-F`).
- M36 closure report enforcement is active (`M36-G`).

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
- M4 security-foundation implementation completed (historical slice record):
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
  - closure now verifies naming-lock CI enforcement of local path leak guard test (`M12-B`).
  - release-contract-smoke workflow contract test now enforces trigger coverage (`push` on `main`).
  - closure audit evidence paths are rendered repository-relative to avoid leaking local absolute workspace paths.
  - closure now verifies benchmark-smoke workflow keeps closure + cross-impl/trend contract guard tests plus strict closure audit wiring (`M13-D`).
  - closure now verifies naming-lock CI enforcement of benchmark-trend workflow contract + guard tests (`M13-E`).
  - closure now verifies naming-lock CI enforcement of sec4 explain audit-coverage contract test (`M13-F`).
  - closure now verifies naming-lock CI enforcement of replay capture contract test (`M14-A`).
  - closure now verifies naming-lock CI enforcement of replay capture compatibility test (`M14-B`).
  - closure now verifies naming-lock CI enforcement of replay stub registry contract test (`M14-C`).
  - closure now verifies naming-lock CI enforcement of replay CLI json contract + guard tests (`M14-D`).
  - closure now verifies naming-lock CI enforcement of M16 runtime HTTP coverage contract + guard tests (`M16-A`).
  - closure now verifies naming-lock CI enforcement of sec4 run hello-api smoke script contract + guard tests (`M16-B`).
  - closure now verifies runtime-smoke workflow contract wiring (`M16-C`).
  - closure now verifies naming-lock CI enforcement of runtime-smoke workflow contract + guard tests + artifact checker/index tests (`M16-D`).
  - closure now verifies naming-lock CI enforcement of sec4 run runtime-flag contract + guard tests (`M16-E`).
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
- Post-alpha backend expansion priority is explicit: WASM/browser profile work starts immediately after no-stub alpha release closure.

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
14. Post-alpha backend expansion: add WASM backend and browser sandbox runtime profile.

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
- [x] Trend-note chapter refreshed from current compare-matrix via local render+import fallback when remote CI artifact fetch is unavailable.
- [x] Trend-note updater now supports deterministic local fallback mode (`--prefer-local` + auto fallback on fetch failure).

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
- Chapter: "M13 Slice: Trend Note Update Local Fallback Mode".
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
- [x] Scope locked and documented.
- [x] Runtime net stub materialization wired in replay `mock` path.
- [x] Runtime DB stub materialization wired in replay `mock` path.
- [x] Runtime FS stub materialization wired in replay `mock` path.
- [x] Replay runtime execution JSON/text contracts defined and tested.
- [x] Runtime replay guard scripts added and wired into naming-lock CI.
- [x] Closure-audit checks expanded with M15 runtime replay enforcement gates.

### M15-S1a follow-up slice (mock execution trace/count contract)
#### Scope
- Expose explicit deterministic replay execution evidence in mock mode so replay output can be treated as runtime materialization evidence, not only compatibility precheck output.

#### Build tasks
- Add mock execution summary counters for net/db/fs replay operations.
- Add deterministic mock execution traces for net/db/fs:
  - net trace includes status/truncated/body kind from selected stub,
  - db/fs traces align with matched dependency-signature ordering.
- Emit execution summaries/traces in both text and JSON replay output modes.
- Extend replay CLI integration tests to lock output contract presence and deterministic values.

#### Acceptance criteria
- `sec4 replay --effects mock` text output includes deterministic execution counters/traces.
- `sec4 replay --effects mock --format json` includes parseable `mockExecutionCounts` and `mockExecutionTraces`.
- Existing replay contract suites remain green.

#### Tracking (live status)
- [x] `mockExecutionCounts` added for net/db/fs in replay JSON output.
- [x] `mockExecutionTraces` added for net/db/fs in replay JSON output.
- [x] Replay text output includes deterministic execution counter/trace summaries.
- [x] Replay integration tests updated for both no-dependency and dependency-backed mock runs.

### M15-S1b follow-up slice (replay execution contract guard hardening)
#### Scope
- Lock the new replay execution-output fields in script-level contract tests so runtime execution evidence cannot silently drift.

#### Build tasks
- Extend replay JSON contract checker with required keys:
  - `mockExecutionCounts`
  - `mockExecutionTraces`
- Extend replay JSON guard script with negative fixtures for missing execution keys.
- Re-run guard scripts to ensure deterministic pass/fail behavior remains stable.

#### Acceptance criteria
- `scripts/test-replay-cli-json-contract.sh` fails if execution keys are removed.
- `scripts/test-replay-cli-json-contract-guard.sh` includes explicit missing-key failures for execution fields.
- Naming-lock workflow still executes replay JSON contract scripts unchanged.

#### Tracking (live status)
- [x] Replay JSON contract checker requires `mockExecutionCounts`.
- [x] Replay JSON contract checker requires `mockExecutionTraces`.
- [x] Guard script includes missing-`mockExecutionCounts` failure case.
- [x] Guard script includes missing-`mockExecutionTraces` failure case.
- [x] Replay JSON contract + guard scripts pass locally after hardening.

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
- Chapter: "M15 Slice: Replay Mock Execution Trace and Count Contract".
- Chapter: "M15 Slice: Replay Execution Contract Guard Hardening".

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

### M16-S9 follow-up slice (runtime trace-correlation bootstrap)
#### Scope
- Introduce deterministic request-scoped trace IDs in the runtime response path so HTTP responses and structured runtime errors share the same trace correlation signal.

#### Build tasks
- Add runtime request trace-id assignment per handled request.
- Emit `X-Trace-Id` response header on runtime responses.
- Wire `sec4_rt_store_std_error_response(...)` to use current request trace-id in error payload.
- Add integration assertions for:
  - trace header on success response,
  - trace id field on runtime gate error payload.

#### Acceptance criteria
- Runtime responses include deterministic `X-Trace-Id`.
- Runtime `req.json(...)` error envelope `traceId` matches request trace id.
- Existing runtime HTTP tests remain stable and green.

#### Tracking (live status)
- [x] Runtime request-scoped trace-id assignment added.
- [x] Response header includes deterministic `X-Trace-Id`.
- [x] Runtime error-envelope trace-id now aligns with request trace id.
- [x] Runtime e2e assertions added for trace header/error trace id fields.

### M16-S10 follow-up slice (runtime standard success-envelope alignment)
#### Scope
- Align runtime JSON success responses with the documented standard success-envelope contract while preserving deterministic bootstrap behavior.

#### Build tasks
- Add runtime helper for standard JSON success envelopes:
  - `ok`,
  - `status`,
  - `traceId`,
  - `timeMs`,
  - `data`,
  - optional `meta`.
- Migrate runtime success responders:
  - `res.json`,
  - `res.ok`,
  - `res.okMeta`.
- Update runtime integration assertions to validate structured envelope fields.

#### Acceptance criteria
- JSON success responders no longer emit ad-hoc `{"ok":true}` payloads only.
- Runtime success payloads contain deterministic envelope fields with request trace correlation.
- Existing HTTP runtime success tests remain green with updated expectations.

#### Tracking (live status)
- [x] Standard success-envelope runtime helper added.
- [x] `res.json`, `res.ok`, and `res.okMeta` migrated to structured success payloads.
- [x] Runtime integration assertions updated for success envelope fields (`status`, `traceId`).

### M16-S11 follow-up slice (`res.okMeta` live runtime coverage)
#### Scope
- Add explicit runtime e2e validation for `res.okMeta(...)` so the structured success envelope `meta` branch is exercised by real HTTP request handling.

#### Build tasks
- Add integration fixture route using:
  - `req.json(...)`,
  - `res.okMeta(...)`.
- Assert deterministic runtime response contract for:
  - status line,
  - JSON content-type,
  - envelope fields including `meta`.

#### Acceptance criteria
- Runtime `res.okMeta(...)` path is covered by end-to-end HTTP integration test.
- Envelope response for `res.okMeta(...)` includes deterministic `meta` field.
- Existing runtime JSON output suite remains green.

#### Tracking (live status)
- [x] `res.okMeta(...)` runtime e2e test added and green.
- [x] Runtime response assertions include deterministic `meta` envelope checks.

### M16-S12 follow-up slice (HTTP method-mismatch `405` semantics)
#### Scope
- Improve runtime HTTP dispatch semantics so method mismatches return `405 Method Not Allowed` rather than `404 Not Found` when a route path exists.

#### Build tasks
- Update runtime route matching:
  - detect path matches with different HTTP method,
  - respond with `405` and deterministic body.
- Extend status-text mapping with `405 Method Not Allowed`.
- Add runtime e2e coverage for `GET` request against `POST`-only route.

#### Acceptance criteria
- Path match + method mismatch yields deterministic `405` response.
- Not-found path behavior remains unchanged (`404`).
- Runtime e2e suite remains green.

#### Tracking (live status)
- [x] Runtime method-mismatch detection added to route dispatch.
- [x] `405 Method Not Allowed` status mapping emitted by runtime.
- [x] Runtime e2e coverage added for method-mismatch branch.

### M16-S13 follow-up slice (`405` Allow-header enrichment)
#### Scope
- Improve `405` runtime response usefulness by emitting a deterministic `Allow` header listing methods registered for the matched path.

#### Build tasks
- Extend runtime response writer to accept optional extra headers.
- Collect route methods for matched path and synthesize `Allow: <methods>` on method-mismatch branch.
- Extend `405` runtime e2e test assertions to validate `Allow` header.

#### Acceptance criteria
- `405` responses include `Allow` header when path exists.
- Existing trace/content-type/dispatch behavior remains intact.
- Runtime e2e suite remains green.

#### Tracking (live status)
- [x] Runtime response writer supports optional extra headers.
- [x] Method-mismatch branch emits deterministic `Allow` header.
- [x] Runtime 405 e2e assertion updated for `Allow` header.

### M16-S14 follow-up slice (CORS preflight runtime path)
#### Scope
- Implement minimal runtime CORS preflight behavior for middleware-enabled routers so `OPTIONS` requests are handled deterministically with security-relevant headers.

#### Build tasks
- Extend router runtime state with CORS-enabled flag set by `with_cors`.
- Intercept `OPTIONS` requests when CORS is enabled and emit:
  - `204 No Content`,
  - `Access-Control-Allow-Origin`,
  - `Access-Control-Allow-Methods`,
  - `Access-Control-Allow-Headers`,
  - `Access-Control-Max-Age`.
- Add live runtime e2e coverage for CORS preflight path.

#### Acceptance criteria
- CORS-enabled router handles `OPTIONS` preflight without route-handler execution.
- Preflight response includes deterministic CORS allow headers.
- Existing runtime HTTP routes continue to pass integration tests unchanged.

#### Tracking (live status)
- [x] `with_cors` toggles runtime CORS-enabled router state.
- [x] Runtime handles `OPTIONS` preflight with deterministic `204` + allow headers.
- [x] Runtime e2e coverage added for CORS preflight path.

### M16-S15 follow-up slice (security headers runtime injection)
#### Scope
- Activate baseline runtime behavior for `sec.withSecurityHeaders(...)` so security headers are emitted in live HTTP responses.

#### Build tasks
- Extend router runtime state with security-headers enabled flag.
- Wire `sec4_rt_with_security_headers(...)` to enable header injection for routed responses.
- Inject deterministic security headers:
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Referrer-Policy: strict-origin-when-cross-origin`
- Add e2e coverage for:
  - successful response path (`200`),
  - not-found response path (`404`).

#### Acceptance criteria
- Security-header middleware invocation changes live runtime HTTP headers.
- Security headers appear on both success and error routes.
- Existing runtime integration suites remain green.

#### Tracking (live status)
- [x] Runtime router state tracks security-header middleware enablement.
- [x] Security headers injected into success and `404` responses.
- [x] Runtime e2e coverage added for both success and not-found branches.

### M16-S16 follow-up slice (CSRF runtime gate for protected methods)
#### Scope
- Activate baseline CSRF middleware behavior in runtime with deterministic double-submit checks for protected HTTP methods.

#### Build tasks
- Extend router runtime state with CSRF-enabled flag set by `with_csrf`.
- Parse request headers (`X-CSRF-Token`, `Cookie`) and enforce token match for protected methods:
  - `POST`, `PUT`, `PATCH`, `DELETE`.
- Return deterministic structured `403` error envelope for missing or mismatched tokens.
- Add runtime e2e coverage for:
  - reject path (missing token),
  - allow path (matching token).

#### Acceptance criteria
- CSRF-enabled router blocks protected requests without valid double-submit tokens.
- Valid matching token requests continue through normal handler success path.
- Runtime integration suite remains green.

#### Tracking (live status)
- [x] Runtime router state tracks CSRF middleware enablement.
- [x] Protected-method CSRF token matching is enforced in request path.
- [x] Runtime e2e coverage added for reject and allow CSRF branches.

### M16-S17 follow-up slice (auth runtime gate for protected requests)
#### Scope
- Activate baseline auth middleware behavior in runtime by enforcing bearer-auth header checks when `auth.withAuth(...)` is enabled.

#### Build tasks
- Extend router runtime state with auth-enabled flag set by `with_auth`.
- For non-`OPTIONS` requests on auth-enabled routers:
  - require `Authorization` header,
  - require `Bearer <token>` format with non-empty token.
- Return deterministic structured `401` envelope for missing/invalid auth.
- Add runtime e2e coverage for:
  - reject path (missing auth header),
  - allow path (valid bearer header).

#### Acceptance criteria
- Auth-enabled routers reject unauthorized requests with deterministic `401`.
- Valid bearer-auth requests continue through handler path.
- Runtime integration suites remain green with auth enforcement active.

#### Tracking (live status)
- [x] Runtime router state tracks auth middleware enablement.
- [x] Runtime enforces bearer-auth header shape for non-OPTIONS requests.
- [x] Runtime e2e coverage added for auth reject and allow branches.

### M16-S18 follow-up slice (CORS allow-origin propagation on normal responses)
#### Scope
- Extend CORS runtime behavior beyond preflight by applying allow-origin header to normal routed responses when CORS middleware is enabled.

#### Build tasks
- Add CORS allow-origin header composition into standard response path (`success` and error branches).
- Keep preflight path deterministic and unchanged.
- Add e2e coverage for non-preflight success response with CORS middleware enabled.

#### Acceptance criteria
- CORS-enabled routers emit `Access-Control-Allow-Origin: *` on normal responses.
- Preflight behavior remains correct (`204` + allow headers).
- Runtime suite remains green with expanded CORS coverage.

#### Tracking (live status)
- [x] Runtime response header composition includes CORS allow-origin for normal responses.
- [x] CORS preflight path remains intact after propagation changes.
- [x] Runtime e2e coverage added for non-preflight CORS response header path.

### M16-S19 follow-up slice (env-configurable request body cap)
#### Scope
- Make runtime request-body cap configurable by environment so deployments can tighten limits without recompiling runtime.

#### Build tasks
- Introduce runtime body-cap env override:
  - `SEC4_RT_HTTP_MAX_BODY_BYTES`.
- Clamp runtime request-body capture cap with configured value when lower than compiled buffer ceiling.
- Keep deterministic `LIMIT.BODY_BYTES` error behavior on overflow.
- Add runtime e2e coverage validating env override enforcement.

#### Acceptance criteria
- Runtime honors env-configured body cap when set.
- Requests exceeding configured cap fail with deterministic `413` and standard error envelope.
- Existing runtime request-gate behavior remains green.

#### Tracking (live status)
- [x] Runtime request-body cap now reads `SEC4_RT_HTTP_MAX_BODY_BYTES`.
- [x] Limit overflow behavior remains deterministic (`LIMIT.BODY_BYTES`).
- [x] Runtime e2e coverage added for env-configured body-cap enforcement.

### M16-S20 follow-up slice (CORS allow-origin on middleware rejection paths)
#### Scope
- Lock CORS header behavior on middleware-generated rejection responses so security middleware does not accidentally drop allow-origin propagation on `401`/`403` error branches.

#### Build tasks
- Add runtime e2e coverage combining CORS middleware with auth rejection path:
  - missing auth header (`401`) should still include `Access-Control-Allow-Origin: *`.
- Add runtime e2e coverage combining CORS middleware with csrf rejection path:
  - missing csrf tokens (`403`) should still include `Access-Control-Allow-Origin: *`.
- Stabilize run-command HTTP oneshot integration timing to reduce suite-level flake under heavy test load.

#### Acceptance criteria
- Auth rejection responses under CORS middleware include deterministic allow-origin header.
- CSRF rejection responses under CORS middleware include deterministic allow-origin header.
- Full `json_output` suite remains green without run-command oneshot flake regressions.

#### Tracking (live status)
- [x] Auth reject (`401`) + CORS allow-origin runtime e2e coverage added and green.
- [x] CSRF reject (`403`) + CORS allow-origin runtime e2e coverage added and green.
- [x] Run-command HTTP oneshot e2e retry/timeout window widened for suite stability.

### M16-S21 follow-up slice (CORS allow-origin on 404/405 branches)
#### Scope
- Extend CORS response-header coverage to all deterministic runtime error branches, not only middleware rejections and success/preflight paths.

#### Build tasks
- Add runtime e2e test for CORS-enabled router returning `404` on missing route and assert allow-origin header.
- Add runtime e2e test for CORS-enabled router returning `405` on method mismatch and assert allow-origin + `Allow` header.
- Keep full runtime integration suite stable after branch coverage expansion.

#### Acceptance criteria
- CORS-enabled `404` responses include deterministic `Access-Control-Allow-Origin: *`.
- CORS-enabled `405` responses include deterministic `Access-Control-Allow-Origin: *`.
- Existing preflight/success/middleware CORS tests remain green.

#### Tracking (live status)
- [x] Runtime e2e coverage added for CORS-enabled `404` branch.
- [x] Runtime e2e coverage added for CORS-enabled `405` branch.
- [x] Full runtime integration suite remains green after branch-coverage expansion.

### M16-S22 follow-up slice (CORS preflight with auth/csrf composition)
#### Scope
- Guarantee CORS preflight interoperability when security middleware is composed, so `OPTIONS` requests are not blocked by auth/csrf gates.

#### Build tasks
- Add runtime e2e test for composed middleware chain:
  - `cors.withCors`,
  - `csrf.withCsrf`,
  - `auth.withAuth`.
- Send preflight `OPTIONS` request without auth/csrf tokens and assert deterministic CORS preflight response contract.
- Keep runtime suite green and deterministic after composition coverage is added.

#### Acceptance criteria
- Composed `cors + auth + csrf` router returns deterministic `204` preflight response.
- Response preserves required CORS allow headers.
- No auth/csrf rejection envelope appears on preflight path.

#### Tracking (live status)
- [x] Runtime e2e coverage added for composed `cors + auth + csrf` preflight path.
- [x] Preflight request remains deterministic `204` with expected CORS allow headers.
- [x] Runtime integration suite remains green after middleware-composition coverage expansion.

### M16-S23 follow-up slice (security headers on auth/csrf rejection paths)
#### Scope
- Extend security-header guarantees from success/not-found branches to middleware rejection branches so sensitive errors keep the same baseline hardening headers.

#### Build tasks
- Add runtime e2e for auth rejection with security headers enabled (`401`).
- Add runtime e2e for csrf rejection with security headers enabled (`403`).
- Assert deterministic presence of:
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Referrer-Policy: strict-origin-when-cross-origin`
- Keep runtime suites green after coverage expansion.

#### Acceptance criteria
- Auth rejection responses include deterministic security headers.
- CSRF rejection responses include deterministic security headers.
- Existing security-header success/not-found coverage remains green.

#### Tracking (live status)
- [x] Runtime e2e coverage added for security-headers + auth rejection (`401`) path.
- [x] Runtime e2e coverage added for security-headers + csrf rejection (`403`) path.
- [x] Runtime integration suites remain green after rejection-branch coverage expansion.

### M16-S24 follow-up slice (security headers on 405 + preflight branches)
#### Scope
- Ensure security-header middleware behavior remains consistent for dispatch (`405`) and preflight (`204`) branches, not only success/not-found/middleware-reject paths.

#### Build tasks
- Add runtime e2e test for security-headers middleware on `405` response path.
- Add runtime e2e test for composed security-headers + CORS middleware on preflight `OPTIONS` path.
- Assert deterministic security headers across both branches.
- Keep full runtime suite green after branch coverage expansion.

#### Acceptance criteria
- `405` responses include deterministic security headers when security middleware is enabled.
- CORS preflight `204` responses include deterministic security headers when security middleware is enabled.
- Existing security-header coverage remains green across previous branches.

#### Tracking (live status)
- [x] Runtime e2e coverage added for security-headers + `405` branch.
- [x] Runtime e2e coverage added for security-headers + preflight `204` branch.
- [x] Runtime integration suites remain green after dispatch/preflight coverage expansion.

### M16-S25 follow-up slice (operator end-to-end smoke script)
#### Scope
- Add an operator-facing smoke script that proves `sec4 run` can serve a real Untrusted<T> HTTP service end-to-end outside Rust test harnesses.

#### Build tasks
- Add a standalone script that:
  - clones `examples/hello-api` into a temporary workspace,
  - patches `http.serve(...)` to a free port for conflict-free local execution,
  - runs `sec4 run` in oneshot mode and validates `GET /health` with auth,
  - runs `sec4 run` in oneshot mode and validates `POST /users` with auth + CSRF headers and JSON envelope output.
- Keep the script deterministic and self-cleaning (`mktemp` + trap).
- Validate the script locally against real runtime execution.

#### Acceptance criteria
- `scripts/smoke-sec4-run-hello-api.sh` exits successfully when runtime execution path is healthy.
- Script fails with actionable diagnostics if runtime exits early or response contracts drift.
- No repository state is mutated by the smoke run (temporary workspace only).

#### Tracking (live status)
- [x] Operator smoke script added (`scripts/smoke-sec4-run-hello-api.sh`).
- [x] Script validates `GET /health` and `POST /users` over real `sec4 run` oneshot executions.
- [x] Script validated locally with passing end-to-end output.

### M16-S26 follow-up slice (runtime-smoke artifact contract hardening)
#### Scope
- Harden runtime-smoke CI observability by validating generated smoke artifacts before upload and locking the artifact contract with deterministic tests.

#### Build tasks
- Add `scripts/check-runtime-smoke-artifacts.sh` to validate required artifact files and response contracts.
- Add deterministic checker test coverage (`scripts/test-check-runtime-smoke-artifacts.sh`) for pass + fail paths.
- Wire runtime-smoke workflow to run artifact validation before artifact upload.
- Extend runtime-smoke workflow contract checks and closure gates (`M16-C`, `M16-D`) to include artifact-validation and artifact-checker enforcement.

#### Acceptance criteria
- Runtime-smoke workflow fails if required smoke artifacts are missing or malformed.
- Artifact checker has deterministic local pass/fail tests.
- Closure audit and naming-lock CI remain green with stricter runtime-smoke contract enforcement.

#### Tracking (live status)
- [x] Runtime-smoke artifact checker added (`scripts/check-runtime-smoke-artifacts.sh`).
- [x] Checker test added (`scripts/test-check-runtime-smoke-artifacts.sh`).
- [x] Runtime-smoke workflow now validates artifacts before upload.
- [x] Closure + naming-lock enforcement updated for stricter runtime-smoke artifact contract.

### M16-S27 follow-up slice (`sec4 run` runtime bridge flags)
#### Scope
- Improve operator ergonomics for runtime test execution by exposing common runtime serve/body-limit toggles directly on `sec4 run`.

#### Build tasks
- Extend `sec4 run` CLI shape with:
  - `--oneshot` (bridges to `SEC4_RT_HTTP_SERVE_MODE=oneshot`)
  - `--max-body-bytes <N>` (bridges to `SEC4_RT_HTTP_MAX_BODY_BYTES=<N>`)
- Keep backward compatibility for existing env-based workflows.
- Add CLI integration coverage:
  - `run --help` includes both runtime bridge flags.
  - run-command HTTP oneshot e2e path uses the new `--oneshot` flag.
  - run-command oversized request path enforces deterministic `413` using `--max-body-bytes`.

#### Acceptance criteria
- Operators can run oneshot runtime flows without setting `SEC4_RT_HTTP_SERVE_MODE` manually.
- Operators can enforce runtime request-body cap through `sec4 run --max-body-bytes`.
- Existing run-command flows remain green and backward compatible.

#### Tracking (live status)
- [x] `sec4 run` now accepts `--oneshot` and `--max-body-bytes`.
- [x] `run --help` integration coverage asserts both flags are exposed.
- [x] Run-command oneshot/runtime e2e coverage now exercises `--oneshot`.
- [x] Run-command body-limit e2e coverage now exercises `--max-body-bytes` with deterministic `413`.

### M16-S28 follow-up slice (operator smoke uses `sec4 run --oneshot`)
#### Scope
- Align operator smoke and CI runtime-smoke execution with the new explicit `sec4 run` oneshot flag to reduce env-only coupling.

#### Build tasks
- Update `scripts/smoke-sec4-run-hello-api.sh` to launch:
  - `cargo run -p sec4 -- run --path <project> --oneshot`
  while keeping timeout behavior (`SEC4_RT_HTTP_SERVE_TIMEOUT_MS`) unchanged.
- Update smoke-script contract expectations so `--oneshot` is required.
- Re-run smoke + artifact-validation flow and workflow contract tests.

#### Acceptance criteria
- Operator smoke passes with run-command oneshot flag flow.
- Smoke-script contract + guard tests pass with updated token set.
- Runtime-smoke workflow contract tests remain green after smoke-script migration.

#### Tracking (live status)
- [x] Smoke script now launches `sec4 run` with `--oneshot`.
- [x] Smoke-script contract now requires `--oneshot` token.
- [x] Smoke + artifact checker flow passes locally after migration.
- [x] Runtime-smoke workflow contract and guard tests remain green.

### M16-S29 follow-up slice (`sec4 run --serve-timeout-ms`)
#### Scope
- Expose runtime serve-timeout control as a first-class `sec4 run` flag and migrate operator flows to use it instead of `SEC4_RT_HTTP_SERVE_TIMEOUT_MS`.

#### Build tasks
- Extend `sec4 run` CLI shape with `--serve-timeout-ms <N>` bridging to `SEC4_RT_HTTP_SERVE_TIMEOUT_MS`.
- Update run-command integration tests to exercise timeout behavior via CLI flag.
- Update operator smoke script launch path to use `--serve-timeout-ms 12000`.
- Update smoke-script contract tokens to require timeout flag usage.

#### Acceptance criteria
- `sec4 run --help` exposes timeout bridge flag.
- Run-command e2e tests pass using `--serve-timeout-ms`.
- Smoke and runtime-smoke contract paths stay green after timeout-flag migration.

#### Tracking (live status)
- [x] `sec4 run` now accepts `--serve-timeout-ms`.
- [x] `run --help` integration coverage asserts timeout flag exposure.
- [x] Run-command oneshot/body-limit e2e tests now pass timeout through `--serve-timeout-ms`.
- [x] Smoke script now launches `sec4 run` with `--serve-timeout-ms 12000`.
- [x] Smoke/runtime-smoke contract suites remain green after migration.

### M16-S30 follow-up slice (`sec4 run --port`)
#### Scope
- Remove operator dependence on source-level port patching by exposing runtime port override on `sec4 run`.

#### Build tasks
- Extend `sec4 run` with `--port <N>` and bridge to `SEC4_RT_HTTP_PORT`.
- Update runtime HTTP serve path to honor `SEC4_RT_HTTP_PORT` when present.
- Add run-command e2e coverage proving `--port` overrides source `http.serve(...)` port.
- Migrate smoke script to pass `--port \"${port}\"` and remove source patching logic.
- Update smoke-script contract tokens to lock `--port` usage.

#### Acceptance criteria
- `sec4 run --help` exposes `--port`.
- Runtime serves on override port provided by run-command flag.
- Smoke script passes on random free ports without mutating source files.

#### Tracking (live status)
- [x] `sec4 run` now accepts `--port`.
- [x] Runtime `http.serve` now honors `SEC4_RT_HTTP_PORT` override.
- [x] Run-command port-override e2e test added and green.
- [x] Smoke script now uses `--port` and no longer patches source port.
- [x] Smoke/runtime-smoke contracts and artifact checks remain green.

### M16-S31 follow-up slice (run-command flag contract CI checks)
#### Scope
- Add lightweight script-based CI contract checks for `sec4 run` runtime-flag bridge wiring so drift is caught in naming-lock workflow without depending on Rust test execution paths.

#### Build tasks
- Add `scripts/test-sec4-run-runtime-flag-contract.sh` to enforce source-level run flag bridge patterns.
- Add `scripts/test-sec4-run-runtime-flag-contract-guard.sh` with deterministic pass/fail fixture coverage.
- Wire both scripts into `.github/workflows/naming-lock.yml`.
- Validate local script execution before closure/doc updates.

#### Acceptance criteria
- Contract script passes against current CLI source and fails on representative drift fixtures.
- Naming-lock workflow includes both new contract and guard checks.
- Existing closure/roadmap alignment checks remain green after workflow expansion.

#### Tracking (live status)
- [x] Added `scripts/test-sec4-run-runtime-flag-contract.sh`.
- [x] Added `scripts/test-sec4-run-runtime-flag-contract-guard.sh`.
- [x] Naming-lock workflow executes both run-flag contract scripts.
- [x] Local contract, guard, and closure-alignment checks pass.

### M16-S32 follow-up slice (runtime-smoke metadata contract expansion)
#### Scope
- Strengthen runtime-smoke artifact metadata so operator/debug context captures which run-command profile produced the artifact set.

#### Build tasks
- Extend `run-metadata.txt` emitted by smoke script with deterministic fields:
  - `oneshot=true`
  - `serveTimeoutMs=12000`
  - `runFlags=--port,--oneshot,--serve-timeout-ms`
- Extend artifact checker to require those fields.
- Extend checker regression test with a failing metadata fixture.
- Re-run smoke + checker flow to validate real artifact output.

#### Acceptance criteria
- Generated runtime-smoke metadata includes deterministic run-profile fields.
- Checker fails on missing required metadata fields.
- Smoke/checker and workflow contract suites remain green.

#### Tracking (live status)
- [x] Smoke script now writes deterministic run-profile metadata fields.
- [x] Artifact checker now validates oneshot/timeout/runFlags metadata fields.
- [x] Checker regression suite includes missing-oneshot metadata failure case.
- [x] Real smoke artifact generation passes with expanded metadata contract.

### M16-S33 follow-up slice (smoke metadata token contract lock)
#### Scope
- Lock metadata emission surface directly in smoke-script contract checks so run-metadata field drift is caught before artifact-checker execution.

#### Build tasks
- Extend `scripts/test-smoke-sec4-run-hello-api-script-contract.sh` required tokens with:
  - `oneshot=true`
  - `serveTimeoutMs=12000`
  - `runFlags=--port,--oneshot,--serve-timeout-ms`
- Re-run smoke-script contract + guard tests and real smoke+artifact checker flow.

#### Acceptance criteria
- Smoke-script contract fails when deterministic metadata emit tokens are removed.
- Existing smoke + checker runtime flow remains green.

#### Tracking (live status)
- [x] Smoke-script contract now requires deterministic metadata emit tokens.
- [x] Smoke-script contract and guard tests pass after token expansion.
- [x] Real smoke + checker flow remains green.

### M16-S34 follow-up slice (smoke metadata guard coverage expansion)
#### Scope
- Ensure smoke-script guard behavior explicitly covers metadata token regressions, not only HTTP-request token regressions.

#### Build tasks
- Extend `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh` with a dedicated metadata drift scenario:
  - mutate `runFlags=--port,--oneshot,--serve-timeout-ms`
  - assert deterministic missing-token diagnostic.
- Keep existing CSRF-token guard scenario intact.

#### Acceptance criteria
- Guard script fails when metadata token is removed from smoke script fixture.
- Guard script reports deterministic missing-token message for metadata drift.

#### Tracking (live status)
- [x] Added metadata-token drift scenario to smoke contract guard script.
- [x] Guard script now validates both request-token and metadata-token drift.
- [x] Smoke contract + guard tests pass after guard expansion.

### M16-S35 follow-up slice (artifact-checker runFlags regression fixture)
#### Scope
- Expand artifact-checker regression coverage to explicitly validate missing `runFlags` metadata failures.

#### Build tasks
- Add `bad-runflags` fixture path in `scripts/test-check-runtime-smoke-artifacts.sh`.
- Assert deterministic checker failure message for missing runFlags metadata.
- Re-run real smoke + checker flow to keep end-to-end artifact contract green.

#### Acceptance criteria
- Checker regression script fails deterministically when runFlags metadata is absent.
- Real smoke artifact flow still passes expanded checker contract.

#### Tracking (live status)
- [x] Added missing-runFlags metadata regression fixture in checker test.
- [x] Checker regression test now asserts deterministic runFlags-missing diagnostic.
- [x] Real smoke + checker flow remains green.

### M16-S36 follow-up slice (closure gate for run-flag CI contracts)
#### Scope
- Promote run-command runtime-flag CI contract enforcement into strict closure auditing so milestone status reflects the new naming-lock guardrail.

#### Build tasks
- Extend `scripts/check-milestone-closure.sh` with gate `M16-E`:
  - detect naming-lock workflow wiring for:
    - `scripts/test-sec4-run-runtime-flag-contract.sh`
    - `scripts/test-sec4-run-runtime-flag-contract-guard.sh`
- Update `scripts/test-check-milestone-closure.sh` fixtures + expected gate ordering for `M16-E`.
- Update roadmap strict-closure table/interpreation to include `M16-E`.

#### Acceptance criteria
- Strict closure audit passes with `M16-E` on current repo state.
- Closure harness tests pass with deterministic `M16-E` gate ordering.
- Roadmap closure-gate alignment test remains green.

#### Tracking (live status)
- [x] Added `M16-E` gate checks to closure-audit script.
- [x] Updated closure test harness fixtures + expected ordering with `M16-E`.
- [x] Roadmap strict-closure table/interpreation now includes `M16-E`.
- [x] Closure + alignment + naming/path guards remain green.

### M16-S37 follow-up slice (source/work metadata checker fields)
#### Scope
- Expand runtime-smoke metadata validation to require source/work project provenance keys in `run-metadata.txt`.

#### Build tasks
- Extend `scripts/check-runtime-smoke-artifacts.sh` to require:
  - `sourceProject=...`
  - `workProject=...`
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with a missing-sourceProject regression fixture and deterministic diagnostic assertion.
- Re-run smoke + checker flow against real generated artifacts.

#### Acceptance criteria
- Checker fails when `sourceProject` metadata is missing.
- Checker enforces `workProject` field presence.
- Real smoke artifact flow remains green with expanded metadata checks.

#### Tracking (live status)
- [x] Checker now validates `sourceProject` and `workProject` metadata fields.
- [x] Checker regression suite includes missing-sourceProject failure fixture.
- [x] Real smoke + checker flow passes after metadata check expansion.

### M16-S38 follow-up slice (artifact-checker missing workProject regression fixture)
#### Scope
- Expand artifact-checker regression coverage to validate deterministic failure when `workProject` metadata is missing.

#### Build tasks
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with a `bad-work` fixture lacking `workProject`.
- Assert deterministic checker diagnostic:
  - `run-metadata.txt missing workProject field`
- Re-run smoke + checker flow to confirm artifact contract remains green.

#### Acceptance criteria
- Checker regression suite fails deterministically when `workProject` metadata is absent.
- Real smoke artifact flow still passes after regression expansion.

#### Tracking (live status)
- [x] Added missing-workProject metadata regression fixture in checker tests.
- [x] Checker regression test now asserts deterministic workProject-missing diagnostic.
- [x] Real smoke + checker flow remains green.

### M16-S39 follow-up slice (smoke-script source/work metadata token contract lock)
#### Scope
- Expand smoke-script static contract coverage so metadata emit tokens for source/work provenance are contract-locked in CI guard tests.

#### Build tasks
- Extend `scripts/test-smoke-sec4-run-hello-api-script-contract.sh` required token list with:
  - `sourceProject=${source_project}`
  - `workProject=${work_project}`
- Extend `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh` with dedicated negative fixtures for missing source/work metadata tokens.
- Re-run smoke-script contract + guard tests and keep broader runtime-smoke validation green.

#### Acceptance criteria
- Smoke-script contract fails when either source/work metadata token is removed.
- Guard script asserts deterministic missing-token diagnostics for both source/work metadata tokens.
- Existing runFlags metadata guard scenario remains green.

#### Tracking (live status)
- [x] Smoke-script contract now requires source/work metadata emit tokens.
- [x] Guard coverage includes deterministic missing-token scenarios for both metadata tokens.
- [x] Smoke-script contract + guard tests pass after provenance-token lock expansion.

### M16-S40 follow-up slice (artifact-checker empty source/work metadata regression fixtures)
#### Scope
- Expand runtime-smoke checker regression coverage to lock non-empty enforcement for `sourceProject` and `workProject` metadata values.

#### Build tasks
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with:
  - `bad-source-empty` fixture (`sourceProject=`)
  - `bad-work-empty` fixture (`workProject=`)
- Assert deterministic checker diagnostics for both empty-value failures.
- Re-run checker regression suite plus real smoke/checker flow.

#### Acceptance criteria
- Checker regression suite fails deterministically for empty `sourceProject` and empty `workProject` values.
- Deterministic diagnostics match existing missing-field message contracts.
- Real smoke artifact flow still passes unchanged.

#### Tracking (live status)
- [x] Added empty-source and empty-work metadata regression fixtures in checker tests.
- [x] Checker regression test now asserts deterministic diagnostics for both empty-value cases.
- [x] Real smoke + checker flow remains green after non-empty regression expansion.

### M16-S41 follow-up slice (artifact-checker source/work divergence enforcement)
#### Scope
- Harden runtime-smoke metadata validation so provenance keys cannot collapse into a single path value.

#### Build tasks
- Extend `scripts/check-runtime-smoke-artifacts.sh` to assert:
  - `sourceProject != workProject`
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with `bad-same-project` fixture where source/work values are identical.
- Assert deterministic checker failure diagnostic for identical provenance values.

#### Acceptance criteria
- Checker fails when `sourceProject` and `workProject` values are identical.
- Regression suite asserts deterministic divergence diagnostic.
- Real smoke artifact flow continues passing with distinct source/work values.

#### Tracking (live status)
- [x] Checker now enforces non-identical source/work metadata values.
- [x] Regression suite includes identical-source/work failure fixture.
- [x] Real smoke + checker flow remains green after divergence enforcement.

### M16-S42 follow-up slice (runtime-smoke success-envelope timeMs contract hardening)
#### Scope
- Tighten runtime-smoke response contract checks so `users.body` must include numeric `timeMs` in the standard success envelope.

#### Build tasks
- Extend `scripts/check-runtime-smoke-artifacts.sh` success-envelope jq contract with:
  - `(.timeMs | type == "number")`
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with malformed `users.body` fixture missing `timeMs`.
- Assert deterministic envelope-contract diagnostic and rerun real smoke/checker flow.

#### Acceptance criteria
- Checker rejects `users.body` envelopes missing numeric `timeMs`.
- Regression suite includes deterministic malformed-envelope failure.
- Real smoke artifact flow remains green.

#### Tracking (live status)
- [x] Success-envelope checker now requires numeric `timeMs` in `users.body`.
- [x] Regression suite includes malformed-envelope fixture missing `timeMs`.
- [x] Real smoke + checker flow remains green after envelope contract hardening.

### M16-S43 follow-up slice (runtime-smoke success-envelope traceId format hardening)
#### Scope
- Tighten runtime-smoke success-envelope validation so `traceId` matches deterministic runtime shape.

#### Build tasks
- Extend `scripts/check-runtime-smoke-artifacts.sh` jq contract to require:
  - `traceId` pattern `^rt-[0-9]+$`
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with malformed-trace fixture.
- Assert deterministic envelope-contract failure and re-run real smoke/checker flow.

#### Acceptance criteria
- Checker rejects success envelopes with malformed traceId format.
- Regression suite includes deterministic malformed-trace failure.
- Real smoke artifact flow remains green with runtime trace ids.

#### Tracking (live status)
- [x] Success-envelope checker now enforces `rt-[0-9]+` traceId format.
- [x] Regression suite includes malformed traceId fixture.
- [x] Real smoke + checker flow remains green after traceId contract hardening.

### M16-S44 follow-up slice (runtime-smoke users trace header/body correlation enforcement)
#### Scope
- Harden runtime-smoke artifact checks so users response headers/body carry consistent trace correlation values.

#### Build tasks
- Extend `scripts/check-runtime-smoke-artifacts.sh` to require:
  - `users.headers` contains `X-Trace-Id`
  - `users.headers` trace id equals `users.body.traceId`
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with:
  - missing-users-trace-header fixture
  - users-trace-mismatch fixture
- Re-run checker regression suite and real smoke/checker flow.

#### Acceptance criteria
- Checker fails when users trace header is missing.
- Checker fails when users header/body trace ids mismatch.
- Real smoke artifact flow remains green.

#### Tracking (live status)
- [x] Checker now requires users `X-Trace-Id` response header.
- [x] Checker now enforces users header/body trace-id equality.
- [x] Regression suite includes missing-header and mismatch fixtures.
- [x] Real smoke + checker flow remains green after correlation hardening.

### M16-S45 follow-up slice (runtime-smoke health trace-header contract hardening)
#### Scope
- Expand runtime-smoke trace-correlation checks to include `/health` response headers.

#### Build tasks
- Extend `scripts/check-runtime-smoke-artifacts.sh` to require:
  - `health.headers` includes `X-Trace-Id`
  - health trace value matches `^rt-[0-9]+$`
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with:
  - missing-health-trace-header fixture
  - malformed-health-trace-header fixture
- Re-run checker regression suite and real smoke/checker flow.

#### Acceptance criteria
- Checker fails when health trace header is missing.
- Checker fails when health trace header format is malformed.
- Real smoke artifact flow remains green.

#### Tracking (live status)
- [x] Checker now requires health `X-Trace-Id` response header.
- [x] Checker now enforces health trace-header `rt-[0-9]+` format.
- [x] Regression suite includes missing and malformed health trace-header fixtures.
- [x] Real smoke + checker flow remains green after health trace hardening.

### M16-S46 follow-up slice (runtime-smoke run-log invocation flag contract hardening)
#### Scope
- Tighten runtime-smoke artifact validation so captured run logs prove expected `sec4 run` flag usage.

#### Build tasks
- Extend `scripts/check-runtime-smoke-artifacts.sh` to require invocation tokens in both run logs:
  - `--port`
  - `--oneshot`
  - `--serve-timeout-ms 12000`
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with:
  - missing `--oneshot` fixture for `health.run.log`
  - missing `--serve-timeout-ms 12000` fixture for `users.run.log`
- Re-run checker regression suite and real smoke/checker flow.

#### Acceptance criteria
- Checker fails deterministically when required invocation tokens are missing from either run log.
- Regression suite includes both health/users run-log missing-token scenarios.
- Real smoke artifact flow remains green with unchanged script invocation.

#### Tracking (live status)
- [x] Checker now enforces required invocation tokens in both run logs.
- [x] Regression suite includes deterministic missing-token fixtures for health/users run logs.
- [x] Real smoke + checker flow remains green after run-log contract hardening.

### M16-S47 follow-up slice (runtime-smoke run-log port/metadata correlation hardening)
#### Scope
- Tighten run-log validation so `--port` token values are correlated with `run-metadata.txt` port value.

#### Build tasks
- Extend `scripts/check-runtime-smoke-artifacts.sh` to:
  - parse `port` from `run-metadata.txt`
  - require exact `--port <port>` token in both `health.run.log` and `users.run.log`
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with a metadata/log port mismatch fixture.
- Re-run checker regression suite and real smoke/checker flow.

#### Acceptance criteria
- Checker fails when run logs contain a different `--port` value than metadata port.
- Regression suite includes deterministic mismatch diagnostic.
- Real smoke artifact flow remains green.

#### Tracking (live status)
- [x] Checker now enforces exact metadata-correlated `--port` token in both run logs.
- [x] Regression suite includes run-log/metadata port mismatch fixture.
- [x] Real smoke + checker flow remains green after port-correlation hardening.

### M16-S48 follow-up slice (runtime-smoke run-log timeout/metadata correlation hardening)
#### Scope
- Tighten run-log validation so timeout invocation tokens are correlated with metadata timeout values.

#### Build tasks
- Extend `scripts/check-runtime-smoke-artifacts.sh` to:
  - parse `serveTimeoutMs` from metadata
  - require exact `--serve-timeout-ms <serveTimeoutMs>` token in both run logs
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with metadata/log timeout mismatch fixture.
- Re-run checker regression suite and real smoke/checker flow.

#### Acceptance criteria
- Checker fails when run logs contain a timeout value different from metadata `serveTimeoutMs`.
- Regression suite includes deterministic mismatch diagnostic.
- Real smoke artifact flow remains green.

#### Tracking (live status)
- [x] Checker now enforces exact metadata-correlated timeout token in both run logs.
- [x] Regression suite includes run-log/metadata timeout mismatch fixture.
- [x] Real smoke + checker flow remains green after timeout-correlation hardening.

### M16-S49 follow-up slice (operator smoke timeout override + variable-token contract lock)
#### Scope
- Extend operator smoke script with timeout override support while preserving deterministic contract checks.

#### Build tasks
- Extend `scripts/smoke-sec4-run-hello-api.sh`:
  - add `--serve-timeout-ms <ms>` argument (default `12000`)
  - validate positive integer input
  - propagate value into runtime invocation + metadata output
- Update smoke-script contract/guard tests:
  - lock variable-based timeout tokens (`${serve_timeout_ms}`) in invocation + metadata emit blocks
  - add guard fixtures for removed timeout flag token and metadata token
- Re-run checker regression + real smoke/checker flows for default and override timeout values.

#### Acceptance criteria
- Smoke script accepts timeout override and continues passing checker validation.
- Contract/guard tests enforce timeout token presence in both invocation and metadata emit paths.
- Default timeout behavior remains unchanged.

#### Tracking (live status)
- [x] Smoke script now accepts `--serve-timeout-ms` with default `12000`.
- [x] Contract/guard coverage now locks variable-based timeout flag + metadata tokens.
- [x] Default and override smoke flows both pass checker validation.

### M16-S50 follow-up slice (runtime-smoke metadata numeric-bound contract hardening)
#### Scope
- Tighten runtime-smoke metadata validation by enforcing numeric bounds on port and timeout fields.

#### Build tasks
- Extend `scripts/check-runtime-smoke-artifacts.sh` to enforce:
  - `port` range `1..65535`
  - `serveTimeoutMs > 0`
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with:
  - out-of-range port fixture (`port=0`)
  - non-positive timeout fixture (`serveTimeoutMs=0`)
- Re-run checker regression suite and real smoke/checker flows.

#### Acceptance criteria
- Checker fails deterministically for out-of-range metadata port values.
- Checker fails deterministically for non-positive metadata timeout values.
- Default and timeout-override smoke flows remain green.

#### Tracking (live status)
- [x] Checker now enforces port range bounds (`1..65535`).
- [x] Checker now enforces positive timeout bounds (`serveTimeoutMs > 0`).
- [x] Regression suite includes dedicated port/timeout range fixtures.
- [x] Default and override smoke flows remain green after metadata bound hardening.

### M16-S51 follow-up slice (operator smoke max-body override + maxBody metadata/log contracts)
#### Scope
- Extend operator smoke script with max-body override support and tighten checker contracts around `maxBodyBytes` metadata/log consistency.

#### Build tasks
- Extend `scripts/smoke-sec4-run-hello-api.sh`:
  - add `--max-body-bytes <bytes>` argument (optional)
  - validate positive integer input
  - emit `maxBodyBytes=${max_body_bytes:-unset}` metadata
  - append `--max-body-bytes <bytes>` to invocation when set
- Extend smoke-script contract/guard tests to lock:
  - `max_body_bytes` variable tokens
  - max-body invocation and metadata emit tokens
- Extend checker + checker tests to enforce:
  - `maxBodyBytes` metadata field presence/shape
  - positive bound when numeric
  - run-log token correlation when numeric
- Re-run default, timeout-override, and max-body-override smoke/checker flows.

#### Acceptance criteria
- Smoke script supports optional max-body override with deterministic metadata emission.
- Checker fails for missing/invalid/non-positive `maxBodyBytes` metadata.
- Checker fails when max-body metadata and run logs diverge.
- Default and override smoke flows remain green.

#### Tracking (live status)
- [x] Smoke script now supports `--max-body-bytes` with validated positive integer input.
- [x] Smoke-script contract/guard checks now lock max-body variable/invocation/metadata tokens.
- [x] Checker now enforces `maxBodyBytes` metadata shape and optional log-correlation checks.
- [x] Default, timeout-override, and max-body-override smoke flows remain green.

### M16-S52 follow-up slice (runtime-smoke users-branch max-body correlation regression fixture)
#### Scope
- Expand checker regression coverage so max-body correlation checks are exercised on both health and users run-log branches.

#### Build tasks
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with fixture that:
  - sets `maxBodyBytes=2048`,
  - gives `health.run.log` a matching max-body token,
  - keeps `users.run.log` without max-body token.
- Assert deterministic failure:
  - `users.run.log missing --max-body-bytes 2048 invocation token`

#### Acceptance criteria
- Regression suite explicitly validates users-branch max-body mismatch behavior.
- Deterministic users-branch mismatch diagnostic is pinned.
- Existing checker + smoke flows remain green.

#### Tracking (live status)
- [x] Added dedicated users-branch max-body mismatch fixture.
- [x] Regression suite now asserts deterministic users-branch mismatch diagnostic.
- [x] Checker regression suite remains green after coverage expansion.

### M16-S53 follow-up slice (runtime-smoke invalid-shape maxBody metadata regression fixture)
#### Scope
- Expand maxBody regression matrix to include explicit invalid-shape metadata values (`abc`), not only missing/range/log-correlation cases.

#### Build tasks
- Extend `scripts/test-check-runtime-smoke-artifacts.sh` with `maxBodyBytes=abc` fixture.
- Assert deterministic checker diagnostic:
  - `run-metadata.txt missing or invalid maxBodyBytes field`
- Re-run checker regression suite.

#### Acceptance criteria
- Checker regression suite fails deterministically for invalid-shape `maxBodyBytes` metadata.
- Deterministic invalid-shape diagnostic is pinned.
- Existing smoke/checker flows remain green.

#### Tracking (live status)
- [x] Added invalid-shape `maxBodyBytes=abc` regression fixture.
- [x] Regression suite now asserts deterministic invalid-shape diagnostic.
- [x] Checker regression suite remains green after invalid-shape coverage expansion.

### M16-S54 follow-up slice (runtime-smoke shape-aware runFlags metadata contract)
#### Scope
- Make `runFlags` metadata branch-aware so optional `--max-body-bytes` appears only when `maxBodyBytes` is configured, and enforce that shape in checker contracts.

#### Build tasks
- Update `scripts/smoke-sec4-run-hello-api.sh` to emit `runFlags` from a computed variable:
  - baseline: `--port,--oneshot,--serve-timeout-ms`
  - append `,--max-body-bytes` when `--max-body-bytes` is set.
- Extend `scripts/check-runtime-smoke-artifacts.sh` to validate:
  - `runFlags` field exists,
  - value equals expected shape for the current `maxBodyBytes` branch (`unset` vs numeric).
- Expand checker regression fixtures for both branch-shape drifts:
  - `maxBodyBytes=unset` with extra max-body flag in `runFlags` (must fail),
  - `maxBodyBytes=<N>` without max-body flag in `runFlags` (must fail).
- Keep existing max-body log-correlation fixtures aligned with new shape contract.

#### Acceptance criteria
- Smoke metadata emits deterministic `runFlags` shape matching optional max-body usage.
- Checker fails deterministically on both branch-shape drift fixtures with a stable diagnostic.
- Existing smoke/checker contract suites remain green.

#### Tracking (live status)
- [x] Smoke script now emits computed shape-aware `runFlags` metadata.
- [x] Checker now enforces deterministic `runFlags` shape by max-body metadata branch.
- [x] Regression matrix now covers unset/set max-body runFlags-shape drift fixtures.
- [x] Existing contract and smoke suites remain green after shape-aware contract hardening.

### M16-S55 follow-up slice (runtime-smoke workflow dual-branch maxBody coverage)
#### Scope
- Ensure CI continuously executes and validates both runtime-smoke metadata branches:
  - default run (`maxBodyBytes=unset`)
  - explicit max-body run (`--max-body-bytes 2048`)

#### Build tasks
- Update `.github/workflows/runtime-smoke.yml` to run:
  - smoke + checker on `build/runtime-smoke/default`
  - smoke + checker on `build/runtime-smoke/max-body` with `--max-body-bytes 2048`
- Update workflow contract + guard scripts to lock both branch commands.
- Update closure-audit workflow fixture/contract checks to require both branch commands.

#### Acceptance criteria
- Runtime-smoke workflow contract fails if either branch run/check step is removed.
- Closure audit remains `PASS` with stricter dual-branch workflow contract.
- Naming-lock CI and closure fixture tests remain green.

#### Tracking (live status)
- [x] Runtime-smoke workflow now executes default and max-body smoke/check branches.
- [x] Workflow contract + guard tests now lock both branch command tokens.
- [x] Closure audit fixture/contracts now include both branch command tokens.
- [x] Naming-lock + closure checks remain green with stricter workflow contract.

### M16-S56 follow-up slice (runtime-smoke aggregated branch artifact index)
#### Scope
- Add deterministic aggregated indexing for runtime-smoke artifacts so CI evidence consumers can inspect both branch outputs (`default`, `max-body`) from one machine-readable file.

#### Build tasks
- Add `scripts/build-runtime-smoke-branch-index.sh`:
  - consumes `build/runtime-smoke/{default,max-body}` artifacts,
  - validates branch profile invariants (`maxBodyBytes` + `runFlags` shape),
  - emits `build/runtime-smoke/runtime-smoke-branch-index.json`.
- Add deterministic regression coverage:
  - `scripts/test-build-runtime-smoke-branch-index.sh` with pass + branch-shape/missing-branch failures.
- Update `runtime-smoke` workflow to build branch index before artifact upload.
- Update workflow contract/guard + closure fixture checks to require index-builder step.
- Update naming-lock CI to execute branch-index builder regression test.

#### Acceptance criteria
- Runtime-smoke artifact bundle includes machine-readable branch index for both profiles.
- Contract tests fail deterministically when index step is missing or branch profiles drift.
- Closure audit remains green with stricter runtime-smoke evidence contract.

#### Tracking (live status)
- [x] Branch-index builder script implemented with deterministic profile validations.
- [x] Branch-index regression suite added and passing.
- [x] Runtime-smoke workflow now emits `runtime-smoke-branch-index.json`.
- [x] Workflow contract/guard + closure fixtures lock index-builder step.
- [x] Naming-lock CI now runs branch-index regression test.

### M16-S57 follow-up slice (runtime-smoke single-pass bundle checker contract)
#### Scope
- Consolidate runtime-smoke validation/indexing into one deterministic command so workflow and operators run one contract-locked checker pass after dual-branch smoke execution.

#### Build tasks
- Add `scripts/check-runtime-smoke-bundle.sh`:
  - validates `default` + `max-body` branch directories exist,
  - runs per-branch artifact checker for each branch,
  - builds/validates aggregated branch index contract.
- Add `scripts/test-check-runtime-smoke-bundle.sh` with deterministic pass/failure fixtures.
- Migrate `.github/workflows/runtime-smoke.yml` to call bundle checker instead of separate per-branch checker/index commands.
- Update runtime-smoke workflow contract/guard tests and closure fixtures for bundle-check command token.
- Update naming-lock CI and closure guard expectations to include bundle-checker test.

#### Acceptance criteria
- Runtime-smoke workflow uses one bundle-check command for branch artifact validation + index generation.
- Bundle-check regression tests fail deterministically for missing branch and profile-drift cases.
- Closure and naming-lock checks remain green after workflow contract migration.

#### Tracking (live status)
- [x] Bundle checker implemented (`scripts/check-runtime-smoke-bundle.sh`).
- [x] Bundle checker regression suite added and passing.
- [x] Runtime-smoke workflow now invokes bundle checker step.
- [x] Workflow contract/guard + closure fixture checks now lock bundle-check token.
- [x] Naming-lock CI now runs bundle-checker regression test.

### Docs/book outputs
- Chapter: "M16 Slice: Live HTTP Runtime Serve Bootstrap".
- Chapter: "M16 Slice: JSON Response Materialization for Runtime Routes".
- Chapter: "M16 Slice: Runtime Status Propagation for Success Responses".
- Chapter: "M16 Slice: Runtime Request JSON Gate for Invalid Payload Handling".
- Chapter: "M16 Slice: Runtime req.json Content-Type Gate Enforcement".
- Chapter: "M16 Slice: sec4 run Live HTTP Command-Path Validation".
- Chapter: "M16 Slice: req.json Request-Size Guard Enforcement".
- Chapter: "M16 Slice: req.json Standard Error Envelope Alignment".
- Chapter: "M16 Slice: Runtime Trace Correlation Header and Error Envelope Sync".
- Chapter: "M16 Slice: Runtime Standard Success Envelope Alignment".
- Chapter: "M16 Slice: res.okMeta Runtime Envelope Coverage".
- Chapter: "M16 Slice: HTTP Method-Mismatch 405 Dispatch Semantics".
- Chapter: "M16 Slice: 405 Allow Header Enrichment".
- Chapter: "M16 Slice: CORS Preflight Runtime Handling".
- Chapter: "M16 Slice: Security Headers Runtime Injection".
- Chapter: "M16 Slice: CSRF Runtime Gate Enforcement".
- Chapter: "M16 Slice: Auth Runtime Gate Enforcement".
- Chapter: "M16 Slice: CORS Allow-Origin Propagation on Runtime Responses".
- Chapter: "M16 Slice: Env-Configurable Runtime Request Body Cap".
- Chapter: "M16 Slice: CORS Allow-Origin Coverage on Middleware Rejection Paths".
- Chapter: "M16 Slice: CORS Allow-Origin Coverage on 404/405 Error Branches".
- Chapter: "M16 Slice: CORS Preflight Interoperability with Auth + CSRF Middleware".
- Chapter: "M16 Slice: Security Headers Coverage on Auth/CSRF Rejection Paths".
- Chapter: "M16 Slice: Security Headers Coverage on 405 and Preflight Branches".
- Chapter: "M16 Slice: sec4 run Hello-API Operator Smoke Script".
- Chapter: "M16 Slice: Runtime-Smoke Workflow Closure Gates".
- Chapter: "M16 Slice: Runtime-Smoke Artifact Checker".
- Chapter: "M16 Slice: sec4 run Runtime Bridge Flags".
- Chapter: "M16 Slice: Operator Smoke Script Migration to sec4 run --oneshot".
- Chapter: "M16 Slice: sec4 run Serve-Timeout Bridge Flag".
- Chapter: "M16 Slice: sec4 run Port Override Bridge and Smoke Port Migration".
- Chapter: "M16 Slice: sec4 run Runtime-Flag Contract and Naming-Lock CI Coverage".
- Chapter: "M16 Slice: Runtime-Smoke Metadata Contract Expansion".
- Chapter: "M16 Slice: Smoke-Script Metadata Token Contract Lock".
- Chapter: "M16 Slice: Smoke-Script Metadata Guard Coverage Expansion".
- Chapter: "M16 Slice: Runtime-Smoke Checker runFlags Regression Fixture".
- Chapter: "M16 Slice: Closure Gate for sec4 run Runtime-Flag CI Contracts".
- Chapter: "M16 Slice: Runtime-Smoke Source/Work Metadata Field Checks".
- Chapter: "M16 Slice: Runtime-Smoke Checker workProject Regression Fixture".
- Chapter: "M16 Slice: Smoke-Script Source/Work Metadata Token Contract Lock".
- Chapter: "M16 Slice: Runtime-Smoke Checker Empty Source/Work Metadata Regression Fixtures".
- Chapter: "M16 Slice: Runtime-Smoke Source/Work Provenance Divergence Enforcement".
- Chapter: "M16 Slice: Runtime-Smoke Success Envelope timeMs Contract Hardening".
- Chapter: "M16 Slice: Runtime-Smoke Success Envelope traceId Format Hardening".
- Chapter: "M16 Slice: Runtime-Smoke Users Trace Header/Body Correlation Enforcement".
- Chapter: "M16 Slice: Runtime-Smoke Health Trace-Header Contract Hardening".
- Chapter: "M16 Slice: Runtime-Smoke Run-Log Invocation Flag Contract Hardening".
- Chapter: "M16 Slice: Runtime-Smoke Run-Log Port/Metadata Correlation Hardening".
- Chapter: "M16 Slice: Runtime-Smoke Run-Log Timeout/Metadata Correlation Hardening".
- Chapter: "M16 Slice: Operator Smoke Timeout Override and Variable Token Contracts".
- Chapter: "M16 Slice: Runtime-Smoke Metadata Numeric-Bound Contract Hardening".
- Chapter: "M16 Slice: Operator Smoke max-body Override and maxBody Metadata/Log Contracts".
- Chapter: "M16 Slice: Runtime-Smoke Users-Branch max-body Correlation Regression Coverage".
- Chapter: "M16 Slice: Runtime-Smoke Invalid-Shape maxBody Metadata Regression Coverage".
- Chapter: "M16 Slice: Runtime-Smoke Shape-Aware runFlags Metadata Contract Hardening".
- Chapter: "M16 Slice: Runtime-Smoke Workflow Dual-Branch maxBody Coverage".
- Chapter: "M16 Slice: Runtime-Smoke Aggregated Branch Artifact Index".
- Chapter: "M16 Slice: Runtime-Smoke Single-Pass Bundle Checker Contract".

## M17 - End-to-End Server Packaging and Operator Handoff
### Trigger condition
- Start after M16 closure gates remain stable across at least one full CI cycle with dual-branch runtime-smoke coverage.

### Scope decision (M17 kickoff)
- Primary scope: package the working runtime/server flow into a clear operator handoff track with deterministic bootstrap and verification commands.
- Included tracks:
  - one-command service bootstrap profile definitions,
  - operator smoke profile matrix (`default`, `max-body`, timeout override),
  - handoff checklist for local + CI verification and artifact inspection.
- Deferred:
  - broader productization/release channel automation beyond alpha hardening,
  - non-essential runtime feature expansion unrelated to operator bootstrap reliability.

### Build tasks
- Define canonical operator bootstrap commands and expected outputs for:
  - `sec4 run` oneshot profile,
  - dual-branch runtime-smoke bundle profile,
  - trend-note local fallback update profile.
- Add a deterministic operator handoff checklist artifact (docs + script references).
- Align docs/book quickstart paths with current runtime-smoke bundle workflow contracts.
- Add a small readiness script or checklist verifier that confirms all required scripts/workflows exist for handoff.

### M17-S1 kickoff acceptance criteria
- M17 kickoff scope and first-slice contract are documented in roadmap + book chapter.
- Operator handoff checklist includes exact commands and expected artifact outputs.
- No existing closure gates regress while adding planning artifacts.

### M17-S1 tracking (live status)
- [x] Kickoff scope defined in roadmap.
- [x] Kickoff chapter stub added to docs/book.
- [x] Operator handoff checklist chapter added with canonical command matrix and expected outputs.
- [x] M17 readiness verifier script + test added and wired into naming-lock CI.
- [x] Operator bootstrap profile helper script added and wired into naming-lock CI.
- [x] Operator troubleshooting matrix script added and wired into naming-lock CI.
- [x] Operator handoff quickstart script added and wired into naming-lock CI.
- [x] Operator handoff CI smoke wrapper script added and wired into naming-lock CI.
- [x] Operator-handoff workflow contract + guard tests added and wired into naming-lock CI.
- [x] Operator handoff artifact inspector script added and wired into naming-lock CI.
- [x] Operator handoff readiness summary script added and wired into naming-lock CI.
- [x] Operator release-packet builder script added and wired into naming-lock CI.
- [x] Final handoff playbook checker script + chapter added and wired into naming-lock CI.
- [x] Clean-clone rehearsal runner script + chapter added and wired into naming-lock CI.
- [x] Live clean-clone rehearsal executed and friction note recorded.

### Exit criteria
- Operator can follow a deterministic checklist and run end-to-end server/bootstrap validation without implicit tribal knowledge.
- Handoff checklist is aligned with current closure gates and CI contracts.
- M17 kickoff artifacts remain naming-lock and closure-gate compatible.

### Docs/book outputs
- Chapter: "M17 Kickoff: End-to-End Server Packaging and Handoff Scope".
- Chapter: "M17 Operator Handoff Checklist and Readiness Verifier".
- Chapter: "M17 Operator Bootstrap Profile Helper".
- Chapter: "M17 Operator Troubleshooting Matrix".
- Chapter: "M17 Operator Handoff Quickstart".
- Chapter: "M17 Operator Handoff CI Smoke Wrapper".
- Chapter: "M17 Operator Handoff Workflow Contracts".
- Chapter: "M17 Operator Handoff Artifact Inspector".
- Chapter: "M17 Operator Readiness Summary".
- Chapter: "M17 Operator Release Packet Builder".
- Chapter: "M17 Operator Handoff Final Playbook".
- Chapter: "M17 Operator Clean-Clone Rehearsal".
- Chapter: "M17 Clean-Clone Rehearsal Results Note".

## M18 - Post-Handoff Prioritization and Transition
### Trigger condition
- Start after M17 clean-clone rehearsal evidence is captured with explicit friction reporting.

### Scope decision (M18 kickoff)
- Primary scope: convert M17 rehearsal evidence into deterministic next-step planning artifacts.
- Included tracks:
  - machine-readable kickoff brief from rehearsal output,
  - stable recommendation surface for the first post-M17 slices,
  - closure-gated CI lock for kickoff artifact generation.
- Deferred:
  - large runtime/editor feature changes before kickoff priorities are explicitly locked,
  - release automation expansion beyond the current alpha lane.

### Build tasks
- Generate kickoff summary from rehearsal report (`PASS/FAIL`, failed step, friction count, recommendations).
- Support both markdown and JSON output for operator + automation consumers.
- Build deterministic runtime/editor/release priority matrix from rehearsal evidence.
- Select one closure-gated next slice from kickoff + matrix inputs.
- Execute first selected slice on the editor track (stable quickfix action IDs).
- Execute next selected release-track slice on publish-integrity contract expansion.
- Execute runtime-track slice via selector-driven runtime-runner contract.
- Build convergence summary across editor/release/runtime track outputs.
- Build deterministic transition handoff packet from kickoff/matrix/selector/convergence artifacts.
- Build final M18 closure report from `M18-A..H` + transition packet summary.
- Lock kickoff/matrix/selector/editor/release/runtime/convergence/handoff/closure-report contracts in naming-lock CI and closure audit.

### M18-S1 kickoff acceptance criteria
- Kickoff generator script + contract test exists.
- Generator validates report shape and fails deterministically on missing/invalid report.
- Closure audit includes a dedicated M18 kickoff gate.

### M18-S1 tracking (live status)
- [x] Kickoff brief generator script added.
- [x] Kickoff brief generator contract test added.
- [x] Book chapter documenting kickoff generator added.
- [x] Naming-lock CI and closure gate updated (`M18-A`).
- [x] Priority matrix artifact script added.
- [x] Priority matrix contract test added.
- [x] Book chapter documenting priority matrix artifact added.
- [x] Naming-lock CI and closure gate updated (`M18-B`).
- [x] Next-slice selector script added.
- [x] Next-slice selector contract test added.
- [x] Book chapter documenting next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M18-C`).
- [x] Editor-path quickfix action IDs added in LSP code actions.
- [x] Editor contract expansion checker added.
- [x] Book chapter documenting editor-path execution slice added.
- [x] Naming-lock CI and closure gate updated (`M18-D`).
- [x] Selector slice identifiers aligned with executed M18 editor/release/runtime tracks.
- [x] Release publish-manifest verifier now checks artifact file hashes against checksum entries.
- [x] Publish-manifest verifier regression includes tampered sample sbom artifact coverage.
- [x] M18 release publish-integrity contract checker added.
- [x] Book chapter documenting release publish-integrity contract expansion added.
- [x] Naming-lock CI and closure gate updated (`M18-E`).
- [x] Runtime-track execution runner script added (`--dry-run`, `text/json` output contracts).
- [x] Runtime-track execution runner contract test added.
- [x] Book chapter documenting runtime-track execution runner added.
- [x] Naming-lock CI and closure gate updated (`M18-F`).
- [x] Track-convergence summary script added (`markdown/json` output contracts).
- [x] Track-convergence summary contract test added.
- [x] Book chapter documenting track-convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M18-G`).
- [x] Transition handoff packet script added (`text/json` output contracts).
- [x] Transition handoff packet contract test added.
- [x] Book chapter documenting transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M18-H`).
- [x] M18 closure report script added (`markdown/json` output contracts).
- [x] M18 closure report contract test added.
- [x] Book chapter documenting M18 closure report added.
- [x] Naming-lock CI and closure gate updated (`M18-I`).

### Exit criteria
- M18 kickoff has a deterministic, reproducible artifact generated from live rehearsal evidence.
- Next milestone slices can be selected using explicit kickoff summary + ranked priority matrix + deterministic selector output.

### Docs/book outputs
- Chapter: "M18 Kickoff Brief Generator".
- Chapter: "M18 Priority Matrix Artifact".
- Chapter: "M18 Next-Slice Selector".
- Chapter: "M18 Editor Contract Expansion".
- Chapter: "M18 Release Publish-Integrity Contract Expansion".
- Chapter: "M18 Runtime-Track Execution Runner".
- Chapter: "M18 Track-Convergence Summary".
- Chapter: "M18 Transition Handoff Packet".
- Chapter: "M18 Closure Report".

## M19 - Kickoff from Closed M18 Baseline
### Trigger condition
- Start after M18 closure report (`M18-I`) is PASS and transition packet is available.

### Scope decision (M19 kickoff)
- Primary scope: use finalized M18 closure + handoff packet to define deterministic M19 start priorities.
- Included tracks:
  - M19 kickoff brief generation from M18 closure/packet artifacts,
  - closure-gated naming-lock contract for kickoff brief.
- Deferred:
  - deeper M19 implementation slices until kickoff brief is stable.

### Build tasks
- Generate M19 kickoff brief from M18 closure report + transition packet.
- Support markdown and JSON output for operator + automation consumers.
- Auto-generate M18 closure json when absent but packet is available.
- Build deterministic M19 priority matrix from kickoff brief.
- Select first executable M19 slice from kickoff + matrix inputs.
- Execute first selected runtime slice via dedicated M19 runtime hardening runner.
- Build executed-slice convergence summary from selector + runtime execution artifacts.
- Build transition handoff packet from kickoff/matrix/selector/runtime/convergence artifacts.
- Build final M19 closure report from strict closure gates + M19 transition packet summary.
- Lock kickoff/matrix/selector/runtime/convergence/transition/closure contracts in naming-lock CI and closure audit.

### M19-S1 kickoff acceptance criteria
- M19 kickoff brief script + contract test exists.
- Script validates packet/closure contract and handles missing M18 closure artifact deterministically.
- Closure audit includes dedicated `M19-A` gate.

### M19-S1 tracking (live status)
- [x] M19 kickoff brief script added.
- [x] M19 kickoff brief contract test added.
- [x] Book chapter documenting M19 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M19-A`).
- [x] M19 priority matrix script added.
- [x] M19 priority matrix contract test added.
- [x] Book chapter documenting M19 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M19-B`).
- [x] M19 next-slice selector script added.
- [x] M19 next-slice selector contract test added.
- [x] Book chapter documenting M19 next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M19-C`).

### M19-S4 first executed slice acceptance criteria
- Selector recommendations use `M19-S4-*` IDs for runtime/release/editor/stabilization tracks.
- Runtime hardening runner enforces runtime-selected `M19-S4-runtime-*` recommendations and supports dry-run/json outputs.
- Closure audit includes dedicated `M19-D` gate.

### M19-S4 tracking (live status)
- [x] M19 selector recommendations aligned to `M19-S4-*` IDs.
- [x] M19 runtime hardening runner script added.
- [x] M19 runtime hardening runner contract test added.
- [x] Book chapter documenting M19 runtime hardening runner added.
- [x] Naming-lock CI and closure gate updated (`M19-D`).

### M19-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script validates selector/runtime execution artifact contracts and recommendation alignment.
- Summary emits deterministic markdown/json outputs with explicit `overall` and `nextAction`.
- Closure audit includes dedicated `M19-E` gate.

### M19-S5 tracking (live status)
- [x] M19 executed-slice convergence summary script added.
- [x] M19 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M19 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M19-E`).

### M19-S6 transition handoff packet acceptance criteria
- Transition packet script validates kickoff/matrix/selector/runtime/convergence contracts and selector/runtime alignment.
- Transition packet copies normalized artifacts into deterministic output directory and emits packet manifest summary.
- Closure audit includes dedicated `M19-F` gate.

### M19-S6 tracking (live status)
- [x] M19 transition handoff packet script added.
- [x] M19 transition handoff packet contract test added.
- [x] Book chapter documenting M19 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M19-F`).

### M19-S7 closure report acceptance criteria
- M19 closure report script validates strict closure gate JSON and M19 transition packet contract.
- Report aggregates `M19-A..M19-F` gate statuses plus packet summary into deterministic markdown/json outputs.
- Closure audit includes dedicated `M19-G` gate.

### M19-S7 tracking (live status)
- [x] M19 closure report script added.
- [x] M19 closure report contract test added.
- [x] Book chapter documenting M19 closure report added.
- [x] Naming-lock CI and closure gate updated (`M19-G`).

### Exit criteria
- M19 kickoff starts from deterministic evidence (`M18` closure + handoff packet), not ad-hoc operator judgment.

### Docs/book outputs
- Chapter: "M19 Kickoff Brief".
- Chapter: "M19 Priority Matrix".
- Chapter: "M19 Next-Slice Selector".
- Chapter: "M19 Runtime Hardening Runner".
- Chapter: "M19 Executed-Slice Convergence Summary".
- Chapter: "M19 Transition Handoff Packet".
- Chapter: "M19 Closure Report".

## M20 - Kickoff from Closed M19 Baseline
### Trigger condition
- Start after M19 closure report (`M19-G`) is PASS and M19 transition packet is available.

### Scope decision (M20 kickoff)
- Primary scope: use finalized M19 closure + handoff packet to define deterministic M20 start priorities.
- Included tracks:
  - M20 kickoff brief generation from M19 closure/packet artifacts,
  - closure-gated naming-lock contract for M20 kickoff brief.
- Deferred:
  - deeper M20 implementation slices until kickoff brief is stable.

### Build tasks
- Generate M20 kickoff brief from M19 closure report + transition packet.
- Support markdown and JSON output for operator + automation consumers.
- Auto-generate M19 closure json when absent but packet is available.
- Build deterministic M20 priority matrix from kickoff brief.
- Select first executable M20 slice from kickoff + matrix inputs.
- Execute first selected runtime slice via dedicated M20 runtime hardening runner.
- Build executed-slice convergence summary from selector + runtime execution artifacts.
- Build transition handoff packet from kickoff/matrix/selector/runtime/convergence artifacts.
- Build final M20 closure report from strict closure gates + M20 transition packet summary.
- Lock kickoff-brief/matrix/selector/runtime/convergence/transition/closure contracts in naming-lock CI and closure audit.

### M20-S1 kickoff acceptance criteria
- M20 kickoff brief script + contract test exists.
- Script validates M19 packet/closure contract and handles missing M19 closure artifact deterministically.
- Closure audit includes dedicated `M20-A` gate.

### M20-S1 tracking (live status)
- [x] M20 kickoff brief script added.
- [x] M20 kickoff brief contract test added.
- [x] Book chapter documenting M20 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M20-A`).

### M20-S2 priority matrix acceptance criteria
- M20 priority matrix script validates kickoff brief contract and produces deterministic track ordering.
- Matrix emits markdown/json outputs with stable `tracks[]` priority/score schema.
- Closure audit includes dedicated `M20-B` gate.

### M20-S2 tracking (live status)
- [x] M20 priority matrix script added.
- [x] M20 priority matrix contract test added.
- [x] Book chapter documenting M20 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M20-B`).

### M20-S3 next-slice selector acceptance criteria
- M20 selector validates kickoff + matrix contracts and emits one deterministic recommendation.
- Selector recommendations use `M20-S4-*` IDs for runtime/release/editor/stabilization tracks.
- Closure audit includes dedicated `M20-C` gate.

### M20-S3 tracking (live status)
- [x] M20 next-slice selector script added.
- [x] M20 next-slice selector contract test added.
- [x] Book chapter documenting M20 next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M20-C`).

### M20-S4 first executed slice acceptance criteria
- Runtime hardening runner enforces runtime-selected `M20-S4-runtime-*` recommendations and supports dry-run/json outputs.
- Runner emits deterministic command plans and execution status.
- Closure audit includes dedicated `M20-D` gate.

### M20-S4 tracking (live status)
- [x] M20 runtime hardening runner script added.
- [x] M20 runtime hardening runner contract test added.
- [x] Book chapter documenting M20 runtime hardening runner added.
- [x] Naming-lock CI and closure gate updated (`M20-D`).

### M20-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script validates selector/runtime execution artifact contracts and recommendation alignment.
- Summary emits deterministic markdown/json outputs with explicit `overall` and `nextAction`.
- Closure audit includes dedicated `M20-E` gate.

### M20-S5 tracking (live status)
- [x] M20 executed-slice convergence summary script added.
- [x] M20 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M20 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M20-E`).

### M20-S6 transition handoff packet acceptance criteria
- Transition packet script validates kickoff/matrix/selector/runtime/convergence contracts and selector/runtime alignment.
- Transition packet copies normalized artifacts into deterministic output directory and emits packet manifest summary.
- Closure audit includes dedicated `M20-F` gate.

### M20-S6 tracking (live status)
- [x] M20 transition handoff packet script added.
- [x] M20 transition handoff packet contract test added.
- [x] Book chapter documenting M20 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M20-F`).

### M20-S7 closure report acceptance criteria
- M20 closure report script validates strict closure gate JSON and M20 transition packet contract.
- Report emits deterministic markdown/json outputs with gate summary and next-action text.
- Closure audit includes dedicated `M20-G` gate.

### M20-S7 tracking (live status)
- [x] M20 closure report script added.
- [x] M20 closure report contract test added.
- [x] Book chapter documenting M20 closure report added.
- [x] Naming-lock CI and closure gate updated (`M20-G`).

### Exit criteria
- M20 kickoff starts from deterministic evidence (`M19` closure + handoff packet), not ad-hoc operator judgment.

### Docs/book outputs
- Chapter: "M20 Kickoff Brief".
- Chapter: "M20 Priority Matrix".
- Chapter: "M20 Next-Slice Selector".
- Chapter: "M20 Runtime Hardening Runner".
- Chapter: "M20 Executed-Slice Convergence Summary".
- Chapter: "M20 Transition Handoff Packet".
- Chapter: "M20 Closure Report".

## M21 - Kickoff from Closed M20 Baseline
### Trigger condition
- Start after M20 closure report (`M20-G`) is PASS and M20 transition packet is available.

### Scope decision (M21 kickoff)
- Primary scope: use finalized M20 closure + handoff packet to define deterministic M21 start priorities.
- Included tracks:
  - M21 kickoff brief generation from M20 closure/packet artifacts,
  - closure-gated naming-lock contract for M21 kickoff brief.
- Deferred:
  - deeper M21 implementation slices until kickoff brief is stable.

### Build tasks
- Generate M21 kickoff brief from M20 closure report + transition packet.
- Support markdown and JSON output for operator + automation consumers.
- Auto-generate M20 closure json when absent but packet is available.
- Build deterministic M21 priority matrix from kickoff brief.
- Select first executable M21 slice from kickoff + matrix inputs.
- Execute first selected runtime slice via dedicated M21 runtime hardening runner.
- Build executed-slice convergence summary from selector + runtime execution artifacts.
- Build transition handoff packet from kickoff/matrix/selector/runtime/convergence artifacts.
- Lock kickoff-brief/matrix/selector/runtime/convergence/transition contracts in naming-lock CI and closure audit.

### M21-S1 kickoff acceptance criteria
- M21 kickoff brief script + contract test exists.
- Script validates M20 packet/closure contract and handles missing M20 closure artifact deterministically.
- Closure audit includes dedicated `M21-A` gate.

### M21-S1 tracking (live status)
- [x] M21 kickoff brief script added.
- [x] M21 kickoff brief contract test added.
- [x] Book chapter documenting M21 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M21-A`).

### M21-S2 priority matrix acceptance criteria
- M21 priority matrix script validates kickoff brief contract and produces deterministic track ordering.
- Matrix emits markdown/json outputs with stable `tracks[]` priority/score schema.
- Closure audit includes dedicated `M21-B` gate.

### M21-S2 tracking (live status)
- [x] M21 priority matrix script added.
- [x] M21 priority matrix contract test added.
- [x] Book chapter documenting M21 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M21-B`).

### M21-S3 next-slice selector acceptance criteria
- M21 selector validates kickoff + matrix contracts and emits one deterministic recommendation.
- Selector recommendations use `M21-S4-*` IDs for runtime/release/editor/stabilization tracks.
- Closure audit includes dedicated `M21-C` gate.

### M21-S3 tracking (live status)
- [x] M21 next-slice selector script added.
- [x] M21 next-slice selector contract test added.
- [x] Book chapter documenting M21 next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M21-C`).

### M21-S4 first executed slice acceptance criteria
- Runtime hardening runner enforces runtime-selected `M21-S4-runtime-*` recommendations and supports dry-run/json outputs.
- Runner emits deterministic command plans and execution status.
- Closure audit includes dedicated `M21-D` gate.

### M21-S4 tracking (live status)
- [x] M21 runtime hardening runner script added.
- [x] M21 runtime hardening runner contract test added.
- [x] Book chapter documenting M21 runtime hardening runner added.
- [x] Naming-lock CI and closure gate updated (`M21-D`).

### M21-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script validates selector/runtime execution artifact contracts and recommendation alignment.
- Summary emits deterministic markdown/json outputs with explicit `overall` and `nextAction`.
- Closure audit includes dedicated `M21-E` gate.

### M21-S5 tracking (live status)
- [x] M21 executed-slice convergence summary script added.
- [x] M21 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M21 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M21-E`).

### M21-S6 transition handoff packet acceptance criteria
- Transition packet script validates kickoff/matrix/selector/runtime/convergence contracts and selector/runtime alignment.
- Transition packet copies normalized artifacts into deterministic output directory and emits packet manifest summary.
- Closure audit includes dedicated `M21-F` gate.

### M21-S6 tracking (live status)
- [x] M21 transition handoff packet script added.
- [x] M21 transition handoff packet contract test added.
- [x] Book chapter documenting M21 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M21-F`).

### M21-S7 closure report acceptance criteria
- Closure report script validates strict closure gates plus transition packet summary and emits deterministic markdown/json outputs.
- PASS `nextAction` advances to M22 kickoff; non-pass states remain pending with explicit remediation guidance.
- Closure audit includes dedicated `M21-G` gate.

### M21-S7 tracking (live status)
- [x] M21 closure report script added.
- [x] M21 closure report contract test added.
- [x] Book chapter documenting M21 closure report added.
- [x] Naming-lock CI and closure gate updated (`M21-G`).

### Exit criteria
- M21 kickoff starts from deterministic evidence (`M20` closure + handoff packet), not ad-hoc operator judgment.

### Docs/book outputs
- Chapter: "M21 Kickoff Brief".
- Chapter: "M21 Priority Matrix".
- Chapter: "M21 Next-Slice Selector".
- Chapter: "M21 Runtime Hardening Runner".
- Chapter: "M21 Executed-Slice Convergence Summary".
- Chapter: "M21 Transition Handoff Packet".
- Chapter: "M21 Closure Report".

## M22 - Kickoff Loop (In Progress)

### Goal
- Start M22 from deterministic M21 closure evidence, then continue the same gated slice progression (`S1..S7`) used for M18-M21.

### M22-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M21 closure report + M21 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M21 closure report JSON from `build-m21-closure-report.sh`.
- Closure audit includes dedicated `M22-A` gate.

### M22-S1 tracking (live status)
- [x] M22 kickoff brief script added.
- [x] M22 kickoff brief contract test added.
- [x] Book chapter documenting M22 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M22-A`).

### M22-S2 priority matrix acceptance criteria
- Priority matrix script consumes M22 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M22-B` gate.

### M22-S2 tracking (live status)
- [x] M22 priority matrix script added.
- [x] M22 priority matrix contract test added.
- [x] Book chapter documenting M22 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M22-B`).

### M22-S3 next-slice selector acceptance criteria
- Selector consumes M22 kickoff + priority matrix artifacts and emits one deterministic executable recommendation with stabilization fallback.
- Recommended IDs map to `M22-S4-*` tracks and include explicit closure gate metadata.
- Closure audit includes dedicated `M22-C` gate.

### M22-S3 tracking (live status)
- [x] M22 next-slice selector script added.
- [x] M22 next-slice selector contract test added.
- [x] Book chapter documenting M22 next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M22-C`).

### M22-S4 runtime hardening runner acceptance criteria
- Runtime runner consumes M22 selector output and only executes runtime-selected `M22-S4-runtime-*` slices.
- Dry-run mode emits deterministic command-plan json/text; execution mode runs runtime smoke bundle + closure checks.
- Closure audit includes dedicated `M22-D` gate.

### M22-S4 tracking (live status)
- [x] M22 runtime hardening runner script added.
- [x] M22 runtime hardening runner contract test added.
- [x] Book chapter documenting M22 runtime hardening runner added.
- [x] Naming-lock CI and closure gate updated (`M22-D`).

### M22-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script validates selector/runtime execution artifact alignment and emits deterministic markdown/json outputs.
- Summary computes `executionPass`, `overall`, and deterministic `nextAction` from runtime execution status.
- Closure audit includes dedicated `M22-E` gate.

### M22-S5 tracking (live status)
- [x] M22 executed-slice convergence summary script added.
- [x] M22 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M22 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M22-E`).

### M22-S6 transition handoff packet acceptance criteria
- Transition packet script validates kickoff/matrix/selector/runtime/convergence artifacts and enforces selector/runtime recommendation alignment.
- Packet copies normalized artifacts into deterministic output directory and emits handoff manifest summary.
- Closure audit includes dedicated `M22-F` gate.

### M22-S6 tracking (live status)
- [x] M22 transition handoff packet script added.
- [x] M22 transition handoff packet contract test added.
- [x] Book chapter documenting M22 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M22-F`).

### M22-S7 closure report acceptance criteria
- Closure report script consumes strict closure JSON + M22 transition packet summary and computes deterministic `overall` + `nextAction`.
- Closure report output includes full required gate snapshot (`M22-A..M22-F`) and packet convergence fields.
- Closure audit includes dedicated `M22-G` gate.

### M22-S7 tracking (live status)
- [x] M22 closure report script added.
- [x] M22 closure report contract test added.
- [x] Book chapter documenting M22 closure report added.
- [x] Naming-lock CI and closure gate updated (`M22-G`).

## M23 - Kickoff Loop (Complete)

### Goal
- Start M23 from deterministic M22 closure evidence, then continue the same gated slice progression (`S1..S7`) used for M18-M22.

### M23-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M22 closure report + M22 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M22 closure report JSON from `build-m22-closure-report.sh`.
- Closure audit includes dedicated `M23-A` gate.

### M23-S1 tracking (live status)
- [x] M23 kickoff brief script added.
- [x] M23 kickoff brief contract test added.
- [x] Book chapter documenting M23 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M23-A`).

### M23-S2 priority matrix acceptance criteria
- Priority matrix script consumes M23 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M23-B` gate.

### M23-S2 tracking (live status)
- [x] M23 priority matrix script added.
- [x] M23 priority matrix contract test added.
- [x] Book chapter documenting M23 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M23-B`).

### M23-S3 next-slice selector acceptance criteria
- Selector consumes M23 kickoff + priority matrix artifacts and emits one deterministic executable recommendation with stabilization fallback.
- Recommended IDs map to `M23-S4-*` tracks and include explicit closure gate metadata.
- Closure audit includes dedicated `M23-C` gate.

### M23-S3 tracking (live status)
- [x] M23 next-slice selector script added.
- [x] M23 next-slice selector contract test added.
- [x] Book chapter documenting M23 next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M23-C`).

### M23-S4 runtime hardening runner acceptance criteria
- Runtime runner consumes M23 selector output and only executes runtime-selected `M23-S4-runtime-*` slices.
- Dry-run mode emits deterministic command-plan json/text; execution mode runs runtime smoke bundle + closure checks.
- Closure audit includes dedicated `M23-D` gate.

### M23-S4 tracking (live status)
- [x] M23 runtime hardening runner script added.
- [x] M23 runtime hardening runner contract test added.
- [x] Book chapter documenting M23 runtime hardening runner added.
- [x] Naming-lock CI and closure gate updated (`M23-D`).

### M23-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script validates selector/runtime execution artifact alignment and emits deterministic markdown/json outputs.
- Summary computes `executionPass`, `overall`, and deterministic `nextAction` from runtime execution status.
- Closure audit includes dedicated `M23-E` gate.

### M23-S5 tracking (live status)
- [x] M23 executed-slice convergence summary script added.
- [x] M23 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M23 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M23-E`).

### M23-S6 transition handoff packet acceptance criteria
- Transition packet script validates kickoff/matrix/selector/runtime/convergence artifacts and enforces selector/runtime recommendation alignment.
- Packet copies normalized artifacts into deterministic output directory and emits handoff manifest summary.
- Closure audit includes dedicated `M23-F` gate.

### M23-S6 tracking (live status)
- [x] M23 transition handoff packet script added.
- [x] M23 transition handoff packet contract test added.
- [x] Book chapter documenting M23 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M23-F`).

### M23-S7 closure report acceptance criteria
- Closure report script consumes strict closure JSON + M23 transition packet summary and computes deterministic `overall` + `nextAction`.
- Closure report output includes full required gate snapshot (`M23-A..M23-F`) and packet convergence fields.
- Closure audit includes dedicated `M23-G` gate.

### M23-S7 tracking (live status)
- [x] M23 closure report script added.
- [x] M23 closure report contract test added.
- [x] Book chapter documenting M23 closure report added.
- [x] Naming-lock CI and closure gate updated (`M23-G`).

### Next planned slice
- M24-S1 kickoff brief + closure gate `M24-A`.

## M24 - Kickoff Loop (Complete)

### Goal
- Start M24 from deterministic M23 closure evidence, then continue the same gated slice progression (`S1..S7`) used for M18-M23.

### M24-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M23 closure report + M23 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M23 closure report JSON from `build-m23-closure-report.sh`.
- Closure audit includes dedicated `M24-A` gate.

### M24-S1 tracking (live status)
- [x] M24 kickoff brief script added.
- [x] M24 kickoff brief contract test added.
- [x] Book chapter documenting M24 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M24-A`).

### M24-S2 priority matrix acceptance criteria
- Priority matrix script consumes M24 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M24-B` gate.

### M24-S2 tracking (live status)
- [x] M24 priority matrix script added.
- [x] M24 priority matrix contract test added.
- [x] Book chapter documenting M24 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M24-B`).

### M24-S3 next-slice selector acceptance criteria
- Selector consumes M24 kickoff + priority matrix artifacts and emits one deterministic executable recommendation with stabilization fallback.
- Recommended IDs map to `M24-S4-*` tracks and include explicit closure gate metadata.
- Closure audit includes dedicated `M24-C` gate.

### M24-S3 tracking (live status)
- [x] M24 next-slice selector script added.
- [x] M24 next-slice selector contract test added.
- [x] Book chapter documenting M24 next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M24-C`).

### M24-S4 runtime hardening runner acceptance criteria
- Runtime runner consumes M24 selector output and only executes runtime-selected `M24-S4-runtime-*` slices.
- Dry-run mode emits deterministic command-plan json/text; execution mode runs runtime smoke bundle + closure checks.
- Closure audit includes dedicated `M24-D` gate.

### M24-S4 tracking (live status)
- [x] M24 runtime hardening runner script added.
- [x] M24 runtime hardening runner contract test added.
- [x] Book chapter documenting M24 runtime hardening runner added.
- [x] Naming-lock CI and closure gate updated (`M24-D`).

### M24-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script validates selector/runtime execution artifact alignment and emits deterministic markdown/json outputs.
- Summary computes `executionPass`, `overall`, and deterministic `nextAction` from runtime execution status.
- Closure audit includes dedicated `M24-E` gate.

### M24-S5 tracking (live status)
- [x] M24 executed-slice convergence summary script added.
- [x] M24 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M24 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M24-E`).

### M24-S6 transition handoff packet acceptance criteria
- Transition packet script validates kickoff/matrix/selector/runtime/convergence artifacts and enforces selector/runtime recommendation alignment.
- Packet copies normalized artifacts into deterministic output directory and emits handoff manifest summary.
- Closure audit includes dedicated `M24-F` gate.

### M24-S6 tracking (live status)
- [x] M24 transition handoff packet script added.
- [x] M24 transition handoff packet contract test added.
- [x] Book chapter documenting M24 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M24-F`).

### M24-S7 closure report acceptance criteria
- Closure report script consumes strict closure JSON + M24 transition packet summary and computes deterministic `overall` + `nextAction`.
- Closure report output includes full required gate snapshot (`M24-A..M24-F`) and packet convergence fields.
- Closure audit includes dedicated `M24-G` gate.

### M24-S7 tracking (live status)
- [x] M24 closure report script added.
- [x] M24 closure report contract test added.
- [x] Book chapter documenting M24 closure report added.
- [x] Naming-lock CI and closure gate updated (`M24-G`).

### Next planned slice
- M25-S1 kickoff brief + closure gate `M25-A`.

## M25 - Kickoff Loop (Complete)

### Goal
- Start M25 from deterministic M24 closure evidence, then continue the same gated slice progression (`S1..S7`) used for M18-M24.

### M25-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M24 closure report + M24 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M24 closure report JSON from `build-m24-closure-report.sh`.
- Closure audit includes dedicated `M25-A` gate.

### M25-S1 tracking (live status)
- [x] M25 kickoff brief script added.
- [x] M25 kickoff brief contract test added.
- [x] Book chapter documenting M25 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M25-A`).

### M25-S2 priority matrix acceptance criteria
- Priority matrix script consumes M25 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M25-B` gate.

### M25-S2 tracking (live status)
- [x] M25 priority matrix script added.
- [x] M25 priority matrix contract test added.
- [x] Book chapter documenting M25 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M25-B`).

### M25-S3 next-slice selector acceptance criteria
- Selector consumes M25 kickoff + priority matrix artifacts and emits one deterministic executable recommendation with stabilization fallback.
- Recommended IDs map to `M25-S4-*` tracks and include explicit closure gate metadata.
- Closure audit includes dedicated `M25-C` gate.

### M25-S3 tracking (live status)
- [x] M25 next-slice selector script added.
- [x] M25 next-slice selector contract test added.
- [x] Book chapter documenting M25 next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M25-C`).

### M25-S4 runtime hardening runner acceptance criteria
- Runtime runner consumes M25 selector output and only executes runtime-selected `M25-S4-runtime-*` slices.
- Dry-run mode emits deterministic command-plan json/text; execution mode runs runtime smoke bundle + closure checks.
- Closure audit includes dedicated `M25-D` gate.

### M25-S4 tracking (live status)
- [x] M25 runtime hardening runner script added.
- [x] M25 runtime hardening runner contract test added.
- [x] Book chapter documenting M25 runtime hardening runner added.
- [x] Naming-lock CI and closure gate updated (`M25-D`).

### M25-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script validates selector/runtime execution artifact alignment and emits deterministic markdown/json outputs.
- Summary computes `executionPass`, `overall`, and deterministic `nextAction` from runtime execution status.
- Closure audit includes dedicated `M25-E` gate.

### M25-S5 tracking (live status)
- [x] M25 executed-slice convergence summary script added.
- [x] M25 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M25 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M25-E`).

### M25-S6 transition handoff packet acceptance criteria
- Transition packet script validates kickoff/matrix/selector/runtime/convergence artifacts and enforces selector/runtime recommendation alignment.
- Packet copies normalized artifacts into deterministic output directory and emits handoff manifest summary.
- Closure audit includes dedicated `M25-F` gate.

### M25-S6 tracking (live status)
- [x] M25 transition handoff packet script added.
- [x] M25 transition handoff packet contract test added.
- [x] Book chapter documenting M25 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M25-F`).

### M25-S7 closure report acceptance criteria
- Closure report script consumes strict closure JSON + M25 transition packet summary and computes deterministic `overall` + `nextAction`.
- Closure report output includes full required gate snapshot (`M25-A..M25-F`) and packet convergence fields.
- Closure audit includes dedicated `M25-G` gate.

### M25-S7 tracking (live status)
- [x] M25 closure report script added.
- [x] M25 closure report contract test added.
- [x] Book chapter documenting M25 closure report added.
- [x] Naming-lock CI and closure gate updated (`M25-G`).

### Next planned slice
- M26-S1 kickoff brief + closure gate `M26-A`.

## M26 - Kickoff Loop (In Progress)

### Goal
- Start M26 from deterministic M25 closure evidence, then continue the same gated slice progression (`S1..S7`) used for M18-M25.

### M26-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M25 closure report + M25 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M25 closure report JSON from `build-m25-closure-report.sh`.
- Closure audit includes dedicated `M26-A` gate.

### M26-S1 tracking (live status)
- [x] M26 kickoff brief script added.
- [x] M26 kickoff brief contract test added.
- [x] Book chapter documenting M26 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M26-A`).

### M26-S2 priority matrix acceptance criteria
- Priority matrix script consumes M26 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M26-B` gate.

### M26-S2 tracking (live status)
- [x] M26 priority matrix script added.
- [x] M26 priority matrix contract test added.
- [x] Book chapter documenting M26 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M26-B`).

### M26-S3 next-slice selector acceptance criteria
- Selector consumes M26 kickoff + priority matrix artifacts and emits one deterministic executable recommendation with stabilization fallback.
- Recommended IDs map to `M26-S4-*` tracks and include explicit closure gate metadata.
- Closure audit includes dedicated `M26-C` gate.

### M26-S3 tracking (live status)
- [x] M26 next-slice selector script added.
- [x] M26 next-slice selector contract test added.
- [x] Book chapter documenting M26 next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M26-C`).

### M26-S4 runtime hardening runner acceptance criteria
- Runtime runner consumes M26 selector output and only executes runtime-selected `M26-S4-runtime-*` slices.
- Dry-run mode emits deterministic command-plan json/text; execution mode runs runtime smoke bundle + closure checks.
- Closure audit includes dedicated `M26-D` gate.

### M26-S4 tracking (live status)
- [x] M26 runtime hardening runner script added.
- [x] M26 runtime hardening runner contract test added.
- [x] Book chapter documenting M26 runtime hardening runner added.
- [x] Naming-lock CI and closure gate updated (`M26-D`).

### M26-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script validates selector/runtime execution artifact alignment and emits deterministic markdown/json outputs.
- Summary computes `executionPass`, `overall`, and deterministic `nextAction` from runtime execution status.
- Closure audit includes dedicated `M26-E` gate.

### M26-S5 tracking (live status)
- [x] M26 executed-slice convergence summary script added.
- [x] M26 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M26 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M26-E`).

### M26-S6 transition handoff packet acceptance criteria
- Transition packet script validates kickoff/matrix/selector/runtime/convergence artifacts and enforces selector/runtime recommendation alignment.
- Packet copies normalized artifacts into deterministic output directory and emits handoff manifest summary.
- Closure audit includes dedicated `M26-F` gate.

### M26-S6 tracking (live status)
- [x] M26 transition handoff packet script added.
- [x] M26 transition handoff packet contract test added.
- [x] Book chapter documenting M26 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M26-F`).

### M26-S7 closure report acceptance criteria
- Closure report script consumes strict closure JSON + M26 transition packet summary and computes deterministic `overall` + `nextAction`.
- Closure report output includes full required gate snapshot (`M26-A..M26-F`) and packet convergence fields.
- Closure audit includes dedicated `M26-G` gate.

### M26-S7 tracking (live status)
- [x] M26 closure report script added.
- [x] M26 closure report contract test added.
- [x] Book chapter documenting M26 closure report added.
- [x] Naming-lock CI and closure gate updated (`M26-G`).

### Next planned slice
- M27-S1 kickoff brief + closure gate `M27-A`.

## M27 - Kickoff Loop (In Progress)

### Goal
- Start M27 from deterministic M26 closure evidence, then continue the same gated slice progression (`S1..S7`) used for M18-M26.

### M27-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M26 closure report + M26 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M26 closure report JSON from `build-m26-closure-report.sh`.
- Closure audit includes dedicated `M27-A` gate.

### M27-S1 tracking (live status)
- [x] M27 kickoff brief script added.
- [x] M27 kickoff brief contract test added.
- [x] Book chapter documenting M27 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M27-A`).

### M27-S2 priority matrix acceptance criteria
- Priority matrix script consumes M27 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M27-B` gate.

### M27-S2 tracking (live status)
- [x] M27 priority matrix script added.
- [x] M27 priority matrix contract test added.
- [x] Book chapter documenting M27 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M27-B`).

### M27-S3 next-slice selector acceptance criteria
- Selector consumes M27 kickoff + priority matrix artifacts and emits one deterministic executable recommendation with stabilization fallback.
- Recommended IDs map to `M27-S4-*` tracks and include explicit closure gate metadata.
- Closure audit includes dedicated `M27-C` gate.

### M27-S3 tracking (live status)
- [x] M27 next-slice selector script added.
- [x] M27 next-slice selector contract test added.
- [x] Book chapter documenting M27 next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M27-C`).

### M27-S4 runtime hardening runner acceptance criteria
- Runtime runner consumes M27 selector output and only executes runtime-selected `M27-S4-runtime-*` slices.
- Dry-run mode emits deterministic command-plan json/text; execution mode runs runtime smoke bundle + closure checks.
- Closure audit includes dedicated `M27-D` gate.

### M27-S4 tracking (live status)
- [x] M27 runtime hardening runner script added.
- [x] M27 runtime hardening runner contract test added.
- [x] Book chapter documenting M27 runtime hardening runner added.
- [x] Naming-lock CI and closure gate updated (`M27-D`).

### M27-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script validates selector/runtime execution artifact alignment and emits deterministic markdown/json outputs.
- Summary computes `executionPass`, `overall`, and deterministic `nextAction` from runtime execution status.
- Closure audit includes dedicated `M27-E` gate.

### M27-S5 tracking (live status)
- [x] M27 executed-slice convergence summary script added.
- [x] M27 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M27 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M27-E`).

### M27-S6 transition handoff packet acceptance criteria
- Transition packet script validates kickoff/matrix/selector/runtime/convergence contracts and enforces selector/runtime recommendation alignment.
- Packet output is deterministic (`handoff-packet.json` + normalized artifact copies) and can auto-generate convergence summary when omitted.
- Closure audit includes dedicated `M27-F` gate.

### M27-S6 tracking (live status)
- [x] M27 transition handoff packet script added.
- [x] M27 transition handoff packet contract test added.
- [x] Book chapter documenting M27 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M27-F`).

### M27-S7 closure report acceptance criteria
- Closure report script consumes strict closure gates + M27 handoff packet summary and computes deterministic `overall` + `nextAction`.
- Required gates `M27-A..M27-F` are validated in one canonical closure artifact (`m27Gates[]`).
- Closure audit includes dedicated `M27-G` gate.

### M27-S7 tracking (live status)
- [x] M27 closure report script added.
- [x] M27 closure report contract test added.
- [x] Book chapter documenting M27 closure report added.
- [x] Naming-lock CI and closure gate updated (`M27-G`).

### Next planned slice
- M28-S1 kickoff brief + closure gate `M28-A`.

## M28 - Kickoff Loop (Complete)

### Goal
- Start M28 from deterministic M27 closure evidence, then continue the same gated slice progression (`S1..S7`) used for M18-M27.

### M28-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M27 closure report + M27 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M27 closure report JSON from `build-m27-closure-report.sh`.
- Closure audit includes dedicated `M28-A` gate.

### M28-S1 tracking (live status)
- [x] M28 kickoff brief script added.
- [x] M28 kickoff brief contract test added.
- [x] Book chapter documenting M28 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M28-A`).

### M28-S2 priority matrix acceptance criteria
- Priority matrix script consumes M28 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M28-B` gate.

### M28-S2 tracking (live status)
- [x] M28 priority matrix script added.
- [x] M28 priority matrix contract test added.
- [x] Book chapter documenting M28 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M28-B`).

### M28-S3 next-slice selector acceptance criteria
- Selector consumes M28 kickoff + priority matrix artifacts and emits one deterministic executable recommendation with stabilization fallback.
- Recommended IDs map to `M28-S4-*` tracks and include explicit closure gate metadata.
- Closure audit includes dedicated `M28-C` gate.

### M28-S3 tracking (live status)
- [x] M28 next-slice selector script added.
- [x] M28 next-slice selector contract test added.
- [x] Book chapter documenting M28 next-slice selector added.
- [x] Naming-lock CI and closure gate updated (`M28-C`).

### M28-S4 runtime hardening runner acceptance criteria
- Runtime runner consumes M28 selector output and only executes runtime-selected `M28-S4-runtime-*` slices.
- Dry-run mode emits deterministic command-plan json/text; execution mode runs runtime smoke bundle + closure checks.
- Closure audit includes dedicated `M28-D` gate.

### M28-S4 tracking (live status)
- [x] M28 runtime hardening runner script added.
- [x] M28 runtime hardening runner contract test added.
- [x] Book chapter documenting M28 runtime hardening runner added.
- [x] Naming-lock CI and closure gate updated (`M28-D`).

### M28-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script validates selector/runtime execution artifact alignment and emits deterministic markdown/json outputs.
- Summary computes `executionPass`, `overall`, and deterministic `nextAction` from runtime execution status.
- Closure audit includes dedicated `M28-E` gate.

### M28-S5 tracking (live status)
- [x] M28 executed-slice convergence summary script added.
- [x] M28 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M28 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M28-E`).

### M28-S6 transition handoff packet acceptance criteria
- Transition packet script validates kickoff/matrix/selector/runtime/convergence contracts and enforces selector/runtime recommendation alignment.
- Packet output is deterministic (`handoff-packet.json` + normalized artifact copies) and can auto-generate convergence summary when omitted.
- Closure audit includes dedicated `M28-F` gate.

### M28-S6 tracking (live status)
- [x] M28 transition handoff packet script added.
- [x] M28 transition handoff packet contract test added.
- [x] Book chapter documenting M28 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M28-F`).

### M28-S7 closure report acceptance criteria
- Closure report script consumes strict closure gates + M28 handoff packet summary and computes deterministic `overall` + `nextAction`.
- Required gates `M28-A..M28-F` are validated in one canonical closure artifact (`m28Gates[]`).
- Closure audit includes dedicated `M28-G` gate.

### M28-S7 tracking (live status)
- [x] M28 closure report script added.
- [x] M28 closure report contract test added.
- [x] Book chapter documenting M28 closure report added.
- [x] Naming-lock CI and closure gate updated (`M28-G`).

### Next planned slice
- M29-S1 kickoff brief + closure gate `M29-A`.

## M29 - Runtime-First Stabilization Loop (In Progress)

### Goal
- Start M29 from deterministic M28 closure evidence and shift implementation weight toward runtime de-stubbing and real typed sink behavior while preserving compile-time security gates.

### M29-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M28 closure report + M28 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M28 closure report JSON from `build-m28-closure-report.sh`.
- Closure audit includes dedicated `M29-A` gate.

### M29-S1 tracking (live status)
- [x] M29 kickoff brief script added.
- [x] M29 kickoff brief contract test added.
- [x] Book chapter documenting M29 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M29-A`).

### M29-S2 priority matrix acceptance criteria
- Priority matrix script consumes M29 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M29-B` gate.

### M29-S2 tracking (live status)
- [x] M29 priority matrix script added.
- [x] M29 priority matrix contract test added.
- [x] Book chapter documenting M29 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M29-B`).

### M29-S3 runtime-first de-stub plan acceptance criteria
- Runtime-first plan consumes M29 kickoff + priority matrix artifacts and emits deterministic db/fs/net/validator/secrets execution ordering.
- Plan output includes explicit rationale + closure metadata for follow-up execution slices.
- Closure audit includes dedicated `M29-C` gate.

### M29-S3 tracking (live status)
- [x] M29 runtime-first de-stub plan script added.
- [x] M29 runtime-first de-stub plan contract test added.
- [x] Book chapter documenting M29 runtime-first de-stub plan added.
- [x] Naming-lock CI and closure gate updated (`M29-C`).

### M29-S4 runtime de-stub execution runner acceptance criteria
- Runtime execution runner consumes M29 de-stub plan and executes only the selected first runtime slice with deterministic dry-run + execute modes.
- Execution emits deterministic status artifact suitable for convergence summary input.
- Closure audit includes dedicated `M29-D` gate.

### M29-S4 tracking (live status)
- [x] M29 runtime de-stub execution runner script added.
- [x] M29 runtime de-stub execution runner contract test added.
- [x] Book chapter documenting M29 runtime de-stub execution runner added.
- [x] Naming-lock CI and closure gate updated (`M29-D`).

### M29-S5 executed-slice convergence summary acceptance criteria
- Convergence summary consumes M29 planner + runner artifacts and computes deterministic `executionPass`/`overall`/`nextAction`.
- Summary validates selected-slice alignment between plan and execution status artifact.
- Closure audit includes dedicated `M29-E` gate.

### M29-S5 tracking (live status)
- [x] M29 executed-slice convergence summary script added.
- [x] M29 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M29 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M29-E`).

### M29-S6 transition handoff packet acceptance criteria
- Transition packet validates kickoff/matrix/plan/runner/convergence contracts and enforces selected-slice alignment.
- Packet output is deterministic (`handoff-packet.json` + normalized artifact copies) and can auto-generate convergence summary when omitted.
- Closure audit includes dedicated `M29-F` gate.

### M29-S6 tracking (live status)
- [x] M29 transition handoff packet script added.
- [x] M29 transition handoff packet contract test added.
- [x] Book chapter documenting M29 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M29-F`).

### M29-S7 closure report acceptance criteria
- Closure report script consumes strict closure gates + M29 handoff packet summary and computes deterministic `overall` + `nextAction`.
- Required gates `M29-A..M29-F` are validated in one canonical closure artifact (`m29Gates[]`).
- Closure audit includes dedicated `M29-G` gate.

### M29-S7 tracking (live status)
- [x] M29 closure report script added.
- [x] M29 closure report contract test added.
- [x] Book chapter documenting M29 closure report added.
- [x] Naming-lock CI and closure gate updated (`M29-G`).

### Next planned slice
- M30-S1 kickoff brief + closure gate `M30-A`.

## M30 - Runtime-First Stabilization Loop (In Progress)

### Goal
- Start M30 from deterministic M29 closure evidence and continue runtime-first stabilization with strict closure-gated slices.

### M30-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M29 closure report + M29 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M29 closure report JSON from `build-m29-closure-report.sh`.
- Closure audit includes dedicated `M30-A` gate.

### M30-S1 tracking (live status)
- [x] M30 kickoff brief script added.
- [x] M30 kickoff brief contract test added.
- [x] Book chapter documenting M30 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M30-A`).

### M30-S2 priority matrix acceptance criteria
- Priority matrix script consumes M30 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M30-B` gate.

### M30-S2 tracking (live status)
- [x] M30 priority matrix script added.
- [x] M30 priority matrix contract test added.
- [x] Book chapter documenting M30 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M30-B`).

### M30-S3 runtime-first de-stub plan acceptance criteria
- Runtime-first plan consumes M30 kickoff + priority matrix artifacts and emits deterministic db/fs/net/validator/secrets execution ordering.
- Plan output includes explicit rationale + closure metadata for follow-up execution slices.
- Closure audit includes dedicated `M30-C` gate.

### M30-S3 tracking (live status)
- [x] M30 runtime-first de-stub plan script added.
- [x] M30 runtime-first de-stub plan contract test added.
- [x] Book chapter documenting M30 runtime-first de-stub plan added.
- [x] Naming-lock CI and closure gate updated (`M30-C`).

### M30-S4 runtime de-stub execution runner acceptance criteria
- Runtime execution runner consumes M30 de-stub plan and executes only the selected first runtime slice with deterministic dry-run + execute modes.
- Execution emits deterministic status artifact suitable for convergence summary input.
- Closure audit includes dedicated `M30-D` gate.

### M30-S4 tracking (live status)
- [x] M30 runtime de-stub execution runner script added.
- [x] M30 runtime de-stub execution runner contract test added.
- [x] Book chapter documenting M30 runtime de-stub execution runner added.
- [x] Naming-lock CI and closure gate updated (`M30-D`).

### M30-S5 executed-slice convergence summary acceptance criteria
- Convergence summary consumes M30 planner + runner artifacts and computes deterministic `executionPass`/`overall`/`nextAction`.
- Summary validates selected-slice alignment between plan and execution status artifact.
- Closure audit includes dedicated `M30-E` gate.

### M30-S5 tracking (live status)
- [x] M30 executed-slice convergence summary script added.
- [x] M30 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M30 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M30-E`).

### M30-S6 transition handoff packet acceptance criteria
- Transition packet validates kickoff/matrix/plan/runner/convergence contracts and enforces selected-slice alignment.
- Packet output is deterministic (`handoff-packet.json` + normalized artifact copies) and can auto-generate convergence summary when omitted.
- Closure audit includes dedicated `M30-F` gate.

### M30-S6 tracking (live status)
- [x] M30 transition handoff packet script added.
- [x] M30 transition handoff packet contract test added.
- [x] Book chapter documenting M30 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M30-F`).

### M30-S7 closure report acceptance criteria
- Closure report script consumes strict closure gates + M30 handoff packet summary and computes deterministic `overall` + `nextAction`.
- Required gates `M30-A..M30-F` are validated in one canonical closure artifact (`m30Gates[]`).
- Closure audit includes dedicated `M30-G` gate.

### M30-S7 tracking (live status)
- [x] M30 closure report script added.
- [x] M30 closure report contract test added.
- [x] Book chapter documenting M30 closure report added.
- [x] Naming-lock CI and closure gate updated (`M30-G`).

## M31 - Runtime-First Stabilization Loop (In Progress)

### Goal
- Start M31 from deterministic M30 closure evidence and continue runtime-first stabilization with strict closure-gated slices.

### M31-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M30 closure report + M30 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M30 closure report JSON from `build-m30-closure-report.sh`.
- Closure audit includes dedicated `M31-A` gate.

### M31-S1 tracking (live status)
- [x] M31 kickoff brief script added.
- [x] M31 kickoff brief contract test added.
- [x] Book chapter documenting M31 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M31-A`).

### M31-S2 priority matrix acceptance criteria
- Priority matrix script consumes M31 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M31-B` gate.

### M31-S2 tracking (live status)
- [x] M31 priority matrix script added.
- [x] M31 priority matrix contract test added.
- [x] Book chapter documenting M31 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M31-B`).

### M31-S3 runtime-first de-stub plan acceptance criteria
- Runtime-first plan consumes M31 kickoff + priority matrix artifacts and emits deterministic db/fs/net/validator/secrets execution ordering.
- Plan output includes explicit rationale + closure metadata for follow-up execution slices.
- Closure audit includes dedicated `M31-C` gate.

### M31-S3 tracking (live status)
- [x] M31 runtime-first de-stub plan script added.
- [x] M31 runtime-first de-stub plan contract test added.
- [x] Book chapter documenting M31 runtime-first de-stub plan added.
- [x] Naming-lock CI and closure gate updated (`M31-C`).

### M31-S4 runtime de-stub execution runner acceptance criteria
- Runtime execution runner consumes M31 de-stub plan and executes only the selected first runtime slice with deterministic dry-run + execute modes.
- Execution emits deterministic status artifact suitable for convergence summary input.
- Closure audit includes dedicated `M31-D` gate.

### M31-S4 tracking (live status)
- [x] M31 runtime de-stub execution runner script added.
- [x] M31 runtime de-stub execution runner contract test added.
- [x] Book chapter documenting M31 runtime de-stub execution runner added.
- [x] Naming-lock CI and closure gate updated (`M31-D`).

### M31-S5 executed-slice convergence summary acceptance criteria
- Convergence summary consumes M31 planner + runner artifacts and computes deterministic `executionPass`/`overall`/`nextAction`.
- Summary validates selected-slice alignment between plan and execution status artifact.
- Closure audit includes dedicated `M31-E` gate.

### M31-S5 tracking (live status)
- [x] M31 executed-slice convergence summary script added.
- [x] M31 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M31 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M31-E`).

### M31-S6 transition handoff packet acceptance criteria
- Transition packet validates kickoff/matrix/plan/runner/convergence contracts and enforces selected-slice alignment.
- Packet output is deterministic (`handoff-packet.json` + normalized artifact copies) and can auto-generate convergence summary when omitted.
- Closure audit includes dedicated `M31-F` gate.

### M31-S6 tracking (live status)
- [x] M31 transition handoff packet script added.
- [x] M31 transition handoff packet contract test added.
- [x] Book chapter documenting M31 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M31-F`).

### M31-S7 closure report acceptance criteria
- Closure report script consumes strict closure gates + M31 handoff packet summary and computes deterministic `overall` + `nextAction`.
- Required gates `M31-A..M31-F` are validated in one canonical closure artifact (`m31Gates[]`).
- Closure audit includes dedicated `M31-G` gate.

### M31-S7 tracking (live status)
- [x] M31 closure report script added.
- [x] M31 closure report contract test added.
- [x] Book chapter documenting M31 closure report added.
- [x] Naming-lock CI and closure gate updated (`M31-G`).

## M32 - Runtime-First Stabilization Loop (In Progress)

### Goal
- Start M32 from deterministic M31 closure evidence and continue runtime-first stabilization with strict closure-gated slices.

### M32-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M31 closure report + M31 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M31 closure report JSON from `build-m31-closure-report.sh`.
- Closure audit includes dedicated `M32-A` gate.

### M32-S1 tracking (live status)
- [x] M32 kickoff brief script added.
- [x] M32 kickoff brief contract test added.
- [x] Book chapter documenting M32 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M32-A`).

### M32-S2 priority matrix acceptance criteria
- Priority matrix script consumes M32 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M32-B` gate.

### M32-S2 tracking (live status)
- [x] M32 priority matrix script added.
- [x] M32 priority matrix contract test added.
- [x] Book chapter documenting M32 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M32-B`).

### M32-S3 runtime-first de-stub plan acceptance criteria
- Runtime-first plan consumes M32 kickoff + priority matrix artifacts and emits deterministic db/fs/net/validator/secrets execution ordering.
- Plan output includes explicit rationale + closure metadata for follow-up execution slices.
- Closure audit includes dedicated `M32-C` gate.

### M32-S3 tracking (live status)
- [x] M32 runtime-first de-stub plan script added.
- [x] M32 runtime-first de-stub plan contract test added.
- [x] Book chapter documenting M32 runtime-first de-stub plan added.
- [x] Naming-lock CI and closure gate updated (`M32-C`).

### M32-S4 runtime de-stub execution runner acceptance criteria
- Runtime execution runner consumes M32 de-stub plan and executes only the selected first runtime slice with deterministic dry-run + execute modes.
- Execution emits deterministic status artifact suitable for convergence summary input.
- Closure audit includes dedicated `M32-D` gate.

### M32-S4 tracking (live status)
- [x] M32 runtime de-stub execution runner script added.
- [x] M32 runtime de-stub execution runner contract test added.
- [x] Book chapter documenting M32 runtime de-stub execution runner added.
- [x] Naming-lock CI and closure gate updated (`M32-D`).

### M32-S5 executed-slice convergence summary acceptance criteria
- Executed-slice convergence summary consumes M32 plan + runtime execution artifacts and verifies selected track/slice alignment.
- Summary emits deterministic markdown/json outputs with `executionPass`, `overall`, and `nextAction` fields.
- Closure audit includes dedicated `M32-E` gate.

### M32-S5 tracking (live status)
- [x] M32 executed-slice convergence summary script added.
- [x] M32 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M32 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M32-E`).

### M32-S6 transition handoff packet acceptance criteria
- Transition packet consumes M32 kickoff/matrix/plan/runtime/convergence artifacts and enforces deterministic selected-track/slice alignment.
- Packet output is deterministic (`handoff-packet.json` + normalized artifact copies) and can auto-generate convergence summary when omitted.
- Closure audit includes dedicated `M32-F` gate.

### M32-S6 tracking (live status)
- [x] M32 transition handoff packet script added.
- [x] M32 transition handoff packet contract test added.
- [x] Book chapter documenting M32 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M32-F`).

### M32-S7 closure report acceptance criteria
- Closure report script consumes strict closure gates + M32 handoff packet summary and computes deterministic `overall` + `nextAction`.
- Required gates `M32-A..M32-F` are validated in one canonical closure artifact (`m32Gates[]`).
- Closure audit includes dedicated `M32-G` gate.

### M32-S7 tracking (live status)
- [x] M32 closure report script added.
- [x] M32 closure report contract test added.
- [x] Book chapter documenting M32 closure report added.
- [x] Naming-lock CI and closure gate updated (`M32-G`).

### M32 re-entry checkpoint (locked)
- Stable checkpoint commit: `3f631c8` (`M32-G` closure report wiring complete).
- If we need to resume unfinished runtime-first work under M32, reopen as `M32-R*` slices from this checkpoint.
- Current post-closure engineering work tracks concrete runtime de-stub implementation and runtime behavior tests before widening the next milestone scope.

### M32 post-closure runtime slices (execution log)
- [x] `M32-R1`: runtime gate handle de-stub across request/gate helpers + `c-bin` runtime behavior test (`65b8001`).
- [x] `M32-R2`: runtime ABI prototype alignment for req/json/res signatures in `sec4_runtime.h` with synchronized C backend runtime-asset expectations.
- [x] `M32-R3`: content-level runtime guards for `headers.name`, `headers.value`, and `path.base` with deterministic C harness coverage.
- [x] `M32-R4`: URL content-level runtime validation for `url.public`/`url.internal` using runtime tracked-string payload mapping + deterministic C harness coverage.
- [x] `M32-R5`: expand request-source payload extraction for query/header values using request-backed parsing and tracked-string mapping.
- [x] `M32-R6`: path-param extraction from route templates + deterministic route-pattern matching/param extraction coverage in oneshot HTTP runtime tests.

## M33 - Runtime-First Stabilization Loop (In Progress)

### Goal
- Start M33 from deterministic M32 closure evidence and continue runtime-first stabilization with strict closure-gated slices.

### M33-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M32 closure report + M32 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M32 closure report JSON from `build-m32-closure-report.sh`.
- Closure audit includes dedicated `M33-A` gate.

### M33-S1 tracking (live status)
- [x] M33 kickoff brief script added.
- [x] M33 kickoff brief contract test added.
- [x] Book chapter documenting M33 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M33-A`).

### M33-S2 priority matrix acceptance criteria
- Priority matrix script consumes M33 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M33-B` gate.

### M33-S2 tracking (live status)
- [x] M33 priority matrix script added.
- [x] M33 priority matrix contract test added.
- [x] Book chapter documenting M33 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M33-B`).

### M33-S3 runtime-first de-stub plan acceptance criteria
- Runtime-first plan consumes M33 kickoff + priority matrix artifacts and emits deterministic db/fs/net/validator/secrets execution ordering.
- Plan output includes explicit rationale + closure metadata for follow-up execution slices.
- Closure audit includes dedicated `M33-C` gate.

### M33-S3 tracking (live status)
- [x] M33 runtime-first de-stub plan script added.
- [x] M33 runtime-first de-stub plan contract test added.
- [x] Book chapter documenting M33 runtime-first de-stub plan added.
- [x] Naming-lock CI and closure gate updated (`M33-C`).

### M33-S4 runtime de-stub execution runner acceptance criteria
- Runtime execution runner consumes M33 de-stub plan and executes only the selected first runtime slice with deterministic dry-run + execute modes.
- Runner emits execution-status artifacts with selected slice/domain metadata and closure gate annotations.
- Closure audit includes dedicated `M33-D` gate.

### M33-S4 tracking (live status)
- [x] M33 runtime de-stub execution runner script added.
- [x] M33 runtime de-stub execution runner contract test added.
- [x] Book chapter documenting M33 runtime de-stub execution runner added.
- [x] Naming-lock CI and closure gate updated (`M33-D`).

### M33-S5 executed-slice convergence summary acceptance criteria
- Executed-slice convergence summary consumes M33 plan + runtime execution artifacts and verifies selected track/slice alignment.
- Summary emits deterministic markdown/json output with explicit convergence status and chosen follow-up action.
- Closure audit includes dedicated `M33-E` gate.

### M33-S5 tracking (live status)
- [x] M33 executed-slice convergence summary script added.
- [x] M33 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M33 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M33-E`).

### M33-S6 transition handoff packet acceptance criteria
- Transition packet consumes M33 kickoff/matrix/plan/runtime/convergence artifacts and enforces deterministic selected-track/slice alignment.
- Packet summary exposes kickoff focus, selected track/slice, runtime status, and convergence outcome.
- Closure audit includes dedicated `M33-F` gate.

### M33-S6 tracking (live status)
- [x] M33 transition handoff packet script added.
- [x] M33 transition handoff packet contract test added.
- [x] Book chapter documenting M33 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M33-F`).

### M33-S7 closure report acceptance criteria
- Closure report script consumes strict closure gates + M33 handoff packet summary and computes deterministic `overall` + `nextAction`.
- Required gates `M33-A..M33-F` are validated in one canonical closure artifact (`m33Gates[]`).
- Closure audit includes dedicated `M33-G` gate.

### M33-S7 tracking (live status)
- [x] M33 closure report script added.
- [x] M33 closure report contract test added.
- [x] Book chapter documenting M33 closure report added.
- [x] Naming-lock CI and closure gate updated (`M33-G`).

## M34 - Runtime-First Stabilization Loop (In Progress)

### Goal
- Start M34 from deterministic M33 closure evidence and continue runtime-first stabilization with strict closure-gated slices.

### M34-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M33 closure report + M33 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M33 closure report JSON from `build-m33-closure-report.sh`.
- Closure audit includes dedicated `M34-A` gate.

### M34-S1 tracking (live status)
- [x] M34 kickoff brief script added.
- [x] M34 kickoff brief contract test added.
- [x] Book chapter documenting M34 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M34-A`).

### M34-S2 priority matrix acceptance criteria
- Priority matrix script consumes M34 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M34-B` gate.

### M34-S2 tracking (live status)
- [x] M34 priority matrix script added.
- [x] M34 priority matrix contract test added.
- [x] Book chapter documenting M34 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M34-B`).

### M34-S3 runtime-first de-stub plan acceptance criteria
- Runtime-first plan consumes M34 kickoff + priority matrix artifacts and emits deterministic db/fs/net/validator/secrets execution ordering.
- Plan output includes explicit rationale + closure metadata for follow-up execution slices.
- Closure audit includes dedicated `M34-C` gate.

### M34-S3 tracking (live status)
- [x] M34 runtime-first de-stub plan script added.
- [x] M34 runtime-first de-stub plan contract test added.
- [x] Book chapter documenting M34 runtime-first de-stub plan added.
- [x] Naming-lock CI and closure gate updated (`M34-C`).

### M34-S4 runtime de-stub execution runner acceptance criteria
- Runtime execution runner consumes M34 de-stub plan and executes only the selected first runtime slice with deterministic dry-run + execute modes.
- Runner emits execution-status artifacts with selected slice/domain metadata and closure gate annotations.
- Closure audit includes dedicated `M34-D` gate.

### M34-S4 tracking (live status)
- [x] M34 runtime de-stub execution runner script added.
- [x] M34 runtime de-stub execution runner contract test added.
- [x] Book chapter documenting M34 runtime de-stub execution runner added.
- [x] Naming-lock CI and closure gate updated (`M34-D`).

### M34-S5 executed-slice convergence summary acceptance criteria
- Executed-slice convergence summary consumes M34 plan + runtime execution artifacts and verifies selected track/slice alignment.
- Summary emits deterministic markdown/json output with explicit convergence status and chosen follow-up action.
- Closure audit includes dedicated `M34-E` gate.

### M34-S5 tracking (live status)
- [x] M34 executed-slice convergence summary script added.
- [x] M34 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M34 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M34-E`).

### M34-S6 transition handoff packet acceptance criteria
- Transition packet consumes M34 kickoff/matrix/plan/runtime/convergence artifacts and enforces deterministic selected-track/slice alignment.
- Packet summary exposes kickoff focus, selected track/slice, runtime status, and convergence outcome.
- Closure audit includes dedicated `M34-F` gate.

### M34-S6 tracking (live status)
- [x] M34 transition handoff packet script added.
- [x] M34 transition handoff packet contract test added.
- [x] Book chapter documenting M34 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M34-F`).

### M34-S7 closure report acceptance criteria
- Closure report script consumes strict closure gates + M34 handoff packet summary and computes deterministic `overall` + `nextAction`.
- Required gates `M34-A..M34-F` are validated in one canonical closure artifact (`m34Gates[]`).
- Closure audit includes dedicated `M34-G` gate.

### M34-S7 tracking (live status)
- [x] M34 closure report script added.
- [x] M34 closure report contract test added.
- [x] Book chapter documenting M34 closure report added.
- [x] Naming-lock CI and closure gate updated (`M34-G`).

### Next planned slice
- M35-S1 kickoff brief + closure gate `M35-A` (completed in this revision).

## M35 - Runtime-First Stabilization Loop (In Progress)

### Goal
- Start M35 from deterministic M34 closure evidence and continue runtime-first stabilization with strict closure-gated slices.

### M35-S1 kickoff brief acceptance criteria
- Kickoff brief script consumes M34 closure report + M34 transition packet artifacts and emits deterministic markdown/json output.
- Script auto-generates missing M34 closure report JSON from `build-m34-closure-report.sh`.
- Closure audit includes dedicated `M35-A` gate.

### M35-S1 tracking (live status)
- [x] M35 kickoff brief script added.
- [x] M35 kickoff brief contract test added.
- [x] Book chapter documenting M35 kickoff brief added.
- [x] Naming-lock CI and closure gate updated (`M35-A`).

### M35-S2 priority matrix acceptance criteria
- Priority matrix script consumes M35 kickoff brief output and computes deterministic runtime/release/editor rankings.
- Matrix emits markdown/json outputs with explicit score ordering and rationales.
- Closure audit includes dedicated `M35-B` gate.

### M35-S2 tracking (live status)
- [x] M35 priority matrix script added.
- [x] M35 priority matrix contract test added.
- [x] Book chapter documenting M35 priority matrix added.
- [x] Naming-lock CI and closure gate updated (`M35-B`).

### M35-S3 runtime-first de-stub plan acceptance criteria
- Runtime-first plan consumes M35 kickoff + priority matrix artifacts and emits deterministic db/fs/net/validator/secrets execution ordering.
- Plan output includes explicit rationale + closure metadata for follow-up execution slices.
- Closure audit includes dedicated `M35-C` gate.

### M35-S3 tracking (live status)
- [x] M35 runtime-first de-stub plan script added.
- [x] M35 runtime-first de-stub plan contract test added.
- [x] Book chapter documenting M35 runtime-first de-stub plan added.
- [x] Naming-lock CI and closure gate updated (`M35-C`).

### M35-S4 runtime de-stub execution runner acceptance criteria
- Runtime execution runner consumes M35 de-stub plan and executes only the selected first runtime slice with deterministic dry-run + execute modes.
- Runner emits execution-status artifacts with selected slice/domain metadata and closure gate annotations.
- Closure audit includes dedicated `M35-D` gate.

### M35-S4 tracking (live status)
- [x] M35 runtime de-stub execution runner script added.
- [x] M35 runtime de-stub execution runner contract test added.
- [x] Book chapter documenting M35 runtime de-stub execution runner added.
- [x] Naming-lock CI and closure gate updated (`M35-D`).

### M35-S5 executed-slice convergence summary acceptance criteria
- Convergence summary script consumes M35 runtime plan + execution artifacts and enforces selected track/slice consistency.
- Summary emits deterministic JSON/markdown outputs with `executionPass`, `overall`, and next-action guidance.
- Closure audit includes dedicated `M35-E` gate.

### M35-S5 tracking (live status)
- [x] M35 executed-slice convergence summary script added.
- [x] M35 executed-slice convergence summary contract test added.
- [x] Book chapter documenting M35 executed-slice convergence summary added.
- [x] Naming-lock CI and closure gate updated (`M35-E`).

### M35-S6 transition handoff packet acceptance criteria
- Transition handoff packet script consumes M35 kickoff/matrix/plan/runtime/convergence artifacts and enforces selected track/slice consistency.
- Handoff packet emits deterministic copied artifacts and `handoff-packet.json` manifest with closure metadata.
- Closure audit includes dedicated `M35-F` gate.

### M35-S6 tracking (live status)
- [x] M35 transition handoff packet script added.
- [x] M35 transition handoff packet contract test added.
- [x] Book chapter documenting M35 transition handoff packet added.
- [x] Naming-lock CI and closure gate updated (`M35-F`).

### M35-S7 closure report acceptance criteria
- Closure report script consumes strict closure gates + M35 transition handoff packet summary and computes deterministic `overall` + `nextAction`.
- Required gates `M35-A..M35-F` are validated in one canonical closure artifact (`m35Gates[]`).
- Closure audit includes dedicated `M35-G` gate.

### M35-S7 tracking (live status)
- [x] M35 closure report script added.
- [x] M35 closure report contract test added.
- [x] Book chapter documenting M35 closure report added.
- [x] Naming-lock CI and closure gate updated (`M35-G`).

### M39-S2C editor formatting and plugin smoke fixture acceptance criteria
- Language server advertises and serves deterministic `textDocument/formatting`.
- Zed plugin path includes a maintained smoke project for diagnostics/navigation/rename/format verification.
- Book chapter captures implementation + operator test flow.

### M39-S2C tracking (live status)
- [x] Added LSP `documentFormattingProvider` capability + `textDocument/formatting` handler.
- [x] Added formatting coverage in `compiler/sec4-lsp/src/main.rs` tests.
- [x] Added `examples/zed-plugin-smoke` project with plugin-focused operator checklist.
- [x] Added book chapter `1036-m39-zed-plugin-formatting-and-smoke-project.md`.

### M39-S2D Zed plugin runtime-launch hardening acceptance criteria
- Zed extension resolves `sec4audit-language-server` with deterministic precedence:
  - settings override path,
  - PATH binary,
  - workspace-local fallback build paths.
- Missing-binary failure mode returns actionable diagnostics with explicit setting key and build command.
- Operator docs describe launch precedence and override configuration.

### M39-S2D tracking (live status)
- [x] Added launch resolution precedence in `zed-extension/src/lib.rs` (settings -> PATH -> local fallback paths).
- [x] Added clear configured-path and missing-binary diagnostics.
- [x] Added operator docs for resolution order + settings override.
- [x] Added book chapter `1040-m39-zed-plugin-runtime-launch-hardening.md`.

### M39-S2E Zed plugin smoke runner script acceptance criteria
- Repository includes a single-command smoke runner for `examples/zed-plugin-smoke`.
- Smoke runner validates grammar pin, sample `sec4 check`, and deterministic formatter output without mutating tracked fixtures.
- Optional fast mode is available for quick local loops.

### M39-S2E tracking (live status)
- [x] Added `scripts/run-zed-plugin-smoke.sh`.
- [x] Added deterministic formatter assertion in temp project copy.
- [x] Added `--fast`/`FAST=1` mode for lighter operator loops.
- [x] Updated Zed extension docs to point to smoke runner command.
- [x] Added book chapter `1041-m39-zed-plugin-smoke-runner-script.md`.

### M39-S2F Zed extension release packaging checklist acceptance criteria
- Repository includes a deterministic release checklist command for Zed extension publishing readiness.
- Checklist verifies extension manifest wiring, grammar pin validity, and plugin smoke runner success.
- Operator-facing docs reference the checklist as canonical pre-release validation.

### M39-S2F tracking (live status)
- [x] Added `scripts/check-zed-extension-release.sh`.
- [x] Included manifest token/wiring assertions for `zed-extension/extension.toml`.
- [x] Wired grammar-pin + fast smoke runner execution into checklist.
- [x] Updated extension README validation section to use release checklist command.

### M39-S2G Zed extension operator install guide acceptance criteria
- Repository includes deterministic operator workflow for:
  - local install,
  - local update,
  - rollback to previous local install.
- Install workflow resolves platform-local extension install paths with override support.
- Operator docs include explicit install/update/rollback commands and backup location notes.

### M39-S2G tracking (live status)
- [x] Added `scripts/manage-zed-extension-local.sh` with `install|update|rollback|status`.
- [x] Added platform-default extension directory resolution + override support (`--extensions-dir`).
- [x] Added timestamped local backup flow for safe rollback.
- [x] Updated `zed-extension/README.md` with local operator install/update/rollback guide.

### M39-S2H Zed extension end-to-end operator smoke acceptance criteria
- Repository includes one-command operator smoke for clean-profile extension flow:
  - install,
  - plugin smoke validation,
  - rollback.
- Smoke flow verifies rollback restoration against seeded previous install state.
- Operator docs reference the command as the canonical full local-flow check.

### M39-S2H tracking (live status)
- [x] Added `scripts/run-zed-extension-operator-smoke.sh`.
- [x] Wired install -> plugin smoke (`--fast`) -> rollback flow using `scripts/manage-zed-extension-local.sh`.
- [x] Added rollback restoration assertion via seeded marker file.
- [x] Updated `zed-extension/README.md` with full operator-flow validation command.

### M39-S2I Zed extension publish bundle staging helper acceptance criteria
- Repository includes deterministic local bundle staging command for Zed extension publish prep.
- Staging command emits a machine-readable manifest with per-file size/hash inventory.
- Operator docs include bundle staging command and output artifact location.

### M39-S2I tracking (live status)
- [x] Added `scripts/stage-zed-extension-bundle.sh`.
- [x] Added deterministic bundle staging under `build/zed-extension-bundle/`.
- [x] Added `bundle-manifest.json` emission with per-file `path/size/sha256`.
- [x] Updated `zed-extension/README.md` with bundle staging usage.

### M39-S2J Zed extension local operator checklist consolidation acceptance criteria
- Repository includes a single entrypoint command for local operator readiness.
- Entry command wraps release checklist (without duplicate smoke), operator smoke, and bundle stage.
- Command emits deterministic pass/fail outcome and validates staged manifest presence.

### M39-S2J tracking (live status)
- [x] Added `scripts/check-zed-extension-operator-readiness.sh`.
- [x] Wrapped `check-zed-extension-release.sh --skip-smoke`, `run-zed-extension-operator-smoke.sh`, and `stage-zed-extension-bundle.sh --clean`.
- [x] Added staged manifest existence assertion in readiness flow.
- [x] Updated `zed-extension/README.md` with one-command readiness entrypoint.

### Next planned slice
- M39-S2K Wire Zed operator readiness command into release-operator handoff docs and lane prompts.

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

1. Keep execution in the alpha scope-reset mode (`implementation-first`, targeted tests, governance suites at PR end/CI).
2. Implement `M39-S2A` project-local multi-file module resolver with deterministic missing/ambiguous module diagnostics.
3. Start `M39-S2B` LASM async backend bootstrap lane (target skeleton + runtime loop baseline) with bounded weekly allocation.

---

This roadmap is the canonical execution path until v0.1-alpha is running and documented as a coherent book.
