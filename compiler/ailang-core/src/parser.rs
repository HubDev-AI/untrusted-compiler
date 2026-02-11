use crate::ast::{
    BinaryOp, Block, EffectSpec, EnumDecl, EnumVariant, Expr, ExprKind, FieldDecl, FunctionDecl,
    Item, ItemKind, MatchArm, Param, Pattern, PatternKind, Program, Stmt, StmtKind, StructDecl,
    TypeExpr, TypeExprKind, UnaryOp, VariantField,
};
use crate::diagnostics::{Diagnostic, Span};
use crate::lexer;
use crate::token::{Keyword, Symbol, Token, TokenKind};
use std::path::Path;

pub fn parse_source(file: &Path, source: &str) -> Result<Program, Vec<Diagnostic>> {
    let tokens = lexer::lex(file, source)?;
    Parser::new(tokens).parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            index: 0,
            diagnostics: Vec::new(),
        }
    }

    fn parse_program(mut self) -> Result<Program, Vec<Diagnostic>> {
        let start_span = self.current().span.clone();
        let mut items = Vec::new();

        while !self.is_eof() {
            let item_result = if let Some(start) = self.match_keyword(Keyword::Fn) {
                self.parse_function_item(start)
            } else if let Some(start) = self.match_keyword(Keyword::Struct) {
                self.parse_struct_item(start)
            } else if let Some(start) = self.match_keyword(Keyword::Enum) {
                self.parse_enum_item(start)
            } else {
                let token = self.current().clone();
                self.diagnostics.push(
                    Diagnostic::error("P2001", "expected top-level declaration", token.span)
                        .with_note("top-level items must start with `fn`, `struct`, or `enum`"),
                );
                self.synchronize_top_level();
                continue;
            };

            match item_result {
                Ok(item) => items.push(item),
                Err(diagnostic) => {
                    self.diagnostics.push(diagnostic);
                    self.synchronize_top_level();
                }
            }
        }

        if !self.diagnostics.is_empty() {
            return Err(self.diagnostics);
        }

        let span = if let Some(last_item) = items.last() {
            join_spans(&start_span, &last_item.span)
        } else {
            start_span
        };

        Ok(Program { items, span })
    }

    fn synchronize_top_level(&mut self) {
        while !self.is_eof() {
            if self.check_keyword(Keyword::Fn)
                || self.check_keyword(Keyword::Struct)
                || self.check_keyword(Keyword::Enum)
            {
                return;
            }
            self.bump();
        }
    }

    fn parse_function_item(&mut self, start: Token) -> Result<Item, Diagnostic> {
        let (name, _name_token) =
            self.expect_identifier("P2002", "expected function name after `fn`")?;
        self.expect_symbol(Symbol::LParen, "P2003", "expected `(` after function name")?;

        let mut params = Vec::new();
        if !self.check_symbol(Symbol::RParen) {
            loop {
                let (param_name, param_name_token) =
                    self.expect_identifier("P2004", "expected parameter name")?;
                self.expect_symbol(Symbol::Colon, "P2005", "expected `:` after parameter name")?;
                let param_type = self.parse_type()?;
                let param_span = join_spans(&param_name_token.span, &param_type.span);
                params.push(Param {
                    name: param_name,
                    ty: param_type,
                    span: param_span,
                });

                if self.match_symbol(Symbol::Comma).is_some() {
                    continue;
                }
                break;
            }
        }

        self.expect_symbol(
            Symbol::RParen,
            "P2006",
            "expected `)` after function parameters",
        )?;

        let mut effects = Vec::new();
        let mut return_type = None;
        loop {
            if effects.is_empty() {
                if let Some(effects_keyword) = self.match_keyword(Keyword::Effects) {
                    effects = self.parse_effects_clause(effects_keyword)?;
                    continue;
                }
            }

            if return_type.is_none() && self.match_symbol(Symbol::Arrow).is_some() {
                return_type = Some(self.parse_type()?);
                continue;
            }

            break;
        }

        let body = self.parse_block()?;
        let span = join_spans(&start.span, &body.span);

        Ok(Item {
            kind: ItemKind::Function(FunctionDecl {
                name,
                params,
                effects,
                return_type,
                body,
            }),
            span,
        })
    }

    fn parse_effects_clause(&mut self, _start: Token) -> Result<Vec<EffectSpec>, Diagnostic> {
        self.expect_symbol(
            Symbol::LBrace,
            "P2007",
            "expected `{` after `effects` keyword",
        )?;

        let mut effects = Vec::new();
        while !self.check_symbol(Symbol::RBrace) && !self.is_eof() {
            let effect = self.parse_effect_path()?;
            effects.push(effect);

            if self.match_symbol(Symbol::Comma).is_some() {
                continue;
            }

            if self.check_symbol(Symbol::RBrace) {
                break;
            }

            return Err(self.error_current("P2008", "expected `,` or `}` in effects declaration"));
        }

        self.expect_symbol(
            Symbol::RBrace,
            "P2009",
            "expected `}` to close effects declaration",
        )?;

        Ok(effects)
    }

    fn parse_effect_path(&mut self) -> Result<EffectSpec, Diagnostic> {
        let (segment, token) = self.expect_identifier("P2040", "expected effect name segment")?;
        let mut path = vec![segment];
        let mut span = token.span;

        while self.match_symbol(Symbol::Dot).is_some() {
            let (next, next_token) =
                self.expect_identifier("P2041", "expected effect segment after `.`")?;
            span = join_spans(&span, &next_token.span);
            path.push(next);
        }

        Ok(EffectSpec { path, span })
    }

    fn parse_struct_item(&mut self, start: Token) -> Result<Item, Diagnostic> {
        let (name, _name_token) =
            self.expect_identifier("P2010", "expected struct name after `struct`")?;
        self.expect_symbol(Symbol::LBrace, "P2011", "expected `{` after struct name")?;

        let mut fields = Vec::new();
        while !self.check_symbol(Symbol::RBrace) && !self.is_eof() {
            let (field_name, field_name_token) =
                self.expect_identifier("P2012", "expected struct field name")?;
            self.expect_symbol(
                Symbol::Colon,
                "P2013",
                "expected `:` after struct field name",
            )?;
            let field_type = self.parse_type()?;
            let field_span = join_spans(&field_name_token.span, &field_type.span);
            fields.push(FieldDecl {
                name: field_name,
                ty: field_type,
                span: field_span,
            });

            if self.match_symbol(Symbol::Comma).is_some() {
                continue;
            }
            if self.check_symbol(Symbol::RBrace) {
                break;
            }

            return Err(self.error_current("P2014", "expected `,` or `}` in struct declaration"));
        }

        let end = self.expect_symbol(Symbol::RBrace, "P2015", "expected `}` to close struct")?;
        let span = join_spans(&start.span, &end.span);

        Ok(Item {
            kind: ItemKind::Struct(StructDecl { name, fields }),
            span,
        })
    }

    fn parse_enum_item(&mut self, start: Token) -> Result<Item, Diagnostic> {
        let (name, _name_token) =
            self.expect_identifier("P2020", "expected enum name after `enum`")?;
        self.expect_symbol(Symbol::LBrace, "P2021", "expected `{` after enum name")?;

        let mut variants = Vec::new();
        while !self.check_symbol(Symbol::RBrace) && !self.is_eof() {
            let (variant_name, variant_name_token) =
                self.expect_identifier("P2022", "expected enum variant name")?;
            let mut variant_span = variant_name_token.span.clone();
            let mut payload = Vec::new();

            if self.match_symbol(Symbol::LParen).is_some() {
                if !self.check_symbol(Symbol::RParen) {
                    loop {
                        let field = self.parse_variant_field()?;
                        variant_span = join_spans(&variant_span, &field.span);
                        payload.push(field);

                        if self.match_symbol(Symbol::Comma).is_some() {
                            if self.check_symbol(Symbol::RParen) {
                                break;
                            }
                            continue;
                        }
                        break;
                    }
                }

                let right_paren = self.expect_symbol(
                    Symbol::RParen,
                    "P2023",
                    "expected `)` after enum variant payload",
                )?;
                variant_span = join_spans(&variant_span, &right_paren.span);
            }

            variants.push(EnumVariant {
                name: variant_name,
                payload,
                span: variant_span,
            });

            if self.match_symbol(Symbol::Comma).is_some() {
                continue;
            }
            if self.check_symbol(Symbol::RBrace) {
                break;
            }

            return Err(self.error_current("P2024", "expected `,` or `}` in enum declaration"));
        }

        let end = self.expect_symbol(Symbol::RBrace, "P2025", "expected `}` to close enum")?;
        let span = join_spans(&start.span, &end.span);

        Ok(Item {
            kind: ItemKind::Enum(EnumDecl { name, variants }),
            span,
        })
    }

    fn parse_variant_field(&mut self) -> Result<VariantField, Diagnostic> {
        if let TokenKind::Identifier(name) = self.current().kind.clone() {
            let is_named = matches!(self.peek_kind(1), Some(TokenKind::Symbol(Symbol::Colon)));

            if is_named {
                let name_token = self.bump();
                self.expect_symbol(
                    Symbol::Colon,
                    "P2026",
                    "expected `:` after named variant payload field",
                )?;
                let ty = self.parse_type()?;
                let span = join_spans(&name_token.span, &ty.span);

                return Ok(VariantField {
                    name: Some(name),
                    ty,
                    span,
                });
            }
        }

        let ty = self.parse_type()?;
        let span = ty.span.clone();
        Ok(VariantField {
            name: None,
            ty,
            span,
        })
    }

    fn parse_type(&mut self) -> Result<TypeExpr, Diagnostic> {
        let (name, name_token) = self.expect_identifier("P2030", "expected type name")?;

        let mut args = Vec::new();
        let mut end_span = name_token.span.clone();
        if self.match_symbol(Symbol::Lt).is_some() {
            if self.check_symbol(Symbol::Gt) {
                return Err(
                    self.error_current("P2031", "generic type argument list cannot be empty")
                );
            }

            loop {
                let arg = self.parse_type()?;
                args.push(arg);

                if self.match_symbol(Symbol::Comma).is_some() {
                    continue;
                }
                break;
            }

            let gt =
                self.expect_symbol(Symbol::Gt, "P2032", "expected `>` after generic arguments")?;
            end_span = gt.span;
        }

        let mut ty = TypeExpr {
            kind: TypeExprKind::Named { name, args },
            span: join_spans(&name_token.span, &end_span),
        };

        if let Some(question) = self.match_symbol(Symbol::Question) {
            let wrapped_span = join_spans(&ty.span, &question.span);
            ty = TypeExpr {
                kind: TypeExprKind::Named {
                    name: "Option".to_string(),
                    args: vec![ty],
                },
                span: wrapped_span,
            };
        }

        Ok(ty)
    }

    fn parse_block(&mut self) -> Result<Block, Diagnostic> {
        let start = self.expect_symbol(Symbol::LBrace, "P2100", "expected `{` to start block")?;

        let mut statements = Vec::new();
        let mut tail = None;

        while !self.check_symbol(Symbol::RBrace) && !self.is_eof() {
            if let Some(let_token) = self.match_keyword(Keyword::Let) {
                let stmt = self.parse_let_statement(false, let_token)?;
                self.consume_statement_terminator()?;
                statements.push(stmt);
                continue;
            }

            if let Some(const_token) = self.match_keyword(Keyword::Const) {
                let stmt = self.parse_let_statement(true, const_token)?;
                self.consume_statement_terminator()?;
                statements.push(stmt);
                continue;
            }

            if let Some(return_token) = self.match_keyword(Keyword::Return) {
                let stmt = self.parse_return_statement(return_token)?;
                self.consume_statement_terminator()?;
                statements.push(stmt);
                continue;
            }

            let expr = self.parse_expr()?;
            if self.match_symbol(Symbol::Semicolon).is_some() {
                let span = expr.span.clone();
                statements.push(Stmt {
                    kind: StmtKind::Expr { expr },
                    span,
                });
                continue;
            }

            if self.check_symbol(Symbol::RBrace) {
                tail = Some(Box::new(expr));
                break;
            }

            return Err(
                self.error_current("P2101", "expected `;` or `}` after expression in block")
            );
        }

        let end = self.expect_symbol(Symbol::RBrace, "P2102", "expected `}` to close block")?;

        Ok(Block {
            statements,
            tail,
            span: join_spans(&start.span, &end.span),
        })
    }

    fn parse_let_statement(&mut self, is_const: bool, start: Token) -> Result<Stmt, Diagnostic> {
        let mutable = if is_const {
            false
        } else {
            self.match_keyword(Keyword::Mut).is_some()
        };

        let (name, _name_token) = self.expect_identifier(
            "P2103",
            if is_const {
                "expected constant name"
            } else {
                "expected variable name"
            },
        )?;

        let ty = if self.match_symbol(Symbol::Colon).is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect_symbol(Symbol::Eq, "P2104", "expected `=` in variable declaration")?;
        let value = self.parse_expr()?;
        let span = join_spans(&start.span, &value.span);

        Ok(Stmt {
            kind: StmtKind::Let {
                is_const,
                mutable,
                name,
                ty,
                value,
            },
            span,
        })
    }

    fn parse_return_statement(&mut self, start: Token) -> Result<Stmt, Diagnostic> {
        let value = if self.check_symbol(Symbol::Semicolon) || self.check_symbol(Symbol::RBrace) {
            None
        } else {
            Some(self.parse_expr()?)
        };

        let span = if let Some(expr) = &value {
            join_spans(&start.span, &expr.span)
        } else {
            start.span
        };

        Ok(Stmt {
            kind: StmtKind::Return { value },
            span,
        })
    }

    fn consume_statement_terminator(&mut self) -> Result<(), Diagnostic> {
        if self.match_symbol(Symbol::Semicolon).is_some() || self.check_symbol(Symbol::RBrace) {
            Ok(())
        } else {
            Err(self.error_current("P2105", "expected `;` after statement"))
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_and()?;
        while self.match_symbol(Symbol::OrOr).is_some() {
            let right = self.parse_and()?;
            let span = join_spans(&expr.span, &right.span);
            expr = Expr {
                kind: ExprKind::Binary {
                    op: BinaryOp::Or,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            };
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_equality()?;
        while self.match_symbol(Symbol::AndAnd).is_some() {
            let right = self.parse_equality()?;
            let span = join_spans(&expr.span, &right.span);
            expr = Expr {
                kind: ExprKind::Binary {
                    op: BinaryOp::And,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = if self.match_symbol(Symbol::EqEq).is_some() {
                Some(BinaryOp::Eq)
            } else if self.match_symbol(Symbol::BangEq).is_some() {
                Some(BinaryOp::Ne)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };

            let right = self.parse_comparison()?;
            let span = join_spans(&expr.span, &right.span);
            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_term()?;

        loop {
            let op = if self.match_symbol(Symbol::Lt).is_some() {
                Some(BinaryOp::Lt)
            } else if self.match_symbol(Symbol::LtEq).is_some() {
                Some(BinaryOp::Le)
            } else if self.match_symbol(Symbol::Gt).is_some() {
                Some(BinaryOp::Gt)
            } else if self.match_symbol(Symbol::GtEq).is_some() {
                Some(BinaryOp::Ge)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };

            let right = self.parse_term()?;
            let span = join_spans(&expr.span, &right.span);
            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_factor()?;

        loop {
            let op = if self.match_symbol(Symbol::Plus).is_some() {
                Some(BinaryOp::Add)
            } else if self.match_symbol(Symbol::Minus).is_some() {
                Some(BinaryOp::Sub)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };

            let right = self.parse_factor()?;
            let span = join_spans(&expr.span, &right.span);
            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = if self.match_symbol(Symbol::Star).is_some() {
                Some(BinaryOp::Mul)
            } else if self.match_symbol(Symbol::Slash).is_some() {
                Some(BinaryOp::Div)
            } else if self.match_symbol(Symbol::Percent).is_some() {
                Some(BinaryOp::Rem)
            } else {
                None
            };

            let Some(op) = op else {
                break;
            };

            let right = self.parse_unary()?;
            let span = join_spans(&expr.span, &right.span);
            expr = Expr {
                kind: ExprKind::Binary {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                },
                span,
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, Diagnostic> {
        if let Some(token) = self.match_symbol(Symbol::Bang) {
            let expr = self.parse_unary()?;
            let span = join_spans(&token.span, &expr.span);
            return Ok(Expr {
                kind: ExprKind::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                },
                span,
            });
        }

        if let Some(token) = self.match_symbol(Symbol::Minus) {
            let expr = self.parse_unary()?;
            let span = join_spans(&token.span, &expr.span);
            return Ok(Expr {
                kind: ExprKind::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                },
                span,
            });
        }

        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_symbol(Symbol::Dot).is_some() {
                let (field, field_token) =
                    self.expect_identifier("P2204", "expected member name after `.`")?;
                let span = join_spans(&expr.span, &field_token.span);
                expr = Expr {
                    kind: ExprKind::Member {
                        object: Box::new(expr),
                        field,
                    },
                    span,
                };
                continue;
            }

            if self.match_symbol(Symbol::LParen).is_some() {
                let mut args = Vec::new();
                if !self.check_symbol(Symbol::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if self.match_symbol(Symbol::Comma).is_some() {
                            continue;
                        }
                        break;
                    }
                }

                let right_paren = self.expect_symbol(
                    Symbol::RParen,
                    "P2202",
                    "expected `)` after call arguments",
                )?;
                let span = join_spans(&expr.span, &right_paren.span);
                expr = Expr {
                    kind: ExprKind::Call {
                        callee: Box::new(expr),
                        args,
                    },
                    span,
                };
                continue;
            }

            break;
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        if self.check_keyword(Keyword::If) {
            return self.parse_if_expr();
        }

        if self.check_keyword(Keyword::Match) {
            return self.parse_match_expr();
        }

        if self.check_symbol(Symbol::LBrace) {
            let block = self.parse_block()?;
            let span = block.span.clone();
            return Ok(Expr {
                kind: ExprKind::Block(block),
                span,
            });
        }

        if self.match_symbol(Symbol::LParen).is_some() {
            let expr = self.parse_expr()?;
            self.expect_symbol(Symbol::RParen, "P2203", "expected `)` after expression")?;
            return Ok(expr);
        }

        let token = self.bump();
        match token.kind {
            TokenKind::Identifier(name) => Ok(Expr {
                kind: ExprKind::Identifier(name),
                span: token.span,
            }),
            TokenKind::Number(value) => Ok(Expr {
                kind: ExprKind::Number(value),
                span: token.span,
            }),
            TokenKind::String(value) => Ok(Expr {
                kind: ExprKind::String(value),
                span: token.span,
            }),
            TokenKind::Bool(value) => Ok(Expr {
                kind: ExprKind::Bool(value),
                span: token.span,
            }),
            _ => Err(
                Diagnostic::error("P2201", "expected expression", token.span)
                    .with_note(format!("found {}", token.kind.describe())),
            ),
        }
    }

    fn parse_if_expr(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.expect_keyword(Keyword::If, "P2210", "expected `if`")?;
        let condition = self.parse_expr()?;
        let then_branch = self.parse_block()?;

        let mut end_span = then_branch.span.clone();
        let else_branch = if self.match_keyword(Keyword::Else).is_some() {
            let expr = if self.check_keyword(Keyword::If) {
                self.parse_if_expr()?
            } else if self.check_symbol(Symbol::LBrace) {
                let block = self.parse_block()?;
                let span = block.span.clone();
                Expr {
                    kind: ExprKind::Block(block),
                    span,
                }
            } else {
                return Err(self.error_current("P2211", "expected `if` or block after `else`"));
            };

            end_span = expr.span.clone();
            Some(Box::new(expr))
        } else {
            None
        };

        Ok(Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch,
                else_branch,
            },
            span: join_spans(&start.span, &end_span),
        })
    }

    fn parse_match_expr(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.expect_keyword(Keyword::Match, "P2220", "expected `match`")?;
        let scrutinee = self.parse_expr()?;
        self.expect_symbol(
            Symbol::LBrace,
            "P2221",
            "expected `{` after match expression",
        )?;

        let mut arms = Vec::new();
        while !self.check_symbol(Symbol::RBrace) && !self.is_eof() {
            let pattern = self.parse_pattern()?;
            self.expect_symbol(Symbol::FatArrow, "P2222", "expected `=>` in match arm")?;
            let value = self.parse_expr()?;
            let span = join_spans(&pattern.span, &value.span);
            arms.push(MatchArm {
                pattern,
                value,
                span,
            });

            if self.match_symbol(Symbol::Comma).is_some() {
                continue;
            }
            if self.check_symbol(Symbol::RBrace) {
                break;
            }

            return Err(self.error_current("P2223", "expected `,` or `}` after match arm"));
        }

        let end = self.expect_symbol(Symbol::RBrace, "P2224", "expected `}` to close match")?;

        Ok(Expr {
            kind: ExprKind::Match {
                scrutinee: Box::new(scrutinee),
                arms,
            },
            span: join_spans(&start.span, &end.span),
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, Diagnostic> {
        let token = self.bump();
        match token.kind {
            TokenKind::Identifier(name) => {
                if name == "_" {
                    return Ok(Pattern {
                        kind: PatternKind::Wildcard,
                        span: token.span,
                    });
                }

                if self.match_symbol(Symbol::LParen).is_some() {
                    let mut args = Vec::new();
                    if !self.check_symbol(Symbol::RParen) {
                        loop {
                            args.push(self.parse_pattern()?);
                            if self.match_symbol(Symbol::Comma).is_some() {
                                continue;
                            }
                            break;
                        }
                    }

                    let end = self.expect_symbol(
                        Symbol::RParen,
                        "P2302",
                        "expected `)` after pattern arguments",
                    )?;
                    return Ok(Pattern {
                        kind: PatternKind::Variant { name, args },
                        span: join_spans(&token.span, &end.span),
                    });
                }

                Ok(Pattern {
                    kind: PatternKind::Identifier(name),
                    span: token.span,
                })
            }
            TokenKind::Number(value) => Ok(Pattern {
                kind: PatternKind::Number(value),
                span: token.span,
            }),
            TokenKind::String(value) => Ok(Pattern {
                kind: PatternKind::String(value),
                span: token.span,
            }),
            TokenKind::Bool(value) => Ok(Pattern {
                kind: PatternKind::Bool(value),
                span: token.span,
            }),
            _ => Err(Diagnostic::error("P2301", "expected pattern", token.span)
                .with_note(format!("found {}", token.kind.describe()))),
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.current().kind, TokenKind::Eof)
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn peek_kind(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens
            .get(self.index + offset)
            .map(|token| &token.kind)
    }

    fn bump(&mut self) -> Token {
        let token = self.current().clone();
        if !matches!(token.kind, TokenKind::Eof) {
            self.index += 1;
        }
        token
    }

    fn check_keyword(&self, keyword: Keyword) -> bool {
        matches!(self.current().kind, TokenKind::Keyword(value) if value == keyword)
    }

    fn match_keyword(&mut self, keyword: Keyword) -> Option<Token> {
        if self.check_keyword(keyword) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn expect_keyword(
        &mut self,
        keyword: Keyword,
        code: &str,
        message: &str,
    ) -> Result<Token, Diagnostic> {
        if self.check_keyword(keyword) {
            Ok(self.bump())
        } else {
            Err(self.error_current(code, message))
        }
    }

    fn check_symbol(&self, symbol: Symbol) -> bool {
        matches!(self.current().kind, TokenKind::Symbol(value) if value == symbol)
    }

    fn match_symbol(&mut self, symbol: Symbol) -> Option<Token> {
        if self.check_symbol(symbol) {
            Some(self.bump())
        } else {
            None
        }
    }

    fn expect_symbol(
        &mut self,
        symbol: Symbol,
        code: &str,
        message: &str,
    ) -> Result<Token, Diagnostic> {
        if self.check_symbol(symbol) {
            Ok(self.bump())
        } else {
            Err(self
                .error_current(code, message)
                .with_note(format!("expected symbol `{}`", symbol.as_str())))
        }
    }

    fn expect_identifier(
        &mut self,
        code: &str,
        message: &str,
    ) -> Result<(String, Token), Diagnostic> {
        let token = self.bump();
        let name = match &token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                return Err(Diagnostic::error(code, message, token.span)
                    .with_note(format!("found {}", token.kind.describe())))
            }
        };

        Ok((name, token))
    }

    fn error_current(&self, code: &str, message: &str) -> Diagnostic {
        let token = self.current().clone();
        Diagnostic::error(code, message, token.span)
            .with_note(format!("found {}", token.kind.describe()))
    }
}

fn join_spans(start: &Span, end: &Span) -> Span {
    Span {
        file: start.file.clone(),
        start_line: start.start_line,
        start_col: start.start_col,
        end_line: end.end_line,
        end_col: end.end_col,
    }
}
