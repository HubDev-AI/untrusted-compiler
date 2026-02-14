# 480 M18 Release Publish-Integrity Contract Expansion

This chapter documents M18-S5: release-track execution slice for publish-manifest artifact integrity hardening.

## 1) What changed

- `scripts/verify-release-publish-manifest.sh` now validates artifact file hashes (policy profile copy, runtime header/source copies, per-sample build metadata, per-sample sbom) against `checksums.txt`.
- `scripts/test-verify-release-publish-manifest.sh` now includes a tampered-artifact regression case (mutated `hello-sbom.json`) that must fail verifier checks.
- Added a naming-lock fast contract checker:
  - `scripts/test-m18-release-publish-integrity.sh`

## 2) Why it matters

Before this slice, publish-manifest verification primarily checked manifest/checksum/summary consistency. This slice adds direct file-hash verification so artifact tampering after manifest generation is caught deterministically.

## 3) How it works

- Verifier now uses `hash_file` + `assert_hash_matches` helpers.
- It validates:
  - policy copy hash == `policy_profile_sha256`,
  - runtime header copy hash == `runtime_header_sha256`,
  - runtime source copy hash == `runtime_source_sha256`,
  - sample build metadata hash == `<sample> build_metadata_sha256`,
  - sample sbom hash == `<sample> sbom_sha256`.
- Any mismatch exits with a stable diagnostic (`artifact checksum mismatch for ...`).

## 4) Verification

- Behavioral regression:
  - `scripts/test-verify-release-publish-manifest.sh`
- Fast contract lock (naming-lock CI):
  - `scripts/test-m18-release-publish-integrity.sh`

## 5) Tradeoff

Hash verification adds minor runtime cost to publish verification, but strengthens release artifact integrity guarantees with deterministic failure modes.
