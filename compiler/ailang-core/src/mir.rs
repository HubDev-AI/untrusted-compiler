use crate::ast::{
    self, BinaryOp, Block, Expr, ExprKind, FunctionDecl, ItemKind, Pattern, PatternKind, StmtKind,
    TypeExpr, UnaryOp,
};
use crate::Span;
use serde::Serialize;
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
}

impl MirProgram {
    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("MIR should serialize")
    }

    pub fn render_text(&self) -> String {
        let mut out = String::new();
        for (index, function) in self.functions.iter().enumerate() {
            if index > 0 {
                out.push('\n');
            }
            render_function(&mut out, function);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<MirParam>,
    pub effects: Vec<String>,
    pub return_type: Option<String>,
    pub blocks: Vec<MirBlock>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MirParam {
    pub name: String,
    pub ty: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MirBlock {
    pub id: usize,
    pub instructions: Vec<MirInstruction>,
    pub terminator: MirTerminator,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MirInstruction {
    pub kind: MirInstructionKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MirInstructionKind {
    Let { name: String, value: String },
    Eval { value: String },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MirTerminator {
    Return {
        value: Option<String>,
        span: Span,
    },
    Goto {
        target: usize,
        span: Span,
    },
    Branch {
        condition: String,
        then_target: usize,
        else_target: usize,
        span: Span,
    },
}

pub fn lower_program_to_mir(program: &ast::Program) -> MirProgram {
    let functions = program
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            ItemKind::Function(function) => Some(lower_function(function, &item.span)),
            ItemKind::Struct(_) | ItemKind::Enum(_) => None,
        })
        .collect::<Vec<_>>();

    MirProgram { functions }
}

fn lower_function(function: &FunctionDecl, span: &Span) -> MirFunction {
    let params = function
        .params
        .iter()
        .map(|param| MirParam {
            name: param.name.clone(),
            ty: type_expr_to_string(&param.ty),
            span: param.span.clone(),
        })
        .collect::<Vec<_>>();

    let effects = function
        .effects
        .iter()
        .map(|effect| effect.as_name())
        .collect::<Vec<_>>();

    let return_type = function.return_type.as_ref().map(type_expr_to_string);

    let mut entry_instructions = Vec::new();
    let mut explicit_terminator = None;

    for stmt in &function.body.statements {
        match &stmt.kind {
            StmtKind::Let { .. } | StmtKind::Expr { .. } => {
                if let Some(instruction) = lower_stmt_to_instruction(stmt) {
                    entry_instructions.push(instruction);
                }
            }
            StmtKind::Return { value } => {
                explicit_terminator = Some(MirTerminator::Return {
                    value: value.as_ref().map(expr_to_string),
                    span: stmt.span.clone(),
                });
                break;
            }
        }
    }

    let blocks = if let Some(terminator) = explicit_terminator {
        vec![MirBlock {
            id: 0,
            instructions: entry_instructions,
            terminator,
        }]
    } else if let Some(tail) = &function.body.tail {
        if let ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } = &tail.kind
        {
            let (then_instructions, then_terminator) = lower_block(then_branch);
            let (else_instructions, else_terminator) = lower_else_branch(else_branch, &tail.span);
            vec![
                MirBlock {
                    id: 0,
                    instructions: entry_instructions,
                    terminator: MirTerminator::Branch {
                        condition: expr_to_string(condition),
                        then_target: 1,
                        else_target: 2,
                        span: tail.span.clone(),
                    },
                },
                MirBlock {
                    id: 1,
                    instructions: then_instructions,
                    terminator: then_terminator,
                },
                MirBlock {
                    id: 2,
                    instructions: else_instructions,
                    terminator: else_terminator,
                },
            ]
        } else {
            vec![MirBlock {
                id: 0,
                instructions: entry_instructions,
                terminator: MirTerminator::Return {
                    value: Some(expr_to_string(tail)),
                    span: function.body.span.clone(),
                },
            }]
        }
    } else {
        vec![MirBlock {
            id: 0,
            instructions: entry_instructions,
            terminator: MirTerminator::Return {
                value: None,
                span: function.body.span.clone(),
            },
        }]
    };

    MirFunction {
        name: function.name.clone(),
        params,
        effects,
        return_type,
        blocks,
        span: span.clone(),
    }
}

fn lower_block(block: &Block) -> (Vec<MirInstruction>, MirTerminator) {
    let mut instructions = Vec::new();
    let mut terminator = None;

    for stmt in &block.statements {
        match &stmt.kind {
            StmtKind::Let { name, value, .. } => {
                instructions.push(MirInstruction {
                    kind: MirInstructionKind::Let {
                        name: name.clone(),
                        value: expr_to_string(value),
                    },
                    span: stmt.span.clone(),
                });
            }
            StmtKind::Expr { expr } => {
                instructions.push(MirInstruction {
                    kind: MirInstructionKind::Eval {
                        value: expr_to_string(expr),
                    },
                    span: stmt.span.clone(),
                });
            }
            StmtKind::Return { value } => {
                terminator = Some(MirTerminator::Return {
                    value: value.as_ref().map(expr_to_string),
                    span: stmt.span.clone(),
                });
                break;
            }
        }
    }

    let terminator = terminator.unwrap_or_else(|| MirTerminator::Return {
        value: block.tail.as_ref().map(|expr| expr_to_string(expr)),
        span: block.span.clone(),
    });

    (instructions, terminator)
}

fn lower_else_branch(
    else_expr: &Option<Box<Expr>>,
    fallback_span: &Span,
) -> (Vec<MirInstruction>, MirTerminator) {
    match else_expr {
        Some(expr) => match &expr.kind {
            ExprKind::Block(block) => lower_block(block),
            _ => (
                Vec::new(),
                MirTerminator::Return {
                    value: Some(expr_to_string(expr)),
                    span: expr.span.clone(),
                },
            ),
        },
        None => (
            Vec::new(),
            MirTerminator::Return {
                value: None,
                span: fallback_span.clone(),
            },
        ),
    }
}

fn lower_stmt_to_instruction(stmt: &ast::Stmt) -> Option<MirInstruction> {
    match &stmt.kind {
        StmtKind::Let { name, value, .. } => Some(MirInstruction {
            kind: MirInstructionKind::Let {
                name: name.clone(),
                value: expr_to_string(value),
            },
            span: stmt.span.clone(),
        }),
        StmtKind::Expr { expr } => Some(MirInstruction {
            kind: MirInstructionKind::Eval {
                value: expr_to_string(expr),
            },
            span: stmt.span.clone(),
        }),
        StmtKind::Return { .. } => None,
    }
}

fn render_function(out: &mut String, function: &MirFunction) {
    let params = function
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.ty))
        .collect::<Vec<_>>()
        .join(", ");

    write!(out, "fn {}({})", function.name, params).expect("write to string must succeed");

    if !function.effects.is_empty() {
        write!(out, " effects {{{}}}", function.effects.join(", "))
            .expect("write to string must succeed");
    }

    if let Some(return_type) = &function.return_type {
        write!(out, " -> {}", return_type).expect("write to string must succeed");
    }
    out.push('\n');

    for block in &function.blocks {
        writeln!(out, "  bb{}:", block.id).expect("write to string must succeed");

        for instruction in &block.instructions {
            match &instruction.kind {
                MirInstructionKind::Let { name, value } => {
                    writeln!(out, "    let {} = {}", name, value)
                        .expect("write to string must succeed");
                }
                MirInstructionKind::Eval { value } => {
                    writeln!(out, "    eval {}", value).expect("write to string must succeed");
                }
            }
        }

        match &block.terminator {
            MirTerminator::Return { value, .. } => {
                if let Some(value) = value {
                    writeln!(out, "    return {}", value).expect("write to string must succeed");
                } else {
                    writeln!(out, "    return").expect("write to string must succeed");
                }
            }
            MirTerminator::Goto { target, .. } => {
                writeln!(out, "    goto bb{}", target).expect("write to string must succeed");
            }
            MirTerminator::Branch {
                condition,
                then_target,
                else_target,
                ..
            } => {
                writeln!(
                    out,
                    "    branch {} ? bb{} : bb{}",
                    condition, then_target, else_target
                )
                .expect("write to string must succeed");
            }
        }
    }
}

fn type_expr_to_string(ty: &TypeExpr) -> String {
    match &ty.kind {
        ast::TypeExprKind::Named { name, args } => {
            if args.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}<{}>",
                    name,
                    args.iter()
                        .map(type_expr_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

fn expr_to_string(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Identifier(name) => name.clone(),
        ExprKind::Number(value) => value.clone(),
        ExprKind::String(value) => format!("{value:?}"),
        ExprKind::Bool(value) => value.to_string(),
        ExprKind::Unary { op, expr } => {
            format!("({}{})", unary_op_to_string(*op), expr_to_string(expr))
        }
        ExprKind::Binary { op, left, right } => format!(
            "({} {} {})",
            expr_to_string(left),
            binary_op_to_string(*op),
            expr_to_string(right)
        ),
        ExprKind::Member { object, field } => format!("{}.{}", expr_to_string(object), field),
        ExprKind::Call { callee, args } => format!(
            "{}({})",
            expr_to_string(callee),
            args.iter()
                .map(expr_to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let mut text = format!(
                "if {} {}",
                expr_to_string(condition),
                block_inline_to_string(then_branch)
            );
            if let Some(else_branch) = else_branch {
                write!(text, " else {}", expr_to_string(else_branch))
                    .expect("write to string must succeed");
            }
            text
        }
        ExprKind::Match { scrutinee, arms } => {
            let rendered_arms = arms
                .iter()
                .map(|arm| {
                    format!(
                        "{} => {}",
                        pattern_to_string(&arm.pattern),
                        expr_to_string(&arm.value)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "match {} {{ {} }}",
                expr_to_string(scrutinee),
                rendered_arms
            )
        }
        ExprKind::Block(block) => block_inline_to_string(block),
    }
}

fn block_inline_to_string(block: &Block) -> String {
    let mut parts = Vec::new();

    for stmt in &block.statements {
        match &stmt.kind {
            StmtKind::Let { name, value, .. } => {
                parts.push(format!("let {} = {}", name, expr_to_string(value)));
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    parts.push(format!("return {}", expr_to_string(value)));
                } else {
                    parts.push("return".to_string());
                }
            }
            StmtKind::Expr { expr } => {
                parts.push(expr_to_string(expr));
            }
        }
    }

    if let Some(tail) = &block.tail {
        parts.push(expr_to_string(tail));
    }

    format!("{{ {} }}", parts.join("; "))
}

fn pattern_to_string(pattern: &Pattern) -> String {
    match &pattern.kind {
        PatternKind::Wildcard => "_".to_string(),
        PatternKind::Identifier(name) => name.clone(),
        PatternKind::Variant { name, args } => {
            if args.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}({})",
                    name,
                    args.iter()
                        .map(pattern_to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        PatternKind::Number(value) => value.clone(),
        PatternKind::String(value) => format!("{value:?}"),
        PatternKind::Bool(value) => value.to_string(),
    }
}

fn unary_op_to_string(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
    }
}

fn binary_op_to_string(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Rem => "%",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
    }
}
