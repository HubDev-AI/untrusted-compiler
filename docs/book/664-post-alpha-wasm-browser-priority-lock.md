# Post-Alpha WASM/Browser Priority Lock

## What it is

This chapter locks roadmap sequencing for a future WASM backend and browser sandbox runtime profile.

WASM/browser work is explicitly prioritized, but only after no-stub alpha closure is complete.

## Why it exists

The project is currently in implementation-first no-stub alpha completion.

Starting WASM too early would split effort and increase risk on the main runtime/compiler closure path.

This lock keeps one clear sequence:

1. Finish no-stub alpha.
2. Validate + benchmark + publish alpha artifacts.
3. Start WASM/browser track as highest-priority new feature work.

## How it works

The roadmap now defines `WASM_START_GATE`.

`WASM_START_GATE` opens only when all are true:

1. no-stub alpha readiness criteria are fully satisfied.
2. alpha decision record is `GO`.
3. benchmark evidence is current/published.
4. alpha artifacts are published and consumable.

When the gate is open, WASM/backend-browser profile becomes the top new-feature priority.

## Validation

- Roadmap contains explicit `Post-alpha priority lock (WASM/browser)` section under no-stub readiness.
- Build strategy execution order includes post-alpha WASM/backend-browser expansion step.
- This chapter is indexed in `docs/book/README.md`.

## Trade-offs

- Defers WASM experimentation until alpha is stable.
- Improves release confidence and avoids partial backend divergence during alpha closure.

## Next

1. Continue current no-stub alpha implementation slices (runtime/compiler hardening).
2. Close alpha release evidence (`GO` + benchmark + publish chain).
3. Open WASM milestone planning immediately after `WASM_START_GATE` is satisfied.
