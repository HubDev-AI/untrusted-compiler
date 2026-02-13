# M14 Slice: Replay Capture Contract Test Harness

This slice bootstraps the M14 replay track with an executable capture-file contract and CI enforcement.

## What it is

Added/updated:
- `captures/sample-capture.json`
- `scripts/check-replay-capture-contract.sh`
- `scripts/test-replay-capture-contract.sh`
- `.github/workflows/naming-lock.yml`
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

M13 explicitly deferred deeper replay IO stubbing to M14. Before runtime-level stubs, we need deterministic, machine-checked capture contract shape so replay tooling has a stable input boundary.

## What changed

1. Added replay capture contract checker
- `scripts/check-replay-capture-contract.sh` validates required capture fields:
  - identity (`captureId`, `traceId`, hashes),
  - request shell (`method`, `path`, `headers`, `body`),
  - determinism (`seed`, time/uuid providers, budget),
  - redaction report (`headers`, `jsonPaths`).
- Body-encoding contract is enforced:
  - `encoding=base64` requires `bytes`,
  - `encoding=none` requires `sha256`.

2. Added replay capture regression tests
- `scripts/test-replay-capture-contract.sh` covers:
  - valid fixture pass,
  - missing `traceId` failure,
  - `encoding=none` without `sha256` failure,
  - unsupported encoding failure.

3. Added CI + closure gate wiring
- Naming-lock now runs `scripts/test-replay-capture-contract.sh`.
- `check-milestone-closure.sh` adds gate `M14-A` enforcing replay-capture contract test presence in naming-lock.
- `test-check-milestone-closure.sh` adds negative fixture coverage for missing replay-capture contract enforcement and includes `M14-A` in JSON gate-order assertions.

## Validation

```bash
scripts/check-replay-capture-contract.sh
scripts/test-replay-capture-contract.sh
scripts/test-check-milestone-closure.sh
scripts/test-roadmap-closure-gate-alignment.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/check-naming-lock.sh
```

## Tradeoffs

- Adds one more static CI guard in naming-lock.
- Does not yet implement runtime replay stubs; this slice focuses on stable input-contract enforcement to unblock later M14 runtime work.
