# 250 M10 Slice: Manifest Hash Verification in Bundle Gate

This chapter documents integrity verification enhancements for benchmark bundle validation.

## What it is

Updated:
- `benchmark-suite/scripts/verify_benchmark_bundle.sh`
- `benchmark-suite/scripts/test_verify_benchmark_bundle.sh`
- `benchmark-suite/README.md`

Key changes:
- bundle verifier now checks artifact hashes against `artifact-manifest.json` by default,
- verifies manifest entry presence and sha256 match for required artifacts,
- added optional bypass flag:
  - `--skip-hash-check`.

## Why it exists

File-existence checks alone do not detect post-run tampering or stale mismatched artifacts. Hash validation makes the verification gate integrity-aware.

## How it works internally

1. Parse optional `--skip-hash-check`.
2. Validate required files and JSON structure.
3. For each required artifact:
   - look up manifest hash entry by relative path,
   - compute current sha256 (`shasum -a 256`),
   - fail if missing entry or mismatch.
4. Continue structural checks on matrix/report summaries.

## Inputs, outputs, and constraints

- Inputs:
  - `verify_benchmark_bundle.sh [--skip-hash-check] <results_dir> [impls_csv] [endpoints_csv]`.
- Outputs:
  - success message for verified bundle,
  - non-zero with explicit mismatch error otherwise.
- Constraints:
  - hash verification requires `shasum`.

## Failure modes and diagnostics

- missing manifest path entry -> `manifest missing artifact hash entry: <path>`.
- content drift/tamper -> `artifact hash mismatch for <path>`.
- `shasum` unavailable -> explicit dependency error (unless skip flag is used).

## Example usage

```bash
benchmark-suite/scripts/verify_benchmark_bundle.sh benchmark-suite/results sec4,node,go,rust ping,decode,users-post,users-get
```

Skip integrity check:

```bash
benchmark-suite/scripts/verify_benchmark_bundle.sh --skip-hash-check benchmark-suite/results node ping
```

## Tradeoffs and next steps

- Tradeoff:
  - hash verification adds runtime cost proportional to artifact size/count.
- Next:
  - optionally cache manifest-validation outcomes for unchanged artifacts in CI.
