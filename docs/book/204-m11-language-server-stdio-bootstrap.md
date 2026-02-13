# 204 M11 Slice: Language Server Stdio Bootstrap

This chapter documents the first executable scaffold for editor integration: a minimal `ailang-language-server` binary.

## What it is

Added a new workspace crate:
- `compiler/ailang-lsp`
- binary name: `ailang-language-server`

The server currently implements a minimal JSON-RPC/LSP stdio loop with support for:
- `initialize`
- `shutdown`
- `exit`
- method-not-found errors for unknown request methods

## Why it exists

The project needed a concrete LSP process endpoint to unblock editor wiring (`--stdio`) and integration planning for Zed/VS Code. This bootstrap establishes the process contract before full compiler-backed features are added.

## How it works internally

1. CLI contract:
   - `ailang-language-server --stdio`
   - `ailang-language-server --version`
2. Message framing:
   - reads `Content-Length` headers
   - parses JSON-RPC payloads
   - writes framed JSON-RPC responses.
3. Request handling:
   - `initialize` returns minimal capabilities and serverInfo
   - `shutdown` returns `null` result
   - `exit` stops loop
   - unknown request methods return `-32601`.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-lsp/Cargo.toml`
  - `compiler/ailang-lsp/src/main.rs`
  - workspace `Cargo.toml` member update
- Outputs:
  - runnable LSP bootstrap binary and unit tests for request/response framing.
- Constraints:
  - no compiler analysis integration yet
  - no diagnostics/completion/definition/hover support yet.

## Failure modes and diagnostics

- Missing/invalid `Content-Length` headers produce framed read errors.
- Invalid JSON payloads produce parse errors and terminate the loop with non-zero exit.
- Unsupported request methods return JSON-RPC method-not-found responses when request IDs are present.

## Example usage

```bash
cargo run -p ailang-language-server -- --stdio
```

```bash
cargo test -p ailang-language-server
```

## Tradeoffs and next steps

- Tradeoff: capabilities are intentionally minimal to avoid advertising unimplemented features.
- Next:
  - connect `initialize` workspace context to compiler project loading
  - add first language feature bridge (`textDocument/publishDiagnostics`) backed by `ailang-core`.
