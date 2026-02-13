use sec4_core::manifest::parse_manifest_str;
use std::fs;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/manifest")
}

fn collect_case_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(dir)
        .expect("fixtures directory should exist")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn manifest_fixtures_match_golden_output() {
    let dir = fixtures_dir();
    let case_files = collect_case_files(&dir);
    assert!(!case_files.is_empty(), "expected at least one fixture case");

    for case in case_files {
        let input = fs::read_to_string(&case).expect("fixture should be readable");
        let output = match parse_manifest_str(Path::new("sec4.toml"), &input) {
            Ok(manifest) => format!(
                "OK\nname={}\nversion={}\nedition={}\nentry={}",
                manifest.package.name,
                manifest.package.version,
                manifest.package.edition,
                manifest.entry_file()
            ),
            Err(diags) => {
                let mut rendered = String::new();
                for (index, diagnostic) in diags.iter().enumerate() {
                    if index > 0 {
                        rendered.push_str("\n\n");
                    }
                    rendered.push_str(&diagnostic.render_plain());
                }
                rendered
            }
        };

        let golden = case.with_extension("golden");
        let expected = fs::read_to_string(&golden)
            .unwrap_or_else(|_| panic!("missing golden file: {}", golden.display()));

        assert_eq!(
            expected.trim_end(),
            output.trim_end(),
            "golden mismatch for fixture {}",
            case.display()
        );
    }
}
