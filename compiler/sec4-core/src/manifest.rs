use crate::diagnostics::{Diagnostic, Span};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const MANIFEST_FILE_NAME: &str = "sec4.toml";
pub const LOCK_FILE_NAME: &str = "sec4.lock";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ManifestFile {
    pub package: PackageSection,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
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
    write_lockfile(project_root, manifest)
}

pub fn write_lockfile(project_root: &Path, manifest: &Manifest) -> Result<(), Diagnostic> {
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let body = render_lockfile(manifest);

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
    validate_lockfile(project_root, manifest)
}

pub fn validate_lockfile(project_root: &Path, manifest: &Manifest) -> Result<(), Diagnostic> {
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

    let expected = render_lockfile(manifest);
    let normalized_current = normalize_lockfile(&current);
    let normalized_expected = normalize_lockfile(&expected);
    if normalized_current != normalized_expected {
        let mut diagnostic = Diagnostic::error(
            "M0203",
            "sec4.lock does not match current sec4.toml package/build/dependency state",
            Span::point(lock_path.clone(), 1, 1),
        )
        .with_note("run `sec4 build` (without --locked) to regenerate sec4.lock");
        if let Some(detail) = first_lockfile_mismatch(&normalized_expected, &normalized_current) {
            diagnostic = diagnostic.with_note(detail);
        }
        return Err(diagnostic);
    }

    Ok(())
}

fn render_lockfile(manifest: &Manifest) -> String {
    let mut lockfile = String::new();
    lockfile.push_str("# Untrusted<T> lockfile v1 (deterministic)\n");
    lockfile.push_str("lock_version = 1\n\n");

    let package_hash = package_hash(manifest);
    lockfile.push_str("[package]\n");
    push_toml_string_line(&mut lockfile, "name", &manifest.package.name);
    push_toml_string_line(&mut lockfile, "version", &manifest.package.version);
    push_toml_string_line(&mut lockfile, "edition", &manifest.package.edition);
    push_toml_string_line(&mut lockfile, "hash", &package_hash);
    lockfile.push('\n');

    let build_hash = build_hash(manifest);
    lockfile.push_str("[build]\n");
    push_toml_string_line(&mut lockfile, "entry", manifest.entry_file());
    push_toml_string_line(&mut lockfile, "hash", &build_hash);
    push_toml_string_line(
        &mut lockfile,
        "manifest_hash",
        &manifest_fingerprint(manifest),
    );

    if !manifest.dependencies.is_empty() {
        lockfile.push('\n');
    }

    for (name, version) in &manifest.dependencies {
        lockfile.push_str("[[dependency]]\n");
        push_toml_string_line(&mut lockfile, "name", name);
        push_toml_string_line(&mut lockfile, "version", version);
        push_toml_string_line(&mut lockfile, "hash", &dependency_hash(name, version));
        lockfile.push('\n');
    }

    lockfile
}

fn push_toml_string_line(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(" = ");
    out.push_str(&toml::Value::String(value.to_string()).to_string());
    out.push('\n');
}

fn manifest_fingerprint(manifest: &Manifest) -> String {
    let mut normalized = format!(
        "name={}\nversion={}\nedition={}\nentry={}\n",
        manifest.package.name,
        manifest.package.version,
        manifest.package.edition,
        manifest.entry_file(),
    );
    for (name, version) in &manifest.dependencies {
        normalized.push_str(&format!("dependency={name}@{version}\n"));
    }
    format!("man_{:016x}", fnv1a64(normalized.as_bytes()))
}

fn package_hash(manifest: &Manifest) -> String {
    let normalized = format!(
        "name={}\nversion={}\nedition={}\n",
        manifest.package.name, manifest.package.version, manifest.package.edition
    );
    format!("pkg_{:016x}", fnv1a64(normalized.as_bytes()))
}

fn build_hash(manifest: &Manifest) -> String {
    let normalized = format!("entry={}\n", manifest.entry_file());
    format!("bld_{:016x}", fnv1a64(normalized.as_bytes()))
}

fn dependency_hash(name: &str, version: &str) -> String {
    let normalized = format!("name={name}\nversion={version}\n");
    format!("dep_{:016x}", fnv1a64(normalized.as_bytes()))
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

fn first_lockfile_mismatch(expected: &str, actual: &str) -> Option<String> {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();
    let max_lines = expected_lines.len().max(actual_lines.len());

    for line_index in 0..max_lines {
        let expected_line = expected_lines.get(line_index).copied();
        let actual_line = actual_lines.get(line_index).copied();
        if expected_line != actual_line {
            let expected_line = expected_line.unwrap_or("<end-of-file>");
            let actual_line = actual_line.unwrap_or("<end-of-file>");
            return Some(format!(
                "first lockfile mismatch at line {}: expected `{expected_line}` but found `{actual_line}`",
                line_index + 1
            ));
        }
    }

    None
}
