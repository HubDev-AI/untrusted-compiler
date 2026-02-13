# 208 M9 Slice: Policy Profiles Artifacts and Baseline Workflow

This chapter documents a Release Hardening (`M9`) slice that promotes policy profiles from spec text into versioned repository artifacts.

## What it is

Added canonical policy profile files:
- `policies/default-secure-prod.sec4.policy`
- `policies/permissive-dev.sec4.policy`

Added parser tests that validate both files as first-class policy inputs.

## Why it exists

`M9` requires finalized secure policy profiles and release audit baselines. Keeping profiles only in prose is error-prone; real files are needed for reproducible audits and release checks.

By committing profile artifacts, CI and release scripts can run `sec4 audit` against the exact same profile definitions.

## How it works internally

1. Added two concrete policy files under `policies/`.
2. Added regression tests in `compiler/sec4-core/tests/policy.rs` using `include_str!`:
   - parse profile contents with `parse_policy_str(...)`,
   - assert core posture toggles (mode/env/cors/internal-net/fs/replay).
3. Profiles use only currently-supported policy keys in the parser, so they are executable inputs, not aspirational examples.

## Inputs, outputs, and constraints

- Inputs:
  - `policies/default-secure-prod.sec4.policy`
  - `policies/permissive-dev.sec4.policy`
  - `compiler/sec4-core/tests/policy.rs`
- Outputs:
  - stable, versioned profile artifacts for audit and release workflows.
- Constraints:
  - profile files are scoped to currently implemented parser keys.
  - broader future keys (for example nested auth cookie/token sub-sections) remain out of this slice.

## Failure modes and diagnostics

- If profile files drift into unsupported/unknown keys, policy tests fail with parser diagnostics (`P6002`/`P6003` paths).
- If defaults/enforcement contracts drift unintentionally, assertion failures highlight profile deltas in CI.

## Example usage

Use a profile in a project root:

```bash
cp policies/default-secure-prod.sec4.policy examples/hello/sec4.policy
sec4 audit --path examples/hello --format json
```

Swap to permissive dev profile for local posture comparisons:

```bash
cp policies/permissive-dev.sec4.policy examples/hello/sec4.policy
sec4 audit --path examples/hello --format text
```

## Tradeoffs and next steps

- Tradeoff:
  - profile artifacts are explicit and testable, but profile selection is still file-copy based (no dedicated CLI profile selector yet).
- Next:
  - add release baseline snapshots produced from these profiles,
  - add optional profile-selection convenience in CLI once release flow is finalized.
