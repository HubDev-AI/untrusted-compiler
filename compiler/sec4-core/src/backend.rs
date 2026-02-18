use crate::c_backend::{emit_c_program, emit_runtime_header, emit_runtime_source};
use crate::lasm_backend::{emit_lasm_program, emit_lasm_program_json};
use crate::mir::MirProgram;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BackendKind {
    C,
    Lasm,
    LasmJson,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RuntimeAssets {
    pub header: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BackendEmitOutput {
    pub source: String,
    pub runtime_assets: Option<RuntimeAssets>,
}

pub trait BackendEmitter {
    fn kind(&self) -> BackendKind;
    fn emit_program(&self, program: &MirProgram) -> BackendEmitOutput;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CBackendEmitter;

impl BackendEmitter for CBackendEmitter {
    fn kind(&self) -> BackendKind {
        BackendKind::C
    }

    fn emit_program(&self, program: &MirProgram) -> BackendEmitOutput {
        BackendEmitOutput {
            source: emit_c_program(program),
            runtime_assets: Some(RuntimeAssets {
                header: emit_runtime_header(),
                source: emit_runtime_source(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LasmBackendEmitter;

impl BackendEmitter for LasmBackendEmitter {
    fn kind(&self) -> BackendKind {
        BackendKind::Lasm
    }

    fn emit_program(&self, program: &MirProgram) -> BackendEmitOutput {
        BackendEmitOutput {
            source: emit_lasm_program(program),
            runtime_assets: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LasmJsonBackendEmitter;

impl BackendEmitter for LasmJsonBackendEmitter {
    fn kind(&self) -> BackendKind {
        BackendKind::LasmJson
    }

    fn emit_program(&self, program: &MirProgram) -> BackendEmitOutput {
        BackendEmitOutput {
            source: emit_lasm_program_json(program),
            runtime_assets: None,
        }
    }
}

pub fn emit_program_with_backend(backend: BackendKind, program: &MirProgram) -> BackendEmitOutput {
    match backend {
        BackendKind::C => CBackendEmitter.emit_program(program),
        BackendKind::Lasm => LasmBackendEmitter.emit_program(program),
        BackendKind::LasmJson => LasmJsonBackendEmitter.emit_program(program),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        emit_program_with_backend, BackendEmitter, BackendKind, CBackendEmitter, LasmBackendEmitter,
        LasmJsonBackendEmitter, RuntimeAssets,
    };
    use crate::c_backend::{emit_c_program, emit_runtime_header, emit_runtime_source};
    use crate::lasm_backend::{emit_lasm_program, emit_lasm_program_json};
    use crate::mir::MirProgram;

    #[test]
    fn c_backend_emitter_reports_kind() {
        assert_eq!(CBackendEmitter.kind(), BackendKind::C);
    }

    #[test]
    fn lasm_backend_emitter_reports_kind() {
        assert_eq!(LasmBackendEmitter.kind(), BackendKind::Lasm);
        assert_eq!(LasmJsonBackendEmitter.kind(), BackendKind::LasmJson);
    }

    #[test]
    fn backend_dispatch_matches_legacy_c_emitter_output() {
        let program = MirProgram { functions: vec![] };
        let emitted = emit_program_with_backend(BackendKind::C, &program);

        assert_eq!(emitted.source, emit_c_program(&program));
        assert_eq!(
            emitted.runtime_assets,
            Some(RuntimeAssets {
                header: emit_runtime_header(),
                source: emit_runtime_source(),
            })
        );
    }

    #[test]
    fn backend_dispatch_emits_lasm_text_without_runtime_assets() {
        let program = MirProgram { functions: vec![] };
        let emitted = emit_program_with_backend(BackendKind::Lasm, &program);
        assert_eq!(emitted.source, emit_lasm_program(&program));
        assert_eq!(emitted.runtime_assets, None);
    }

    #[test]
    fn backend_dispatch_emits_lasm_json_without_runtime_assets() {
        let program = MirProgram { functions: vec![] };
        let emitted = emit_program_with_backend(BackendKind::LasmJson, &program);
        assert_eq!(emitted.source, emit_lasm_program_json(&program));
        assert_eq!(emitted.runtime_assets, None);
    }
}
