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
