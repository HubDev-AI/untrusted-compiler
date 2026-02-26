# Workbench Plan: Prompt-First Cross-Backend Benchmark

Date: 2026-02-26  
Owner: sec4 core/runtime lane

## Goal

Build one larger, realistic backend app contract with real DB behavior, generate equivalent implementations across multiple backends from a single prompt contract, and benchmark all implementations (including sec4/sec4-lasm) under the same conditions.

## Scope

1. One canonical "feature-rich API" contract:
   - auth-required endpoints
   - read/write DB paths
   - transaction path
   - filtered list/search path
   - deterministic error envelopes
2. Prompt-first generation workflow:
   - one canonical generation prompt
   - backend-specific adaptation prompt
   - deterministic acceptance checklist
3. Benchmark execution through existing `benchmark-suite` runners and artifact contracts.

## Out of Scope (for this slice)

1. Full automation of AI generation loop in CI.
2. Performance tuning campaign.
3. New language semantics.

## Backend Matrix (initial)

1. `sec4` (C runtime path)
2. `sec4-lasm`
3. `node`
4. `go`
5. `rust`

## Workbench App Contract (v1)

The app is a "task board + comments + labels" API with real DB usage.

Required characteristics:

1. Auth gate:
   - `Authorization: Bearer <token>` required for all mutating endpoints.
2. DB write path:
   - create task
   - create comment
3. DB read path:
   - get task by id
   - list tasks with pagination/filter
4. Transaction path:
   - create task + initial comment atomically.
5. Deterministic response envelope:
   - success: `{ ok, status, traceId, timeMs, data }`
   - error: `{ ok:false, status, traceId, timeMs, error:{ code, kind, message } }`

## Prompt-First Generation Flow

### Phase A: Contract freeze

1. Write canonical API + DB schema contract (`benchmark-suite/workbench/spec/feature-app-v1.md`).
2. Freeze deterministic envelope + validation/error codes.
3. Freeze benchmark request mix and payload set.

### Phase B: Generate implementations

1. Use canonical prompt (`benchmark-suite/workbench/prompts/generate-feature-app-v1.md`).
2. Generate one implementation per backend into:
   - `benchmark-suite/services/<impl>-workbench/`
3. Run backend-specific smoke checks.
4. Reject generated code that violates contract or introduces placeholder behavior.

### Phase C: Benchmark and compare

1. Add workbench endpoint profiles/load scripts.
2. Run fixed-target and step profiles across all implementations.
3. Build compare matrix and report using current artifact flow.
4. Publish summary with throughput, p95/p99, error-rate, and RSS.

## Execution Order

1. Define/freeze contract and generation prompt.
2. Implement sec4/sec4-lasm reference first.
3. Generate node/go/rust against frozen prompt and contract.
4. Run parity checks.
5. Run benchmark matrix.

## Guardrails

1. No per-backend feature drift; contract parity is mandatory.
2. No synthetic in-memory fake DB for DB endpoints; real Postgres path required for matrix runs.
3. Deterministic error envelopes across all backends.
4. Same load profile parameters and host constraints across implementations.

## Deliverables

1. `benchmark-suite/workbench/spec/feature-app-v1.md`
2. `benchmark-suite/workbench/prompts/generate-feature-app-v1.md`
3. `benchmark-suite/workbench/matrix.backends.json`
4. Benchmark report section: "Workbench Feature App Matrix"

## Immediate Next Steps

1. Land workbench scaffold files (contract + prompts + matrix config).
2. Add sec4/sec4-lasm workbench service skeletons.
3. Add one dry-run generation pass for node/go/rust.
