# 479 M18 Editor Contract Expansion

This chapter documents M18-S4: editor-path execution slice for stable quickfix identifiers in the LSP layer.

## 1) What changed

In `compiler/sec4-lsp/src/main.rs` quickfix actions now include stable action IDs:

- `security.insert_validate_gate`
- `security.insert_redact`
- `effects.insert_missing_declaration`

The action payload now emits:

- `data.id` per quickfix action.

## 2) Why it matters

Stable action IDs make editor integrations deterministic for filtering/telemetry and align with the security-first tooling spec.

## 3) Verification

- Rust unit tests in `compiler/sec4-lsp/src/main.rs` now assert each quickfix ID.
- Contract checker:
  - `scripts/test-m18-editor-contract-expansion.sh`

## 4) Tradeoff

The contract checker is source-token based (fast in naming-lock CI). Behavioral guarantees remain covered by Rust unit tests.
