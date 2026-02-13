# 313 M13 Release Promotion Playbook

This playbook defines the promotion-ready operator flow that binds release-gate artifacts to downstream publication inputs.

## Scope

Applies to v0.1-alpha promotion candidates produced from the main repository workflow.

## Required inputs

- Repository checkout at release candidate commit.
- Toolchain prerequisites available (`cargo`, `jq`, sha256 tool: `shasum` or `sha256sum`).
- Policy profile artifact in repository (`policies/default-secure-prod.sec4.policy`).

## Promotion flow (canonical)

1. Run alpha release gate

```bash
scripts/release-alpha-gate.sh
```

Outputs under `build/release-alpha-gate/` include:
- copied policy/runtime artifacts,
- per-sample metadata/sbom/security_map/audit outputs,
- `checksums.txt`,
- `summary.txt`.

2. Verify promotion inputs

```bash
scripts/verify-release-promotion-inputs.sh
```

Verifier enforces:
- checksum integrity for copied artifacts,
- summary/checksum identity consistency,
- metadata/audit identity consistency for each sample,
- per-sample metadata/sbom checksum consistency.

3. Confirm naming lock

```bash
scripts/check-naming-lock.sh
```

4. Generate publish-consumption manifest

```bash
scripts/generate-release-publish-manifest.sh
```

5. Verify publish-manifest consumption contract

```bash
scripts/verify-release-publish-manifest.sh
```

6. Record release evidence

Capture artifacts from `build/release-alpha-gate/` and attach/upload as promotion evidence.

## CI-equivalent flow

`alpha-release-gate.yml` now executes:
1. `scripts/release-alpha-gate.sh`
2. `scripts/verify-release-promotion-inputs.sh`
3. `scripts/generate-release-publish-manifest.sh`
4. `scripts/verify-release-publish-manifest.sh`
5. artifact upload

## External publish handoff contract

For downstream publish systems, handoff is complete only when all of the following are present in `build/release-alpha-gate/`:
- `publish-manifest.json`
- `checksums.txt`
- `summary.txt`
- policy profile copy (`*.sec4.policy`)
- runtime ABI files (`sec4_runtime.h`, `sec4_runtime.c`)
- per-sample artifacts referenced by manifest (`*-build_metadata.json`, `*-sbom.json`, `*-audit.json`, `*-security_map.json`)

Downstream consumers should bind strictly to manifest fields:
- `identity.policyProfileSha256`
- `identity.policyIdentityHash`
- `identity.compilerIdentityHash`
- `identity.runtimeIdentityHash`
- `artifacts.samples[].{name,buildMetadata,sbom,audit,securityMap}`

Recommended handoff gate:
1. run `scripts/verify-release-publish-manifest.sh`,
2. archive/upload the entire `build/release-alpha-gate/` directory as immutable release evidence,
3. use manifest fields (not ad-hoc filename discovery) in external publish automation.

This CI path is the canonical non-local promotion gate.

## Promotion pass criteria

Promotion candidate is valid only when:
- release gate exits successfully,
- promotion-input verifier exits successfully,
- naming lock check exits successfully,
- artifacts are retained for traceability.

## Failure handling

- `release-alpha-gate` failure:
  - fix build/policy/audit determinism issues before retry.
- verifier failure:
  - treat as artifact integrity/consistency issue,
  - regenerate artifacts from clean run and investigate drift cause.
- naming-lock failure:
  - resolve naming contract drift before promotion.

## Operator notes

- Do not bypass verifier failures by manually editing artifacts.
- Re-run gate + verifier from clean workspace when investigating drift.
- Keep promotion evidence attached to release records for auditability.
