use ailang_core::ast::{Block, Expr, ExprKind, ItemKind, Program, Stmt, StmtKind, TypeExpr, TypeExprKind};
use ailang_core::{analyze_program, parse_source, Diagnostic as CoreDiagnostic, Severity as CoreSeverity, Span};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use url::Url;

const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_REQUEST: i64 = -32600;

#[derive(Default)]
struct ServerState {
    documents: HashMap<String, String>,
}

#[derive(Clone)]
struct FunctionSymbol {
    name: String,
    span: Span,
    signature: String,
}

#[derive(Clone)]
struct IdentifierHit {
    name: String,
    span: Span,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let first = args.next();
    match first.as_deref() {
        Some("--version") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
        }
        Some("--stdio") => {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let mut reader = BufReader::new(stdin.lock());
            let mut writer = stdout.lock();
            if let Err(err) = run_stdio(&mut reader, &mut writer) {
                eprintln!("ailang-language-server: {err}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("usage: ailang-language-server [--stdio|--version]");
            std::process::exit(2);
        }
    }
}

fn run_stdio<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    let mut state = ServerState::default();
    loop {
        let Some(message) = read_message(reader)? else {
            return Ok(());
        };
        if handle_message(&message, &mut state, writer)? {
            return Ok(());
        }
    }
}

fn handle_message<W: Write>(
    message: &Value,
    state: &mut ServerState,
    writer: &mut W,
) -> io::Result<bool> {
    let id = message.get("id").cloned();
    let method = message.get("method").and_then(Value::as_str);

    match method {
        Some("initialize") => {
            if let Some(id) = id {
                send_response(
                    writer,
                    id,
                    json!({
                        "capabilities": {
                            "textDocumentSync": {
                                "openClose": true,
                                "change": 1
                            },
                            "definitionProvider": true,
                            "hoverProvider": true,
                            "referencesProvider": true
                        },
                        "serverInfo": {
                            "name": "ailang-language-server",
                            "version": env!("CARGO_PKG_VERSION"),
                        }
                    }),
                )?;
            }
        }
        Some("shutdown") => {
            if let Some(id) = id {
                send_response(writer, id, Value::Null)?;
            }
        }
        Some("exit") => {
            return Ok(true);
        }
        Some("textDocument/didOpen") => {
            if let Some((uri, text)) = parse_did_open(message) {
                state.documents.insert(uri.clone(), text.clone());
                publish_analysis(writer, &uri, &text)?;
            } else if let Some(id) = id {
                send_error_response(
                    writer,
                    id,
                    INVALID_REQUEST,
                    "invalid didOpen notification payload".to_string(),
                )?;
            }
        }
        Some("textDocument/didChange") => {
            if let Some((uri, text)) = parse_did_change(message) {
                state.documents.insert(uri.clone(), text.clone());
                publish_analysis(writer, &uri, &text)?;
            } else if let Some(id) = id {
                send_error_response(
                    writer,
                    id,
                    INVALID_REQUEST,
                    "invalid didChange notification payload".to_string(),
                )?;
            }
        }
        Some("textDocument/didClose") => {
            if let Some(uri) = parse_did_close(message) {
                state.documents.remove(&uri);
                publish_diagnostics(writer, &uri, Vec::new())?;
            } else if let Some(id) = id {
                send_error_response(
                    writer,
                    id,
                    INVALID_REQUEST,
                    "invalid didClose notification payload".to_string(),
                )?;
            }
        }
        Some("textDocument/definition") => {
            if let Some(id) = id {
                if let Some((uri, line, character)) = parse_text_document_position(message) {
                    match definition_at_position(state, &uri, line, character) {
                        Ok(result) => send_response(writer, id, result)?,
                        Err(err) => send_error_response(writer, id, INVALID_REQUEST, err.to_string())?,
                    }
                } else {
                    send_error_response(
                        writer,
                        id,
                        INVALID_REQUEST,
                        "invalid definition request payload".to_string(),
                    )?;
                }
            }
        }
        Some("textDocument/hover") => {
            if let Some(id) = id {
                if let Some((uri, line, character)) = parse_text_document_position(message) {
                    match hover_at_position(state, &uri, line, character) {
                        Ok(result) => send_response(writer, id, result)?,
                        Err(err) => send_error_response(writer, id, INVALID_REQUEST, err.to_string())?,
                    }
                } else {
                    send_error_response(
                        writer,
                        id,
                        INVALID_REQUEST,
                        "invalid hover request payload".to_string(),
                    )?;
                }
            }
        }
        Some("textDocument/references") => {
            if let Some(id) = id {
                if let Some((uri, line, character, include_declaration)) =
                    parse_references_position(message)
                {
                    match references_at_position(state, &uri, line, character, include_declaration)
                    {
                        Ok(result) => send_response(writer, id, result)?,
                        Err(err) => {
                            send_error_response(writer, id, INVALID_REQUEST, err.to_string())?
                        }
                    }
                } else {
                    send_error_response(
                        writer,
                        id,
                        INVALID_REQUEST,
                        "invalid references request payload".to_string(),
                    )?;
                }
            }
        }
        Some(other) => {
            if let Some(id) = id {
                send_error_response(
                    writer,
                    id,
                    METHOD_NOT_FOUND,
                    format!("method not found: {other}"),
                )?;
            }
        }
        None => {
            if let Some(id) = id {
                send_error_response(
                    writer,
                    id,
                    INVALID_REQUEST,
                    "invalid request: missing method".to_string(),
                )?;
            }
        }
    }

    Ok(false)
}

fn parse_did_open(message: &Value) -> Option<(String, String)> {
    let text_document = message.get("params")?.get("textDocument")?;
    let uri = text_document.get("uri")?.as_str()?.to_string();
    let text = text_document.get("text")?.as_str()?.to_string();
    Some((uri, text))
}

fn parse_did_change(message: &Value) -> Option<(String, String)> {
    let params = message.get("params")?;
    let uri = params.get("textDocument")?.get("uri")?.as_str()?.to_string();
    let changes = params.get("contentChanges")?.as_array()?;
    let text = changes
        .last()
        .and_then(|change| change.get("text"))
        .and_then(Value::as_str)?
        .to_string();
    Some((uri, text))
}

fn parse_did_close(message: &Value) -> Option<String> {
    message
        .get("params")?
        .get("textDocument")?
        .get("uri")?
        .as_str()
        .map(|uri| uri.to_string())
}

fn parse_text_document_position(message: &Value) -> Option<(String, usize, usize)> {
    let params = message.get("params")?;
    let uri = params
        .get("textDocument")?
        .get("uri")?
        .as_str()?
        .to_string();
    let line = params.get("position")?.get("line")?.as_u64()? as usize;
    let character = params.get("position")?.get("character")?.as_u64()? as usize;
    Some((uri, line, character))
}

fn parse_references_position(message: &Value) -> Option<(String, usize, usize, bool)> {
    let (uri, line, character) = parse_text_document_position(message)?;
    let include_declaration = message
        .get("params")
        .and_then(|params| params.get("context"))
        .and_then(|context| context.get("includeDeclaration"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Some((uri, line, character, include_declaration))
}

fn definition_at_position(
    state: &ServerState,
    uri: &str,
    line: usize,
    character: usize,
) -> io::Result<Value> {
    let (path, source) = load_document_source(state, uri)?;
    let program = match parse_source(&path, &source) {
        Ok(program) => program,
        Err(_) => return Ok(Value::Null),
    };

    let Some(hit) = find_identifier_at_position(&program, &path, line, character) else {
        return Ok(Value::Null);
    };

    let symbols = collect_function_symbols(&program);
    let Some(symbol) = symbols
        .into_iter()
        .find(|symbol| symbol.name == hit.name)
    else {
        return Ok(Value::Null);
    };

    Ok(location_from_span(uri, &symbol.span))
}

fn hover_at_position(
    state: &ServerState,
    uri: &str,
    line: usize,
    character: usize,
) -> io::Result<Value> {
    let (path, source) = load_document_source(state, uri)?;
    let program = match parse_source(&path, &source) {
        Ok(program) => program,
        Err(_) => return Ok(Value::Null),
    };

    let Some(hit) = find_identifier_at_position(&program, &path, line, character) else {
        return Ok(Value::Null);
    };

    let symbols = collect_function_symbols(&program);
    let Some(symbol) = symbols
        .into_iter()
        .find(|symbol| symbol.name == hit.name)
    else {
        return Ok(Value::Null);
    };

    Ok(json!({
        "contents": {
            "kind": "markdown",
            "value": format!("```ailang\\n{}\\n```", symbol.signature),
        },
        "range": range_from_span(&hit.span),
    }))
}

fn references_at_position(
    state: &ServerState,
    uri: &str,
    line: usize,
    character: usize,
    include_declaration: bool,
) -> io::Result<Value> {
    let (path, source) = load_document_source(state, uri)?;
    let program = match parse_source(&path, &source) {
        Ok(program) => program,
        Err(_) => return Ok(Value::Array(Vec::new())),
    };

    let Some(hit) = find_identifier_at_position(&program, &path, line, character) else {
        return Ok(Value::Array(Vec::new()));
    };

    let symbols = collect_function_symbols(&program);
    let Some(symbol) = symbols.into_iter().find(|symbol| symbol.name == hit.name) else {
        return Ok(Value::Array(Vec::new()));
    };

    let mut locations = collect_identifier_hits_by_name(&program, &path, &symbol.name)
        .into_iter()
        .map(|found| location_from_span(uri, &found.span))
        .collect::<Vec<_>>();
    if include_declaration {
        locations.insert(0, location_from_span(uri, &symbol.span));
    }
    Ok(Value::Array(locations))
}

fn load_document_source(state: &ServerState, uri: &str) -> io::Result<(PathBuf, String)> {
    let Some(path) = uri_to_path(uri) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported document URI (expected file://): {uri}"),
        ));
    };

    if let Some(text) = state.documents.get(uri) {
        return Ok((path, text.clone()));
    }

    let text = fs::read_to_string(&path)?;
    Ok((path, text))
}

fn collect_function_symbols(program: &Program) -> Vec<FunctionSymbol> {
    program
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            ItemKind::Function(function) => {
                let params = function
                    .params
                    .iter()
                    .map(|param| format!("{}: {}", param.name, format_type(&param.ty)))
                    .collect::<Vec<_>>()
                    .join(", ");

                let mut signature = format!("fn {}({})", function.name, params);
                if !function.effects.is_empty() {
                    let effects = function
                        .effects
                        .iter()
                        .map(|effect| effect.as_name())
                        .collect::<Vec<_>>()
                        .join(", ");
                    signature.push_str(&format!(" effects {{ {} }}", effects));
                }
                if let Some(return_type) = &function.return_type {
                    signature.push_str(&format!(" -> {}", format_type(return_type)));
                }

                Some(FunctionSymbol {
                    name: function.name.clone(),
                    span: item.span.clone(),
                    signature,
                })
            }
            _ => None,
        })
        .collect()
}

fn format_type(ty: &TypeExpr) -> String {
    match &ty.kind {
        TypeExprKind::Named { name, args } => {
            if args.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}<{}>",
                    name,
                    args.iter().map(format_type).collect::<Vec<_>>().join(", ")
                )
            }
        }
    }
}

fn find_identifier_at_position(
    program: &Program,
    file: &PathBuf,
    line: usize,
    character: usize,
) -> Option<IdentifierHit> {
    for item in &program.items {
        if let ItemKind::Function(function) = &item.kind {
            if let Some(hit) = find_identifier_in_block(&function.body, file, line, character) {
                return Some(hit);
            }
        }
    }
    None
}

fn find_identifier_in_block(
    block: &Block,
    file: &PathBuf,
    line: usize,
    character: usize,
) -> Option<IdentifierHit> {
    for statement in &block.statements {
        if let Some(hit) = find_identifier_in_statement(statement, file, line, character) {
            return Some(hit);
        }
    }

    block
        .tail
        .as_ref()
        .and_then(|tail| find_identifier_in_expr(tail, file, line, character))
}

fn find_identifier_in_statement(
    statement: &Stmt,
    file: &PathBuf,
    line: usize,
    character: usize,
) -> Option<IdentifierHit> {
    match &statement.kind {
        StmtKind::Let { value, .. } => find_identifier_in_expr(value, file, line, character),
        StmtKind::Return { value } => value
            .as_ref()
            .and_then(|expr| find_identifier_in_expr(expr, file, line, character)),
        StmtKind::Expr { expr } => find_identifier_in_expr(expr, file, line, character),
    }
}

fn find_identifier_in_expr(
    expr: &Expr,
    file: &PathBuf,
    line: usize,
    character: usize,
) -> Option<IdentifierHit> {
    match &expr.kind {
        ExprKind::Identifier(name) => {
            if span_contains(&expr.span, file, line, character) {
                Some(IdentifierHit {
                    name: name.clone(),
                    span: expr.span.clone(),
                })
            } else {
                None
            }
        }
        ExprKind::Unary { expr, .. } => find_identifier_in_expr(expr, file, line, character),
        ExprKind::Binary { left, right, .. } => find_identifier_in_expr(left, file, line, character)
            .or_else(|| find_identifier_in_expr(right, file, line, character)),
        ExprKind::Member { object, .. } => find_identifier_in_expr(object, file, line, character),
        ExprKind::Call { callee, args } => find_identifier_in_expr(callee, file, line, character)
            .or_else(|| {
                args.iter()
                    .find_map(|arg| find_identifier_in_expr(arg, file, line, character))
            }),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => find_identifier_in_expr(condition, file, line, character)
            .or_else(|| find_identifier_in_block(then_branch, file, line, character))
            .or_else(|| {
                else_branch
                    .as_ref()
                    .and_then(|expr| find_identifier_in_expr(expr, file, line, character))
            }),
        ExprKind::Match { scrutinee, arms } => find_identifier_in_expr(scrutinee, file, line, character)
            .or_else(|| {
                arms.iter()
                    .find_map(|arm| find_identifier_in_expr(&arm.value, file, line, character))
            }),
        ExprKind::Block(block) => find_identifier_in_block(block, file, line, character),
        ExprKind::Number(_) | ExprKind::String(_) | ExprKind::Bool(_) => None,
    }
}

fn collect_identifier_hits_by_name(
    program: &Program,
    file: &PathBuf,
    target_name: &str,
) -> Vec<IdentifierHit> {
    let mut hits = Vec::new();
    for item in &program.items {
        if let ItemKind::Function(function) = &item.kind {
            collect_identifier_hits_in_block(&function.body, file, target_name, &mut hits);
        }
    }
    hits
}

fn collect_identifier_hits_in_block(
    block: &Block,
    file: &PathBuf,
    target_name: &str,
    hits: &mut Vec<IdentifierHit>,
) {
    for statement in &block.statements {
        collect_identifier_hits_in_statement(statement, file, target_name, hits);
    }
    if let Some(tail) = &block.tail {
        collect_identifier_hits_in_expr(tail, file, target_name, hits);
    }
}

fn collect_identifier_hits_in_statement(
    statement: &Stmt,
    file: &PathBuf,
    target_name: &str,
    hits: &mut Vec<IdentifierHit>,
) {
    match &statement.kind {
        StmtKind::Let { value, .. } => collect_identifier_hits_in_expr(value, file, target_name, hits),
        StmtKind::Return { value } => {
            if let Some(expr) = value {
                collect_identifier_hits_in_expr(expr, file, target_name, hits);
            }
        }
        StmtKind::Expr { expr } => collect_identifier_hits_in_expr(expr, file, target_name, hits),
    }
}

fn collect_identifier_hits_in_expr(
    expr: &Expr,
    file: &PathBuf,
    target_name: &str,
    hits: &mut Vec<IdentifierHit>,
) {
    match &expr.kind {
        ExprKind::Identifier(name) => {
            if name == target_name && &expr.span.file == file {
                hits.push(IdentifierHit {
                    name: name.clone(),
                    span: expr.span.clone(),
                });
            }
        }
        ExprKind::Unary { expr, .. } => collect_identifier_hits_in_expr(expr, file, target_name, hits),
        ExprKind::Binary { left, right, .. } => {
            collect_identifier_hits_in_expr(left, file, target_name, hits);
            collect_identifier_hits_in_expr(right, file, target_name, hits);
        }
        ExprKind::Member { object, .. } => {
            collect_identifier_hits_in_expr(object, file, target_name, hits);
        }
        ExprKind::Call { callee, args } => {
            collect_identifier_hits_in_expr(callee, file, target_name, hits);
            for arg in args {
                collect_identifier_hits_in_expr(arg, file, target_name, hits);
            }
        }
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_identifier_hits_in_expr(condition, file, target_name, hits);
            collect_identifier_hits_in_block(then_branch, file, target_name, hits);
            if let Some(expr) = else_branch {
                collect_identifier_hits_in_expr(expr, file, target_name, hits);
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            collect_identifier_hits_in_expr(scrutinee, file, target_name, hits);
            for arm in arms {
                collect_identifier_hits_in_expr(&arm.value, file, target_name, hits);
            }
        }
        ExprKind::Block(block) => collect_identifier_hits_in_block(block, file, target_name, hits),
        ExprKind::Number(_) | ExprKind::String(_) | ExprKind::Bool(_) => {}
    }
}

fn span_contains(span: &Span, file: &PathBuf, line: usize, character: usize) -> bool {
    if &span.file != file {
        return false;
    }

    let line_1 = line.saturating_add(1);
    let col_1 = character.saturating_add(1);

    if line_1 < span.start_line || line_1 > span.end_line {
        return false;
    }
    if line_1 == span.start_line && col_1 < span.start_col {
        return false;
    }
    if line_1 == span.end_line && col_1 > span.end_col {
        return false;
    }
    true
}

fn location_from_span(uri: &str, span: &Span) -> Value {
    json!({
        "uri": uri,
        "range": range_from_span(span),
    })
}

fn range_from_span(span: &Span) -> Value {
    let start_line = span.start_line.saturating_sub(1);
    let start_col = span.start_col.saturating_sub(1);
    let mut end_line = span.end_line.saturating_sub(1);
    let mut end_col = span.end_col.saturating_sub(1);
    if end_line < start_line {
        end_line = start_line;
        end_col = start_col.saturating_add(1);
    }
    if end_line == start_line && end_col <= start_col {
        end_col = start_col.saturating_add(1);
    }

    json!({
        "start": {
            "line": start_line,
            "character": start_col,
        },
        "end": {
            "line": end_line,
            "character": end_col,
        }
    })
}

fn publish_analysis<W: Write>(writer: &mut W, uri: &str, text: &str) -> io::Result<()> {
    let diagnostics = diagnostics_for_document(uri, text);
    publish_diagnostics(writer, uri, diagnostics)
}

fn diagnostics_for_document(uri: &str, text: &str) -> Vec<Value> {
    let Some(path) = uri_to_path(uri) else {
        return vec![json!({
            "range": {
                "start": {"line": 0, "character": 0},
                "end": {"line": 0, "character": 1}
            },
            "severity": 1,
            "code": "L9001",
            "source": "ailang-lsp",
            "message": format!("unsupported document URI (expected file://): {uri}")
        })];
    };

    let diagnostics = match parse_source(&path, text) {
        Ok(program) => match analyze_program(&program) {
            Ok(()) => Vec::new(),
            Err(diags) => diags,
        },
        Err(diags) => diags,
    };

    diagnostics
        .into_iter()
        .filter(|diag| diag.span.file == path)
        .map(core_diagnostic_to_lsp)
        .collect()
}

fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let parsed = Url::parse(uri).ok()?;
    if parsed.scheme() != "file" {
        return None;
    }
    parsed.to_file_path().ok()
}

fn core_diagnostic_to_lsp(diagnostic: CoreDiagnostic) -> Value {
    let start_line = diagnostic.span.start_line.saturating_sub(1);
    let start_col = diagnostic.span.start_col.saturating_sub(1);

    let mut end_line = diagnostic.span.end_line.saturating_sub(1);
    let mut end_col = diagnostic.span.end_col.saturating_sub(1);

    if end_line < start_line {
        end_line = start_line;
        end_col = start_col.saturating_add(1);
    }
    if end_line == start_line && end_col <= start_col {
        end_col = start_col.saturating_add(1);
    }

    let message = if diagnostic.notes.is_empty() {
        diagnostic.message.clone()
    } else {
        format!("{}\n{}", diagnostic.message, diagnostic.notes.join("\n"))
    };

    json!({
        "range": {
            "start": {
                "line": start_line,
                "character": start_col,
            },
            "end": {
                "line": end_line,
                "character": end_col,
            }
        },
        "severity": severity_to_lsp(&diagnostic.severity),
        "code": diagnostic.code,
        "source": "ailang",
        "message": message,
        "data": {
            "tags": diagnostic.tags,
            "notes": diagnostic.notes,
        }
    })
}

fn severity_to_lsp(severity: &CoreSeverity) -> i64 {
    match severity {
        CoreSeverity::Error => 1,
        CoreSeverity::Warning => 2,
        CoreSeverity::Info => 3,
    }
}

fn publish_diagnostics<W: Write>(writer: &mut W, uri: &str, diagnostics: Vec<Value>) -> io::Result<()> {
    send_notification(
        writer,
        "textDocument/publishDiagnostics",
        json!({
            "uri": uri,
            "diagnostics": diagnostics,
        }),
    )
}

fn send_notification<W: Write>(writer: &mut W, method: &str, params: Value) -> io::Result<()> {
    send_message(
        writer,
        &json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }),
    )
}

fn send_response<W: Write>(writer: &mut W, id: Value, result: Value) -> io::Result<()> {
    send_message(
        writer,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        }),
    )
}

fn send_error_response<W: Write>(
    writer: &mut W,
    id: Value,
    code: i64,
    message: String,
) -> io::Result<()> {
    send_message(
        writer,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message,
            },
        }),
    )
}

fn send_message<W: Write>(writer: &mut W, payload: &Value) -> io::Result<()> {
    let body = serde_json::to_vec(payload)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}

fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<Value>> {
    let mut content_length = None;
    let mut saw_header = false;

    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            if !saw_header {
                return Ok(None);
            }
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unexpected EOF while reading LSP headers",
            ));
        }
        saw_header = true;

        if line == "\r\n" || line == "\n" {
            break;
        }

        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                let length = value.trim().parse::<usize>().map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid Content-Length header: {err}"),
                    )
                })?;
                content_length = Some(length);
            }
        }
    }

    let Some(content_length) = content_length else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing Content-Length header",
        ));
    };

    let mut body = vec![0; content_length];
    reader.read_exact(&mut body)?;
    let payload = serde_json::from_slice::<Value>(&body).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid JSON-RPC payload: {err}"),
        )
    })?;
    Ok(Some(payload))
}

#[cfg(test)]
mod tests {
    use super::{read_message, run_stdio};
    use serde_json::{json, Value};
    use std::io::{BufReader, Cursor};

    fn encode_message(value: Value) -> Vec<u8> {
        let body = serde_json::to_vec(&value).expect("payload should serialize");
        let mut bytes = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
        bytes.extend(body);
        bytes
    }

    fn collect_messages(output: Vec<u8>) -> Vec<Value> {
        let mut reader = BufReader::new(Cursor::new(output));
        let mut messages = Vec::new();
        while let Some(message) = read_message(&mut reader)
            .expect("output message should parse")
        {
            messages.push(message);
        }
        messages
    }

    #[test]
    fn run_stdio_handles_initialize_shutdown_and_exit() {
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown",
            "params": null,
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "exit",
        })));

        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        run_stdio(&mut reader, &mut output).expect("stdio loop should succeed");

        let messages = collect_messages(output);
        assert_eq!(messages.len(), 2, "initialize + shutdown responses expected");
        assert_eq!(messages[0].get("id"), Some(&json!(1)));
        assert_eq!(
            messages[0]
                .get("result")
                .and_then(|result| result.get("capabilities"))
                .and_then(|caps| caps.get("textDocumentSync"))
                .and_then(|sync| sync.get("change"))
                .and_then(Value::as_i64),
            Some(1),
            "initialize response should advertise full text sync"
        );
        assert_eq!(
            messages[0]
                .get("result")
                .and_then(|result| result.get("capabilities"))
                .and_then(|caps| caps.get("referencesProvider"))
                .and_then(Value::as_bool),
            Some(true),
            "initialize response should advertise references provider"
        );
        assert_eq!(messages[1].get("id"), Some(&json!(2)));
    }

    #[test]
    fn did_open_publishes_diagnostics() {
        let uri = "file:///tmp/lsp_did_open.ai";
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "ailang",
                    "version": 1,
                    "text": "fn main( -> Int {\n  0\n}\n"
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "exit",
        })));

        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        run_stdio(&mut reader, &mut output).expect("stdio loop should succeed");

        let messages = collect_messages(output);
        let publish = messages
            .iter()
            .find(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")))
            .expect("didOpen should publish diagnostics");

        assert_eq!(
            publish
                .get("params")
                .and_then(|params| params.get("uri"))
                .and_then(Value::as_str),
            Some(uri)
        );

        let diagnostic_count = publish
            .get("params")
            .and_then(|params| params.get("diagnostics"))
            .and_then(Value::as_array)
            .map(|diags| diags.len())
            .unwrap_or(0);
        assert!(diagnostic_count > 0, "expected parser diagnostics for invalid source");
    }

    #[test]
    fn did_change_can_clear_diagnostics() {
        let uri = "file:///tmp/lsp_did_change.ai";
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "ailang",
                    "version": 1,
                    "text": "fn main( -> Int {\n  0\n}\n"
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "version": 2
                },
                "contentChanges": [
                    {"text": "fn main() -> Int {\n  0\n}\n"}
                ]
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "exit",
        })));

        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        run_stdio(&mut reader, &mut output).expect("stdio loop should succeed");

        let messages = collect_messages(output);
        let publishes: Vec<&Value> = messages
            .iter()
            .filter(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")))
            .collect();
        assert_eq!(publishes.len(), 2, "expected diagnostics publish for open and change");

        let first_count = publishes[0]
            .get("params")
            .and_then(|params| params.get("diagnostics"))
            .and_then(Value::as_array)
            .map(|diags| diags.len())
            .unwrap_or(0);
        assert!(first_count > 0, "open should publish diagnostics for invalid source");

        let second_count = publishes[1]
            .get("params")
            .and_then(|params| params.get("diagnostics"))
            .and_then(Value::as_array)
            .map(|diags| diags.len())
            .unwrap_or(0);
        assert_eq!(second_count, 0, "valid change should clear diagnostics");
    }

    #[test]
    fn did_close_clears_diagnostics() {
        let uri = "file:///tmp/lsp_did_close.ai";
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "ailang",
                    "version": 1,
                    "text": "fn main( -> Int {\n  0\n}\n"
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": {
                "textDocument": {"uri": uri}
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "exit",
        })));

        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        run_stdio(&mut reader, &mut output).expect("stdio loop should succeed");

        let messages = collect_messages(output);
        let publishes: Vec<&Value> = messages
            .iter()
            .filter(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")))
            .collect();
        assert_eq!(publishes.len(), 2, "expected diagnostics publish for open and close");

        let close_count = publishes[1]
            .get("params")
            .and_then(|params| params.get("diagnostics"))
            .and_then(Value::as_array)
            .map(|diags| diags.len())
            .unwrap_or(1);
        assert_eq!(close_count, 0, "didClose should clear diagnostics for URI");
    }

    #[test]
    fn definition_returns_function_location_for_call_identifier() {
        let uri = "file:///tmp/lsp_definition.ai";
        let source = "fn helper() -> Int {\n  1\n}\n\nfn main() -> Int {\n  helper()\n}\n";
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {"uri": uri},
                "position": {"line": 5, "character": 2}
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "exit",
        })));

        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        run_stdio(&mut reader, &mut output).expect("stdio loop should succeed");

        let messages = collect_messages(output);
        let definition_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(2)))
            .expect("definition response should exist");

        assert_eq!(
            definition_response
                .get("result")
                .and_then(|result| result.get("uri"))
                .and_then(Value::as_str),
            Some(uri),
            "definition URI should point to current document",
        );
        assert_eq!(
            definition_response
                .get("result")
                .and_then(|result| result.get("range"))
                .and_then(|range| range.get("start"))
                .and_then(|start| start.get("line"))
                .and_then(Value::as_u64),
            Some(0),
            "definition should resolve to helper function declaration start line",
        );
    }

    #[test]
    fn hover_returns_function_signature_for_call_identifier() {
        let uri = "file:///tmp/lsp_hover.ai";
        let source = "fn helper() -> Int {\n  1\n}\n\nfn main() -> Int {\n  helper()\n}\n";
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {"uri": uri},
                "position": {"line": 5, "character": 2}
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "exit",
        })));

        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        run_stdio(&mut reader, &mut output).expect("stdio loop should succeed");

        let messages = collect_messages(output);
        let hover_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(3)))
            .expect("hover response should exist");
        let contents = hover_response
            .get("result")
            .and_then(|result| result.get("contents"))
            .and_then(|contents| contents.get("value"))
            .and_then(Value::as_str)
            .unwrap_or("");
        assert!(
            contents.contains("fn helper() -> Int"),
            "hover should include helper signature"
        );
    }

    #[test]
    fn references_returns_call_sites_without_declaration_when_excluded() {
        let uri = "file:///tmp/lsp_references_calls.ai";
        let source =
            "fn helper() -> Int {\n  1\n}\n\nfn one() -> Int {\n  helper()\n}\n\nfn two() -> Int {\n  helper()\n}\n";
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/references",
            "params": {
                "textDocument": {"uri": uri},
                "position": {"line": 5, "character": 2},
                "context": {"includeDeclaration": false}
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "exit",
        })));

        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        run_stdio(&mut reader, &mut output).expect("stdio loop should succeed");

        let messages = collect_messages(output);
        let references_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(4)))
            .expect("references response should exist");
        let references = references_response
            .get("result")
            .and_then(Value::as_array)
            .expect("references result should be an array");
        assert_eq!(
            references.len(),
            2,
            "only call-site references should be returned when declaration is excluded",
        );

        let mut lines = references
            .iter()
            .filter_map(|location| {
                location
                    .get("range")
                    .and_then(|range| range.get("start"))
                    .and_then(|start| start.get("line"))
                    .and_then(Value::as_u64)
            })
            .collect::<Vec<_>>();
        lines.sort_unstable();
        assert_eq!(lines, vec![5, 9], "expected both helper call-site lines");
    }

    #[test]
    fn references_can_include_declaration_when_requested() {
        let uri = "file:///tmp/lsp_references_declaration.ai";
        let source =
            "fn helper() -> Int {\n  1\n}\n\nfn one() -> Int {\n  helper()\n}\n\nfn two() -> Int {\n  helper()\n}\n";
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/references",
            "params": {
                "textDocument": {"uri": uri},
                "position": {"line": 5, "character": 2},
                "context": {"includeDeclaration": true}
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "exit",
        })));

        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        run_stdio(&mut reader, &mut output).expect("stdio loop should succeed");

        let messages = collect_messages(output);
        let references_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(5)))
            .expect("references response should exist");
        let references = references_response
            .get("result")
            .and_then(Value::as_array)
            .expect("references result should be an array");
        assert_eq!(
            references.len(),
            3,
            "declaration should be added when includeDeclaration is true",
        );

        let declaration_line = references
            .first()
            .and_then(|location| location.get("range"))
            .and_then(|range| range.get("start"))
            .and_then(|start| start.get("line"))
            .and_then(Value::as_u64);
        assert_eq!(
            declaration_line,
            Some(0),
            "first reference should be helper declaration",
        );
    }

    #[test]
    fn run_stdio_returns_method_not_found_for_unknown_requests() {
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "workspace/unknownMethod",
            "params": {},
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "exit",
        })));

        let mut reader = BufReader::new(Cursor::new(input));
        let mut output = Vec::new();
        run_stdio(&mut reader, &mut output).expect("stdio loop should succeed");

        let messages = collect_messages(output);
        let response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(7)))
            .expect("expected unknown-method error response");
        assert_eq!(
            response
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(Value::as_i64),
            Some(-32601),
            "unknown request should return method-not-found code"
        );
    }
}
