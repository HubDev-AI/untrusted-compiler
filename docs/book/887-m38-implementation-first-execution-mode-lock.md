# 887 M38 Slice: Implementation-First Execution Mode Lock

This chapter records the execution-mode change agreed for ongoing work.

## Why this lock exists

Recent progress showed a repeated failure mode:

- too much time spent in governance/gate loops,
- too many test-only slices without enough runtime/compiler behavior movement.

To keep alpha progress practical, implementation throughput is now the primary optimization target.

## Effective operating contract (from this slice onward)

Every normal development PR must include a real logic delta in at least one of:

- compiler semantic/type/effect/security behavior,
- runtime C behavior,
- CLI behavior that affects real user execution paths.

## Slice acceptance rules

1. A PR is **implementation-first** when it changes runtime/compiler/CLI logic, not only tests or gate scripts.
2. Tests must be **targeted to touched logic**. Avoid broad gate suites during active coding loops.
3. Governance checks are **bounded** in daily iteration:
   - run focused implementation validation first,
   - run governance/closure suites once near PR completion (or rely on CI).
4. Documentation must be updated for each meaningful behavior change:
   - book chapter or existing chapter delta,
   - roadmap/README alignment when workflow contracts change.

## Explicit non-goals during implementation loops

- No gate-only churn unless fixing a real failing CI contract.
- No large “test harness refactor only” PRs without behavior movement.
- No repeated re-running of the full naming-lock contract bundle after each small logic edit.

## Default local validation order

1. `cargo run -p sec4 -- check --path <target>`
2. `cargo run -p sec4 -- build --path <target> --emit c-bin`
3. Targeted tests for touched behavior (for example specific `cargo test -p sec4 --test ...` filters)
4. Route/runtime smoke for affected endpoints when runtime logic changes
5. Governance bundle/checks once before PR handoff:
   - `scripts/run-naming-lock-contract-suite.sh --dry-run`
   - closure audit checks as needed by the touched milestone scope

## Progress intent

This lock is meant to keep work weighted toward real language/runtime capabilities and reduce time lost in low-yield gate loops.
