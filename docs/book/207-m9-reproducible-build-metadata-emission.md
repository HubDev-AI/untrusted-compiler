# 207 M9 Slice: Reproducible Build Metadata Emission

This chapter documents a Release Hardening (`M9`) slice that adds deterministic build metadata output for release/audit workflows.

## What it is

`ailang build` now writes `build/build_metadata.json` on successful builds.

The metadata contains deterministic build identity fields:
- package identity (`name`, `version`, `edition`, `entry`),
- `policyHash`,
- `compilerHash`,
- `runtimeHash`,
- `lockFingerprint`.

## Why it exists

`M9` requires reproducible build metadata for release hardening. Before this slice, security audit output had hash fields, but build artifacts themselves did not include a stable metadata file.

By emitting deterministic metadata from build inputs, CI/release pipelines can compare artifacts without relying on log parsing.

## How it works internally

1. Added core module `build_metadata`:
   - computes `compilerHash` from package version,
   - computes `runtimeHash` from runtime C header/source payloads,
   - computes `lockFingerprint` from normalized lockfile content.
2. Added core writer:
   - `write_build_metadata(project_root, manifest, policy)` -> `build/build_metadata.json`.
3. Wired CLI `build` flow:
   - after lockfile write/validation,
   - loads policy and writes metadata file,
   - reports metadata path in text output mode.
4. Updated audit build summary hash source:
   - `sec.audit` now uses shared compiler/runtime hash helpers for consistency.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/build_metadata.rs`
  - `compiler/ailang-core/src/audit.rs`
  - `compiler/ailang-cli/src/main.rs`
- Outputs:
  - deterministic `build/build_metadata.json` file.
- Constraints:
  - metadata intentionally excludes wall-clock timestamps to keep identical-input builds byte-stable.
  - lockfile must exist (build flow guarantees this via write/locked validation).

## Failure modes and diagnostics

- `M0301`: lockfile read failure while assembling metadata.
- `M0302`: build directory creation failure for metadata output.
- `M0303`: metadata write failure.

All emit standard span+note diagnostics and fail the build command.

## Example usage

```bash
ailang build --path examples/hello
cat examples/hello/build/build_metadata.json
```

The file is stable across repeated builds when manifest/policy/runtime/lockfile inputs are unchanged.

## Tradeoffs and next steps

- Tradeoff:
  - current metadata schema is intentionally small and release-focused.
- Next:
  - add optional SBOM emission alongside metadata (`M9` optional item),
  - add release profile automation that captures `sec.audit` JSON + `build_metadata.json` together.
