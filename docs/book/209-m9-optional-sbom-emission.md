# 209 M9 Slice: Optional SBOM Emission

This chapter documents the optional `M9` release-hardening item: SBOM generation.

## What it is

Added optional SBOM output in build flow:

```bash
sec4 build --sbom
```

When enabled, build writes `build/sbom.json` alongside lockfile and build metadata.

## Why it exists

`M9` calls out optional SBOM generation. A built-in path removes ad-hoc scripting from release pipelines and keeps artifact provenance tied to deterministic Untrusted<T> metadata.

## How it works internally

1. Added core SBOM module (`sbom.rs`) with deterministic writer:
   - `write_sbom(project_root, build_metadata)`.
2. SBOM document includes:
   - CycloneDX envelope (`bomFormat`, `specVersion`),
   - application component (package name/version),
   - provenance properties from build metadata:
     - policy hash,
     - compiler hash,
     - runtime hash,
     - lock fingerprint.
3. CLI `build` now accepts `--sbom`:
   - always writes lockfile + build metadata first,
   - then emits `build/sbom.json`,
   - prints `wrote sbom: ...` in text mode.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/sec4-core/src/sbom.rs`
  - `compiler/sec4-cli/src/main.rs`
- Outputs:
  - deterministic `build/sbom.json` artifact when `--sbom` is set.
- Constraints:
  - SBOM shape is intentionally minimal in v0.1.
  - generation depends on already-written deterministic build metadata.

## Failure modes and diagnostics

- `M0311`: build directory creation failure for SBOM.
- `M0312`: SBOM write failure.

CLI also fails if it cannot parse generated build metadata before SBOM generation.

## Example usage

```bash
sec4 build --path examples/hello --sbom
cat examples/hello/build/sbom.json
```

## Tradeoffs and next steps

- Tradeoff:
  - minimal SBOM schema keeps implementation small but does not yet include dependency graph entries.
- Next:
  - extend SBOM components when dependency resolution/lock graph matures,
  - add release checklist automation to require `--sbom` for tagged builds.
