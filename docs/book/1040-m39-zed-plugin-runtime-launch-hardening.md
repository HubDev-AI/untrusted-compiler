# 1040 M39 Slice: Zed Plugin Runtime-Launch Hardening

This slice hardens `zed-extension` language-server launch behavior for local development and clearer operator failure recovery.

## What changed

1. Added deterministic launch resolution in `/Users/vladimirtrifonov/src/ai/AILang/zed-extension/src/lib.rs`:
   - reads optional `lsp.sec4audit-lsp.binary.path` override from Zed settings,
   - otherwise resolves `sec4audit-language-server` from `PATH`,
   - otherwise checks local workspace fallback paths under `target/{debug,release}` and `compiler/sec4-lsp/target/{debug,release}`.
2. Added explicit startup diagnostics:
   - invalid configured path now returns a clear extension error with the exact setting key,
   - unresolved default path now reports the full searched fallback set and the local build recovery command.
3. Added deterministic command argument/environment handling:
   - honors configured `binary.arguments`/`binary.env`,
   - always guarantees `--stdio` is present.
4. Updated extension operator docs with:
   - binary resolution order,
   - explicit Zed settings override example.

## Why

The previous extension assumed the language-server binary was already on `PATH`, which produced opaque startup failures for local contributors. This slice makes startup behavior predictable and recoverable without trial-and-error.

## Validation

- `scripts/check-zed-grammar-pin.sh`
- `cargo run -p sec4 -- check --path examples/zed-plugin-smoke`

## Notes

The repository does not currently include a standalone Cargo target for `zed-extension`, so this slice validates behavior through deterministic code-path review and operator smoke flow rather than crate-level compile tests.
