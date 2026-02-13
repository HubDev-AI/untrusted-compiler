# 315 M13 Slice: Release Publish Manifest Generation

This chapter documents adding a publish-consumption manifest in release automation.

## What it is

Updated:
- `scripts/generate-release-publish-manifest.sh`
- `scripts/test-generate-release-publish-manifest.sh`
- `.github/workflows/alpha-release-gate.yml`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/210-release-notes-and-compatibility.md`
- `docs/book/313-m13-release-promotion-playbook.md`

## Why it exists

Promotion verification proved artifact consistency, but downstream release tooling also needs a structured summary payload. This slice creates a deterministic manifest that can be consumed by publish automation.

## How it works internally

`generate-release-publish-manifest.sh` reads `build/release-alpha-gate/` artifacts and emits `publish-manifest.json` containing:
- identity hashes (`policyProfileSha256`, `policyIdentityHash`, `compilerIdentityHash`, `runtimeIdentityHash`),
- canonical artifact filenames (`checksums`, `summary`, `policy`, runtime files),
- per-sample artifact references (`buildMetadata`, `sbom`, `audit`, `securityMap`),
- generation timestamp and schema version.

The script fails fast if required checksum keys or sample artifacts are missing.

## Tests

`test-generate-release-publish-manifest.sh`:
1. runs release gate + promotion verifier on temp artifacts,
2. generates publish manifest and validates required fields,
3. removes required checksum entry and verifies generator fails.

## CI integration

`alpha-release-gate.yml` now runs:
1. release gate,
2. promotion verifier,
3. publish manifest generation,
4. artifact upload.

## Inputs, outputs, and constraints

- Inputs:
  - `build/release-alpha-gate/` artifact set.
- Output:
  - `build/release-alpha-gate/publish-manifest.json`.
- Constraints:
  - requires `jq` and deterministic checksum keys from release gate.

## Example usage

```bash
scripts/release-alpha-gate.sh --skip-tests
scripts/verify-release-promotion-inputs.sh
scripts/generate-release-publish-manifest.sh
cat build/release-alpha-gate/publish-manifest.json
```

## Tradeoffs and next steps

- Tradeoff:
  - manifest generation is local-to-repo schema and not yet consumed by external publish tooling.
- Next:
  - wire manifest as explicit input contract in downstream release publication steps.
