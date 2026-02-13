# 258 M11 Slice: Zed Extension Scaffold

This chapter documents introducing the initial `zed-extension` scaffold for AILang editor integration.

## What it is

Added:
- `zed-extension/extension.toml`
- `zed-extension/languages/ailang/config.toml`
- `zed-extension/src/lib.rs`
- `zed-extension/README.md`

Key scaffold behavior:
- declares `AILang` language metadata for `.ai` files.
- registers `ailang-lsp` language server wiring.
- launches `ailang-language-server --stdio` from the extension runtime.
- registers tree-sitter grammar source placeholder (`repository` + `rev`).

## Why it exists

M11 is not complete with server-only work. Editor adoption requires a client integration layer. This scaffold establishes the official Zed packaging path without blocking on full tree-sitter grammar completion.

## How it works internally

1. Zed reads `extension.toml` to discover language + server + grammar declarations.
2. Zed loads `languages/ailang/config.toml` for file association/comments/brackets.
3. Zed invokes extension Rust entrypoint (`src/lib.rs`) and requests server command.
4. Extension returns `ailang-language-server --stdio`.
5. Existing LSP server handles diagnostics/navigation/completion/rename/code-action flows.

## Inputs, outputs, and constraints

- Inputs:
  - `.ai` files opened in Zed.
  - local availability of `ailang-language-server`.
- Outputs:
  - Zed-to-LSP connection command contract.
- Constraints:
  - grammar revision is a placeholder and must be replaced with a concrete tree-sitter commit.
  - scaffold is not yet distributed/published as a packaged extension artifact in this slice.

## Failure modes and diagnostics

- missing `ailang-language-server` in PATH -> extension cannot launch LSP.
- unresolved grammar commit placeholder -> syntax-highlighting integration is incomplete until updated.

## Verification

- Existing `ailang-language-server` unit suite remains green:
  - `cargo test -p ailang-language-server`

## Tradeoffs and next steps

- Tradeoff:
  - scaffold-first approach unblocks wiring early but leaves grammar publishing/final packaging for follow-up.
- Next:
  - add `tree-sitter-ailang` grammar repository and pin real commit SHA,
  - add extension packaging/distribution checks,
  - add Zed smoke workflow against real `.ai` fixtures.
