# 285 M11 Slice: Signature-Window-Aware Effect Quickfix

This chapter documents improving `E2001` quick-fix anchoring with parser-derived function signature windows.

## What it is

Updated:
- `compiler/ailang-lsp/src/main.rs`

Key changes:
- effect quick-fix now derives a function signature search window from parsed AST spans:
  - function item span start,
  - function body span start (signature boundary).
- signature line detection is constrained to that window before text insertion.
- fallback behavior remains available when parser-based window cannot be derived.

## Why it exists

Pure text scanning for nearest `fn` line can be confused by `fn` text inside function bodies (for example comments/strings). Parser-informed signature windows reduce these false anchors while keeping edits lightweight.

## How it works internally

1. Parse current document and find function containing diagnostic line.
2. Compute signature search window from function/item spans.
3. Search for insertion anchors (`effects`, `->`, `{`) only within that window.
4. Emit insertion/append edit as before.

## Inputs, outputs, and constraints

- Inputs:
  - diagnostic line, source text, and parsed function spans.
- Outputs:
  - more reliable effect quick-fix edits in files containing body text that includes `fn`.
- Constraints:
  - final rewrite is still text-edit based; this is not full AST rewrite emission.

## Failure modes and diagnostics

- if parser window derivation fails, quick-fix falls back to legacy text-only search path.

## Tests added/updated

`compiler/ailang-lsp` now includes:
- `code_action_effect_quickfix_uses_function_signature_window_not_body_text_hits`:
  - validates `fn` text in function body comments does not break effect quick-fix anchoring.

## Tradeoffs and next steps

- Tradeoff:
  - significantly better anchoring without full rewrite complexity.
- Next:
  - move from window-constrained text edits to fully AST-aware code-action rewrites for complex signatures and formatting preservation.
