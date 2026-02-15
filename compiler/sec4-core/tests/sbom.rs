use sec4_core::{
    validate_lockfile_stub, validate_project, write_build_metadata, write_lockfile_stub,
    write_sbom, BuildMetadata,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    format!("{nanos}")
}

fn temp_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("{prefix}-{}", unique_suffix()));
    fs::create_dir_all(&path).expect("temp directory should be created");
    path
}

fn write_project(project_dir: &PathBuf, package_name: &str, version: &str) {
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        format!(
            "[package]\nname = \"{package_name}\"\nversion = \"{version}\"\n\n[build]\nentry = \"src/main.ut\"\n"
        ),
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("source should be written");
}

#[test]
fn write_sbom_outputs_expected_shape() {
    let project_dir = temp_dir("sec4-core-sbom-shape");
    write_project(&project_dir, "sbom_shape", "0.1.0");

    let manifest = validate_project(&project_dir).expect("project should validate");
    write_lockfile_stub(&project_dir, &manifest).expect("lockfile should be written");
    let policy = sec4_core::policy::load_policy(&project_dir).expect("policy should load");
    let metadata_path = write_build_metadata(&project_dir, &manifest, &policy)
        .expect("metadata write should succeed");
    let metadata_raw = fs::read_to_string(&metadata_path).expect("metadata should be readable");
    let metadata: BuildMetadata =
        serde_json::from_str(&metadata_raw).expect("metadata should parse");

    let sbom_path = write_sbom(&project_dir, &metadata).expect("sbom write should succeed");
    let sbom_raw = fs::read_to_string(&sbom_path).expect("sbom should be readable");
    let sbom_json: serde_json::Value = serde_json::from_str(&sbom_raw).expect("sbom should parse");

    assert_eq!(sbom_json["bomFormat"], "CycloneDX");
    assert_eq!(sbom_json["specVersion"], "1.5");
    assert_eq!(sbom_json["metadata"]["component"]["name"], "sbom_shape");
    assert_eq!(sbom_json["metadata"]["component"]["version"], "0.1.0");

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn write_sbom_is_deterministic_for_same_metadata() {
    let project_dir = temp_dir("sec4-core-sbom-deterministic");
    write_project(&project_dir, "sbom_deterministic", "0.1.0");

    let manifest = validate_project(&project_dir).expect("project should validate");
    write_lockfile_stub(&project_dir, &manifest).expect("lockfile should be written");
    let policy = sec4_core::policy::load_policy(&project_dir).expect("policy should load");
    let metadata_path = write_build_metadata(&project_dir, &manifest, &policy)
        .expect("metadata write should succeed");
    let metadata_raw = fs::read_to_string(&metadata_path).expect("metadata should be readable");
    let metadata: BuildMetadata =
        serde_json::from_str(&metadata_raw).expect("metadata should parse");

    let first_path = write_sbom(&project_dir, &metadata).expect("first sbom should be written");
    let first = fs::read_to_string(&first_path).expect("first sbom should be readable");
    let second_path = write_sbom(&project_dir, &metadata).expect("second sbom should be written");
    let second = fs::read_to_string(&second_path).expect("second sbom should be readable");
    assert_eq!(first, second, "sbom content should be deterministic");

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn lockfile_writes_sorted_dependency_entries_with_hashes() {
    let project_dir = temp_dir("sec4-core-lock-sorted-deps");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "lock_sorted"
version = "0.1.0"

[dependencies]
zlib = "2.0.0"
alpha = "1.0.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("source should be written");

    let manifest = validate_project(&project_dir).expect("project should validate");
    write_lockfile_stub(&project_dir, &manifest).expect("lockfile should be written");
    let lock_path = project_dir.join("sec4.lock");
    let first = fs::read_to_string(&lock_path).expect("first lockfile should be readable");
    write_lockfile_stub(&project_dir, &manifest).expect("lockfile should be rewritten");
    let second = fs::read_to_string(&lock_path).expect("second lockfile should be readable");

    assert_eq!(first, second, "lockfile content should be deterministic");
    assert!(
        first.contains("[package]"),
        "lockfile should include package table"
    );
    assert!(
        first.contains("hash = \"pkg_"),
        "lockfile should include package hash field:\n{first}"
    );
    assert_eq!(
        first.matches("[[dependency]]").count(),
        2,
        "lockfile should include one block per dependency:\n{first}"
    );
    assert_eq!(
        first.matches("hash = \"dep_").count(),
        2,
        "lockfile should include dependency hash fields:\n{first}"
    );
    let alpha_pos = first
        .find("name = \"alpha\"")
        .expect("alpha dependency should be present");
    let zlib_pos = first
        .find("name = \"zlib\"")
        .expect("zlib dependency should be present");
    assert!(
        alpha_pos < zlib_pos,
        "dependencies should be rendered in deterministic sorted order:\n{first}"
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn lockfile_validation_reports_manifest_dependency_mismatch() {
    let project_dir = temp_dir("sec4-core-lock-dep-mismatch");
    fs::create_dir_all(project_dir.join("src")).expect("src directory should be created");
    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "lock_mismatch"
version = "0.1.0"

[dependencies]
alpha = "1.0.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("manifest should be written");
    fs::write(
        project_dir.join("src/main.ut"),
        "fn main() -> Int {\n  0\n}\n",
    )
    .expect("source should be written");

    let manifest = validate_project(&project_dir).expect("project should validate");
    write_lockfile_stub(&project_dir, &manifest).expect("lockfile should be written");

    fs::write(
        project_dir.join("sec4.toml"),
        r#"[package]
name = "lock_mismatch"
version = "0.1.0"

[dependencies]
alpha = "2.0.0"

[build]
entry = "src/main.ut"
"#,
    )
    .expect("updated manifest should be written");
    let stale_manifest =
        validate_project(&project_dir).expect("updated project should still validate");
    let diagnostic =
        validate_lockfile_stub(&project_dir, &stale_manifest).expect_err("stale lock should fail");

    assert_eq!(diagnostic.code, "M0203");
    assert!(
        diagnostic
            .message
            .contains("package/build/dependency state"),
        "diagnostic message should explain strict locked validation:\n{}",
        diagnostic.message
    );
    assert!(
        diagnostic
            .notes
            .iter()
            .any(|note| note.contains("first lockfile mismatch at line")),
        "diagnostic notes should include a clear mismatch location:\n{:?}",
        diagnostic.notes
    );

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}
