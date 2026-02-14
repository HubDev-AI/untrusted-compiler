# 432 M16 Slice: Operator Smoke Script Migration to sec4 run --oneshot

This chapter documents migrating the operator smoke script from env-driven oneshot launch (`SEC4_RT_HTTP_SERVE_MODE=oneshot`) to the explicit run-command flag (`--oneshot`).

## 1) What it is

`/scripts/smoke-sec4-run-hello-api.sh` now starts runtime smoke requests with:

```bash
cargo run -p sec4 -- run --path "${work_project}" --oneshot
```

instead of setting `SEC4_RT_HTTP_SERVE_MODE=oneshot` manually.

## 2) Why it exists

The previous slice (`M16-S27`) added explicit runtime bridge flags on `sec4 run`. The smoke script should dogfood that CLI contract so operator and CI workflows use the same discoverable command surface.

## 3) How it works internally

1. In `request_once()`, the script launch command now appends `--oneshot` to `sec4 run`.
2. Timeout behavior remains env-driven through `SEC4_RT_HTTP_SERVE_TIMEOUT_MS=12000`.
3. Contract test token expectations were updated in `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`:
   - removed hard requirement for `SEC4_RT_HTTP_SERVE_MODE=oneshot`
   - added required `--oneshot` token
4. Guard test remained valid because it targets CSRF-token token removal behavior.

## 4) Inputs, outputs, constraints

Inputs:

- Smoke script options: `--project`, `--artifacts-dir`
- Runtime command path: `sec4 run --oneshot`

Outputs:

- Same smoke artifacts (`health.*`, `users.*`, logs, metadata)
- Same runtime behavior and deterministic oneshot exit model

Constraints:

- Script still depends on `cargo`, `curl`, `jq`, `python3`.
- Timeout remains controlled by `SEC4_RT_HTTP_SERVE_TIMEOUT_MS`.

## 5) Failure modes and diagnostics

- If oneshot execution exits early, smoke script still emits:
  - `sec4 run exited before <name> request completed`
- If response contract drifts, headers/body dumps are printed before exit.
- Contract drift in script content is caught by:
  - `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
  - `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`

## 6) Example usage

```bash
scripts/smoke-sec4-run-hello-api.sh --artifacts-dir build/runtime-smoke
scripts/check-runtime-smoke-artifacts.sh --artifacts-dir build/runtime-smoke
```

## 7) Trade-offs and next steps

Trade-offs:

- Keeping timeout env configuration means smoke launch still uses one env knob.
- That is acceptable because timeout has no dedicated run-command flag yet.

Next steps:

- optionally expose runtime serve timeout as a dedicated run-command flag once timeout contract is finalized,
- keep smoke/workflow contracts synchronized when runtime launch arguments evolve.

## Verification

- `scripts/test-smoke-sec4-run-hello-api-script-contract.sh`
- `scripts/test-smoke-sec4-run-hello-api-script-contract-guard.sh`
- `scripts/smoke-sec4-run-hello-api.sh --artifacts-dir <tmp>`
- `scripts/check-runtime-smoke-artifacts.sh --artifacts-dir <tmp>`
- `scripts/test-runtime-smoke-workflow-contract.sh`
- `scripts/test-runtime-smoke-workflow-contract-guard.sh`
