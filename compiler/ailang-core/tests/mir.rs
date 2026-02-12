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

#[test]
fn mir_lowering_splits_tail_match_into_switch_blocks() {
    let source = r#"
fn pick(x: Bool) -> Int {
  match x {
    true => 1,
    false => 0
  }
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);

    assert_eq!(mir.functions.len(), 1);
    assert_eq!(mir.functions[0].blocks.len(), 3);

    let rendered = mir.render_text();
    assert!(rendered.contains("bb0:"));
    assert!(rendered.contains("switch x { true => bb1, false => bb2 }"));
    assert!(rendered.contains("bb1:"));
    assert!(rendered.contains("return 1"));
    assert!(rendered.contains("bb2:"));
    assert!(rendered.contains("return 0"));
}

#[test]
fn mir_lowering_splits_return_if_into_branch_blocks() {
    let source = r#"
fn classify(x: Int) -> Int {
  let marker = 1;
  return if x > 0 {
    1
  } else {
    0
  };
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);

    assert_eq!(mir.functions.len(), 1);
    assert_eq!(mir.functions[0].blocks.len(), 3);

    let rendered = mir.render_text();
    assert!(rendered.contains("let marker = 1"));
    assert!(rendered.contains("branch (x > 0) ? bb1 : bb2"));
    assert!(rendered.contains("bb1:"));
    assert!(rendered.contains("return 1"));
    assert!(rendered.contains("bb2:"));
    assert!(rendered.contains("return 0"));
    assert!(
        !rendered.contains("return if"),
        "return-if should be lowered into explicit branch blocks"
    );
}

#[test]
fn mir_lowering_splits_return_match_into_switch_blocks() {
    let source = r#"
fn pick(x: Bool) -> Int {
  let marker = 1;
  return match x {
    true => 1,
    false => 0
  };
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);

    assert_eq!(mir.functions.len(), 1);
    assert_eq!(mir.functions[0].blocks.len(), 3);

    let rendered = mir.render_text();
    assert!(rendered.contains("let marker = 1"));
    assert!(rendered.contains("switch x { true => bb1, false => bb2 }"));
    assert!(rendered.contains("bb1:"));
    assert!(rendered.contains("return 1"));
    assert!(rendered.contains("bb2:"));
    assert!(rendered.contains("return 0"));
    assert!(
        !rendered.contains("return match"),
        "return-match should be lowered into explicit switch blocks"
    );
}

#[test]
fn mir_lowering_splits_statement_if_into_continuation_blocks() {
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

    assert_eq!(mir.functions.len(), 1);
    assert_eq!(mir.functions[0].blocks.len(), 4);

    let rendered = mir.render_text();
    assert!(rendered.contains("branch (x > 0) ? bb2 : bb3"));
    assert!(rendered.contains("bb2:"));
    assert!(rendered.contains("eval 1"));
    assert!(rendered.contains("goto bb1"));
    assert!(rendered.contains("bb3:"));
    assert!(rendered.contains("eval 0"));
    assert!(rendered.contains("goto bb1"));
    assert!(rendered.contains("bb1:"));
    assert!(rendered.contains("return 5"));
}

#[test]
fn mir_lowering_splits_statement_match_into_continuation_blocks() {
    let source = r#"
fn flow(x: Bool) -> Int {
  match x {
    true => 1,
    false => 0
  };
  5
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);

    assert_eq!(mir.functions.len(), 1);
    assert_eq!(mir.functions[0].blocks.len(), 4);

    let rendered = mir.render_text();
    assert!(rendered.contains("switch x { true => bb2, false => bb3 }"));
    assert!(rendered.contains("bb2:"));
    assert!(rendered.contains("eval 1"));
    assert!(rendered.contains("goto bb1"));
    assert!(rendered.contains("bb3:"));
    assert!(rendered.contains("eval 0"));
    assert!(rendered.contains("goto bb1"));
    assert!(rendered.contains("bb1:"));
    assert!(rendered.contains("return 5"));
}

#[test]
fn mir_lowering_recurses_nested_statement_if_in_branch_tail() {
    let source = r#"
fn nested(x: Int, y: Bool) -> Int {
  if x > 0 {
    if y {
      1
    } else {
      2
    }
  } else {
    0
  };
  9
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let rendered = mir.render_text();

    assert!(rendered.contains("branch (x > 0) ? bb2 : bb3"));
    assert!(rendered.contains("bb2:"));
    assert!(rendered.contains("branch y ? bb4 : bb5"));
    assert!(rendered.contains("bb4:"));
    assert!(rendered.contains("return 1") || rendered.contains("eval 1"));
    assert!(rendered.contains("bb5:"));
    assert!(rendered.contains("return 2") || rendered.contains("eval 2"));
    assert!(rendered.contains("bb3:"));
    assert!(rendered.contains("eval 0"));
    assert!(rendered.contains("bb1:"));
    assert!(rendered.contains("return 9"));
}

#[test]
fn mir_lowering_recurses_nested_return_if_match() {
    let source = r#"
fn nested(x: Bool, y: Bool) -> Int {
  return if x {
    match y {
      true => 1,
      false => 2
    }
  } else {
    0
  };
}
"#;

    let program = parse_source(Path::new("main.ai"), source).expect("source should parse");
    let mir = lower_program_to_mir(&program);
    let rendered = mir.render_text();

    assert!(rendered.contains("branch x ? bb1 : bb2"));
    assert!(rendered.contains("bb1:"));
    assert!(rendered.contains("switch y { true => bb3, false => bb4 }"));
    assert!(rendered.contains("bb3:"));
    assert!(rendered.contains("return 1"));
    assert!(rendered.contains("bb4:"));
    assert!(rendered.contains("return 2"));
    assert!(rendered.contains("bb2:"));
    assert!(rendered.contains("return 0"));
}
