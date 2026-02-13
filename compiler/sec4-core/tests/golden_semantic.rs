use sec4_core::{analyze_program, parse_source};
use std::fs;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/semantic")
}

fn collect_case_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(dir)
        .expect("fixtures directory should exist")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "ut"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn semantic_fixtures_match_golden_output() {
    let dir = fixtures_dir();
    let case_files = collect_case_files(&dir);
    assert!(!case_files.is_empty(), "expected semantic fixtures");

    for case in case_files {
        let input = fs::read_to_string(&case).expect("fixture should be readable");
        let virtual_path = Path::new(
            case.file_name()
                .and_then(|name| name.to_str())
                .expect("fixture file should have UTF-8 name"),
        );

        let output = match parse_source(virtual_path, &input) {
            Ok(program) => match analyze_program(&program) {
                Ok(()) => "OK".to_string(),
                Err(diags) => render_diags(&diags),
            },
            Err(diags) => render_diags(&diags),
        };

        let golden = case.with_extension("golden");
        let expected = fs::read_to_string(&golden)
            .unwrap_or_else(|_| panic!("missing golden file: {}", golden.display()));

        assert_eq!(
            normalize_golden_text(&expected),
            normalize_golden_text(&output),
            "golden mismatch for fixture {}",
            case.display()
        );
    }
}

fn render_diags(diags: &[sec4_core::Diagnostic]) -> String {
    let mut rendered = String::new();
    for (index, diagnostic) in diags.iter().enumerate() {
        if index > 0 {
            rendered.push_str("\n\n");
        }
        rendered.push_str(&diagnostic.render_plain());
    }
    rendered
}

fn normalize_golden_text(input: &str) -> String {
    input
        .lines()
        .filter(|line| !line.trim_start().starts_with("tags: "))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}
