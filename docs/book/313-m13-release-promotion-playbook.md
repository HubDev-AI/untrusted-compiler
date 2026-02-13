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

4. Record release evidence

Capture artifacts from `build/release-alpha-gate/` and attach/upload as promotion evidence.

## CI-equivalent flow

`alpha-release-gate.yml` now executes:
1. `scripts/release-alpha-gate.sh`
2. `scripts/verify-release-promotion-inputs.sh`
3. artifact upload

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
