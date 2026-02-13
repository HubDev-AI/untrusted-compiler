use sec4_core::AuditReport;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .to_path_buf()
}

#[test]
fn release_audit_baselines_parse_as_audit_reports() {
    let root = workspace_root();
    let default_secure_path = root.join("baselines/sec-audit/default-secure-prod.hello.json");
    let permissive_path = root.join("baselines/sec-audit/permissive-dev.hello.json");

    let default_secure_raw = fs::read_to_string(&default_secure_path)
        .expect("default-secure baseline should be readable");
    let permissive_raw =
        fs::read_to_string(&permissive_path).expect("permissive baseline should be readable");

    let default_secure: AuditReport =
        serde_json::from_str(&default_secure_raw).expect("default-secure baseline should parse");
    let permissive: AuditReport =
        serde_json::from_str(&permissive_raw).expect("permissive baseline should parse");

    assert_eq!(default_secure.policy.name, "default-secure-prod");
    assert_eq!(permissive.policy.name, "permissive-dev");
}
