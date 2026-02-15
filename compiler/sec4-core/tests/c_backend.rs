use sec4_core::{
    emit_c_program, emit_runtime_header, emit_runtime_source, lower_program_to_mir, parse_source,
};
use std::path::Path;

#[test]
fn c_backend_emits_minimal_program_for_simple_function() {
    let source = r#"
fn main() -> Int {
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("#include <stdbool.h>"));
    assert!(c.contains("#include <stdint.h>"));
    assert!(c.contains("#include \"sec4_runtime.h\""));
    assert!(c.contains("int main(void);"));
    assert!(c.contains("int main(void) {"));
    assert!(c.contains("bb0:"));
    assert!(c.contains("return sec4_rt_identity_i64(0);"));
}

#[test]
fn c_backend_emits_branch_and_goto_control_flow() {
    let source = r#"
fn flow(x: Int) -> Int {
  if x > 0 {
    1
  } else {
    0
  };
  5
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("if ((x > 0)) goto bb1; else goto bb2;"));
    assert!(c.contains("goto bb3;"));
    assert!(c.contains("return sec4_rt_identity_i64(5);"));
}

#[test]
fn c_backend_routes_bool_returns_through_runtime_identity() {
    let source = r#"
fn truthy(flag: Bool) -> Bool {
  flag
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("bool truthy(bool flag);"));
    assert!(c.contains("return sec4_rt_identity_bool(flag);"));
}

#[test]
fn c_backend_emits_runtime_header_and_source() {
    let header = emit_runtime_header();
    let source = emit_runtime_source();

    assert!(header.contains("#ifndef SEC4_RUNTIME_H"));
    assert!(header.contains("int64_t sec4_rt_identity_i64(int64_t value);"));
    assert!(header.contains("bool sec4_rt_identity_bool(bool value);"));
    assert!(header.contains("int64_t sec4_rt_time_now(void);"));
    assert!(header.contains("void sec4_rt_log_any(int64_t event);"));
    assert!(header.contains("int64_t sec4_rt_log_event(const char *event_name);"));
    assert!(header.contains("int64_t sec4_rt_log_field(const char *key, int64_t value);"));
    assert!(header.contains("int64_t sec4_rt_log_obj(int64_t field);"));
    assert!(header.contains("int64_t sec4_rt_log_str(const char *value);"));
    assert!(header.contains("int64_t sec4_rt_log_i64(int64_t value);"));
    assert!(header.contains("int64_t sec4_rt_log_bool(int64_t value);"));
    assert!(header.contains("int64_t sec4_rt_log_redacted(const char *value);"));
    assert!(header.contains("int64_t sec4_rt_log_attr_redacted(const char *value);"));
    assert!(header.contains("int64_t sec4_rt_log_with_attr(int64_t event, const char *key, int64_t value);"));
    assert!(header.contains("int64_t sec4_rt_log_with_http("));
    assert!(header.contains("int64_t sec4_rt_log_with_error(int64_t event, int64_t error);"));
    assert!(header.contains("int64_t sec4_rt_req_json(int64_t schema);"));
    assert!(
        header.contains("int64_t sec4_rt_json_decode(int64_t ctx, int64_t schema, int64_t raw);")
    );
    assert!(header.contains("int64_t sec4_rt_json_encode(int64_t schema, int64_t value);"));
    assert!(header.contains("int64_t sec4_rt_req_body(int64_t ctx, int64_t req);"));
    assert!(header.contains("int64_t sec4_rt_req_query(const char *name);"));
    assert!(header.contains("int64_t sec4_rt_req_path_param(const char *name);"));
    assert!(header.contains("int64_t sec4_rt_req_header(const char *name);"));
    assert!(header.contains("int64_t sec4_rt_res_json(int64_t schema, int64_t value);"));
    assert!(
        header.contains("int64_t sec4_rt_res_ok(int64_t status, int64_t schema, int64_t value);")
    );
    assert!(header.contains(
        "int64_t sec4_rt_res_ok_meta(int64_t status, int64_t schema, int64_t value, int64_t meta);"
    ));
    assert!(header.contains("int64_t sec4_rt_res_html();"));
    assert!(header.contains("int64_t sec4_rt_res_text(int64_t status, const char *body);"));
    assert!(header.contains("int64_t sec4_rt_set_header(int64_t name, int64_t value);"));
    assert!(header.contains("int64_t sec4_rt_cookie_build(const char *name, const char *value);"));
    assert!(header.contains("int64_t sec4_rt_set_cookie(int64_t cookie);"));
    assert!(header.contains("int64_t sec4_rt_sql_q(const char *query_template, int64_t params);"));
    assert!(header.contains("int64_t sec4_rt_db_exec(int64_t db, int64_t query);"));
    assert!(header.contains("int64_t sec4_rt_db_tx(int64_t db);"));
    assert!(header.contains("int64_t sec4_rt_db_exec_tx(int64_t tx, int64_t query);"));
    assert!(header
        .contains("int64_t sec4_rt_db_query_one(int64_t db, int64_t query, int64_t row_schema);"));
    assert!(header.contains("int64_t sec4_rt_fs_read(int64_t fs, int64_t path);"));
    assert!(header.contains("int64_t sec4_rt_fs_write(int64_t fs, int64_t path, int64_t value);"));
    assert!(header.contains("int64_t sec4_rt_http_get(int64_t net, int64_t url);"));
    assert!(header.contains("int64_t sec4_rt_http_get_internal(int64_t net, int64_t url);"));
    assert!(header.contains("int64_t sec4_rt_secret_get(int64_t secrets_cap, const char *name);"));
    assert!(header.contains("int64_t sec4_rt_secret_redact(int64_t secret_value);"));
    assert!(header.contains("int64_t sec4_rt_secret_reveal(int64_t secrets_cap, int64_t secret_value);"));
    assert!(header.contains("bool sec4_rt_crypto_ct_eq(int64_t left_secret, int64_t right_secret);"));
    assert!(header.contains("int64_t sec4_rt_validate_header_value(int64_t input);"));
    assert!(header.contains("int64_t sec4_rt_validate_email(int64_t input);"));
    assert!(header.contains("int64_t sec4_rt_validate_uuid(int64_t input);"));
    assert!(header.contains("int64_t sec4_rt_validate_int64(int64_t input);"));
    assert!(header.contains("int64_t sec4_rt_validate_non_empty(int64_t input);"));
    assert!(header.contains("int64_t sec4_rt_sanitize_html(int64_t input);"));
    assert!(header.contains("int64_t sec4_rt_url_public(int64_t input);"));
    assert!(header.contains("int64_t sec4_rt_url_internal(int64_t input);"));
    assert!(header.contains("int64_t sec4_rt_path_under(int64_t base, int64_t input);"));
    assert!(header.contains("int64_t sec4_rt_path_base(const char *input);"));
    assert!(header.contains("int64_t sec4_rt_headers_name(const char *input);"));
    assert!(header.contains("int64_t sec4_rt_headers_value(const char *input);"));
    assert!(header.contains("int64_t sec4_rt_http_router(void);"));
    assert!(header.contains("int64_t sec4_rt_http_route_get("));
    assert!(header.contains("int64_t sec4_rt_http_route_post("));
    assert!(header.contains("int64_t sec4_rt_http_serve(int64_t port, int64_t router);"));
    assert!(header.contains("int64_t sec4_rt_with_cors(int64_t router, int64_t cfg);"));
    assert!(header.contains("int64_t sec4_rt_with_security_headers(int64_t router, int64_t cfg);"));
    assert!(header.contains("int64_t sec4_rt_with_csrf(int64_t router, int64_t cfg);"));
    assert!(header.contains("int64_t sec4_rt_with_auth(int64_t router, int64_t cfg);"));
    assert!(header.contains("int64_t sec4_rt_sec_default_headers(void);"));
    assert!(header.contains("int64_t sec4_rt_sec_csp(void);"));
    assert!(header.contains("int64_t sec4_rt_sec_csp_add(int64_t csp, const char *directive, const char *value);"));
    assert!(header.contains("int64_t sec4_rt_cors_from_policy(void);"));
    assert!(header.contains("int64_t sec4_rt_cors_origin(int64_t origin);"));
    assert!(header.contains("int64_t sec4_rt_csrf_from_policy(void);"));
    assert!(header.contains("int64_t sec4_rt_csrf_issue_token(int64_t ctx);"));
    assert!(header.contains("int64_t sec4_rt_auth_from_policy(void);"));
    assert!(header.contains("int64_t sec4_rt_auth_require(int64_t ctx);"));
    assert!(header.contains("int64_t sec4_rt_auth_require_role(int64_t ctx, const char *required_role);"));
    assert!(header.contains("int64_t sec4_rt_err_validation(const char *code, const char *message);"));
    assert!(header.contains("int64_t sec4_rt_err_auth(const char *code, const char *message, int64_t status);"));
    assert!(header.contains("int64_t sec4_rt_err_not_found(const char *code, const char *message);"));
    assert!(header.contains("int64_t sec4_rt_err_conflict(const char *code, const char *message);"));
    assert!(header.contains("int64_t sec4_rt_err_rate_limit(const char *code, const char *message, int64_t limit);"));
    assert!(header.contains("int64_t sec4_rt_err_internal(const char *message);"));
    assert!(header.contains("int64_t sec4_rt_err_with_path(int64_t error, const char *path);"));
    assert!(header.contains("int64_t sec4_rt_err_with_detail(int64_t error, const char *key, int64_t value);"));
    assert!(header.contains("int64_t sec4_rt_err_with_limit(int64_t error, const char *name, int64_t value, int64_t max);"));
    assert!(header.contains("int64_t sec4_rt_err_with_dependency("));
    assert!(header.contains("int64_t sec4_rt_err_with_cause(int64_t error, int64_t cause);"));

    assert!(source.contains("#include \"sec4_runtime.h\""));
    assert!(source.contains("int64_t sec4_rt_identity_i64(int64_t value)"));
    assert!(source.contains("bool sec4_rt_identity_bool(bool value)"));
    assert!(source.contains("int64_t sec4_rt_time_now(void)"));
    assert!(source.contains("void sec4_rt_log_any(int64_t event)"));
    assert!(source.contains("int64_t sec4_rt_log_event(const char *event_name)"));
    assert!(source.contains("int64_t sec4_rt_log_field(const char *key, int64_t value)"));
    assert!(source.contains("int64_t sec4_rt_log_obj(int64_t field)"));
    assert!(source.contains("int64_t sec4_rt_log_str(const char *value)"));
    assert!(source.contains("int64_t sec4_rt_log_i64(int64_t value)"));
    assert!(source.contains("int64_t sec4_rt_log_bool(int64_t value)"));
    assert!(source.contains("int64_t sec4_rt_log_redacted(const char *value)"));
    assert!(source.contains("int64_t sec4_rt_log_attr_redacted(const char *value)"));
    assert!(source.contains("int64_t sec4_rt_log_with_attr(int64_t event, const char *key, int64_t value)"));
    assert!(source.contains("int64_t sec4_rt_log_with_http("));
    assert!(source.contains("int64_t sec4_rt_log_with_error(int64_t event, int64_t error)"));
    assert!(source.contains("int64_t sec4_rt_req_json(int64_t schema)"));
    assert!(
        source.contains("int64_t sec4_rt_json_decode(int64_t ctx, int64_t schema, int64_t raw)")
    );
    assert!(source.contains("int64_t sec4_rt_json_encode(int64_t schema, int64_t value)"));
    assert!(source.contains("int64_t sec4_rt_req_body(int64_t ctx, int64_t req)"));
    assert!(source.contains("int64_t sec4_rt_req_query(const char *name)"));
    assert!(source.contains("int64_t sec4_rt_req_path_param(const char *name)"));
    assert!(source.contains("int64_t sec4_rt_req_header(const char *name)"));
    assert!(source.contains("int64_t sec4_rt_res_json(int64_t schema, int64_t value)"));
    assert!(
        source.contains("int64_t sec4_rt_res_ok(int64_t status, int64_t schema, int64_t value)")
    );
    assert!(source.contains(
        "int64_t sec4_rt_res_ok_meta(int64_t status, int64_t schema, int64_t value, int64_t meta)"
    ));
    assert!(source.contains("int64_t sec4_rt_res_html()"));
    assert!(source.contains("int64_t sec4_rt_res_text(int64_t status, const char *body)"));
    assert!(source.contains("int64_t sec4_rt_set_header(int64_t name, int64_t value)"));
    assert!(source.contains("int64_t sec4_rt_cookie_build(const char *name, const char *value)"));
    assert!(source.contains("int64_t sec4_rt_set_cookie(int64_t cookie)"));
    assert!(source.contains("int64_t sec4_rt_sql_q(const char *query_template, int64_t params)"));
    assert!(source.contains("int64_t sec4_rt_db_exec(int64_t db, int64_t query)"));
    assert!(source.contains("int64_t sec4_rt_db_tx(int64_t db)"));
    assert!(source.contains("int64_t sec4_rt_db_exec_tx(int64_t tx, int64_t query)"));
    assert!(source.contains(
        "int64_t sec4_rt_db_query_one(int64_t db, int64_t query, int64_t row_schema)"
    ));
    assert!(source.contains("int64_t sec4_rt_fs_read(int64_t fs, int64_t path)"));
    assert!(source.contains("int64_t sec4_rt_fs_write(int64_t fs, int64_t path, int64_t value)"));
    assert!(source.contains("int64_t sec4_rt_http_get(int64_t net, int64_t url)"));
    assert!(source.contains("int64_t sec4_rt_http_get_internal(int64_t net, int64_t url)"));
    assert!(source.contains("int64_t sec4_rt_secret_get(int64_t secrets_cap, const char *name)"));
    assert!(source.contains("int64_t sec4_rt_secret_redact(int64_t secret_value)"));
    assert!(source.contains("int64_t sec4_rt_secret_reveal(int64_t secrets_cap, int64_t secret_value)"));
    assert!(source.contains("bool sec4_rt_crypto_ct_eq(int64_t left_secret, int64_t right_secret)"));
    assert!(source.contains("int64_t sec4_rt_validate_header_value(int64_t input)"));
    assert!(source.contains("int64_t sec4_rt_validate_email(int64_t input)"));
    assert!(source.contains("int64_t sec4_rt_validate_uuid(int64_t input)"));
    assert!(source.contains("int64_t sec4_rt_validate_int64(int64_t input)"));
    assert!(source.contains("int64_t sec4_rt_validate_non_empty(int64_t input)"));
    assert!(source.contains("int64_t sec4_rt_sanitize_html(int64_t input)"));
    assert!(source.contains("int64_t sec4_rt_url_public(int64_t input)"));
    assert!(source.contains("int64_t sec4_rt_url_internal(int64_t input)"));
    assert!(source.contains("int64_t sec4_rt_path_under(int64_t base, int64_t input)"));
    assert!(source.contains("int64_t sec4_rt_path_base(const char *input)"));
    assert!(source.contains("int64_t sec4_rt_headers_name(const char *input)"));
    assert!(source.contains("int64_t sec4_rt_headers_value(const char *input)"));
    assert!(source.contains("int64_t sec4_rt_http_router(void)"));
    assert!(source.contains("int64_t sec4_rt_http_route_get("));
    assert!(source.contains("int64_t sec4_rt_http_route_post("));
    assert!(source.contains("int64_t sec4_rt_http_serve(int64_t port, int64_t router)"));
    assert!(source.contains("int64_t sec4_rt_with_cors(int64_t router, int64_t cfg)"));
    assert!(source.contains("int64_t sec4_rt_with_security_headers(int64_t router, int64_t cfg)"));
    assert!(source.contains("int64_t sec4_rt_with_csrf(int64_t router, int64_t cfg)"));
    assert!(source.contains("int64_t sec4_rt_with_auth(int64_t router, int64_t cfg)"));
    assert!(source.contains("int64_t sec4_rt_sec_default_headers(void)"));
    assert!(source.contains("int64_t sec4_rt_sec_csp(void)"));
    assert!(source.contains("int64_t sec4_rt_sec_csp_add(int64_t csp, const char *directive, const char *value)"));
    assert!(source.contains("int64_t sec4_rt_cors_from_policy(void)"));
    assert!(source.contains("int64_t sec4_rt_cors_origin(int64_t origin)"));
    assert!(source.contains("int64_t sec4_rt_csrf_from_policy(void)"));
    assert!(source.contains("int64_t sec4_rt_csrf_issue_token(int64_t ctx)"));
    assert!(source.contains("int64_t sec4_rt_auth_from_policy(void)"));
    assert!(source.contains("int64_t sec4_rt_auth_require(int64_t ctx)"));
    assert!(source.contains("int64_t sec4_rt_auth_require_role(int64_t ctx, const char *required_role)"));
    assert!(source.contains("int64_t sec4_rt_err_validation(const char *code, const char *message)"));
    assert!(source.contains("int64_t sec4_rt_err_auth(const char *code, const char *message, int64_t status)"));
    assert!(source.contains("int64_t sec4_rt_err_not_found(const char *code, const char *message)"));
    assert!(source.contains("int64_t sec4_rt_err_conflict(const char *code, const char *message)"));
    assert!(source.contains("int64_t sec4_rt_err_rate_limit(const char *code, const char *message, int64_t limit)"));
    assert!(source.contains("int64_t sec4_rt_err_internal(const char *message)"));
    assert!(source.contains("int64_t sec4_rt_err_with_path(int64_t error, const char *path)"));
    assert!(source.contains("int64_t sec4_rt_err_with_detail(int64_t error, const char *key, int64_t value)"));
    assert!(source.contains("int64_t sec4_rt_err_with_limit(int64_t error, const char *name, int64_t value, int64_t max)"));
    assert!(source.contains("int64_t sec4_rt_err_with_dependency("));
    assert!(source.contains("int64_t sec4_rt_err_with_cause(int64_t error, int64_t cause)"));
}

#[test]
fn c_backend_rewrites_time_now_intrinsic_to_runtime_symbol() {
    let source = r#"
fn current() -> Int64 {
  time.now()
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("return sec4_rt_identity_i64(sec4_rt_time_now());"));
}

#[test]
fn c_backend_rewrites_log_intrinsics_to_runtime_symbol() {
    let source = r#"
fn main() effects { log } -> Int {
  log.info(log.event("event"));
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(sec4_rt_log_any(sec4_rt_log_event(\"event\")));"));
}

#[test]
fn c_backend_rewrites_log_builder_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() -> Int {
  let event = log.event("user.created");
  let num = log.i64(1);
  let field = log.field("count", num);
  let obj = log.obj(field);
  let text = log.str("ok");
  let flag = log.bool(true);
  let secret = log.redacted("secret");
  let attrSecret = log.attrRedacted("token");
  let withAttr = log.withAttr(event, "token", attrSecret);
  let withHttp = log.withHttp(withAttr, "POST", "/users", 200, 42);
  let error = err.internal("boom");
  let withError = log.withError(withHttp, error);
  event;
  field;
  obj;
  text;
  num;
  flag;
  secret;
  attrSecret;
  withAttr;
  withHttp;
  error;
  withError;
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("sec4_rt_log_event(\"user.created\");"));
    assert!(c.contains("sec4_rt_log_field(\"count\", num);"));
    assert!(c.contains("sec4_rt_log_obj(field);"));
    assert!(c.contains("sec4_rt_log_str(\"ok\");"));
    assert!(c.contains("sec4_rt_log_i64(1);"));
    assert!(c.contains("sec4_rt_log_bool(true);"));
    assert!(c.contains("sec4_rt_log_redacted(\"secret\");"));
    assert!(c.contains("sec4_rt_log_attr_redacted(\"token\");"));
    assert!(c.contains("sec4_rt_log_with_attr(event, \"token\", attrSecret);"));
    assert!(c.contains("sec4_rt_log_with_http(withAttr, \"POST\", \"/users\", 200, 42);"));
    assert!(c.contains("sec4_rt_err_internal(\"boom\");"));
    assert!(c.contains("sec4_rt_log_with_error(withHttp, error);"));
}

#[test]
fn c_backend_rewrites_req_and_res_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() effects { net } -> Int {
  let schema = 1;
  req.body(1, 2);
  req.query(1, 2);
  req.pathParam(1, 2);
  req.header(1, 2);
  req.json(schema);
  res.json(schema, 1);
  res.ok(201, schema, 1);
  res.okMeta(201, schema, 1, 2);
  res.html(1);
  res.text(200, 1);
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(sec4_rt_req_body(1, 2));"));
    assert!(c.contains("(void)(sec4_rt_req_query(1, 2));"));
    assert!(c.contains("(void)(sec4_rt_req_path_param(1, 2));"));
    assert!(c.contains("(void)(sec4_rt_req_header(1, 2));"));
    assert!(c.contains("(void)(sec4_rt_req_json(schema));"));
    assert!(c.contains("(void)(sec4_rt_res_json(schema, 1));"));
    assert!(c.contains("(void)(sec4_rt_res_ok(201, schema, 1));"));
    assert!(c.contains("(void)(sec4_rt_res_ok_meta(201, schema, 1, 2));"));
    assert!(c.contains("(void)(sec4_rt_res_html(1));"));
    assert!(c.contains("(void)(sec4_rt_res_text(200, 1));"));
}

#[test]
fn c_backend_rewrites_json_helper_intrinsics_to_runtime_symbols() {
    let source = r#"
fn useJson(ctx: Ctx, schema: Schema<Int>, raw: Untrusted<Bytes>) -> Int {
  json.decode(ctx, schema, raw);
  json.encode(schema, 3);
  0
}

fn main() -> Int {
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(sec4_rt_json_decode(ctx, schema, raw));"));
    assert!(c.contains("(void)(sec4_rt_json_encode(schema, 3));"));
}

#[test]
fn c_backend_rewrites_header_and_cookie_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() effects { net } -> Int {
  let cookie = cookie.build("name", "value");
  res.setHeader(1, 2);
  res.addCookie(cookie);
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("sec4_rt_cookie_build(\"name\", \"value\");"));
    assert!(c.contains("(void)(sec4_rt_set_header(1, 2));"));
    assert!(c.contains("(void)(sec4_rt_set_cookie(cookie));"));
}

#[test]
fn c_backend_rewrites_db_fs_and_net_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() effects { db.write, db.read, db.tx, fs.read, fs.write, net } -> Int {
  let query = sql.q("SELECT 1", 2);
  db.tx(1);
  db.execTx(1, query);
  db.exec(1, query);
  db.queryOne(1, query, 3);
  fs.read(1, 2);
  fs.write(1, 2, 3);
  httpClient.get(1, 2);
  httpClient.getInternal(1, 2);
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("sec4_rt_sql_q(\"SELECT 1\", 2);"));
    assert!(c.contains("(void)(sec4_rt_db_tx(1));"));
    assert!(c.contains("(void)(sec4_rt_db_exec_tx(1, query));"));
    assert!(c.contains("(void)(sec4_rt_db_exec(1, query));"));
    assert!(c.contains("(void)(sec4_rt_db_query_one(1, query, 3));"));
    assert!(c.contains("(void)(sec4_rt_fs_read(1, 2));"));
    assert!(c.contains("(void)(sec4_rt_fs_write(1, 2, 3));"));
    assert!(c.contains("(void)(sec4_rt_http_get(1, 2));"));
    assert!(c.contains("(void)(sec4_rt_http_get_internal(1, 2));"));
}

#[test]
fn c_backend_rewrites_secret_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() effects { secrets.read, secrets.reveal } -> Int {
  secrets.get(1, 2);
  secrets.redact(2);
  secrets.reveal(1, 2);
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(sec4_rt_secret_get(1, 2));"));
    assert!(c.contains("(void)(sec4_rt_secret_redact(2));"));
    assert!(c.contains("(void)(sec4_rt_secret_reveal(1, 2));"));
}

#[test]
fn c_backend_rewrites_crypto_ct_eq_intrinsic_to_runtime_symbol() {
    let source = r#"
fn compare(a: Secret<String>, b: Secret<String>) -> Bool {
  crypto.ctEq(a, b)
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("return sec4_rt_identity_bool(sec4_rt_crypto_ct_eq(a, b));"));
}

#[test]
fn c_backend_rewrites_gate_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() -> Int {
  let input = 1;
  let base = 2;
  validate.headerValue(input);
  validate.email(input);
  validate.uuid(input);
  validate.int64(input);
  validate.nonEmpty(input);
  sanitize.html(input);
  url.public(input);
  url.internal(input);
  path.under(base, input);
  path.base(1);
  headers.name(1);
  headers.value(1);
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(sec4_rt_validate_header_value(input));"));
    assert!(c.contains("(void)(sec4_rt_validate_email(input));"));
    assert!(c.contains("(void)(sec4_rt_validate_uuid(input));"));
    assert!(c.contains("(void)(sec4_rt_validate_int64(input));"));
    assert!(c.contains("(void)(sec4_rt_validate_non_empty(input));"));
    assert!(c.contains("(void)(sec4_rt_sanitize_html(input));"));
    assert!(c.contains("(void)(sec4_rt_url_public(input));"));
    assert!(c.contains("(void)(sec4_rt_url_internal(input));"));
    assert!(c.contains("(void)(sec4_rt_path_under(base, input));"));
    assert!(c.contains("(void)(sec4_rt_path_base(1));"));
    assert!(c.contains("(void)(sec4_rt_headers_name(1));"));
    assert!(c.contains("(void)(sec4_rt_headers_value(1));"));
}

#[test]
fn c_backend_rewrites_http_router_intrinsics_to_runtime_symbols() {
    let source = r#"
fn handler() -> Int {
  0
}

fn main() effects { net } -> Int {
  let router = http.router();
  http.get(router, "/health", handler);
  http.post(router, "/users", handler);
  http.serve(1, router);
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("router = sec4_rt_http_router();"));
    assert!(c.contains("(void)(sec4_rt_http_route_get(router, \"/health\", handler));"));
    assert!(c.contains("(void)(sec4_rt_http_route_post(router, \"/users\", handler));"));
    assert!(c.contains("(void)(sec4_rt_http_serve(1, router));"));
}

#[test]
fn c_backend_rewrites_security_middleware_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() -> Int {
  withSecurityHeaders();
  withCors();
  withCsrf();
  withAuth();
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(sec4_rt_with_security_headers());"));
    assert!(c.contains("(void)(sec4_rt_with_cors());"));
    assert!(c.contains("(void)(sec4_rt_with_csrf());"));
    assert!(c.contains("(void)(sec4_rt_with_auth());"));
}

#[test]
fn c_backend_rewrites_policy_config_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() -> Int {
  sec.defaultHeaders();
  let csp = sec.csp();
  sec.cspAdd(csp, "default-src", "'self'");
  cors.fromPolicy();
  csrf.fromPolicy();
  auth.fromPolicy();
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(sec4_rt_sec_default_headers());"));
    assert!(c.contains("sec4_rt_sec_csp();"));
    assert!(c.contains("(void)(sec4_rt_sec_csp_add(csp, \"default-src\", \"'self'\"));"));
    assert!(c.contains("(void)(sec4_rt_cors_from_policy());"));
    assert!(c.contains("(void)(sec4_rt_csrf_from_policy());"));
    assert!(c.contains("(void)(sec4_rt_auth_from_policy());"));
}

#[test]
fn c_backend_rewrites_cors_origin_intrinsic_to_runtime_symbol() {
    let source = r#"
fn main() -> Int {
  cors.origin(1);
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(sec4_rt_cors_origin(1));"));
}

#[test]
fn c_backend_rewrites_csrf_issue_token_intrinsic_to_runtime_symbol() {
    let source = r#"
fn main() effects { net } -> Int {
  csrf.issueToken(1);
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(sec4_rt_csrf_issue_token(1));"));
}

#[test]
fn c_backend_rewrites_auth_requirement_intrinsics_to_runtime_symbols() {
    let source = r#"
fn enforceAuth(ctx: Ctx) -> Int {
  auth.require(ctx);
  auth.requireRole(ctx, "admin");
  0
}

fn main() -> Int {
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(sec4_rt_auth_require(ctx));"));
    assert!(c.contains("(void)(sec4_rt_auth_require_role(ctx, \"admin\"));"));
}

#[test]
fn c_backend_rewrites_error_builder_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() -> Int {
  let base = err.validation("VALIDATION.BAD_REQUEST", "invalid input");
  let auth = err.auth("AUTH.FORBIDDEN", "forbidden", 401);
  let notFound = err.notFound("RESOURCE.NOT_FOUND", "missing");
  let conflict = err.conflict("RESOURCE.CONFLICT", "conflict");
  let limited = err.rateLimit("LIMIT.RATE", "rate limited", 3);
  let internal = err.internal("internal");
  let withPath = err.withPath(base, "$.field");
  let withDetail = err.withDetail(base, "field", 2);
  let withLimit = err.withLimit(base, "limit", 2, 3);
  let withDependency = err.withDependency(base, "postgres", "query", true);
  let withCause = err.withCause(base, internal);
  auth;
  notFound;
  conflict;
  limited;
  withPath;
  withDetail;
  withLimit;
  withDependency;
  withCause;
  0
}
"#;

    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("sec4_rt_err_validation(\"VALIDATION.BAD_REQUEST\", \"invalid input\");"));
    assert!(c.contains("sec4_rt_err_auth(\"AUTH.FORBIDDEN\", \"forbidden\", 401);"));
    assert!(c.contains("sec4_rt_err_not_found(\"RESOURCE.NOT_FOUND\", \"missing\");"));
    assert!(c.contains("sec4_rt_err_conflict(\"RESOURCE.CONFLICT\", \"conflict\");"));
    assert!(c.contains("sec4_rt_err_rate_limit(\"LIMIT.RATE\", \"rate limited\", 3);"));
    assert!(c.contains("sec4_rt_err_internal(\"internal\");"));
    assert!(c.contains("sec4_rt_err_with_path(base, \"$.field\");"));
    assert!(c.contains("sec4_rt_err_with_detail(base, \"field\", 2);"));
    assert!(c.contains("sec4_rt_err_with_limit(base, \"limit\", 2, 3);"));
    assert!(c.contains("sec4_rt_err_with_dependency(base, \"postgres\", \"query\", true);"));
    assert!(c.contains("sec4_rt_err_with_cause(base, internal);"));
}
