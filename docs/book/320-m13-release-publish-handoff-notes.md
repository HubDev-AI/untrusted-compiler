# 320 M13 Slice: Release Publish Handoff Notes

This chapter documents the external publish handoff contract for release artifacts produced by `sec4` promotion workflows.

## What it is

Updated:
- `docs/book/313-m13-release-promotion-playbook.md`
- `docs/05-sec4-master-roadmap.md`

This slice defines the minimum artifact and field-level contract external publish tooling must consume.

## Why it exists

Release artifact generation and verification are already automated, but downstream publish systems need an explicit, deterministic contract instead of ad-hoc file discovery.

## Handoff contract (required)

External publish tooling must consume artifacts from `build/release-alpha-gate/` and require:
- `publish-manifest.json`,
- `checksums.txt`,
- `summary.txt`,
- copied policy profile (`*.sec4.policy`),
- runtime ABI files (`sec4_runtime.h`, `sec4_runtime.c`),
- per-sample manifest-referenced files:
  - `*-build_metadata.json`,
  - `*-sbom.json`,
  - `*-audit.json`,
  - `*-security_map.json`.

Mandatory manifest identity fields:
- `identity.policyProfileSha256`
- `identity.policyIdentityHash`
- `identity.compilerIdentityHash`
- `identity.runtimeIdentityHash`

Mandatory sample bindings:
- `artifacts.samples[].name`
- `artifacts.samples[].buildMetadata`
- `artifacts.samples[].sbom`
- `artifacts.samples[].audit`
- `artifacts.samples[].securityMap`

## Operator checklist

1. Run release gate and promotion verifiers.
2. Generate publish manifest.
3. Verify publish manifest consumption contract.
4. Hand off/upload the full `build/release-alpha-gate/` directory.
5. Configure downstream publish automation to read manifest fields directly.

## Inputs, outputs, and constraints

- Inputs:
  - verified release-gate artifact directory.
- Outputs:
  - deterministic handoff package for external publish automation.
- Constraints:
  - downstream systems must reject partial packages and manifest/checksum identity drift.

## Tradeoffs and next steps

- Tradeoff:
  - contract is intentionally strict and may require adapter work in existing external tooling.
- Next:
  - add downstream adapter examples that map `publish-manifest.json` into concrete release publishing actions.
