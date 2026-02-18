use crate::diagnostics::{Diagnostic, Span};
use crate::manifest::Manifest;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ModuleImport {
    pub module_path: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ResolvedModuleSource {
    pub module_path: String,
    pub file_path: PathBuf,
    pub raw_source: String,
    pub source_without_uses: String,
    pub imports: Vec<ModuleImport>,
}

#[derive(Debug, Clone)]
pub struct ResolvedProjectSources {
    pub modules: Vec<ResolvedModuleSource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Visited,
}

pub fn resolve_project_modules(
    project_root: &Path,
    manifest: &Manifest,
) -> Result<ResolvedProjectSources, Vec<Diagnostic>> {
    let entry_path = manifest.entry_path(project_root);
    let source_root = determine_source_root(project_root, &entry_path);
    resolve_modules_from_entry(source_root.as_path(), entry_path.as_path())
}

pub fn resolve_modules_from_entry(
    source_root: &Path,
    entry_path: &Path,
) -> Result<ResolvedProjectSources, Vec<Diagnostic>> {
    let module_candidates = build_module_candidates(source_root)?;
    let entry_module = derive_module_name(source_root, entry_path)?;

    let mut states: HashMap<String, VisitState> = HashMap::new();
    let mut loaded: HashMap<String, ResolvedModuleSource> = HashMap::new();
    let mut order = Vec::new();
    let mut stack = Vec::new();
    let mut diagnostics = Vec::new();

    let entry_source = load_module_source(&entry_module, entry_path)?;
    loaded.insert(entry_module.clone(), entry_source);

    visit_module(
        &entry_module,
        &module_candidates,
        &mut states,
        &mut loaded,
        &mut order,
        &mut stack,
        &mut diagnostics,
    );

    if !diagnostics.is_empty() {
        diagnostics.sort_by(|a, b| {
            a.code
                .cmp(&b.code)
                .then_with(|| a.span.file.cmp(&b.span.file))
                .then_with(|| a.span.start_line.cmp(&b.span.start_line))
                .then_with(|| a.span.start_col.cmp(&b.span.start_col))
                .then_with(|| a.message.cmp(&b.message))
        });
        return Err(diagnostics);
    }

    let modules = order
        .into_iter()
        .map(|module| {
            loaded
                .remove(&module)
                .expect("resolved module should be loaded")
        })
        .collect::<Vec<_>>();

    Ok(ResolvedProjectSources { modules })
}

fn determine_source_root(project_root: &Path, entry_path: &Path) -> PathBuf {
    let src_root = project_root.join("src");
    if entry_path.starts_with(&src_root) {
        src_root
    } else {
        entry_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| project_root.to_path_buf())
    }
}

fn build_module_candidates(source_root: &Path) -> Result<BTreeMap<String, Vec<PathBuf>>, Vec<Diagnostic>> {
    let mut files = Vec::new();
    collect_ut_files_recursive(source_root, &mut files)?;
    files.sort();

    let mut candidates: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for file in files {
        let module_path = derive_module_name(source_root, &file)?;
        candidates.entry(module_path).or_default().push(file);
    }

    Ok(candidates)
}

fn collect_ut_files_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), Vec<Diagnostic>> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_file() {
        if is_ut_file(path) {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    let entries = fs::read_dir(path).map_err(|err| {
        vec![
            Diagnostic::error(
                "M0301",
                "could not read module directory",
                Span::point(path.to_path_buf(), 1, 1),
            )
            .with_note(err.to_string()),
        ]
    })?;

    let mut children = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| {
            vec![
                Diagnostic::error(
                    "M0301",
                    "could not read module directory entry",
                    Span::point(path.to_path_buf(), 1, 1),
                )
                .with_note(err.to_string()),
            ]
        })?;
        children.push(entry.path());
    }
    children.sort();

    for child in children {
        if child.is_dir() {
            collect_ut_files_recursive(&child, files)?;
        } else if is_ut_file(&child) {
            files.push(child);
        }
    }

    Ok(())
}

fn derive_module_name(source_root: &Path, file_path: &Path) -> Result<String, Vec<Diagnostic>> {
    let rel = file_path.strip_prefix(source_root).map_err(|_| {
        vec![
            Diagnostic::error(
                "M0302",
                "module file must be under source root",
                Span::point(file_path.to_path_buf(), 1, 1),
            )
            .with_note(format!(
                "source root: {}",
                source_root.display()
            )),
        ]
    })?;

    let mut parts = rel
        .iter()
        .map(|part| part.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return Err(vec![Diagnostic::error(
            "M0302",
            "module file path is invalid",
            Span::point(file_path.to_path_buf(), 1, 1),
        )]);
    }

    let file_name = parts.pop().expect("file name should exist");
    let stem = Path::new(&file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    if stem == "mod" {
        if parts.is_empty() {
            return Err(vec![
                Diagnostic::error(
                    "M0302",
                    "root `mod.ut` is not supported",
                    Span::point(file_path.to_path_buf(), 1, 1),
                )
                .with_note("rename the file (for example `main.ut`) or move it under a module directory"),
            ]);
        }
    } else {
        parts.push(stem.to_string());
    }

    Ok(parts.join("."))
}

fn is_ut_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ut"))
}

fn visit_module(
    module_name: &str,
    candidates: &BTreeMap<String, Vec<PathBuf>>,
    states: &mut HashMap<String, VisitState>,
    loaded: &mut HashMap<String, ResolvedModuleSource>,
    order: &mut Vec<String>,
    stack: &mut Vec<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if states.get(module_name) == Some(&VisitState::Visited) {
        return;
    }

    if states.get(module_name) == Some(&VisitState::Visiting) {
        return;
    }

    states.insert(module_name.to_string(), VisitState::Visiting);
    stack.push(module_name.to_string());

    let imports = loaded
        .get(module_name)
        .map(|module| module.imports.clone())
        .unwrap_or_default();

    for import in imports {
        match resolve_import_target(module_name, &import, candidates, loaded) {
            Ok(Some(target_path)) => {
                if states.get(&import.module_path) == Some(&VisitState::Visiting) {
                    diagnostics.push(cycle_diagnostic(&import, stack));
                    continue;
                }

                if !loaded.contains_key(&import.module_path) {
                    match load_module_source(&import.module_path, &target_path) {
                        Ok(module) => {
                            loaded.insert(import.module_path.clone(), module);
                        }
                        Err(diags) => {
                            diagnostics.extend(diags);
                            continue;
                        }
                    }
                }

                visit_module(
                    &import.module_path,
                    candidates,
                    states,
                    loaded,
                    order,
                    stack,
                    diagnostics,
                );
            }
            Ok(None) => {}
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    stack.pop();
    states.insert(module_name.to_string(), VisitState::Visited);
    if !order.iter().any(|name| name == module_name) {
        order.push(module_name.to_string());
    }
}

fn resolve_import_target(
    importer: &str,
    import: &ModuleImport,
    candidates: &BTreeMap<String, Vec<PathBuf>>,
    loaded: &HashMap<String, ResolvedModuleSource>,
) -> Result<Option<PathBuf>, Diagnostic> {
    let Some(paths) = candidates.get(&import.module_path) else {
        let mut diagnostic = Diagnostic::error("M0303", "module import not found", import.span.clone())
            .with_note(format!("importer module: {importer}"))
            .with_note(format!("missing module: {}", import.module_path));
        for suggestion in suggest_module_paths(import.module_path.as_str(), candidates) {
            diagnostic = diagnostic.with_note(format!("did you mean: {suggestion}"));
        }
        return Err(diagnostic);
    };

    if paths.len() > 1 {
        let mut diagnostic =
            Diagnostic::error("M0304", "ambiguous module import target", import.span.clone())
                .with_note(format!("importer module: {importer}"))
                .with_note(format!("module: {}", import.module_path));
        for candidate in paths {
            diagnostic = diagnostic.with_note(format!("candidate: {}", candidate.display()));
        }
        return Err(diagnostic);
    }

    let target = paths[0].clone();
    if let Some(existing) = loaded.get(&import.module_path) {
        if existing.file_path != target {
            return Err(
                Diagnostic::error("M0304", "ambiguous module import target", import.span.clone())
                    .with_note(format!("importer module: {importer}"))
                    .with_note(format!("existing target: {}", existing.file_path.display()))
                    .with_note(format!("candidate target: {}", target.display())),
            );
        }
    }

    Ok(Some(target))
}

fn suggest_module_paths(
    missing_module_path: &str,
    candidates: &BTreeMap<String, Vec<PathBuf>>,
) -> Vec<String> {
    let threshold = std::cmp::max(2usize, missing_module_path.chars().count() / 3);
    let mut scored = candidates
        .keys()
        .map(|candidate| {
            (
                levenshtein_distance(missing_module_path, candidate.as_str()),
                candidate.clone(),
            )
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    scored
        .into_iter()
        .filter(|(distance, _)| *distance <= threshold)
        .take(3)
        .map(|(_, candidate)| candidate)
        .collect()
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    if left == right {
        return 0;
    }
    if left.is_empty() {
        return right.chars().count();
    }
    if right.is_empty() {
        return left.chars().count();
    }

    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right_chars.len()).collect::<Vec<_>>();
    let mut current = vec![0usize; right_chars.len() + 1];

    for (left_index, left_char) in left_chars.iter().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_char) in right_chars.iter().enumerate() {
            let substitution_cost = usize::from(left_char != right_char);
            let deletion = previous[right_index + 1] + 1;
            let insertion = current[right_index] + 1;
            let substitution = previous[right_index] + substitution_cost;
            current[right_index + 1] = deletion.min(insertion).min(substitution);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right_chars.len()]
}

fn cycle_diagnostic(import: &ModuleImport, stack: &[String]) -> Diagnostic {
    let start = stack
        .iter()
        .position(|module| module == &import.module_path)
        .unwrap_or(0);
    let mut chain = stack[start..].to_vec();
    chain.push(import.module_path.clone());

    Diagnostic::error("M0305", "cyclic module dependency", import.span.clone())
        .with_note(format!("cycle: {}", chain.join(" -> ")))
}

fn load_module_source(module_path: &str, file_path: &Path) -> Result<ResolvedModuleSource, Vec<Diagnostic>> {
    let raw_source = fs::read_to_string(file_path).map_err(|err| {
        vec![
            Diagnostic::error(
                "M0301",
                "could not read module source file",
                Span::point(file_path.to_path_buf(), 1, 1),
            )
            .with_note(err.to_string()),
        ]
    })?;

    let (imports, source_without_uses) = parse_use_directives(file_path, &raw_source)?;

    Ok(ResolvedModuleSource {
        module_path: module_path.to_string(),
        file_path: file_path.to_path_buf(),
        raw_source,
        source_without_uses,
        imports,
    })
}

fn parse_use_directives(
    file_path: &Path,
    source: &str,
) -> Result<(Vec<ModuleImport>, String), Vec<Diagnostic>> {
    let mut imports = Vec::new();
    let mut parser_lines = Vec::new();
    let mut diagnostics = Vec::new();
    let mut saw_declaration = false;

    for (index, line) in source.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") {
            parser_lines.push(line.to_string());
            continue;
        }

        if starts_with_use_keyword(trimmed) {
            if saw_declaration {
                let col = line.find('u').unwrap_or(0) + 1;
                diagnostics.push(
                    Diagnostic::error(
                        "M0307",
                        "`use` directives must appear before declarations",
                        Span::point(file_path.to_path_buf(), line_no, col),
                    )
                    .with_note("move module imports to the top of the file"),
                );
                parser_lines.push(line.to_string());
                continue;
            }

            match parse_use_module_path(trimmed, file_path, line_no, line) {
                Ok(import) => {
                    imports.push(import);
                    parser_lines.push(String::new());
                }
                Err(diagnostic) => {
                    diagnostics.push(diagnostic);
                    parser_lines.push(line.to_string());
                }
            }
            continue;
        }

        saw_declaration = true;
        parser_lines.push(line.to_string());
    }

    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut source_without_uses = parser_lines.join("\n");
    if source.ends_with('\n') {
        source_without_uses.push('\n');
    }

    Ok((imports, source_without_uses))
}

fn starts_with_use_keyword(trimmed: &str) -> bool {
    trimmed == "use" || trimmed.starts_with("use ") || trimmed.starts_with("use\t")
}

fn parse_use_module_path(
    trimmed: &str,
    file_path: &Path,
    line_no: usize,
    line: &str,
) -> Result<ModuleImport, Diagnostic> {
    let Some(rest) = trimmed.strip_prefix("use") else {
        return Err(Diagnostic::error(
            "M0306",
            "invalid `use` directive",
            Span::point(file_path.to_path_buf(), line_no, 1),
        ));
    };

    let module_with_semicolon = rest.trim_start();
    if !module_with_semicolon.ends_with(';') {
        let col = line.find('u').unwrap_or(0) + 1;
        return Err(
            Diagnostic::error(
                "M0306",
                "invalid `use` directive",
                Span::point(file_path.to_path_buf(), line_no, col),
            )
            .with_note("expected syntax: use module.path;"),
        );
    }

    let module_path = module_with_semicolon
        .trim_end_matches(';')
        .trim();
    if module_path.is_empty() {
        let col = line.find('u').unwrap_or(0) + 1;
        return Err(
            Diagnostic::error(
                "M0306",
                "invalid `use` directive",
                Span::point(file_path.to_path_buf(), line_no, col),
            )
            .with_note("expected syntax: use module.path;"),
        );
    }

    if !is_valid_module_path(module_path) {
        let col = line.find('u').unwrap_or(0) + 1;
        return Err(
            Diagnostic::error(
                "M0306",
                "invalid module path in `use` directive",
                Span::point(file_path.to_path_buf(), line_no, col),
            )
            .with_note(format!("found: `{module_path}`"))
            .with_note("module path segments must match [A-Za-z_][A-Za-z0-9_]*"),
        );
    }

    let col = line.find('u').unwrap_or(0) + 1;
    Ok(ModuleImport {
        module_path: module_path.to_string(),
        span: Span::point(file_path.to_path_buf(), line_no, col),
    })
}

fn is_valid_module_path(module_path: &str) -> bool {
    module_path.split('.').all(is_valid_module_segment)
}

fn is_valid_module_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }

    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::resolve_project_modules;
    use crate::manifest::parse_manifest_str;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&dir).expect("temp directory should be created");
        dir
    }

    fn write_manifest(root: &Path) {
        fs::write(
            root.join("sec4.toml"),
            "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n",
        )
        .expect("manifest should be written");
    }

    fn load_manifest(root: &Path) -> crate::manifest::Manifest {
        let manifest_path = root.join("sec4.toml");
        let source = fs::read_to_string(&manifest_path).expect("manifest should be readable");
        parse_manifest_str(&manifest_path, &source).expect("manifest should parse")
    }

    #[test]
    fn resolves_multi_file_module_graph() {
        let root = temp_dir("sec4-modules-pass");
        fs::create_dir_all(root.join("src")).expect("src should exist");
        write_manifest(&root);
        fs::write(root.join("src/main.ut"), "use util;\n\nfn main() -> Int {\n  helper();\n  0\n}\n")
            .expect("main should be written");
        fs::write(root.join("src/util.ut"), "fn helper() -> Int {\n  0\n}\n")
            .expect("util should be written");

        let manifest = load_manifest(&root);
        let resolved = resolve_project_modules(&root, &manifest).expect("module graph should resolve");

        assert_eq!(resolved.modules.len(), 2);
        assert_eq!(resolved.modules[0].module_path, "util");
        assert_eq!(resolved.modules[1].module_path, "main");
        assert!(resolved.modules[1].imports.iter().any(|import| import.module_path == "util"));

        fs::remove_dir_all(root).expect("temp cleanup should succeed");
    }

    #[test]
    fn reports_missing_module_import() {
        let root = temp_dir("sec4-modules-missing");
        fs::create_dir_all(root.join("src")).expect("src should exist");
        write_manifest(&root);
        fs::write(root.join("src/main.ut"), "use missing.module;\n\nfn main() -> Int {\n  0\n}\n")
            .expect("main should be written");

        let manifest = load_manifest(&root);
        let diagnostics = resolve_project_modules(&root, &manifest)
            .expect_err("missing module should fail resolution");

        assert!(diagnostics.iter().any(|diag| diag.code == "M0303"));

        fs::remove_dir_all(root).expect("temp cleanup should succeed");
    }

    #[test]
    fn missing_module_import_includes_close_match_suggestion() {
        let root = temp_dir("sec4-modules-missing-suggestion");
        fs::create_dir_all(root.join("src")).expect("src should exist");
        write_manifest(&root);
        fs::write(root.join("src/main.ut"), "use utl;\n\nfn main() -> Int {\n  helper();\n  0\n}\n")
            .expect("main should be written");
        fs::write(root.join("src/util.ut"), "fn helper() -> Int {\n  0\n}\n")
            .expect("util should be written");

        let manifest = load_manifest(&root);
        let diagnostics = resolve_project_modules(&root, &manifest)
            .expect_err("missing module should fail resolution with suggestions");

        let missing_import = diagnostics
            .iter()
            .find(|diag| diag.code == "M0303")
            .expect("missing import diagnostic should exist");
        assert!(
            missing_import.notes.iter().any(|note| note == "did you mean: util"),
            "missing import diagnostics should include deterministic close-match suggestion: {:?}",
            missing_import.notes
        );

        fs::remove_dir_all(root).expect("temp cleanup should succeed");
    }

    #[test]
    fn reports_ambiguous_module_import_target() {
        let root = temp_dir("sec4-modules-ambiguous");
        fs::create_dir_all(root.join("src/foo")).expect("src/foo should exist");
        write_manifest(&root);
        fs::write(root.join("src/main.ut"), "use foo;\n\nfn main() -> Int {\n  0\n}\n")
            .expect("main should be written");
        fs::write(root.join("src/foo.ut"), "fn from_file() -> Int {\n  0\n}\n")
            .expect("foo file should be written");
        fs::write(root.join("src/foo/mod.ut"), "fn from_mod() -> Int {\n  0\n}\n")
            .expect("foo mod file should be written");

        let manifest = load_manifest(&root);
        let diagnostics = resolve_project_modules(&root, &manifest)
            .expect_err("ambiguous module should fail resolution");

        assert!(diagnostics.iter().any(|diag| diag.code == "M0304"));

        fs::remove_dir_all(root).expect("temp cleanup should succeed");
    }

    #[test]
    fn reports_cyclic_module_dependency() {
        let root = temp_dir("sec4-modules-cycle");
        fs::create_dir_all(root.join("src")).expect("src should exist");
        write_manifest(&root);
        fs::write(root.join("src/main.ut"), "use a;\n\nfn main() -> Int {\n  0\n}\n")
            .expect("main should be written");
        fs::write(root.join("src/a.ut"), "use b;\n\nfn fa() -> Int {\n  0\n}\n")
            .expect("a should be written");
        fs::write(root.join("src/b.ut"), "use a;\n\nfn fb() -> Int {\n  0\n}\n")
            .expect("b should be written");

        let manifest = load_manifest(&root);
        let diagnostics = resolve_project_modules(&root, &manifest)
            .expect_err("cycle should fail resolution");

        assert!(diagnostics.iter().any(|diag| diag.code == "M0305"));

        fs::remove_dir_all(root).expect("temp cleanup should succeed");
    }
}
