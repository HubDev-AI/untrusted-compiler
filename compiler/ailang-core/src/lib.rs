pub mod ast;
pub mod diagnostics;
pub mod lexer;
pub mod manifest;
pub mod parser;
pub mod semantic;
pub mod token;

pub use diagnostics::{Diagnostic, Severity, Span};
pub use manifest::{Manifest, ManifestFile, PackageSection};
pub use parser::parse_source;
pub use semantic::analyze_program;

use std::fs;
use std::path::Path;

pub fn validate_project(project_root: &Path) -> Result<Manifest, Vec<Diagnostic>> {
    let manifest = manifest::load_manifest(project_root)?;
    let mut diagnostics = manifest::validate_for_build(project_root, &manifest);

    if diagnostics.is_empty() {
        Ok(manifest)
    } else {
        diagnostics.sort_by(|a, b| a.code.cmp(&b.code));
        Err(diagnostics)
    }
}

pub fn write_lockfile_stub(project_root: &Path, manifest: &Manifest) -> Result<(), Diagnostic> {
    manifest::write_lockfile_stub(project_root, manifest)
}

pub fn parse_entry_ast(
    project_root: &Path,
    manifest: &Manifest,
) -> Result<ast::Program, Vec<Diagnostic>> {
    let entry_path = manifest.entry_path(project_root);
    let source = match fs::read_to_string(&entry_path) {
        Ok(source) => source,
        Err(err) => {
            let diagnostic = Diagnostic::error(
                "M0103",
                "could not read entry source file",
                Span::point(entry_path, 1, 1),
            )
            .with_note(err.to_string());
            return Err(vec![diagnostic]);
        }
    };

    parser::parse_source(&manifest.entry_path(project_root), &source)
}

pub fn analyze_entry(
    project_root: &Path,
    manifest: &Manifest,
) -> Result<ast::Program, Vec<Diagnostic>> {
    let program = parse_entry_ast(project_root, manifest)?;
    semantic::analyze_program(&program)?;
    Ok(program)
}
