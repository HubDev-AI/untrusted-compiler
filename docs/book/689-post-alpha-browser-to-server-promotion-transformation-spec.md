# Post-Alpha Browser-to-Server Promotion Transformation Spec

## What it is

This chapter defines the deterministic transformation contract for:

- browser-first prototype profile (`zero deploy`)
- mechanical promotion to real server target (`sec4 promote --from browser --to server`).

The goal is to reuse domain logic unchanged while swapping target adapters and generating server runtime scaffolding.

## Why it exists

AI app builders spend too many tokens on backend glue, deployment setup, and repeated fix loops.

A browser-first backend profile reduces that cost during prototyping. Promotion then converts a working local prototype into a deployable service with deterministic compiler-guided rewrites instead of manual backend rewrites.

## Scope

### MVP (first post-alpha delivery)

- Browser profile + capability restrictions.
- Domain/repo/adapters architecture contract.
- `sec4 promote --from browser --to server` transformation.
- Generated server scaffolding (repo adapter, schema/migrations skeleton, runtime entry wiring).
- Promotion diagnostics and precondition checks.

### Later

- local export -> server import plan generator with migration assistant.
- sync/conflict primitives for local-first promotion flows.
- richer deployment target generation (edge/container presets).

## Required architecture contract

Promotion requires project code to be split into three layers:

1. Domain module (target-agnostic):
   - business logic + validation rules.
   - no browser-only or server-only effects.
2. Repository interface:
   - abstract methods (`get/put/list/...`) used by domain.
3. Target adapters:
   - browser adapter using `localdb.*`.
   - server adapter using `db.*`.

Composition root is the only place where concrete adapter binding is chosen.

## CLI contract

```text
sec4 promote --from browser --to server [--path <project>] [--dry-run] [--out <dir>]
```

Behavior:

- `--dry-run`: emit transformation plan and diagnostics only.
- default: apply rewrite + generate artifacts.

Determinism requirements:

- same input tree + policy + CLI args => byte-identical plan output.
- generated file ordering and import ordering are stable.

## Promotion pipeline

1. Load project + policy + module graph.
2. Validate promotion preconditions.
3. Compute transformation plan.
4. Rewrite composition bindings.
5. Generate server artifacts.
6. Emit report (text/json) with changed files + warnings.

## Promotion precondition checks

Promotion fails (non-zero exit) if any condition is violated:

1. Domain module imports browser-only APIs.
2. Domain module imports server-only APIs.
3. Composition root does not bind through repository interface.
4. Browser adapter/server adapter missing required repository methods.
5. Browser profile still contains forbidden server capabilities.

Diagnostic expectations:

- stable code
- span-aware location
- exact failing contract + suggested fix.

## Rewrite rules (MVP)

At composition root only:

1. Replace adapter binding:
   - `LocalRepo(...)` -> `ServerRepo(...)`
2. Keep domain function signatures unchanged.
3. Keep route handler domain callsites unchanged.
4. Add required server capabilities/effects at bootstrap boundary only.

No rewrite inside domain module bodies.

## Generated artifacts (MVP)

Promotion may create:

- `src/adapters/server_repo.ut`
- `db/schema.sql`
- `db/migrations/0001_init.sql`
- `deploy/sec4.server.toml` (baseline runtime/deploy profile)
- `build/promotion-report.json`

If files already exist, generator is idempotent and only updates marked sections or exits with deterministic conflict diagnostics.

## Data portability contract

Browser-to-server promotion must include explicit data transfer path:

- browser export command produces deterministic JSON snapshot.
- server import scaffold validates schema + version before write.

MVP allows manual import execution, but artifact format and validation must be deterministic.

## Security rules

### Browser profile

- forbidden: `db.*`, `secrets.*`, `net.listen`, internal-net sinks.
- allowed: `localdb.*`, constrained public fetch gates, UI/session-safe local flows.

### Server profile

- allows `db.*`/server runtime sinks under policy.
- requires explicit policy for auth/cors/csrf/security headers.

Important boundary:

- browser must never embed service DB credentials.
- direct browser-to-DB access is only acceptable with explicit user-scoped auth + policy model designed for that exposure.

## Real use-case fit

Best fit for browser-first:

1. CRUD prototypes and internal workflow demos.
2. Form-heavy apps with validation and local persistence.
3. OAuth-personal dashboards that call user-scoped APIs.
4. AI-generated proof-of-concepts where infra setup is the main token sink.

Promotion triggers:

1. multi-user shared state,
2. webhooks/background jobs,
3. centralized auth/audit/rate limiting,
4. production reliability/ops requirements.

## Acceptance criteria

1. Browser profile can run prototype without deploy.
2. `sec4 promote` rewrites adapter binding only at composition boundary.
3. Domain modules remain byte-identical after promotion.
4. Server scaffold builds/runs with generated baseline artifacts.
5. Promotion report is deterministic across repeated runs.
6. Invalid project shape yields deterministic diagnostics.

## Trade-offs

- Requires stronger project structure discipline (domain/repo/adapters).
- Adds compiler transformation complexity, but reduces repeated AI fix loops and manual backend rewrites.
- MVP generation is scaffold-level; production-grade infra templates can evolve iteratively.

## Next

1. Add explicit post-alpha milestone slices for:
   - browser profile enforcement,
   - promotion analyzer + rewrite engine,
   - scaffold generation contracts,
   - data export/import bridge.
2. Define diagnostics taxonomy for promotion failures and quick-fix suggestions.
3. Add end-to-end golden fixtures: browser prototype -> promoted server build/run.
