# 303 M9 Slice: Release Gate Compiler/Runtime Hash Stamps

This chapter documents extending release-gate identity stamping to compiler/runtime assets.

## What it is

Updated:
- `scripts/release-alpha-gate.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/210-release-notes-and-compatibility.md`

## Why it exists

Policy and naming-lock stamps improved release traceability, but alpha artifacts still lacked direct compiler/runtime binary identity in the captured checksums. Adding these hashes strengthens provenance and reproducibility checks.

## How it works internally

`scripts/release-alpha-gate.sh` now:

1. Computes SHA256 for:
   - `target/debug/sec4`
   - `runtime/c/sec4_runtime.h`
   - `runtime/c/sec4_runtime.c`
2. Copies runtime ABI files into release artifact directory.
3. Writes new hash lines into `checksums.txt`.
4. Writes the same hash stamps into `summary.txt`.

## Inputs, outputs, and constraints

- Inputs:
  - built `sec4` binary,
  - runtime ABI source/header files.
- Outputs:
  - additional hash stamps in:
    - `build/release-alpha-gate/checksums.txt`
    - `build/release-alpha-gate/summary.txt`
  - copied runtime files:
    - `build/release-alpha-gate/sec4_runtime.h`
    - `build/release-alpha-gate/sec4_runtime.c`
- Constraint:
  - hashes are for the local gate build profile (`target/debug/sec4`) in current script behavior.

## Failure modes and diagnostics

- Missing runtime ABI file:
  - script fails during hash or copy step.
- Missing hash utility:
  - script fails with explicit `shasum/sha256sum` tool message.

## Example usage

```bash
scripts/release-alpha-gate.sh --skip-tests
cat build/release-alpha-gate/checksums.txt
cat build/release-alpha-gate/summary.txt
```

## Tradeoffs and next steps

- Tradeoff:
  - current stamps do not yet enforce consistency against external release-publish metadata.
- Next:
  - verify stamped compiler/runtime hashes against release-publish metadata before final artifact promotion.
