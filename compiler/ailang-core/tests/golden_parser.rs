use ailang_core::ast::{Expr, ExprKind, ItemKind, Program, StmtKind, TypeExpr, TypeExprKind};
use ailang_core::parse_source;
use std::fs;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parser")
}

fn collect_case_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(dir)
        .expect("fixtures directory should exist")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "ai"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn parser_fixtures_match_golden_output() {
    let dir = fixtures_dir();
    let case_files = collect_case_files(&dir);
    assert!(!case_files.is_empty(), "expected at least one parser fixture");

    for case in case_files {
        let input = fs::read_to_string(&case).expect("fixture should be readable");
        let virtual_path = Path::new(
            case.file_name()
                .and_then(|name| name.to_str())
                .expect("fixture file should have UTF-8 name"),
        );

        let output = match parse_source(virtual_path, &input) {
            Ok(program) => render_program_signature(&program),
            Err(diags) => {
                let mut rendered = String::new();
                for (index, diagnostic) in diags.iter().enumerate() {
                    if index > 0 {
                        rendered.push_str("\n\n");
                    }
                    rendered.push_str(&diagnostic.render_plain());
                }
                rendered
            }
        };

        let golden = case.with_extension("golden");
        let expected = fs::read_to_string(&golden)
            .unwrap_or_else(|_| panic!("missing golden file: {}", golden.display()));

        assert_eq!(
            expected.trim_end(),
            output.trim_end(),
            "golden mismatch for fixture {}",
            case.display()
        );
    }
}

fn render_program_signature(program: &Program) -> String {
    let mut out = String::new();
    out.push_str("OK\n");
    out.push_str(&format!("program.items={}\n", program.items.len()));

    for item in &program.items {
        match &item.kind {
            ItemKind::Struct(decl) => {
                let fields = decl
                    .fields
                    .iter()
                    .map(|field| format!("{}: {}", field.name, render_type(&field.ty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("struct {} {{ {} }}\n", decl.name, fields));
            }
            ItemKind::Enum(decl) => {
                let variants = decl
                    .variants
                    .iter()
                    .map(|variant| {
                        if variant.payload.is_empty() {
                            variant.name.clone()
                        } else {
                            let payload = variant
                                .payload
                                .iter()
                                .map(|field| match &field.name {
                                    Some(name) => format!("{name}: {}", render_type(&field.ty)),
                                    None => render_type(&field.ty),
                                })
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("{}({payload})", variant.name)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("enum {} {{ {} }}\n", decl.name, variants));
            }
            ItemKind::Function(decl) => {
                let params = decl
                    .params
                    .iter()
                    .map(|param| format!("{}: {}", param.name, render_type(&param.ty)))
                    .collect::<Vec<_>>()
                    .join(", ");
                let return_type = decl
                    .return_type
                    .as_ref()
                    .map(render_type)
                    .unwrap_or_else(|| "Unit".to_string());
                let effects = if decl.effects.is_empty() {
                    "none".to_string()
                } else {
                    decl.effects
                        .iter()
                        .map(|effect| effect.as_name())
                        .collect::<Vec<_>>()
                        .join(",")
                };

                out.push_str(&format!(
                    "fn {}({}) effects=[{}] -> {} {{ stmts={}, tail={} }}\n",
                    decl.name,
                    params,
                    effects,
                    return_type,
                    decl.body.statements.len(),
                    decl.body
                        .tail
                        .as_ref()
                        .map(|tail| render_expr_kind(tail))
                        .unwrap_or_else(|| "none".to_string())
                ));

                for (index, stmt) in decl.body.statements.iter().enumerate() {
                    match &stmt.kind {
                        StmtKind::Let {
                            is_const,
                            mutable,
                            name,
                            ty,
                            value,
                        } => {
                            let binding = if *is_const {
                                "const"
                            } else if *mutable {
                                "let mut"
                            } else {
                                "let"
                            };
                            let ty_text = ty
                                .as_ref()
                                .map(render_type)
                                .map(|text| format!(": {text}"))
                                .unwrap_or_default();
                            out.push_str(&format!(
                                "  stmt{}={} {}{} = {}\n",
                                index,
                                binding,
                                name,
                                ty_text,
                                render_expr_kind(value)
                            ));
                        }
                        StmtKind::Return { value } => out.push_str(&format!(
                            "  stmt{}=return {}\n",
                            index,
                            value
                                .as_ref()
                                .map(render_expr_kind)
                                .unwrap_or_else(|| "none".to_string())
                        )),
                        StmtKind::Expr { expr } => {
                            out.push_str(&format!("  stmt{}=expr {}\n", index, render_expr_kind(expr)))
                        }
                    }
                }
            }
        }
    }

    out
}

fn render_type(ty: &TypeExpr) -> String {
    match &ty.kind {
        TypeExprKind::Named { name, args } => {
            if args.is_empty() {
                name.clone()
            } else {
                let args_text = args.iter().map(render_type).collect::<Vec<_>>().join(", ");
                format!("{name}<{args_text}>")
            }
        }
    }
}

fn render_expr_kind(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Identifier(name) => format!("ident({name})"),
        ExprKind::Number(value) => format!("number({value})"),
        ExprKind::String(value) => format!("string({value})"),
        ExprKind::Bool(value) => format!("bool({value})"),
        ExprKind::Unary { .. } => "unary".to_string(),
        ExprKind::Binary { .. } => "binary".to_string(),
        ExprKind::Call { callee, args } => {
            format!("call({}, args={})", render_expr_kind(callee), args.len())
        }
        ExprKind::If { .. } => "if".to_string(),
        ExprKind::Match { arms, .. } => format!("match(arms={})", arms.len()),
        ExprKind::Block(_) => "block".to_string(),
    }
}
