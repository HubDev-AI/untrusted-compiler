use crate::diagnostics::{Diagnostic, Span};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const MANIFEST_FILE_NAME: &str = "ailang.toml";
pub const LOCK_FILE_NAME: &str = "ailang.lock";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ManifestFile {
    pub package: PackageSection,
    #[serde(default)]
    pub build: Option<BuildSection>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PackageSection {
    pub name: String,
    pub version: String,
    #[serde(default = "default_edition")]
    pub edition: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct BuildSection {
    #[serde(default = "default_entry")]
    pub entry: String,
}

fn default_edition() -> String {
    "2026".to_string()
}

fn default_entry() -> String {
    "src/main.ai".to_string()
}

pub type Manifest = ManifestFile;

impl ManifestFile {
    pub fn entry_file(&self) -> &str {
        self.build
            .as_ref()
            .map(|build| build.entry.as_str())
            .unwrap_or("src/main.ai")
    }

    pub fn entry_path(&self, root: &Path) -> PathBuf {
        root.join(self.entry_file())
    }
}

pub fn parse_manifest_str(manifest_path: &Path, source: &str) -> Result<Manifest, Vec<Diagnostic>> {
    let parsed: ManifestFile = match toml::from_str(source) {
        Ok(value) => value,
        Err(err) => {
            let diagnostic = Diagnostic::error(
                "M0002",
                "failed to parse ailang.toml",
                Span::point(manifest_path.to_path_buf(), 1, 1),
            )
            .with_note(err.to_string());
            return Err(vec![diagnostic]);
        }
    };

    let mut diagnostics = Vec::new();
    if parsed.package.name.trim().is_empty() {
        diagnostics.push(
            Diagnostic::error(
                "M0003",
                "package.name must not be empty",
                Span::point(manifest_path.to_path_buf(), 1, 1),
            )
            .with_note("set [package].name to a non-empty identifier"),
        );
    }

    if parsed.package.version.trim().is_empty() {
        diagnostics.push(
            Diagnostic::error(
                "M0004",
                "package.version must not be empty",
                Span::point(manifest_path.to_path_buf(), 1, 1),
            )
            .with_note("set [package].version to a semantic version string"),
        );
    }

    if diagnostics.is_empty() {
        Ok(parsed)
    } else {
        Err(diagnostics)
    }
}

pub fn load_manifest(project_root: &Path) -> Result<Manifest, Vec<Diagnostic>> {
    let manifest_path = project_root.join(MANIFEST_FILE_NAME);
    let source = match fs::read_to_string(&manifest_path) {
        Ok(content) => content,
        Err(err) => {
            let diagnostic = Diagnostic::error(
                "M0001",
                "could not read ailang.toml",
                Span::point(manifest_path, 1, 1),
            )
            .with_note(err.to_string())
            .with_note("run this command from an AILang project root");
            return Err(vec![diagnostic]);
        }
    };

    parse_manifest_str(&project_root.join(MANIFEST_FILE_NAME), &source)
}

pub fn validate_for_build(project_root: &Path, manifest: &Manifest) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let entry_path = manifest.entry_path(project_root);

    if !entry_path.exists() {
        diagnostics.push(
            Diagnostic::error(
                "M0101",
                "entry file not found",
                Span::point(project_root.join(MANIFEST_FILE_NAME), 1, 1),
            )
            .with_note(format!("expected entry path: {}", entry_path.display()))
            .with_note("set [build].entry in ailang.toml or create src/main.ai"),
        );
    }

    if let Some(ext) = entry_path.extension() {
        if ext != "ai" {
            diagnostics.push(
                Diagnostic::error(
                    "M0102",
                    "entry file must use .ai extension",
                    Span::point(entry_path.clone(), 1, 1),
                )
                .with_note(format!("found extension: .{}", ext.to_string_lossy())),
            );
        }
    }

    diagnostics
}

pub fn write_lockfile_stub(project_root: &Path, manifest: &Manifest) -> Result<(), Diagnostic> {
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let body = format!(
        "# AILang lockfile v0 (stub)\n\
[package]\n\
name = \"{}\"\n\
version = \"{}\"\n\
edition = \"{}\"\n",
        manifest.package.name, manifest.package.version, manifest.package.edition
    );

    fs::write(&lock_path, body).map_err(|err| {
        Diagnostic::error(
            "M0201",
            "failed to write ailang.lock",
            Span::point(lock_path.clone(), 1, 1),
        )
        .with_note(err.to_string())
    })
}
