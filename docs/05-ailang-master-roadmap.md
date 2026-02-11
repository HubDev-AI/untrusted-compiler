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
  - `sec.audit` includes deterministic allowlist hygiene findings (high-risk bypasses, expiry/soon-expiry, and exception-count posture signal).
  - Parser/semantic/security-map now support dotted stdlib call names (`db.exec`, `req.json`, `cors.withCors`, etc.) in addition to underscore intrinsic aliases.
  - Semantic flow checks now reject `Secret<_>`/`Untrusted<_>` values across log, JSON, SQL, URL/net, filesystem, and header/cookie sinks with explicit diagnostics (`E1002`, `E1003`, `E1004`, `E1005`).
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
  - `sec.audit` now emits deterministic logging/SQL posture findings:
    - `LOG_STRUCTURED_ONLY_DISABLED`
    - `LOG_REMOTE_IP_ENABLED`
    - `LOG_USER_AGENT_ENABLED`
    - `SQL_RAW_ALLOWED_BY_POLICY`
    - `SQL_LIMIT_RULE_DISABLED`
  - policy parser now validates `sql.require_limit_on_select` values (`off|warn|enforce`) with dedicated diagnostics.
- Security posture specs were expanded with:
  - typed security middleware baseline (`CORS + security headers + CSRF + auth`),
  - deterministic `sec.audit` contract,
  - compiler-emitted security metadata tags for robust audit tooling.
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

1. Extend capability/sink enforcement from intrinsic calls to typed stdlib API symbols.
2. Add first-class trust-gate flow checks (`req.json(schema)`) and origin-trace sink diagnostics.
3. Expand deterministic `sec.audit` findings for SQL/logging/privacy posture and richer callsite evidence.
4. Prepare M10 benchmark harness scaffold once M9 stability gate is reached.
5. Prepare M11 editor tooling scaffold once semantic outputs are stabilized for LSP use.

---

This roadmap is the canonical execution path until v0.1-alpha is running and documented as a coherent book.
