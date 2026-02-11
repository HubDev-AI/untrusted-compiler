# 31 Parser Design and AST

This chapter documents the recursive-descent parser in `compiler/ailang-core/src/parser.rs` and AST model in `compiler/ailang-core/src/ast.rs`.

## What it is

A span-preserving parser for core v0.1-lite declarations, statements, expressions, and types.

## Why it exists

M1 establishes the first real language frontend stage so `ailang check` can validate syntax and produce AST output.

## AST scope (M1)

Top-level items:
- `fn`
- `struct`
- `enum`

Statements:
- `let` / `let mut`
- `const`
- `return`
- expression statement

Expressions:
- literals, identifiers
- unary and binary operators
- call expressions
- `if` expressions
- `match` expressions
- block expressions

Types:
- named types with generic arguments (`Result<Option<User>, String>`)
- optional sugar (`T?` lowered to `Option<T>` in AST)

## Parser diagnostics (M1)

- `P2001+`: top-level declaration parsing errors
- `P2010+`: struct parsing errors
- `P2020+`: enum parsing errors
- `P2030+`: type parsing errors
- `P2100+`: block/statement parsing errors
- `P2200+`: expression parsing errors
- `P2300+`: pattern parsing errors

All diagnostics preserve file/line/column spans for deterministic tooling.

## Current parsing model

- Precedence-based expression parser for logical, equality, comparison, additive, multiplicative, unary, and call levels.
- Match arms require explicit `=>` and support wildcard/identifier/variant/literal patterns.
- Block tails are expression-based, while non-tail expressions require `;`.

## Constraints and tradeoffs

- Parser recovery is currently limited (syncs at top-level declarations).
- No module/import grammar yet.
- No semantic analysis yet (that starts in M2).
