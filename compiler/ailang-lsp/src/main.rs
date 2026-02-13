use ailang_core::{analyze_program, parse_source, Diagnostic as CoreDiagnostic, Severity as CoreSeverity};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use url::Url;

const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_REQUEST: i64 = -32600;

#[derive(Default)]
struct ServerState {
    documents: HashMap<String, String>,
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
    fn run_stdio_returns_method_not_found_for_unknown_requests() {
        let mut input = Vec::new();
        input.extend(encode_message(json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "textDocument/definition",
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
