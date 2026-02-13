# 292 M11 Slice: AST-Backed Effect Quickfix Rewrites

This chapter documents AST-backed insertion anchors for missing-effect code actions.

## What it is

Updated:
- `compiler/sec4-lsp/src/main.rs`
- `docs/05-sec4-master-roadmap.md`

Key changes:
- `build_missing_effect_edit(...)` now derives function signature context from parsed AST (`ItemKind::Function`) for the diagnostic location.
- Missing-effect quickfix behavior now uses AST data first:
  - if effects already exist, insert after the last declared effect span,
  - if effect already declared, omit quickfix immediately.
- Existing signature-window fallback remains for no-effects/arrow insertion paths.

## Why it exists

M11 still had one remaining precision gap: complex multiline signature/effects layouts could require brittle text scanning to place effect edits.

This slice ties the primary insertion path to parsed function/effect spans so edits remain stable as signature formatting changes.

## How it works internally

1. Parse active source and locate the function that encloses the diagnostic line.
2. Build `FunctionSignatureContext` containing:
   - signature line window,
   - declared effect names,
   - last effect span (if present).
3. If the requested effect is already declared, return no action.
4. If the function already has effects, insert `, <effect>` at the end of the last effect span.
5. If no effects exist, reuse existing signature-window insertion fallback.

## Tests added or updated

Updated in `compiler/sec4-lsp/src/main.rs`:
- `code_action_can_append_missing_effect_for_multiline_effect_list_block`

Existing quickfix tests remain green:
- `code_action_can_append_missing_effect_for_multiline_effects_signature`
- `code_action_effect_quickfix_uses_function_signature_window_not_body_text_hits`
- `code_action_omits_missing_effect_quickfix_when_already_declared`

## Tradeoffs and next steps

- Tradeoff:
  - AST-first insertion is currently focused on effect-append cases; insertion for no-effects signatures still uses fallback scanning.
- Next:
  - continue with M12/M9+ stabilization and any deeper code-action AST rewrites only if new edge cases appear.
