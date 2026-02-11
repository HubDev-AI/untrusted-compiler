use ailang_core::{write_lockfile_stub, Diagnostic};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "ailang", version, about = "AILang compiler CLI (M0 skeleton)")]
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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build { path } => cmd_build(&path),
        Commands::Check { path } => cmd_check(&path),
        Commands::Run { path } => cmd_run(&path),
        Commands::Test { path } => cmd_test(&path),
        Commands::Fmt { path } => cmd_fmt(&path),
        Commands::Lint { path } => cmd_lint(&path),
    };

    if let Err(code) = result {
        std::process::exit(code);
    }
}

fn cmd_build(path: &Path) -> Result<(), i32> {
    match ailang_core::validate_project(path) {
        Ok(manifest) => {
            if let Err(diag) = write_lockfile_stub(path, &manifest) {
                print_diagnostics(&[diag]);
                return Err(1);
            }

            println!(
                "build succeeded (M0 stub): package={}, entry={}",
                manifest.package.name,
                manifest.entry_file()
            );
            println!("wrote lockfile stub: {}", path.join("ailang.lock").display());
            Ok(())
        }
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            Err(1)
        }
    }
}

fn cmd_check(path: &Path) -> Result<(), i32> {
    match ailang_core::validate_project(path) {
        Ok(manifest) => {
            println!(
                "check succeeded (M0 stub): package={}, entry={}",
                manifest.package.name,
                manifest.entry_file()
            );
            Ok(())
        }
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            Err(1)
        }
    }
}

fn cmd_run(path: &Path) -> Result<(), i32> {
    cmd_check(path)?;
    println!("run not implemented yet (M0): this command will execute compiled output in M6+");
    Ok(())
}

fn cmd_test(path: &Path) -> Result<(), i32> {
    cmd_check(path)?;
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
    cmd_check(path)?;
    println!("lint command placeholder (M0): policy/security lint pass lands in M7");
    Ok(())
}

fn print_diagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        eprintln!("{}", diagnostic.render_color());
    }
}
