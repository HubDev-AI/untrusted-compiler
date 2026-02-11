# 30 Lexical Grammar and Tokens

This chapter documents the M1 lexer implementation in `compiler/ailang-core/src/lexer.rs` and token model in `compiler/ailang-core/src/token.rs`.

## What it is

A deterministic lexer that turns `.ai` source text into a stream of tokens with source spans.

## Why it exists

Parser and diagnostics quality both depend on stable tokenization and precise locations.

## Supported token classes (M1)

- Keywords: `fn`, `let`, `const`, `mut`, `if`, `else`, `match`, `return`, `struct`, `enum`
- Literals: number, string, bool
- Identifiers
- Symbols/operators: grouping, separators, `->`, `=>`, arithmetic, comparisons, logic, assignment, `?`
- End-of-file marker

## How it works internally

- Lexing is single-pass over UTF-8 chars with line/column tracking.
- Every produced token includes a `Span` with start/end positions.
- `//` comments and whitespace are skipped.
- Strings support basic escapes (`\n`, `\t`, `\r`, `\"`, `\\`).

## Lexer diagnostics (M1)

- `L1001`: unexpected character
- `L1002`: unterminated string literal
- `L1003`: invalid escape sequence

## Constraints and tradeoffs

- No multiline string literals yet.
- Numeric literals are currently represented as source text (not constant-folded).
- The token set is intentionally minimal for M1 parsing goals.
