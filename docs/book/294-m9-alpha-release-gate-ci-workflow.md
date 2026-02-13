# 294 M9 Slice: Alpha Release Gate CI Workflow

This chapter documents CI wiring for the alpha release gate.

## What it is

Added:
- `.github/workflows/alpha-release-gate.yml`

Updated:
- `scripts/release-alpha-gate.sh`
- `docs/book/210-release-notes-and-compatibility.md`
- `docs/05-sec4-master-roadmap.md`

Key behavior:
- Runs release gate on:
  - manual dispatch (`workflow_dispatch`),
  - alpha tag pushes (`v0.1.0-alpha*`).
- Uploads captured release-gate artifacts from `build/release-alpha-gate/`.
- Includes naming-lock validation as part of the gate script before build/audit checks.

## Why it exists

M9 release hardening needs enforcement beyond local runs. CI wiring ensures alpha tags always pass the same deterministic gate used locally.

## How it works internally

1. Checkout repository.
2. Install stable Rust toolchain.
3. Execute `scripts/release-alpha-gate.sh`.
4. Upload gate artifacts (always) for inspection/debugging.

## Script hardening included

`release-alpha-gate.sh` now supports both hash utilities for checksum checks:
- `shasum` (macOS-friendly)
- `sha256sum` (Linux-friendly)

This keeps deterministic artifact verification portable in local and CI environments.

## Tradeoffs and next steps

- Tradeoff:
  - current trigger scope is alpha tags + manual dispatch only; branch-level CI gating can be added later if needed.
- Next:
  - optionally gate release publication on successful completion of this workflow.
