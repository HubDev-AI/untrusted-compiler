pub mod ast;
pub mod audit;
pub mod diagnostics;
pub mod lexer;
pub mod manifest;
pub mod parser;
pub mod policy;
pub mod security_map;
pub mod semantic;
pub mod token;

pub use audit::{
    render_security_audit_text, run_security_audit, should_fail, AuditReport, AuditSeverity,
};
pub use diagnostics::{Diagnostic, Severity, Span};
pub use manifest::{Manifest, ManifestFile, PackageSection};
pub use parser::parse_source;
pub use policy::{Policy, PolicyMode, POLICY_FILE_NAME};
pub use security_map::{build_security_map, SecurityMap, SECURITY_MAP_FILE_NAME};
pub use semantic::analyze_program;

use std::fs;
use std::path::{Path, PathBuf};

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
    let policy = policy::load_policy(project_root)?;
    semantic::analyze_program_with_policy(&program, &policy)?;
    Ok(program)
}

pub fn write_security_map(
    project_root: &Path,
    security_map: &SecurityMap,
) -> Result<PathBuf, Diagnostic> {
    let build_dir = project_root.join("build");
    if let Err(err) = fs::create_dir_all(&build_dir) {
        return Err(Diagnostic::error(
            "M0104",
            "could not create build directory",
            Span::point(build_dir.clone(), 1, 1),
        )
        .with_note(err.to_string()));
    }

    let output_path = build_dir.join(SECURITY_MAP_FILE_NAME);
    let content =
        serde_json::to_string_pretty(security_map).expect("security map should serialize");
    if let Err(err) = fs::write(&output_path, content) {
        return Err(Diagnostic::error(
            "M0105",
            "could not write security map",
            Span::point(output_path.clone(), 1, 1),
        )
        .with_note(err.to_string()));
    }

    Ok(output_path)
}
