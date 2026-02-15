use crate::c_backend::{emit_c_program, emit_runtime_header, emit_runtime_source};
use crate::mir::MirProgram;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BackendKind {
    C,
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

pub fn emit_program_with_backend(backend: BackendKind, program: &MirProgram) -> BackendEmitOutput {
    match backend {
        BackendKind::C => CBackendEmitter.emit_program(program),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        emit_program_with_backend, BackendEmitter, BackendKind, CBackendEmitter, RuntimeAssets,
    };
    use crate::c_backend::{emit_c_program, emit_runtime_header, emit_runtime_source};
    use crate::mir::MirProgram;

    #[test]
    fn c_backend_emitter_reports_kind() {
        assert_eq!(CBackendEmitter.kind(), BackendKind::C);
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
}
