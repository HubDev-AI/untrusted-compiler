# 304 M9 Slice: Release Gate Identity-Hash Consistency Checks

This chapter documents a release-hardening update that verifies semantic identity hashes across release artifacts.

## What it is

Updated:
- `scripts/release-alpha-gate.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/210-release-notes-and-compatibility.md`

## Why it exists

The release gate already stamped file-level SHA256 hashes for policy/runtime/compiler assets, but downstream publishing also depends on semantic identity hashes in `build_metadata.json` and `sec4 audit` output. This slice prevents drift between these identity channels.

## How it works internally

`release-alpha-gate.sh` now adds identity verification for each sample:

1. Parses `build_metadata.json` fields:
   - `policyHash`
   - `compilerHash`
   - `runtimeHash`
2. Parses matching `sec4 audit` JSON fields:
   - `.policy.hash`
   - `.build.compilerHash`
   - `.build.runtimeHash`
3. Fails if any metadata/audit value mismatches for a sample.
4. Fails if identity hashes differ across gated samples.
5. Writes verified identity hashes to:
   - `build/release-alpha-gate/checksums.txt`
   - `build/release-alpha-gate/summary.txt`

## Inputs, outputs, and constraints

- Inputs:
  - sample `build/build_metadata.json`
  - sample audit report (`--write-report` JSON)
- Outputs:
  - `policy_identity_hash`
  - `compiler_identity_hash`
  - `runtime_identity_hash`
- Constraint:
  - requires `jq` for JSON field extraction/validation.

## Failure modes and diagnostics

- Missing JSON fields:
  - release gate fails with explicit missing-field error.
- Per-sample metadata vs audit mismatch:
  - release gate fails with mismatch error.
- Cross-sample identity mismatch:
  - release gate fails before summary generation.

## Example usage

```bash
scripts/release-alpha-gate.sh --skip-tests
cat build/release-alpha-gate/checksums.txt
cat build/release-alpha-gate/summary.txt
```

## Tradeoffs and next steps

- Tradeoff:
  - this verifies consistency inside gate artifacts, but does not yet enforce promotion-pipeline consumption of these identity stamps.
- Next:
  - add release-publish checks that require matching identity stamps before artifact promotion.
