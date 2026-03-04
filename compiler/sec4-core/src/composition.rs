use crate::ast::{Expr, ExprKind, Item, ItemKind, MatchArm, Stmt, StmtKind};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PromoteBindingReference {
    pub file: PathBuf,
    pub line: usize,
}

pub fn collect_promote_binding_references(
    source_files: &[PathBuf],
    needle: &str,
) -> Vec<PromoteBindingReference> {
    let target = needle.trim_end_matches('.').trim().to_string();
    if target.is_empty() {
        return Vec::new();
    }

    let mut references = Vec::new();
    let mut seen = HashSet::<(PathBuf, usize, usize)>::new();
    let mut roots = HashSet::new();
    roots.insert(target);

    for source_file in source_files {
        let source = match std::fs::read_to_string(source_file) {
            Ok(source) => source,
            Err(_) => continue,
        };

        let program = match crate::parse_source(source_file, &source) {
            Ok(program) => program,
            Err(_) => continue,
        };

        for item in &program.items {
            collect_references_from_item(item, source_file, &roots, &mut references, &mut seen);
        }
    }

    references.sort_by(|left, right| {
        (
            &left.file,
            left.line,
            left.file.to_string_lossy().as_ref(),
        )
            .cmp(&(
                &right.file,
                right.line,
                right.file.to_string_lossy().as_ref(),
            ))
    });

    references
}

fn collect_references_from_item(
    item: &Item,
    file: &PathBuf,
    roots: &HashSet<String>,
    references: &mut Vec<PromoteBindingReference>,
    seen: &mut HashSet<(PathBuf, usize, usize)>,
) {
    match &item.kind {
        ItemKind::Function(function) => {
            collect_references_from_block(&function.body, file, roots, references, seen);
        }
        _ => {}
    }
}

fn collect_references_from_block(
    block: &crate::ast::Block,
    file: &PathBuf,
    roots: &HashSet<String>,
    references: &mut Vec<PromoteBindingReference>,
    seen: &mut HashSet<(PathBuf, usize, usize)>,
) {
    for statement in &block.statements {
        collect_references_from_statement(statement, file, roots, references, seen);
    }

    if let Some(tail) = block.tail.as_deref() {
        collect_references_from_expr(tail, file, roots, references, seen);
    }
}

fn collect_references_from_statement(
    statement: &Stmt,
    file: &PathBuf,
    roots: &HashSet<String>,
    references: &mut Vec<PromoteBindingReference>,
    seen: &mut HashSet<(PathBuf, usize, usize)>,
) {
    match &statement.kind {
        StmtKind::Let { value, .. } => {
            collect_references_from_expr(value, file, roots, references, seen);
        }
        StmtKind::Return { value } => {
            if let Some(value) = value {
                collect_references_from_expr(value, file, roots, references, seen);
            }
        }
        StmtKind::Expr { expr } => {
            collect_references_from_expr(expr, file, roots, references, seen);
        }
    }
}

fn collect_references_from_expr(
    expr: &Expr,
    file: &PathBuf,
    roots: &HashSet<String>,
    references: &mut Vec<PromoteBindingReference>,
    seen: &mut HashSet<(PathBuf, usize, usize)>,
) {
    if is_root_member_expression(expr, roots)
        && seen.insert((file.clone(), expr.span.start_line, expr.span.start_col))
    {
        references.push(PromoteBindingReference {
            file: file.clone(),
            line: expr.span.start_line,
        });
    }

    match &expr.kind {
        ExprKind::Unary { expr, .. } => {
            collect_references_from_expr(expr, file, roots, references, seen);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_references_from_expr(left, file, roots, references, seen);
            collect_references_from_expr(right, file, roots, references, seen);
        }
        ExprKind::Call { callee, args } => {
            collect_references_from_expr(callee, file, roots, references, seen);
            for arg in args {
                collect_references_from_expr(arg, file, roots, references, seen);
            }
        }
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_references_from_expr(condition, file, roots, references, seen);
            collect_references_from_block(then_branch, file, roots, references, seen);
            if let Some(else_expr) = else_branch {
                collect_references_from_expr(else_expr, file, roots, references, seen);
            }
        }
        ExprKind::Match {
            scrutinee,
            arms,
        } => {
            collect_references_from_expr(scrutinee, file, roots, references, seen);
            for arm in arms {
                collect_references_from_match_arm(arm, file, roots, references, seen);
            }
        }
        ExprKind::Block(block) => {
            collect_references_from_block(block, file, roots, references, seen);
        }
        ExprKind::Member { .. }
        | ExprKind::Identifier(_)
        | ExprKind::Number(_)
        | ExprKind::String(_)
        | ExprKind::Bool(_) => {}
    }
}

fn collect_references_from_match_arm(
    arm: &MatchArm,
    file: &PathBuf,
    roots: &HashSet<String>,
    references: &mut Vec<PromoteBindingReference>,
    seen: &mut HashSet<(PathBuf, usize, usize)>,
) {
    collect_references_from_expr(&arm.value, file, roots, references, seen);
}

fn is_root_member_expression(expr: &Expr, roots: &HashSet<String>) -> bool {
    let mut current = expr;

    loop {
        match &current.kind {
            ExprKind::Member { object, .. } => {
                current = object;
            }
            ExprKind::Identifier(name) => return roots.contains(name),
            _ => return false,
        }
    }
}
