use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};

const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_REQUEST: i64 = -32600;

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
    loop {
        let Some(message) = read_message(reader)? else {
            return Ok(());
        };
        if handle_message(&message, writer)? {
            return Ok(());
        }
    }
}

fn handle_message<W: Write>(message: &Value, writer: &mut W) -> io::Result<bool> {
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
                            "textDocumentSync": 1
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

        let mut output_reader = BufReader::new(Cursor::new(output));
        let first = read_message(&mut output_reader)
            .expect("first message should parse")
            .expect("first message should exist");
        assert_eq!(first.get("id"), Some(&json!(1)));
        assert!(
            first
                .get("result")
                .and_then(|result| result.get("capabilities"))
                .is_some(),
            "initialize response should include capabilities"
        );

        let second = read_message(&mut output_reader)
            .expect("second message should parse")
            .expect("second message should exist");
        assert_eq!(second.get("id"), Some(&json!(2)));
        assert_eq!(second.get("result"), Some(&Value::Null));

        assert!(
            read_message(&mut output_reader)
                .expect("end of output should parse")
                .is_none(),
            "no additional responses expected"
        );
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

        let mut output_reader = BufReader::new(Cursor::new(output));
        let response = read_message(&mut output_reader)
            .expect("response should parse")
            .expect("response should exist");
        assert_eq!(response.get("id"), Some(&json!(7)));
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
