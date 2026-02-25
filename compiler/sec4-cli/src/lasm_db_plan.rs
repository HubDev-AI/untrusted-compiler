use std::collections::{BTreeMap, HashMap, HashSet};

use crate::{
    is_lasm_schema_row_call, is_lasm_validate_header_value_call, is_lasm_validate_int64_call,
    is_lasm_validate_non_empty_call, parse_lasm_request_text_placeholder,
    parse_lasm_res_text_template, resolve_response_expr, resolve_route_registration_expr,
    LASM_INTERNAL_DB_HANDLE_HEADER, LASM_INTERNAL_DB_OP_HEADER, LASM_INTERNAL_DB_PARAMS_HEADER,
    LASM_INTERNAL_DB_ROW_SCHEMA_HEADER, LASM_INTERNAL_DB_TEMPLATE_HEADER,
    LASM_INTERNAL_DB_TX_DB_HEADER, LASM_INTERNAL_DB_TX_HEADER,
};

#[derive(Debug, Clone)]
pub(crate) struct LasmSqlQueryPlan {
    pub(crate) template: String,
    pub(crate) params: String,
}

#[derive(Debug, Clone)]
pub(crate) enum LasmDbTxPlan {
    FromDb { db: String },
    Handle { tx: String },
}

#[derive(Debug, Clone)]
pub(crate) enum LasmDbOperationPlan {
    Exec {
        db: String,
        query: LasmSqlQueryPlan,
    },
    ExecTx {
        tx: LasmDbTxPlan,
        query: LasmSqlQueryPlan,
    },
    QueryOne {
        db: String,
        row_schema: String,
        query: LasmSqlQueryPlan,
    },
}

pub(crate) fn apply_lasm_db_operation_plan_headers(
    headers: &mut BTreeMap<String, String>,
    operation: &LasmDbOperationPlan,
) {
    match operation {
        LasmDbOperationPlan::Exec { db, query } => {
            headers.insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "exec".to_string());
            headers.insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), db.clone());
            headers.insert(
                LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
                query.template.clone(),
            );
            headers.insert(
                LASM_INTERNAL_DB_PARAMS_HEADER.to_string(),
                query.params.clone(),
            );
            headers.remove(LASM_INTERNAL_DB_TX_HEADER);
            headers.remove(LASM_INTERNAL_DB_TX_DB_HEADER);
            headers.remove(LASM_INTERNAL_DB_ROW_SCHEMA_HEADER);
        }
        LasmDbOperationPlan::ExecTx { tx, query } => {
            headers.insert(LASM_INTERNAL_DB_OP_HEADER.to_string(), "execTx".to_string());
            headers.insert(
                LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
                query.template.clone(),
            );
            headers.insert(
                LASM_INTERNAL_DB_PARAMS_HEADER.to_string(),
                query.params.clone(),
            );
            match tx {
                LasmDbTxPlan::FromDb { db } => {
                    headers.insert(LASM_INTERNAL_DB_TX_DB_HEADER.to_string(), db.clone());
                    headers.remove(LASM_INTERNAL_DB_TX_HEADER);
                }
                LasmDbTxPlan::Handle { tx } => {
                    headers.insert(LASM_INTERNAL_DB_TX_HEADER.to_string(), tx.clone());
                    headers.remove(LASM_INTERNAL_DB_TX_DB_HEADER);
                }
            }
            headers.remove(LASM_INTERNAL_DB_HANDLE_HEADER);
            headers.remove(LASM_INTERNAL_DB_ROW_SCHEMA_HEADER);
        }
        LasmDbOperationPlan::QueryOne {
            db,
            row_schema,
            query,
        } => {
            headers.insert(
                LASM_INTERNAL_DB_OP_HEADER.to_string(),
                "queryOne".to_string(),
            );
            headers.insert(LASM_INTERNAL_DB_HANDLE_HEADER.to_string(), db.clone());
            headers.insert(
                LASM_INTERNAL_DB_TEMPLATE_HEADER.to_string(),
                query.template.clone(),
            );
            headers.insert(
                LASM_INTERNAL_DB_PARAMS_HEADER.to_string(),
                query.params.clone(),
            );
            headers.insert(
                LASM_INTERNAL_DB_ROW_SCHEMA_HEADER.to_string(),
                row_schema.clone(),
            );
            headers.remove(LASM_INTERNAL_DB_TX_HEADER);
            headers.remove(LASM_INTERNAL_DB_TX_DB_HEADER);
        }
    }
}

pub(crate) fn extract_lasm_db_operation_with_count(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
) -> (Option<LasmDbOperationPlan>, usize) {
    let mut visited = HashSet::new();
    let mut operation_count = 0usize;
    let operation = extract_lasm_db_operation_in_function(
        functions,
        function_name,
        &mut visited,
        &mut operation_count,
        None,
    );
    (operation, operation_count)
}

fn extract_lasm_db_operation_in_function(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    visited: &mut HashSet<String>,
    operation_count: &mut usize,
    seed_bindings: Option<HashMap<String, sec4_core::ast::Expr>>,
) -> Option<LasmDbOperationPlan> {
    if visited.contains(function_name) {
        return None;
    }
    visited.insert(function_name.to_string());
    let Some(function) = functions.get(function_name) else {
        visited.remove(function_name);
        return None;
    };
    let mut bindings = seed_bindings.unwrap_or_default();
    let operation = extract_lasm_db_operation_in_block(
        functions,
        &function.body,
        visited,
        operation_count,
        &mut bindings,
    );
    visited.remove(function_name);
    operation
}

fn extract_lasm_db_operation_in_block(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    block: &sec4_core::ast::Block,
    visited: &mut HashSet<String>,
    operation_count: &mut usize,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmDbOperationPlan> {
    let mut operation = None;
    for statement in &block.statements {
        if let Some(next) = extract_lasm_db_operation_in_stmt(
            functions,
            statement,
            visited,
            operation_count,
            bindings,
        ) {
            operation = Some(next);
        }
    }
    if let Some(tail) = &block.tail {
        if let Some(next) =
            extract_lasm_db_operation_in_expr(functions, tail, visited, operation_count, bindings)
        {
            operation = Some(next);
        }
    }
    operation
}

fn extract_lasm_db_operation_in_stmt(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    statement: &sec4_core::ast::Stmt,
    visited: &mut HashSet<String>,
    operation_count: &mut usize,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmDbOperationPlan> {
    match &statement.kind {
        sec4_core::ast::StmtKind::Let { name, value, .. } => {
            let operation = extract_lasm_db_operation_in_expr(
                functions,
                value,
                visited,
                operation_count,
                bindings,
            );
            bindings.insert(name.clone(), value.clone());
            operation
        }
        sec4_core::ast::StmtKind::Return { value } => value.as_ref().and_then(|entry| {
            extract_lasm_db_operation_in_expr(functions, entry, visited, operation_count, bindings)
        }),
        sec4_core::ast::StmtKind::Expr { expr } => {
            extract_lasm_db_operation_in_expr(functions, expr, visited, operation_count, bindings)
        }
    }
}

fn extract_lasm_db_operation_in_expr(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    expr: &sec4_core::ast::Expr,
    visited: &mut HashSet<String>,
    operation_count: &mut usize,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmDbOperationPlan> {
    match &expr.kind {
        sec4_core::ast::ExprKind::Call { callee, args } => {
            let mut operation = None;
            let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
                .unwrap_or_else(|| callee.as_ref().clone());
            if let Some(next) = extract_lasm_db_operation_in_expr(
                functions,
                callee,
                visited,
                operation_count,
                bindings,
            ) {
                operation = Some(next);
            }
            for argument in args {
                if let Some(next) = extract_lasm_db_operation_in_expr(
                    functions,
                    argument,
                    visited,
                    operation_count,
                    bindings,
                ) {
                    operation = Some(next);
                }
            }
            if let Some(next) = match_lasm_db_operation_call(&resolved_callee, args, bindings) {
                *operation_count = operation_count.saturating_add(1);
                operation = Some(next);
            }
            if let sec4_core::ast::ExprKind::Identifier(function_name) = &resolved_callee.kind {
                let call_bindings = collect_lasm_db_operation_call_bindings(
                    functions,
                    function_name,
                    args,
                    bindings,
                );
                if let Some(next) = extract_lasm_db_operation_in_function(
                    functions,
                    function_name,
                    visited,
                    operation_count,
                    Some(call_bindings),
                ) {
                    operation = Some(next);
                }
            }
            operation
        }
        sec4_core::ast::ExprKind::Unary { expr, .. } => {
            extract_lasm_db_operation_in_expr(functions, expr, visited, operation_count, bindings)
        }
        sec4_core::ast::ExprKind::Binary { left, right, .. } => {
            let mut operation = extract_lasm_db_operation_in_expr(
                functions,
                left,
                visited,
                operation_count,
                bindings,
            );
            if let Some(next) = extract_lasm_db_operation_in_expr(
                functions,
                right,
                visited,
                operation_count,
                bindings,
            ) {
                operation = Some(next);
            }
            operation
        }
        sec4_core::ast::ExprKind::Member { object, .. } => {
            extract_lasm_db_operation_in_expr(functions, object, visited, operation_count, bindings)
        }
        sec4_core::ast::ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => extract_lasm_db_operation_in_expr(
            functions,
            condition,
            visited,
            operation_count,
            bindings,
        )
        .or_else(|| {
            let mut then_bindings = bindings.clone();
            extract_lasm_db_operation_in_block(
                functions,
                then_branch,
                visited,
                operation_count,
                &mut then_bindings,
            )
        })
        .or_else(|| {
            let mut else_bindings = bindings.clone();
            else_branch.as_ref().and_then(|entry| {
                extract_lasm_db_operation_in_expr(
                    functions,
                    entry,
                    visited,
                    operation_count,
                    &mut else_bindings,
                )
            })
        }),
        sec4_core::ast::ExprKind::Match { scrutinee, arms } => {
            if let Some(operation) = extract_lasm_db_operation_in_expr(
                functions,
                scrutinee,
                visited,
                operation_count,
                bindings,
            ) {
                return Some(operation);
            }
            for arm in arms {
                let mut arm_bindings = bindings.clone();
                if let Some(operation) = extract_lasm_db_operation_in_expr(
                    functions,
                    &arm.value,
                    visited,
                    operation_count,
                    &mut arm_bindings,
                ) {
                    return Some(operation);
                }
            }
            None
        }
        sec4_core::ast::ExprKind::Block(block) => {
            let mut block_bindings = bindings.clone();
            extract_lasm_db_operation_in_block(
                functions,
                block,
                visited,
                operation_count,
                &mut block_bindings,
            )
        }
        sec4_core::ast::ExprKind::Identifier(_)
        | sec4_core::ast::ExprKind::Number(_)
        | sec4_core::ast::ExprKind::String(_)
        | sec4_core::ast::ExprKind::Bool(_) => None,
    }
}

fn collect_lasm_db_operation_call_bindings(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> HashMap<String, sec4_core::ast::Expr> {
    let mut call_bindings = HashMap::new();
    let Some(function) = functions.get(function_name) else {
        return call_bindings;
    };
    for (param, arg) in function.params.iter().zip(args.iter()) {
        let resolved =
            resolve_route_registration_expr(arg, bindings, 0).unwrap_or_else(|| arg.clone());
        call_bindings.insert(param.name.clone(), resolved);
    }
    call_bindings
}

fn match_lasm_db_operation_call(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmDbOperationPlan> {
    if is_lasm_db_exec_call(callee) {
        let (db_index, query_index) = match args.len() {
            2 => (0, 1),
            3 => (1, 2),
            _ => return None,
        };
        let db = extract_lasm_db_value_template(&args[db_index], bindings, 0)
            .unwrap_or_else(|| "1".to_string());
        let query = extract_lasm_sql_query_plan(&args[query_index], bindings, 0)?;
        return Some(LasmDbOperationPlan::Exec { db, query });
    }
    if is_lasm_db_exec_tx_call(callee) {
        let (tx_index, query_index) = match args.len() {
            2 => (0, 1),
            3 => (1, 2),
            _ => return None,
        };
        let tx = extract_lasm_db_tx_plan(&args[tx_index], bindings, 0)?;
        let query = extract_lasm_sql_query_plan(&args[query_index], bindings, 0)?;
        return Some(LasmDbOperationPlan::ExecTx { tx, query });
    }
    if is_lasm_db_query_one_call(callee) {
        let (db_index, query_index, row_schema_index) = match args.len() {
            3 => (0, 1, 2),
            4 => (1, 2, 3),
            _ => return None,
        };
        let db = extract_lasm_db_value_template(&args[db_index], bindings, 0)
            .unwrap_or_else(|| "1".to_string());
        let row_schema = extract_lasm_db_value_template(&args[row_schema_index], bindings, 0)
            .unwrap_or_else(|| "1".to_string());
        let query = extract_lasm_sql_query_plan(&args[query_index], bindings, 0)?;
        return Some(LasmDbOperationPlan::QueryOne {
            db,
            row_schema,
            query,
        });
    }
    None
}

fn extract_lasm_db_tx_plan(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<LasmDbTxPlan> {
    if depth > 32 {
        return None;
    }
    let resolved = resolve_response_expr(expr, bindings, depth)?;
    if let sec4_core::ast::ExprKind::Call { callee, args } = &resolved.kind {
        let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
            .unwrap_or_else(|| callee.as_ref().clone());
        if is_lasm_db_tx_call(&resolved_callee) {
            let db_index = match args.len() {
                1 => 0,
                2 => 1,
                _ => return None,
            };
            let db = extract_lasm_db_value_template(&args[db_index], bindings, depth + 1)
                .unwrap_or_else(|| "1".to_string());
            return Some(LasmDbTxPlan::FromDb { db });
        }
    }
    let tx = extract_lasm_db_value_template(&resolved, bindings, depth + 1)?;
    Some(LasmDbTxPlan::Handle { tx })
}

fn extract_lasm_sql_query_plan(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<LasmSqlQueryPlan> {
    if depth > 32 {
        return None;
    }
    let resolved = resolve_response_expr(expr, bindings, depth)?;
    let sec4_core::ast::ExprKind::Call { callee, args } = &resolved.kind else {
        return None;
    };
    let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
        .unwrap_or_else(|| callee.as_ref().clone());
    if !is_lasm_sql_q_call(&resolved_callee) || args.len() < 2 {
        return None;
    }
    let template = extract_lasm_db_value_template(&args[0], bindings, depth + 1)?;
    if template.trim().is_empty() {
        return None;
    }
    let params = extract_lasm_db_value_template(&args[1], bindings, depth + 1)
        .unwrap_or_else(|| "0".to_string());
    Some(LasmSqlQueryPlan { template, params })
}

fn extract_lasm_db_value_template(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<String> {
    if depth > 32 {
        return None;
    }
    let resolved = resolve_response_expr(expr, bindings, depth)?;
    match &resolved.kind {
        sec4_core::ast::ExprKind::String(value) => Some(value.clone()),
        sec4_core::ast::ExprKind::Number(value) => Some(value.clone()),
        sec4_core::ast::ExprKind::Bool(value) => Some(if *value {
            "1".to_string()
        } else {
            "0".to_string()
        }),
        sec4_core::ast::ExprKind::Call { callee, args } => {
            let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
                .unwrap_or_else(|| callee.as_ref().clone());
            if let Some(placeholder) =
                parse_lasm_request_text_placeholder(&resolved_callee, args, bindings)
            {
                return Some(placeholder);
            }
            if is_lasm_validate_non_empty_call(&resolved_callee)
                || is_lasm_validate_header_value_call(&resolved_callee)
                || is_lasm_validate_int64_call(&resolved_callee)
                || is_lasm_schema_row_call(&resolved_callee)
            {
                return args
                    .first()
                    .and_then(|value| extract_lasm_db_value_template(value, bindings, depth + 1));
            }
            if is_lasm_headers_name_call(&resolved_callee)
                || is_lasm_headers_value_call(&resolved_callee)
            {
                return args
                    .first()
                    .and_then(|value| extract_lasm_db_value_template(value, bindings, depth + 1));
            }
            if is_lasm_db_cap_constructor_call(&resolved_callee) {
                return Some("1".to_string());
            }
            parse_lasm_res_text_template(&resolved, bindings, depth + 1)
        }
        sec4_core::ast::ExprKind::Binary { op, left, right }
            if *op == sec4_core::ast::BinaryOp::Add =>
        {
            let lhs = extract_lasm_db_value_template(left, bindings, depth + 1)?;
            let rhs = extract_lasm_db_value_template(right, bindings, depth + 1)?;
            Some(format!("{lhs}{rhs}"))
        }
        _ => parse_lasm_res_text_template(&resolved, bindings, depth + 1),
    }
}

fn is_lasm_sql_q_call(callee: &sec4_core::ast::Expr) -> bool {
    match &callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => matches!(name.as_str(), "sql_q"),
        sec4_core::ast::ExprKind::Member { object, field } => {
            matches!(
                (&object.kind, field.as_str()),
                (sec4_core::ast::ExprKind::Identifier(name), "q") if name == "sql"
            )
        }
        _ => false,
    }
}

fn is_lasm_db_exec_call(callee: &sec4_core::ast::Expr) -> bool {
    match &callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => {
            matches!(name.as_str(), "db_exec" | "db_write")
        }
        sec4_core::ast::ExprKind::Member { object, field } => {
            matches!(
                (&object.kind, field.as_str()),
                (sec4_core::ast::ExprKind::Identifier(name), "exec") if name == "db"
            )
        }
        _ => false,
    }
}

fn is_lasm_db_tx_call(callee: &sec4_core::ast::Expr) -> bool {
    match &callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => matches!(name.as_str(), "db_tx"),
        sec4_core::ast::ExprKind::Member { object, field } => {
            matches!(
                (&object.kind, field.as_str()),
                (sec4_core::ast::ExprKind::Identifier(name), "tx") if name == "db"
            )
        }
        _ => false,
    }
}

fn is_lasm_db_exec_tx_call(callee: &sec4_core::ast::Expr) -> bool {
    match &callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => {
            matches!(name.as_str(), "db_exec_tx")
        }
        sec4_core::ast::ExprKind::Member { object, field } => {
            matches!(
                (&object.kind, field.as_str()),
                (sec4_core::ast::ExprKind::Identifier(name), "execTx") if name == "db"
            )
        }
        _ => false,
    }
}

fn is_lasm_db_query_one_call(callee: &sec4_core::ast::Expr) -> bool {
    match &callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => matches!(name.as_str(), "db_read"),
        sec4_core::ast::ExprKind::Member { object, field } => {
            matches!(
                (&object.kind, field.as_str()),
                (sec4_core::ast::ExprKind::Identifier(name), "queryOne") if name == "db"
            )
        }
        _ => false,
    }
}

fn is_lasm_db_cap_constructor_call(callee: &sec4_core::ast::Expr) -> bool {
    matches!(callee.kind, sec4_core::ast::ExprKind::Identifier(ref name) if name == "DbCap")
}

fn is_lasm_headers_name_call(callee: &sec4_core::ast::Expr) -> bool {
    matches!(
        &callee.kind,
        sec4_core::ast::ExprKind::Member { object, field }
            if field == "name"
                && matches!(
                    object.kind,
                    sec4_core::ast::ExprKind::Identifier(ref namespace) if namespace == "headers"
                )
    )
}

fn is_lasm_headers_value_call(callee: &sec4_core::ast::Expr) -> bool {
    matches!(
        &callee.kind,
        sec4_core::ast::ExprKind::Member { object, field }
            if field == "value"
                && matches!(
                    object.kind,
                    sec4_core::ast::ExprKind::Identifier(ref namespace) if namespace == "headers"
                )
    )
}
