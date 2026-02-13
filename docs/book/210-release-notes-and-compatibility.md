# 210 Release Notes and Compatibility (v0.1-alpha)

This chapter captures the current v0.1-alpha release posture after M9 hardening slices.

## Release highlights

- Security-first compiler checks for typed sinks, trust boundaries, secrets, effects, capabilities, and policy-driven restrictions.
- Deterministic security posture reporting via `ailang sec audit` and `security_map` metadata.
- Deterministic lock strategy:
  - `ailang build` writes deterministic `ailang.lock`,
  - `ailang build --locked` validates lock freshness.
- Deterministic build provenance artifacts:
  - `build/build_metadata.json`,
  - optional `build/sbom.json` via `--sbom`.
- Canonical policy profiles committed as runnable artifacts:
  - `policies/default-secure-prod.ailang.policy`,
  - `policies/permissive-dev.ailang.policy`.

## Compatibility contract

## CLI compatibility (alpha)

Stable in v0.1-alpha:
- `check`, `build`, `run`, `sec audit`
- `build --emit mir|mir-json|c|c-bin`
- `build --locked`
- `build --sbom`

Alpha-stable behavior expectations:
- deterministic diagnostic codes,
- deterministic `sec.audit` finding IDs/severities for same policy+inputs,
- deterministic `build_metadata.json` and `sbom.json` for same inputs.

May still evolve within alpha:
- exact text formatting of human-readable output,
- optional/auxiliary output lines that do not alter JSON mode contracts.

## Artifact compatibility

Stable artifact names:
- `build/security_map.json`
- `build/build_metadata.json`
- `build/sbom.json` (when `--sbom` enabled)
- `ailang.lock`

Versioned schemas:
- `AuditReport.version = "0.1"`
- `BuildMetadata.version = "0.1"`

## Policy compatibility

Policy parser is strict:
- unknown keys fail fast,
- invalid value domains fail fast,
- cross-key security constraints are validated during parse/load.

Canonical profile files in `policies/` are the compatibility reference for current parser coverage.

## Release checklist (alpha gate)

Required before tagging an alpha candidate:
1. `cargo test -q` passes in clean workspace.
2. `ailang build --locked` passes on sample projects.
3. `ailang build --sbom` produces deterministic SBOM output.
4. `ailang sec audit --format json` passes for `default-secure-prod` profile at required threshold.
5. `build_metadata.json` and `sbom.json` are captured for release artifacts.
6. Known-limits and roadmap chapters are reviewed with release notes.

## Upgrade notes

From earlier milestone snapshots to current alpha:
- lockfile format changed from stub-only to deterministic package+build fingerprint content,
- build flow now emits metadata by default and optional SBOM on demand,
- diagnostics include source snippets and tags in text mode.
