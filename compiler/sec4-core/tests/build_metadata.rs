use sec4_core::{validate_project, write_build_metadata, write_lockfile_stub, BuildMetadata};
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
fn write_build_metadata_produces_expected_fields() {
    let project_dir = temp_dir("sec4-core-build-metadata-fields");
    write_project(&project_dir, "metadata_fields", "0.1.0");

    let manifest = validate_project(&project_dir).expect("project should validate");
    write_lockfile_stub(&project_dir, &manifest).expect("lockfile should be written");
    let policy = sec4_core::policy::load_policy(&project_dir).expect("policy should load");

    let metadata_path = write_build_metadata(&project_dir, &manifest, &policy)
        .expect("metadata write should succeed");
    let raw = fs::read_to_string(&metadata_path).expect("metadata should be readable");
    let parsed: BuildMetadata =
        serde_json::from_str(&raw).expect("metadata should deserialize to BuildMetadata");

    assert_eq!(parsed.version, "0.1");
    assert_eq!(parsed.package.name, "metadata_fields");
    assert_eq!(parsed.package.version, "0.1.0");
    assert_eq!(parsed.package.entry, "src/main.ut");
    assert!(parsed.policy_hash.starts_with("pol_"));
    assert!(parsed.compiler_hash.starts_with("cpl_"));
    assert!(parsed.runtime_hash.starts_with("rt_"));
    assert!(parsed.lock_fingerprint.starts_with("lock_"));

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn write_build_metadata_is_deterministic_for_same_inputs() {
    let project_dir = temp_dir("sec4-core-build-metadata-deterministic");
    write_project(&project_dir, "metadata_deterministic", "0.1.0");

    let manifest = validate_project(&project_dir).expect("project should validate");
    write_lockfile_stub(&project_dir, &manifest).expect("lockfile should be written");
    let policy = sec4_core::policy::load_policy(&project_dir).expect("policy should load");

    let first_path =
        write_build_metadata(&project_dir, &manifest, &policy).expect("first write should succeed");
    let first = fs::read_to_string(&first_path).expect("first metadata read should succeed");
    let second_path = write_build_metadata(&project_dir, &manifest, &policy)
        .expect("second write should succeed");
    let second = fs::read_to_string(&second_path).expect("second metadata read should succeed");

    assert_eq!(first, second, "metadata content should be deterministic");

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}
