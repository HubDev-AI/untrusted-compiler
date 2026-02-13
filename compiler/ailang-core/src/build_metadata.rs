use crate::c_backend::{emit_runtime_header, emit_runtime_source};
use crate::diagnostics::{Diagnostic, Span};
use crate::manifest::{Manifest, LOCK_FILE_NAME};
use crate::policy::Policy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const BUILD_METADATA_FILE_NAME: &str = "build_metadata.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildMetadata {
    pub version: String,
    pub package: BuildMetadataPackage,
    #[serde(rename = "policyHash")]
    pub policy_hash: String,
    #[serde(rename = "compilerHash")]
    pub compiler_hash: String,
    #[serde(rename = "runtimeHash")]
    pub runtime_hash: String,
    #[serde(rename = "lockFingerprint")]
    pub lock_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildMetadataPackage {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub entry: String,
}

pub fn compiler_hash() -> String {
    format!("cpl_{}", env!("CARGO_PKG_VERSION").replace('.', "_"))
}

pub fn runtime_hash() -> String {
    let payload = format!("{}\n{}", emit_runtime_header(), emit_runtime_source());
    format!("rt_{:016x}", fnv1a64(payload.as_bytes()))
}

pub fn write_build_metadata(
    project_root: &Path,
    manifest: &Manifest,
    policy: &Policy,
) -> Result<PathBuf, Diagnostic> {
    let lock_path = project_root.join(LOCK_FILE_NAME);
    let lock_content = fs::read_to_string(&lock_path).map_err(|err| {
        Diagnostic::error(
            "M0301",
            "failed to read lockfile for build metadata",
            Span::point(lock_path.clone(), 1, 1),
        )
        .with_note(err.to_string())
        .with_note("run `ailang build` to regenerate lockfile")
    })?;

    let metadata = BuildMetadata {
        version: "0.1".to_string(),
        package: BuildMetadataPackage {
            name: manifest.package.name.clone(),
            version: manifest.package.version.clone(),
            edition: manifest.package.edition.clone(),
            entry: manifest.entry_file().to_string(),
        },
        policy_hash: policy.policy_hash(),
        compiler_hash: compiler_hash(),
        runtime_hash: runtime_hash(),
        lock_fingerprint: format!("lock_{:016x}", fnv1a64(normalize(&lock_content).as_bytes())),
    };

    let build_dir = project_root.join("build");
    fs::create_dir_all(&build_dir).map_err(|err| {
        Diagnostic::error(
            "M0302",
            "failed to create build directory for metadata",
            Span::point(build_dir.clone(), 1, 1),
        )
        .with_note(err.to_string())
    })?;

    let output_path = build_dir.join(BUILD_METADATA_FILE_NAME);
    let content = serde_json::to_string_pretty(&metadata).expect("build metadata should serialize");
    fs::write(&output_path, content).map_err(|err| {
        Diagnostic::error(
            "M0303",
            "failed to write build metadata",
            Span::point(output_path.clone(), 1, 1),
        )
        .with_note(err.to_string())
    })?;

    Ok(output_path)
}

fn normalize(content: &str) -> String {
    content.replace("\r\n", "\n")
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
