# ailang-language-server

`ailang-language-server` is the LSP endpoint for AILang editor integrations.

## Run

```bash
cargo run -p ailang-language-server -- --stdio
```

Version:

```bash
cargo run -p ailang-language-server -- --version
```

## Current LSP support

- document lifecycle:
  - `textDocument/didOpen`
  - `textDocument/didChange`
  - `textDocument/didClose`
- diagnostics:
  - `textDocument/publishDiagnostics`
- navigation:
  - `textDocument/definition`
  - `textDocument/hover`
  - `textDocument/references`
  - `textDocument/implementation`
- editing assist:
  - `textDocument/completion`
  - `textDocument/prepareRename`
  - `textDocument/rename`
  - `textDocument/codeAction`

## Request/analysis limits

Environment variables:

- `AILANG_LSP_ANALYSIS_BUDGET_MS` (default `200`)
- `AILANG_LSP_MAX_DIAGNOSTICS` (default `200`)
- `AILANG_LSP_REQUEST_BUDGET_MS` (default uses analysis budget)
- `AILANG_LSP_SCAN_UNOPENED_FILES` (default `true`; set `false`, `0`, `off`, or `no` to disable unopened-file scan)

When analysis budget is exceeded, diagnostics include info code `I9001`.

## Security defaults

- no code execution.
- no build script execution.
- no network actions in server request handling.
- all analysis comes from `ailang-core` parser/semantic pipeline.
