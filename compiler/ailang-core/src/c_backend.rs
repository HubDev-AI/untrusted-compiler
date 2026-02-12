use crate::mir::{MirFunction, MirInstructionKind, MirProgram, MirTerminator};
use std::collections::BTreeSet;
use std::fmt::Write;

pub fn emit_c_program(program: &MirProgram) -> String {
    let mut out = String::new();
    out.push_str("#include <stdbool.h>\n#include <stdint.h>\n#include \"ailang_runtime.h\"\n\n");

    for function in &program.functions {
        writeln!(&mut out, "{};", c_function_signature(function))
            .expect("write to string must succeed");
    }

    if !program.functions.is_empty() {
        out.push('\n');
    }

    for (index, function) in program.functions.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        render_function(&mut out, function);
    }

    out
}

pub fn emit_runtime_header() -> &'static str {
    include_str!("../../../runtime/c/ailang_runtime.h")
}

pub fn emit_runtime_source() -> &'static str {
    include_str!("../../../runtime/c/ailang_runtime.c")
}

fn render_function(out: &mut String, function: &MirFunction) {
    writeln!(out, "{} {{", c_function_signature(function)).expect("write to string must succeed");
    let runtime_return_identity = runtime_identity_for_return_type(function.return_type.as_deref());

    let locals = collect_locals(function);
    for local in &locals {
        writeln!(out, "  int64_t {} = 0;", local).expect("write to string must succeed");
    }

    if !function.blocks.is_empty() {
        writeln!(out, "  goto bb0;").expect("write to string must succeed");
    }

    for block in &function.blocks {
        writeln!(out, "bb{}:", block.id).expect("write to string must succeed");

        for instruction in &block.instructions {
            match &instruction.kind {
                MirInstructionKind::Let { name, value } => {
                    writeln!(out, "  {} = {};", name, lower_c_expr(value))
                        .expect("write to string must succeed");
                }
                MirInstructionKind::Eval { value } => {
                    writeln!(out, "  (void)({});", lower_c_expr(value))
                        .expect("write to string must succeed");
                }
            }
        }

        match &block.terminator {
            MirTerminator::Return { value, .. } => {
                if let Some(value) = value {
                    let value = lower_c_expr(value);
                    if let Some(identity_fn) = runtime_return_identity {
                        writeln!(out, "  return {}({});", identity_fn, value)
                            .expect("write to string must succeed");
                    } else {
                        writeln!(out, "  return {};", value).expect("write to string must succeed");
                    }
                } else {
                    writeln!(out, "  return;").expect("write to string must succeed");
                }
            }
            MirTerminator::Goto { target, .. } => {
                writeln!(out, "  goto bb{};", target).expect("write to string must succeed");
            }
            MirTerminator::Branch {
                condition,
                then_target,
                else_target,
                ..
            } => {
                let condition = lower_c_expr(condition);
                writeln!(
                    out,
                    "  if ({}) goto bb{}; else goto bb{};",
                    condition, then_target, else_target
                )
                .expect("write to string must succeed");
            }
            MirTerminator::Switch {
                scrutinee, targets, ..
            } => {
                let scrutinee = lower_c_expr(scrutinee);
                let mut default_target = None;
                for target in targets {
                    if target.pattern == "_" {
                        default_target = Some(target.target);
                    } else {
                        writeln!(
                            out,
                            "  if ({} == {}) goto bb{};",
                            scrutinee, target.pattern, target.target
                        )
                        .expect("write to string must succeed");
                    }
                }

                let fallback_target =
                    default_target.or_else(|| targets.first().map(|target| target.target));
                if let Some(target) = fallback_target {
                    writeln!(out, "  goto bb{};", target).expect("write to string must succeed");
                } else {
                    writeln!(out, "  return;").expect("write to string must succeed");
                }
            }
        }
    }

    writeln!(out, "}}").expect("write to string must succeed");
}

fn collect_locals(function: &MirFunction) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for block in &function.blocks {
        for instruction in &block.instructions {
            if let MirInstructionKind::Let { name, .. } = &instruction.kind {
                names.insert(name.clone());
            }
        }
    }
    names
}

fn c_function_signature(function: &MirFunction) -> String {
    let return_type = if function.name == "main" {
        "int"
    } else {
        c_type(function.return_type.as_deref())
    };
    let params = function
        .params
        .iter()
        .map(|param| format!("{} {}", c_type(Some(param.ty.as_str())), param.name))
        .collect::<Vec<_>>()
        .join(", ");

    if params.is_empty() {
        format!("{} {}(void)", return_type, function.name)
    } else {
        format!("{} {}({})", return_type, function.name, params)
    }
}

fn c_type(type_name: Option<&str>) -> &'static str {
    match type_name {
        Some("Bool") => "bool",
        Some("Unit") => "void",
        Some("Int") | Some("Int64") => "int64_t",
        None => "void",
        Some(_) => "int64_t",
    }
}

fn runtime_identity_for_return_type(type_name: Option<&str>) -> Option<&'static str> {
    match type_name {
        Some("Bool") => Some("ailang_rt_identity_bool"),
        Some("Int") | Some("Int64") => Some("ailang_rt_identity_i64"),
        _ => None,
    }
}

fn lower_c_expr(expr: &str) -> String {
    let mut lowered = expr.to_string();
    lowered = lowered.replace("time.now(", "__AILANG_INTRINSIC_TIME_NOW__(");
    lowered = lowered.replace("time_now(", "__AILANG_INTRINSIC_TIME_NOW__(");
    lowered = lowered.replace("__AILANG_INTRINSIC_TIME_NOW__(", "ailang_rt_time_now(");
    lowered
}
