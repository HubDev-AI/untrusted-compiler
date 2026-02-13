# 206 M9 Slice: Locked Build and Deterministic Lockfile Validation

This chapter documents a Release Hardening (`M9`) slice that adds explicit lockfile validation controls for deterministic builds.

## What it is

Added `ailang build --locked` with deterministic lockfile validation:
- `build` (default) writes deterministic `ailang.lock`,
- `build --locked` refuses to proceed unless `ailang.lock` exists and matches current manifest inputs.

Lockfile contents now include a deterministic manifest fingerprint derived from package/build fields.

## Why it exists

`M9` requires deterministic build controls and lock strategy. Before this slice, `build` always rewrote a lockfile stub and there was no CI-safe mode to assert lock freshness.

`--locked` gives release/CI workflows a strict guardrail: no implicit lock refresh during verification steps.

## How it works internally

1. Core manifest layer now renders deterministic lock content:
   - package name/version/edition,
   - build entry path,
   - `manifest_fingerprint` (`man_<fnv64>` over normalized manifest/build fields).
2. Added `validate_lockfile_stub(project_root, manifest)`:
   - missing lockfile -> `M0202`,
   - stale/mismatched lockfile -> `M0203`.
3. CLI `build` command gained `--locked`:
   - default mode writes lockfile (`write_lockfile_stub`),
   - locked mode validates lockfile only (`validate_lockfile_stub`),
   - success text reports `verified lockfile: ...` in locked mode.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/manifest.rs`
  - `compiler/ailang-core/src/lib.rs`
  - `compiler/ailang-cli/src/main.rs`
- Outputs:
  - deterministic lockfile body with manifest fingerprint,
  - CLI locked-mode control path with explicit diagnostics.
- Constraints:
  - `--locked` does not auto-heal stale lockfiles by design.
  - lock validation compares normalized text (CRLF/LF tolerant).

## Failure modes and diagnostics

- `M0202`: lockfile missing/unreadable in locked mode.
  - note includes regeneration guidance (`ailang build`).
- `M0203`: lockfile stale vs current manifest/build entry.
  - note points to non-locked rebuild for refresh.

## Example usage

Generate/refresh lockfile:

```bash
ailang build --path examples/hello
```

Verify lockfile in CI:

```bash
ailang build --path examples/hello --locked
```

If `ailang.toml` changes without lock refresh, locked mode fails with `M0203`.

## Tradeoffs and next steps

- Tradeoff:
  - deterministic lock strategy is currently manifest-entry scoped; dependency graph locking remains intentionally minimal in v0.1.
- Next:
  - extend lock schema for dependency pins once package resolution lands,
  - optionally add `check --locked` parity for lock freshness checks outside build flows.
