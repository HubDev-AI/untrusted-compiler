use ailang_core::{analyze_program, parse_allow_annotations, parse_source, Diagnostic};
use std::path::Path;

fn find_diag<'a>(diagnostics: &'a [Diagnostic], code: &str) -> &'a Diagnostic {
    diagnostics
        .iter()
        .find(|diag| diag.code == code)
        .unwrap_or_else(|| panic!("expected diagnostic code `{code}`, got {:?}", diagnostics))
}

#[test]
fn sink_untrusted_sql_diagnostic_has_security_taint_tags() {
    let source = r#"
fn bad(cap: DbCap, query: Untrusted<String>) effects { db.write } -> Int {
  db.exec(cap, query);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E1002");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "taint"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn sink_secret_log_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad(token: Secret<String>) effects { log } -> Int {
  log.info(token);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E1003");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn capability_diagnostic_has_security_capability_tags() {
    let source = r#"
fn bad() effects { db.write } -> Int {
  db_write();
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E2003");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "capability"));
}

#[test]
fn policy_effect_diagnostic_has_policy_effect_tags() {
    let source = r#"
fn bad() effects { secrets.reveal } -> Int {
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E2002");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "policy"));
    assert!(diag.tags.iter().any(|tag| tag == "effects"));
}

#[test]
fn strict_json_schema_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad() effects { net } -> Int {
  res.json(1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4004");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn req_json_schema_gate_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad() effects { net } -> Int {
  req.json(123);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn json_encode_schema_argument_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad() -> Int {
  json.encode(1, 2);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "json.encode schema argument is invalid")
        .expect("expected json.encode schema diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn json_encode_non_schema_argument_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad() -> Int {
  json.encode("schema", 2);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "json.encode schema argument must be `Schema<_>`")
        .expect("expected json.encode non-schema diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn json_decode_schema_argument_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad() -> Int {
  json.decode(1, 2, 3);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "json.decode schema argument is invalid")
        .expect("expected json.decode schema diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn json_decode_non_schema_argument_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad(ctx: Ctx, raw: Untrusted<Bytes>) -> Int {
  json.decode(ctx, "schema", raw);
  0
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "json.decode schema argument must be `Schema<_>`")
        .expect("expected json.decode non-schema diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn json_decode_context_argument_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad(schema: Schema<Int>, raw: Untrusted<Bytes>) -> Int {
  json.decode(1, schema, raw);
  0
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "json.decode first argument must be `Ctx`")
        .expect("expected json.decode context diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn json_decode_raw_argument_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad(ctx: Ctx, schema: Schema<Int>) -> Int {
  json.decode(ctx, schema, 3);
  0
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "json.decode raw argument must be `Untrusted<Bytes>`")
        .expect("expected json.decode raw diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn res_html_sink_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad() effects { net } -> Int {
  res.html(1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn set_header_sink_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad() effects { net } -> Int {
  res.setHeader(1, 2);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn headers_name_gate_diagnostic_has_security_tag() {
    let source = r#"
fn bad() effects { net } -> Int {
  headers.name(1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn sec_csp_add_signature_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  let csp = sec.csp();
  sec.cspAdd(csp, 1, 2);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "sec.cspAdd directive argument must be `String`")
        .expect("expected sec.cspAdd directive argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn cookie_build_signature_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  cookie.build(1, "value");
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "cookie.build name argument must be `String`")
        .expect("expected cookie.build name argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn req_query_signature_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad() effects { net } -> Int {
  req.query(1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn path_base_signature_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  path.base(1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn req_body_signature_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad() effects { net } -> Int {
  req.body(1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn trust_gate_arity_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad(input: Untrusted<String>) -> Int {
  validate.email(input, input);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn db_sink_shape_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad(cap: DbCap) effects { db.write } -> Int {
  db.exec(cap);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn db_sink_context_type_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad(db: DbCap, query: SqlQuery) effects { db.write } -> Int {
  db.exec(1, db, query);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn db_sink_query_type_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad(db: DbCap) effects { db.write } -> Int {
  db.exec(db, 1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "db sink query argument must be `SqlQuery`")
        .expect("expected db sink query type diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn db_query_one_row_schema_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad(db: DbCap, query: SqlQuery) effects { db.read } -> Int {
  db.queryOne(db, query, 1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "db.queryOne row schema argument is invalid")
        .expect("expected db.queryOne row schema diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn sql_q_template_type_diagnostic_has_security_schema_tags() {
    let source = r#"
fn bad(db: DbCap) effects { db.write } -> Int {
  let query = sql.q(1, 2);
  db.exec(db, query);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "sql.q template argument must be `String`")
        .expect("expected sql.q template argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "schema"));
}

#[test]
fn sql_q_params_secret_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad(secret: Secret<String>) -> Int {
  sql.q("SELECT 1", secret);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "sql.q params argument cannot be `Secret<_>`")
        .expect("expected sql.q secret params diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
}

#[test]
fn sql_q_params_untrusted_diagnostic_has_security_taint_tags() {
    let source = r#"
fn bad(param: Untrusted<String>) -> Int {
  sql.q("SELECT 1", param);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "sql.q params argument cannot be `Untrusted<_>`")
        .expect("expected sql.q untrusted params diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "taint"));
}

#[test]
fn db_tx_shape_diagnostic_has_security_capability_tags() {
    let source = r#"
fn bad(db: DbCap) effects { db.tx } -> Int {
  db.tx(1, db, 2);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "capability"));
}

#[test]
fn db_tx_context_type_diagnostic_has_security_capability_tags() {
    let source = r#"
fn bad(db: DbCap) effects { db.tx } -> Int {
  db.tx(1, db);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "capability"));
}

#[test]
fn net_sink_shape_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad(net: NetCap) effects { net } -> Int {
  httpClient.get(net);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn net_sink_context_type_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad(net: NetCap, url: PublicUrl) effects { net } -> Int {
  httpClient.get(1, net, url);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn net_sink_url_type_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad(net: NetCap) effects { net } -> Int {
  httpClient.get(net, 1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "net sink URL argument must be `PublicUrl`")
        .expect("expected net sink URL type diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn fs_sink_shape_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad(fs: FsCap) effects { fs.write } -> Int {
  fs.write(fs);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn fs_sink_context_type_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad(fs: FsCap, path: PathSafe) effects { fs.read } -> Int {
  fs.read(1, fs, path);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn fs_sink_path_type_diagnostic_has_security_sink_tags() {
    let source = r#"
fn bad(fs: FsCap) effects { fs.read } -> Int {
  fs.read(fs, 1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "fs sink path argument must be `PathSafe`")
        .expect("expected fs sink path type diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "sink"));
}

#[test]
fn secret_source_shape_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad(sec: SecretsCap) effects { secrets.read } -> Int {
  secrets.get(sec);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
}

#[test]
fn secret_source_context_type_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad(sec: SecretsCap) effects { secrets.read } -> Int {
  secrets.get(1, sec, "API_TOKEN");
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
}

#[test]
fn secret_source_name_type_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad(sec: SecretsCap) effects { secrets.read } -> Int {
  secrets.get(sec, 1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "secret source name argument must be `String`")
        .expect("expected secret source name diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
}

#[test]
fn secret_redact_shape_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad() -> Int {
  secrets.redact();
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
}

#[test]
fn secret_redact_value_type_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad() -> Int {
  secrets.redact("token");
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "secret redact argument must be `Secret<_>`")
        .expect("expected secret redact argument type diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
}

#[test]
fn secret_reveal_shape_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad(sec: SecretsCap) effects { secrets.reveal } -> Int {
  secrets.reveal(sec);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
}

#[test]
fn secret_reveal_context_type_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad(sec: SecretsCap, token: Secret<String>) effects { secrets.reveal } -> Int {
  secrets.reveal(1, sec, token);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "secret reveal context argument must be `Ctx`")
        .expect("expected secret reveal context diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
}

#[test]
fn secret_reveal_value_type_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad(sec: SecretsCap, token: String) effects { secrets.reveal } -> Int {
  secrets.reveal(sec, token);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "secret reveal value argument must be `Secret<_>`")
        .expect("expected secret reveal value diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
}

#[test]
fn auth_helper_shape_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  auth.require(1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = find_diag(&diagnostics, "E4001");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_with_detail_secret_value_diagnostic_has_security_secret_tags() {
    let source = r#"
fn bad(secret: Secret<String>) -> Int {
  let base = err.validation("VAL.BAD", "bad");
  err.withDetail(base, "token", secret);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.withDetail value argument cannot be `Secret<_>`")
        .expect("expected err.withDetail secret-value diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "secret"));
}

#[test]
fn err_with_detail_error_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  err.withDetail(1, "token", 2);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.withDetail error argument must be `StdError`")
        .expect("expected err.withDetail error argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_with_path_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  let base = err.validation("VAL.BAD", "bad");
  err.withPath(base, 2);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.withPath path argument must be `String`")
        .expect("expected err.withPath path argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_with_path_error_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  err.withPath(1, "$.field");
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.withPath error argument must be `StdError`")
        .expect("expected err.withPath error argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_with_limit_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  let base = err.validation("VAL.BAD", "bad");
  err.withLimit(base, 2, 3, 4);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.withLimit name argument must be `String`")
        .expect("expected err.withLimit name argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_with_limit_error_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  err.withLimit(1, "limit", 3, 4);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.withLimit error argument must be `StdError`")
        .expect("expected err.withLimit error argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_with_dependency_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  let base = err.validation("VAL.BAD", "bad");
  err.withDependency(base, 2, 3, 4);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.withDependency name argument must be `String`")
        .expect("expected err.withDependency name argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_with_dependency_error_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  err.withDependency(1, "dep", "op", true);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.withDependency error argument must be `StdError`")
        .expect("expected err.withDependency error argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_with_cause_error_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  let cause = err.internal("boom");
  err.withCause(1, cause);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.withCause error argument must be `StdError`")
        .expect("expected err.withCause error argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_with_cause_cause_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  let base = err.validation("VAL.BAD", "bad");
  err.withCause(base, 1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.withCause cause argument must be `StdError`")
        .expect("expected err.withCause cause argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_internal_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  err.internal(1);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.internal message argument must be `String`")
        .expect("expected err.internal message argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_validation_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  err.validation(1, 2);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.validation code argument must be `String`")
        .expect("expected err.validation code argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_auth_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  err.auth(1, 2, 401);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.auth code argument must be `String`")
        .expect("expected err.auth code argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_not_found_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  err.notFound(1, 2);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.notFound code argument must be `String`")
        .expect("expected err.notFound code argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_conflict_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  err.conflict(1, 2);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.conflict code argument must be `String`")
        .expect("expected err.conflict code argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn err_rate_limit_argument_diagnostic_has_security_tag() {
    let source = r#"
fn bad() -> Int {
  err.rateLimit(1, 2, 3);
  1
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let diagnostics = analyze_program(&program).expect_err("analysis should fail");
    let diag = diagnostics
        .iter()
        .find(|diag| diag.message == "err.rateLimit code argument must be `String`")
        .expect("expected err.rateLimit code argument diagnostic");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
}

#[test]
fn allow_annotation_diagnostic_has_security_policy_tags() {
    let source = r#"
@allow(
  policy = "effects.forbid",
  bypass = ["effect.secrets.reveal"],
  reason = "legacy path",
  ticket = "SEC-124",
  expires = "2000-01-01",
)
fn main() -> Int { 1 }
"#;

    let diagnostics = parse_allow_annotations(Path::new("main.ai"), source)
        .expect_err("expired annotation should be rejected");
    let diag = find_diag(&diagnostics, "A7002");
    assert!(diag.tags.iter().any(|tag| tag == "security"));
    assert!(diag.tags.iter().any(|tag| tag == "policy"));
}
