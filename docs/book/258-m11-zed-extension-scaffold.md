# 258 M11 Slice: Zed Extension Scaffold

This chapter documents introducing the initial `zed-extension` scaffold for Untrusted<T> editor integration.

## What it is

Added:
- `zed-extension/extension.toml`
- `zed-extension/languages/untrusted/config.toml`
- `zed-extension/src/lib.rs`
- `zed-extension/README.md`

Key scaffold behavior:
- declares `Untrusted<T>` language metadata for `.ut` files.
- registers `sec4audit-lsp` language server wiring.
- launches `sec4audit-language-server --stdio` from the extension runtime.
- registers tree-sitter grammar source placeholder (`repository` + `rev`).

## Why it exists

M11 is not complete with server-only work. Editor adoption requires a client integration layer. This scaffold establishes the official Zed packaging path without blocking on full tree-sitter grammar completion.

## How it works internally

1. Zed reads `extension.toml` to discover language + server + grammar declarations.
2. Zed loads `languages/untrusted/config.toml` for file association/comments/brackets.
3. Zed invokes extension Rust entrypoint (`src/lib.rs`) and requests server command.
4. Extension returns `sec4audit-language-server --stdio`.
5. Existing LSP server handles diagnostics/navigation/completion/rename/code-action flows.

## Inputs, outputs, and constraints

- Inputs:
  - `.ut` files opened in Zed.
  - local availability of `sec4audit-language-server`.
- Outputs:
  - Zed-to-LSP connection command contract.
- Constraints:
  - grammar revision is a placeholder and must be replaced with a concrete tree-sitter commit.
  - scaffold is not yet distributed/published as a packaged extension artifact in this slice.

## Failure modes and diagnostics

- missing `sec4audit-language-server` in PATH -> extension cannot launch LSP.
- unresolved grammar commit placeholder -> syntax-highlighting integration is incomplete until updated.

## Verification

- Existing `sec4audit-language-server` unit suite remains green:
  - `cargo test -p sec4audit-language-server`

## Tradeoffs and next steps

- Tradeoff:
  - scaffold-first approach unblocks wiring early but leaves grammar publishing/final packaging for follow-up.
- Next:
  - add `tree-sitter-untrusted` grammar repository and pin real commit SHA,
  - add extension packaging/distribution checks,
  - add Zed smoke workflow against real `.ut` fixtures.
