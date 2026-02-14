# M15 Runtime Replay Stubbing Scope and Plan

M15 moves replay from bootstrap contract verification into deterministic runtime stub execution for `--effects mock`.

## What this milestone is

A runtime-focused replay milestone that executes net/db/fs stubs deterministically and reports stable, machine-checkable replay execution evidence.

## Why this milestone exists

M14 proved replay contract correctness and CI guardability.
M15 is required to convert that contract baseline into practical replay execution behavior teams can use for deterministic debugging and incident triage.

## Scope (locked for M15)

Included:
- deterministic runtime net/db/fs stub materialization during replay,
- deterministic replay execution diagnostics and output contracts,
- CI/closure guardrails for runtime replay contract drift.

Deferred:
- multi-service distributed replay orchestration,
- large-scale capture recording expansion,
- non-deterministic adaptive replay modes.

## Implementation slices (planned)

1. Runtime net stub materialization.
2. Runtime DB stub materialization.
3. Runtime FS stub materialization.
4. Replay runtime diagnostics contract.
5. Replay runtime output contract guard + CI closure enforcement.

## Acceptance criteria

- Replay `mock` mode executes deterministic runtime stub materialization (not only precheck validation).
- Replay text/json outputs include deterministic executed-stub summary fields.
- Runtime replay failures are signature-backed with stable diagnostics.
- CI/closure guards fail if runtime replay contracts regress.

## Risks and mitigations

Risk:
- runtime output drift across slices.
Mitigation:
- add field-presence and deterministic-order guard scripts per slice.

Risk:
- ambiguous stub selection.
Mitigation:
- preserve and enforce signature uniqueness constraints from M14.

Risk:
- mode confusion between `deny`/`mock`/`allow`.
Mitigation:
- keep explicit mode diagnostics and per-mode contract tests.

## Immediate next step

Implement first runtime net stub materialization slice with deterministic JSON/text output deltas and guard tests.
