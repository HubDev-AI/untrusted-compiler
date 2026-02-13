# 205 M9 Slice: Diagnostic Source Snippets and Tag Context

This chapter documents a Release Hardening (`M9`) slice that improves compiler diagnostic explainability in CLI output.

## What it is

Diagnostics rendered by the CLI now include:
- security/effects/schema tag context when present (`tags: ...`),
- source-line snippets with caret markers when source text is available.

The renderer still preserves existing location and note lines.

## Why it exists

`M9` requires improved diagnostics quality and error explainability. Prior output showed code/message/location, but developers still had to open files manually to map spans to source text.

Adding source snippets and tag context makes failures easier to triage during local work and CI log review.

## How it works internally

1. `Diagnostic::render_plain` and `Diagnostic::render_color` now load source text from `span.file` (best effort).
2. Both paths use a shared renderer:
   - builds heading + location,
   - prints `tags:` line when tags exist,
   - renders a three-line snippet block:
     - gutter line,
     - numbered source line,
     - caret marker aligned to span start/end.
3. If source cannot be read, rendering falls back to heading/location/tags/notes only.

## Inputs, outputs, and constraints

- Inputs:
  - `compiler/ailang-core/src/diagnostics.rs`
  - `compiler/ailang-cli/tests/json_output.rs`
- Outputs:
  - richer human-facing diagnostic text for non-JSON CLI mode,
  - regression tests for snippet + tag rendering behavior.
- Constraints:
  - JSON diagnostic mode is unchanged.
  - snippet rendering currently anchors to the start line only (multi-line spans are intentionally compact in this slice).

## Failure modes and diagnostics

- If source loading fails (missing file/path), diagnostics still render without snippets.
- If tag rendering regresses, new unit/integration tests fail with explicit missing-token assertions.
- If caret alignment logic regresses, tests fail on missing marker patterns.

## Example usage

Before:

```text
error[E4001]: req.query argument must be `String`
  --> src/main.ai:2:24
  note: found `Int`
```

After:

```text
error[E4001]: req.query argument must be `String`
  --> src/main.ai:2:24
  tags: security, schema
    |
  2 |   let name = req.query(12);
    |                        ^
  note: found `Int`
```

## Tradeoffs and next steps

- Tradeoff:
  - line-level snippet rendering is lightweight and deterministic, but does not yet show multi-line excerpts.
- Next:
  - add optional multi-line snippet windows for long-range spans,
  - include structured snippet payloads in LSP diagnostics once M11 resumes.
