# M37-S5 Final No-Stub Verification Pass

## What it is

M37-S5 is the full verification pass for the no-stub alpha baseline on `main`.

This slice is not new feature implementation; it is the proof step that the implemented runtime/compiler behavior holds together across the full test matrix.

## Why it exists

After runtime de-stubbing slices (redirect parity, JSON hardening, middleware policy materialization, structured logging), alpha readiness depends on evidence, not assumptions.

This pass establishes that evidence with one canonical matrix.

## How it works

### Verification matrix executed

1. `cargo test -p sec4 --test json_output`
2. `cargo test -p sec4 --test commands`
3. `cargo test -p sec4 --test alpha_smoke`
4. `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`

### Results

- `json_output`: `161 passed; 0 failed`
- `commands`: `36 passed; 0 failed`
- `alpha_smoke`: `2 passed; 0 failed`
- `c_backend` targeted contract: passed

### Residual-gap note (for alpha decision)

No remaining placeholder/stub behavior was found in the supported v0.1 runtime/CLI paths exercised by this matrix.

Known non-blocking limits remain:

- TLS runtime support still depends on OpenSSL toolchain availability; `--tls-backend auto` now falls back to non-TLS compilation when OpenSSL linkage is unavailable.

## Validation

- `cargo test -p sec4 --test json_output`
- `cargo test -p sec4 --test commands`
- `cargo test -p sec4 --test alpha_smoke`
- `cargo test -p sec4-core --test c_backend c_backend_emits_runtime_header_and_source`
- `scripts/test-roadmap-closure-gate-alignment.sh`

## Trade-offs

- A full verification pass costs more runtime than targeted checks, but it prevents regressions from being hidden behind narrow green tests.
- Residual limits are explicitly documented to avoid overstating alpha scope.

## Next

1. Draft alpha release notes from verified no-stub evidence and M37-S6 checklist artifacts.
2. Decide alpha tag based on checklist + release-note package.
3. Record M37 closure outcome in roadmap/book.
