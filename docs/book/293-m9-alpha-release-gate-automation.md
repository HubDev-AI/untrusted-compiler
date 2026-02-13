# 293 M9 Slice: Alpha Release Gate Automation

This chapter documents the executable release gate for v0.1-alpha.

## What it is

Added:
- `scripts/release-alpha-gate.sh`

Updated:
- `docs/book/210-release-notes-and-compatibility.md`
- `docs/05-sec4-master-roadmap.md`

Key behavior:
- Runs the release checklist as one command.
- Validates deterministic build artifacts (`build_metadata.json`, `sbom.json`) across repeated builds.
- Applies secure policy profile during sample audit checks.
- Captures release artifacts under `build/release-alpha-gate/`.

## Why it exists

M9 requires a repeatable release hardening gate, not a manual checklist. This script turns checklist items into enforceable pass/fail automation.

## How it works internally

1. Builds `sec4` CLI binary.
2. Optionally runs full workspace tests (`cargo test -q`).
3. Creates isolated temp workspaces for sample projects (`examples/hello`, `examples/hello-api`).
4. Injects selected policy profile as `sec4.policy`.
5. Runs:
   - `sec4 build --locked`
   - `sec4 build --sbom` twice and compares sha256 hashes.
6. Runs `sec4 audit --format json --fail-on risk>=HIGH` and writes audit reports.
7. Copies release artifacts (`build_metadata`, `sbom`, `security_map`, audit report) to output dir.
8. Writes summary and checksum files.

## CLI usage

Default:

```bash
scripts/release-alpha-gate.sh
```

Options:

```bash
scripts/release-alpha-gate.sh --profile policies/default-secure-prod.sec4.policy
scripts/release-alpha-gate.sh --out-dir build/release-alpha-gate
scripts/release-alpha-gate.sh --skip-tests
```

## Artifacts produced

By default under `build/release-alpha-gate/`:
- `<sample>-build_metadata.json`
- `<sample>-sbom.json`
- `<sample>-security_map.json`
- `<sample>-audit.json`
- `checksums.txt`
- `summary.txt`

## Validation status

Validated in-repo by running both:
- `scripts/release-alpha-gate.sh --skip-tests`
- `scripts/release-alpha-gate.sh`

## Tradeoffs and next steps

- Tradeoff:
  - current gate runs against canonical sample projects; production targets should add service-specific profiles and sample sets.
- Next:
  - wire this script as the default CI release gate for alpha tags.
