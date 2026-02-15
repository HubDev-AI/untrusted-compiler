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
| `M16-C` | PASS | Runtime-smoke workflow runs dual-branch sec4 run smoke on pull_request + main push, validates artifacts, builds branch index, and uploads runtime artifacts | `.github/workflows/runtime-smoke.yml` |
| `M16-D` | PASS | Naming-lock CI enforces runtime-smoke workflow contract + guard tests + artifact checker/index/bundle tests | `.github/workflows/naming-lock.yml` |
| `M16-E` | PASS | Naming-lock CI enforces sec4 run runtime-flag contract + guard tests | `.github/workflows/naming-lock.yml` |
| `M17-A` | PASS | Naming-lock CI enforces M17 operator handoff readiness checker | `.github/workflows/naming-lock.yml` |
| `M17-B` | PASS | Naming-lock CI enforces M17 operator bootstrap profile helper | `.github/workflows/naming-lock.yml` |
| `M17-C` | PASS | Naming-lock CI enforces M17 operator troubleshooting matrix | `.github/workflows/naming-lock.yml` |
| `M17-D` | PASS | Naming-lock CI enforces M17 operator handoff quickstart | `.github/workflows/naming-lock.yml` |
| `M17-E` | PASS | Naming-lock CI enforces M17 operator handoff CI smoke wrapper | `.github/workflows/naming-lock.yml` |
| `M17-F` | PASS | Operator-handoff workflow executes CI smoke wrapper + artifact upload on pull_request + main push | `.github/workflows/operator-handoff-smoke.yml` |
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
  - release-contract-smoke workflow contract test now also enforces trigger coverage (`pull_request` + `push` on `main`).
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
- M30-S2 priority matrix + closure gate `M30-B`.

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
- M30-S2 priority matrix + closure gate `M30-B`.

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

### Next planned slice
- M30-S2 priority matrix + closure gate `M30-B`.

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

1. Implement M30-S2 priority matrix from M30 kickoff brief and wire `M30-B`.
2. Continue M30 runtime-first stabilization sequence (`S3..S7`) with strict closure gating parity.

---

This roadmap is the canonical execution path until v0.1-alpha is running and documented as a coherent book.
