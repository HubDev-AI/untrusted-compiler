# 213 M9 Slice: Release Audit Baselines

This chapter documents the `M9` baseline artifacts for `sec4 audit` under canonical policy profiles.

## What it is

Added committed baseline audit reports:
- `baselines/sec-audit/default-secure-prod.hello.json`
- `baselines/sec-audit/permissive-dev.hello.json`

Both baselines are generated against `examples/hello` with the corresponding policy profile.

## Why it exists

`M9` requires finalized policy profiles and release audit baselines. Baselines make risk posture drift explicit and give release/CI workflows a concrete comparison anchor.

## How it works internally

1. Apply profile policy file to the sample project.
2. Run:
   - `sec4 audit --path examples/hello --format json`
3. Store resulting JSON report under `baselines/sec-audit/`.

The baseline contains:
- policy/build fingerprints,
- posture snapshot,
- finding inventory,
- summary risk score and highest severity.

## Inputs, outputs, and constraints

- Inputs:
  - `policies/default-secure-prod.sec4.policy`
  - `policies/permissive-dev.sec4.policy`
  - `examples/hello`
- Outputs:
  - baseline audit JSON snapshots in `baselines/sec-audit/`.
- Constraints:
  - baseline reflects sample-project posture, not full production service posture.
  - `build.timeMs` is snapshot-time data by design in `AuditReport`.

## Failure modes and diagnostics

- If policy parser behavior changes, baseline regeneration may fail or shift findings.
- If audit rule mapping changes, expected finding IDs/severities can drift and require explicit baseline review.

## Example usage

Compare current report to baseline:

```bash
sec4 audit --path examples/hello --format json --baseline baselines/sec-audit/default-secure-prod.hello.json
```

Use threshold gating:

```bash
sec4 audit --path examples/hello --format json --fail-on risk>=HIGH
```

## Tradeoffs and next steps

- Tradeoff:
  - baselines are currently maintained as committed snapshots for simple reproducibility.
- Next:
  - add automated baseline refresh/check scripts in release workflow,
  - extend baseline set to include `hello-api` and representative service shapes before M10 reporting.
