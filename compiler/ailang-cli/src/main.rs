use ailang_core::{
    analyze_entry, analyze_entry_with_allows, build_security_map_with_allows, emit_c_program,
    emit_runtime_header, emit_runtime_source, render_security_audit_text,
    run_security_audit_with_baseline, should_fail, write_lockfile_stub, write_security_map,
    AuditReport, AuditSeverity, Diagnostic,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(name = "ailang", version, about = "AILang compiler CLI (M1 parser)")]
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
    Sec {
        #[command(subcommand)]
        command: SecCommands,
    },
}

#[derive(Subcommand, Debug)]
enum SecCommands {
    Audit {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = AuditOutputFormat::Text)]
        format: AuditOutputFormat,
        #[arg(long)]
        baseline: Option<PathBuf>,
        #[arg(long)]
        history_dir: Option<PathBuf>,
        #[arg(long)]
        write_report: Option<PathBuf>,
        #[arg(long)]
        fail_on: Option<String>,
    },
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
        Commands::Build { path, emit } => cmd_build(&path, emit),
        Commands::Check { path, emit } => cmd_check(&path, emit),
        Commands::Run { path } => cmd_run(&path),
        Commands::Test { path } => cmd_test(&path),
        Commands::Fmt { path } => cmd_fmt(&path),
        Commands::Lint { path } => cmd_lint(&path),
        Commands::Sec { command } => cmd_sec(command),
    };

    if let Err(code) = result {
        std::process::exit(code);
    }
}

fn cmd_sec(command: SecCommands) -> Result<(), i32> {
    match command {
        SecCommands::Audit {
            path,
            format,
            baseline,
            history_dir,
            write_report,
            fail_on,
        } => cmd_sec_audit(
            &path,
            format,
            baseline.as_deref(),
            history_dir.as_deref(),
            write_report.as_deref(),
            fail_on.as_deref(),
        ),
    }
}

fn cmd_sec_audit(
    path: &Path,
    format: AuditOutputFormat,
    baseline_path: Option<&Path>,
    history_dir_path: Option<&Path>,
    write_report_path: Option<&Path>,
    fail_on: Option<&str>,
) -> Result<(), i32> {
    match ailang_core::validate_project(path) {
        Ok(manifest) => match analyze_entry_with_allows(path, &manifest) {
            Ok((program, allows)) => {
                let policy = match ailang_core::policy::load_policy(path) {
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
                let report = run_security_audit_with_baseline(
                    &policy,
                    &security_map,
                    baseline_report.as_ref(),
                );
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

fn cmd_build(path: &Path, emit: Option<BuildEmitTarget>) -> Result<(), i32> {
    let mir_json_mode = matches!(emit, Some(BuildEmitTarget::MirJson));
    match ailang_core::validate_project(path) {
        Ok(manifest) => {
            let program = match analyze_entry(path, &manifest) {
                Ok(program) => program,
                Err(diagnostics) => {
                    print_diagnostics(&diagnostics);
                    return Err(1);
                }
            };

            if let Err(diag) = write_lockfile_stub(path, &manifest) {
                print_diagnostics(&[diag]);
                return Err(1);
            }

            let mir = emit.map(|_| ailang_core::lower_program_to_mir(&program));
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
                println!(
                    "wrote lockfile stub: {}",
                    path.join("ailang.lock").display()
                );
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

    let runtime_header_path = build_dir.join("ailang_runtime.h");
    if let Err(err) = fs::write(&runtime_header_path, emit_runtime_header()) {
        eprintln!(
            "could not write runtime header `{}`: {err}",
            runtime_header_path.display()
        );
        return Err(2);
    }

    let runtime_source_path = build_dir.join("ailang_runtime.c");
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
    match ailang_core::validate_project(path) {
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
    let manifest = match ailang_core::validate_project(path) {
        Ok(manifest) => manifest,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            return Err(1);
        }
    };

    cmd_build(path, Some(BuildEmitTarget::CBin))?;

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
            ailang_core::Span::point(path.to_path_buf(), 1, 1),
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
