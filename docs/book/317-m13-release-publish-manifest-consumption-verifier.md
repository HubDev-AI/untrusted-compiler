# 317 M13 Slice: Release Publish-Manifest Consumption Verifier

This chapter documents adding downstream publish-manifest consumption checks.

## What it is

Updated:
- `scripts/verify-release-publish-manifest.sh`
- `scripts/test-verify-release-publish-manifest.sh`
- `.github/workflows/alpha-release-gate.yml`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/313-m13-release-promotion-playbook.md`
- `docs/book/315-m13-release-publish-manifest-generation.md`

## Why it exists

`publish-manifest.json` was generated, but publish tooling also needs an explicit verifier to assert manifest integrity against release artifacts before handoff.

## How it works internally

`verify-release-publish-manifest.sh` validates:
1. manifest schema essentials (`version`, `tool`),
2. required artifact references exist,
3. manifest identity values match `checksums.txt`,
4. each manifest sample entry is complete and has matching checksum records.

Any mismatch fails the verifier.

## Tests

`test-verify-release-publish-manifest.sh`:
- builds temporary artifacts,
- verifies pass path,
- tampers manifest identity hash,
- verifies fail path.

## CI integration

`alpha-release-gate.yml` now runs:
1. release gate,
2. promotion verifier,
3. publish manifest generation,
4. publish manifest verification,
5. artifact upload.

## Inputs, outputs, and constraints

- Inputs:
  - `publish-manifest.json`, release artifacts dir.
- Output:
  - pass/fail signal for downstream publish handoff safety.
- Constraint:
  - relies on deterministic release checksum key conventions.

## Example usage

```bash
scripts/generate-release-publish-manifest.sh
scripts/verify-release-publish-manifest.sh
scripts/test-verify-release-publish-manifest.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - verifier enforces local manifest contract only; external publish systems still need explicit integration.
- Next:
  - integrate manifest verifier outputs into external publish pipeline adapters.
