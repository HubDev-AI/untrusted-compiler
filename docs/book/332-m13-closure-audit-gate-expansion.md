# M13 Slice: Closure Audit Gate Expansion

This slice strengthens milestone closure verification so `PASS` means both evidence presence and evidence contract integrity.

## What it is

Updated:
- `scripts/check-milestone-closure.sh`
- `scripts/test-check-milestone-closure.sh`
- `docs/05-sec4-master-roadmap.md`
- `docs/book/328-milestone-closure-audit-checklist.md`

## Why it exists

The previous closure checker could still pass in two risky cases:
1. M10 coverage could pass from unioned impl IDs even if one endpoint was missing an impl.
2. M13 trend evidence could pass without validating the trend workflow still enforced strict quality and regression guards.

This slice closes those gaps with deterministic checks.

## What changed

1. M10 coverage check tightened (`M10-A`)
- now requires `sec4/go/node/rust` coverage for each endpoint row.

2. New compare-matrix contract gate (`M10-B`)
- validates endpoint/leader alignment,
- validates compared-row endpoint consistency,
- validates leader row membership inside compared rows,
- validates presence/type shape for core leader quality metadata.

3. New cross-impl workflow contract gate (`M10-C`)
- requires benchmark cross-impl workflow file,
- requires scoped run contract (`--impls sec4,node,go,rust`, `--endpoints ping,decode`),
- requires strict quality check invocation with `--fail-on-warning`,
- requires cross-impl artifact upload contract.

4. New trend-workflow guard gate (`M13-B`)
- requires scheduled trend workflow file,
- requires strict quality check invocation with `--fail-on-warning`,
- requires regression-threshold check step.

5. New trend-artifact upload gate (`M13-C`)
- requires `actions/upload-artifact@v4` in trend workflow,
- requires `benchmark-trend-*` artifact naming,
- requires artifact path coverage for `benchmark-suite/results`.

## Tests

`test-check-milestone-closure.sh` now covers:
- pass path with all gates satisfied,
- fail when trend entry is missing,
- fail when endpoint-level impl coverage is incomplete,
- fail when benchmark trend workflow drops strict guardrails.

## Usage

```bash
scripts/check-milestone-closure.sh
scripts/check-milestone-closure.sh --fail-on-pending
scripts/test-check-milestone-closure.sh
```

## Tradeoffs

- Closure checks are stricter but remain static and deterministic.
- This is not a replacement for full benchmark analysis; it verifies closure evidence contracts and required guardrails.
