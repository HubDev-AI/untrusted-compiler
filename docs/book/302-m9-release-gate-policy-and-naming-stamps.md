# 302 M9 Slice: Release Gate Policy and Naming Stamps

This chapter documents extending alpha release-gate artifact identity stamping.

## What it is

Updated:
- `scripts/release-alpha-gate.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/210-release-notes-and-compatibility.md`

## Why it exists

Release artifacts should capture not only build outputs but also the exact policy identity and naming-lock posture used to produce them. This improves traceability and auditability of alpha candidates.

## How it works internally

`scripts/release-alpha-gate.sh` now:

1. Computes SHA256 for the active policy profile.
2. Copies the policy profile file into release artifacts directory.
3. Writes `policy_profile_sha256` into `checksums.txt`.
4. Writes both policy hash and naming-lock status into `summary.txt`.

## Inputs, outputs, and constraints

- Input:
  - selected policy profile (`--profile`, default `policies/default-secure-prod.sec4.policy`).
- Outputs:
  - `build/release-alpha-gate/<policy-file>`
  - `build/release-alpha-gate/checksums.txt` (includes policy hash)
  - `build/release-alpha-gate/summary.txt` (includes policy hash + naming-lock stamp)
- Constraint:
  - relies on `shasum` or `sha256sum` availability (already handled by portable hash fallback).

## Failure modes and diagnostics

- Missing policy file:
  - gate fails before build checks.
- Missing hash utility:
  - gate fails with explicit hash-tool error.

## Example usage

```bash
scripts/release-alpha-gate.sh --skip-tests
cat build/release-alpha-gate/summary.txt
cat build/release-alpha-gate/checksums.txt
```

## Tradeoffs and next steps

- Tradeoff:
  - stamp scope currently covers policy and naming-lock identity only.
- Next:
  - extend stamps with compiler/runtime executable identity in the same summary/checksum artifacts.
