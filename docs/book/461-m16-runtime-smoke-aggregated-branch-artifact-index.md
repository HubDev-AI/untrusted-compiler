# 461 M16 Slice: Runtime-Smoke Aggregated Branch Artifact Index

This chapter documents adding a machine-readable aggregate index for runtime-smoke branch artifacts.

## 1) What it is

`scripts/build-runtime-smoke-branch-index.sh` builds:

- `build/runtime-smoke/runtime-smoke-branch-index.json`

from the two branch artifact directories:

- `build/runtime-smoke/default`
- `build/runtime-smoke/max-body`

## 2) Why it exists

Runtime-smoke CI now executes two branches. Without an aggregate index, evidence consumers need to manually open both directories to understand profile differences and key metadata.

## 3) How it works internally

1. Validate branch directories exist.
2. Validate required branch files (`run-metadata.txt`, `users.headers`, `users.body`).
3. Parse metadata and enforce branch invariants:
   - `default` requires `maxBodyBytes=unset`
   - `max-body` requires numeric `maxBodyBytes`
   - `runFlags` shape must match branch profile
4. Validate users `traceId` header/body correlation per branch.
5. Emit deterministic JSON index with `branchOrder` and per-branch summaries.

## 4) Inputs, outputs, constraints

Inputs:

- `build/runtime-smoke/default/*`
- `build/runtime-smoke/max-body/*`

Outputs:

- `runtime-smoke-branch-index.json`

Constraints:

- branch names are fixed (`default`, `max-body`)
- profile invariants are enforced before index emission

## 5) Failure modes and diagnostics

- missing branch directory:
  - `missing runtime-smoke branch artifacts directory: <branch>`
- wrong default profile:
  - `runtime-smoke index expects default branch maxBodyBytes=unset`
- wrong max-body profile:
  - `runtime-smoke index expects max-body branch maxBodyBytes to be numeric`
- runFlags/profile mismatch:
  - `runtime-smoke <branch> runFlags shape does not match expected branch profile`

## 6) Example usage

```bash
scripts/build-runtime-smoke-branch-index.sh \
  --artifacts-root build/runtime-smoke \
  --out build/runtime-smoke/runtime-smoke-branch-index.json
```

## 7) Trade-offs and next steps

Trade-offs:

- index generation adds one more contract step in CI.

Next steps:

- include lightweight request/response envelope digests per branch for faster artifact diffing.

## Verification

- `scripts/test-build-runtime-smoke-branch-index.sh`
- `scripts/test-runtime-smoke-workflow-contract.sh`
- `scripts/test-runtime-smoke-workflow-contract-guard.sh`
- `scripts/test-check-milestone-closure.sh`
