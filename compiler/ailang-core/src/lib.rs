pub mod diagnostics;
pub mod manifest;

pub use diagnostics::{Diagnostic, Severity, Span};
pub use manifest::{Manifest, ManifestFile, PackageSection};

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
