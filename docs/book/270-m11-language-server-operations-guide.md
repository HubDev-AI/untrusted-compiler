# 270 M11 Slice: Language Server Operations Guide

This chapter documents adding operational documentation for `sec4audit-language-server`.

## What it is

Added:
- `compiler/sec4-lsp/README.md`

Guide content includes:
- startup commands (`--stdio`, `--version`),
- supported LSP methods,
- runtime configuration via environment variables,
- default security posture.

## Why it exists

M11 now has substantial LSP behavior surface. A concrete operations guide is needed for local editor integration, reproducible testing, and future CI/editor packaging work.

## Inputs, outputs, and constraints

- Inputs:
  - local development/IDE setup.
- Outputs:
  - deterministic runbook for operating the language server.
- Constraints:
  - guide reflects current implementation scope (v0.1 baseline).

## Tradeoffs and next steps

- Tradeoff:
  - this is implementation-documentation, not a formal protocol spec replacement.
- Next:
  - align this runtime guide with Zed extension packaging docs once extension distribution flow is finalized.
