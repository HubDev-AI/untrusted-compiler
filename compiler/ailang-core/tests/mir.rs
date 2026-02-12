use ailang_core::{lower_program_to_mir, parse_source};
use std::path::Path;

#[test]
fn mir_lowering_builds_single_block_for_simple_function() {
    let source = r#"
fn add(a: Int, b: Int) effects { db.read } -> Int {
  let sum = a + b;
  sum
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);

    assert_eq!(mir.functions.len(), 1);
    let function = &mir.functions[0];
    assert_eq!(function.name, "add");
    assert_eq!(function.params.len(), 2);
    assert_eq!(function.effects, vec!["db.read".to_string()]);
    assert_eq!(function.return_type.as_deref(), Some("Int"));
    assert_eq!(function.blocks.len(), 1);

    let block = &function.blocks[0];
    assert_eq!(block.id, 0);
    assert_eq!(block.instructions.len(), 1);

    let rendered = mir.render_text();
    assert!(rendered.contains("fn add(a: Int, b: Int) effects {db.read} -> Int"));
    assert!(rendered.contains("bb0:"));
    assert!(rendered.contains("let sum = (a + b)"));
    assert!(rendered.contains("return sum"));
}

#[test]
fn mir_lowering_honors_explicit_return_terminator() {
    let source = r#"
fn main() -> Int {
  let x = 1;
  return x;
  0
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let rendered = mir.render_text();

    assert!(rendered.contains("let x = 1"));
    assert!(rendered.contains("return x"));
    assert!(
        !rendered.contains("return 0"),
        "tail expression after explicit return should not become terminator"
    );
}

#[test]
fn mir_lowering_splits_tail_if_into_branch_blocks() {
    let source = r#"
fn classify(x: Int) -> Int {
  if x > 0 {
    1
  } else {
    0
  }
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);

    assert_eq!(mir.functions.len(), 1);
    assert_eq!(mir.functions[0].blocks.len(), 3);

    let rendered = mir.render_text();
    assert!(rendered.contains("bb0:"));
    assert!(rendered.contains("branch (x > 0) ? bb1 : bb2"));
    assert!(rendered.contains("bb1:"));
    assert!(rendered.contains("return 1"));
    assert!(rendered.contains("bb2:"));
    assert!(rendered.contains("return 0"));
}
