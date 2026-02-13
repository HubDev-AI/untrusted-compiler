# 210 Release Notes and Compatibility (v0.1-alpha)

This chapter captures the current v0.1-alpha release posture after M9 hardening slices.

## Release highlights

- Security-first compiler checks for typed sinks, trust boundaries, secrets, effects, capabilities, and policy-driven restrictions.
- Deterministic security posture reporting via `sec4 audit` and `security_map` metadata.
- Deterministic lock strategy:
  - `sec4 build` writes deterministic `sec4.lock`,
  - `sec4 build --locked` validates lock freshness.
- Deterministic build provenance artifacts:
  - `build/build_metadata.json`,
  - optional `build/sbom.json` via `--sbom`.
- Canonical policy profiles committed as runnable artifacts:
  - `policies/default-secure-prod.sec4.policy`,
  - `policies/permissive-dev.sec4.policy`.

## Compatibility contract

## CLI compatibility (alpha)

Stable in v0.1-alpha:
- `check`, `build`, `run`, `audit`, `gate`, `explain`
- `build --emit mir|mir-json|c|c-bin`
- `build --locked`
- `build --sbom`

Alpha-stable behavior expectations:
- deterministic diagnostic codes,
- deterministic `sec4 audit` finding IDs/severities for same policy+inputs,
- deterministic `build_metadata.json` and `sbom.json` for same inputs.

May still evolve within alpha:
- exact text formatting of human-readable output,
- optional/auxiliary output lines that do not alter JSON mode contracts.

## Artifact compatibility

Stable artifact names:
- `build/security_map.json`
- `build/build_metadata.json`
- `build/sbom.json` (when `--sbom` enabled)
- `sec4.lock`

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
2. `scripts/check-naming-lock.sh` passes.
3. `sec4 build --locked` passes on sample projects.
4. `sec4 build --sbom` produces deterministic SBOM output.
5. `sec4 audit --format json` passes for `default-secure-prod` profile at required threshold.
6. `build_metadata.json` and `sbom.json` are captured for release artifacts.
7. Known-limits and roadmap chapters are reviewed with release notes.

Canonical automation:
- `scripts/release-alpha-gate.sh`

This script executes the checklist end-to-end against sample projects using the secure policy profile, validates naming lock compliance, verifies deterministic `build_metadata.json` + `sbom.json` hashes across repeated builds, verifies metadata identity-hash consistency with `sec4 audit` output (`policyHash`, `compilerHash`, `runtimeHash`), gates `sec4 audit` at `risk>=HIGH`, and captures release artifacts under `build/release-alpha-gate/`.

Release gate artifact stamping includes:
- copied active policy profile file in release artifact directory,
- policy profile SHA256 in `checksums.txt` and `summary.txt`,
- explicit `naming lock: PASS` in `summary.txt`,
- `sec4` binary SHA256 in `checksums.txt` and `summary.txt`,
- runtime ABI source/header SHA256 in `checksums.txt` and `summary.txt`,
- copied runtime ABI files (`sec4_runtime.h`, `sec4_runtime.c`) in release artifacts,
- verified identity-hash stamps (`policy identity`, `compiler identity`, `runtime identity`) in `checksums.txt` and `summary.txt`.

CI wiring:
- `.github/workflows/alpha-release-gate.yml`
- `.github/workflows/naming-lock.yml`

The workflow runs the same release gate on manual dispatch and alpha tag pushes, then uploads captured artifacts from `build/release-alpha-gate/`.
The naming-lock workflow runs `scripts/check-naming-lock.sh` on pull requests and pushes to `main`.

## Upgrade notes

From earlier milestone snapshots to current alpha:
- lockfile format changed from stub-only to deterministic package+build fingerprint content,
- build flow now emits metadata by default and optional SBOM on demand,
- diagnostics include source snippets and tags in text mode.
