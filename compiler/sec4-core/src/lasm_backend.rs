use crate::mir::{
    MirBlock, MirFunction, MirInstructionKind, MirProgram, MirSwitchTarget, MirTerminator,
};
use serde::Serialize;
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LasmProgram {
    pub version: &'static str,
    pub entry: Option<LasmEntrypoint>,
    pub functions: Vec<LasmFunction>,
}

impl LasmProgram {
    pub fn render_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(&mut out, ".lasm {}", self.version);
        if let Some(entry) = &self.entry {
            let _ = writeln!(
                &mut out,
                ".entry {} params={} ret={}",
                entry.name, entry.param_count, entry.return_type
            );
        }
        for function in &self.functions {
            render_function(&mut out, function);
        }
        out
    }

    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("LASM should serialize")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LasmEntrypoint {
    pub name: String,
    pub param_count: usize,
    pub return_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LasmFunction {
    pub name: String,
    pub params: Vec<LasmParam>,
    pub effects: Vec<String>,
    pub return_type: Option<String>,
    pub blocks: Vec<LasmBlock>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LasmParam {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LasmBlock {
    pub label: String,
    pub ops: Vec<LasmOp>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LasmSwitchTarget {
    pub pattern: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum LasmOp {
    Let {
        dst: String,
        value: String,
    },
    Eval {
        value: String,
    },
    Ret {
        value: Option<String>,
    },
    Jmp {
        target: String,
    },
    Br {
        condition: String,
        then_target: String,
        else_target: String,
    },
    Switch {
        scrutinee: String,
        targets: Vec<LasmSwitchTarget>,
    },
}

pub fn lower_mir_to_lasm(program: &MirProgram) -> LasmProgram {
    let functions = program
        .functions
        .iter()
        .map(lower_function_to_lasm)
        .collect::<Vec<_>>();
    LasmProgram {
        version: "v0",
        entry: select_entrypoint(&functions),
        functions,
    }
}

pub fn emit_lasm_program(program: &MirProgram) -> String {
    lower_mir_to_lasm(program).render_text()
}

pub fn emit_lasm_program_json(program: &MirProgram) -> String {
    lower_mir_to_lasm(program).to_pretty_json()
}

fn lower_function_to_lasm(function: &MirFunction) -> LasmFunction {
    LasmFunction {
        name: function.name.clone(),
        params: function
            .params
            .iter()
            .map(|param| LasmParam {
                name: param.name.clone(),
                ty: param.ty.clone(),
            })
            .collect(),
        effects: function.effects.clone(),
        return_type: function.return_type.clone(),
        blocks: function.blocks.iter().map(lower_block_to_lasm).collect(),
    }
}

fn select_entrypoint(functions: &[LasmFunction]) -> Option<LasmEntrypoint> {
    let function = functions
        .iter()
        .find(|function| function.name == "main")
        .or_else(|| functions.first())?;

    Some(LasmEntrypoint {
        name: function.name.clone(),
        param_count: function.params.len(),
        return_type: function
            .return_type
            .clone()
            .unwrap_or_else(|| "Void".to_string()),
    })
}

fn lower_block_to_lasm(block: &MirBlock) -> LasmBlock {
    let mut ops = Vec::new();
    for instruction in &block.instructions {
        match &instruction.kind {
            MirInstructionKind::Let { name, value } => ops.push(LasmOp::Let {
                dst: name.clone(),
                value: value.clone(),
            }),
            MirInstructionKind::Eval { value } => ops.push(LasmOp::Eval {
                value: value.clone(),
            }),
        }
    }

    match &block.terminator {
        MirTerminator::Return { value, .. } => ops.push(LasmOp::Ret {
            value: value.clone(),
        }),
        MirTerminator::Goto { target, .. } => ops.push(LasmOp::Jmp {
            target: format!("bb{target}"),
        }),
        MirTerminator::Branch {
            condition,
            then_target,
            else_target,
            ..
        } => ops.push(LasmOp::Br {
            condition: condition.clone(),
            then_target: format!("bb{then_target}"),
            else_target: format!("bb{else_target}"),
        }),
        MirTerminator::Switch {
            scrutinee, targets, ..
        } => {
            let mapped_targets = targets
                .iter()
                .map(map_switch_target)
                .collect::<Vec<_>>();
            ops.push(LasmOp::Switch {
                scrutinee: scrutinee.clone(),
                targets: mapped_targets,
            });
        }
    }

    LasmBlock {
        label: format!("bb{}", block.id),
        ops,
    }
}

fn map_switch_target(target: &MirSwitchTarget) -> LasmSwitchTarget {
    LasmSwitchTarget {
        pattern: target.pattern.clone(),
        target: format!("bb{}", target.target),
    }
}

fn render_function(out: &mut String, function: &LasmFunction) {
    let _ = writeln!(out, ".fn {}", function.name);
    let params = if function.params.is_empty() {
        "()".to_string()
    } else {
        function
            .params
            .iter()
            .map(|param| format!("{}:{}", param.name, param.ty))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let effects = if function.effects.is_empty() {
        "none".to_string()
    } else {
        function.effects.join(", ")
    };
    let ret = function
        .return_type
        .clone()
        .unwrap_or_else(|| "Void".to_string());
    let _ = writeln!(out, "  .params {params}");
    let _ = writeln!(out, "  .effects {effects}");
    let _ = writeln!(out, "  .ret {ret}");
    for block in &function.blocks {
        render_block(out, block);
    }
    let _ = writeln!(out, ".end");
}

fn render_block(out: &mut String, block: &LasmBlock) {
    let _ = writeln!(out, "  {}:", block.label);
    for op in &block.ops {
        match op {
            LasmOp::Let { dst, value } => {
                let _ = writeln!(out, "    let {dst} = {value}");
            }
            LasmOp::Eval { value } => {
                let _ = writeln!(out, "    eval {value}");
            }
            LasmOp::Ret { value } => match value {
                Some(value) => {
                    let _ = writeln!(out, "    ret {value}");
                }
                None => {
                    let _ = writeln!(out, "    ret");
                }
            },
            LasmOp::Jmp { target } => {
                let _ = writeln!(out, "    jmp {target}");
            }
            LasmOp::Br {
                condition,
                then_target,
                else_target,
            } => {
                let _ = writeln!(out, "    br {condition} {then_target} {else_target}");
            }
            LasmOp::Switch { scrutinee, targets } => {
                let target_text = targets
                    .iter()
                    .map(|target| format!("{}->{}", target.pattern, target.target))
                    .collect::<Vec<_>>()
                    .join(", ");
                let _ = writeln!(out, "    switch {scrutinee} [{target_text}]");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{emit_lasm_program, emit_lasm_program_json};
    use crate::mir::{
        MirBlock, MirFunction, MirInstruction, MirInstructionKind, MirProgram, MirTerminator,
    };
    use crate::Span;
    use std::path::PathBuf;

    fn test_span() -> Span {
        Span::point(PathBuf::from("src/main.ut"), 1, 1)
    }

    #[test]
    fn emits_lasm_text_for_simple_program() {
        let span = test_span();
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "main".to_string(),
                params: vec![],
                effects: vec![],
                return_type: Some("Int".to_string()),
                blocks: vec![MirBlock {
                    id: 0,
                    instructions: vec![MirInstruction {
                        kind: MirInstructionKind::Let {
                            name: "x".to_string(),
                            value: "1".to_string(),
                        },
                        span: span.clone(),
                    }],
                    terminator: MirTerminator::Return {
                        value: Some("0".to_string()),
                        span: span.clone(),
                    },
                }],
                span,
            }],
        };

        let emitted = emit_lasm_program(&program);
        assert!(
            emitted.contains(".lasm v0"),
            "lasm text should include version header:\n{emitted}"
        );
        assert!(
            emitted.contains(".entry main params=0 ret=Int"),
            "lasm text should include explicit entrypoint metadata:\n{emitted}"
        );
        assert!(
            emitted.contains(".fn main"),
            "lasm text should include function header:\n{emitted}"
        );
        assert!(
            emitted.contains("let x = 1") && emitted.contains("ret 0"),
            "lasm text should include lowered ops:\n{emitted}"
        );
    }

    #[test]
    fn emits_lasm_json_for_simple_program() {
        let span = test_span();
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "main".to_string(),
                params: vec![],
                effects: vec!["net".to_string()],
                return_type: Some("Int".to_string()),
                blocks: vec![MirBlock {
                    id: 0,
                    instructions: vec![],
                    terminator: MirTerminator::Return {
                        value: Some("0".to_string()),
                        span: span.clone(),
                    },
                }],
                span,
            }],
        };

        let emitted = emit_lasm_program_json(&program);
        assert!(
            emitted.contains("\"version\": \"v0\""),
            "lasm json should include version:\n{emitted}"
        );
        assert!(
            emitted.contains("\"entry\"") && emitted.contains("\"name\": \"main\""),
            "lasm json should include deterministic entrypoint metadata:\n{emitted}"
        );
        assert!(
            emitted.contains("\"name\": \"main\"") && emitted.contains("\"op\": \"ret\""),
            "lasm json should include function and return op:\n{emitted}"
        );
    }

    #[test]
    fn emits_lasm_entry_for_first_function_when_main_is_missing() {
        let span = test_span();
        let program = MirProgram {
            functions: vec![MirFunction {
                name: "bootstrap".to_string(),
                params: vec![],
                effects: vec![],
                return_type: Some("Int".to_string()),
                blocks: vec![MirBlock {
                    id: 0,
                    instructions: vec![],
                    terminator: MirTerminator::Return {
                        value: Some("0".to_string()),
                        span: span.clone(),
                    },
                }],
                span,
            }],
        };

        let emitted = emit_lasm_program(&program);
        assert!(
            emitted.contains(".entry bootstrap params=0 ret=Int"),
            "lasm text should select first function as deterministic fallback entrypoint:\n{emitted}"
        );
    }
}
