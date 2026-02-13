# 211 Known Limits of v0.1-alpha

This chapter records explicit technical limits for the v0.1-alpha release line.

## Language/runtime limits

- No full async runtime or async/await surface yet.
- No trait/interface system.
- No macro/derive system.
- Restricted generics only (security + ergonomics scoped use cases).
- Runtime integrations are intentionally bridge-stage in several areas (typed contracts first, richer behavior next).

## Tooling limits

- LSP/editor integration is bootstrap-stage and not yet feature-complete for all workflows.
- Some diagnostics currently show single-line snippets only.
- Security quick-fix automation in editors is planned but not fully delivered in current release line.

## Build/release limits

- Lockfile is deterministic but dependency graph resolution is still minimal.
- SBOM generation is available, but component depth is intentionally shallow until dependency management expands.
- Reproducible metadata is deterministic for current inputs, but full hermetic build guarantees across environments remain a future objective.

## Security model limits

- Policy surface is broad for v0.1, but profile selection is file-based (no dedicated profile selection CLI UX yet).
- Certain advanced posture checks are still rule-based/static and do not replace runtime operational controls.
- Capture/replay and audit are deterministic by design, but broader ecosystem integrations (dashboards, centralized policy registries) are out of scope for v0.1-alpha.

## Benchmarking limits

- Benchmark spec and methodology are documented, but cross-language benchmark publication is part of post-stability milestone work (M10).

## Migration/compatibility limits

- Alpha compatibility focuses on stable diagnostic codes and deterministic artifact schemas.
- Minor CLI text-output wording may still evolve.
- Internal helper APIs can change while external command contracts remain consistent.
