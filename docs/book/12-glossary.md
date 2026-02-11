# 12 Glossary (M0)

## AST
Abstract Syntax Tree produced by parsing source text.

## HIR
High-level intermediate representation, typically post-name-resolution/type-attachment.

## MIR
Mid-level intermediate representation used for explicit control/data flow and backend lowering.

## Span
Source location metadata (file/line/column range) attached to diagnostics and syntax structures.

## Diagnostic
Structured compiler feedback object containing severity, code, message, span, and notes.

## Typed sink
A type-safe boundary API that prevents unsafe operations by construction (e.g., SQL query object types).

## Effect
A declared side-effect category used to audit runtime behavior (e.g., db.read, db.write, log).
