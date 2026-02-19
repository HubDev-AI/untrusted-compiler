# 1008 M39 Slice: LASM Default `run` Backend and Explicit C Fallback

## What It Is

This slice switches `sec4 run` backend default from C to LASM while preserving explicit C fallback:

- CLI default is now `--backend lasm` (implicit when flag is omitted),
- C runtime remains available through explicit `--backend c`,
- command tests lock both the default-help surface and C-only guard behavior under explicit C selection.

## Why It Exists

Post-DB parity execution order requires LASM to become the primary server runtime path before load hardening and adapter progression.

Keeping C as explicit fallback preserves compatibility for paths still tied to native runtime assumptions, but default operator flow now targets LASM first.

## How It Works Internally

1. CLI argument default:
   - `Run.backend` default in `compiler/sec4-cli/src/main.rs` changed from `RunBackend::C` to `RunBackend::Lasm`.

2. Backend dispatch:
   - existing runtime split is preserved:
     - LASM branch when backend is `Lasm`,
     - C build/run branch when backend is `C`.

3. Tests:
   - C-only guard tests now pass `--backend c` explicitly,
   - added help-surface test asserting `run --help` reports `[default: lasm]`,
   - added oneshot integration coverage showing LASM-only flags are accepted when backend is omitted (implicit LASM default),
   - promotion run test that depends on C behavior now opts into explicit `--backend c`.

## Inputs / Outputs and Constraints

Inputs:
- any `sec4 run` invocation that omits `--backend`.

Outputs:
- command now executes LASM runtime path by default.

Constraints:
- LASM-only flags remain LASM-only (unchanged diagnostics),
- LASM-only flags are valid on default runs because omitted backend now resolves to LASM,
- C fallback remains opt-in via explicit `--backend c`.

## Failure Modes and Diagnostics

No new error codes were introduced.

Existing guard diagnostics remain unchanged for explicit C runs:

- `run failed: --max-runtime-steps is only supported with --backend lasm`
- `run failed: --max-keep-alive-requests is only supported with --backend lasm`
- `run failed: --max-pending is only supported with --backend lasm`
- `run failed: --db-base is only supported with --backend lasm`
- `run failed: --overflow-probe-timeout-ms is only supported with --backend lasm`

## Example Usage

Default LASM:

```bash
sec4 run --path .
```

Explicit C fallback:

```bash
sec4 run --path . --backend c
```

## Tradeoffs and Next Steps

Tradeoffs:
- operators now get LASM-first behavior by default, but workflows that implicitly depended on C must pass `--backend c` explicitly.

Next steps:
1. continue fixed-order plan with LASM stability/load hardening,
2. keep C fallback as explicit compatibility lane while LASM throughput/latency/memory baselines are hardened.
