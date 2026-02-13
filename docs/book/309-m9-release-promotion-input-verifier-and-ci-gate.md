# 309 M9 Slice: Release Promotion Input Verifier and CI Gate

This chapter documents adding release-promotion input verification on top of alpha gate artifacts.

## What it is

Updated:
- `scripts/verify-release-promotion-inputs.sh`
- `scripts/test-verify-release-promotion-inputs.sh`
- `.github/workflows/alpha-release-gate.yml`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/210-release-notes-and-compatibility.md`

## Why it exists

`release-alpha-gate` now emits rich identity/checksum artifacts, but promotion workflows need an explicit verifier to consume and enforce those stamps before publication.

## How it works internally

`verify-release-promotion-inputs.sh` validates `build/release-alpha-gate/` artifacts:

1. Required files exist (`checksums.txt`, `summary.txt`, copied policy/runtime artifacts).
2. File-level SHA256 checks match stamped checksum entries.
3. Summary identity stamps match checksum identity entries.
4. For each sample (`*-build_metadata.json`):
   - metadata/sbom checksums match `checksums.txt`,
   - metadata identity hashes match expected `policy/compiler/runtime` identities,
   - matching audit report build/policy hashes match expected identities.

If any mismatch is found, verification fails.

## Test coverage

`test-verify-release-promotion-inputs.sh`:
- generates fresh artifacts via `release-alpha-gate.sh --skip-tests --out-dir <tmp>`,
- verifies pass path,
- tampers one audit identity field and verifies failure path.

## CI integration

`alpha-release-gate.yml` now runs:
1. `scripts/release-alpha-gate.sh`
2. `scripts/verify-release-promotion-inputs.sh`
3. artifact upload

This enforces promotion consistency before artifact publication in the release workflow.

## Inputs, outputs, and constraints

- Input:
  - release gate artifact directory (`build/release-alpha-gate/` by default).
- Output:
  - pass/fail verification signal suitable for promotion gating.
- Constraint:
  - requires `jq` and sha256 tool availability (`shasum` or `sha256sum`).

## Example usage

```bash
scripts/release-alpha-gate.sh --skip-tests
scripts/verify-release-promotion-inputs.sh
scripts/test-verify-release-promotion-inputs.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - verification enforces internal artifact consistency but does not yet stamp external release package manifests.
- Next:
  - bind this verifier output into final publication/promote steps and release playbook checklist.
