use crate::ast::{Expr, ExprKind, Item, ItemKind, MatchArm, Stmt, StmtKind};
use crate::{manifest::Manifest, project::ResolvedProjectSources};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PromoteBindingReference {
    pub file: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PromoteContractViolation {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub file: PathBuf,
    pub line: usize,
}

pub fn collect_promote_binding_references(
    source_files: &[PathBuf],
    needle: &str,
) -> Result<Vec<PromoteBindingReference>, String> {
    let target = needle.trim_end_matches('.').trim().to_string();
    if target.is_empty() {
        return Ok(Vec::new());
    }

    let mut references = Vec::new();
    let mut seen = HashSet::<(PathBuf, usize, usize)>::new();
    let mut roots = HashSet::new();
    roots.insert(target);

    for source_file in source_files {
        let source = fs::read_to_string(source_file).map_err(|err| {
            format!(
                "could not read source file `{}`: {err}",
                source_file.display()
            )
        })?;

        let source_without_uses = strip_leading_use_directives(&source);

        let program = crate::parse_source(source_file, &source_without_uses).map_err(|errors| {
            let primary = errors
                .first()
                .map(|error| error.message.clone())
                .unwrap_or_else(|| "failed to parse source file".to_string());

            format!(
                "could not parse source file `{}`: {primary}",
                source_file.display()
            )
        })?;

        for item in &program.items {
            collect_references_from_item(item, source_file, &roots, &mut references, &mut seen);
        }
    }

    references.sort_by(|left, right| {
        (left.file.as_os_str(), left.line).cmp(&(right.file.as_os_str(), right.line))
    });

    Ok(references)
}

fn strip_leading_use_directives(source: &str) -> String {
    let mut lines = source.lines().collect::<Vec<_>>();

    while let Some(line) = lines.first() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            lines.remove(0);
            continue;
        }

        let is_use =
            trimmed == "use" || trimmed.starts_with("use ") || trimmed.starts_with("use\t");
        if is_use {
            lines.remove(0);
            continue;
        }

        break;
    }

    if lines.is_empty() {
        return String::new();
    }

    let mut output = lines.join("\n");
    if source.ends_with('\n') {
        output.push('\n');
    }
    output
}

pub fn collect_promote_contract_violations(
    project_root: &Path,
    manifest: &Manifest,
) -> Result<Vec<PromoteContractViolation>, Vec<crate::diagnostics::Diagnostic>> {
    let resolved = crate::resolve_project_modules(project_root, manifest)?;
    collect_promote_contract_violations_from_modules(&resolved)
}

fn collect_promote_contract_violations_from_modules(
    resolved: &ResolvedProjectSources,
) -> Result<Vec<PromoteContractViolation>, Vec<crate::diagnostics::Diagnostic>> {
    let mut issues = Vec::new();

    for module in &resolved.modules {
        collect_contract_violations_from_imports(module, &mut issues);
    }

    let (browser_methods, server_methods) = collect_repo_adapter_methods(&resolved.modules)?;
    issues.extend(collect_repo_interface_parity_issues(
        &browser_methods,
        &server_methods,
    ));

    issues.sort_by(|left, right| {
        (
            &left.code,
            &left.file,
            left.line,
            &left.message,
            &left.severity,
        )
            .cmp(&(
                &right.code,
                &right.file,
                right.line,
                &right.message,
                &right.severity,
            ))
    });

    Ok(issues)
}

fn collect_contract_violations_from_imports(
    module: &crate::project::ResolvedModuleSource,
    issues: &mut Vec<PromoteContractViolation>,
) {
    if module.module_path == "main" {
        return;
    }

    for import in &module.imports {
        if import.module_path.starts_with("repo.") {
            issues.push(PromoteContractViolation {
                code: "PROMOTE.P9401".to_string(),
                severity: "error".to_string(),
                message: "repo adapter imports must only appear in composition root `main.ut` for promotion readiness".to_string(),
                file: module.file_path.clone(),
                line: import.span.start_line,
            });
        }
    }
}

fn collect_repo_adapter_methods(
    modules: &[crate::project::ResolvedModuleSource],
) -> Result<
    (
        BTreeMap<String, (PathBuf, usize)>,
        BTreeMap<String, (PathBuf, usize)>,
    ),
    Vec<crate::diagnostics::Diagnostic>,
> {
    let mut browser_methods: BTreeMap<String, (PathBuf, usize)> = BTreeMap::new();
    let mut server_methods: BTreeMap<String, (PathBuf, usize)> = BTreeMap::new();

    for module in modules {
        if !module.module_path.starts_with("repo.") {
            continue;
        }

        let program = crate::parse_source(&module.file_path, &module.source_without_uses).map_err(
            |errors| {
                errors
                    .into_iter()
                    .map(|diagnostic| {
                        diagnostic.with_note(format!(
                            "failed to parse module for promote contract analyzer: {}",
                            module.module_path
                        ))
                    })
                    .collect::<Vec<_>>()
            },
        )?;

        for item in program.items {
            if let ItemKind::Function(function) = item.kind {
                let function_name = function.name;
                if let Some(stripped) = function_name.strip_prefix("browser_") {
                    browser_methods.insert(
                        stripped.to_string(),
                        (module.file_path.clone(), item.span.start_line),
                    );
                } else if let Some(stripped) = function_name.strip_prefix("server_") {
                    server_methods.insert(
                        stripped.to_string(),
                        (module.file_path.clone(), item.span.start_line),
                    );
                }
            }
        }
    }

    Ok((browser_methods, server_methods))
}

fn collect_repo_interface_parity_issues(
    browser_methods: &BTreeMap<String, (PathBuf, usize)>,
    server_methods: &BTreeMap<String, (PathBuf, usize)>,
) -> Vec<PromoteContractViolation> {
    let mut issues = Vec::new();

    let browser_only = browser_methods
        .keys()
        .filter(|method| !server_methods.contains_key(*method))
        .cloned()
        .collect::<Vec<_>>();
    let server_only = server_methods
        .keys()
        .filter(|method| !browser_methods.contains_key(*method))
        .cloned()
        .collect::<Vec<_>>();

    if browser_methods.is_empty() || server_methods.is_empty() {
        return issues;
    }

    for method in browser_only {
        let (_, line) = browser_methods
            .get(method.as_str())
            .expect("browser-only method should be present");
        issues.push(PromoteContractViolation {
            code: "PROMOTE.P9402".to_string(),
            severity: "error".to_string(),
            message: format!(
                "repo adapter parity mismatch: browser_ method `{method}` is missing server_ counterpart"
            ),
            file: browser_methods
                .get(method.as_str())
                .expect("browser method details should exist")
                .0
                .clone(),
            line: *line,
        });
    }

    for method in server_only {
        let (_, line) = server_methods
            .get(method.as_str())
            .expect("server-only method should be present");
        issues.push(PromoteContractViolation {
            code: "PROMOTE.P9403".to_string(),
            severity: "error".to_string(),
            message: format!(
                "repo adapter parity mismatch: server_ method `{method}` is missing browser_ counterpart"
            ),
            file: server_methods
                .get(method.as_str())
                .expect("server method details should exist")
                .0
                .clone(),
            line: *line,
        });
    }

    issues
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
        ExprKind::Match { scrutinee, arms } => {
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
