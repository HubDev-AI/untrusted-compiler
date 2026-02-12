use ailang_core::{
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

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("#include <stdbool.h>"));
    assert!(c.contains("#include <stdint.h>"));
    assert!(c.contains("#include \"ailang_runtime.h\""));
    assert!(c.contains("int main(void);"));
    assert!(c.contains("int main(void) {"));
    assert!(c.contains("bb0:"));
    assert!(c.contains("return ailang_rt_identity_i64(0);"));
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

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("if ((x > 0)) goto bb1; else goto bb2;"));
    assert!(c.contains("goto bb3;"));
    assert!(c.contains("return ailang_rt_identity_i64(5);"));
}

#[test]
fn c_backend_routes_bool_returns_through_runtime_identity() {
    let source = r#"
fn truthy(flag: Bool) -> Bool {
  flag
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("bool truthy(bool flag);"));
    assert!(c.contains("return ailang_rt_identity_bool(flag);"));
}

#[test]
fn c_backend_emits_runtime_header_and_source() {
    let header = emit_runtime_header();
    let source = emit_runtime_source();

    assert!(header.contains("#ifndef AILANG_RUNTIME_H"));
    assert!(header.contains("int64_t ailang_rt_identity_i64(int64_t value);"));
    assert!(header.contains("bool ailang_rt_identity_bool(bool value);"));
    assert!(header.contains("int64_t ailang_rt_time_now(void);"));
    assert!(header.contains("void ailang_rt_log_any();"));
    assert!(header.contains("int64_t ailang_rt_req_json();"));
    assert!(header.contains("int64_t ailang_rt_res_json();"));
    assert!(header.contains("int64_t ailang_rt_res_html();"));
    assert!(header.contains("int64_t ailang_rt_set_header();"));
    assert!(header.contains("int64_t ailang_rt_set_cookie();"));
    assert!(header.contains("int64_t ailang_rt_db_exec();"));
    assert!(header.contains("int64_t ailang_rt_db_query_one();"));
    assert!(header.contains("int64_t ailang_rt_fs_read();"));
    assert!(header.contains("int64_t ailang_rt_fs_write();"));
    assert!(header.contains("int64_t ailang_rt_http_get();"));
    assert!(header.contains("int64_t ailang_rt_http_get_internal();"));

    assert!(source.contains("#include \"ailang_runtime.h\""));
    assert!(source.contains("int64_t ailang_rt_identity_i64(int64_t value)"));
    assert!(source.contains("bool ailang_rt_identity_bool(bool value)"));
    assert!(source.contains("int64_t ailang_rt_time_now(void)"));
    assert!(source.contains("void ailang_rt_log_any()"));
    assert!(source.contains("int64_t ailang_rt_req_json()"));
    assert!(source.contains("int64_t ailang_rt_res_json()"));
    assert!(source.contains("int64_t ailang_rt_res_html()"));
    assert!(source.contains("int64_t ailang_rt_set_header()"));
    assert!(source.contains("int64_t ailang_rt_set_cookie()"));
    assert!(source.contains("int64_t ailang_rt_db_exec()"));
    assert!(source.contains("int64_t ailang_rt_db_query_one()"));
    assert!(source.contains("int64_t ailang_rt_fs_read()"));
    assert!(source.contains("int64_t ailang_rt_fs_write()"));
    assert!(source.contains("int64_t ailang_rt_http_get()"));
    assert!(source.contains("int64_t ailang_rt_http_get_internal()"));
}

#[test]
fn c_backend_rewrites_time_now_intrinsic_to_runtime_symbol() {
    let source = r#"
fn current() -> Int64 {
  time.now()
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("return ailang_rt_identity_i64(ailang_rt_time_now());"));
}

#[test]
fn c_backend_rewrites_log_intrinsics_to_runtime_symbol() {
    let source = r#"
fn main() effects { log } -> Int {
  log.info(1);
  0
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(ailang_rt_log_any(1));"));
}

#[test]
fn c_backend_rewrites_req_and_res_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() effects { net } -> Int {
  let schema = 1;
  req.json(schema);
  res.json(schema, 1);
  res.html(1);
  0
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(ailang_rt_req_json(schema));"));
    assert!(c.contains("(void)(ailang_rt_res_json(schema, 1));"));
    assert!(c.contains("(void)(ailang_rt_res_html(1));"));
}

#[test]
fn c_backend_rewrites_header_and_cookie_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() effects { net } -> Int {
  res.setHeader(1, 2);
  res.addCookie(1);
  0
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(ailang_rt_set_header(1, 2));"));
    assert!(c.contains("(void)(ailang_rt_set_cookie(1));"));
}

#[test]
fn c_backend_rewrites_db_fs_and_net_intrinsics_to_runtime_symbols() {
    let source = r#"
fn main() effects { db.write, db.read, fs.read, fs.write, net } -> Int {
  db.exec(1, 2);
  db.queryOne(1, 2, 3);
  fs.read(1, 2);
  fs.write(1, 2, 3);
  httpClient.get(1, 2);
  httpClient.getInternal(1, 2);
  0
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let c = emit_c_program(&mir);

    assert!(c.contains("(void)(ailang_rt_db_exec(1, 2));"));
    assert!(c.contains("(void)(ailang_rt_db_query_one(1, 2, 3));"));
    assert!(c.contains("(void)(ailang_rt_fs_read(1, 2));"));
    assert!(c.contains("(void)(ailang_rt_fs_write(1, 2, 3));"));
    assert!(c.contains("(void)(ailang_rt_http_get(1, 2));"));
    assert!(c.contains("(void)(ailang_rt_http_get_internal(1, 2));"));
}
