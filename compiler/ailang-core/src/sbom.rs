use crate::build_metadata::BuildMetadata;
use crate::diagnostics::{Diagnostic, Span};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const SBOM_FILE_NAME: &str = "sbom.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SbomDocument {
    #[serde(rename = "bomFormat")]
    pub bom_format: String,
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    pub version: i32,
    pub metadata: SbomMetadata,
    pub components: Vec<SbomComponent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SbomMetadata {
    pub component: SbomComponent,
    pub properties: Vec<SbomProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SbomComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SbomProperty {
    pub name: String,
    pub value: String,
}

pub fn write_sbom(project_root: &Path, build_metadata: &BuildMetadata) -> Result<PathBuf, Diagnostic> {
    let build_dir = project_root.join("build");
    fs::create_dir_all(&build_dir).map_err(|err| {
        Diagnostic::error(
            "M0311",
            "failed to create build directory for sbom",
            Span::point(build_dir.clone(), 1, 1),
        )
        .with_note(err.to_string())
    })?;

    let doc = SbomDocument {
        bom_format: "CycloneDX".to_string(),
        spec_version: "1.5".to_string(),
        version: 1,
        metadata: SbomMetadata {
            component: SbomComponent {
                component_type: "application".to_string(),
                name: build_metadata.package.name.clone(),
                version: build_metadata.package.version.clone(),
            },
            properties: vec![
                SbomProperty {
                    name: "ailang:policyHash".to_string(),
                    value: build_metadata.policy_hash.clone(),
                },
                SbomProperty {
                    name: "ailang:compilerHash".to_string(),
                    value: build_metadata.compiler_hash.clone(),
                },
                SbomProperty {
                    name: "ailang:runtimeHash".to_string(),
                    value: build_metadata.runtime_hash.clone(),
                },
                SbomProperty {
                    name: "ailang:lockFingerprint".to_string(),
                    value: build_metadata.lock_fingerprint.clone(),
                },
            ],
        },
        components: vec![
            SbomComponent {
                component_type: "library".to_string(),
                name: "ailang-compiler".to_string(),
                version: build_metadata.compiler_hash.clone(),
            },
            SbomComponent {
                component_type: "library".to_string(),
                name: "ailang-runtime".to_string(),
                version: build_metadata.runtime_hash.clone(),
            },
        ],
    };

    let output_path = build_dir.join(SBOM_FILE_NAME);
    let content = serde_json::to_string_pretty(&doc).expect("sbom should serialize");
    fs::write(&output_path, content).map_err(|err| {
        Diagnostic::error(
            "M0312",
            "failed to write sbom",
            Span::point(output_path.clone(), 1, 1),
        )
        .with_note(err.to_string())
    })?;

    Ok(output_path)
}
