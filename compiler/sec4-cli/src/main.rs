use clap::{Args, Parser, Subcommand, ValueEnum};
use sec4_core::{
    analyze_entry, analyze_entry_with_allows, build_security_map_with_allows, emit_c_program,
    emit_runtime_header, emit_runtime_source, render_security_audit_text,
    run_security_audit_with_baseline, should_fail, summarize_history_window,
    validate_lockfile_stub, write_build_metadata, write_lockfile_stub, write_sbom,
    write_security_map, AuditHistoryWindowSummary, AuditReport, AuditSeverity, Diagnostic,
};
use std::collections::{HashMap, HashSet};
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
    Replay {
        #[arg(long)]
        capture: PathBuf,
        #[arg(long)]
        stubs: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = ReplayEffectsMode::Deny)]
        effects: ReplayEffectsMode,
        #[arg(long, value_enum, default_value_t = ReplayOutputFormat::Text)]
        format: ReplayOutputFormat,
        #[arg(long)]
        policy_hash: String,
        #[arg(long)]
        compiler_hash: String,
        #[arg(long)]
        runtime_hash: String,
        #[arg(long, default_value_t = false)]
        allow_policy_mismatch: bool,
    },
    Explain {
        code: String,
        #[arg(long, value_enum, default_value_t = ExplainOutputFormat::Text)]
        format: ExplainOutputFormat,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ExplainOutputFormat {
    Text,
    Json,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ReplayEffectsMode {
    Deny,
    Mock,
    Allow,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ReplayOutputFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockNetStubMatch {
    status: i64,
    truncated: bool,
    body_kind: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayDbStubDetails {
    entries: usize,
    unique_query_template_ids: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayFsStubDetails {
    entries: usize,
    read_ops: usize,
    write_ops: usize,
    other_ops: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockDependencyMatches {
    db: usize,
    fs: usize,
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
        Commands::Replay {
            capture,
            stubs,
            effects,
            format,
            policy_hash,
            compiler_hash,
            runtime_hash,
            allow_policy_mismatch,
        } => cmd_replay_check(
            &capture,
            &policy_hash,
            &compiler_hash,
            &runtime_hash,
            allow_policy_mismatch,
            effects,
            format,
            stubs.as_deref(),
        ),
        Commands::Explain { code, format } => cmd_explain(&code, format),
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

fn cmd_replay_check(
    capture_path: &Path,
    expected_policy_hash: &str,
    expected_compiler_hash: &str,
    expected_runtime_hash: &str,
    allow_policy_mismatch: bool,
    effects_mode: ReplayEffectsMode,
    output_format: ReplayOutputFormat,
    stubs_path: Option<&Path>,
) -> Result<(), i32> {
    let capture_bytes = match fs::read(capture_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!(
                "replay compatibility failed: cannot read capture {}: {err}",
                capture_path.display()
            );
            return Err(1);
        }
    };

    let capture_json: serde_json::Value = match serde_json::from_slice(&capture_bytes) {
        Ok(value) => value,
        Err(err) => {
            eprintln!(
                "replay compatibility failed: capture {} is not valid JSON: {err}",
                capture_path.display()
            );
            return Err(1);
        }
    };

    if let Err(message) = validate_replay_capture_contract(&capture_json) {
        eprintln!("replay compatibility failed: {message}");
        return Err(1);
    }

    let capture_policy_hash =
        json_required_str(&capture_json, "policyHash").expect("policyHash checked by contract");
    let capture_compiler_hash =
        json_required_str(&capture_json, "compilerHash").expect("compilerHash checked by contract");
    let capture_runtime_hash =
        json_required_str(&capture_json, "runtimeHash").expect("runtimeHash checked by contract");
    let mut warnings = Vec::new();
    let mut stub_counts: Option<(usize, usize, usize)> = None;
    let mut stub_details: Option<(ReplayDbStubDetails, ReplayFsStubDetails)> = None;
    let mut net_stubs: Option<HashMap<String, ReplayMockNetStubMatch>> = None;
    let mut db_stub_signatures: Option<HashSet<String>> = None;
    let mut fs_stub_signatures: Option<HashSet<String>> = None;
    let mut mock_request_signature: Option<String> = None;
    let mut mock_matched_stub: Option<ReplayMockNetStubMatch> = None;
    let mut mock_dependency_matches: Option<ReplayMockDependencyMatches> = None;

    if capture_compiler_hash != expected_compiler_hash {
        eprintln!(
            "replay compatibility failed: compilerHash mismatch ({capture_compiler_hash} != {expected_compiler_hash})"
        );
        return Err(1);
    }

    if capture_runtime_hash != expected_runtime_hash {
        eprintln!(
            "replay compatibility failed: runtimeHash mismatch ({capture_runtime_hash} != {expected_runtime_hash})"
        );
        return Err(1);
    }

    if capture_policy_hash != expected_policy_hash {
        if allow_policy_mismatch {
            let warning = format!(
                "policyHash mismatch allowed ({capture_policy_hash} != {expected_policy_hash})"
            );
            warnings.push(warning.clone());
            eprintln!("warning: {warning}");
        } else {
            eprintln!(
                "replay compatibility failed: policyHash mismatch ({capture_policy_hash} != {expected_policy_hash})"
            );
            return Err(1);
        }
    }

    if effects_mode == ReplayEffectsMode::Mock && stubs_path.is_none() {
        eprintln!("replay compatibility failed: mock effects mode requires --stubs <path>");
        return Err(1);
    }

    if effects_mode == ReplayEffectsMode::Allow {
        let warning =
            "replay effects mode is allow; use deny/mock outside isolated environments".to_string();
        warnings.push(warning.clone());
        eprintln!("warning: {warning}");
    }

    if let Some(stubs_path) = stubs_path {
        let stubs_bytes = match fs::read(stubs_path) {
            Ok(bytes) => bytes,
            Err(err) => {
                eprintln!(
                    "replay compatibility failed: cannot read stub registry {}: {err}",
                    stubs_path.display()
                );
                return Err(1);
            }
        };

        let stubs_json: serde_json::Value = match serde_json::from_slice(&stubs_bytes) {
            Ok(value) => value,
            Err(err) => {
                eprintln!(
                    "replay compatibility failed: stub registry {} is not valid JSON: {err}",
                    stubs_path.display()
                );
                return Err(1);
            }
        };

        if let Err(message) = validate_replay_stub_registry_contract(&stubs_json) {
            eprintln!("replay compatibility failed: stub registry contract invalid: {message}");
            return Err(1);
        }

        let stubs_obj = stubs_json
            .get("stubs")
            .and_then(serde_json::Value::as_object)
            .expect("stub registry stubs object shape already validated");
        let net_count = stubs_obj
            .get("net")
            .and_then(serde_json::Value::as_array)
            .expect("stub registry stubs.net shape already validated")
            .len();
        let db_count = stubs_obj
            .get("db")
            .and_then(serde_json::Value::as_array)
            .map_or(0, Vec::len);
        let fs_count = stubs_obj
            .get("fs")
            .and_then(serde_json::Value::as_array)
            .map_or(0, Vec::len);
        stub_counts = Some((net_count, db_count, fs_count));
        stub_details = match collect_replay_db_fs_stub_details(&stubs_json) {
            Ok(details) => Some(details),
            Err(message) => {
                eprintln!(
                    "replay compatibility failed: stub registry contract invalid: {message}"
                );
                return Err(1);
            }
        };
        net_stubs = match collect_replay_net_stubs(&stubs_json) {
            Ok(entries) => Some(entries),
            Err(message) => {
                eprintln!(
                    "replay compatibility failed: stub registry contract invalid: {message}"
                );
                return Err(1);
            }
        };
        db_stub_signatures = match collect_replay_db_stub_signatures(&stubs_json) {
            Ok(entries) => Some(entries),
            Err(message) => {
                eprintln!(
                    "replay compatibility failed: stub registry contract invalid: {message}"
                );
                return Err(1);
            }
        };
        fs_stub_signatures = match collect_replay_fs_stub_signatures(&stubs_json) {
            Ok(entries) => Some(entries),
            Err(message) => {
                eprintln!(
                    "replay compatibility failed: stub registry contract invalid: {message}"
                );
                return Err(1);
            }
        };
    }

    if effects_mode == ReplayEffectsMode::Mock {
        let signature = match capture_net_request_signature(&capture_json) {
            Ok(signature) => signature,
            Err(message) => {
                eprintln!("replay compatibility failed: {message}");
                return Err(1);
            }
        };
        let stubs = net_stubs
            .as_ref()
            .expect("mock mode requires stubs and prevalidated signature extraction");
        let stub_match = if let Some(entry) = stubs.get(&signature) {
            entry.clone()
        } else {
            eprintln!("replay compatibility failed: REPLAY.STUB_MISSING: {signature}");
            return Err(1);
        };
        let (capture_db_signatures, capture_fs_signatures) =
            match collect_capture_db_fs_dependency_signatures(&capture_json) {
                Ok(signatures) => signatures,
                Err(message) => {
                    eprintln!("replay compatibility failed: {message}");
                    return Err(1);
                }
            };
        let db_signatures = db_stub_signatures
            .as_ref()
            .expect("mock mode requires db signatures for loaded stubs");
        for signature in &capture_db_signatures {
            if !db_signatures.contains(signature) {
                eprintln!("replay compatibility failed: REPLAY.DB_STUB_MISSING: {signature}");
                return Err(1);
            }
        }
        let fs_signatures = fs_stub_signatures
            .as_ref()
            .expect("mock mode requires fs signatures for loaded stubs");
        for signature in &capture_fs_signatures {
            if !fs_signatures.contains(signature) {
                eprintln!("replay compatibility failed: REPLAY.FS_STUB_MISSING: {signature}");
                return Err(1);
            }
        }
        mock_request_signature = Some(signature);
        mock_matched_stub = Some(stub_match);
        mock_dependency_matches = Some(ReplayMockDependencyMatches {
            db: capture_db_signatures.len(),
            fs: capture_fs_signatures.len(),
        });
    }

    match output_format {
        ReplayOutputFormat::Text => {
            println!("replay capture compatibility check passed");
            if let Some((net, db, fs)) = stub_counts {
                println!("replay stubs loaded: net={net} db={db} fs={fs}");
            }
            if let Some(signature) = &mock_request_signature {
                println!("replay mock stub matched: {signature}");
            }
            if let Some(stub) = &mock_matched_stub {
                println!(
                    "replay mock stub response: status={} truncated={} bodyKind={}",
                    stub.status, stub.truncated, stub.body_kind
                );
            }
            if let Some((db, fs)) = &stub_details {
                println!(
                    "replay stub details: dbEntries={} dbTemplates={} fsEntries={} fsOps(read={},write={},other={})",
                    db.entries,
                    db.unique_query_template_ids,
                    fs.entries,
                    fs.read_ops,
                    fs.write_ops,
                    fs.other_ops
                );
            }
            if let Some(matches) = &mock_dependency_matches {
                println!(
                    "replay mock dependency matches: db={} fs={}",
                    matches.db, matches.fs
                );
            }
        }
        ReplayOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "capture": capture_path.to_string_lossy(),
                "stubs": stubs_path.map(|path| path.to_string_lossy().to_string()),
                "effectsMode": match effects_mode {
                    ReplayEffectsMode::Deny => "deny",
                    ReplayEffectsMode::Mock => "mock",
                    ReplayEffectsMode::Allow => "allow",
                },
                "policyHashMatched": capture_policy_hash == expected_policy_hash,
                "compilerHashMatched": capture_compiler_hash == expected_compiler_hash,
                "runtimeHashMatched": capture_runtime_hash == expected_runtime_hash,
                "allowPolicyMismatch": allow_policy_mismatch,
                "warnings": warnings,
                "stubCounts": stub_counts.map(|(net, db, fs)| serde_json::json!({
                    "net": net,
                    "db": db,
                    "fs": fs,
                })),
                "mockRequestSignature": mock_request_signature,
                "mockMatchedStub": mock_matched_stub.as_ref().map(|stub| serde_json::json!({
                    "status": stub.status,
                    "truncated": stub.truncated,
                    "bodyKind": stub.body_kind,
                })),
                "mockDependencyMatches": mock_dependency_matches.as_ref().map(|matches| serde_json::json!({
                    "db": matches.db,
                    "fs": matches.fs,
                })),
                "stubDetails": stub_details.as_ref().map(|(db, fs)| serde_json::json!({
                    "db": {
                        "entries": db.entries,
                        "uniqueQueryTemplateIds": db.unique_query_template_ids,
                    },
                    "fs": {
                        "entries": fs.entries,
                        "readOps": fs.read_ops,
                        "writeOps": fs.write_ops,
                        "otherOps": fs.other_ops,
                    },
                })),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload)
                    .expect("replay payload should serialize as JSON")
            );
        }
    }
    Ok(())
}

fn json_required_str<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .filter(|entry| !entry.is_empty())
}

fn validate_replay_capture_contract(capture: &serde_json::Value) -> Result<(), String> {
    if capture.get("version").and_then(serde_json::Value::as_str) != Some("0.1") {
        return Err("capture.version must be \"0.1\"".to_string());
    }

    for key in [
        "captureId",
        "traceId",
        "policyHash",
        "compilerHash",
        "runtimeHash",
    ] {
        if json_required_str(capture, key).is_none() {
            return Err(format!("capture.{key} must be a non-empty string"));
        }
    }

    if capture
        .get("timeMs")
        .and_then(serde_json::Value::as_i64)
        .is_none()
    {
        return Err("capture.timeMs must be an integer timestamp".to_string());
    }

    let request = capture
        .get("request")
        .ok_or_else(|| "capture.request must be an object".to_string())?;
    if request
        .get("method")
        .and_then(serde_json::Value::as_str)
        .is_none()
    {
        return Err("capture.request.method must be a string".to_string());
    }
    if request
        .get("path")
        .and_then(serde_json::Value::as_str)
        .is_none()
    {
        return Err("capture.request.path must be a string".to_string());
    }
    if request
        .get("headers")
        .and_then(serde_json::Value::as_object)
        .is_none()
    {
        return Err("capture.request.headers must be an object".to_string());
    }

    let body = request
        .get("body")
        .ok_or_else(|| "capture.request.body must be an object".to_string())?;
    let encoding = body
        .get("encoding")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "capture.request.body.encoding must be a string".to_string())?;
    if body
        .get("truncated")
        .and_then(serde_json::Value::as_bool)
        .is_none()
    {
        return Err("capture.request.body.truncated must be a boolean".to_string());
    }
    match encoding {
        "base64" => {
            if body
                .get("bytes")
                .and_then(serde_json::Value::as_str)
                .is_none()
            {
                return Err(
                    "capture.request.body.bytes must be present for encoding=base64".to_string(),
                );
            }
        }
        "none" => {
            if body
                .get("sha256")
                .and_then(serde_json::Value::as_str)
                .is_none()
            {
                return Err(
                    "capture.request.body.sha256 must be present for encoding=none".to_string(),
                );
            }
        }
        _ => {
            return Err(format!(
                "capture.request.body.encoding must be base64|none (got {encoding})"
            ))
        }
    }

    let determinism = capture
        .get("determinism")
        .ok_or_else(|| "capture.determinism must be an object".to_string())?;
    if determinism
        .get("seed")
        .and_then(serde_json::Value::as_i64)
        .is_none()
    {
        return Err("capture.determinism.seed must be an integer".to_string());
    }
    if determinism
        .get("time")
        .and_then(serde_json::Value::as_object)
        .and_then(|time| time.get("mode"))
        .and_then(serde_json::Value::as_str)
        .is_none()
    {
        return Err("capture.determinism.time.mode must be a string".to_string());
    }
    if determinism
        .get("time")
        .and_then(serde_json::Value::as_object)
        .and_then(|time| time.get("nowMs"))
        .and_then(serde_json::Value::as_i64)
        .is_none()
    {
        return Err("capture.determinism.time.nowMs must be an integer".to_string());
    }
    if determinism
        .get("uuid")
        .and_then(serde_json::Value::as_object)
        .and_then(|uuid| uuid.get("mode"))
        .and_then(serde_json::Value::as_str)
        .is_none()
    {
        return Err("capture.determinism.uuid.mode must be a string".to_string());
    }

    let budget = determinism
        .get("budget")
        .ok_or_else(|| "capture.determinism.budget must be an object".to_string())?;
    for key in ["maxBodyBytes", "maxJsonBytes", "maxJsonDepth", "deadlineMs"] {
        if budget
            .get(key)
            .and_then(serde_json::Value::as_i64)
            .is_none()
        {
            return Err(format!(
                "capture.determinism.budget.{key} must be an integer"
            ));
        }
    }

    let redaction = capture
        .get("redaction")
        .ok_or_else(|| "capture.redaction must be an object".to_string())?;
    if redaction
        .get("headers")
        .and_then(serde_json::Value::as_array)
        .is_none()
    {
        return Err("capture.redaction.headers must be an array".to_string());
    }
    if redaction
        .get("jsonPaths")
        .and_then(serde_json::Value::as_array)
        .is_none()
    {
        return Err("capture.redaction.jsonPaths must be an array".to_string());
    }

    if let Some(dependencies) = capture.get("dependencies") {
        let dependencies_obj = dependencies
            .as_object()
            .ok_or_else(|| "capture.dependencies must be an object when present".to_string())?;

        if let Some(db_entries) = dependencies_obj.get("db") {
            let db_entries = db_entries
                .as_array()
                .ok_or_else(|| "capture.dependencies.db must be an array when present".to_string())?;
            let mut db_signatures = HashSet::new();
            for (index, entry) in db_entries.iter().enumerate() {
                let request = entry
                    .get("request")
                    .and_then(serde_json::Value::as_object)
                    .ok_or_else(|| {
                        format!("capture.dependencies.db[{index}].request must be an object")
                    })?;
                request
                    .get("queryTemplateId")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        format!("capture.dependencies.db[{index}].request.queryTemplateId must be a non-empty string")
                    })?;
                if request.get("paramsSha256").is_some() {
                    request
                        .get("paramsSha256")
                        .and_then(serde_json::Value::as_str)
                        .filter(|value| !value.is_empty())
                        .ok_or_else(|| {
                            format!(
                                "capture.dependencies.db[{index}].request.paramsSha256 must be a non-empty string when present"
                            )
                        })?;
                }
                let signature = replay_db_request_signature(
                    request
                        .get("queryTemplateId")
                        .and_then(serde_json::Value::as_str)
                        .expect("queryTemplateId validated as non-empty string"),
                    request
                        .get("paramsSha256")
                        .and_then(serde_json::Value::as_str)
                        .filter(|value| !value.is_empty()),
                );
                if !db_signatures.insert(signature.clone()) {
                    return Err(format!(
                        "REPLAY.CAPTURE_DB_DEPENDENCY_DUPLICATE: {signature}"
                    ));
                }
            }
        }

        if let Some(fs_entries) = dependencies_obj.get("fs") {
            let fs_entries = fs_entries
                .as_array()
                .ok_or_else(|| "capture.dependencies.fs must be an array when present".to_string())?;
            let mut fs_signatures = HashSet::new();
            for (index, entry) in fs_entries.iter().enumerate() {
                let request = entry
                    .get("request")
                    .and_then(serde_json::Value::as_object)
                    .ok_or_else(|| {
                        format!("capture.dependencies.fs[{index}].request must be an object")
                    })?;
                request
                    .get("op")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        format!("capture.dependencies.fs[{index}].request.op must be a non-empty string")
                    })?;
                request
                    .get("pathSha256")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        format!("capture.dependencies.fs[{index}].request.pathSha256 must be a non-empty string")
                    })?;
                let signature = replay_fs_request_signature(
                    request
                        .get("op")
                        .and_then(serde_json::Value::as_str)
                        .expect("op validated as non-empty string"),
                    request
                        .get("pathSha256")
                        .and_then(serde_json::Value::as_str)
                        .expect("pathSha256 validated as non-empty string"),
                );
                if !fs_signatures.insert(signature.clone()) {
                    return Err(format!(
                        "REPLAY.CAPTURE_FS_DEPENDENCY_DUPLICATE: {signature}"
                    ));
                }
            }
        }
    }

    Ok(())
}

fn validate_replay_stub_registry_contract(stubs: &serde_json::Value) -> Result<(), String> {
    if stubs.get("version").and_then(serde_json::Value::as_str) != Some("0.1") {
        return Err("stub registry version must be \"0.1\"".to_string());
    }

    let stubs_obj = stubs
        .get("stubs")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "stub registry stubs must be an object".to_string())?;

    let net_entries = stubs_obj
        .get("net")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "stub registry stubs.net must be an array".to_string())?;

    let db_entries = stubs_obj
        .get("db")
        .map(|db| {
            db.as_array()
                .ok_or_else(|| "stub registry stubs.db must be an array when present".to_string())
        })
        .transpose()?
        .map_or(&[][..], Vec::as_slice);

    let fs_entries = stubs_obj
        .get("fs")
        .map(|fs| {
            fs.as_array()
                .ok_or_else(|| "stub registry stubs.fs must be an array when present".to_string())
        })
        .transpose()?
        .map_or(&[][..], Vec::as_slice);

    let redaction = stubs
        .get("redaction")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "stub registry redaction must be an object".to_string())?;
    if redaction
        .get("headers")
        .and_then(serde_json::Value::as_array)
        .is_none()
    {
        return Err("stub registry redaction.headers must be an array".to_string());
    }
    if redaction
        .get("jsonPaths")
        .and_then(serde_json::Value::as_array)
        .is_none()
    {
        return Err("stub registry redaction.jsonPaths must be an array".to_string());
    }
    let redaction_headers = redaction
        .get("headers")
        .and_then(serde_json::Value::as_array)
        .expect("headers array shape already validated");
    let mut normalized_headers = HashSet::new();
    for (index, header) in redaction_headers.iter().enumerate() {
        let normalized = header
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!("stub registry redaction.headers[{index}] must be a non-empty string value")
            })?
            .to_ascii_lowercase();
        normalized_headers.insert(normalized);
    }
    for required in ["authorization", "cookie", "set-cookie"] {
        if !normalized_headers.contains(required) {
            return Err(format!(
                "stub registry redaction.headers must include required header '{required}'"
            ));
        }
    }
    let redaction_paths = redaction
        .get("jsonPaths")
        .and_then(serde_json::Value::as_array)
        .expect("jsonPaths array shape already validated");
    let mut normalized_paths = HashSet::new();
    for (index, path) in redaction_paths.iter().enumerate() {
        let normalized = path
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "stub registry redaction.jsonPaths[{index}] must be a non-empty string value"
                )
            })?
            .to_ascii_lowercase();
        normalized_paths.insert(normalized);
    }
    for required in ["$.password", "$.token", "$.secret", "$.apikey"] {
        if !normalized_paths.contains(required) {
            return Err(format!(
                "stub registry redaction.jsonPaths must include required path '{required}'"
            ));
        }
    }

    let mut signatures = HashSet::new();
    for (index, entry) in net_entries.iter().enumerate() {
        let request = entry
            .get("request")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("stub registry stubs.net[{index}].request must be an object"))?;
        let method = request
            .get("method")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "stub registry stubs.net[{index}].request.method must be a non-empty string"
                )
            })?;
        let url = request
            .get("url")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!("stub registry stubs.net[{index}].request.url must be a non-empty string")
            })?;
        let body_sha = if request.get("bodySha256").is_some() {
            Some(
                request
                    .get("bodySha256")
                    .and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        format!(
                            "stub registry stubs.net[{index}].request.bodySha256 must be a non-empty string when present"
                        )
                    })?,
            )
        } else {
            None
        };

        let response = entry
            .get("response")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| {
                format!("stub registry stubs.net[{index}].response must be an object")
            })?;
        let status = response
            .get("status")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                format!("stub registry stubs.net[{index}].response.status must be an integer")
            })?;
        if !(100..=599).contains(&status) {
            return Err(format!(
                "stub registry stubs.net[{index}].response.status must be between 100 and 599"
            ));
        }
        if response
            .get("truncated")
            .and_then(serde_json::Value::as_bool)
            .is_none()
        {
            return Err(format!(
                "stub registry stubs.net[{index}].response.truncated must be a boolean"
            ));
        }

        let has_body_base64 = response
            .get("bodyBase64")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.is_empty());
        let has_body_sha = response
            .get("bodySha256")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.is_empty());
        if !has_body_base64 && !has_body_sha {
            return Err(format!(
                "stub registry stubs.net[{index}].response must include bodyBase64 or bodySha256"
            ));
        }

        let signature = replay_net_request_signature(method, url, body_sha);
        if !signatures.insert(signature) {
            return Err(format!(
                "stub registry stubs.net has duplicate request signature at index {index}"
            ));
        }
    }

    let mut db_signatures = HashSet::new();
    for (index, entry) in db_entries.iter().enumerate() {
        let request = entry
            .get("request")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("stub registry stubs.db[{index}].request must be an object"))?;
        let query_template_id = request
            .get("queryTemplateId")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "stub registry stubs.db[{index}].request.queryTemplateId must be a non-empty string"
                )
            })?;
        let params_sha = if request.get("paramsSha256").is_some() {
            Some(
                request
                .get("paramsSha256")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    format!(
                        "stub registry stubs.db[{index}].request.paramsSha256 must be a non-empty string when present"
                    )
                })?,
            )
        } else {
            None
        };

        let response = entry
            .get("response")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| {
                format!("stub registry stubs.db[{index}].response must be an object")
            })?;
        let row_count = response
            .get("rowCount")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                format!("stub registry stubs.db[{index}].response.rowCount must be an integer")
            })?;
        if row_count < 0 {
            return Err(format!(
                "stub registry stubs.db[{index}].response.rowCount must be >= 0"
            ));
        }
        if response
            .get("truncated")
            .and_then(serde_json::Value::as_bool)
            .is_none()
        {
            return Err(format!(
                "stub registry stubs.db[{index}].response.truncated must be a boolean"
            ));
        }

        let signature = replay_db_request_signature(query_template_id, params_sha);
        if !db_signatures.insert(signature) {
            return Err(format!(
                "stub registry stubs.db has duplicate request signature at index {index}"
            ));
        }
    }

    let mut fs_signatures = HashSet::new();
    for (index, entry) in fs_entries.iter().enumerate() {
        let request = entry
            .get("request")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("stub registry stubs.fs[{index}].request must be an object"))?;
        let op = request
            .get("op")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!("stub registry stubs.fs[{index}].request.op must be a non-empty string")
            })?;
        let path_sha = request
            .get("pathSha256")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "stub registry stubs.fs[{index}].request.pathSha256 must be a non-empty string"
                )
            })?;

        let response = entry
            .get("response")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| {
                format!("stub registry stubs.fs[{index}].response must be an object")
            })?;
        response
            .get("ok")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| {
                format!("stub registry stubs.fs[{index}].response.ok must be a boolean")
            })?;
        if response
            .get("truncated")
            .and_then(serde_json::Value::as_bool)
            .is_none()
        {
            return Err(format!(
                "stub registry stubs.fs[{index}].response.truncated must be a boolean"
            ));
        }
        if let Some(bytes) = response.get("bytes") {
            let size = bytes.as_i64().ok_or_else(|| {
                format!("stub registry stubs.fs[{index}].response.bytes must be an integer")
            })?;
            if size < 0 {
                return Err(format!(
                    "stub registry stubs.fs[{index}].response.bytes must be >= 0"
                ));
            }
        }

        let signature = replay_fs_request_signature(op, path_sha);
        if !fs_signatures.insert(signature) {
            return Err(format!(
                "stub registry stubs.fs has duplicate request signature at index {index}"
            ));
        }
    }

    Ok(())
}

fn replay_net_request_signature(method: &str, url: &str, body_sha256: Option<&str>) -> String {
    format!(
        "{}|{}|{}",
        method.to_ascii_uppercase(),
        url,
        body_sha256.unwrap_or("-")
    )
}

fn replay_db_request_signature(query_template_id: &str, params_sha256: Option<&str>) -> String {
    format!("{}|{}", query_template_id, params_sha256.unwrap_or("-"))
}

fn replay_fs_request_signature(op: &str, path_sha256: &str) -> String {
    format!("{}|{}", op.to_ascii_lowercase(), path_sha256)
}

fn capture_net_request_signature(capture: &serde_json::Value) -> Result<String, String> {
    let request = capture
        .get("request")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "capture.request must be an object".to_string())?;
    let method = request
        .get("method")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "capture.request.method must be a non-empty string".to_string())?;
    let path = request
        .get("path")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "capture.request.path must be a non-empty string".to_string())?;
    let url = if let Some(value) = request
        .get("url")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
    {
        value.to_string()
    } else {
        let scheme = request
            .get("scheme")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                "capture.request.url must be present or capture.request.scheme/host must be non-empty strings".to_string()
            })?;
        let host = request
            .get("host")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                "capture.request.url must be present or capture.request.scheme/host must be non-empty strings".to_string()
            })?;
        let normalized_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        let mut built = format!("{scheme}://{host}{normalized_path}");
        if let Some(query) = request
            .get("query")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
        {
            if query.starts_with('?') {
                built.push_str(query);
            } else {
                built.push('?');
                built.push_str(query);
            }
        }
        built
    };
    let body_sha256 = request
        .get("body")
        .and_then(serde_json::Value::as_object)
        .and_then(|body| body.get("sha256"))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty());
    Ok(replay_net_request_signature(method, &url, body_sha256))
}

fn collect_capture_db_fs_dependency_signatures(
    capture: &serde_json::Value,
) -> Result<(Vec<String>, Vec<String>), String> {
    let dependencies = if let Some(value) = capture.get("dependencies") {
        value
            .as_object()
            .ok_or_else(|| "capture.dependencies must be an object when present".to_string())?
    } else {
        return Ok((Vec::new(), Vec::new()));
    };

    let mut db_signatures = Vec::new();
    let mut seen_db_signatures = HashSet::new();
    if let Some(db_entries) = dependencies.get("db") {
        let db_entries = db_entries
            .as_array()
            .ok_or_else(|| "capture.dependencies.db must be an array when present".to_string())?;
        for (index, entry) in db_entries.iter().enumerate() {
            let request = entry
                .get("request")
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| {
                    format!("capture.dependencies.db[{index}].request must be an object")
                })?;
            let query_template_id = request
                .get("queryTemplateId")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    format!("capture.dependencies.db[{index}].request.queryTemplateId must be a non-empty string")
                })?;
            let params_sha256 = request
                .get("paramsSha256")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.is_empty());
            let signature = replay_db_request_signature(query_template_id, params_sha256);
            if !seen_db_signatures.insert(signature.clone()) {
                return Err(format!(
                    "REPLAY.CAPTURE_DB_DEPENDENCY_DUPLICATE: {signature}"
                ));
            }
            db_signatures.push(signature);
        }
    }

    let mut fs_signatures = Vec::new();
    let mut seen_fs_signatures = HashSet::new();
    if let Some(fs_entries) = dependencies.get("fs") {
        let fs_entries = fs_entries
            .as_array()
            .ok_or_else(|| "capture.dependencies.fs must be an array when present".to_string())?;
        for (index, entry) in fs_entries.iter().enumerate() {
            let request = entry
                .get("request")
                .and_then(serde_json::Value::as_object)
                .ok_or_else(|| {
                    format!("capture.dependencies.fs[{index}].request must be an object")
                })?;
            let op = request
                .get("op")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    format!("capture.dependencies.fs[{index}].request.op must be a non-empty string")
                })?;
            let path_sha256 = request
                .get("pathSha256")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    format!("capture.dependencies.fs[{index}].request.pathSha256 must be a non-empty string")
                })?;
            let signature = replay_fs_request_signature(op, path_sha256);
            if !seen_fs_signatures.insert(signature.clone()) {
                return Err(format!(
                    "REPLAY.CAPTURE_FS_DEPENDENCY_DUPLICATE: {signature}"
                ));
            }
            fs_signatures.push(signature);
        }
    }

    Ok((db_signatures, fs_signatures))
}

fn collect_replay_net_stubs(
    stubs: &serde_json::Value,
) -> Result<HashMap<String, ReplayMockNetStubMatch>, String> {
    let net_entries = stubs
        .get("stubs")
        .and_then(serde_json::Value::as_object)
        .and_then(|entries| entries.get("net"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "stub registry stubs.net must be an array".to_string())?;
    let mut signatures = HashMap::new();
    for (index, entry) in net_entries.iter().enumerate() {
        let request = entry
            .get("request")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("stub registry stubs.net[{index}].request must be an object"))?;
        let method = request
            .get("method")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "stub registry stubs.net[{index}].request.method must be a non-empty string"
                )
            })?;
        let url = request
            .get("url")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!("stub registry stubs.net[{index}].request.url must be a non-empty string")
            })?;
        let body_sha256 = request
            .get("bodySha256")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty());
        let response = entry
            .get("response")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("stub registry stubs.net[{index}].response must be an object"))?;
        let status = response
            .get("status")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                format!("stub registry stubs.net[{index}].response.status must be an integer")
            })?;
        let truncated = response
            .get("truncated")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| {
                format!("stub registry stubs.net[{index}].response.truncated must be a boolean")
            })?;
        let has_body_base64 = response
            .get("bodyBase64")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.is_empty());
        let has_body_sha = response
            .get("bodySha256")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.is_empty());
        if !has_body_base64 && !has_body_sha {
            return Err(format!(
                "stub registry stubs.net[{index}].response must include bodyBase64 or bodySha256"
            ));
        }
        let body_kind = if has_body_base64 { "base64" } else { "sha256" };
        let signature = replay_net_request_signature(method, url, body_sha256);
        if signatures
            .insert(
                signature,
                ReplayMockNetStubMatch {
                    status,
                    truncated,
                    body_kind,
                },
            )
            .is_some()
        {
            return Err(format!(
                "stub registry stubs.net has duplicate request signature at index {index}"
            ));
        }
    }
    Ok(signatures)
}

fn collect_replay_db_fs_stub_details(
    stubs: &serde_json::Value,
) -> Result<(ReplayDbStubDetails, ReplayFsStubDetails), String> {
    let stubs_obj = stubs
        .get("stubs")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "stub registry stubs must be an object".to_string())?;
    let db_entries = stubs_obj
        .get("db")
        .and_then(serde_json::Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let fs_entries = stubs_obj
        .get("fs")
        .and_then(serde_json::Value::as_array)
        .map_or(&[][..], Vec::as_slice);

    let mut unique_templates = HashSet::new();
    for (index, entry) in db_entries.iter().enumerate() {
        let query_template_id = entry
            .get("request")
            .and_then(serde_json::Value::as_object)
            .and_then(|request| request.get("queryTemplateId"))
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "stub registry stubs.db[{index}].request.queryTemplateId must be a non-empty string"
                )
            })?;
        unique_templates.insert(query_template_id.to_string());
    }

    let mut read_ops = 0usize;
    let mut write_ops = 0usize;
    let mut other_ops = 0usize;
    for (index, entry) in fs_entries.iter().enumerate() {
        let op = entry
            .get("request")
            .and_then(serde_json::Value::as_object)
            .and_then(|request| request.get("op"))
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!("stub registry stubs.fs[{index}].request.op must be a non-empty string")
            })?;
        match op.to_ascii_lowercase().as_str() {
            "read" => read_ops += 1,
            "write" => write_ops += 1,
            _ => other_ops += 1,
        }
    }

    Ok((
        ReplayDbStubDetails {
            entries: db_entries.len(),
            unique_query_template_ids: unique_templates.len(),
        },
        ReplayFsStubDetails {
            entries: fs_entries.len(),
            read_ops,
            write_ops,
            other_ops,
        },
    ))
}

fn collect_replay_db_stub_signatures(stubs: &serde_json::Value) -> Result<HashSet<String>, String> {
    let entries = stubs
        .get("stubs")
        .and_then(serde_json::Value::as_object)
        .and_then(|stubs_obj| stubs_obj.get("db"))
        .and_then(serde_json::Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let mut signatures = HashSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let request = entry
            .get("request")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("stub registry stubs.db[{index}].request must be an object"))?;
        let query_template_id = request
            .get("queryTemplateId")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "stub registry stubs.db[{index}].request.queryTemplateId must be a non-empty string"
                )
            })?;
        let params_sha256 = request
            .get("paramsSha256")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty());
        signatures.insert(replay_db_request_signature(query_template_id, params_sha256));
    }
    Ok(signatures)
}

fn collect_replay_fs_stub_signatures(stubs: &serde_json::Value) -> Result<HashSet<String>, String> {
    let entries = stubs
        .get("stubs")
        .and_then(serde_json::Value::as_object)
        .and_then(|stubs_obj| stubs_obj.get("fs"))
        .and_then(serde_json::Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let mut signatures = HashSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let request = entry
            .get("request")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("stub registry stubs.fs[{index}].request must be an object"))?;
        let op = request
            .get("op")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!("stub registry stubs.fs[{index}].request.op must be a non-empty string")
            })?;
        let path_sha256 = request
            .get("pathSha256")
            .and_then(serde_json::Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "stub registry stubs.fs[{index}].request.pathSha256 must be a non-empty string"
                )
            })?;
        signatures.insert(replay_fs_request_signature(op, path_sha256));
    }
    Ok(signatures)
}

fn cmd_explain(code: &str, format: ExplainOutputFormat) -> Result<(), i32> {
    let code = code.trim();
    if code.is_empty() {
        eprintln!("explain requires a diagnostic code");
        return Err(2);
    }

    let normalized = code.to_ascii_uppercase();
    let (topic, summary, fixes, docs_path) = explain_topic(&normalized);
    let related_commands = vec![
        "sec4 check --emit diagnostics-json".to_string(),
        "sec4 audit --format text".to_string(),
        "sec4 gate --fail-on 'risk>=HIGH'".to_string(),
    ];

    match format {
        ExplainOutputFormat::Text => {
            println!("{normalized} - {topic}");
            println!("{summary}");
            println!();
            println!("Likely actions:");
            for fix in fixes {
                println!("- {fix}");
            }
            println!();
            println!("Related commands:");
            for command in &related_commands {
                println!("- {command}");
            }
            println!("Docs: {docs_path}");
        }
        ExplainOutputFormat::Json => {
            let payload = serde_json::json!({
                "code": normalized,
                "topic": topic,
                "summary": summary,
                "likelyActions": fixes,
                "relatedCommands": related_commands,
                "docsPath": docs_path,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload)
                    .expect("explain payload should serialize as JSON")
            );
        }
    }

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
        "CORS_CREDENTIALS_WITH_WILDCARD" => {
            return (
                "CORS Credentials With Wildcard Origin",
                "Credentialed CORS cannot be paired with wildcard origins; switch to explicit origin allowlists.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "CORS_ANY_ORIGIN" => {
            return (
                "CORS Any-Origin Exposure",
                "Wildcard origins increase exposure; use explicit allowlists when possible.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "CORS_REFLECT_ORIGIN_ENABLED" => {
            return (
                "CORS Reflect-Origin Risk",
                "Origin reflection should be disabled unless tightly constrained by policy and matching rules.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "CORS_VARY_ORIGIN_MISSING" => {
            return (
                "CORS Missing Vary-Origin",
                "Allowlist-based CORS responses should emit Vary: Origin for correct cache and policy behavior.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "CSP_DISABLED" => {
            return (
                "CSP Disabled",
                "Content-Security-Policy is disabled; enable CSP (report-only first if needed) for stronger browser-side defenses.",
                &POLICY_FIXES,
                "docs/book/63-security-middleware-baseline.md",
            );
        }
        "CSP_REPORT_ONLY" => {
            return (
                "CSP Report-Only Mode",
                "CSP is in report-only mode; move to enforcing mode for stronger production posture when ready.",
                &POLICY_FIXES,
                "docs/book/63-security-middleware-baseline.md",
            );
        }
        "HSTS_DISABLED_IN_PROD" => {
            return (
                "HSTS Disabled in Production",
                "HSTS is disabled for production HTTPS posture; enable HSTS with explicit max-age and subdomain policy.",
                &POLICY_FIXES,
                "docs/book/63-security-middleware-baseline.md",
            );
        }
        "REFERRER_POLICY_WEAK" => {
            return (
                "Weak Referrer Policy",
                "Referrer policy is weaker than recommended defaults and may expose extra cross-origin request context.",
                &POLICY_FIXES,
                "docs/book/63-security-middleware-baseline.md",
            );
        }
        "NOSNIFF_DISABLED" => {
            return (
                "X-Content-Type-Options Disabled",
                "The nosniff header is disabled; enable X-Content-Type-Options to reduce content-type confusion risk.",
                &POLICY_FIXES,
                "docs/book/63-security-middleware-baseline.md",
            );
        }
        "XFO_DISABLED" => {
            return (
                "X-Frame-Options Disabled",
                "X-Frame-Options is not set to a hardened mode; enable DENY or SAMEORIGIN per security posture.",
                &POLICY_FIXES,
                "docs/book/63-security-middleware-baseline.md",
            );
        }
        "CSRF_REQUIRED_BUT_DISABLED" => {
            return (
                "CSRF Required but Disabled",
                "CSRF protection is required for cookie-auth surfaces and must be enabled or auth mode adjusted.",
                &POLICY_FIXES,
                "docs/book/68-auth-policy-keys-and-csrf-coupling.md",
            );
        }
        "CSRF_PROTECTED_METHODS_INCOMPLETE" => {
            return (
                "CSRF Protected Methods Incomplete",
                "CSRF protection method coverage is incomplete; include unsafe methods (POST/PUT/PATCH/DELETE).",
                &POLICY_FIXES,
                "docs/book/63-security-middleware-baseline.md",
            );
        }
        "COOKIE_CROSS_SITE_WITHOUT_CORS_CREDS" => {
            return (
                "Cookie Cross-Site Without CORS Credentials",
                "Cross-site cookie auth requires credentialed CORS with explicit origin allowlists.",
                &POLICY_FIXES,
                "docs/book/68-auth-policy-keys-and-csrf-coupling.md",
            );
        }
        "COOKIE_CROSS_SITE_WITH_WILDCARD_ORIGIN" => {
            return (
                "Cookie Cross-Site With Wildcard Origin",
                "Cross-site cookie auth cannot rely on wildcard CORS origins; use explicit origin allowlists.",
                &POLICY_FIXES,
                "docs/book/68-auth-policy-keys-and-csrf-coupling.md",
            );
        }
        "COOKIE_SAMESITE_NONE_WITHOUT_SECURE" => {
            return (
                "SameSite=None Without Secure Cookie",
                "SameSite=None cookie posture requires Secure=true to avoid cross-site downgrade risk.",
                &POLICY_FIXES,
                "docs/book/68-auth-policy-keys-and-csrf-coupling.md",
            );
        }
        "INTERNAL_NET_ENABLED_NO_ALLOWLIST" => {
            return (
                "Internal Network Enabled Without Allowlist",
                "Internal network access is enabled without CIDR/domain allowlists and must be constrained.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "INTERNAL_NET_CALL_ALLOWLISTED" => {
            return (
                "Internal Network Call Allowlisted",
                "An internal-network call is allowlisted; verify ticket/expiry and ensure the allowlist remains narrow.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "PUBLIC_REDIRECTS_ENABLED_WITHOUT_REVALIDATION" => {
            return (
                "Public Redirects Without Revalidation",
                "Public redirect handling is enabled without per-hop revalidation; disable redirects or enforce hop checks.",
                &POLICY_FIXES,
                "docs/book/53-v0-security-baseline.md",
            );
        }
        "DNS_RESOLUTION_DISABLED" => {
            return (
                "DNS Resolution Disabled for SSRF Checks",
                "SSRF hardening is incomplete because DNS resolution checks are disabled for public URL validation.",
                &POLICY_FIXES,
                "docs/book/53-v0-security-baseline.md",
            );
        }
        "PUBLIC_EGRESS_NO_DOMAIN_POLICY" => {
            return (
                "Public Egress Domain Policy Missing",
                "Public egress is unconstrained by allowlist/blocklist policy; define domain controls for tighter posture.",
                &POLICY_FIXES,
                "docs/book/60-v0-policy-keys-spec.md",
            );
        }
        "FS_ENABLED_NO_BASE_ALLOWLIST" => {
            return (
                "Filesystem Enabled Without Base Allowlist",
                "Filesystem access is enabled without constrained base paths; restrict writable/readable roots.",
                &POLICY_FIXES,
                "docs/book/60-v0-policy-keys-spec.md",
            );
        }
        "SYMLINK_POLICY_WEAK" => {
            return (
                "Filesystem Symlink Policy Weak",
                "Filesystem policy allows weak symlink handling; enforce stricter symlink restrictions for path safety.",
                &POLICY_FIXES,
                "docs/book/60-v0-policy-keys-spec.md",
            );
        }
        "CAPTURE_REDACTION_INCOMPLETE" => {
            return (
                "Capture Redaction Incomplete",
                "Request capture redaction policy is missing mandatory sensitive header/path redactions.",
                &POLICY_FIXES,
                "docs/book/59-request-capture-and-deterministic-replay.md",
            );
        }
        "CAPTURE_ALL_IN_PROD" => {
            return (
                "Capture-All Mode in Production",
                "Capture mode is too broad for production; use errors-only or sampling with strict redaction controls.",
                &POLICY_FIXES,
                "docs/book/59-request-capture-and-deterministic-replay.md",
            );
        }
        "REPLAY_EFFECTS_ALLOW" => {
            return (
                "Replay Effects Allow Mode",
                "Replay is configured to allow live effects; prefer deny/mock outside isolated environments.",
                &POLICY_FIXES,
                "docs/book/59-request-capture-and-deterministic-replay.md",
            );
        }
        "LOG_STRUCTURED_ONLY_DISABLED" => {
            return (
                "Structured Logging Requirement Disabled",
                "Structured-only logging is disabled; enforce structured logs to reduce injection/leakage risk.",
                &POLICY_FIXES,
                "docs/book/58-success-envelope-and-log-event-schema.md",
            );
        }
        "LOG_REMOTE_IP_ENABLED" => {
            return (
                "Remote IP Logging Enabled",
                "Remote IP logging is enabled and can increase privacy exposure; verify explicit policy intent.",
                &POLICY_FIXES,
                "docs/book/58-success-envelope-and-log-event-schema.md",
            );
        }
        "LOG_USER_AGENT_ENABLED" => {
            return (
                "User-Agent Logging Enabled",
                "User-Agent logging is enabled and may capture high-cardinality/sensitive client metadata.",
                &POLICY_FIXES,
                "docs/book/58-success-envelope-and-log-event-schema.md",
            );
        }
        "SQL_RAW_ALLOWED_BY_POLICY" => {
            return (
                "SQL Raw Usage Allowed by Policy",
                "Policy permits raw SQL paths; tighten policy and prefer typed SqlQuery construction.",
                &POLICY_FIXES,
                "docs/book/53-v0-security-baseline.md",
            );
        }
        "SQL_LIMIT_RULE_DISABLED" => {
            return (
                "SQL Limit Rule Disabled",
                "SELECT limit hygiene rule is disabled; enable warn/enforce policy mode for safer query posture.",
                &POLICY_FIXES,
                "docs/book/60-v0-policy-keys-spec.md",
            );
        }
        "SQL_SELECT_WITHOUT_LIMIT" => {
            return (
                "SQL Select Without Limit",
                "A SELECT query path lacks LIMIT under current policy expectations.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "SECRETS_REVEAL_USED" => {
            return (
                "Secrets Reveal Usage Detected",
                "Secret reveal paths are present and must be removed or tightly allowlisted with governance metadata.",
                &POLICY_FIXES,
                "docs/book/53-v0-security-baseline.md",
            );
        }
        "SECRETS_REVEAL_ALLOWLISTED" => {
            return (
                "Secrets Reveal Allowlisted",
                "Secret reveal usage is allowlisted; review expiry/ticket and minimize scope.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "ALLOW_COUNT_HIGH" => {
            return (
                "Allowlist Exception Count High",
                "The project has a high number of allowlist exceptions and should reduce bypass surface.",
                &POLICY_FIXES,
                "docs/book/64-sec-audit-spec.md",
            );
        }
        "ALLOW_EXPIRY_WINDOW_ROLLUP" => {
            return (
                "Allowlist Expiry Window Risk Rollup",
                "Allowlist expiry concentration indicates renewal/removal backlog and elevated governance risk.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "ALLOW_EXPIRED" => {
            return (
                "Expired Policy Allowlist Exception",
                "An allowlisted exception has passed its expiry and must be removed, renewed, or replaced.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "ALLOW_EXPIRING_SOON" => {
            return (
                "Allowlist Exception Near Expiry",
                "An allowlisted exception is nearing expiry and requires review to renew or remove.",
                &POLICY_FIXES,
                "docs/book/66-deterministic-severity-mapping-for-sec-audit.md",
            );
        }
        "E1002" => {
            return (
                "Untrusted Input Reached Typed Sink",
                "An Untrusted<T> value reached a sink that requires a trusted/safe typed input.",
                &TRUST_FIXES,
                "docs/book/56-security-diagnostics-taxonomy.md",
            );
        }
        "E1003" => {
            return (
                "Secret Value Passed to Logging",
                "A Secret<T> value was routed into logging output without approved redaction/reveal flow.",
                &TRUST_FIXES,
                "docs/book/56-security-diagnostics-taxonomy.md",
            );
        }
        "E2001" => {
            return (
                "Missing Effect Declaration",
                "Function body uses one or more effects that are not declared in its signature.",
                &EFFECT_FIXES,
                "docs/book/55-v0-typing-effects-security-rules.md",
            );
        }
        "E2002" => {
            return (
                "Effect Forbidden by Policy",
                "An effect usage is blocked by policy unless explicitly allowlisted with required metadata.",
                &POLICY_FIXES,
                "docs/book/60-v0-policy-keys-spec.md",
            );
        }
        "E2003" => {
            return (
                "Missing Required Capability",
                "A sensitive API call was made without the required capability token in scope.",
                &EFFECT_FIXES,
                "docs/book/54-v0-stdlib-security-surface.md",
            );
        }
        "E4001" => {
            return (
                "Invalid Typed API Call Contract",
                "A typed security/schema API call has invalid arity or argument typing for its required contract.",
                &SCHEMA_FIXES,
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
