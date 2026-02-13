# sec4audit-language-server

`sec4audit-language-server` is the LSP endpoint for Untrusted<T> editor integrations.

## Run

```bash
cargo run -p sec4audit-language-server -- --stdio
```

Version:

```bash
cargo run -p sec4audit-language-server -- --version
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

- `SEC4AUDIT_LSP_ANALYSIS_BUDGET_MS` (default `200`)
- `SEC4AUDIT_LSP_MAX_DIAGNOSTICS` (default `200`)
- `SEC4AUDIT_LSP_REQUEST_BUDGET_MS` (default uses analysis budget)
- `SEC4AUDIT_LSP_SCAN_UNOPENED_FILES` (default `true`; set `false`, `0`, `off`, or `no` to disable unopened-file scan)

When analysis budget is exceeded, diagnostics include info code `I9001`.

## Security defaults

- no code execution.
- no build script execution.
- no network actions in server request handling.
- all analysis comes from `sec4-core` parser/semantic pipeline.
