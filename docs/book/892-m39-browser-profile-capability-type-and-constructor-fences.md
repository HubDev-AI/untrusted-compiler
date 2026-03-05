# M39-S1 Extension - Browser Profile Capability Type and Constructor Fences

## What Changed

- Extended semantic profile fencing in `compiler/sec4-core/src/semantic.rs` to reject server-only capability usage in browser profile at type and constructor layers.
- Added type-level browser-profile diagnostics for:
  - `Ctx`
  - `DbCap`
  - `FsCap`
  - `NetCap`
  - `InternalNetCap`
  - `SecretsCap`
- Added constructor-level browser-profile diagnostics for:
  - `Ctx()`
  - `DbCap()`
  - `FsCap()`
  - `NetCap()`
  - `InternalNetCap()`
  - `SecretsCap()`

## Why

Callsite fences (`db.*`, `secrets.*`, `http.serve`, `httpClient.getInternal`, `url.internal`) still allowed browser-profile programs to carry server-only capability types in signatures or local constructor calls. That weakens determinism for browser-target authoring and leaves avoidable migration churn before WASM work.

## Diagnostic Contract

- Type usage rejection:
  - code: `E2002`
  - message: `server-only capability type is disabled in browser profile`
- Constructor usage rejection:
  - code: `E2002`
  - message: `server-only capability constructor is disabled in browser profile`

Both diagnostics include profile guidance to switch to `[build].profile = "server"` when server capability access is required.

## Validation

- `cargo test -p sec4 --test commands`
- Added test:
  - `check_fails_when_browser_profile_uses_server_capability_types_and_constructors`

## Trade-offs

- This extension intentionally fences explicit type/constructor usage only.
- Promotion-architecture composition analysis (`domain` vs adapter boundaries) remains tracked as `M39-S2`.
