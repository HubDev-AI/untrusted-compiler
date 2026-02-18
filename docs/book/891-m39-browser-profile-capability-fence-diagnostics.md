# M39-S1 - Browser Profile Capability Fence Diagnostics

## What Changed

- Added `[build].profile` parsing in `compiler/sec4-core/src/manifest.rs` with deterministic values:
  - `server` (default)
  - `browser`
- Added manifest validation diagnostic `M0005` when profile is outside the allowed set.
- Extended semantic analysis in `compiler/sec4-core/src/semantic.rs` with browser-profile fences that emit `E2002` for server-only intrinsic groups.
- Wired `analyze_entry_with_allows(...)` in `compiler/sec4-core/src/lib.rs` to pass manifest profile into semantic analysis.
- Updated `sec4 init` in `compiler/sec4-cli/src/main.rs` to emit explicit `profile = "server"` in generated `sec4.toml`.

## Why

The post-alpha roadmap requires a browser/WASM direction, but the runtime today is still server-first. Without compile-time fencing, browser-target projects can accidentally compile server-only intrinsic usage and fail later during runtime/target transitions.

## Capability Fences Added

When `[build].profile = "browser"`, semantic analysis now rejects:

- `db.*` and `sql.q`
- `secrets.*`
- `http.serve`
- internal-net sink intrinsics (`httpClient.getInternal`, `url.internal`)

All of these diagnostics use deterministic `E2002` policy/security tags and include remediation guidance to switch profile when server capabilities are needed.

## Validation

- `cargo test -p sec4 --test commands`
- Key new tests:
  - `check_fails_when_browser_profile_uses_server_only_intrinsics`
  - `check_succeeds_when_browser_profile_uses_allowed_public_net_intrinsics`

## Trade-offs

- This slice intentionally fences intrinsic callsites, not all possible type/value paths yet.
- Constructor/capability alias-level browser-profile fencing is deferred to the next slice (`M39-S2`) to keep this change narrow and deterministic.
