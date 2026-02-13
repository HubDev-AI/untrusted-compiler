use ailang_core::{validate_lockfile_stub, validate_project, write_lockfile_stub};
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
        project_dir.join("ailang.toml"),
        format!(
            "[package]\nname = \"{package_name}\"\nversion = \"{version}\"\n\n[build]\nentry = \"src/main.ai\"\n"
        ),
    )
    .expect("manifest should be written");
    fs::write(project_dir.join("src/main.ai"), "fn main() -> Int {\n  0\n}\n")
        .expect("source should be written");
}

#[test]
fn validate_lockfile_stub_accepts_fresh_lockfile() {
    let project_dir = temp_dir("ailang-core-lock-fresh");
    write_project(&project_dir, "lock_fresh", "0.1.0");

    let manifest = validate_project(&project_dir).expect("project should validate");
    write_lockfile_stub(&project_dir, &manifest).expect("lockfile should be written");
    validate_lockfile_stub(&project_dir, &manifest).expect("fresh lockfile should validate");

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}

#[test]
fn validate_lockfile_stub_rejects_stale_lockfile() {
    let project_dir = temp_dir("ailang-core-lock-stale");
    write_project(&project_dir, "lock_stale", "0.1.0");

    let manifest = validate_project(&project_dir).expect("project should validate");
    write_lockfile_stub(&project_dir, &manifest).expect("lockfile should be written");

    write_project(&project_dir, "lock_stale", "0.2.0");
    let stale_manifest = validate_project(&project_dir).expect("project should still validate");
    let diagnostic =
        validate_lockfile_stub(&project_dir, &stale_manifest).expect_err("stale lock should fail");
    assert_eq!(diagnostic.code, "M0203");

    fs::remove_dir_all(&project_dir).expect("temp project cleanup should succeed");
}
