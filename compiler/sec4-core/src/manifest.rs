use crate::diagnostics::{Diagnostic, Span};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const MANIFEST_FILE_NAME: &str = "sec4.toml";
pub const LOCK_FILE_NAME: &str = "sec4.lock";

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
    "src/main.ut".to_string()
}

pub type Manifest = ManifestFile;

impl ManifestFile {
    pub fn entry_file(&self) -> &str {
        self.build
            .as_ref()
            .map(|build| build.entry.as_str())
            .unwrap_or("src/main.ut")
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
                "failed to parse sec4.toml",
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
                "could not read sec4.toml",
                Span::point(manifest_path, 1, 1),
            )
            .with_note(err.to_string())
            .with_note("run this command from an Untrusted<T> project root");
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
            .with_note("set [build].entry in sec4.toml or create src/main.ut"),
        );
    }

    if let Some(ext) = entry_path.extension() {
        if ext != "ut" {
            diagnostics.push(
                Diagnostic::error(
                    "M0102",
                    "entry file must use .ut extension",
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
    let body = render_lockfile_stub(manifest);

    fs::write(&lock_path, body).map_err(|err| {
        Diagnostic::error(
            "M0201",
            "failed to write sec4.lock",
            Span::point(lock_path.clone(), 1, 1),
        )
        .with_note(err.to_string())
    })
}

pub fn validate_lockfile_stub(project_root: &Path, manifest: &Manifest) -> Result<(), Diagnostic> {
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let current = fs::read_to_string(&lock_path).map_err(|err| {
        let message = if err.kind() == std::io::ErrorKind::NotFound {
            "sec4.lock is required in --locked mode"
        } else {
            "failed to read sec4.lock in --locked mode"
        };

        let mut diagnostic =
            Diagnostic::error("M0202", message, Span::point(lock_path.clone(), 1, 1))
                .with_note(err.to_string());
        if err.kind() == std::io::ErrorKind::NotFound {
            diagnostic = diagnostic.with_note("run `sec4 build` once to generate sec4.lock");
        }
        diagnostic
    })?;

    let expected = render_lockfile_stub(manifest);
    if normalize_lockfile(&current) != normalize_lockfile(&expected) {
        return Err(Diagnostic::error(
            "M0203",
            "sec4.lock is out of date for current manifest",
            Span::point(lock_path.clone(), 1, 1),
        )
        .with_note("run `sec4 build` (without --locked) to refresh sec4.lock"));
    }

    Ok(())
}

fn render_lockfile_stub(manifest: &Manifest) -> String {
    let manifest_fingerprint = manifest_fingerprint(manifest);
    format!(
        "# Untrusted<T> lockfile v0 (deterministic)\n\
[package]\n\
name = \"{}\"\n\
version = \"{}\"\n\
edition = \"{}\"\n\
\n\
[build]\n\
entry = \"{}\"\n\
manifest_fingerprint = \"{}\"\n",
        manifest.package.name,
        manifest.package.version,
        manifest.package.edition,
        manifest.entry_file(),
        manifest_fingerprint,
    )
}

fn manifest_fingerprint(manifest: &Manifest) -> String {
    let normalized = format!(
        "name={}\nversion={}\nedition={}\nentry={}\n",
        manifest.package.name,
        manifest.package.version,
        manifest.package.edition,
        manifest.entry_file(),
    );
    format!("man_{:016x}", fnv1a64(normalized.as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn normalize_lockfile(content: &str) -> String {
    content.replace("\r\n", "\n")
}
