use ailang_core::ast::{Block, Expr, ExprKind, ItemKind, Program, Stmt, StmtKind, TypeExpr, TypeExprKind};
use ailang_core::{analyze_program, parse_source, Diagnostic as CoreDiagnostic, Severity as CoreSeverity, Span};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::Instant;
use url::Url;

const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_REQUEST: i64 = -32600;

#[derive(Default)]
struct ServerState {
    documents: HashMap<String, String>,
    parsed_programs: HashMap<String, Program>,
    open_document_symbols: HashMap<String, Vec<FunctionSymbol>>,
    open_document_imports: HashMap<String, HashSet<String>>,
    reverse_open_document_imports: HashMap<String, HashSet<String>>,
}

#[derive(Clone)]
struct FunctionSymbol {
    id: String,
    name: String,
    span: Span,
    signature: String,
}

#[derive(Clone)]
struct IdentifierHit {
    name: String,
    span: Span,
}

#[derive(Clone)]
struct DeclarationMatch {
    uri: String,
    symbol: FunctionSymbol,
    source: String,
}

struct RequestDeadline {
    started_at: Instant,
    budget_ms: u128,
}

impl RequestDeadline {
    fn new(budget_ms: u128) -> Self {
        Self {
            started_at: Instant::now(),
            budget_ms,
        }
    }

    fn is_expired(&self) -> bool {
        self.started_at.elapsed().as_millis() >= self.budget_ms
    }
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
                            "referencesProvider": true,
                            "implementationProvider": true,
                            "completionProvider": {
                                "resolveProvider": false
                            },
                            "renameProvider": {
                                "prepareProvider": true
                            },
                            "codeActionProvider": {
                                "codeActionKinds": ["quickfix"]
                            }
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
                refresh_program_cache(state, &uri, &text);
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
                refresh_program_cache(state, &uri, &text);
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
                state.parsed_programs.remove(&uri);
                state.open_document_symbols.remove(&uri);
                remove_open_document_import_edges(state, &uri);
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
        Some("textDocument/implementation") => {
            if let Some(id) = id {
                if let Some((uri, line, character)) = parse_text_document_position(message) {
                    match implementation_at_position(state, &uri, line, character) {
                        Ok(result) => send_response(writer, id, result)?,
                        Err(err) => send_error_response(writer, id, INVALID_REQUEST, err.to_string())?,
                    }
                } else {
                    send_error_response(
                        writer,
                        id,
                        INVALID_REQUEST,
                        "invalid implementation request payload".to_string(),
                    )?;
                }
            }
        }
        Some("textDocument/completion") => {
            if let Some(id) = id {
                if let Some((uri, line, character)) = parse_text_document_position(message) {
                    match completion_at_position(state, &uri, line, character) {
                        Ok(result) => send_response(writer, id, result)?,
                        Err(err) => send_error_response(writer, id, INVALID_REQUEST, err.to_string())?,
                    }
                } else {
                    send_error_response(
                        writer,
                        id,
                        INVALID_REQUEST,
                        "invalid completion request payload".to_string(),
                    )?;
                }
            }
        }
        Some("textDocument/prepareRename") => {
            if let Some(id) = id {
                if let Some((uri, line, character)) = parse_text_document_position(message) {
                    match prepare_rename_at_position(state, &uri, line, character) {
                        Ok(result) => send_response(writer, id, result)?,
                        Err(err) => send_error_response(writer, id, INVALID_REQUEST, err.to_string())?,
                    }
                } else {
                    send_error_response(
                        writer,
                        id,
                        INVALID_REQUEST,
                        "invalid prepareRename request payload".to_string(),
                    )?;
                }
            }
        }
        Some("textDocument/rename") => {
            if let Some(id) = id {
                if let Some((uri, line, character, new_name)) = parse_rename_request(message) {
                    match rename_at_position(state, &uri, line, character, &new_name) {
                        Ok(result) => send_response(writer, id, result)?,
                        Err(err) => send_error_response(writer, id, INVALID_REQUEST, err.to_string())?,
                    }
                } else {
                    send_error_response(
                        writer,
                        id,
                        INVALID_REQUEST,
                        "invalid rename request payload".to_string(),
                    )?;
                }
            }
        }
        Some("textDocument/codeAction") => {
            if let Some(id) = id {
                if let Some((uri, diagnostics)) = parse_code_action_request(message) {
                    let actions = code_actions_from_diagnostics(state, &uri, &diagnostics);
                    send_response(writer, id, Value::Array(actions))?;
                } else {
                    send_error_response(
                        writer,
                        id,
                        INVALID_REQUEST,
                        "invalid codeAction request payload".to_string(),
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

fn parse_rename_request(message: &Value) -> Option<(String, usize, usize, String)> {
    let (uri, line, character) = parse_text_document_position(message)?;
    let new_name = message
        .get("params")
        .and_then(|params| params.get("newName"))
        .and_then(Value::as_str)?
        .to_string();
    Some((uri, line, character, new_name))
}

fn parse_code_action_request(message: &Value) -> Option<(String, Vec<Value>)> {
    let uri = message
        .get("params")
        .and_then(|params| params.get("textDocument"))
        .and_then(|doc| doc.get("uri"))
        .and_then(Value::as_str)?
        .to_string();
    let diagnostics = message
        .get("params")
        .and_then(|params| params.get("context"))
        .and_then(|context| context.get("diagnostics"))
        .and_then(Value::as_array)?
        .clone();
    Some((uri, diagnostics))
}

fn definition_at_position(
    state: &ServerState,
    uri: &str,
    line: usize,
    character: usize,
) -> io::Result<Value> {
    let deadline = RequestDeadline::new(request_budget_ms());
    let (path, source) = load_document_source(state, uri)?;
    let program = match load_cached_program(state, uri, &path, &source, Some(&deadline)) {
        Some(program) => program,
        None => return Ok(Value::Null),
    };

    let Some(hit) = find_identifier_at_position(&program, &path, line, character, Some(&deadline))
    else {
        return Ok(Value::Null);
    };
    let Some(symbol_match) =
        find_symbol_declaration_in_workspace(state, &hit.name, uri, &source, &deadline)
    else {
        return Ok(Value::Null);
    };

    Ok(location_from_span(&symbol_match.uri, &symbol_match.symbol.span))
}

fn hover_at_position(
    state: &ServerState,
    uri: &str,
    line: usize,
    character: usize,
) -> io::Result<Value> {
    let deadline = RequestDeadline::new(request_budget_ms());
    let (path, source) = load_document_source(state, uri)?;
    let program = match load_cached_program(state, uri, &path, &source, Some(&deadline)) {
        Some(program) => program,
        None => return Ok(Value::Null),
    };

    let Some(hit) = find_identifier_at_position(&program, &path, line, character, Some(&deadline))
    else {
        return Ok(Value::Null);
    };
    let Some(symbol_match) =
        find_symbol_declaration_in_workspace(state, &hit.name, uri, &source, &deadline)
    else {
        return Ok(Value::Null);
    };

    Ok(json!({
        "contents": {
            "kind": "markdown",
            "value": format!("```ailang\\n{}\\n```", symbol_match.symbol.signature),
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
    let deadline = RequestDeadline::new(request_budget_ms());
    let (path, source) = load_document_source(state, uri)?;
    let program = match load_cached_program(state, uri, &path, &source, Some(&deadline)) {
        Some(program) => program,
        None => return Ok(Value::Array(Vec::new())),
    };

    let Some(hit) = find_identifier_at_position(&program, &path, line, character, Some(&deadline))
    else {
        return Ok(Value::Array(Vec::new()));
    };

    let target_name = hit.name.clone();
    let declaration =
        find_symbol_declaration_in_workspace(state, &target_name, uri, &source, &deadline);
    let mut locations = Vec::new();
    for (doc_uri, doc_source) in workspace_document_entries(state, uri, &source, &deadline) {
        if deadline.is_expired() {
            break;
        }
        let Some(doc_path) = uri_to_path(&doc_uri) else {
            continue;
        };
        let Some(doc_program) =
            load_cached_program(state, &doc_uri, &doc_path, &doc_source, Some(&deadline))
        else {
            continue;
        };
        locations.extend(
            collect_identifier_hits_by_name(&doc_program, &doc_path, &target_name, Some(&deadline))
                .into_iter()
                .map(|found| location_from_span(&doc_uri, &found.span)),
        );
    }
    if include_declaration {
        if let Some(found) = declaration {
            locations.insert(0, location_from_span(&found.uri, &found.symbol.span));
        }
    }
    Ok(Value::Array(locations))
}

fn implementation_at_position(
    state: &ServerState,
    uri: &str,
    line: usize,
    character: usize,
) -> io::Result<Value> {
    let definition = definition_at_position(state, uri, line, character)?;
    if definition.is_null() {
        Ok(Value::Array(Vec::new()))
    } else {
        Ok(Value::Array(vec![definition]))
    }
}

fn completion_at_position(
    state: &ServerState,
    uri: &str,
    _line: usize,
    _character: usize,
) -> io::Result<Value> {
    let deadline = RequestDeadline::new(request_budget_ms());
    let (path, source) = load_document_source(state, uri)?;
    let mut items = completion_keyword_items();
    let mut seen_labels = items
        .iter()
        .filter_map(|item| item.get("label").and_then(Value::as_str))
        .map(|label| label.to_string())
        .collect::<HashSet<_>>();

    if let Some(symbols) = state.open_document_symbols.get(uri) {
        for symbol in symbols {
            if seen_labels.insert(symbol.name.clone()) {
                items.push(json!({
                    "label": symbol.name,
                    "kind": 3,
                    "detail": symbol.signature,
                }));
            }
        }
    } else if let Some(program) = load_cached_program(state, uri, &path, &source, Some(&deadline)) {
        for symbol in collect_function_symbols(&program, Some(&deadline)) {
            if seen_labels.insert(symbol.name.clone()) {
                items.push(json!({
                    "label": symbol.name,
                    "kind": 3,
                    "detail": symbol.signature,
                }));
            }
        }
    }

    Ok(Value::Array(items))
}

fn completion_keyword_items() -> Vec<Value> {
    vec![
        json!({"label": "fn", "kind": 14}),
        json!({"label": "let", "kind": 14}),
        json!({"label": "return", "kind": 14}),
        json!({"label": "if", "kind": 14}),
        json!({"label": "match", "kind": 14}),
    ]
}

fn prepare_rename_at_position(
    state: &ServerState,
    uri: &str,
    line: usize,
    character: usize,
) -> io::Result<Value> {
    let deadline = RequestDeadline::new(request_budget_ms());
    let (path, source) = load_document_source(state, uri)?;
    let program = match load_cached_program(state, uri, &path, &source, Some(&deadline)) {
        Some(program) => program,
        None => return Ok(Value::Null),
    };

    let Some(hit) = find_identifier_at_position(&program, &path, line, character, Some(&deadline))
    else {
        return Ok(Value::Null);
    };
    if find_symbol_declaration_in_workspace(state, &hit.name, uri, &source, &deadline).is_none() {
        return Ok(Value::Null);
    }

    Ok(json!({
        "range": range_from_span(&hit.span),
        "placeholder": hit.name,
    }))
}

fn rename_at_position(
    state: &ServerState,
    uri: &str,
    line: usize,
    character: usize,
    new_name: &str,
) -> io::Result<Value> {
    let deadline = RequestDeadline::new(request_budget_ms());
    if !is_valid_identifier_name(new_name) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid rename target `{new_name}`"),
        ));
    }

    let (path, source) = load_document_source(state, uri)?;
    let program = match load_cached_program(state, uri, &path, &source, Some(&deadline)) {
        Some(program) => program,
        None => {
            return Ok(json!({
                "changes": {
                    uri: []
                }
            }))
        }
    };

    let Some(hit) = find_identifier_at_position(&program, &path, line, character, Some(&deadline))
    else {
        return Ok(json!({
            "changes": {
                uri: []
            }
        }));
    };
    let target_name = hit.name;
    let Some(declaration) =
        find_symbol_declaration_in_workspace(state, &target_name, uri, &source, &deadline)
    else {
        return Ok(json!({
            "changes": {
                uri: []
            }
        }));
    };
    let mut edits_by_uri = BTreeMap::<String, Vec<Value>>::new();

    for (doc_uri, doc_source) in workspace_document_entries(state, uri, &source, &deadline) {
        if deadline.is_expired() {
            break;
        }
        let Some(doc_path) = uri_to_path(&doc_uri) else {
            continue;
        };
        let Some(doc_program) =
            load_cached_program(state, &doc_uri, &doc_path, &doc_source, Some(&deadline))
        else {
            continue;
        };

        let edits =
            collect_identifier_hits_by_name(&doc_program, &doc_path, &target_name, Some(&deadline))
            .into_iter()
            .map(|found| {
                json!({
                    "range": range_from_span(&found.span),
                    "newText": new_name,
                })
            })
            .collect::<Vec<_>>();
        if !edits.is_empty() {
            edits_by_uri.insert(doc_uri, edits);
        }
    }

    if let Some(decl_span) = function_declaration_name_span(&declaration.symbol, &declaration.source)
    {
        edits_by_uri
            .entry(declaration.uri)
            .or_default()
            .insert(
                0,
                json!({
                    "range": range_from_span(&decl_span),
                    "newText": new_name,
                }),
            );
    }

    let mut changes = serde_json::Map::new();
    for (edit_uri, edits) in edits_by_uri {
        if !edits.is_empty() {
            changes.insert(edit_uri, Value::Array(edits));
        }
    }

    Ok(json!({ "changes": changes }))
}

fn function_declaration_name_span(symbol: &FunctionSymbol, source: &str) -> Option<Span> {
    let line_index = symbol.span.start_line.checked_sub(1)?;
    let line_text = source.lines().nth(line_index)?;
    let needle = format!("fn {}", symbol.name);
    let fn_index = line_text.find(&needle)?;
    let name_start_col = fn_index + 3;
    let name_end_col = name_start_col + symbol.name.len();
    Some(Span {
        file: symbol.span.file.clone(),
        start_line: symbol.span.start_line,
        start_col: name_start_col + 1,
        end_line: symbol.span.start_line,
        end_col: name_end_col,
    })
}

fn is_valid_identifier_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn code_actions_from_diagnostics(
    state: &ServerState,
    uri: &str,
    diagnostics: &[Value],
) -> Vec<Value> {
    let mut actions = Vec::new();
    let mut seen_titles = HashSet::new();
    let source = state.documents.get(uri).cloned();

    for diagnostic in diagnostics {
        let Some(code) = diagnostic.get("code").and_then(Value::as_str) else {
            continue;
        };

        let (title, edit) = match code {
            "E1002" => {
                let validate_edit = source.as_ref().and_then(|text| {
                    extract_range_text_from_diagnostic(diagnostic, text).map(|selected| {
                        json!({
                            "changes": {
                                uri: [
                                    {
                                        "range": diagnostic.get("range").cloned().unwrap_or(Value::Null),
                                        "newText": format!("validate({selected})?"),
                                    }
                                ]
                            }
                        })
                    })
                });
                if validate_edit.is_some() {
                    (Some("Wrap with validate(...)?"), validate_edit)
                } else {
                    (Some("Insert validate/sanitize gate for untrusted value"), None)
                }
            }
            "E1003" | "E1004" | "E1005" => {
                let redact_edit = source.as_ref().and_then(|text| {
                    extract_range_text_from_diagnostic(diagnostic, text).map(|selected| {
                        json!({
                            "changes": {
                                uri: [
                                    {
                                        "range": diagnostic.get("range").cloned().unwrap_or(Value::Null),
                                        "newText": format!("redact({selected})"),
                                    }
                                ]
                            }
                        })
                    })
                });
                (Some("Wrap with redact(...)"), redact_edit)
            }
            "E2001" => {
                let effect_edit = source
                    .as_ref()
                    .and_then(|text| build_missing_effect_edit(uri, text, diagnostic));
                if let Some(edit) = effect_edit {
                    (Some("Add missing effect declaration"), Some(edit))
                } else if source.is_none() {
                    (Some("Declare missing effect in function signature"), None)
                } else {
                    (None, None)
                }
            }
            _ => (None, None),
        };

        let Some(title) = title else {
            continue;
        };
        if !seen_titles.insert(title.to_string()) {
            continue;
        }

        let mut action = json!({
            "title": title,
            "kind": "quickfix",
            "diagnostics": [diagnostic.clone()],
        });
        if let Some(edit) = edit {
            if let Some(action_obj) = action.as_object_mut() {
                action_obj.insert("edit".to_string(), edit);
            }
        }
        actions.push(action);
    }

    actions
}

fn extract_range_text_from_diagnostic(diagnostic: &Value, source: &str) -> Option<String> {
    let range = diagnostic.get("range")?;
    extract_range_text(source, range)
}

fn extract_range_text(source: &str, range: &Value) -> Option<String> {
    let start = range.get("start")?;
    let end = range.get("end")?;
    let start_line = start.get("line")?.as_u64()? as usize;
    let start_char = start.get("character")?.as_u64()? as usize;
    let end_line = end.get("line")?.as_u64()? as usize;
    let end_char = end.get("character")?.as_u64()? as usize;
    if start_line != end_line {
        return None;
    }

    let line = source.lines().nth(start_line)?;
    if end_char < start_char {
        return None;
    }
    let selected = line
        .chars()
        .skip(start_char)
        .take(end_char.saturating_sub(start_char))
        .collect::<String>();
    if selected.is_empty() {
        return None;
    }
    Some(selected)
}

fn build_missing_effect_edit(uri: &str, source: &str, diagnostic: &Value) -> Option<Value> {
    let effect = extract_effect_name_from_diagnostic(diagnostic)?;
    let start_line = diagnostic
        .get("range")
        .and_then(|range| range.get("start"))
        .and_then(|start| start.get("line"))
        .and_then(Value::as_u64)? as usize;

    let lines = source.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return None;
    }
    let search_end = start_line.min(lines.len().saturating_sub(1));
    let mut signature_line_idx = None;
    for idx in (0..=search_end).rev() {
        if lines[idx].contains("fn ") {
            signature_line_idx = Some(idx);
            break;
        }
    }
    let line_idx = signature_line_idx?;
    let mut signature_end_idx = line_idx;
    for idx in line_idx..=search_end {
        signature_end_idx = idx;
        if lines[idx].contains('{') {
            break;
        }
    }

    for idx in line_idx..=signature_end_idx {
        let signature_line = lines[idx];
        if let Some(effects_idx) = signature_line.find("effects {") {
            let open_brace = signature_line[effects_idx..].find('{')? + effects_idx;
            let close_brace = signature_line[open_brace + 1..].find('}')? + open_brace + 1;
            let existing_effects = signature_line[open_brace + 1..close_brace]
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();
            if existing_effects.iter().any(|value| *value == effect) {
                return None;
            }

            return Some(json!({
                "changes": {
                    uri: [
                        {
                            "range": {
                                "start": {"line": idx, "character": close_brace},
                                "end": {"line": idx, "character": close_brace}
                            },
                            "newText": format!(", {effect}")
                        }
                    ]
                }
            }));
        }
    }

    let insertion = (line_idx..=signature_end_idx)
        .find_map(|idx| lines[idx].find("->").map(|character| (idx, character)))
        .or_else(|| {
            (line_idx..=signature_end_idx)
                .find_map(|idx| lines[idx].find('{').map(|character| (idx, character)))
        })?;

    Some(json!({
        "changes": {
            uri: [
                {
                    "range": {
                        "start": {"line": insertion.0, "character": insertion.1},
                        "end": {"line": insertion.0, "character": insertion.1}
                    },
                    "newText": format!(" effects {{ {effect} }}")
                }
            ]
        }
    }))
}

fn extract_effect_name_from_diagnostic(diagnostic: &Value) -> Option<String> {
    if let Some(message) = diagnostic.get("message").and_then(Value::as_str) {
        if let Some(token) = extract_backtick_token(message) {
            return Some(token);
        }
    }

    let notes = diagnostic
        .get("data")
        .and_then(|data| data.get("notes"))
        .and_then(Value::as_array)?;
    for note in notes {
        if let Some(text) = note.as_str() {
            if let Some(token) = extract_backtick_token(text) {
                return Some(token);
            }
        }
    }
    None
}

fn extract_backtick_token(text: &str) -> Option<String> {
    let mut parts = text.split('`');
    let _ = parts.next()?;
    let token = parts.next()?.trim();
    if token.is_empty() {
        return None;
    }
    if token
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
    {
        return Some(token.to_string());
    }
    None
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

fn refresh_program_cache(state: &mut ServerState, uri: &str, text: &str) {
    update_open_document_import_edges(state, uri, text);
    invalidate_open_document_dependents(state, uri);

    let Some(path) = uri_to_path(uri) else {
        state.parsed_programs.remove(uri);
        state.open_document_symbols.remove(uri);
        return;
    };
    match parse_source(&path, text) {
        Ok(program) => {
            let symbols = collect_function_symbols(&program, None);
            state.parsed_programs.insert(uri.to_string(), program);
            state.open_document_symbols.insert(uri.to_string(), symbols);
        }
        Err(_) => {
            state.parsed_programs.remove(uri);
            state.open_document_symbols.remove(uri);
        }
    }
}

fn update_open_document_import_edges(state: &mut ServerState, uri: &str, text: &str) {
    remove_open_document_import_edges(state, uri);

    let imports = extract_open_document_import_uris(uri, text);
    if imports.is_empty() {
        return;
    }

    for import_uri in &imports {
        state
            .reverse_open_document_imports
            .entry(import_uri.clone())
            .or_default()
            .insert(uri.to_string());
    }
    state.open_document_imports.insert(uri.to_string(), imports);
}

fn remove_open_document_import_edges(state: &mut ServerState, uri: &str) {
    let Some(previous_imports) = state.open_document_imports.remove(uri) else {
        return;
    };

    for import_uri in previous_imports {
        if let Some(dependents) = state.reverse_open_document_imports.get_mut(&import_uri) {
            dependents.remove(uri);
            if dependents.is_empty() {
                state.reverse_open_document_imports.remove(&import_uri);
            }
        }
    }
}

fn invalidate_open_document_dependents(state: &mut ServerState, changed_uri: &str) {
    let mut stack = vec![changed_uri.to_string()];
    let mut visited = HashSet::from([changed_uri.to_string()]);

    while let Some(current_uri) = stack.pop() {
        let dependents = state
            .reverse_open_document_imports
            .get(&current_uri)
            .cloned()
            .unwrap_or_default();
        for dependent_uri in dependents {
            if !visited.insert(dependent_uri.clone()) {
                continue;
            }
            state.parsed_programs.remove(&dependent_uri);
            state.open_document_symbols.remove(&dependent_uri);
            stack.push(dependent_uri);
        }
    }
}

fn extract_open_document_import_uris(source_uri: &str, text: &str) -> HashSet<String> {
    let Some(source_path) = uri_to_path(source_uri) else {
        return HashSet::new();
    };
    let Some(base_dir) = source_path.parent() else {
        return HashSet::new();
    };

    let mut imports = HashSet::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("import ") {
            continue;
        }
        let Some(path_literal) = extract_quoted_path_literal(trimmed) else {
            continue;
        };
        if !path_literal.starts_with('.') {
            continue;
        }

        let candidate = normalize_joined_path(base_dir, &path_literal);
        let Ok(candidate_uri) = Url::from_file_path(candidate) else {
            continue;
        };
        imports.insert(candidate_uri.to_string());
    }
    imports
}

fn extract_quoted_path_literal(input: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let Some(start) = input.find(quote) else {
            continue;
        };
        let tail = &input[start + 1..];
        let Some(end) = tail.find(quote) else {
            continue;
        };
        let literal = tail[..end].trim();
        if literal.is_empty() {
            continue;
        }
        return Some(literal.to_string());
    }
    None
}

fn normalize_joined_path(base_dir: &std::path::Path, relative: &str) -> PathBuf {
    let joined = base_dir.join(relative);
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn load_cached_program(
    state: &ServerState,
    uri: &str,
    path: &PathBuf,
    source: &str,
    deadline: Option<&RequestDeadline>,
) -> Option<Program> {
    if let Some(program) = state.parsed_programs.get(uri) {
        return Some(program.clone());
    }
    if deadline_exceeded(deadline) {
        return None;
    }
    parse_source(path, source).ok()
}

fn workspace_document_entries(
    state: &ServerState,
    primary_uri: &str,
    primary_source: &str,
    deadline: &RequestDeadline,
) -> Vec<(String, String)> {
    let mut entries = state
        .documents
        .iter()
        .map(|(uri, source)| (uri.clone(), source.clone()))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    let mut seen_uris = entries
        .iter()
        .map(|(uri, _)| uri.clone())
        .collect::<HashSet<_>>();

    if !seen_uris.contains(primary_uri) {
        entries.push((primary_uri.to_string(), primary_source.to_string()));
        seen_uris.insert(primary_uri.to_string());
    }

    if scan_unopened_workspace_files() {
        if let Some(root) = project_root_for_uri(primary_uri) {
            let mut ai_files = Vec::new();
            collect_ai_files(&root, &mut ai_files);
            ai_files.sort();

            for file in ai_files {
                if deadline.is_expired() {
                    break;
                }
                let Ok(file_url) = Url::from_file_path(&file) else {
                    continue;
                };
                let file_uri = file_url.to_string();
                if seen_uris.contains(&file_uri) {
                    continue;
                }
                let Ok(file_source) = fs::read_to_string(&file) else {
                    continue;
                };
                entries.push((file_uri.clone(), file_source));
                seen_uris.insert(file_uri);
            }
        }
    }

    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
}

fn scan_unopened_workspace_files() -> bool {
    std::env::var("AILANG_LSP_SCAN_UNOPENED_FILES")
        .ok()
        .map(|raw| parse_env_bool(&raw))
        .unwrap_or(true)
}

fn parse_env_bool(raw: &str) -> bool {
    !matches!(raw.trim().to_ascii_lowercase().as_str(), "0" | "false" | "off" | "no")
}

fn project_root_for_uri(uri: &str) -> Option<PathBuf> {
    let path = uri_to_path(uri)?;
    let mut cursor = path.parent()?.to_path_buf();
    loop {
        if cursor.join("ailang.toml").is_file() {
            return Some(cursor);
        }
        if !cursor.pop() {
            break;
        }
    }
    path.parent().map(|parent| parent.to_path_buf())
}

fn collect_ai_files(root: &PathBuf, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_ai_files(&path, out);
            continue;
        }
        if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("ai") {
            out.push(path);
        }
    }
}

fn find_symbol_declaration_in_workspace(
    state: &ServerState,
    target_name: &str,
    primary_uri: &str,
    primary_source: &str,
    deadline: &RequestDeadline,
) -> Option<DeclarationMatch> {
    let mut matched: Option<DeclarationMatch> = None;
    for (doc_uri, doc_source) in
        workspace_document_entries(state, primary_uri, primary_source, deadline)
    {
        if deadline.is_expired() {
            break;
        }
        let symbols = if let Some(cached_symbols) = state.open_document_symbols.get(&doc_uri) {
            cached_symbols.clone()
        } else {
            let Some(doc_path) = uri_to_path(&doc_uri) else {
                continue;
            };
            let Some(doc_program) =
                load_cached_program(state, &doc_uri, &doc_path, &doc_source, Some(deadline))
            else {
                continue;
            };
            collect_function_symbols(&doc_program, Some(deadline))
        };

        if let Some(symbol) = symbols
            .into_iter()
            .find(|symbol| symbol.name == target_name)
        {
            let candidate = DeclarationMatch {
                uri: doc_uri,
                symbol,
                source: doc_source,
            };
            if let Some(existing) = &matched {
                if existing.symbol.id != candidate.symbol.id {
                    return None;
                }
                continue;
            }
            matched = Some(candidate);
        }
    }
    matched
}

fn deadline_exceeded(deadline: Option<&RequestDeadline>) -> bool {
    deadline.is_some_and(RequestDeadline::is_expired)
}

fn collect_function_symbols(program: &Program, deadline: Option<&RequestDeadline>) -> Vec<FunctionSymbol> {
    let mut symbols = Vec::new();
    for item in &program.items {
        if deadline_exceeded(deadline) {
            break;
        }
        if let Some(symbol) = match &item.kind {
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
                    id: function_symbol_id(&item.span, &function.name),
                    name: function.name.clone(),
                    span: item.span.clone(),
                    signature,
                })
            }
            _ => None,
        } {
            symbols.push(symbol);
        }
    }
    symbols
}

fn function_symbol_id(span: &Span, name: &str) -> String {
    format!(
        "{}:{}:{}:{}",
        span.file.display(),
        span.start_line,
        span.start_col,
        name
    )
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
    deadline: Option<&RequestDeadline>,
) -> Option<IdentifierHit> {
    if deadline_exceeded(deadline) {
        return None;
    }
    for item in &program.items {
        if deadline_exceeded(deadline) {
            return None;
        }
        if let ItemKind::Function(function) = &item.kind {
            if let Some(hit) =
                find_identifier_in_block(&function.body, file, line, character, deadline)
            {
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
    deadline: Option<&RequestDeadline>,
) -> Option<IdentifierHit> {
    if deadline_exceeded(deadline) {
        return None;
    }
    for statement in &block.statements {
        if deadline_exceeded(deadline) {
            return None;
        }
        if let Some(hit) = find_identifier_in_statement(statement, file, line, character, deadline) {
            return Some(hit);
        }
    }

    block
        .tail
        .as_ref()
        .and_then(|tail| find_identifier_in_expr(tail, file, line, character, deadline))
}

fn find_identifier_in_statement(
    statement: &Stmt,
    file: &PathBuf,
    line: usize,
    character: usize,
    deadline: Option<&RequestDeadline>,
) -> Option<IdentifierHit> {
    if deadline_exceeded(deadline) {
        return None;
    }
    match &statement.kind {
        StmtKind::Let { value, .. } => find_identifier_in_expr(value, file, line, character, deadline),
        StmtKind::Return { value } => value
            .as_ref()
            .and_then(|expr| find_identifier_in_expr(expr, file, line, character, deadline)),
        StmtKind::Expr { expr } => find_identifier_in_expr(expr, file, line, character, deadline),
    }
}

fn find_identifier_in_expr(
    expr: &Expr,
    file: &PathBuf,
    line: usize,
    character: usize,
    deadline: Option<&RequestDeadline>,
) -> Option<IdentifierHit> {
    if deadline_exceeded(deadline) {
        return None;
    }
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
        ExprKind::Unary { expr, .. } => find_identifier_in_expr(expr, file, line, character, deadline),
        ExprKind::Binary { left, right, .. } => {
            find_identifier_in_expr(left, file, line, character, deadline)
                .or_else(|| find_identifier_in_expr(right, file, line, character, deadline))
        }
        ExprKind::Member { object, .. } => {
            find_identifier_in_expr(object, file, line, character, deadline)
        }
        ExprKind::Call { callee, args } => find_identifier_in_expr(callee, file, line, character, deadline)
            .or_else(|| {
                args.iter()
                    .find_map(|arg| find_identifier_in_expr(arg, file, line, character, deadline))
            }),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => find_identifier_in_expr(condition, file, line, character, deadline)
            .or_else(|| find_identifier_in_block(then_branch, file, line, character, deadline))
            .or_else(|| {
                else_branch
                    .as_ref()
                    .and_then(|expr| find_identifier_in_expr(expr, file, line, character, deadline))
            }),
        ExprKind::Match { scrutinee, arms } => {
            find_identifier_in_expr(scrutinee, file, line, character, deadline).or_else(|| {
                arms.iter()
                    .find_map(|arm| find_identifier_in_expr(&arm.value, file, line, character, deadline))
            })
        }
        ExprKind::Block(block) => find_identifier_in_block(block, file, line, character, deadline),
        ExprKind::Number(_) | ExprKind::String(_) | ExprKind::Bool(_) => None,
    }
}

fn collect_identifier_hits_by_name(
    program: &Program,
    file: &PathBuf,
    target_name: &str,
    deadline: Option<&RequestDeadline>,
) -> Vec<IdentifierHit> {
    let mut hits = Vec::new();
    if deadline_exceeded(deadline) {
        return hits;
    }
    for item in &program.items {
        if deadline_exceeded(deadline) {
            break;
        }
        if let ItemKind::Function(function) = &item.kind {
            collect_identifier_hits_in_block(&function.body, file, target_name, &mut hits, deadline);
        }
    }
    hits
}

fn collect_identifier_hits_in_block(
    block: &Block,
    file: &PathBuf,
    target_name: &str,
    hits: &mut Vec<IdentifierHit>,
    deadline: Option<&RequestDeadline>,
) {
    if deadline_exceeded(deadline) {
        return;
    }
    for statement in &block.statements {
        if deadline_exceeded(deadline) {
            return;
        }
        collect_identifier_hits_in_statement(statement, file, target_name, hits, deadline);
    }
    if let Some(tail) = &block.tail {
        collect_identifier_hits_in_expr(tail, file, target_name, hits, deadline);
    }
}

fn collect_identifier_hits_in_statement(
    statement: &Stmt,
    file: &PathBuf,
    target_name: &str,
    hits: &mut Vec<IdentifierHit>,
    deadline: Option<&RequestDeadline>,
) {
    if deadline_exceeded(deadline) {
        return;
    }
    match &statement.kind {
        StmtKind::Let { value, .. } => {
            collect_identifier_hits_in_expr(value, file, target_name, hits, deadline)
        }
        StmtKind::Return { value } => {
            if let Some(expr) = value {
                collect_identifier_hits_in_expr(expr, file, target_name, hits, deadline);
            }
        }
        StmtKind::Expr { expr } => {
            collect_identifier_hits_in_expr(expr, file, target_name, hits, deadline)
        }
    }
}

fn collect_identifier_hits_in_expr(
    expr: &Expr,
    file: &PathBuf,
    target_name: &str,
    hits: &mut Vec<IdentifierHit>,
    deadline: Option<&RequestDeadline>,
) {
    if deadline_exceeded(deadline) {
        return;
    }
    match &expr.kind {
        ExprKind::Identifier(_) => {}
        ExprKind::Unary { expr, .. } => {
            collect_identifier_hits_in_expr(expr, file, target_name, hits, deadline)
        }
        ExprKind::Binary { left, right, .. } => {
            collect_identifier_hits_in_expr(left, file, target_name, hits, deadline);
            collect_identifier_hits_in_expr(right, file, target_name, hits, deadline);
        }
        ExprKind::Member { object, .. } => {
            collect_identifier_hits_in_expr(object, file, target_name, hits, deadline);
        }
        ExprKind::Call { callee, args } => {
            if let ExprKind::Identifier(name) = &callee.kind {
                if name == target_name && &callee.span.file == file {
                    hits.push(IdentifierHit {
                        name: name.clone(),
                        span: callee.span.clone(),
                    });
                }
            }
            collect_identifier_hits_in_expr(callee, file, target_name, hits, deadline);
            for arg in args {
                collect_identifier_hits_in_expr(arg, file, target_name, hits, deadline);
            }
        }
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_identifier_hits_in_expr(condition, file, target_name, hits, deadline);
            collect_identifier_hits_in_block(then_branch, file, target_name, hits, deadline);
            if let Some(expr) = else_branch {
                collect_identifier_hits_in_expr(expr, file, target_name, hits, deadline);
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            collect_identifier_hits_in_expr(scrutinee, file, target_name, hits, deadline);
            for arm in arms {
                collect_identifier_hits_in_expr(&arm.value, file, target_name, hits, deadline);
            }
        }
        ExprKind::Block(block) => {
            collect_identifier_hits_in_block(block, file, target_name, hits, deadline)
        }
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
    diagnostics_for_document_with_limits(
        uri,
        text,
        analysis_budget_ms(),
        max_diagnostics_per_document(),
    )
}

fn diagnostics_for_document_with_limits(
    uri: &str,
    text: &str,
    analysis_budget_ms: u128,
    max_diagnostics_per_document: usize,
) -> Vec<Value> {
    let started_at = Instant::now();
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

    let mut diagnostics = diagnostics
        .into_iter()
        .filter(|diag| diag.span.file == path)
        .map(core_diagnostic_to_lsp)
        .collect::<Vec<_>>();
    if diagnostics.len() > max_diagnostics_per_document {
        diagnostics.truncate(max_diagnostics_per_document);
    }
    if started_at.elapsed().as_millis() >= analysis_budget_ms {
        diagnostics.push(json!({
            "range": {
                "start": {"line": 0, "character": 0},
                "end": {"line": 0, "character": 1}
            },
            "severity": 3,
            "code": "I9001",
            "source": "ailang-lsp",
            "message": format!(
                "analysis budget exceeded ({}ms); results may be incomplete",
                analysis_budget_ms
            )
        }));
    }
    diagnostics
}

fn analysis_budget_ms() -> u128 {
    std::env::var("AILANG_LSP_ANALYSIS_BUDGET_MS")
        .ok()
        .and_then(|raw| raw.parse::<u128>().ok())
        .unwrap_or(200)
}

fn max_diagnostics_per_document() -> usize {
    std::env::var("AILANG_LSP_MAX_DIAGNOSTICS")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .map(|value| value.max(1))
        .unwrap_or(200)
}

fn request_budget_ms() -> u128 {
    std::env::var("AILANG_LSP_REQUEST_BUDGET_MS")
        .ok()
        .and_then(|raw| raw.parse::<u128>().ok())
        .unwrap_or_else(analysis_budget_ms)
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
    use super::{
        collect_function_symbols, collect_identifier_hits_by_name, diagnostics_for_document_with_limits,
        find_identifier_at_position, load_cached_program, parse_env_bool, read_message,
        refresh_program_cache, run_stdio, update_open_document_import_edges, RequestDeadline,
        ServerState,
    };
    use ailang_core::parse_source;
    use serde_json::{json, Value};
    use std::fs;
    use std::io::{BufReader, Cursor};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use url::Url;

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

    fn make_temp_workspace(prefix: &str) -> PathBuf {
        let mut root = std::env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        root.push(format!("{prefix}_{}_{}", std::process::id(), stamp));
        fs::create_dir_all(&root).expect("temp workspace should be created");
        root
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
        assert_eq!(
            messages[0]
                .get("result")
                .and_then(|result| result.get("capabilities"))
                .and_then(|caps| caps.get("implementationProvider"))
                .and_then(Value::as_bool),
            Some(true),
            "initialize response should advertise implementation provider"
        );
        assert_eq!(
            messages[0]
                .get("result")
                .and_then(|result| result.get("capabilities"))
                .and_then(|caps| caps.get("completionProvider"))
                .and_then(|completion| completion.get("resolveProvider"))
                .and_then(Value::as_bool),
            Some(false),
            "initialize response should advertise completion provider",
        );
        assert_eq!(
            messages[0]
                .get("result")
                .and_then(|result| result.get("capabilities"))
                .and_then(|caps| caps.get("renameProvider"))
                .and_then(|rename| rename.get("prepareProvider"))
                .and_then(Value::as_bool),
            Some(true),
            "initialize response should advertise rename provider",
        );
        assert_eq!(
            messages[0]
                .get("result")
                .and_then(|result| result.get("capabilities"))
                .and_then(|caps| caps.get("codeActionProvider"))
                .and_then(|provider| provider.get("codeActionKinds"))
                .and_then(Value::as_array)
                .map(|kinds| kinds.iter().any(|kind| kind.as_str() == Some("quickfix"))),
            Some(true),
            "initialize response should advertise quickfix code actions",
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
    fn diagnostics_budget_exceeded_adds_info_diagnostic() {
        let uri = "file:///tmp/lsp_budget.ai";
        let source = "fn main() -> Int {\n  0\n}\n";
        let diagnostics = diagnostics_for_document_with_limits(uri, source, 0, 200);
        assert!(
            diagnostics.iter().any(|diag| {
                diag.get("code")
                    .and_then(Value::as_str)
                    .map(|code| code == "I9001")
                    .unwrap_or(false)
            }),
            "budget overflow should append I9001 diagnostic",
        );
    }

    #[test]
    fn program_cache_tracks_parse_validity() {
        let uri = "file:///tmp/lsp_cache.ai";
        let valid_source = "fn main() -> Int {\n  0\n}\n";
        let invalid_source = "fn main( -> Int {\n  0\n}\n";
        let mut state = ServerState::default();

        refresh_program_cache(&mut state, uri, valid_source);
        assert!(
            state.parsed_programs.contains_key(uri),
            "valid source should populate parsed program cache",
        );
        assert!(
            state.open_document_symbols.contains_key(uri),
            "valid source should populate open-document symbol cache",
        );

        refresh_program_cache(&mut state, uri, invalid_source);
        assert!(
            !state.parsed_programs.contains_key(uri),
            "invalid source should evict parsed program cache entry",
        );
        assert!(
            !state.open_document_symbols.contains_key(uri),
            "invalid source should evict open-document symbol cache entry",
        );
    }

    #[test]
    fn collect_function_symbols_emits_stable_ids() {
        let path = PathBuf::from("/tmp/lsp_symbol_ids.ai");
        let source = "fn alpha() -> Int {\n  1\n}\n";
        let program_one = parse_source(&path, source).expect("source should parse");
        let program_two = parse_source(&path, source).expect("source should parse");

        let symbols_one = collect_function_symbols(&program_one, None);
        let symbols_two = collect_function_symbols(&program_two, None);
        let id_one = symbols_one
            .first()
            .map(|symbol| symbol.id.clone())
            .expect("expected first symbol");
        let id_two = symbols_two
            .first()
            .map(|symbol| symbol.id.clone())
            .expect("expected first symbol");
        assert_eq!(
            id_one, id_two,
            "symbol ids should be deterministic for equivalent source snapshots",
        );
    }

    #[test]
    fn refresh_program_cache_invalidates_open_document_dependents() {
        let dependency_uri = "file:///tmp/lsp_dep_a.ai";
        let dependent_uri = "file:///tmp/lsp_dep_b.ai";
        let dependency_source = "fn alpha() -> Int {\n  1\n}\n";
        let dependent_source = "fn beta() -> Int {\n  alpha()\n}\n";
        let invalid_dependency_source = "fn alpha( -> Int {\n  1\n}\n";
        let mut state = ServerState::default();

        refresh_program_cache(&mut state, dependency_uri, dependency_source);
        refresh_program_cache(&mut state, dependent_uri, dependent_source);
        update_open_document_import_edges(
            &mut state,
            dependent_uri,
            "import \"./lsp_dep_a.ai\"\n",
        );
        assert!(
            state.parsed_programs.contains_key(dependency_uri)
                && state.parsed_programs.contains_key(dependent_uri),
            "initial valid refresh should populate both dependency and dependent cache entries",
        );
        assert!(
            state.open_document_symbols.contains_key(dependent_uri),
            "dependent symbol cache should be populated before dependency invalidation",
        );

        refresh_program_cache(&mut state, dependency_uri, invalid_dependency_source);
        assert!(
            !state.parsed_programs.contains_key(dependency_uri),
            "invalid dependency refresh should evict dependency parse cache",
        );
        assert!(
            !state.parsed_programs.contains_key(dependent_uri),
            "dependency refresh should invalidate dependent parse cache entries",
        );
        assert!(
            !state.open_document_symbols.contains_key(dependent_uri),
            "dependency refresh should invalidate dependent symbol cache entries",
        );
    }

    #[test]
    fn callsite_hit_collection_ignores_non_callee_identifiers() {
        let path = PathBuf::from("/tmp/lsp_callsite_hits.ai");
        let source = "fn helper() -> Int {\n  1\n}\n\nfn apply(v: Int) -> Int {\n  v\n}\n\nfn one() -> Int {\n  helper();\n  apply(helper)\n}\n";
        let program = parse_source(&path, source).expect("source should parse");

        let hits = collect_identifier_hits_by_name(&program, &path, "helper", None);
        assert_eq!(
            hits.len(),
            1,
            "function callsite collection should only include callee identifiers",
        );
        assert_eq!(
            hits[0].span.start_line,
            10,
            "only direct `helper()` callee should be collected",
        );
    }

    #[test]
    fn request_deadline_zero_budget_expires_immediately() {
        let deadline = RequestDeadline::new(0);
        assert!(deadline.is_expired(), "zero budget should be treated as expired");
    }

    #[test]
    fn parse_env_bool_handles_common_false_values() {
        assert!(!parse_env_bool("false"));
        assert!(!parse_env_bool("FALSE"));
        assert!(!parse_env_bool("0"));
        assert!(!parse_env_bool("off"));
        assert!(!parse_env_bool("no"));
        assert!(parse_env_bool("true"));
        assert!(parse_env_bool("1"));
        assert!(parse_env_bool("yes"));
    }

    #[test]
    fn deadline_aware_semantic_walks_can_short_circuit() {
        let path = PathBuf::from("/tmp/lsp_deadline_semantic_walk.ai");
        let source = "fn alpha() -> Int {\n  beta()\n}\n\nfn beta() -> Int {\n  1\n}\n";
        let program = parse_source(&path, source).expect("source should parse");
        let deadline = RequestDeadline::new(0);

        let symbols = collect_function_symbols(&program, Some(&deadline));
        assert!(
            symbols.is_empty(),
            "symbol collection should short-circuit immediately for expired deadlines"
        );

        let identifier = find_identifier_at_position(&program, &path, 1, 2, Some(&deadline));
        assert!(
            identifier.is_none(),
            "identifier lookup should short-circuit immediately for expired deadlines"
        );

        let hits = collect_identifier_hits_by_name(&program, &path, "beta", Some(&deadline));
        assert!(
            hits.is_empty(),
            "identifier hit collection should short-circuit immediately for expired deadlines"
        );
    }

    #[test]
    fn load_cached_program_skips_parse_when_deadline_is_expired() {
        let state = ServerState::default();
        let path = PathBuf::from("/tmp/lsp_deadline_parse_skip.ai");
        let source = "fn alpha() -> Int { 1 }\n";
        let deadline = RequestDeadline::new(0);

        let program = load_cached_program(
            &state,
            "file:///tmp/lsp_deadline_parse_skip.ai",
            &path,
            source,
            Some(&deadline),
        );
        assert!(
            program.is_none(),
            "expired deadline should skip on-demand parse for uncached documents"
        );
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
    fn definition_resolves_to_unopened_workspace_file_declaration() {
        let root = make_temp_workspace("ailang_lsp_definition_unopened");
        let file_a = root.join("a.ai");
        let file_b = root.join("b.ai");
        let policy_file = root.join("ailang.toml");
        let source_a = "fn helper() -> Int {\n  1\n}\n";
        let source_b = "fn main() -> Int {\n  helper()\n}\n";
        fs::write(&policy_file, "name = \"test\"\n").expect("policy marker should be written");
        fs::write(&file_a, source_a).expect("source a should be written");
        fs::write(&file_b, source_b).expect("source b should be written");
        let uri_a = Url::from_file_path(&file_a)
            .expect("file a uri")
            .to_string();
        let uri_b = Url::from_file_path(&file_b)
            .expect("file b uri")
            .to_string();

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
                    "uri": uri_b,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source_b
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 18,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {"uri": uri_b},
                "position": {"line": 1, "character": 2}
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
            .find(|msg| msg.get("id") == Some(&json!(18)))
            .expect("definition response should exist");
        assert_eq!(
            definition_response
                .get("result")
                .and_then(|result| result.get("uri"))
                .and_then(Value::as_str),
            Some(uri_a.as_str()),
            "definition should resolve to helper declaration in unopened workspace file",
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn hover_resolves_signature_from_unopened_workspace_file_declaration() {
        let root = make_temp_workspace("ailang_lsp_hover_unopened");
        let file_a = root.join("a.ai");
        let file_b = root.join("b.ai");
        let policy_file = root.join("ailang.toml");
        let source_a = "fn helper() -> Int {\n  1\n}\n";
        let source_b = "fn main() -> Int {\n  helper()\n}\n";
        fs::write(&policy_file, "name = \"test\"\n").expect("policy marker should be written");
        fs::write(&file_a, source_a).expect("source a should be written");
        fs::write(&file_b, source_b).expect("source b should be written");
        let uri_b = Url::from_file_path(&file_b)
            .expect("file b uri")
            .to_string();

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
                    "uri": uri_b,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source_b
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 19,
            "method": "textDocument/hover",
            "params": {
                "textDocument": {"uri": uri_b},
                "position": {"line": 1, "character": 2}
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
            .find(|msg| msg.get("id") == Some(&json!(19)))
            .expect("hover response should exist");
        let contents = hover_response
            .get("result")
            .and_then(|result| result.get("contents"))
            .and_then(|contents| contents.get("value"))
            .and_then(Value::as_str)
            .unwrap_or("");
        assert!(
            contents.contains("fn helper() -> Int"),
            "hover should resolve helper signature from unopened workspace file",
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn definition_returns_null_when_workspace_declaration_is_ambiguous() {
        let root = make_temp_workspace("ailang_lsp_definition_ambiguous");
        let file_a = root.join("a.ai");
        let file_b = root.join("b.ai");
        let file_c = root.join("c.ai");
        let policy_file = root.join("ailang.toml");
        let source_a = "fn helper() -> Int {\n  1\n}\n";
        let source_b = "fn main() -> Int {\n  helper()\n}\n";
        let source_c = "fn helper() -> Int {\n  2\n}\n";
        fs::write(&policy_file, "name = \"test\"\n").expect("policy marker should be written");
        fs::write(&file_a, source_a).expect("source a should be written");
        fs::write(&file_b, source_b).expect("source b should be written");
        fs::write(&file_c, source_c).expect("source c should be written");
        let uri_b = Url::from_file_path(&file_b)
            .expect("file b uri")
            .to_string();

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
                    "uri": uri_b,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source_b
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 20,
            "method": "textDocument/definition",
            "params": {
                "textDocument": {"uri": uri_b},
                "position": {"line": 1, "character": 2}
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
            .find(|msg| msg.get("id") == Some(&json!(20)))
            .expect("definition response should exist");
        assert!(
            definition_response
                .get("result")
                .map(Value::is_null)
                .unwrap_or(false),
            "ambiguous declarations should return null definition",
        );

        let _ = fs::remove_dir_all(root);
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
    fn references_include_hits_from_multiple_open_documents() {
        let uri_a = "file:///tmp/lsp_refs_multi_a.ai";
        let uri_b = "file:///tmp/lsp_refs_multi_b.ai";
        let source_a = "fn helper() -> Int {\n  1\n}\n\nfn one() -> Int {\n  helper()\n}\n";
        let source_b = "fn two() -> Int {\n  helper()\n}\n";
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
                    "uri": uri_a,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source_a
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri_b,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source_b
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 12,
            "method": "textDocument/references",
            "params": {
                "textDocument": {"uri": uri_b},
                "position": {"line": 1, "character": 2},
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
            .find(|msg| msg.get("id") == Some(&json!(12)))
            .expect("references response should exist");
        let references = references_response
            .get("result")
            .and_then(Value::as_array)
            .expect("references result should be an array");
        let uris = references
            .iter()
            .filter_map(|location| location.get("uri").and_then(Value::as_str))
            .collect::<Vec<_>>();
        assert!(
            uris.contains(&uri_a),
            "references should include locations from first open document",
        );
        assert!(
            uris.contains(&uri_b),
            "references should include locations from second open document",
        );
    }

    #[test]
    fn references_include_hits_from_unopened_workspace_files() {
        let root = make_temp_workspace("ailang_lsp_refs_unopened");
        let file_a = root.join("a.ai");
        let file_b = root.join("b.ai");
        let policy_file = root.join("ailang.toml");
        let source_a = "fn helper() -> Int {\n  1\n}\n\nfn one() -> Int {\n  helper()\n}\n";
        let source_b = "fn two() -> Int {\n  helper()\n}\n";
        fs::write(&policy_file, "name = \"test\"\n").expect("policy marker should be written");
        fs::write(&file_a, source_a).expect("source a should be written");
        fs::write(&file_b, source_b).expect("source b should be written");
        let uri_a = Url::from_file_path(&file_a)
            .expect("file a uri")
            .to_string();
        let uri_b = Url::from_file_path(&file_b)
            .expect("file b uri")
            .to_string();

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
                    "uri": uri_b,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source_b
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 15,
            "method": "textDocument/references",
            "params": {
                "textDocument": {"uri": uri_b},
                "position": {"line": 1, "character": 2},
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
            .find(|msg| msg.get("id") == Some(&json!(15)))
            .expect("references response should exist");
        let references = references_response
            .get("result")
            .and_then(Value::as_array)
            .expect("references result should be an array");
        let uris = references
            .iter()
            .filter_map(|location| location.get("uri").and_then(Value::as_str))
            .collect::<Vec<_>>();
        assert!(
            uris.contains(&uri_a.as_str()),
            "references should include hits from unopened workspace file",
        );
        assert!(
            uris.contains(&uri_b.as_str()),
            "references should include hits from active open file",
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn implementation_returns_declaration_location_for_call_identifier() {
        let uri = "file:///tmp/lsp_implementation.ai";
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
            "id": 6,
            "method": "textDocument/implementation",
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
        let implementation_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(6)))
            .expect("implementation response should exist");
        let locations = implementation_response
            .get("result")
            .and_then(Value::as_array)
            .expect("implementation result should be an array");
        assert_eq!(locations.len(), 1, "implementation should resolve to one declaration location");
        assert_eq!(
            locations[0]
                .get("range")
                .and_then(|range| range.get("start"))
                .and_then(|start| start.get("line"))
                .and_then(Value::as_u64),
            Some(0),
            "implementation should resolve to helper declaration line",
        );
    }

    #[test]
    fn completion_returns_keywords_and_function_symbols() {
        let uri = "file:///tmp/lsp_completion.ai";
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
            "id": 8,
            "method": "textDocument/completion",
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
        let completion_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(8)))
            .expect("completion response should exist");
        let items = completion_response
            .get("result")
            .and_then(Value::as_array)
            .expect("completion result should be an array");
        let labels = items
            .iter()
            .filter_map(|item| item.get("label").and_then(Value::as_str))
            .collect::<Vec<_>>();
        assert!(
            labels.contains(&"helper"),
            "completion should include function symbols from the document",
        );
        assert!(
            labels.contains(&"let"),
            "completion should include keyword items",
        );
    }

    #[test]
    fn prepare_rename_returns_range_and_placeholder_for_call_identifier() {
        let uri = "file:///tmp/lsp_prepare_rename.ai";
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
            "id": 9,
            "method": "textDocument/prepareRename",
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
        let prepare_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(9)))
            .expect("prepareRename response should exist");
        assert_eq!(
            prepare_response
                .get("result")
                .and_then(|result| result.get("placeholder"))
                .and_then(Value::as_str),
            Some("helper"),
            "prepareRename should expose function name placeholder",
        );
        assert_eq!(
            prepare_response
                .get("result")
                .and_then(|result| result.get("range"))
                .and_then(|range| range.get("start"))
                .and_then(|start| start.get("line"))
                .and_then(Value::as_u64),
            Some(5),
            "prepareRename should point to helper call-site range",
        );
    }

    #[test]
    fn rename_returns_workspace_edit_for_declaration_and_call_sites() {
        let uri = "file:///tmp/lsp_rename.ai";
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
            "id": 10,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {"uri": uri},
                "position": {"line": 5, "character": 2},
                "newName": "assist"
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
        if let Some(publish) = messages
            .iter()
            .find(|msg| msg.get("method") == Some(&json!("textDocument/publishDiagnostics")))
        {
            eprintln!(
                "rename precision diagnostics: {}",
                publish
                    .get("params")
                    .and_then(|params| params.get("diagnostics"))
                    .and_then(Value::as_array)
                    .map(|items| items.len())
                    .unwrap_or(0)
            );
        }
        let rename_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(10)))
            .expect("rename response should exist");
        let edits = rename_response
            .get("result")
            .and_then(|result| result.get("changes"))
            .and_then(|changes| changes.get(uri))
            .and_then(Value::as_array)
            .expect("rename response should include changes for current uri");
        assert_eq!(edits.len(), 3, "rename should include declaration and both call sites");
        assert!(
            edits.iter().all(|edit| edit.get("newText").and_then(Value::as_str) == Some("assist")),
            "all rename edits should apply the requested new name",
        );
    }

    #[test]
    fn rename_returns_workspace_edits_for_multiple_open_documents() {
        let uri_a = "file:///tmp/lsp_rename_multi_a.ai";
        let uri_b = "file:///tmp/lsp_rename_multi_b.ai";
        let source_a = "fn helper() -> Int {\n  1\n}\n\nfn one() -> Int {\n  helper()\n}\n";
        let source_b = "fn two() -> Int {\n  helper()\n}\n";
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
                    "uri": uri_a,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source_a
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri_b,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source_b
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 13,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {"uri": uri_b},
                "position": {"line": 1, "character": 2},
                "newName": "assist"
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
        let rename_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(13)))
            .expect("rename response should exist");
        let changes = rename_response
            .get("result")
            .and_then(|result| result.get("changes"))
            .expect("rename result should include changes object");
        let edits_a = changes
            .get(uri_a)
            .and_then(Value::as_array)
            .expect("rename should include edits for first document");
        let edits_b = changes
            .get(uri_b)
            .and_then(Value::as_array)
            .expect("rename should include edits for second document");
        assert!(
            edits_a.len() >= 2,
            "first document should include declaration and call-site edits",
        );
        assert_eq!(
            edits_b.len(),
            1,
            "second document should include one call-site edit",
        );
        assert!(
            edits_a
                .iter()
                .chain(edits_b.iter())
                .all(|edit| edit.get("newText").and_then(Value::as_str) == Some("assist")),
            "all workspace rename edits should apply requested name",
        );
    }

    #[test]
    fn rename_returns_edits_for_unopened_workspace_files() {
        let root = make_temp_workspace("ailang_lsp_rename_unopened");
        let file_a = root.join("a.ai");
        let file_b = root.join("b.ai");
        let policy_file = root.join("ailang.toml");
        let source_a = "fn helper() -> Int {\n  1\n}\n\nfn one() -> Int {\n  helper()\n}\n";
        let source_b = "fn two() -> Int {\n  helper()\n}\n";
        fs::write(&policy_file, "name = \"test\"\n").expect("policy marker should be written");
        fs::write(&file_a, source_a).expect("source a should be written");
        fs::write(&file_b, source_b).expect("source b should be written");
        let uri_a = Url::from_file_path(&file_a)
            .expect("file a uri")
            .to_string();
        let uri_b = Url::from_file_path(&file_b)
            .expect("file b uri")
            .to_string();

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
                    "uri": uri_b,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source_b
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 16,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {"uri": uri_b},
                "position": {"line": 1, "character": 2},
                "newName": "assist"
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
        let rename_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(16)))
            .expect("rename response should exist");
        let changes = rename_response
            .get("result")
            .and_then(|result| result.get("changes"))
            .expect("rename result should include changes object");
        assert!(
            changes.get(&uri_a).is_some(),
            "rename should include edits for unopened workspace file",
        );
        assert!(
            changes.get(&uri_b).is_some(),
            "rename should include edits for active open file",
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rename_returns_empty_changes_when_workspace_declaration_is_ambiguous() {
        let root = make_temp_workspace("ailang_lsp_rename_ambiguous");
        let file_a = root.join("a.ai");
        let file_b = root.join("b.ai");
        let file_c = root.join("c.ai");
        let policy_file = root.join("ailang.toml");
        let source_a = "fn helper() -> Int {\n  1\n}\n";
        let source_b = "fn main() -> Int {\n  helper()\n}\n";
        let source_c = "fn helper() -> Int {\n  2\n}\n";
        fs::write(&policy_file, "name = \"test\"\n").expect("policy marker should be written");
        fs::write(&file_a, source_a).expect("source a should be written");
        fs::write(&file_b, source_b).expect("source b should be written");
        fs::write(&file_c, source_c).expect("source c should be written");
        let uri_b = Url::from_file_path(&file_b)
            .expect("file b uri")
            .to_string();

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
                    "uri": uri_b,
                    "languageId": "ailang",
                    "version": 1,
                    "text": source_b
                }
            }
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 21,
            "method": "textDocument/rename",
            "params": {
                "textDocument": {"uri": uri_b},
                "position": {"line": 1, "character": 2},
                "newName": "assist"
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
        let rename_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(21)))
            .expect("rename response should exist");
        let edits = rename_response
            .get("result")
            .and_then(|result| result.get("changes"))
            .and_then(|changes| changes.get(&uri_b))
            .and_then(Value::as_array)
            .expect("rename result should include current-uri changes array");
        assert!(
            edits.is_empty(),
            "ambiguous declarations should prevent rename edits",
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn code_action_returns_security_quickfix_for_untrusted_sink_diagnostic() {
        let uri = "file:///tmp/lsp_code_action.ai";
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {},
        })));
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 11,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 0, "character": 5}
                },
                "context": {
                    "diagnostics": [
                        {
                            "code": "E1002",
                            "message": "Untrusted data cannot flow into sink SqlQuery.",
                            "range": {
                                "start": {"line": 0, "character": 0},
                                "end": {"line": 0, "character": 5}
                            }
                        }
                    ]
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
        let code_action_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(11)))
            .expect("codeAction response should exist");
        let actions = code_action_response
            .get("result")
            .and_then(Value::as_array)
            .expect("codeAction result should be an array");
        assert!(
            actions.iter().any(|action| {
                action
                    .get("title")
                    .and_then(Value::as_str)
                    .map(|title| title.contains("validate/sanitize"))
                    .unwrap_or(false)
            }),
            "code actions should include a validate/sanitize quickfix for E1002",
        );
    }

    #[test]
    fn code_action_can_emit_validate_edit_for_untrusted_diagnostic() {
        let uri = "file:///tmp/lsp_code_action_validate.ai";
        let source = "fn main() -> Int {\n  input\n}\n";
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
            "id": 17,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {
                    "start": {"line": 1, "character": 2},
                    "end": {"line": 1, "character": 7}
                },
                "context": {
                    "diagnostics": [
                        {
                            "code": "E1002",
                            "message": "Untrusted data cannot flow into sink.",
                            "range": {
                                "start": {"line": 1, "character": 2},
                                "end": {"line": 1, "character": 7}
                            }
                        }
                    ]
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
        let code_action_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(17)))
            .expect("codeAction response should exist");
        let actions = code_action_response
            .get("result")
            .and_then(Value::as_array)
            .expect("codeAction result should be an array");
        let validate_action = actions
            .iter()
            .find(|action| {
                action
                    .get("title")
                    .and_then(Value::as_str)
                    .map(|title| title.contains("validate"))
                    .unwrap_or(false)
            })
            .expect("expected validate quickfix action");
        assert_eq!(
            validate_action
                .get("edit")
                .and_then(|edit| edit.get("changes"))
                .and_then(|changes| changes.get(uri))
                .and_then(Value::as_array)
                .and_then(|edits| edits.first())
                .and_then(|edit| edit.get("newText"))
                .and_then(Value::as_str),
            Some("validate(input)?"),
            "validate quickfix should wrap selected diagnostic range text",
        );
    }

    #[test]
    fn code_action_can_emit_redact_edit_for_secret_diagnostic() {
        let uri = "file:///tmp/lsp_code_action_redact.ai";
        let source = "fn main() -> Int {\n  token\n}\n";
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
            "id": 14,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {
                    "start": {"line": 1, "character": 2},
                    "end": {"line": 1, "character": 7}
                },
                "context": {
                    "diagnostics": [
                        {
                            "code": "E1003",
                            "message": "Secret value cannot be logged.",
                            "range": {
                                "start": {"line": 1, "character": 2},
                                "end": {"line": 1, "character": 7}
                            }
                        }
                    ]
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
        let code_action_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(14)))
            .expect("codeAction response should exist");
        let actions = code_action_response
            .get("result")
            .and_then(Value::as_array)
            .expect("codeAction result should be an array");
        let redact_action = actions
            .iter()
            .find(|action| {
                action
                    .get("title")
                    .and_then(Value::as_str)
                    .map(|title| title.contains("redact"))
                    .unwrap_or(false)
            })
            .expect("expected redact quickfix action");
        assert_eq!(
            redact_action
                .get("edit")
                .and_then(|edit| edit.get("changes"))
                .and_then(|changes| changes.get(uri))
                .and_then(Value::as_array)
                .and_then(|edits| edits.first())
                .and_then(|edit| edit.get("newText"))
                .and_then(Value::as_str),
            Some("redact(token)"),
            "redact quickfix should wrap selected diagnostic range text",
        );
    }

    #[test]
    fn code_action_can_emit_missing_effect_declaration_edit() {
        let uri = "file:///tmp/lsp_code_action_effect.ai";
        let source = "fn handler() -> Int {\n  call()\n}\n";
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
            "id": 22,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {
                    "start": {"line": 1, "character": 2},
                    "end": {"line": 1, "character": 6}
                },
                "context": {
                    "diagnostics": [
                        {
                            "code": "E2001",
                            "message": "Function uses effect `net` but does not declare it.",
                            "range": {
                                "start": {"line": 1, "character": 2},
                                "end": {"line": 1, "character": 6}
                            }
                        }
                    ]
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
        let code_action_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(22)))
            .expect("codeAction response should exist");
        let actions = code_action_response
            .get("result")
            .and_then(Value::as_array)
            .expect("codeAction result should be an array");
        let effect_action = actions
            .iter()
            .find(|action| {
                action
                    .get("title")
                    .and_then(Value::as_str)
                    .map(|title| title.contains("effect"))
                    .unwrap_or(false)
            })
            .expect("expected missing-effect quickfix action");
        assert_eq!(
            effect_action
                .get("edit")
                .and_then(|edit| edit.get("changes"))
                .and_then(|changes| changes.get(uri))
                .and_then(Value::as_array)
                .and_then(|edits| edits.first())
                .and_then(|edit| edit.get("newText"))
                .and_then(Value::as_str),
            Some(" effects { net }"),
            "effect quickfix should insert missing effect declaration",
        );
    }

    #[test]
    fn code_action_can_append_missing_effect_to_existing_effects_block() {
        let uri = "file:///tmp/lsp_code_action_effect_append.ai";
        let source = "fn handler() effects { log } -> Int {\n  call()\n}\n";
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
            "id": 23,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {
                    "start": {"line": 1, "character": 2},
                    "end": {"line": 1, "character": 6}
                },
                "context": {
                    "diagnostics": [
                        {
                            "code": "E2001",
                            "message": "Function uses effect `net` but does not declare it.",
                            "range": {
                                "start": {"line": 1, "character": 2},
                                "end": {"line": 1, "character": 6}
                            }
                        }
                    ]
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
        let code_action_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(23)))
            .expect("codeAction response should exist");
        let actions = code_action_response
            .get("result")
            .and_then(Value::as_array)
            .expect("codeAction result should be an array");
        let effect_action = actions
            .iter()
            .find(|action| {
                action
                    .get("title")
                    .and_then(Value::as_str)
                    .map(|title| title.contains("effect"))
                    .unwrap_or(false)
            })
            .expect("expected missing-effect quickfix action");
        assert_eq!(
            effect_action
                .get("edit")
                .and_then(|edit| edit.get("changes"))
                .and_then(|changes| changes.get(uri))
                .and_then(Value::as_array)
                .and_then(|edits| edits.first())
                .and_then(|edit| edit.get("newText"))
                .and_then(Value::as_str),
            Some(", net"),
            "effect quickfix should append missing effect inside existing declaration",
        );
    }

    #[test]
    fn code_action_omits_missing_effect_quickfix_when_already_declared() {
        let uri = "file:///tmp/lsp_code_action_effect_noop.ai";
        let source = "fn handler() effects { net } -> Int {\n  call()\n}\n";
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
            "id": 24,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {
                    "start": {"line": 1, "character": 2},
                    "end": {"line": 1, "character": 6}
                },
                "context": {
                    "diagnostics": [
                        {
                            "code": "E2001",
                            "message": "Function uses effect `net` but does not declare it.",
                            "range": {
                                "start": {"line": 1, "character": 2},
                                "end": {"line": 1, "character": 6}
                            }
                        }
                    ]
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
        let code_action_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(24)))
            .expect("codeAction response should exist");
        let actions = code_action_response
            .get("result")
            .and_then(Value::as_array)
            .expect("codeAction result should be an array");
        let has_effect_action = actions.iter().any(|action| {
            action
                .get("title")
                .and_then(Value::as_str)
                .map(|title| title.contains("effect"))
                .unwrap_or(false)
        });
        assert!(
            !has_effect_action,
            "effect quickfix should be omitted when declaration already contains the missing effect",
        );
    }

    #[test]
    fn code_action_can_emit_missing_effect_edit_for_multiline_signature() {
        let uri = "file:///tmp/lsp_code_action_effect_multiline.ai";
        let source = "fn handler(\n  value: Int\n) -> Int {\n  call()\n}\n";
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
            "id": 25,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {
                    "start": {"line": 3, "character": 2},
                    "end": {"line": 3, "character": 6}
                },
                "context": {
                    "diagnostics": [
                        {
                            "code": "E2001",
                            "message": "Function uses effect `net` but does not declare it.",
                            "range": {
                                "start": {"line": 3, "character": 2},
                                "end": {"line": 3, "character": 6}
                            }
                        }
                    ]
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
        let code_action_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(25)))
            .expect("codeAction response should exist");
        let actions = code_action_response
            .get("result")
            .and_then(Value::as_array)
            .expect("codeAction result should be an array");
        let effect_action = actions
            .iter()
            .find(|action| {
                action
                    .get("title")
                    .and_then(Value::as_str)
                    .map(|title| title.contains("effect"))
                    .unwrap_or(false)
            })
            .expect("expected missing-effect quickfix action");
        assert_eq!(
            effect_action
                .get("edit")
                .and_then(|edit| edit.get("changes"))
                .and_then(|changes| changes.get(uri))
                .and_then(Value::as_array)
                .and_then(|edits| edits.first())
                .and_then(|edit| edit.get("newText"))
                .and_then(Value::as_str),
            Some(" effects { net }"),
            "effect quickfix should insert missing effect declaration for multiline signatures",
        );
    }

    #[test]
    fn code_action_can_append_missing_effect_for_multiline_effects_signature() {
        let uri = "file:///tmp/lsp_code_action_effect_multiline_append.ai";
        let source = "fn handler(\n  value: Int\n) effects { log } -> Int {\n  call()\n}\n";
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
            "id": 26,
            "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {
                    "start": {"line": 3, "character": 2},
                    "end": {"line": 3, "character": 6}
                },
                "context": {
                    "diagnostics": [
                        {
                            "code": "E2001",
                            "message": "Function uses effect `net` but does not declare it.",
                            "range": {
                                "start": {"line": 3, "character": 2},
                                "end": {"line": 3, "character": 6}
                            }
                        }
                    ]
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
        let code_action_response = messages
            .iter()
            .find(|msg| msg.get("id") == Some(&json!(26)))
            .expect("codeAction response should exist");
        let actions = code_action_response
            .get("result")
            .and_then(Value::as_array)
            .expect("codeAction result should be an array");
        let effect_action = actions
            .iter()
            .find(|action| {
                action
                    .get("title")
                    .and_then(Value::as_str)
                    .map(|title| title.contains("effect"))
                    .unwrap_or(false)
            })
            .expect("expected missing-effect quickfix action");
        assert_eq!(
            effect_action
                .get("edit")
                .and_then(|edit| edit.get("changes"))
                .and_then(|changes| changes.get(uri))
                .and_then(Value::as_array)
                .and_then(|edits| edits.first())
                .and_then(|edit| edit.get("newText"))
                .and_then(Value::as_str),
            Some(", net"),
            "effect quickfix should append missing effect in multiline signatures with existing effects",
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
