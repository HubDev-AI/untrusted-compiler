# 212 Road to v0.2

This chapter outlines the next-step direction after v0.1-alpha stabilization.

## Immediate next milestone sequence

1. Complete M10 benchmarking and comparative validation.
2. Resume M11 editor tooling and Zed integration on stabilized semantics.
3. Fold benchmark + tooling outcomes into v0.2 scope planning.

## v0.2 technical priorities

## 1) Performance and operability

- Land reproducible benchmark harness as part of regular CI/perf workflows.
- Publish cross-language comparative reports with raw artifacts.
- Expand runtime instrumentation and failure-mode observability.

## 2) Tooling maturity

- Complete LSP feature set (definition/references/hover/completion/rename/code actions) at production reliability.
- Tighten incremental analysis budgets and large-workspace behavior.
- Ship first-class Zed integration and baseline tree-sitter queries.

## 3) Build and supply-chain hardening

- Extend lock strategy from manifest-level determinism to richer dependency graph pinning.
- Expand SBOM depth and release provenance automation.
- Introduce stricter release checks around policy profile usage and audit baselines.

## 4) Security model deepening

- Add richer policy exception governance (ticketing/expiry workflows at scale).
- Continue hardening sensitive sink/source coverage where bridge-stage behavior remains.
- Improve security diagnostic traces for IDE quick-fix workflows.

## 5) Runtime ergonomics

- Improve handler/runtime contracts beyond current bridge constraints.
- Expand sample-service coverage for recommended architecture patterns.
- Keep security-first defaults mandatory while reducing developer friction.

## Exit criteria proposal for v0.2 planning gate

- Benchmark suite is reproducible and published.
- Editor tooling supports daily development workflows end-to-end.
- Release pipeline includes lock validation, metadata, SBOM, and audit gating by default.
- Major bridge-stage constraints have explicit graduation or documented defer decisions.

## Non-goals carried forward

- No trait/macro complexity unless directly justified by security/runtime goals.
- No dilution of typed sink/trust/effect/capability guarantees.
