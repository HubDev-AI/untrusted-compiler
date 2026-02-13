# 301 M10 Slice: Benchmark Artifact Contract Spec

This chapter documents centralizing benchmark artifact naming/schema into one canonical spec.

## What it is

Added:
- `benchmark-suite/spec/artifact-contract-v0.1.md`

Updated:
- `benchmark-suite/README.md`
- `scripts/check-naming-lock.sh`
- `docs/05-sec4-master-roadmap.md`

## Why it exists

Benchmark scripts and test fixtures already implied artifact contracts, but those contracts were distributed across scripts/tests. A single explicit spec reduces ambiguity and keeps naming/schema drift reviewable.

## How it works internally

1. `benchmark-suite/spec/artifact-contract-v0.1.md` now declares:
   - canonical artifact filename patterns,
   - canonical implementation IDs,
   - required top-level keys for major JSON artifacts,
   - synchronized-change policy (spec + tests + naming-lock updates together).
2. `scripts/check-naming-lock.sh` now enforces contract-spec presence and required tokens.
3. `benchmark-suite/README.md` points to the canonical contract file.

## Inputs, outputs, and constraints

- Input:
  - benchmark scripts/testdata and artifact naming conventions.
- Output:
  - explicit, centralized benchmark artifact contract for reviewers and CI checks.
- Constraint:
  - current contract is markdown-token validated, not full JSON-schema enforcement.

## Failure modes and diagnostics

- Missing contract file:
  - naming-lock fails with missing-spec error.
- Contract token drift:
  - naming-lock fails with missing-token error.

## Example usage

```bash
cat benchmark-suite/spec/artifact-contract-v0.1.md
scripts/check-naming-lock.sh
```

## Tradeoffs and next steps

- Tradeoff:
  - markdown token checks are intentionally lightweight.
- Next:
  - add machine-validated JSON schema assets for report/matrix artifacts and wire them into benchmark script tests.
