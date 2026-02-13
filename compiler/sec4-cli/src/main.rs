use clap::{Args, Parser, Subcommand, ValueEnum};
use sec4_core::{
    analyze_entry, analyze_entry_with_allows, build_security_map_with_allows, emit_c_program,
    emit_runtime_header, emit_runtime_source, render_security_audit_text,
    run_security_audit_with_baseline, should_fail, summarize_history_window,
    validate_lockfile_stub, write_build_metadata, write_lockfile_stub, write_sbom,
    write_security_map, AuditHistoryWindowSummary, AuditReport, AuditSeverity, Diagnostic,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(
    name = "sec4",
    version,
    about = "Untrusted<T> compiler CLI (M1 parser)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Build {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum)]
        emit: Option<BuildEmitTarget>,
        #[arg(long, default_value_t = false)]
        locked: bool,
        #[arg(long, default_value_t = false)]
        sbom: bool,
    },
    Run {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    Check {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum)]
        emit: Option<EmitTarget>,
    },
    Test {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    Fmt {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    Lint {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    Audit(AuditArgs),
    Gate {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = AuditOutputFormat::Text)]
        format: AuditOutputFormat,
        #[arg(long)]
        fail_on: Option<String>,
    },
    Explain {
        code: String,
    },
}

#[derive(Args, Debug)]
struct AuditArgs {
    #[arg(long, default_value = ".")]
    path: PathBuf,
    #[arg(long, value_enum, default_value_t = AuditOutputFormat::Text)]
    format: AuditOutputFormat,
    #[arg(long)]
    baseline: Option<PathBuf>,
    #[arg(long)]
    history_dir: Option<PathBuf>,
    #[arg(long)]
    history_window: Option<usize>,
    #[arg(long)]
    write_history_summary: Option<PathBuf>,
    #[arg(long)]
    write_report: Option<PathBuf>,
    #[arg(long)]
    fail_on: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum AuditOutputFormat {
    Text,
    Json,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum BuildEmitTarget {
    Mir,
    MirJson,
    C,
    CBin,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum EmitTarget {
    Ast,
    DiagnosticsJson,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build {
            path,
            emit,
            locked,
            sbom,
        } => cmd_build(&path, emit, locked, sbom),
        Commands::Check { path, emit } => cmd_check(&path, emit),
        Commands::Run { path } => cmd_run(&path),
        Commands::Test { path } => cmd_test(&path),
        Commands::Fmt { path } => cmd_fmt(&path),
        Commands::Lint { path } => cmd_lint(&path),
        Commands::Audit(args) => cmd_audit(args),
        Commands::Gate {
            path,
            format,
            fail_on,
        } => cmd_sec_audit(
            &path,
            format,
            None,
            None,
            None,
            None,
            None,
            Some(fail_on.as_deref().unwrap_or("risk>=HIGH")),
        ),
        Commands::Explain { code } => cmd_explain(&code),
    };

    if let Err(code) = result {
        std::process::exit(code);
    }
}

fn cmd_audit(args: AuditArgs) -> Result<(), i32> {
    cmd_sec_audit(
        &args.path,
        args.format,
        args.baseline.as_deref(),
        args.history_dir.as_deref(),
        args.history_window,
        args.write_history_summary.as_deref(),
        args.write_report.as_deref(),
        args.fail_on.as_deref(),
    )
}

fn cmd_explain(code: &str) -> Result<(), i32> {
    let code = code.trim();
    if code.is_empty() {
        eprintln!("explain requires a diagnostic code");
        return Err(2);
    }

    let normalized = code.to_ascii_uppercase();
    let (topic, summary, fixes, docs_path) = explain_topic(&normalized);

    println!("{normalized} - {topic}");
    println!("{summary}");
    println!();
    println!("Likely actions:");
    for fix in fixes {
        println!("- {fix}");
    }
    println!();
    println!("Related commands:");
    println!("- sec4 check --emit diagnostics-json");
    println!("- sec4 audit --format text");
    println!("- sec4 gate --fail-on 'risk>=HIGH'");
    println!("Docs: {docs_path}");

    Ok(())
}

fn explain_topic(
    code: &str,
) -> (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static str,
) {
    const TRUST_FIXES: [&str; 3] = [
        "introduce an explicit trust gate (schema decode, validate.*, sanitize.*)",
        "pass typed sink values (SqlQuery/HtmlSafe/PublicUrl/PathSafe/HeaderValue)",
        "redact or remove Secret<_> values before log/json/formatting",
    ];
    const EFFECT_FIXES: [&str; 3] = [
        "declare all used effects in the function signature",
        "thread the required capability (DbCap/NetCap/FsCap/SecretsCap)",
        "check policy forbids/allowlist annotations for restricted effects",
    ];
    const TYPE_FIXES: [&str; 3] = [
        "inspect expected vs actual type at the primary span",
        "align shape fields and optional handling (Option/Result)",
        "prefer explicit conversion/gate calls over implicit assumptions",
    ];
    const SCHEMA_FIXES: [&str; 3] = [
        "add explicit schema descriptors where required (req.json/res.json/json.encode/decode)",
        "ensure schema/value type pairing is valid (Schema<T> with T)",
        "keep Budget and context arguments wired for decode paths",
    ];
    const TEMPLATE_FIXES: [&str; 3] = [
        "replace raw string interpolation with typed SQL/HTML helpers",
        "ensure SQL params are trusted non-secret values",
        "ensure HTML interpolation uses HtmlSafe values only",
    ];
    const POLICY_FIXES: [&str; 3] = [
        "review the effective policy profile for forbidden effects/features",
        "use narrow allowlist annotations with reason/ticket/expiry only when needed",
        "re-run sec4 audit and gate on severity thresholds in CI",
    ];
    const PARSE_FIXES: [&str; 3] = [
        "fix syntax around the highlighted token/span first",
        "re-run sec4 check to surface semantic diagnostics after parse recovery",
        "use --emit diagnostics-json for machine-readable span details",
    ];
    const BUDGET_FIXES: [&str; 3] = [
        "increase analysis/request budget settings for large files",
        "reduce file complexity while iterating (split modules, fewer open files)",
        "retry the same command/editor action after budget adjustment",
    ];
    const BUILD_FIXES: [&str; 3] = [
        "verify sec4.toml/sec4.lock entry path and project layout",
        "check policy/profile/build artifact paths exist and are readable",
        "re-run sec4 build with --locked only when lockfile is up to date",
    ];
    const UNKNOWN_FIXES: [&str; 3] = [
        "inspect the diagnostic span, notes, and tags first",
        "run sec4 check --emit diagnostics-json for full structured details",
        "search the roadmap/book chapters for the specific code family",
    ];

    match code {
        "E2003" => {
            return (
                "Missing Required Capability",
                "A sensitive API call was made without the required capability token in scope.",
                &EFFECT_FIXES,
                "docs/book/54-v0-stdlib-security-surface.md",
            );
        }
        "E4004" => {
            return (
                "Invalid Schema Contract Usage",
                "A schema-required API was called with missing/invalid schema descriptors or mismatched value shape.",
                &SCHEMA_FIXES,
                "docs/book/54-v0-stdlib-security-surface.md",
            );
        }
        "E5001" => {
            return (
                "Invalid SQL Template Parameter",
                "A SQL template parameter is not an allowed trusted parameter shape for typed query construction.",
                &TEMPLATE_FIXES,
                "docs/book/55-v0-typing-effects-security-rules.md",
            );
        }
        "E6001" => {
            return (
                "Internal Network Policy Block",
                "Internal network access is forbidden by active policy or missing explicit allowlist governance.",
                &POLICY_FIXES,
                "docs/book/64-sec-audit-spec.md",
            );
        }
        _ => {}
    }

    if code == "I9001" {
        return (
            "Analysis Budget/Deadline Interruption",
            "The compiler or language server stopped early because a budget/deadline interrupt fired.",
            &BUDGET_FIXES,
            "docs/book/260-m11-diagnostics-analysis-budget-guardrails.md",
        );
    }

    if code.starts_with('L') || code.starts_with('P') {
        return (
            "Lexing/Parsing Syntax Issue",
            "Source text could not be parsed into a valid program shape at the highlighted span.",
            &PARSE_FIXES,
            "docs/book/31-parser-design-and-ast.md",
        );
    }

    if code.starts_with("E1") {
        return (
            "Trust Boundary and Secret Flow",
            "A value crossed a security boundary without the required trusted/safe type or secret handling rule.",
            &TRUST_FIXES,
            "docs/book/53-v0-security-baseline.md",
        );
    }

    if code.starts_with("E2") {
        return (
            "Effects and Capability Contract",
            "An effect declaration/capability requirement was missing, mismatched, or forbidden by policy.",
            &EFFECT_FIXES,
            "docs/book/50-effect-system-and-auditable-side-effects.md",
        );
    }

    if code.starts_with("E3") {
        return (
            "Type/Shape Compatibility",
            "The expression or shape does not match the required type contract.",
            &TYPE_FIXES,
            "docs/book/41-type-system-v0.1-lite.md",
        );
    }

    if code.starts_with("E4") {
        return (
            "Schema and Encoding/Decoding Contract",
            "Schema-gated APIs were called with invalid descriptors, arity, or value pairings.",
            &SCHEMA_FIXES,
            "docs/book/54-v0-stdlib-security-surface.md",
        );
    }

    if code.starts_with("E5") {
        return (
            "SQL/HTML Template Safety",
            "Template/sink restrictions were violated for SQL or HTML safe construction.",
            &TEMPLATE_FIXES,
            "docs/book/55-v0-typing-effects-security-rules.md",
        );
    }

    if code.starts_with("E6") {
        return (
            "Policy Enforcement Violation",
            "A policy rule blocked the operation or required additional allowlist constraints.",
            &POLICY_FIXES,
            "docs/book/64-sec-audit-spec.md",
        );
    }

    if code.starts_with('M') {
        return (
            "Manifest/Build/Artifact Contract",
            "Project manifest, lockfile, or build artifact validation failed.",
            &BUILD_FIXES,
            "docs/book/11-cli-and-build-lifecycle.md",
        );
    }

    (
        "Unknown Diagnostic Family",
        "The code family is not yet mapped by sec4 explain; use structured diagnostics and docs to triage.",
        &UNKNOWN_FIXES,
        "docs/05-sec4-master-roadmap.md",
    )
}

fn cmd_sec_audit(
    path: &Path,
    format: AuditOutputFormat,
    baseline_path: Option<&Path>,
    history_dir_path: Option<&Path>,
    history_window: Option<usize>,
    write_history_summary_path: Option<&Path>,
    write_report_path: Option<&Path>,
    fail_on: Option<&str>,
) -> Result<(), i32> {
    if history_window.is_some() && history_dir_path.is_none() {
        eprintln!("--history-window requires --history-dir");
        return Err(2);
    }
    if history_window.is_some_and(|window| window == 0) {
        eprintln!("--history-window must be >= 1");
        return Err(2);
    }
    if write_history_summary_path.is_some() && history_window.is_none() {
        eprintln!("--write-history-summary requires --history-window");
        return Err(2);
    }

    match sec4_core::validate_project(path) {
        Ok(manifest) => match analyze_entry_with_allows(path, &manifest) {
            Ok((program, allows)) => {
                let policy = match sec4_core::policy::load_policy(path) {
                    Ok(policy) => policy,
                    Err(diagnostics) => {
                        print_diagnostics(&diagnostics);
                        return Err(1);
                    }
                };

                let security_map = build_security_map_with_allows(&program, &policy, allows);
                let security_map_path = match write_security_map(path, &security_map) {
                    Ok(output_path) => output_path,
                    Err(diagnostic) => {
                        print_diagnostics(&[diagnostic]);
                        return Err(1);
                    }
                };

                let mut baseline_source = None;
                let baseline_report = if let Some(baseline_path) = baseline_path {
                    baseline_source = Some(baseline_path.to_path_buf());
                    Some(load_audit_baseline(baseline_path)?)
                } else if let Some(history_dir_path) = history_dir_path {
                    if let Some((report, source_path)) =
                        load_latest_history_baseline(history_dir_path)?
                    {
                        baseline_source = Some(source_path);
                        Some(report)
                    } else {
                        None
                    }
                } else {
                    None
                };
                let mut report = run_security_audit_with_baseline(
                    &policy,
                    &security_map,
                    baseline_report.as_ref(),
                );
                if let (Some(history_dir_path), Some(window)) = (history_dir_path, history_window) {
                    report.history_window =
                        compute_history_window_summary(history_dir_path, window, &report)?;
                }
                match format {
                    AuditOutputFormat::Text => println!("{}", render_security_audit_text(&report)),
                    AuditOutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report)
                                .expect("security audit report should serialize")
                        );
                    }
                }

                if let Some(write_report_path) = write_report_path {
                    if let Err(code) = write_audit_report(write_report_path, &report) {
                        return Err(code);
                    }
                }

                let history_report_path = if let Some(history_dir_path) = history_dir_path {
                    Some(write_history_report(history_dir_path, &report)?)
                } else {
                    None
                };

                if let Some(baseline_source) = baseline_source {
                    print_aux_line(
                        format,
                        &format!("baseline report: {}", baseline_source.display()),
                    );
                }
                if let Some(history_report_path) = history_report_path {
                    print_aux_line(
                        format,
                        &format!("history report: {}", history_report_path.display()),
                    );
                }
                print_aux_line(
                    format,
                    &format!("security map: {}", security_map_path.display()),
                );
                if let Some(summary) = report.history_window.as_ref() {
                    print_history_window_summary(format, summary);
                }
                if let Some(write_history_summary_path) = write_history_summary_path {
                    let Some(summary) = report.history_window.as_ref() else {
                        eprintln!("history-window summary not available to write");
                        return Err(2);
                    };
                    write_history_window_summary(write_history_summary_path, summary)?;
                    print_aux_line(
                        format,
                        &format!("history summary: {}", write_history_summary_path.display()),
                    );
                }

                if let Some(threshold) = fail_on {
                    let Some(threshold) = AuditSeverity::parse_threshold(threshold) else {
                        eprintln!(
                            "invalid --fail-on value `{}`; use values like `risk>=HIGH`, `HIGH`, `MEDIUM`",
                            threshold
                        );
                        return Err(2);
                    };

                    if should_fail(&report, threshold) {
                        eprintln!(
                            "security audit failed: findings at or above threshold {:?}",
                            threshold
                        );
                        return Err(1);
                    }
                }

                Ok(())
            }
            Err(diagnostics) => {
                print_diagnostics(&diagnostics);
                Err(1)
            }
        },
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            Err(1)
        }
    }
}

fn print_aux_line(format: AuditOutputFormat, line: &str) {
    if matches!(format, AuditOutputFormat::Json) {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }
}

fn load_audit_baseline(path: &Path) -> Result<AuditReport, i32> {
    let raw = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("could not read baseline report `{}`: {err}", path.display());
            return Err(2);
        }
    };

    match serde_json::from_str::<AuditReport>(&raw) {
        Ok(report) => Ok(report),
        Err(err) => {
            eprintln!(
                "could not parse baseline report `{}` as audit JSON: {err}",
                path.display()
            );
            Err(2)
        }
    }
}

fn load_latest_history_baseline(history_dir: &Path) -> Result<Option<(AuditReport, PathBuf)>, i32> {
    if !history_dir.exists() {
        return Ok(None);
    }

    let entries = match fs::read_dir(history_dir) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!(
                "could not read history directory `{}`: {err}",
                history_dir.display()
            );
            return Err(2);
        }
    };

    let mut candidates = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Ok(None);
    }

    candidates.sort();
    let latest = candidates
        .pop()
        .expect("history candidate list should not be empty");
    let report = load_audit_baseline(&latest)?;
    Ok(Some((report, latest)))
}

fn load_recent_history_reports(history_dir: &Path, window: usize) -> Result<Vec<AuditReport>, i32> {
    if !history_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = match fs::read_dir(history_dir) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!(
                "could not read history directory `{}`: {err}",
                history_dir.display()
            );
            return Err(2);
        }
    };

    let mut candidates = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    if window == 0 {
        return Ok(Vec::new());
    }

    candidates.sort();
    let keep = window;
    let start = candidates.len().saturating_sub(keep);
    let mut reports = Vec::new();
    for path in candidates.into_iter().skip(start) {
        let report = load_audit_baseline(&path)?;
        reports.push(report);
    }
    Ok(reports)
}

fn compute_history_window_summary(
    history_dir: &Path,
    window: usize,
    current_report: &AuditReport,
) -> Result<Option<AuditHistoryWindowSummary>, i32> {
    let mut reports = load_recent_history_reports(history_dir, window.saturating_sub(1))?;
    reports.push(current_report.clone());
    Ok(summarize_history_window(&reports, window))
}

fn print_history_window_summary(format: AuditOutputFormat, summary: &AuditHistoryWindowSummary) {
    match format {
        AuditOutputFormat::Text => {
            print_aux_line(
                format,
                &format!(
                    "history window summary: reports={}/{}, oldestRisk={}, latestRisk={}, minRisk={}, maxRisk={}, riskDelta={}, avgRisk={:.2}, highestSeen={}, oldestTimeMs={}, latestTimeMs={}",
                    summary.reports,
                    summary.window,
                    summary.oldest_risk_score,
                    summary.latest_risk_score,
                    summary.min_risk_score,
                    summary.max_risk_score,
                    summary.risk_score_delta,
                    summary.average_risk_score,
                    summary.highest_severity_seen,
                    summary.oldest_time_ms,
                    summary.latest_time_ms,
                ),
            );
        }
        AuditOutputFormat::Json => {
            print_aux_line(
                format,
                &format!(
                    "history window summary: {}",
                    serde_json::to_string(summary)
                        .expect("history window summary payload should serialize")
                ),
            );
        }
    }
}

fn write_history_window_summary(
    path: &Path,
    summary: &AuditHistoryWindowSummary,
) -> Result<(), i32> {
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!(
                "could not create history-summary directory `{}`: {err}",
                parent.display()
            );
            return Err(2);
        }
    }

    let serialized = match serde_json::to_string_pretty(summary) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("could not serialize history summary: {err}");
            return Err(2);
        }
    };

    if let Err(err) = fs::write(path, serialized) {
        eprintln!(
            "could not write history summary `{}`: {err}",
            path.display()
        );
        return Err(2);
    }

    Ok(())
}

fn write_audit_report(path: &Path, report: &AuditReport) -> Result<(), i32> {
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!(
                "could not create report directory `{}`: {err}",
                parent.display()
            );
            return Err(2);
        }
    }

    let serialized = match serde_json::to_string_pretty(report) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("could not serialize audit report: {err}");
            return Err(2);
        }
    };

    if let Err(err) = fs::write(path, serialized) {
        eprintln!("could not write audit report `{}`: {err}", path.display());
        return Err(2);
    }
    Ok(())
}

fn write_history_report(history_dir: &Path, report: &AuditReport) -> Result<PathBuf, i32> {
    if let Err(err) = fs::create_dir_all(history_dir) {
        eprintln!(
            "could not create history directory `{}`: {err}",
            history_dir.display()
        );
        return Err(2);
    }

    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let file_name = format!("audit-{}-{now_nanos}.json", report.build.time_ms);
    let output_path = history_dir.join(file_name);
    write_audit_report(&output_path, report)?;
    Ok(output_path)
}

fn cmd_build(
    path: &Path,
    emit: Option<BuildEmitTarget>,
    locked: bool,
    sbom: bool,
) -> Result<(), i32> {
    let mir_json_mode = matches!(emit, Some(BuildEmitTarget::MirJson));
    match sec4_core::validate_project(path) {
        Ok(manifest) => {
            let program = match analyze_entry(path, &manifest) {
                Ok(program) => program,
                Err(diagnostics) => {
                    print_diagnostics(&diagnostics);
                    return Err(1);
                }
            };

            if locked {
                if let Err(diag) = validate_lockfile_stub(path, &manifest) {
                    print_diagnostics(&[diag]);
                    return Err(1);
                }
            } else if let Err(diag) = write_lockfile_stub(path, &manifest) {
                print_diagnostics(&[diag]);
                return Err(1);
            }

            let policy = match sec4_core::policy::load_policy(path) {
                Ok(policy) => policy,
                Err(diagnostics) => {
                    print_diagnostics(&diagnostics);
                    return Err(1);
                }
            };
            let build_metadata_path = match write_build_metadata(path, &manifest, &policy) {
                Ok(path) => path,
                Err(diag) => {
                    print_diagnostics(&[diag]);
                    return Err(1);
                }
            };
            let sbom_path = if sbom {
                match std::fs::read_to_string(&build_metadata_path)
                    .ok()
                    .and_then(|raw| serde_json::from_str::<sec4_core::BuildMetadata>(&raw).ok())
                {
                    Some(metadata) => match write_sbom(path, &metadata) {
                        Ok(path) => Some(path),
                        Err(diag) => {
                            print_diagnostics(&[diag]);
                            return Err(1);
                        }
                    },
                    None => {
                        eprintln!(
                            "could not parse build metadata from `{}` before sbom generation",
                            build_metadata_path.display()
                        );
                        return Err(1);
                    }
                }
            } else {
                None
            };

            let mir = emit.map(|_| sec4_core::lower_program_to_mir(&program));
            let c_source = if matches!(emit, Some(BuildEmitTarget::C | BuildEmitTarget::CBin)) {
                Some(emit_c_program(
                    mir.as_ref()
                        .expect("MIR should be lowered when emit target is set"),
                ))
            } else {
                None
            };

            if !mir_json_mode {
                println!(
                    "build succeeded (M3 effects): package={}, entry={}",
                    manifest.package.name,
                    manifest.entry_file()
                );
                if locked {
                    println!("verified lockfile: {}", path.join("sec4.lock").display());
                } else {
                    println!("wrote lockfile stub: {}", path.join("sec4.lock").display());
                }
                println!("wrote build metadata: {}", build_metadata_path.display());
                if let Some(sbom_path) = sbom_path {
                    println!("wrote sbom: {}", sbom_path.display());
                }
            }

            match emit {
                Some(BuildEmitTarget::Mir) => {
                    println!(
                        "{}",
                        mir.as_ref()
                            .expect("MIR should be lowered when emit target is set")
                            .render_text()
                    );
                }
                Some(BuildEmitTarget::MirJson) => {
                    println!(
                        "{}",
                        mir.as_ref()
                            .expect("MIR should be lowered when emit target is set")
                            .to_pretty_json()
                    );
                }
                Some(BuildEmitTarget::C) => {
                    println!(
                        "{}",
                        c_source
                            .as_ref()
                            .expect("C source should be available for c emit target")
                    );
                }
                Some(BuildEmitTarget::CBin) => {
                    let (c_path, bin_path) = compile_c_binary(
                        path,
                        &manifest.package.name,
                        c_source
                            .as_ref()
                            .expect("C source should be available for c-bin emit target"),
                    )?;
                    println!("generated c source: {}", c_path.display());
                    println!("compiled binary: {}", bin_path.display());
                }
                None => {}
            }
            Ok(())
        }
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            Err(1)
        }
    }
}

fn compile_c_binary(
    project_root: &Path,
    package_name: &str,
    c_source: &str,
) -> Result<(PathBuf, PathBuf), i32> {
    let build_dir = project_root.join("build");
    if let Err(err) = fs::create_dir_all(&build_dir) {
        eprintln!(
            "could not create build directory `{}`: {err}",
            build_dir.display()
        );
        return Err(2);
    }

    let c_path = build_dir.join("generated.c");
    if let Err(err) = fs::write(&c_path, c_source) {
        eprintln!("could not write generated C `{}`: {err}", c_path.display());
        return Err(2);
    }

    let runtime_header_path = build_dir.join("sec4_runtime.h");
    if let Err(err) = fs::write(&runtime_header_path, emit_runtime_header()) {
        eprintln!(
            "could not write runtime header `{}`: {err}",
            runtime_header_path.display()
        );
        return Err(2);
    }

    let runtime_source_path = build_dir.join("sec4_runtime.c");
    if let Err(err) = fs::write(&runtime_source_path, emit_runtime_source()) {
        eprintln!(
            "could not write runtime source `{}`: {err}",
            runtime_source_path.display()
        );
        return Err(2);
    }

    let binary_path = build_dir.join(package_name);
    let output = match Command::new("clang")
        .arg(&c_path)
        .arg(&runtime_source_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-I")
        .arg(&build_dir)
        .arg("-o")
        .arg(&binary_path)
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            eprintln!("could not execute clang: {err}");
            return Err(2);
        }
    };

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("clang failed while compiling `{}`", c_path.display());
        if !stdout.trim().is_empty() {
            eprintln!("{stdout}");
        }
        if !stderr.trim().is_empty() {
            eprintln!("{stderr}");
        }
        return Err(1);
    }

    Ok((c_path, binary_path))
}

fn cmd_check(path: &Path, emit: Option<EmitTarget>) -> Result<(), i32> {
    let diagnostics_json_mode = matches!(emit, Some(EmitTarget::DiagnosticsJson));
    match sec4_core::validate_project(path) {
        Ok(manifest) => match analyze_entry(path, &manifest) {
            Ok(program) => {
                if !diagnostics_json_mode {
                    println!(
                        "check succeeded (M3 effects): package={}, entry={}",
                        manifest.package.name,
                        manifest.entry_file()
                    );
                }
                match emit {
                    Some(EmitTarget::Ast) => println!("{}", program.to_pretty_json()),
                    Some(EmitTarget::DiagnosticsJson) => println!("[]"),
                    None => {}
                }
                Ok(())
            }
            Err(diagnostics) => {
                if diagnostics_json_mode {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&diagnostics)
                            .expect("diagnostics should serialize")
                    );
                } else {
                    print_diagnostics(&diagnostics);
                }
                Err(1)
            }
        },
        Err(diagnostics) => {
            if diagnostics_json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&diagnostics)
                        .expect("diagnostics should serialize")
                );
            } else {
                print_diagnostics(&diagnostics);
            }
            Err(1)
        }
    }
}

fn cmd_run(path: &Path) -> Result<(), i32> {
    let manifest = match sec4_core::validate_project(path) {
        Ok(manifest) => manifest,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            return Err(1);
        }
    };

    cmd_build(path, Some(BuildEmitTarget::CBin), false, false)?;

    let binary_path = path.join("build").join(&manifest.package.name);
    let status = match Command::new(&binary_path).status() {
        Ok(status) => status,
        Err(err) => {
            eprintln!(
                "could not execute binary `{}`: {err}",
                binary_path.display()
            );
            return Err(2);
        }
    };

    if status.success() {
        Ok(())
    } else {
        Err(status.code().unwrap_or(1))
    }
}

fn cmd_test(path: &Path) -> Result<(), i32> {
    cmd_check(path, None)?;
    println!("test command placeholder (M0): language-level tests land in M1-M3");
    Ok(())
}

fn cmd_fmt(path: &Path) -> Result<(), i32> {
    if !path.exists() {
        let diag = Diagnostic::error(
            "C0001",
            "path does not exist",
            sec4_core::Span::point(path.to_path_buf(), 1, 1),
        );
        print_diagnostics(&[diag]);
        return Err(1);
    }
    println!("fmt command placeholder (M0): canonical formatter lands in M8");
    Ok(())
}

fn cmd_lint(path: &Path) -> Result<(), i32> {
    cmd_check(path, None)?;
    println!("lint command placeholder (M0): policy/security lint pass lands in M7");
    Ok(())
}

fn print_diagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        eprintln!("{}", diagnostic.render_color());
    }
}
