use ailang_core::{emit_c_program, lower_program_to_mir, parse_source};
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
    assert!(c.contains("int64_t main(void);"));
    assert!(c.contains("int64_t main(void) {"));
    assert!(c.contains("bb0:"));
    assert!(c.contains("return 0;"));
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
    assert!(c.contains("return 5;"));
}
