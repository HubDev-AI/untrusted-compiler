# AILang Master Roadmap (From Line 1 to Minimal Real Working Compiler)

This roadmap is the execution plan for building a minimal, real, runnable AILang implementation.

It explicitly includes a parallel documentation workflow so `docs/` evolves into a book-quality project history and technical guide.

## Current Status (2026-02-11)

- M0 bootstrap started and initial foundation implemented.
- Rust workspace, CLI skeleton, manifest validation, diagnostics framework, and golden test scaffolding are in place.
- M0 book chapters were added under `docs/book/`.

## 0. Product Direction (Locked Constraints)

These constraints come from current AILang docs and the new files:
- `/Users/vladimirtrifonov/src/ai/AILang/docs/06-ailang-typescript-like-profile.md`
- `/Users/vladimirtrifonov/src/ai/AILang/docs/07-ailang-no-inheritance-composition-model.md`

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
- `Untrusted<T>` and `Secret<T>` semantics (or equivalent enforced boundary model).
- Typed sinks (`SqlQuery`, `HtmlSafe`, etc.).
- Security policies and lints integrated into compilation pipeline.

## 1. Definition of Done (Minimal Real Working AILang)

A minimal real working AILang (v0.1-alpha) means:
- Compiler CLI exists and builds a runnable binary from `.ai` source.
- Lexer + parser + name resolution + minimal type checking are operational.
- MIR lowering exists and can be emitted for inspection.
- One backend exists (C emission first), producing working executables.
- Runtime ABI supports at least: strings/bytes, JSON parse/encode, HTTP serve, simple logging.
- End-to-end app works: `GET /health` and one JSON `POST` endpoint.
- Security gates/sinks are enforced for at least one path (e.g., SQL + request decode).
- Tests are automated for parser, type checks, MIR, and E2E sample.
- Documentation is updated for every implemented subsystem and can be read as a coherent book.

## 2. Build Strategy (Execution Order)

Implementation order is intentionally linear to reduce thrash:
1. Freeze v0.1-lite language profile.
2. Bootstrap compiler workspace + CLI.
3. Implement frontend pipeline (lexer/parser/AST).
4. Add semantic layer (names/types/effects minimal subset).
5. Lower to MIR + add MIR text output.
6. Implement C backend + runtime ABI stubs.
7. Add HTTP/JSON vertical slice.
8. Add security constraints and lints for chosen sinks.
9. Harden tests, diagnostics, packaging.

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
- Implement module/import resolution.
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
- Wire request boundary model (`Untrusted`/schema decode path).

### Exit criteria
- Compiler rejects undeclared effect use.
- Minimal security boundary rules are enforced in sample service.

### Docs/book outputs
- Chapter: "Effect System and Auditable Side Effects".
- Chapter: "Trust Boundaries and Untrusted Data".

## M4 - MIR Lowering + Introspection
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

## M5 - C Backend + Runtime ABI (First Runnable Target)
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

## M6 - HTTP/JSON Vertical Slice
### Build tasks
- Implement minimal HTTP router/runtime bridge.
- Implement JSON parse/encode primitives and schema decode path.
- Build sample service with `/health` + one typed POST endpoint.

### Exit criteria
- `ailang run examples/hello-api` serves both endpoints.
- E2E tests verify status, decode behavior, and errors.

### Docs/book outputs
- Chapter: "HTTP Runtime and Request Lifecycle".
- Chapter: "Schema-Driven JSON".
- Chapter: "Building Your First AILang API".

## M7 - Security-by-Construction Slice
### Build tasks
- Enforce typed SQL sink path (`sql"..."` to `SqlQuery`).
- Enforce secret handling policy for logging/encoding.
- Add lints/policy config for forbidden patterns.

### Exit criteria
- Compiler/linter rejects representative insecure patterns.
- Security tests included in CI.

### Docs/book outputs
- Chapter: "Security Model".
- Chapter: "Typed Sinks and Safe Boundaries".
- Chapter: "Policy and Lint Rules".

## M8 - Release Hardening
### Build tasks
- Improve diagnostics quality and error explainability.
- Add deterministic build controls and lock strategy.
- Finalize minimal stdlib and sample apps.
- Prepare alpha release checklist.

### Exit criteria
- Repeatable build across clean environments.
- Test suite stable with release tag candidate.

### Docs/book outputs
- Chapter: "Release Notes and Compatibility".
- Chapter: "Known Limits of v0.1-alpha".
- Chapter: "Road to v0.2".

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
- M3 effects boundaries minimal + effects/security chapter update.

Days 11-13:
- M4 MIR + emit + MIR chapter.

Day 14:
- Consolidation, tests, docs cleanup, next sprint planning.

## 7. Immediate Next Actions (Start Here)

1. Freeze v0.1-lite feature matrix from existing docs + two new files.
2. Decide implementation language/toolchain for compiler and runtime.
3. Create repo structure and CLI skeleton (M0).
4. Create `docs/book/` skeleton and wire milestone chapter ownership.
5. Start M1 with parser fixtures first (tests before full parser completion).

---

This roadmap is the canonical execution path until v0.1-alpha is running and documented as a coherent book.
