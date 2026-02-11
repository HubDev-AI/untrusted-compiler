use ailang_core::{
    analyze_entry, build_security_map, render_security_audit_text, run_security_audit, should_fail,
    write_lockfile_stub, write_security_map, AuditSeverity, Diagnostic,
};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};

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
        fail_on: Option<String>,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum AuditOutputFormat {
    Text,
    Json,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum EmitTarget {
    Ast,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build { path } => cmd_build(&path),
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
            fail_on,
        } => cmd_sec_audit(&path, format, fail_on.as_deref()),
    }
}

fn cmd_sec_audit(path: &Path, format: AuditOutputFormat, fail_on: Option<&str>) -> Result<(), i32> {
    match ailang_core::validate_project(path) {
        Ok(manifest) => match analyze_entry(path, &manifest) {
            Ok(program) => {
                let policy = match ailang_core::policy::load_policy(path) {
                    Ok(policy) => policy,
                    Err(diagnostics) => {
                        print_diagnostics(&diagnostics);
                        return Err(1);
                    }
                };

                let security_map = build_security_map(&program, &policy);
                let security_map_path = match write_security_map(path, &security_map) {
                    Ok(output_path) => output_path,
                    Err(diagnostic) => {
                        print_diagnostics(&[diagnostic]);
                        return Err(1);
                    }
                };

                let report = run_security_audit(&policy, &security_map);
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

                println!("security map: {}", security_map_path.display());

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

fn cmd_build(path: &Path) -> Result<(), i32> {
    match ailang_core::validate_project(path) {
        Ok(manifest) => {
            if let Err(diagnostics) = analyze_entry(path, &manifest) {
                print_diagnostics(&diagnostics);
                return Err(1);
            }

            if let Err(diag) = write_lockfile_stub(path, &manifest) {
                print_diagnostics(&[diag]);
                return Err(1);
            }

            println!(
                "build succeeded (M3 effects): package={}, entry={}",
                manifest.package.name,
                manifest.entry_file()
            );
            println!(
                "wrote lockfile stub: {}",
                path.join("ailang.lock").display()
            );
            Ok(())
        }
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            Err(1)
        }
    }
}

fn cmd_check(path: &Path, emit: Option<EmitTarget>) -> Result<(), i32> {
    match ailang_core::validate_project(path) {
        Ok(manifest) => match analyze_entry(path, &manifest) {
            Ok(program) => {
                println!(
                    "check succeeded (M3 effects): package={}, entry={}",
                    manifest.package.name,
                    manifest.entry_file()
                );
                if let Some(EmitTarget::Ast) = emit {
                    println!("{}", program.to_pretty_json());
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

fn cmd_run(path: &Path) -> Result<(), i32> {
    cmd_check(path, None)?;
    println!("run not implemented yet (M0): this command will execute compiled output in M6+");
    Ok(())
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
