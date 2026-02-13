# 282 M11 Slice: Zed Grammar Pin Validation Gate

This chapter documents adding a release guard for Zed grammar revision pinning.

## What it is

Updated:
- `scripts/check-zed-grammar-pin.sh`
- `zed-extension/README.md`

Key changes:
- new validation script enforces that `zed-extension/extension.toml` has a non-placeholder grammar revision.
- script checks:
  - revision exists,
  - revision is not `TODO_SET_COMMIT_SHA`,
  - revision matches commit-like SHA format.

## Why it exists

M11 still requires pinning Zed grammar integration to an immutable published revision. Until the final repo/rev is set, this guardrail prevents accidental release with placeholder metadata.

## How it works internally

1. Read `grammars.sec4.rev` from `zed-extension/extension.toml`.
2. Validate presence and non-placeholder value.
3. Validate commit-like SHA shape.
4. Exit non-zero with actionable error if invalid.

## Inputs, outputs, and constraints

- Inputs:
  - `zed-extension/extension.toml`.
- Outputs:
  - pass/fail signal for CI/pre-release checks.
- Constraints:
  - it validates format/presence only; it does not verify remote repository existence.

## Failure modes and diagnostics

- missing file, missing revision, placeholder revision, or malformed revision all fail with explicit error messages.

## Tests added/updated

- no Rust test changes for this slice.
- operational validation is done by invoking:
  - `scripts/check-zed-grammar-pin.sh`

## Tradeoffs and next steps

- Tradeoff:
  - this is a policy/checkpoint gate, not the final pin itself.
- Next:
  - publish/finalize `tree-sitter-untrusted` repo revision and replace placeholder with pinned immutable SHA.
