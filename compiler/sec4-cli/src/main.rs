use base64::Engine;
use clap::{Args, Parser, Subcommand, ValueEnum};
use sec4_core::{
    analyze_entry, analyze_entry_with_allows, analyze_program_with_policy,
    build_security_map_with_allows, emit_program_with_backend, parse_source,
    render_security_audit_text, run_security_audit_with_baseline, should_fail,
    strip_allow_annotations, summarize_history_window, validate_lockfile_stub,
    write_build_metadata, write_lockfile_stub, write_sbom, write_security_map,
    AuditHistoryWindowSummary, AuditReport, AuditSeverity, BackendEmitOutput, BackendKind,
    Diagnostic, Policy,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, TrySendError};
use std::sync::{Arc, Mutex};
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
        #[arg(long, value_enum, default_value_t = BuildTlsBackend::Auto)]
        tls_backend: BuildTlsBackend,
    },
    Init {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        name: Option<String>,
    },
    Run {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        port: Option<u16>,
        #[arg(long, default_value_t = false)]
        oneshot: bool,
        #[arg(long)]
        max_body_bytes: Option<u64>,
        #[arg(long)]
        max_concurrency: Option<u64>,
        #[arg(long)]
        max_pending: Option<u64>,
        #[arg(long)]
        serve_timeout_ms: Option<u64>,
        #[arg(long, value_enum, default_value_t = RunBackend::C)]
        backend: RunBackend,
        #[arg(long, value_enum, default_value_t = BuildTlsBackend::Auto)]
        tls_backend: BuildTlsBackend,
    },
    Promote {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum)]
        from: PromoteTarget,
        #[arg(long, value_enum)]
        to: PromoteTarget,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    LasmSmoke {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value = "GET")]
        method: String,
        #[arg(long, default_value = "/health")]
        route: String,
        #[arg(long)]
        request_path: Option<String>,
        #[arg(long)]
        max_in_flight: Option<usize>,
        #[arg(long)]
        max_pending: Option<usize>,
        #[arg(long)]
        max_request_ms: Option<u64>,
        #[arg(long)]
        runtime_script: Option<String>,
        #[arg(long, default_value_t = 1)]
        requests: usize,
        #[arg(long, default_value_t = 128)]
        max_steps: usize,
        #[arg(long, default_value_t = false)]
        fail_on_errors: bool,
        #[arg(long, value_enum, default_value_t = LasmSmokeOutputFormat::Text)]
        format: LasmSmokeOutputFormat,
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
    Lasm,
    LasmJson,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum BuildTlsBackend {
    Auto,
    None,
    Openssl,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum RunBackend {
    C,
    Lasm,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum EmitTarget {
    Ast,
    DiagnosticsJson,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum PromoteTarget {
    Browser,
    Server,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum LasmSmokeOutputFormat {
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockDependencySignatures {
    db: Vec<String>,
    fs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockDbStubMatch {
    row_count: i64,
    truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockFsStubMatch {
    ok: bool,
    truncated: bool,
    bytes: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockDbDependencyStubSummary {
    signature: String,
    row_count: i64,
    truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockFsDependencyStubSummary {
    signature: String,
    ok: bool,
    truncated: bool,
    bytes: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockDependencyStubSummaries {
    db: Vec<ReplayMockDbDependencyStubSummary>,
    fs: Vec<ReplayMockFsDependencyStubSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockDbDependencyTrace {
    index: usize,
    trace_id: String,
    signature: String,
    row_count: i64,
    truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockFsDependencyTrace {
    index: usize,
    trace_id: String,
    signature: String,
    ok: bool,
    truncated: bool,
    bytes: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockDependencyTraces {
    db: Vec<ReplayMockDbDependencyTrace>,
    fs: Vec<ReplayMockFsDependencyTrace>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockExecutionCounts {
    net: usize,
    db: usize,
    fs: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockNetExecutionTrace {
    index: usize,
    trace_id: String,
    signature: String,
    status: i64,
    truncated: bool,
    body_kind: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayMockExecutionTraces {
    net: Vec<ReplayMockNetExecutionTrace>,
    db: Vec<ReplayMockDbDependencyTrace>,
    fs: Vec<ReplayMockFsDependencyTrace>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PromoteBindingReference {
    file: String,
    line: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PromotePrecondition {
    code: String,
    severity: String,
    message: String,
    file: Option<String>,
    line: Option<usize>,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Build {
            path,
            emit,
            locked,
            sbom,
            tls_backend,
        } => cmd_build(&path, emit, locked, sbom, tls_backend),
        Commands::Init { path, name } => cmd_init(&path, name.as_deref()),
        Commands::Check { path, emit } => cmd_check(&path, emit),
        Commands::Run {
            path,
            port,
            oneshot,
            max_body_bytes,
            max_concurrency,
            max_pending,
            serve_timeout_ms,
            backend,
            tls_backend,
        } => cmd_run(
            &path,
            port,
            oneshot,
            max_body_bytes,
            max_concurrency,
            max_pending,
            serve_timeout_ms,
            backend,
            tls_backend,
        ),
        Commands::Promote {
            path,
            from,
            to,
            dry_run,
            out,
        } => cmd_promote(&path, from, to, dry_run, out.as_deref()),
        Commands::LasmSmoke {
            path,
            method,
            route,
            request_path,
            max_in_flight,
            max_pending,
            max_request_ms,
            runtime_script,
            requests,
            max_steps,
            fail_on_errors,
            format,
        } => cmd_lasm_smoke(
            &path,
            &method,
            &route,
            request_path.as_deref(),
            max_in_flight,
            max_pending,
            max_request_ms,
            runtime_script.as_deref(),
            requests,
            max_steps,
            fail_on_errors,
            format,
        ),
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

fn cmd_lasm_smoke(
    path: &Path,
    method: &str,
    route: &str,
    request_path: Option<&str>,
    max_in_flight: Option<usize>,
    max_pending: Option<usize>,
    max_request_ms: Option<u64>,
    runtime_script: Option<&str>,
    requests: usize,
    max_steps: usize,
    fail_on_errors: bool,
    format: LasmSmokeOutputFormat,
) -> Result<(), i32> {
    if requests == 0 {
        eprintln!("lasm-smoke failed: --requests must be >= 1");
        return Err(2);
    }
    if max_steps == 0 {
        eprintln!("lasm-smoke failed: --max-steps must be >= 1");
        return Err(2);
    }

    let manifest = match sec4_core::validate_project(path) {
        Ok(manifest) => manifest,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            return Err(1);
        }
    };

    let program = match analyze_entry(path, &manifest) {
        Ok(program) => program,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            return Err(1);
        }
    };

    let mir = sec4_core::lower_program_to_mir(&program);
    let lasm_program = sec4_core::lower_mir_to_lasm(&mir);
    let Some(entry) = lasm_program.entry.as_ref() else {
        eprintln!("lasm-smoke failed: compiled program does not expose an entrypoint");
        return Err(1);
    };

    let method = method.trim();
    if method.is_empty() {
        eprintln!("lasm-smoke failed: --method must not be empty");
        return Err(2);
    }

    let route = route.trim();
    if route.is_empty() {
        eprintln!("lasm-smoke failed: --route must not be empty");
        return Err(2);
    }
    let request_path = request_path.unwrap_or(route).trim();
    if request_path.is_empty() {
        eprintln!("lasm-smoke failed: --request-path must not be empty when provided");
        return Err(2);
    }

    let runtime_actions = if let Some(script) = runtime_script {
        match parse_lasm_runtime_script(script) {
            Ok(actions) => actions,
            Err(message) => {
                eprintln!("lasm-smoke failed: {message}");
                return Err(2);
            }
        }
    } else {
        vec![
            sec4_core::RuntimeAction::Yield,
            sec4_core::RuntimeAction::Complete(0),
        ]
    };

    let mut runtime = sec4_core::LasmHttpRuntime::default();
    let mut effective_max_in_flight = None;
    let mut effective_max_pending = None;
    let mut effective_max_request_ms = None;
    if let Some(limit) = max_in_flight {
        if let Err(message) = runtime.set_max_in_flight(limit) {
            eprintln!("lasm-smoke failed: {message}");
            return Err(2);
        }
        effective_max_in_flight = Some(limit);
    }
    if let Some(limit) = max_pending {
        if let Err(message) = runtime.set_max_pending(limit) {
            eprintln!("lasm-smoke failed: {message}");
            return Err(2);
        }
        effective_max_pending = Some(limit);
    }
    if let Some(limit) = max_request_ms {
        if let Err(message) = runtime.set_max_request_duration_ms(limit) {
            eprintln!("lasm-smoke failed: {message}");
            return Err(2);
        }
        effective_max_request_ms = Some(limit);
    }
    let (response_status, response_headers, response_body, response_origin) =
        match resolve_lasm_smoke_route_plan(&program, entry.name.as_str(), method, route) {
            Some(route_plan) => (
                route_plan.status,
                route_plan.headers,
                route_plan.body,
                format!("handler:{}", route_plan.handler_name),
            ),
            None => (
                200,
                BTreeMap::new(),
                format!("lasm entry {} ok", entry.name),
                "entry".to_string(),
            ),
        };
    let mut response = sec4_core::HttpResponse::text(response_status, response_body);
    response.headers = response_headers;
    if let Err(message) = runtime.register_route(method, route, runtime_actions, response) {
        eprintln!("lasm-smoke failed: {message}");
        return Err(1);
    }

    for _ in 0..requests {
        runtime.submit(sec4_core::HttpRequest::new(method, request_path));
    }
    let report = runtime.run_until_idle(max_steps);
    if !report.idle {
        eprintln!(
            "lasm-smoke failed: runtime remained active after step budget ({max_steps}); in_flight={} pending={}",
            runtime.in_flight_request_count(),
            runtime.pending_request_count()
        );
        return Err(1);
    }

    let mut response_count = 0usize;
    let mut ok_count = 0usize;
    let mut error_count = 0usize;
    let mut status_counts: BTreeMap<u16, usize> = BTreeMap::new();
    let mut min_duration_ms: Option<u64> = None;
    let mut max_duration_ms: Option<u64> = None;
    let mut total_duration_ms: u128 = 0;
    let mut first_request_id = None;
    let mut first_response_id = None;
    let mut first_status = None;
    let mut first_duration_ms = None;
    let mut first_path_params = None;
    let mut first_headers = None;
    let mut first_body = None;
    while let Some(exchange) = runtime.pop_response() {
        let duration_ms = exchange.duration_ms();
        response_count += 1;
        *status_counts.entry(exchange.response.status).or_insert(0) += 1;
        min_duration_ms = Some(match min_duration_ms {
            Some(current) => current.min(duration_ms),
            None => duration_ms,
        });
        max_duration_ms = Some(match max_duration_ms {
            Some(current) => current.max(duration_ms),
            None => duration_ms,
        });
        total_duration_ms = total_duration_ms.saturating_add(duration_ms as u128);
        if exchange.response.status < 400 {
            ok_count += 1;
        } else {
            error_count += 1;
        }
        if first_request_id.is_none() {
            first_request_id = Some(exchange.request_id);
            first_response_id = Some(exchange.request_id);
            first_status = Some(exchange.response.status);
            first_duration_ms = Some(duration_ms);
            first_path_params = Some(exchange.path_params.clone());
            first_headers = Some(exchange.response.headers.clone());
            first_body = Some(String::from_utf8_lossy(&exchange.response.body).to_string());
        }
    }

    if response_count != requests {
        eprintln!("lasm-smoke failed: expected {requests} responses, received {response_count}");
        return Err(1);
    }

    if fail_on_errors && error_count > 0 {
        let status_counts_text = status_counts
            .iter()
            .map(|(status, count)| format!("{status}:{count}"))
            .collect::<Vec<_>>()
            .join(",");
        eprintln!(
            "lasm-smoke failed: error responses observed with --fail-on-errors (errors={} statusCounts={})",
            error_count, status_counts_text
        );
        return Err(1);
    }

    let min_duration_ms = min_duration_ms.unwrap_or(0);
    let max_duration_ms = max_duration_ms.unwrap_or(0);
    let avg_duration_ms = if response_count == 0 {
        0
    } else {
        (total_duration_ms / response_count as u128) as u64
    };

    let first_path_params = first_path_params.unwrap_or_default();
    let first_path_params_text = if first_path_params.is_empty() {
        "-".to_string()
    } else {
        first_path_params
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    let first_headers = first_headers.unwrap_or_default();
    let first_body = first_body.unwrap_or_default();
    match format {
        LasmSmokeOutputFormat::Text => {
            let max_in_flight_text = effective_max_in_flight
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unbounded".to_string());
            let max_pending_text = effective_max_pending
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unbounded".to_string());
            let max_request_ms_text = effective_max_request_ms
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unbounded".to_string());
            let status_counts_text = status_counts
                .iter()
                .map(|(status, count)| format!("{status}:{count}"))
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "lasm smoke succeeded: requestId={} responseRequestId={} entry={} origin={} requests={} maxInFlight={} maxPending={} maxRequestMs={} ok={} errors={} statusCounts={} durationMinMs={} durationMaxMs={} durationAvgMs={} steps={} nowMs={} status={} firstDurationMs={} pathParams={} headerCount={} body={}",
                first_request_id.unwrap_or(0),
                first_response_id.unwrap_or(0),
                entry.name,
                response_origin,
                requests,
                max_in_flight_text,
                max_pending_text,
                max_request_ms_text,
                ok_count,
                error_count,
                status_counts_text,
                min_duration_ms,
                max_duration_ms,
                avg_duration_ms,
                report.steps,
                report.now_ms,
                first_status.unwrap_or(0),
                first_duration_ms.unwrap_or(0),
                first_path_params_text,
                first_headers.len(),
                first_body
            );
        }
        LasmSmokeOutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "requestId": first_request_id.unwrap_or(0),
                "responseRequestId": first_response_id.unwrap_or(0),
                "entry": entry.name,
                "origin": response_origin,
                "requests": requests,
                "maxInFlight": effective_max_in_flight,
                "maxPending": effective_max_pending,
                "maxRequestMs": effective_max_request_ms,
                "requestPath": request_path,
                "okCount": ok_count,
                "errorCount": error_count,
                "statusCounts": status_counts
                    .iter()
                    .map(|(status, count)| (status.to_string(), *count))
                    .collect::<BTreeMap<String, usize>>(),
                "durationMs": {
                    "min": min_duration_ms,
                    "max": max_duration_ms,
                    "avg": avg_duration_ms
                },
                "steps": report.steps,
                "nowMs": report.now_ms,
                "status": first_status.unwrap_or(0),
                "firstDurationMs": first_duration_ms.unwrap_or(0),
                "pathParams": first_path_params,
                "headers": first_headers,
                "body": first_body,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload)
                    .expect("lasm-smoke payload should serialize as JSON")
            );
        }
    }
    Ok(())
}

fn parse_lasm_runtime_script(script: &str) -> Result<Vec<sec4_core::RuntimeAction>, String> {
    let mut actions = Vec::new();
    for raw_segment in script.split(',') {
        let segment = raw_segment.trim();
        if segment.is_empty() {
            continue;
        }
        if segment.eq_ignore_ascii_case("yield") {
            actions.push(sec4_core::RuntimeAction::Yield);
            continue;
        }
        if let Some(raw_ms) = segment.strip_prefix("sleep:") {
            let duration_ms = raw_ms.trim().parse::<u64>().map_err(|_| {
                format!("invalid --runtime-script segment `{segment}`: sleep duration must be u64")
            })?;
            actions.push(sec4_core::RuntimeAction::SleepMs(duration_ms));
            continue;
        }
        if let Some(raw_code) = segment.strip_prefix("complete:") {
            let code = raw_code.trim().parse::<i64>().map_err(|_| {
                format!("invalid --runtime-script segment `{segment}`: complete code must be i64")
            })?;
            actions.push(sec4_core::RuntimeAction::Complete(code));
            continue;
        }
        return Err(format!(
            "invalid --runtime-script segment `{segment}`: expected `yield`, `sleep:<ms>`, or `complete:<code>`"
        ));
    }

    if actions.is_empty() {
        return Err("invalid --runtime-script: expected at least one action segment".to_string());
    }
    Ok(actions)
}

#[derive(Debug, Clone)]
struct LasmSmokeRoutePlan {
    handler_name: String,
    status: u16,
    body: String,
    headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct LasmRouteRegistration {
    method: String,
    path: String,
    handler_name: String,
}

#[derive(Debug, Clone)]
struct LasmRunRoutePlan {
    method: String,
    path: String,
    status: u16,
    body: String,
    headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct LasmResponsePlan {
    status: u16,
    body: String,
    default_content_type: Option<String>,
}

fn collect_lasm_route_plans(
    program: &sec4_core::ast::Program,
    entry_name: &str,
) -> Vec<LasmRunRoutePlan> {
    let functions = program
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            sec4_core::ast::ItemKind::Function(function) => {
                Some((function.name.as_str(), function))
            }
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    let mut visited_functions = HashSet::new();
    let mut registrations = Vec::new();
    collect_route_registrations_in_function(
        &functions,
        entry_name,
        &mut visited_functions,
        &mut registrations,
    );

    let mut plans = Vec::new();
    let mut seen_routes = HashSet::new();
    for registration in registrations {
        let route_key = format!("{} {}", registration.method, registration.path);
        if !seen_routes.insert(route_key) {
            continue;
        }
        let Some(response_plan) = extract_response_plan(&functions, registration.handler_name.as_str())
        else {
            continue;
        };
        let mut headers = extract_response_headers(&functions, registration.handler_name.as_str());
        if let Some(content_type) = response_plan.default_content_type {
            headers.entry("Content-Type".to_string()).or_insert(content_type);
        }
        plans.push(LasmRunRoutePlan {
            method: registration.method,
            path: registration.path,
            status: response_plan.status,
            body: response_plan.body,
            headers,
        });
    }

    plans
}

fn resolve_lasm_smoke_route_plan(
    program: &sec4_core::ast::Program,
    entry_name: &str,
    method: &str,
    route: &str,
) -> Option<LasmSmokeRoutePlan> {
    let functions = program
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            sec4_core::ast::ItemKind::Function(function) => {
                Some((function.name.as_str(), function))
            }
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    let handler_name = find_route_handler_name(&functions, entry_name, method, route)?;
    let response_plan = extract_response_plan(&functions, handler_name.as_str())?;
    let mut headers = extract_response_headers(&functions, handler_name.as_str());
    if let Some(content_type) = response_plan.default_content_type {
        headers.entry("Content-Type".to_string()).or_insert(content_type);
    }

    Some(LasmSmokeRoutePlan {
        handler_name,
        status: response_plan.status,
        body: response_plan.body,
        headers,
    })
}

fn collect_route_registrations_in_function(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    visited: &mut HashSet<String>,
    registrations: &mut Vec<LasmRouteRegistration>,
) {
    if !visited.insert(function_name.to_string()) {
        return;
    }
    let Some(function) = functions.get(function_name) else {
        return;
    };
    let mut local_bindings = HashMap::new();
    collect_route_registrations_in_block(
        functions,
        &function.body,
        visited,
        registrations,
        &mut local_bindings,
    );
}

fn collect_route_registrations_in_block(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    block: &sec4_core::ast::Block,
    visited: &mut HashSet<String>,
    registrations: &mut Vec<LasmRouteRegistration>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) {
    for statement in &block.statements {
        collect_route_registrations_in_stmt(functions, statement, visited, registrations, bindings);
    }
    if let Some(tail) = &block.tail {
        collect_route_registrations_in_expr(functions, tail, visited, registrations, bindings);
    }
}

fn collect_route_registrations_in_stmt(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    statement: &sec4_core::ast::Stmt,
    visited: &mut HashSet<String>,
    registrations: &mut Vec<LasmRouteRegistration>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) {
    match &statement.kind {
        sec4_core::ast::StmtKind::Let { name, value, .. } => {
            collect_route_registrations_in_expr(functions, value, visited, registrations, bindings);
            bindings.insert(name.clone(), value.clone());
        }
        sec4_core::ast::StmtKind::Return { value } => {
            if let Some(value) = value {
                collect_route_registrations_in_expr(
                    functions,
                    value,
                    visited,
                    registrations,
                    bindings,
                );
            }
        }
        sec4_core::ast::StmtKind::Expr { expr } => {
            collect_route_registrations_in_expr(functions, expr, visited, registrations, bindings)
        }
    }
}

fn collect_route_registrations_in_expr(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    expr: &sec4_core::ast::Expr,
    visited: &mut HashSet<String>,
    registrations: &mut Vec<LasmRouteRegistration>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) {
    match &expr.kind {
        sec4_core::ast::ExprKind::Call { callee, args } => {
            if let Some(registration) = match_route_registration_details(callee, args, bindings) {
                registrations.push(registration);
            }
            if let sec4_core::ast::ExprKind::Identifier(function_name) = &callee.kind {
                collect_route_registrations_in_function(
                    functions,
                    function_name,
                    visited,
                    registrations,
                );
            }
            collect_route_registrations_in_expr(
                functions,
                callee,
                visited,
                registrations,
                bindings,
            );
            for argument in args {
                collect_route_registrations_in_expr(
                    functions,
                    argument,
                    visited,
                    registrations,
                    bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Unary { expr, .. } => {
            collect_route_registrations_in_expr(functions, expr, visited, registrations, bindings)
        }
        sec4_core::ast::ExprKind::Binary { left, right, .. } => {
            collect_route_registrations_in_expr(functions, left, visited, registrations, bindings);
            collect_route_registrations_in_expr(functions, right, visited, registrations, bindings);
        }
        sec4_core::ast::ExprKind::Member { object, .. } => {
            collect_route_registrations_in_expr(functions, object, visited, registrations, bindings)
        }
        sec4_core::ast::ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_route_registrations_in_expr(
                functions,
                condition,
                visited,
                registrations,
                bindings,
            );
            let mut then_bindings = bindings.clone();
            collect_route_registrations_in_block(
                functions,
                then_branch,
                visited,
                registrations,
                &mut then_bindings,
            );
            if let Some(else_branch) = else_branch {
                let mut else_bindings = bindings.clone();
                collect_route_registrations_in_expr(
                    functions,
                    else_branch,
                    visited,
                    registrations,
                    &mut else_bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Match { scrutinee, arms } => {
            collect_route_registrations_in_expr(
                functions,
                scrutinee,
                visited,
                registrations,
                bindings,
            );
            for arm in arms {
                let mut arm_bindings = bindings.clone();
                collect_route_registrations_in_expr(
                    functions,
                    &arm.value,
                    visited,
                    registrations,
                    &mut arm_bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Block(block) => {
            let mut block_bindings = bindings.clone();
            collect_route_registrations_in_block(
                functions,
                block,
                visited,
                registrations,
                &mut block_bindings,
            )
        }
        sec4_core::ast::ExprKind::Identifier(_)
        | sec4_core::ast::ExprKind::Number(_)
        | sec4_core::ast::ExprKind::String(_)
        | sec4_core::ast::ExprKind::Bool(_) => {}
    }
}

fn find_route_handler_name(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    entry_name: &str,
    method: &str,
    route: &str,
) -> Option<String> {
    let normalized_method = method.trim().to_ascii_uppercase();
    let mut visited = HashSet::new();
    let mut bindings = HashMap::new();
    find_route_handler_name_in_function(
        functions,
        entry_name,
        normalized_method.as_str(),
        route,
        &mut visited,
        &mut bindings,
    )
}

fn find_route_handler_name_in_function(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    method: &str,
    route: &str,
    visited: &mut HashSet<String>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    if !visited.insert(function_name.to_string()) {
        return None;
    }
    let function = functions.get(function_name)?;
    find_route_handler_name_in_block(functions, &function.body, method, route, visited, bindings)
}

fn find_route_handler_name_in_block(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    block: &sec4_core::ast::Block,
    method: &str,
    route: &str,
    visited: &mut HashSet<String>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    for statement in &block.statements {
        if let Some(handler_name) =
            find_route_handler_name_in_stmt(functions, statement, method, route, visited, bindings)
        {
            return Some(handler_name);
        }
    }
    if let Some(tail) = &block.tail {
        return find_route_handler_name_in_expr(functions, tail, method, route, visited, bindings);
    }
    None
}

fn find_route_handler_name_in_stmt(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    statement: &sec4_core::ast::Stmt,
    method: &str,
    route: &str,
    visited: &mut HashSet<String>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    match &statement.kind {
        sec4_core::ast::StmtKind::Let { name, value, .. } => {
            if let Some(handler_name) =
                find_route_handler_name_in_expr(functions, value, method, route, visited, bindings)
            {
                return Some(handler_name);
            }
            bindings.insert(name.clone(), value.clone());
            None
        }
        sec4_core::ast::StmtKind::Return { value } => value.as_ref().and_then(|entry| {
            find_route_handler_name_in_expr(functions, entry, method, route, visited, bindings)
        }),
        sec4_core::ast::StmtKind::Expr { expr } => {
            find_route_handler_name_in_expr(functions, expr, method, route, visited, bindings)
        }
    }
}

fn find_route_handler_name_in_expr(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    expr: &sec4_core::ast::Expr,
    method: &str,
    route: &str,
    visited: &mut HashSet<String>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    match &expr.kind {
        sec4_core::ast::ExprKind::Call { callee, args } => {
            if let Some(handler_name) =
                match_route_registration_call(callee, args, method, route, bindings)
            {
                return Some(handler_name);
            }
            if let sec4_core::ast::ExprKind::Identifier(function_name) = &callee.kind {
                let mut callee_bindings = HashMap::new();
                if let Some(handler_name) = find_route_handler_name_in_function(
                    functions,
                    function_name,
                    method,
                    route,
                    visited,
                    &mut callee_bindings,
                ) {
                    return Some(handler_name);
                }
            }
            if let Some(handler_name) =
                find_route_handler_name_in_expr(functions, callee, method, route, visited, bindings)
            {
                return Some(handler_name);
            }
            for argument in args {
                if let Some(handler_name) = find_route_handler_name_in_expr(
                    functions, argument, method, route, visited, bindings,
                ) {
                    return Some(handler_name);
                }
            }
            None
        }
        sec4_core::ast::ExprKind::Unary { expr, .. } => {
            find_route_handler_name_in_expr(functions, expr, method, route, visited, bindings)
        }
        sec4_core::ast::ExprKind::Binary { left, right, .. } => {
            find_route_handler_name_in_expr(functions, left, method, route, visited, bindings)
                .or_else(|| {
                    find_route_handler_name_in_expr(
                        functions, right, method, route, visited, bindings,
                    )
                })
        }
        sec4_core::ast::ExprKind::Member { object, .. } => {
            find_route_handler_name_in_expr(functions, object, method, route, visited, bindings)
        }
        sec4_core::ast::ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            find_route_handler_name_in_expr(functions, condition, method, route, visited, bindings)
                .or_else(|| {
                    let mut then_bindings = bindings.clone();
                    find_route_handler_name_in_block(
                        functions,
                        then_branch,
                        method,
                        route,
                        visited,
                        &mut then_bindings,
                    )
                })
                .or_else(|| {
                    else_branch.as_ref().and_then(|entry| {
                        let mut else_bindings = bindings.clone();
                        find_route_handler_name_in_expr(
                            functions,
                            entry,
                            method,
                            route,
                            visited,
                            &mut else_bindings,
                        )
                    })
                })
        }
        sec4_core::ast::ExprKind::Match { scrutinee, arms } => {
            if let Some(handler_name) = find_route_handler_name_in_expr(
                functions, scrutinee, method, route, visited, bindings,
            ) {
                return Some(handler_name);
            }
            for arm in arms {
                let mut arm_bindings = bindings.clone();
                if let Some(handler_name) = find_route_handler_name_in_expr(
                    functions,
                    &arm.value,
                    method,
                    route,
                    visited,
                    &mut arm_bindings,
                ) {
                    return Some(handler_name);
                }
            }
            None
        }
        sec4_core::ast::ExprKind::Block(block) => {
            let mut block_bindings = bindings.clone();
            find_route_handler_name_in_block(
                functions,
                block,
                method,
                route,
                visited,
                &mut block_bindings,
            )
        }
        sec4_core::ast::ExprKind::Identifier(_)
        | sec4_core::ast::ExprKind::Number(_)
        | sec4_core::ast::ExprKind::String(_)
        | sec4_core::ast::ExprKind::Bool(_) => None,
    }
}

fn match_route_registration_call(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    method: &str,
    route: &str,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    let registration = match_route_registration_details(callee, args, bindings)?;
    if registration.method != method || registration.path != route {
        return None;
    }
    Some(registration.handler_name)
}

fn match_route_registration_details(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmRouteRegistration> {
    let sec4_core::ast::ExprKind::Member { object, field } = &callee.kind else {
        return None;
    };
    let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind else {
        return None;
    };
    if namespace != "http" {
        return None;
    }
    let route_method = match field.as_str() {
        "get" => "GET",
        "post" => "POST",
        "put" => "PUT",
        "patch" => "PATCH",
        "delete" => "DELETE",
        "options" => "OPTIONS",
        "head" => "HEAD",
        _ => return None,
    };
    let (route_arg_index, handler_arg_index) = if args.len() >= 3 {
        (1usize, 2usize)
    } else if args.len() >= 2 {
        (0usize, 1usize)
    } else {
        return None;
    };

    let route_path = extract_route_path_literal(&args[route_arg_index], bindings)?;
    let handler_name = extract_handler_identifier(&args[handler_arg_index], bindings)?;
    Some(LasmRouteRegistration {
        method: route_method.to_string(),
        path: route_path,
        handler_name,
    })
}

fn extract_route_path_literal(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    let resolved = resolve_route_registration_expr(expr, bindings, 0)?;
    let sec4_core::ast::ExprKind::String(path) = &resolved.kind else {
        return None;
    };
    Some(path.clone())
}

fn extract_handler_identifier(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    let resolved = resolve_route_registration_expr(expr, bindings, 0)?;
    let sec4_core::ast::ExprKind::Identifier(name) = &resolved.kind else {
        return None;
    };
    Some(name.clone())
}

fn resolve_route_registration_expr(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<sec4_core::ast::Expr> {
    if depth > 32 {
        return None;
    }
    match &expr.kind {
        sec4_core::ast::ExprKind::Identifier(name) => {
            if let Some(bound) = bindings.get(name) {
                resolve_route_registration_expr(bound, bindings, depth + 1)
            } else {
                Some(expr.clone())
            }
        }
        _ => Some(expr.clone()),
    }
}

fn extract_response_plan(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
) -> Option<LasmResponsePlan> {
    let mut visited = HashSet::new();
    extract_response_plan_in_function(functions, function_name, &mut visited)
}

fn extract_response_plan_in_function(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    visited: &mut HashSet<String>,
) -> Option<LasmResponsePlan> {
    if !visited.insert(function_name.to_string()) {
        return None;
    }
    let function = functions.get(function_name)?;
    let mut bindings = HashMap::new();
    extract_response_plan_in_block(functions, &function.body, visited, &mut bindings)
}

fn extract_response_plan_in_block(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    block: &sec4_core::ast::Block,
    visited: &mut HashSet<String>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    for statement in &block.statements {
        if let Some(response) =
            extract_response_plan_in_stmt(functions, statement, visited, bindings)
        {
            return Some(response);
        }
    }
    if let Some(tail) = &block.tail {
        return extract_response_plan_in_expr(functions, tail, visited, bindings);
    }
    None
}

fn extract_response_plan_in_stmt(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    statement: &sec4_core::ast::Stmt,
    visited: &mut HashSet<String>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    match &statement.kind {
        sec4_core::ast::StmtKind::Let { name, value, .. } => {
            if let Some(response) =
                extract_response_plan_in_expr(functions, value, visited, bindings)
            {
                return Some(response);
            }
            bindings.insert(name.clone(), value.clone());
            None
        }
        sec4_core::ast::StmtKind::Return { value } => value
            .as_ref()
            .and_then(|entry| extract_response_plan_in_expr(functions, entry, visited, bindings)),
        sec4_core::ast::StmtKind::Expr { expr } => {
            extract_response_plan_in_expr(functions, expr, visited, bindings)
        }
    }
}

fn extract_response_plan_in_expr(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    expr: &sec4_core::ast::Expr,
    visited: &mut HashSet<String>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    match &expr.kind {
        sec4_core::ast::ExprKind::Call { callee, args } => {
            if let Some(response) = match_response_helper_call(callee, args, bindings) {
                return Some(response);
            }
            if let sec4_core::ast::ExprKind::Identifier(function_name) = &callee.kind {
                if let Some(response) =
                    extract_response_plan_in_function(functions, function_name, visited)
                {
                    return Some(response);
                }
            }
            if let Some(response) =
                extract_response_plan_in_expr(functions, callee, visited, bindings)
            {
                return Some(response);
            }
            for argument in args {
                if let Some(response) =
                    extract_response_plan_in_expr(functions, argument, visited, bindings)
                {
                    return Some(response);
                }
            }
            None
        }
        sec4_core::ast::ExprKind::Unary { expr, .. } => {
            extract_response_plan_in_expr(functions, expr, visited, bindings)
        }
        sec4_core::ast::ExprKind::Binary { left, right, .. } => {
            extract_response_plan_in_expr(functions, left, visited, bindings)
                .or_else(|| extract_response_plan_in_expr(functions, right, visited, bindings))
        }
        sec4_core::ast::ExprKind::Member { object, .. } => {
            extract_response_plan_in_expr(functions, object, visited, bindings)
        }
        sec4_core::ast::ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => extract_response_plan_in_expr(functions, condition, visited, bindings)
            .or_else(|| {
                let mut then_bindings = bindings.clone();
                extract_response_plan_in_block(functions, then_branch, visited, &mut then_bindings)
            })
            .or_else(|| {
                let mut else_bindings = bindings.clone();
                else_branch.as_ref().and_then(|entry| {
                    extract_response_plan_in_expr(functions, entry, visited, &mut else_bindings)
                })
            }),
        sec4_core::ast::ExprKind::Match { scrutinee, arms } => {
            if let Some(response) =
                extract_response_plan_in_expr(functions, scrutinee, visited, bindings)
            {
                return Some(response);
            }
            for arm in arms {
                let mut arm_bindings = bindings.clone();
                if let Some(response) =
                    extract_response_plan_in_expr(functions, &arm.value, visited, &mut arm_bindings)
                {
                    return Some(response);
                }
            }
            None
        }
        sec4_core::ast::ExprKind::Block(block) => {
            let mut block_bindings = bindings.clone();
            extract_response_plan_in_block(functions, block, visited, &mut block_bindings)
        }
        sec4_core::ast::ExprKind::Identifier(_)
        | sec4_core::ast::ExprKind::Number(_)
        | sec4_core::ast::ExprKind::String(_)
        | sec4_core::ast::ExprKind::Bool(_) => None,
    }
}

fn match_response_helper_call(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    let sec4_core::ast::ExprKind::Member { object, field } = &callee.kind else {
        return None;
    };
    let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind else {
        return None;
    };
    if namespace != "res" {
        return None;
    }

    match field.as_str() {
        "text" => match_res_text_call(args, bindings),
        "html" => match_res_html_call(args, bindings),
        "json" => match_res_json_call(args, bindings),
        "ok" => match_res_ok_call(args, bindings),
        "okMeta" => match_res_ok_meta_call(args, bindings),
        _ => None,
    }
}

fn match_res_text_call(
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    if args.len() < 2 {
        return None;
    }
    let status = parse_status_literal(&args[0], bindings)?;
    let body = parse_string_literal(&args[1], bindings)?;
    Some(LasmResponsePlan {
        status,
        body,
        default_content_type: Some("text/plain; charset=utf-8".to_string()),
    })
}

fn match_res_html_call(
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    if args.len() != 1 {
        return None;
    }
    let body =
        parse_string_literal(&args[0], bindings).unwrap_or_else(|| "<html></html>".to_string());
    Some(LasmResponsePlan {
        status: 200,
        body,
        default_content_type: Some("text/html; charset=utf-8".to_string()),
    })
}

fn match_res_json_call(
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    let status = match args.len() {
        2 => 200,
        length if length >= 3 => parse_status_literal(&args[0], bindings)?,
        _ => return None,
    };
    Some(LasmResponsePlan {
        status,
        body: "json response".to_string(),
        default_content_type: Some("application/json; charset=utf-8".to_string()),
    })
}

fn match_res_ok_call(
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    if args.len() < 3 {
        return None;
    }
    let status = parse_status_literal(&args[0], bindings)?;
    Some(LasmResponsePlan {
        status,
        body: "ok response".to_string(),
        default_content_type: Some("application/json; charset=utf-8".to_string()),
    })
}

fn match_res_ok_meta_call(
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    if args.len() < 4 {
        return None;
    }
    let status = parse_status_literal(&args[0], bindings)?;
    Some(LasmResponsePlan {
        status,
        body: "ok response".to_string(),
        default_content_type: Some("application/json; charset=utf-8".to_string()),
    })
}

fn parse_status_literal(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<u16> {
    let resolved = resolve_response_expr(expr, bindings, 0)?;
    let sec4_core::ast::ExprKind::Number(status_literal) = &resolved.kind else {
        return None;
    };
    status_literal.parse::<u16>().ok()
}

fn parse_string_literal(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    let resolved = resolve_response_expr(expr, bindings, 0)?;
    let sec4_core::ast::ExprKind::String(value) = &resolved.kind else {
        return None;
    };
    Some(value.clone())
}

fn resolve_response_expr(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<sec4_core::ast::Expr> {
    if depth > 32 {
        return None;
    }
    match &expr.kind {
        sec4_core::ast::ExprKind::Identifier(name) => {
            let bound = bindings.get(name)?;
            resolve_response_expr(bound, bindings, depth + 1)
        }
        _ => Some(expr.clone()),
    }
}

fn extract_response_headers(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
) -> BTreeMap<String, String> {
    let mut headers = BTreeMap::new();
    let mut visited = HashSet::new();
    extract_response_headers_in_function(functions, function_name, &mut visited, &mut headers);
    headers
}

fn extract_response_headers_in_function(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    visited: &mut HashSet<String>,
    headers: &mut BTreeMap<String, String>,
) {
    if !visited.insert(function_name.to_string()) {
        return;
    }
    let Some(function) = functions.get(function_name) else {
        return;
    };
    let mut local_bindings = HashMap::new();
    extract_response_headers_in_block(
        functions,
        &function.body,
        visited,
        headers,
        &mut local_bindings,
    );
}

fn extract_response_headers_in_block(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    block: &sec4_core::ast::Block,
    visited: &mut HashSet<String>,
    headers: &mut BTreeMap<String, String>,
    bindings: &mut HashMap<String, String>,
) {
    for statement in &block.statements {
        extract_response_headers_in_stmt(functions, statement, visited, headers, bindings);
    }
    if let Some(tail) = &block.tail {
        extract_response_headers_in_expr(functions, tail, visited, headers, bindings);
    }
}

fn extract_response_headers_in_stmt(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    statement: &sec4_core::ast::Stmt,
    visited: &mut HashSet<String>,
    headers: &mut BTreeMap<String, String>,
    bindings: &mut HashMap<String, String>,
) {
    match &statement.kind {
        sec4_core::ast::StmtKind::Let { name, value, .. } => {
            if let Some(binding_value) = extract_header_binding_literal(value, bindings) {
                bindings.insert(name.clone(), binding_value);
            }
            extract_response_headers_in_expr(functions, value, visited, headers, bindings)
        }
        sec4_core::ast::StmtKind::Return { value } => {
            if let Some(value) = value {
                extract_response_headers_in_expr(functions, value, visited, headers, bindings);
            }
        }
        sec4_core::ast::StmtKind::Expr { expr } => {
            extract_response_headers_in_expr(functions, expr, visited, headers, bindings)
        }
    }
}

fn extract_response_headers_in_expr(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    expr: &sec4_core::ast::Expr,
    visited: &mut HashSet<String>,
    headers: &mut BTreeMap<String, String>,
    bindings: &mut HashMap<String, String>,
) {
    match &expr.kind {
        sec4_core::ast::ExprKind::Call { callee, args } => {
            if let Some((name, value)) = match_res_set_header_call(callee, args, bindings) {
                headers.insert(name, value);
            }
            if let Some(cookie) = match_res_add_cookie_call(callee, args, bindings) {
                headers.insert("Set-Cookie".to_string(), cookie);
            }
            if let sec4_core::ast::ExprKind::Identifier(function_name) = &callee.kind {
                extract_response_headers_in_function(functions, function_name, visited, headers);
            }
            extract_response_headers_in_expr(functions, callee, visited, headers, bindings);
            for argument in args {
                extract_response_headers_in_expr(functions, argument, visited, headers, bindings);
            }
        }
        sec4_core::ast::ExprKind::Unary { expr, .. } => {
            extract_response_headers_in_expr(functions, expr, visited, headers, bindings)
        }
        sec4_core::ast::ExprKind::Binary { left, right, .. } => {
            extract_response_headers_in_expr(functions, left, visited, headers, bindings);
            extract_response_headers_in_expr(functions, right, visited, headers, bindings);
        }
        sec4_core::ast::ExprKind::Member { object, .. } => {
            extract_response_headers_in_expr(functions, object, visited, headers, bindings)
        }
        sec4_core::ast::ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            extract_response_headers_in_expr(functions, condition, visited, headers, bindings);
            let mut then_bindings = bindings.clone();
            extract_response_headers_in_block(
                functions,
                then_branch,
                visited,
                headers,
                &mut then_bindings,
            );
            if let Some(else_branch) = else_branch {
                let mut else_bindings = bindings.clone();
                extract_response_headers_in_expr(
                    functions,
                    else_branch,
                    visited,
                    headers,
                    &mut else_bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Match { scrutinee, arms } => {
            extract_response_headers_in_expr(functions, scrutinee, visited, headers, bindings);
            for arm in arms {
                let mut arm_bindings = bindings.clone();
                extract_response_headers_in_expr(
                    functions,
                    &arm.value,
                    visited,
                    headers,
                    &mut arm_bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Block(block) => {
            let mut block_bindings = bindings.clone();
            extract_response_headers_in_block(
                functions,
                block,
                visited,
                headers,
                &mut block_bindings,
            )
        }
        sec4_core::ast::ExprKind::Identifier(_)
        | sec4_core::ast::ExprKind::Number(_)
        | sec4_core::ast::ExprKind::String(_)
        | sec4_core::ast::ExprKind::Bool(_) => {}
    }
}

fn match_res_set_header_call(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, String>,
) -> Option<(String, String)> {
    if args.len() < 2 {
        return None;
    }
    let sec4_core::ast::ExprKind::Member { object, field } = &callee.kind else {
        return None;
    };
    let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind else {
        return None;
    };
    if namespace != "res" || field != "setHeader" {
        return None;
    }
    let name = extract_header_gate_literal(&args[0], "name", bindings)?;
    let value = extract_header_gate_literal(&args[1], "value", bindings)?;
    Some((name, value))
}

fn match_res_add_cookie_call(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, String>,
) -> Option<String> {
    if args.is_empty() {
        return None;
    }
    let sec4_core::ast::ExprKind::Member { object, field } = &callee.kind else {
        return None;
    };
    let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind else {
        return None;
    };
    if namespace != "res" || field != "addCookie" {
        return None;
    }
    extract_cookie_literal(&args[0], bindings)
}

fn extract_header_binding_literal(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, String>,
) -> Option<String> {
    extract_header_gate_literal(expr, "name", bindings)
        .or_else(|| extract_header_gate_literal(expr, "value", bindings))
        .or_else(|| extract_cookie_literal(expr, bindings))
}

fn extract_cookie_literal(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, String>,
) -> Option<String> {
    match &expr.kind {
        sec4_core::ast::ExprKind::String(value) => Some(value.clone()),
        sec4_core::ast::ExprKind::Identifier(name) => bindings.get(name).cloned(),
        sec4_core::ast::ExprKind::Call { callee, args } => {
            let sec4_core::ast::ExprKind::Member { object, field } = &callee.kind else {
                return None;
            };
            let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind else {
                return None;
            };
            if namespace != "cookie" || field != "build" || args.len() < 2 {
                return None;
            }
            let name = extract_string_literal_or_binding(&args[0], bindings)?;
            let value = extract_string_literal_or_binding(&args[1], bindings)?;
            Some(format!("{name}={value}"))
        }
        _ => None,
    }
}

fn extract_string_literal_or_binding(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, String>,
) -> Option<String> {
    match &expr.kind {
        sec4_core::ast::ExprKind::String(value) => Some(value.clone()),
        sec4_core::ast::ExprKind::Identifier(name) => bindings.get(name).cloned(),
        _ => None,
    }
}

fn extract_header_gate_literal(
    expr: &sec4_core::ast::Expr,
    expected_gate: &str,
    bindings: &HashMap<String, String>,
) -> Option<String> {
    match &expr.kind {
        sec4_core::ast::ExprKind::String(value) => Some(value.clone()),
        sec4_core::ast::ExprKind::Identifier(name) => bindings.get(name).cloned(),
        sec4_core::ast::ExprKind::Call { callee, args } => {
            let sec4_core::ast::ExprKind::Member { object, field } = &callee.kind else {
                return None;
            };
            let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind else {
                return None;
            };
            if namespace != "headers" || field != expected_gate || args.is_empty() {
                return None;
            }
            let sec4_core::ast::ExprKind::String(value) = &args[0].kind else {
                return None;
            };
            Some(value.clone())
        }
        _ => None,
    }
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
    let mut db_stubs: Option<HashMap<String, ReplayMockDbStubMatch>> = None;
    let mut fs_stubs: Option<HashMap<String, ReplayMockFsStubMatch>> = None;
    let mut mock_request_signature: Option<String> = None;
    let mut mock_matched_stub: Option<ReplayMockNetStubMatch> = None;
    let mut mock_dependency_matches: Option<ReplayMockDependencyMatches> = None;
    let mut mock_dependency_signatures: Option<ReplayMockDependencySignatures> = None;
    let mut mock_dependency_stub_summaries: Option<ReplayMockDependencyStubSummaries> = None;
    let mut mock_dependency_traces: Option<ReplayMockDependencyTraces> = None;
    let mut mock_execution_counts: Option<ReplayMockExecutionCounts> = None;
    let mut mock_execution_traces: Option<ReplayMockExecutionTraces> = None;

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
                eprintln!("replay compatibility failed: stub registry contract invalid: {message}");
                return Err(1);
            }
        };
        net_stubs = match collect_replay_net_stubs(&stubs_json) {
            Ok(entries) => Some(entries),
            Err(message) => {
                eprintln!("replay compatibility failed: stub registry contract invalid: {message}");
                return Err(1);
            }
        };
        db_stubs = match collect_replay_db_stubs(&stubs_json) {
            Ok(entries) => Some(entries),
            Err(message) => {
                eprintln!("replay compatibility failed: stub registry contract invalid: {message}");
                return Err(1);
            }
        };
        fs_stubs = match collect_replay_fs_stubs(&stubs_json) {
            Ok(entries) => Some(entries),
            Err(message) => {
                eprintln!("replay compatibility failed: stub registry contract invalid: {message}");
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
        let db_signatures = db_stubs
            .as_ref()
            .expect("mock mode requires db stubs for loaded stubs");
        let mut db_summary_entries = Vec::new();
        let mut db_trace_entries = Vec::new();
        for (index, signature) in capture_db_signatures.iter().enumerate() {
            if let Some(stub) = db_signatures.get(signature) {
                db_summary_entries.push(ReplayMockDbDependencyStubSummary {
                    signature: signature.clone(),
                    row_count: stub.row_count,
                    truncated: stub.truncated,
                });
                db_trace_entries.push(ReplayMockDbDependencyTrace {
                    index,
                    trace_id: format!("db:{index}"),
                    signature: signature.clone(),
                    row_count: stub.row_count,
                    truncated: stub.truncated,
                });
            } else {
                eprintln!("replay compatibility failed: REPLAY.DB_STUB_MISSING: {signature}");
                return Err(1);
            }
        }
        let fs_signatures = fs_stubs
            .as_ref()
            .expect("mock mode requires fs stubs for loaded stubs");
        let mut fs_summary_entries = Vec::new();
        let mut fs_trace_entries = Vec::new();
        for (index, signature) in capture_fs_signatures.iter().enumerate() {
            if let Some(stub) = fs_signatures.get(signature) {
                fs_summary_entries.push(ReplayMockFsDependencyStubSummary {
                    signature: signature.clone(),
                    ok: stub.ok,
                    truncated: stub.truncated,
                    bytes: stub.bytes,
                });
                fs_trace_entries.push(ReplayMockFsDependencyTrace {
                    index,
                    trace_id: format!("fs:{index}"),
                    signature: signature.clone(),
                    ok: stub.ok,
                    truncated: stub.truncated,
                    bytes: stub.bytes,
                });
            } else {
                eprintln!("replay compatibility failed: REPLAY.FS_STUB_MISSING: {signature}");
                return Err(1);
            }
        }
        let db_match_count = capture_db_signatures.len();
        let fs_match_count = capture_fs_signatures.len();
        let db_execution_traces = db_trace_entries.clone();
        let fs_execution_traces = fs_trace_entries.clone();
        let net_execution_trace = ReplayMockNetExecutionTrace {
            index: 0,
            trace_id: "net:0".to_string(),
            signature: signature.clone(),
            status: stub_match.status,
            truncated: stub_match.truncated,
            body_kind: stub_match.body_kind,
        };
        mock_request_signature = Some(signature);
        mock_matched_stub = Some(stub_match);
        mock_dependency_signatures = Some(ReplayMockDependencySignatures {
            db: capture_db_signatures,
            fs: capture_fs_signatures,
        });
        mock_dependency_stub_summaries = Some(ReplayMockDependencyStubSummaries {
            db: db_summary_entries,
            fs: fs_summary_entries,
        });
        mock_dependency_traces = Some(ReplayMockDependencyTraces {
            db: db_trace_entries,
            fs: fs_trace_entries,
        });
        mock_dependency_matches = Some(ReplayMockDependencyMatches {
            db: db_match_count,
            fs: fs_match_count,
        });
        mock_execution_counts = Some(ReplayMockExecutionCounts {
            net: 1,
            db: db_match_count,
            fs: fs_match_count,
        });
        mock_execution_traces = Some(ReplayMockExecutionTraces {
            net: vec![net_execution_trace],
            db: db_execution_traces,
            fs: fs_execution_traces,
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
            if let Some(signatures) = &mock_dependency_signatures {
                let db = if signatures.db.is_empty() {
                    "-".to_string()
                } else {
                    signatures.db.join(",")
                };
                let fs = if signatures.fs.is_empty() {
                    "-".to_string()
                } else {
                    signatures.fs.join(",")
                };
                println!("replay mock dependency signatures: db={db} fs={fs}");
            }
            if let Some(summaries) = &mock_dependency_stub_summaries {
                let db = if summaries.db.is_empty() {
                    "-".to_string()
                } else {
                    summaries
                        .db
                        .iter()
                        .map(|entry| {
                            format!(
                                "{}(rowCount={},truncated={})",
                                entry.signature, entry.row_count, entry.truncated
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let fs = if summaries.fs.is_empty() {
                    "-".to_string()
                } else {
                    summaries
                        .fs
                        .iter()
                        .map(|entry| {
                            let bytes = entry
                                .bytes
                                .map_or_else(|| "-".to_string(), |value| value.to_string());
                            format!(
                                "{}(ok={},truncated={},bytes={})",
                                entry.signature, entry.ok, entry.truncated, bytes
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                };
                println!("replay mock dependency stub summaries: db={db} fs={fs}");
            }
            if let Some(traces) = &mock_dependency_traces {
                let db = if traces.db.is_empty() {
                    "-".to_string()
                } else {
                    traces
                        .db
                        .iter()
                        .map(|entry| format!("{}({})", entry.trace_id, entry.signature))
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let fs = if traces.fs.is_empty() {
                    "-".to_string()
                } else {
                    traces
                        .fs
                        .iter()
                        .map(|entry| format!("{}({})", entry.trace_id, entry.signature))
                        .collect::<Vec<_>>()
                        .join(",")
                };
                println!("replay mock dependency traces: db={db} fs={fs}");
            }
            if let Some(counts) = &mock_execution_counts {
                println!(
                    "replay mock executed stubs: net={} db={} fs={}",
                    counts.net, counts.db, counts.fs
                );
            }
            if let Some(traces) = &mock_execution_traces {
                let net = if traces.net.is_empty() {
                    "-".to_string()
                } else {
                    traces
                        .net
                        .iter()
                        .map(|entry| format!("{}({})", entry.trace_id, entry.signature))
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let db = if traces.db.is_empty() {
                    "-".to_string()
                } else {
                    traces
                        .db
                        .iter()
                        .map(|entry| format!("{}({})", entry.trace_id, entry.signature))
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let fs = if traces.fs.is_empty() {
                    "-".to_string()
                } else {
                    traces
                        .fs
                        .iter()
                        .map(|entry| format!("{}({})", entry.trace_id, entry.signature))
                        .collect::<Vec<_>>()
                        .join(",")
                };
                println!("replay mock execution traces: net={net} db={db} fs={fs}");
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
                "mockDependencySignatures": mock_dependency_signatures.as_ref().map(|signatures| serde_json::json!({
                    "db": signatures.db,
                    "fs": signatures.fs,
                })),
                "mockDependencyStubSummaries": mock_dependency_stub_summaries.as_ref().map(|summaries| serde_json::json!({
                    "db": summaries.db.iter().map(|entry| serde_json::json!({
                        "signature": entry.signature,
                        "rowCount": entry.row_count,
                        "truncated": entry.truncated,
                    })).collect::<Vec<_>>(),
                    "fs": summaries.fs.iter().map(|entry| serde_json::json!({
                        "signature": entry.signature,
                        "ok": entry.ok,
                        "truncated": entry.truncated,
                        "bytes": entry.bytes,
                    })).collect::<Vec<_>>(),
                })),
                "mockDependencyTraces": mock_dependency_traces.as_ref().map(|traces| serde_json::json!({
                    "db": traces.db.iter().map(|entry| serde_json::json!({
                        "index": entry.index,
                        "traceId": entry.trace_id,
                        "signature": entry.signature,
                        "rowCount": entry.row_count,
                        "truncated": entry.truncated,
                    })).collect::<Vec<_>>(),
                    "fs": traces.fs.iter().map(|entry| serde_json::json!({
                        "index": entry.index,
                        "traceId": entry.trace_id,
                        "signature": entry.signature,
                        "ok": entry.ok,
                        "truncated": entry.truncated,
                        "bytes": entry.bytes,
                    })).collect::<Vec<_>>(),
                })),
                "mockExecutionCounts": mock_execution_counts.as_ref().map(|counts| serde_json::json!({
                    "net": counts.net,
                    "db": counts.db,
                    "fs": counts.fs,
                })),
                "mockExecutionTraces": mock_execution_traces.as_ref().map(|traces| serde_json::json!({
                    "net": traces.net.iter().map(|entry| serde_json::json!({
                        "index": entry.index,
                        "traceId": entry.trace_id,
                        "signature": entry.signature,
                        "status": entry.status,
                        "truncated": entry.truncated,
                        "bodyKind": entry.body_kind,
                    })).collect::<Vec<_>>(),
                    "db": traces.db.iter().map(|entry| serde_json::json!({
                        "index": entry.index,
                        "traceId": entry.trace_id,
                        "signature": entry.signature,
                        "rowCount": entry.row_count,
                        "truncated": entry.truncated,
                    })).collect::<Vec<_>>(),
                    "fs": traces.fs.iter().map(|entry| serde_json::json!({
                        "index": entry.index,
                        "traceId": entry.trace_id,
                        "signature": entry.signature,
                        "ok": entry.ok,
                        "truncated": entry.truncated,
                        "bytes": entry.bytes,
                    })).collect::<Vec<_>>(),
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
        .filter(|entry| !entry.is_empty())
        .filter(|entry| entry.chars().all(|ch| ch.is_ascii_uppercase()))
        .is_none()
    {
        return Err("capture.request.method must be a non-empty uppercase string".to_string());
    }
    if request
        .get("path")
        .and_then(serde_json::Value::as_str)
        .filter(|entry| !entry.is_empty())
        .is_none()
    {
        return Err("capture.request.path must be a non-empty string".to_string());
    }
    if request
        .get("headers")
        .and_then(serde_json::Value::as_object)
        .is_none()
    {
        return Err("capture.request.headers must be an object".to_string());
    }
    if request.get("url").is_some()
        && request
            .get("url")
            .and_then(serde_json::Value::as_str)
            .filter(|entry| !entry.is_empty())
            .is_none()
    {
        return Err("capture.request.url must be a non-empty string when present".to_string());
    }
    if request.get("scheme").is_some()
        && request
            .get("scheme")
            .and_then(serde_json::Value::as_str)
            .filter(|entry| !entry.is_empty())
            .is_none()
    {
        return Err("capture.request.scheme must be a non-empty string when present".to_string());
    }
    if request.get("host").is_some()
        && request
            .get("host")
            .and_then(serde_json::Value::as_str)
            .filter(|entry| !entry.is_empty())
            .is_none()
    {
        return Err("capture.request.host must be a non-empty string when present".to_string());
    }
    if request.get("route").is_some()
        && request
            .get("route")
            .and_then(serde_json::Value::as_str)
            .filter(|entry| !entry.is_empty())
            .is_none()
    {
        return Err("capture.request.route must be a non-empty string when present".to_string());
    }
    if request.get("query").is_some()
        && request
            .get("query")
            .and_then(serde_json::Value::as_str)
            .is_none()
    {
        return Err("capture.request.query must be a string when present".to_string());
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
            let bytes = body
                .get("bytes")
                .and_then(serde_json::Value::as_str)
                .filter(|entry| !entry.is_empty())
                .ok_or_else(|| {
                    "capture.request.body.bytes must be present for encoding=base64".to_string()
                })?;
            if base64::engine::general_purpose::STANDARD
                .decode(bytes)
                .is_err()
            {
                return Err(
                    "capture.request.body.bytes must be valid base64 for encoding=base64"
                        .to_string(),
                );
            }
            if body
                .get("sha256")
                .and_then(serde_json::Value::as_str)
                .filter(|entry| !entry.is_empty())
                .is_none()
            {
                return Err(
                    "capture.request.body.sha256 must be present for encoding=base64".to_string(),
                );
            }
        }
        "none" => {
            if body.get("bytes").is_some() {
                return Err(
                    "capture.request.body.bytes must be absent for encoding=none".to_string(),
                );
            }
            if body
                .get("sha256")
                .and_then(serde_json::Value::as_str)
                .filter(|entry| !entry.is_empty())
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

    if let Err(message) = capture_net_request_signature(capture) {
        return Err(message);
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
            let db_entries = db_entries.as_array().ok_or_else(|| {
                "capture.dependencies.db must be an array when present".to_string()
            })?;
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
            let fs_entries = fs_entries.as_array().ok_or_else(|| {
                "capture.dependencies.fs must be an array when present".to_string()
            })?;
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
            .ok_or_else(|| format!("stub registry stubs.db[{index}].response must be an object"))?;
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
            .ok_or_else(|| format!("stub registry stubs.fs[{index}].response must be an object"))?;
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
                    format!(
                        "capture.dependencies.fs[{index}].request.op must be a non-empty string"
                    )
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
            .ok_or_else(|| {
                format!("stub registry stubs.net[{index}].response must be an object")
            })?;
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

fn collect_replay_db_stubs(
    stubs: &serde_json::Value,
) -> Result<HashMap<String, ReplayMockDbStubMatch>, String> {
    let entries = stubs
        .get("stubs")
        .and_then(serde_json::Value::as_object)
        .and_then(|stubs_obj| stubs_obj.get("db"))
        .and_then(serde_json::Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let mut stubs = HashMap::new();
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
        let response = entry
            .get("response")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("stub registry stubs.db[{index}].response must be an object"))?;
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
        let truncated = response
            .get("truncated")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| {
                format!("stub registry stubs.db[{index}].response.truncated must be a boolean")
            })?;
        let signature = replay_db_request_signature(query_template_id, params_sha256);
        stubs.insert(
            signature,
            ReplayMockDbStubMatch {
                row_count,
                truncated,
            },
        );
    }
    Ok(stubs)
}

fn collect_replay_fs_stubs(
    stubs: &serde_json::Value,
) -> Result<HashMap<String, ReplayMockFsStubMatch>, String> {
    let entries = stubs
        .get("stubs")
        .and_then(serde_json::Value::as_object)
        .and_then(|stubs_obj| stubs_obj.get("fs"))
        .and_then(serde_json::Value::as_array)
        .map_or(&[][..], Vec::as_slice);
    let mut stubs = HashMap::new();
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
        let response = entry
            .get("response")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("stub registry stubs.fs[{index}].response must be an object"))?;
        let ok = response
            .get("ok")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| {
                format!("stub registry stubs.fs[{index}].response.ok must be a boolean")
            })?;
        let truncated = response
            .get("truncated")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| {
                format!("stub registry stubs.fs[{index}].response.truncated must be a boolean")
            })?;
        let bytes = if let Some(value) = response.get("bytes") {
            let bytes = value.as_i64().ok_or_else(|| {
                format!("stub registry stubs.fs[{index}].response.bytes must be an integer")
            })?;
            if bytes < 0 {
                return Err(format!(
                    "stub registry stubs.fs[{index}].response.bytes must be >= 0"
                ));
            }
            Some(bytes)
        } else {
            None
        };
        let signature = replay_fs_request_signature(op, path_sha256);
        stubs.insert(
            signature,
            ReplayMockFsStubMatch {
                ok,
                truncated,
                bytes,
            },
        );
    }
    Ok(stubs)
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

fn cmd_init(path: &Path, name: Option<&str>) -> Result<(), i32> {
    if path.exists() {
        if !path.is_dir() {
            eprintln!(
                "init failed: target path `{}` is not a directory",
                path.display()
            );
            return Err(1);
        }

        let has_entries = match directory_has_entries(path) {
            Ok(has_entries) => has_entries,
            Err(err) => {
                eprintln!(
                    "init failed: could not read target directory `{}`: {err}",
                    path.display()
                );
                return Err(2);
            }
        };
        if has_entries && path.join("sec4.toml").exists() {
            eprintln!(
                "init failed: target directory `{}` already contains sec4.toml",
                path.display()
            );
            return Err(1);
        }
    } else if let Err(err) = fs::create_dir_all(path) {
        eprintln!(
            "init failed: could not create target directory `{}`: {err}",
            path.display()
        );
        return Err(2);
    }

    let package_name = match name {
        Some(name) => {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                eprintln!("init failed: --name must not be empty");
                return Err(1);
            }
            trimmed.to_string()
        }
        None => default_package_name_for_init(path),
    };

    let src_dir = path.join("src");
    if let Err(err) = fs::create_dir_all(&src_dir) {
        eprintln!(
            "init failed: could not create source directory `{}`: {err}",
            src_dir.display()
        );
        return Err(2);
    }

    let manifest_path = path.join("sec4.toml");
    let policy_path = path.join("sec4.policy");
    let entry_path = src_dir.join("main.ut");

    let manifest_body = format!(
        "[package]\n\
name = \"{package_name}\"\n\
version = \"0.1.0\"\n\
edition = \"2026\"\n\
\n\
[build]\n\
entry = \"src/main.ut\"\n\
profile = \"server\"\n"
    );
    let policy_body = "[policy]\n\
name = \"default-secure\"\n\
version = \"0.1\"\n\
mode = \"enforce\"\n\
env = \"prod\"\n\
\n\
[security_headers]\n\
enabled = true\n\
\n\
[security_headers.csp]\n\
enabled = true\n\
report_only = false\n";
    let entry_body = "fn health() effects { net } -> Int {\n\
  res.text(200, \"hello from sec4\");\n\
  0\n\
}\n\
\n\
fn main() effects { net } -> Int {\n\
  let router = http.router();\n\
  http.get(router, \"/health\", health);\n\
  http.serve(8080, router);\n\
  0\n\
}\n";

    if let Err(err) = fs::write(&manifest_path, manifest_body) {
        eprintln!(
            "init failed: could not write manifest `{}`: {err}",
            manifest_path.display()
        );
        return Err(2);
    }
    if let Err(err) = fs::write(&policy_path, policy_body) {
        eprintln!(
            "init failed: could not write policy `{}`: {err}",
            policy_path.display()
        );
        return Err(2);
    }
    if let Err(err) = fs::write(&entry_path, entry_body) {
        eprintln!(
            "init failed: could not write entry `{}`: {err}",
            entry_path.display()
        );
        return Err(2);
    }

    println!("init succeeded: {}", path.display());
    println!("created files:");
    println!("  {}", manifest_path.display());
    println!("  {}", policy_path.display());
    println!("  {}", entry_path.display());

    Ok(())
}

fn directory_has_entries(path: &Path) -> Result<bool, std::io::Error> {
    let mut entries = fs::read_dir(path)?;
    match entries.next() {
        Some(Ok(_)) => Ok(true),
        Some(Err(err)) => Err(err),
        None => Ok(false),
    }
}

fn default_package_name_for_init(path: &Path) -> String {
    let fallback = "sec4-app";
    let candidate = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(fallback);
    let normalized = normalize_package_name(candidate);
    if normalized.is_empty() {
        fallback.to_string()
    } else {
        normalized
    }
}

fn normalize_package_name(raw: &str) -> String {
    let mut normalized = String::with_capacity(raw.len());
    let mut last_was_separator = false;

    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            last_was_separator = false;
            continue;
        }

        if !last_was_separator {
            normalized.push('-');
            last_was_separator = true;
        }
    }

    normalized.trim_matches('-').to_string()
}

fn cmd_build(
    path: &Path,
    emit: Option<BuildEmitTarget>,
    locked: bool,
    sbom: bool,
    tls_backend: BuildTlsBackend,
) -> Result<(), i32> {
    let structured_emit_mode = matches!(
        emit,
        Some(BuildEmitTarget::MirJson | BuildEmitTarget::LasmJson)
    );
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
            let backend_emit = match emit {
                Some(BuildEmitTarget::C | BuildEmitTarget::CBin) => {
                    Some(emit_program_with_backend(
                        BackendKind::C,
                        mir.as_ref()
                            .expect("MIR should be lowered when emit target is set"),
                    ))
                }
                Some(BuildEmitTarget::Lasm) => Some(emit_program_with_backend(
                    BackendKind::Lasm,
                    mir.as_ref()
                        .expect("MIR should be lowered when emit target is set"),
                )),
                Some(BuildEmitTarget::LasmJson) => Some(emit_program_with_backend(
                    BackendKind::LasmJson,
                    mir.as_ref()
                        .expect("MIR should be lowered when emit target is set"),
                )),
                _ => None,
            };

            if !structured_emit_mode {
                println!(
                    "build succeeded (M3 effects): package={}, entry={}",
                    manifest.package.name,
                    manifest.entry_file()
                );
                if locked {
                    println!("verified lockfile: {}", path.join("sec4.lock").display());
                } else {
                    println!("wrote lockfile: {}", path.join("sec4.lock").display());
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
                        backend_emit
                            .as_ref()
                            .expect("backend output should be available for c emit target")
                            .source
                            .as_str()
                    );
                }
                Some(BuildEmitTarget::CBin) => {
                    let (c_path, bin_path) = compile_c_binary(
                        path,
                        &manifest.package.name,
                        backend_emit
                            .as_ref()
                            .expect("backend output should be available for c-bin emit target"),
                        tls_backend,
                    )?;
                    println!("generated c source: {}", c_path.display());
                    println!("compiled binary: {}", bin_path.display());
                }
                Some(BuildEmitTarget::Lasm) | Some(BuildEmitTarget::LasmJson) => {
                    println!(
                        "{}",
                        backend_emit
                            .as_ref()
                            .expect("backend output should be available for lasm emit target")
                            .source
                            .as_str()
                    );
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
    backend_emit: &BackendEmitOutput,
    tls_backend: BuildTlsBackend,
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
    if let Err(err) = fs::write(&c_path, &backend_emit.source) {
        eprintln!("could not write generated C `{}`: {err}", c_path.display());
        return Err(2);
    }

    let runtime_assets = match backend_emit.runtime_assets {
        Some(runtime_assets) => runtime_assets,
        None => {
            eprintln!("selected backend does not provide C runtime assets");
            return Err(2);
        }
    };

    let (runtime_source_path, runtime_include_dir) = match canonical_runtime_sources() {
        Some(source_path) => {
            let include_dir = source_path
                .parent()
                .expect("runtime source should have parent directory")
                .to_path_buf();
            (source_path, include_dir)
        }
        None => {
            let runtime_header_path = build_dir.join("sec4_runtime.h");
            if let Err(err) = fs::write(&runtime_header_path, runtime_assets.header) {
                eprintln!(
                    "could not write runtime header `{}`: {err}",
                    runtime_header_path.display()
                );
                return Err(2);
            }

            let runtime_source_path = build_dir.join("sec4_runtime.c");
            if let Err(err) = fs::write(&runtime_source_path, runtime_assets.source) {
                eprintln!(
                    "could not write runtime source `{}`: {err}",
                    runtime_source_path.display()
                );
                return Err(2);
            }

            (runtime_source_path, build_dir.to_path_buf())
        }
    };

    let binary_path = build_dir.join(package_name);

    let run_clang = |with_openssl: bool| {
        let mut clang = Command::new("clang");
        clang
            .arg(&c_path)
            .arg(&runtime_source_path)
            .arg("-std=c11")
            .arg("-O2")
            .arg("-Wno-int-conversion")
            .arg("-I")
            .arg(&runtime_include_dir);
        if with_openssl {
            clang.arg("-DSEC4_RT_ENABLE_OPENSSL_TLS");
        }
        clang.arg("-o").arg(&binary_path);
        if with_openssl {
            clang.arg("-lssl").arg("-lcrypto");
        }
        clang.output()
    };

    let emit_failure = |output: &std::process::Output, openssl_requested: bool| {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if openssl_requested {
            eprintln!(
                "TLS backend linkage failed for `--tls-backend openssl`: clang could not compile/link OpenSSL runtime (`-DSEC4_RT_ENABLE_OPENSSL_TLS -lssl -lcrypto`)"
            );
        }
        eprintln!("clang failed while compiling `{}`", c_path.display());
        if !stdout.trim().is_empty() {
            eprintln!("{stdout}");
        }
        if !stderr.trim().is_empty() {
            eprintln!("{stderr}");
        }
    };

    match tls_backend {
        BuildTlsBackend::None => {
            let output = match run_clang(false) {
                Ok(output) => output,
                Err(err) => {
                    eprintln!("could not execute clang: {err}");
                    return Err(2);
                }
            };
            if !output.status.success() {
                emit_failure(&output, false);
                return Err(1);
            }
        }
        BuildTlsBackend::Openssl => {
            let output = match run_clang(true) {
                Ok(output) => output,
                Err(err) => {
                    eprintln!("could not execute clang: {err}");
                    return Err(2);
                }
            };
            if !output.status.success() {
                emit_failure(&output, true);
                return Err(1);
            }
        }
        BuildTlsBackend::Auto => {
            let output = match run_clang(true) {
                Ok(output) => output,
                Err(err) => {
                    eprintln!("could not execute clang: {err}");
                    return Err(2);
                }
            };
            if output.status.success() {
                return Ok((c_path, binary_path));
            }

            eprintln!(
                "warning: OpenSSL TLS backend unavailable in auto mode; falling back to `--tls-backend none`"
            );
            let fallback = match run_clang(false) {
                Ok(output) => output,
                Err(err) => {
                    eprintln!("could not execute clang: {err}");
                    return Err(2);
                }
            };
            if !fallback.status.success() {
                emit_failure(&fallback, false);
                return Err(1);
            }
        }
    }

    Ok((c_path, binary_path))
}

fn canonical_runtime_sources() -> Option<PathBuf> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())?
        .to_path_buf();
    let runtime_dir = repo_root.join("runtime").join("c");
    let header = runtime_dir.join("sec4_runtime.h");
    let source = runtime_dir.join("sec4_runtime.c");
    if header.exists() && source.exists() {
        Some(source)
    } else {
        None
    }
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

fn cmd_run(
    path: &Path,
    port: Option<u16>,
    oneshot: bool,
    max_body_bytes: Option<u64>,
    max_concurrency: Option<u64>,
    max_pending: Option<u64>,
    serve_timeout_ms: Option<u64>,
    backend: RunBackend,
    tls_backend: BuildTlsBackend,
) -> Result<(), i32> {
    if max_body_bytes == Some(0) {
        eprintln!("run failed: --max-body-bytes must be >= 1");
        return Err(2);
    }
    if max_concurrency == Some(0) {
        eprintln!("run failed: --max-concurrency must be >= 1");
        return Err(2);
    }
    if max_pending == Some(0) {
        eprintln!("run failed: --max-pending must be >= 1");
        return Err(2);
    }
    if serve_timeout_ms == Some(0) {
        eprintln!("run failed: --serve-timeout-ms must be >= 1");
        return Err(2);
    }

    let manifest = match sec4_core::validate_project(path) {
        Ok(manifest) => manifest,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            return Err(1);
        }
    };
    let policy = match sec4_core::policy::load_policy(path) {
        Ok(policy) => policy,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            return Err(1);
        }
    };

    if backend == RunBackend::Lasm {
        return cmd_run_lasm_backend(
            path,
            &manifest,
            &policy,
            port,
            oneshot,
            max_body_bytes,
            max_concurrency,
            max_pending,
            serve_timeout_ms,
        );
    }

    cmd_build(path, Some(BuildEmitTarget::CBin), false, false, tls_backend)?;

    let binary_path = path.join("build").join(&manifest.package.name);
    let mut cmd = Command::new(&binary_path);
    cmd.env(
        "SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES",
        policy.net_public.allowed_schemes.join(","),
    );
    cmd.env(
        "SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS",
        if policy.net_public.allow_redirects {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_NET_PUBLIC_MAX_REDIRECTS",
        policy.net_public.max_redirects.to_string(),
    );
    cmd.env(
        "SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS",
        policy.net_public.allowed_domains.join(","),
    );
    cmd.env(
        "SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS",
        policy.net_public.blocked_domains.join(","),
    );
    cmd.env(
        "SEC4_RT_NET_PUBLIC_ALLOWED_PORTS",
        policy
            .net_public
            .allowed_ports
            .iter()
            .map(|port| port.to_string())
            .collect::<Vec<_>>()
            .join(","),
    );
    cmd.env(
        "SEC4_RT_ALLOW_INTERNAL_NET",
        if policy.net_internal.enabled {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_NET_INTERNAL_ALLOWED_DOMAINS",
        policy.net_internal.allowed_domains.join(","),
    );
    cmd.env(
        "SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS",
        policy.net_internal.allowed_cidrs.join(","),
    );
    cmd.env(
        "SEC4_RT_NET_SSRF_BLOCK_PRIVATE_RANGES",
        if policy.net_ssrf.block_private_ranges {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_NET_SSRF_BLOCK_LOOPBACK",
        if policy.net_ssrf.block_loopback {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_NET_SSRF_BLOCK_LINK_LOCAL",
        if policy.net_ssrf.block_link_local {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_NET_SSRF_BLOCK_METADATA_IPS",
        if policy.net_ssrf.block_metadata_ips {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_NET_SSRF_REVALIDATE_REDIRECTS",
        if policy.net_ssrf.revalidate_redirects {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_NET_SSRF_RESOLVE_DNS",
        if policy.net_ssrf.resolve_dns {
            "1"
        } else {
            "0"
        },
    );
    cmd.env("SEC4_RT_JSON_MAX_BYTES", policy.json.max_bytes.to_string());
    cmd.env("SEC4_RT_JSON_MAX_DEPTH", policy.json.max_depth.to_string());
    cmd.env(
        "SEC4_RT_HTTP_MAX_BODY_BYTES",
        policy.http.max_body_bytes.to_string(),
    );
    cmd.env(
        "SEC4_RT_HTTP_MAX_CONCURRENCY",
        policy.http.max_concurrency.to_string(),
    );
    cmd.env(
        "SEC4_RT_HTTP_MAX_HEADER_BYTES",
        policy.http.max_header_bytes.to_string(),
    );
    cmd.env(
        "SEC4_RT_HTTP_MAX_MULTIPART_BYTES",
        policy.http.max_multipart_bytes.to_string(),
    );
    cmd.env(
        "SEC4_RT_HTTP_SERVE_TIMEOUT_MS",
        policy.http.default_timeout_ms.to_string(),
    );
    cmd.env(
        "SEC4_RT_CORS_ENABLED",
        if policy.cors.enabled { "1" } else { "0" },
    );
    cmd.env(
        "SEC4_RT_CORS_ALLOWED_ORIGINS",
        policy.cors.allowed_origins.join(","),
    );
    cmd.env(
        "SEC4_RT_CORS_ALLOWED_METHODS",
        policy.cors.allowed_methods.join(","),
    );
    cmd.env(
        "SEC4_RT_CORS_ALLOWED_HEADERS",
        policy.cors.allowed_headers.join(","),
    );
    cmd.env(
        "SEC4_RT_CORS_EXPOSED_HEADERS",
        policy.cors.exposed_headers.join(","),
    );
    cmd.env(
        "SEC4_RT_CORS_MAX_AGE_SECONDS",
        policy.cors.max_age_seconds.to_string(),
    );
    cmd.env(
        "SEC4_RT_CORS_ALLOW_CREDENTIALS",
        if policy.cors.allow_credentials {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_CORS_ALLOW_PRIVATE_NETWORK",
        if policy.cors.allow_private_network {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_CORS_REQUIRE_VARY_ORIGIN",
        if policy.cors.require_vary_origin {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_ENABLED",
        if policy.security_headers.enabled {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_HSTS_ENABLED",
        if policy.security_headers.hsts_enabled {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_HSTS_MAX_AGE_SECONDS",
        policy.security_headers.hsts_max_age_seconds.to_string(),
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_HSTS_INCLUDE_SUBDOMAINS",
        if policy.security_headers.hsts_include_subdomains {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_HSTS_PRELOAD",
        if policy.security_headers.hsts_preload {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_X_CONTENT_TYPE_OPTIONS",
        if policy.security_headers.x_content_type_options {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_X_FRAME_OPTIONS",
        policy.security_headers.x_frame_options.as_str(),
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_REFERRER_POLICY",
        policy.security_headers.referrer_policy.as_str(),
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_CSP_ENABLED",
        if policy.security_headers.csp_enabled {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_CSP_REPORT_ONLY",
        if policy.security_headers.csp_report_only {
            "1"
        } else {
            "0"
        },
    );
    cmd.env(
        "SEC4_RT_SECURITY_HEADERS_CSP_POLICY",
        policy.security_headers.csp_policy.as_str(),
    );
    cmd.env(
        "SEC4_RT_CSRF_ENABLED",
        if policy.csrf.enabled { "1" } else { "0" },
    );
    cmd.env("SEC4_RT_CSRF_MODE", policy.csrf.mode.as_str());
    cmd.env("SEC4_RT_CSRF_COOKIE_NAME", policy.csrf.cookie_name.as_str());
    cmd.env("SEC4_RT_CSRF_HEADER_NAME", policy.csrf.header_name.as_str());
    cmd.env(
        "SEC4_RT_CSRF_PROTECTED_METHODS",
        policy.csrf.protected_methods.join(","),
    );
    cmd.env("SEC4_RT_AUTH_MODE", policy.auth.mode.as_str());
    cmd.env("SEC4_RT_AUTH_COOKIE_NAME", policy.auth.cookie_name.as_str());
    if let Some(port) = port {
        cmd.env("SEC4_RT_HTTP_PORT", port.to_string());
    }
    if oneshot {
        cmd.env("SEC4_RT_HTTP_SERVE_MODE", "oneshot");
    }
    if let Some(bytes) = max_body_bytes {
        cmd.env("SEC4_RT_HTTP_MAX_BODY_BYTES", bytes.to_string());
    }
    if let Some(limit) = max_concurrency {
        cmd.env("SEC4_RT_HTTP_MAX_CONCURRENCY", limit.to_string());
    }
    if let Some(timeout_ms) = serve_timeout_ms {
        cmd.env("SEC4_RT_HTTP_SERVE_TIMEOUT_MS", timeout_ms.to_string());
    }

    let status = match cmd.status() {
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
        if let Some(code) = status.code() {
            eprintln!(
                "run failed: binary `{}` exited with status {code}",
                binary_path.display()
            );
            Err(code)
        } else {
            eprintln!(
                "run failed: binary `{}` terminated by signal",
                binary_path.display()
            );
            Err(1)
        }
    }
}

fn cmd_run_lasm_backend(
    path: &Path,
    manifest: &sec4_core::Manifest,
    policy: &Policy,
    port: Option<u16>,
    oneshot: bool,
    max_body_bytes: Option<u64>,
    max_concurrency: Option<u64>,
    max_pending: Option<u64>,
    serve_timeout_ms: Option<u64>,
) -> Result<(), i32> {
    let program = match analyze_entry(path, manifest) {
        Ok(program) => program,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            return Err(1);
        }
    };
    let mir = sec4_core::lower_program_to_mir(&program);
    let lasm_program = sec4_core::lower_mir_to_lasm(&mir);
    let Some(entry) = lasm_program.entry.as_ref() else {
        eprintln!("run failed: compiled program does not expose an entrypoint");
        return Err(1);
    };

    let policy_max_in_flight = match u64::try_from(policy.http.max_concurrency) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: policy http.max_concurrency must be >= 0");
            return Err(2);
        }
    };
    let effective_max_in_flight = max_concurrency.unwrap_or(policy_max_in_flight);
    let effective_max_in_flight = match usize::try_from(effective_max_in_flight) {
        Ok(value) if value >= 1 => value,
        Ok(_) => {
            eprintln!("run failed: effective max concurrency must be >= 1");
            return Err(2);
        }
        Err(_) => {
            eprintln!("run failed: effective max concurrency exceeds platform limits");
            return Err(2);
        }
    };
    let effective_max_pending = max_pending.unwrap_or(effective_max_in_flight as u64);
    let effective_max_pending = match usize::try_from(effective_max_pending) {
        Ok(value) if value >= 1 => value,
        Ok(_) => {
            eprintln!("run failed: effective max pending must be >= 1");
            return Err(2);
        }
        Err(_) => {
            eprintln!("run failed: effective max pending exceeds platform limits");
            return Err(2);
        }
    };
    let policy_timeout_ms = match u64::try_from(policy.http.default_timeout_ms) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: policy http.default_timeout_ms must be >= 0");
            return Err(2);
        }
    };
    let effective_timeout_ms = serve_timeout_ms.unwrap_or(policy_timeout_ms);
    let policy_max_header_bytes = match u64::try_from(policy.http.max_header_bytes) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: policy http.max_header_bytes must be >= 0");
            return Err(2);
        }
    };
    let effective_max_header_bytes = match usize::try_from(policy_max_header_bytes) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: policy http.max_header_bytes exceeds platform limits");
            return Err(2);
        }
    };
    let policy_max_body_bytes = match u64::try_from(policy.http.max_body_bytes) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: policy http.max_body_bytes must be >= 0");
            return Err(2);
        }
    };
    let effective_max_body_bytes = max_body_bytes.unwrap_or(policy_max_body_bytes);
    let effective_max_body_bytes = match usize::try_from(effective_max_body_bytes) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: effective max body bytes exceeds platform limits");
            return Err(2);
        }
    };

    let routes = collect_lasm_route_plans(&program, entry.name.as_str());
    if routes.is_empty() {
        eprintln!(
            "run failed: no HTTP routes discovered from entry `{}` for LASM backend",
            entry.name
        );
        return Err(1);
    }
    let listen_port = port.unwrap_or(8080);
    let listener = match TcpListener::bind(("127.0.0.1", listen_port)) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!(
                "run failed: could not bind LASM backend listener on 127.0.0.1:{listen_port}: {err}"
            );
            return Err(2);
        }
    };

    let mut worker_handles = Vec::new();
    let trace_counter = Arc::new(AtomicU64::new(0));
    let header_defaults = Arc::new(build_lasm_response_header_defaults(policy));
    let mut oneshot_runtime = if oneshot {
        Some(
            build_lasm_http_runtime(&routes, effective_timeout_ms).map_err(|message| {
                eprintln!("run failed: {message}");
                2
            })?,
        )
    } else {
        None
    };
    let worker_sender = if oneshot {
        None
    } else {
        let (sender, receiver) = sync_channel::<TcpStream>(effective_max_pending);
        let shared_receiver = Arc::new(Mutex::new(receiver));
        for _ in 0..effective_max_in_flight {
            let worker_receiver = Arc::clone(&shared_receiver);
            let routes_for_worker = routes.clone();
            let trace_counter_for_worker = Arc::clone(&trace_counter);
            let header_defaults_for_worker = Arc::clone(&header_defaults);
            worker_handles.push(std::thread::spawn(move || {
                let mut runtime =
                    match build_lasm_http_runtime(&routes_for_worker, effective_timeout_ms) {
                        Ok(runtime) => runtime,
                        Err(message) => {
                            eprintln!("warning: LASM worker bootstrap failed: {message}");
                            return;
                        }
                    };
                loop {
                    let next_stream = {
                        let receiver = match worker_receiver.lock() {
                            Ok(receiver) => receiver,
                            Err(_) => return,
                        };
                        receiver.recv()
                    };
                    let mut stream = match next_stream {
                        Ok(stream) => stream,
                        Err(_) => break,
                    };
                    if let Err(message) = process_lasm_connection_with_runtime(
                        &mut stream,
                        &mut runtime,
                        effective_max_header_bytes,
                        effective_max_body_bytes,
                        trace_counter_for_worker.as_ref(),
                        header_defaults_for_worker.as_ref(),
                    ) {
                        eprintln!("warning: LASM backend worker failed: {message}");
                    }
                }
            }));
        }
        Some(sender)
    };

    for incoming in listener.incoming() {
        let mut stream = match incoming {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("run failed: LASM backend accept error: {err}");
                return Err(2);
            }
        };
        if let Err(err) =
            stream.set_read_timeout(Some(std::time::Duration::from_millis(effective_timeout_ms)))
        {
            eprintln!("run failed: LASM backend could not set read timeout: {err}");
            return Err(2);
        }

        if oneshot {
            if let Err(message) = process_lasm_connection_with_runtime(
                &mut stream,
                oneshot_runtime
                    .as_mut()
                    .expect("oneshot runtime should be initialized"),
                effective_max_header_bytes,
                effective_max_body_bytes,
                trace_counter.as_ref(),
                header_defaults.as_ref(),
            ) {
                eprintln!("warning: LASM backend worker failed: {message}");
            };
            break;
        }

        let sender = worker_sender
            .as_ref()
            .expect("worker sender should exist for non-oneshot LASM backend");
        match sender.try_send(stream) {
            Ok(()) => {}
            Err(TrySendError::Full(mut stream)) => {
                let request_head = read_lasm_request_head(&mut stream, effective_max_header_bytes).ok();
                let request_headers = request_head.as_ref().map(|head| &head.headers);
                let include_cors_defaults =
                    should_include_lasm_cors_defaults(request_headers, header_defaults.as_ref());
                let mut response =
                    sec4_core::HttpResponse::text(503, "server busy: max concurrency reached");
                apply_lasm_request_origin_header(
                    &mut response,
                    request_headers,
                    header_defaults.as_ref(),
                );
                stamp_lasm_trace_id(&mut response, trace_counter.as_ref());
                let _ = write_lasm_http_response(
                    &mut stream,
                    &response,
                    header_defaults.as_ref(),
                    include_cors_defaults,
                );
            }
            Err(TrySendError::Disconnected(_stream)) => {
                eprintln!("run failed: LASM worker pool disconnected unexpectedly");
                return Err(2);
            }
        }
    }

    drop(worker_sender);
    for handle in worker_handles {
        let _ = handle.join();
    }

    Ok(())
}

fn build_lasm_http_runtime(
    routes: &[LasmRunRoutePlan],
    timeout_ms: u64,
) -> Result<sec4_core::LasmHttpRuntime, String> {
    let mut runtime = sec4_core::LasmHttpRuntime::default();
    runtime
        .set_max_in_flight(1)
        .map_err(|message| format!("could not configure LASM runtime max in-flight: {message}"))?;
    runtime
        .set_max_request_duration_ms(timeout_ms)
        .map_err(|message| {
            format!("could not configure LASM runtime request timeout: {message}")
        })?;

    for route in routes {
        let mut response = sec4_core::HttpResponse::text(route.status, route.body.clone());
        response.headers = route.headers.clone();
        runtime
            .register_route(
                route.method.as_str(),
                route.path.as_str(),
                vec![
                    sec4_core::RuntimeAction::Yield,
                    sec4_core::RuntimeAction::Complete(0),
                ],
                response,
            )
            .map_err(|message| format!("could not register LASM route: {message}"))?;
    }

    Ok(runtime)
}

fn build_lasm_response_header_defaults(policy: &Policy) -> LasmResponseHeaderDefaults {
    let mut headers = BTreeMap::new();
    let cors_enabled = policy.cors.enabled;
    let mut cors_allow_any_origin = false;
    let mut cors_allowed_origins = Vec::new();
    let mut cors_allowed_methods = Vec::new();
    let mut cors_allowed_headers = Vec::new();
    let mut cors_default_origin = None;
    let cors_allow_private_network = policy.cors.allow_private_network;
    if cors_enabled {
        cors_allow_any_origin = policy.cors.has_wildcard_origin();
        if !cors_allow_any_origin {
            cors_allowed_origins = policy.cors.allowed_origins.clone();
        }
        cors_allowed_methods = policy
            .cors
            .allowed_methods
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_uppercase())
            .collect();
        cors_allowed_headers = policy
            .cors
            .allowed_headers
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_lowercase())
            .collect();
        cors_default_origin = Some(if cors_allow_any_origin {
            "*".to_string()
        } else {
            policy
                .cors
                .allowed_origins
                .first()
                .cloned()
                .unwrap_or_else(|| "*".to_string())
        });
        if policy.cors.allow_credentials {
            headers.insert(
                "Access-Control-Allow-Credentials".to_string(),
                "true".to_string(),
            );
        }
        if !policy.cors.allowed_methods.is_empty() {
            headers.insert(
                "Access-Control-Allow-Methods".to_string(),
                policy.cors.allowed_methods.join(","),
            );
        }
        if !policy.cors.allowed_headers.is_empty() {
            headers.insert(
                "Access-Control-Allow-Headers".to_string(),
                policy.cors.allowed_headers.join(","),
            );
        }
        if policy.cors.max_age_seconds >= 0 {
            headers.insert(
                "Access-Control-Max-Age".to_string(),
                policy.cors.max_age_seconds.to_string(),
            );
        }
        if policy.cors.allow_private_network {
            headers.insert(
                "Access-Control-Allow-Private-Network".to_string(),
                "true".to_string(),
            );
        }
        if policy.cors.require_vary_origin {
            headers.insert("Vary".to_string(), "Origin".to_string());
        }
        let exposed_headers = if policy.cors.exposed_headers.is_empty() {
            "x-trace-id,x-showcase".to_string()
        } else {
            policy.cors.exposed_headers.join(",")
        };
        headers.insert("Access-Control-Expose-Headers".to_string(), exposed_headers);
    }
    if policy.security_headers.enabled {
        if policy.security_headers.hsts_enabled {
            let mut hsts = format!("max-age={}", policy.security_headers.hsts_max_age_seconds);
            if policy.security_headers.hsts_include_subdomains {
                hsts.push_str("; includeSubDomains");
            }
            if policy.security_headers.hsts_preload {
                hsts.push_str("; preload");
            }
            headers.insert("Strict-Transport-Security".to_string(), hsts);
        }
        if policy.security_headers.x_content_type_options {
            headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        }
        if !policy.security_headers.x_frame_options.trim().is_empty() {
            headers.insert(
                "X-Frame-Options".to_string(),
                policy.security_headers.x_frame_options.clone(),
            );
        }
        if !policy.security_headers.referrer_policy.trim().is_empty() {
            headers.insert(
                "Referrer-Policy".to_string(),
                policy.security_headers.referrer_policy.clone(),
            );
        }
        if policy.security_headers.csp_enabled && !policy.security_headers.csp_policy.trim().is_empty()
        {
            let header_name = if policy.security_headers.csp_report_only {
                "Content-Security-Policy-Report-Only"
            } else {
                "Content-Security-Policy"
            };
            headers.insert(
                header_name.to_string(),
                policy.security_headers.csp_policy.clone(),
            );
        }
    }
    LasmResponseHeaderDefaults {
        cors_enabled,
        cors_allow_any_origin,
        cors_allowed_origins,
        cors_allowed_methods,
        cors_allowed_headers,
        cors_allow_private_network,
        cors_default_origin,
        headers,
    }
}

fn process_lasm_connection_with_runtime(
    stream: &mut TcpStream,
    runtime: &mut sec4_core::LasmHttpRuntime,
    max_header_bytes: usize,
    max_body_bytes: usize,
    trace_counter: &AtomicU64,
    header_defaults: &LasmResponseHeaderDefaults,
) -> Result<(), String> {
    let request = match read_lasm_http_request(stream, max_header_bytes, max_body_bytes) {
        Ok(request) => request,
        Err(err) => {
            let mut response = sec4_core::HttpResponse::text(err.status, err.message);
            apply_lasm_request_origin_header(&mut response, None, header_defaults);
            stamp_lasm_trace_id(&mut response, trace_counter);
            write_lasm_http_response(stream, &response, header_defaults, true)?;
            return Ok(());
        }
    };
    match evaluate_lasm_cors_preflight_request(&request, header_defaults) {
        LasmCorsPreflightDecision::NotPreflight => {}
        LasmCorsPreflightDecision::Accept => {
            let mut response = sec4_core::HttpResponse::text(204, "");
            response.body.clear();
            apply_lasm_request_origin_header(&mut response, Some(&request.headers), header_defaults);
            stamp_lasm_trace_id(&mut response, trace_counter);
            write_lasm_http_response(stream, &response, header_defaults, true)?;
            return Ok(());
        }
        LasmCorsPreflightDecision::Reject { status, message } => {
            let mut response = sec4_core::HttpResponse::text(status, message);
            stamp_lasm_trace_id(&mut response, trace_counter);
            write_lasm_http_response(stream, &response, header_defaults, false)?;
            return Ok(());
        }
    }
    let include_cors_defaults = should_include_lasm_cors_defaults(Some(&request.headers), header_defaults);

    let request_method = request.method.clone();
    let mut runtime_request = sec4_core::HttpRequest::new(request.method.clone(), request.path.clone());
    runtime_request.headers = request.headers.clone();
    runtime_request.body = request.body.clone();
    let request_id = runtime.submit(runtime_request);
    let report = runtime.run_until_idle(65_536);
    if !report.idle {
        let mut response = sec4_core::HttpResponse::text(
            500,
            "run failed: LASM runtime remained active after step budget",
        );
        apply_lasm_request_origin_header(&mut response, Some(&request.headers), header_defaults);
        stamp_lasm_trace_id(&mut response, trace_counter);
        write_lasm_http_response(stream, &response, header_defaults, include_cors_defaults)?;
        return Ok(());
    }

    let mut matched = None;
    while let Some(exchange) = runtime.pop_response() {
        if exchange.request_id == request_id {
            matched = Some(exchange.response);
            break;
        }
    }
    let mut response = matched
        .unwrap_or_else(|| sec4_core::HttpResponse::text(500, "missing LASM response for request"));
    if request_method.eq_ignore_ascii_case("HEAD") {
        response.body.clear();
    }
    apply_lasm_request_origin_header(&mut response, Some(&request.headers), header_defaults);
    stamp_lasm_trace_id(&mut response, trace_counter);
    write_lasm_http_response(stream, &response, header_defaults, include_cors_defaults)
}

fn evaluate_lasm_cors_preflight_request(
    request: &LasmRunRequest,
    header_defaults: &LasmResponseHeaderDefaults,
) -> LasmCorsPreflightDecision {
    if !header_defaults.cors_enabled || !request.method.eq_ignore_ascii_case("OPTIONS") {
        return LasmCorsPreflightDecision::NotPreflight;
    }
    let requested_method = find_lasm_header_value(&request.headers, "Access-Control-Request-Method")
        .map(str::trim)
        .unwrap_or("");
    let requested_headers = find_lasm_header_value(&request.headers, "Access-Control-Request-Headers")
        .map(str::trim)
        .unwrap_or("");
    let private_network_requested =
        find_lasm_header_value(&request.headers, "Access-Control-Request-Private-Network")
            .map(str::trim)
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    let origin = find_lasm_header_value(&request.headers, "Origin")
        .map(str::trim)
        .unwrap_or("");
    if requested_method.is_empty() && origin.is_empty() {
        return LasmCorsPreflightDecision::NotPreflight;
    }
    if origin.is_empty() {
        return LasmCorsPreflightDecision::Reject {
            status: 400,
            message: "cors preflight missing origin",
        };
    }
    if !is_lasm_cors_origin_value_valid(origin) {
        return LasmCorsPreflightDecision::Reject {
            status: 400,
            message: "cors preflight origin invalid",
        };
    }
    if !is_lasm_cors_origin_allowed(origin, header_defaults) {
        return LasmCorsPreflightDecision::Reject {
            status: 403,
            message: "cors preflight origin not allowed",
        };
    }
    if requested_method.is_empty() {
        return LasmCorsPreflightDecision::Reject {
            status: 400,
            message: "cors preflight missing requested method",
        };
    }
    if !is_lasm_cors_requested_method_allowed(requested_method, header_defaults) {
        return LasmCorsPreflightDecision::Reject {
            status: 403,
            message: "cors preflight method not allowed",
        };
    }
    if !are_lasm_cors_requested_headers_allowed(requested_headers, header_defaults) {
        return LasmCorsPreflightDecision::Reject {
            status: 403,
            message: "cors preflight headers not allowed",
        };
    }
    if private_network_requested && !header_defaults.cors_allow_private_network {
        return LasmCorsPreflightDecision::Reject {
            status: 403,
            message: "cors preflight private network not allowed",
        };
    }
    LasmCorsPreflightDecision::Accept
}

fn is_lasm_cors_origin_value_valid(origin: &str) -> bool {
    if origin.is_empty() || origin == "*" {
        return false;
    }
    if origin.chars().any(char::is_whitespace) {
        return false;
    }
    origin.starts_with("http://") || origin.starts_with("https://")
}

fn is_lasm_cors_origin_allowed(origin: &str, header_defaults: &LasmResponseHeaderDefaults) -> bool {
    if header_defaults.cors_allow_any_origin || header_defaults.cors_allowed_origins.is_empty() {
        return true;
    }
    header_defaults
        .cors_allowed_origins
        .iter()
        .any(|allowed| allowed == origin)
}

fn is_lasm_cors_requested_method_allowed(
    requested_method: &str,
    header_defaults: &LasmResponseHeaderDefaults,
) -> bool {
    if header_defaults.cors_allowed_methods.is_empty() {
        return true;
    }
    let normalized = requested_method.trim().to_ascii_uppercase();
    header_defaults
        .cors_allowed_methods
        .iter()
        .any(|allowed| allowed == &normalized)
}

fn are_lasm_cors_requested_headers_allowed(
    requested_headers: &str,
    header_defaults: &LasmResponseHeaderDefaults,
) -> bool {
    if requested_headers.is_empty() || header_defaults.cors_allowed_headers.is_empty() {
        return true;
    }
    requested_headers
        .split(',')
        .map(str::trim)
        .filter(|header| !header.is_empty())
        .all(|header| {
            let normalized = header.to_ascii_lowercase();
            header_defaults
                .cors_allowed_headers
                .iter()
                .any(|allowed| allowed == &normalized)
        })
}

fn should_include_lasm_cors_defaults(
    request_headers: Option<&BTreeMap<String, String>>,
    header_defaults: &LasmResponseHeaderDefaults,
) -> bool {
    if !header_defaults.cors_enabled {
        return false;
    }
    let Some(request_headers) = request_headers else {
        return true;
    };
    let Some(origin) = find_lasm_header_value(request_headers, "Origin").map(str::trim) else {
        return true;
    };
    if origin.is_empty() {
        return true;
    }
    is_lasm_cors_origin_allowed(origin, header_defaults)
}

fn find_lasm_header_value<'a>(headers: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find_map(|(key, value)| key.eq_ignore_ascii_case(name).then_some(value.as_str()))
}

fn resolve_lasm_allow_origin(
    request_headers: Option<&BTreeMap<String, String>>,
    header_defaults: &LasmResponseHeaderDefaults,
) -> Option<String> {
    if !header_defaults.cors_enabled {
        return None;
    }
    if header_defaults.cors_allow_any_origin {
        return Some("*".to_string());
    }
    if let Some(request_headers) = request_headers {
        if let Some(origin) = find_lasm_header_value(request_headers, "Origin") {
            if header_defaults.cors_allowed_origins.is_empty() {
                return Some(origin.to_string());
            }
            if header_defaults
                .cors_allowed_origins
                .iter()
                .any(|allowed| allowed == origin)
            {
                return Some(origin.to_string());
            }
            return None;
        }
    }
    header_defaults.cors_default_origin.clone()
}

fn apply_lasm_request_origin_header(
    response: &mut sec4_core::HttpResponse,
    request_headers: Option<&BTreeMap<String, String>>,
    header_defaults: &LasmResponseHeaderDefaults,
) {
    if let Some(allow_origin) = resolve_lasm_allow_origin(request_headers, header_defaults) {
        response
            .headers
            .insert("Access-Control-Allow-Origin".to_string(), allow_origin);
    }
}

fn read_lasm_request_head(
    stream: &mut TcpStream,
    max_header_bytes: usize,
) -> Result<LasmRequestHead, String> {
    let mut reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|err| format!("could not clone stream while reading request head: {err}"))?,
    );
    let mut consumed = 0usize;
    let mut line = String::new();
    let mut is_first_line = true;
    let mut headers = BTreeMap::new();
    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|err| format!("could not read request while reading overload head: {err}"))?;
        if read == 0 {
            break;
        }
        consumed = consumed.saturating_add(read);
        if consumed > max_header_bytes {
            break;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if is_first_line {
            is_first_line = false;
            continue;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_string(), value.trim().to_string());
        }
    }
    Ok(LasmRequestHead { headers })
}

#[derive(Debug, Clone)]
struct LasmRunRequest {
    method: String,
    path: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

#[derive(Debug, Clone)]
struct LasmRequestHead {
    headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct LasmRequestReadError {
    status: u16,
    message: String,
}

#[derive(Debug, Clone)]
struct LasmResponseHeaderDefaults {
    cors_enabled: bool,
    cors_allow_any_origin: bool,
    cors_allowed_origins: Vec<String>,
    cors_allowed_methods: Vec<String>,
    cors_allowed_headers: Vec<String>,
    cors_allow_private_network: bool,
    cors_default_origin: Option<String>,
    headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LasmCorsPreflightDecision {
    NotPreflight,
    Accept,
    Reject { status: u16, message: &'static str },
}

fn read_lasm_http_request(
    stream: &mut TcpStream,
    max_header_bytes: usize,
    max_body_bytes: usize,
) -> Result<LasmRunRequest, LasmRequestReadError> {
    let make_error = |status: u16, message: String| LasmRequestReadError { status, message };
    let mut reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|err| make_error(400, format!("could not clone stream: {err}")))?,
    );
    let mut request_line = String::new();
    let bytes = reader
        .read_line(&mut request_line)
        .map_err(|err| make_error(400, format!("could not read request line: {err}")))?;
    if bytes == 0 {
        return Err(make_error(400, "empty request".to_string()));
    }
    let mut consumed = bytes;
    let mut parts = request_line.trim_end().split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| make_error(400, "invalid request line: missing method".to_string()))?;
    let request_target = parts
        .next()
        .ok_or_else(|| make_error(400, "invalid request line: missing path".to_string()))?;

    let mut headers = BTreeMap::new();
    let mut content_length = 0usize;

    let mut header_line = String::new();
    loop {
        header_line.clear();
        let read = reader
            .read_line(&mut header_line)
            .map_err(|err| make_error(400, format!("could not read header line: {err}")))?;
        if read == 0 {
            break;
        }
        consumed = consumed.saturating_add(read);
        if consumed > max_header_bytes {
            return Err(make_error(
                400,
                "request headers exceed configured limit ({max_header_bytes} bytes)".to_string(),
            ));
        }
        if header_line == "\r\n" || header_line == "\n" {
            break;
        }
        let header = header_line.trim_end_matches(['\r', '\n']);
        let Some((name_raw, value_raw)) = header.split_once(':') else {
            return Err(make_error(
                400,
                "invalid header line: missing ':' separator".to_string(),
            ));
        };
        let name = name_raw.trim();
        let value = value_raw.trim();
        if name.is_empty() {
            return Err(make_error(
                400,
                "invalid header line: empty header name".to_string(),
            ));
        }
        if name.eq_ignore_ascii_case("content-length") {
            content_length = value.parse::<usize>().map_err(|_| {
                make_error(
                    400,
                    "invalid content-length header: expected usize".to_string(),
                )
            })?;
        }
        headers.insert(name.to_string(), value.to_string());
    }

    if content_length > max_body_bytes {
        return Err(make_error(
            413,
            format!("request body exceeds configured limit ({max_body_bytes} bytes)"),
        ));
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|err| make_error(400, format!("could not read request body: {err}")))?;
    }

    let path = request_target
        .split_once('?')
        .map(|(path, _)| path)
        .unwrap_or(request_target);

    Ok(LasmRunRequest {
        method: method.to_ascii_uppercase(),
        path: path.to_string(),
        headers,
        body,
    })
}

fn write_lasm_http_response(
    stream: &mut TcpStream,
    response: &sec4_core::HttpResponse,
    header_defaults: &LasmResponseHeaderDefaults,
    include_cors_defaults: bool,
) -> Result<(), String> {
    let mut headers = response.headers.clone();
    if !include_cors_defaults {
        headers.retain(|name, _| !is_lasm_cors_default_header_name(name.as_str()));
    }
    for (name, value) in &header_defaults.headers {
        if !include_cors_defaults && is_lasm_cors_default_header_name(name.as_str()) {
            continue;
        }
        headers
            .entry(name.clone())
            .or_insert_with(|| value.clone());
    }
    headers
        .entry("Content-Length".to_string())
        .or_insert_with(|| response.body.len().to_string());
    headers
        .entry("Connection".to_string())
        .or_insert_with(|| "close".to_string());
    headers
        .entry("Content-Type".to_string())
        .or_insert_with(|| "text/plain; charset=utf-8".to_string());

    let status_text = http_status_text(response.status);
    let mut response_head = format!("HTTP/1.1 {} {}\r\n", response.status, status_text);
    for (name, value) in headers {
        response_head.push_str(name.as_str());
        response_head.push_str(": ");
        response_head.push_str(value.as_str());
        response_head.push_str("\r\n");
    }
    response_head.push_str("\r\n");
    stream
        .write_all(response_head.as_bytes())
        .map_err(|err| format!("could not write response headers: {err}"))?;
    stream
        .write_all(&response.body)
        .map_err(|err| format!("could not write response body: {err}"))?;
    stream
        .flush()
        .map_err(|err| format!("could not flush response stream: {err}"))?;
    Ok(())
}

fn is_lasm_cors_default_header_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("Vary")
        || name.eq_ignore_ascii_case("Access-Control-Allow-Origin")
        || name.eq_ignore_ascii_case("Access-Control-Allow-Credentials")
        || name.eq_ignore_ascii_case("Access-Control-Allow-Methods")
        || name.eq_ignore_ascii_case("Access-Control-Allow-Headers")
        || name.eq_ignore_ascii_case("Access-Control-Max-Age")
        || name.eq_ignore_ascii_case("Access-Control-Allow-Private-Network")
        || name.eq_ignore_ascii_case("Access-Control-Expose-Headers")
}

fn next_lasm_trace_id(trace_counter: &AtomicU64) -> String {
    let next = trace_counter.fetch_add(1, Ordering::Relaxed) + 1;
    format!("rt-{next}")
}

fn stamp_lasm_trace_id(response: &mut sec4_core::HttpResponse, trace_counter: &AtomicU64) {
    response
        .headers
        .insert("X-Trace-Id".to_string(), next_lasm_trace_id(trace_counter));
}

fn http_status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        401 => "Unauthorized",
        403 => "Forbidden",
        400 => "Bad Request",
        413 => "Payload Too Large",
        404 => "Not Found",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Status",
    }
}

fn cmd_test(path: &Path) -> Result<(), i32> {
    let manifest = match sec4_core::validate_project(path) {
        Ok(manifest) => manifest,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            return Err(1);
        }
    };

    if let Err(diagnostics) = analyze_entry(path, &manifest) {
        print_diagnostics(&diagnostics);
        return Err(1);
    }

    let policy = match sec4_core::policy::load_policy(path) {
        Ok(policy) => policy,
        Err(diagnostics) => {
            print_diagnostics(&diagnostics);
            return Err(1);
        }
    };

    let tests_root = path.join("tests");
    let test_entries = collect_ut_files(&tests_root)?;
    let discovered_tests = test_entries.len();
    let mut static_passed = 0usize;
    let mut static_failed = 0usize;
    let mut runtime_passed = 0usize;
    let mut runtime_failed = 0usize;
    let mut runnable_entries = 0usize;
    let mut skipped_non_entry = 0usize;

    for (test_index, test_entry) in test_entries.iter().enumerate() {
        match analyze_test_entry(&tests_root, &test_entry, &policy) {
            Ok(program) => {
                if !program_declares_main(&program) {
                    skipped_non_entry += 1;
                    continue;
                }
                runnable_entries += 1;
                static_passed += 1;
                match compile_and_execute_test_entry(path, test_entry, test_index, &program) {
                    Ok(()) => runtime_passed += 1,
                    Err(_) => runtime_failed += 1,
                }
            }
            Err(diagnostics) => {
                static_failed += 1;
                print_diagnostics(&diagnostics);
            }
        }
    }

    if runnable_entries == 0 && static_failed == 0 {
        eprintln!(
            "test failed: no runnable test entrypoints found under `{}` (expected at least one `fn main`)",
            tests_root.display()
        );
        return Err(1);
    }

    println!(
        "test summary: discovered={discovered_tests}, static_passed={static_passed}, static_failed={static_failed}, runtime_passed={runtime_passed}, runtime_failed={runtime_failed}, skipped_non_entry={skipped_non_entry}"
    );
    if static_failed == 0 && runtime_failed == 0 {
        Ok(())
    } else {
        Err(1)
    }
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

    let mut changed = 0usize;
    let ut_files = collect_ut_files(path)?;
    for ut_file in ut_files {
        let source = match fs::read_to_string(&ut_file) {
            Ok(source) => source,
            Err(err) => {
                let diagnostic = Diagnostic::error(
                    "C0002",
                    "could not read .ut source file",
                    sec4_core::Span::point(ut_file, 1, 1),
                )
                .with_note(err.to_string());
                print_diagnostics(&[diagnostic]);
                return Err(1);
            }
        };

        let formatted = format_ut_source(&source);
        if formatted != source {
            if let Err(err) = fs::write(&ut_file, formatted) {
                let diagnostic = Diagnostic::error(
                    "C0003",
                    "could not write formatted .ut source file",
                    sec4_core::Span::point(ut_file, 1, 1),
                )
                .with_note(err.to_string());
                print_diagnostics(&[diagnostic]);
                return Err(1);
            }
            changed += 1;
        }
    }

    println!("fmt summary: changed={changed}");
    Ok(())
}

fn cmd_lint(path: &Path) -> Result<(), i32> {
    cmd_check(path, None)?;
    cmd_sec_audit(
        path,
        AuditOutputFormat::Text,
        None,
        None,
        None,
        None,
        None,
        Some("risk>=HIGH"),
    )
}

fn cmd_promote(
    path: &Path,
    from: PromoteTarget,
    to: PromoteTarget,
    dry_run: bool,
    out_path: Option<&Path>,
) -> Result<(), i32> {
    if from != PromoteTarget::Browser || to != PromoteTarget::Server {
        eprintln!(
            "promote failed: unsupported promotion route `{} -> {}` (only `browser -> server` is available)",
            promote_target_label(from),
            promote_target_label(to)
        );
        return Err(2);
    }

    let source_root = path.join("src");
    let source_files = collect_ut_files(&source_root)?;
    let scanned_sources = source_files
        .iter()
        .map(|file| project_relative_path(path, file))
        .collect::<Vec<_>>();

    let localdb_references = collect_promote_binding_references(path, &source_files, "localdb.")?;
    let mut preconditions = Vec::new();

    if source_files.is_empty() {
        preconditions.push(PromotePrecondition {
            code: "PROMOTE.P9301".to_string(),
            severity: "error".to_string(),
            message: "no .ut source files found under src/".to_string(),
            file: None,
            line: None,
        });
    }

    if !source_root.join("main.ut").exists() {
        preconditions.push(PromotePrecondition {
            code: "PROMOTE.P9302".to_string(),
            severity: "error".to_string(),
            message: "expected composition entry `src/main.ut` for promotion planning".to_string(),
            file: Some("src/main.ut".to_string()),
            line: Some(1),
        });
    }

    let manifest = match sec4_core::validate_project(path) {
        Ok(manifest) => Some(manifest),
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                preconditions.push(precondition_from_diagnostic(path, &diagnostic, true));
            }
            None
        }
    };

    if let Some(manifest) = manifest.as_ref() {
        if let Err(diagnostics) = analyze_entry(path, manifest) {
            for diagnostic in diagnostics {
                preconditions.push(precondition_from_diagnostic(path, &diagnostic, false));
            }
        }
    }

    if localdb_references.is_empty() {
        preconditions.push(PromotePrecondition {
            code: "PROMOTE.P9303".to_string(),
            severity: "warning".to_string(),
            message: "no localdb usage found; storage adapter rewrite may be a no-op".to_string(),
            file: None,
            line: None,
        });
    }
    if localdb_references
        .iter()
        .any(|reference| reference.file != "src/main.ut")
    {
        preconditions.push(PromotePrecondition {
            code: "PROMOTE.P9304".to_string(),
            severity: "warning".to_string(),
            message: "localdb references outside src/main.ut are preserved by composition-root-only rewrite guard".to_string(),
            file: Some("src/main.ut".to_string()),
            line: Some(1),
        });
    }

    preconditions.sort_by(|left, right| {
        (
            left.severity.as_str(),
            left.code.as_str(),
            left.file.as_deref().unwrap_or(""),
            left.line.unwrap_or(0),
            left.message.as_str(),
        )
            .cmp(&(
                right.severity.as_str(),
                right.code.as_str(),
                right.file.as_deref().unwrap_or(""),
                right.line.unwrap_or(0),
                right.message.as_str(),
            ))
    });

    let blocking_preconditions = preconditions.iter().any(|item| item.severity == "error");
    let localdb_references_json = localdb_references
        .iter()
        .map(|entry| {
            serde_json::json!({
                "file": entry.file,
                "line": entry.line
            })
        })
        .collect::<Vec<_>>();
    let generated_files = vec![
        "server/db/schema.sql".to_string(),
        "server/deploy/sec4.server.toml".to_string(),
        "server/reports/promote-plan.json".to_string(),
        "server/sec4.policy".to_string(),
        "server/sec4.toml".to_string(),
        "server/src/main.ut".to_string(),
        "server/src/repo/db_repo.ut".to_string(),
    ];
    let mode = if dry_run { "dry-run" } else { "apply-precheck" };

    let plan = serde_json::json!({
        "version": "0.1",
        "mode": mode,
        "from": promote_target_label(from),
        "to": promote_target_label(to),
        "scannedSources": scanned_sources,
        "changedBindings": [
            {
                "name": "runtime.transport",
                "from": "browser-wasm",
                "to": "server-http",
                "reason": "server promotion switches browser-local dispatch to server request handling"
            },
            {
                "name": "storage.adapter",
                "from": "localdb.*",
                "to": "db.*",
                "reason": "server promotion replaces browser-local persistence with shared database adapters",
                "references": localdb_references_json
            }
        ],
        "generatedFiles": generated_files,
        "preconditions": preconditions
            .iter()
            .map(|item| {
                serde_json::json!({
                    "code": item.code,
                    "severity": item.severity,
                    "message": item.message,
                    "file": item.file,
                    "line": item.line
                })
            })
            .collect::<Vec<_>>(),
        "ready": !blocking_preconditions
    });

    if dry_run || blocking_preconditions {
        if let Some(out_path) = out_path {
            write_json_artifact(out_path, &plan, "promotion plan")?;
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&plan).expect("promotion plan should serialize as JSON")
        );
        return if blocking_preconditions {
            Err(1)
        } else {
            Ok(())
        };
    }

    let apply_report = apply_promote_plan(path, &generated_files, &localdb_references, &plan)?;
    if let Some(out_path) = out_path {
        write_json_artifact(out_path, &apply_report, "promotion apply report")?;
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&apply_report)
            .expect("promotion apply report should serialize as JSON")
    );

    Ok(())
}

fn promote_target_label(target: PromoteTarget) -> &'static str {
    match target {
        PromoteTarget::Browser => "browser",
        PromoteTarget::Server => "server",
    }
}

fn apply_promote_plan(
    project_root: &Path,
    generated_files: &[String],
    localdb_references: &[PromoteBindingReference],
    precheck_plan: &serde_json::Value,
) -> Result<serde_json::Value, i32> {
    let composition_root = project_root.join("src/main.ut");
    let source = match fs::read_to_string(&composition_root) {
        Ok(source) => source,
        Err(err) => {
            eprintln!(
                "promote failed: could not read composition root `{}`: {err}",
                composition_root.display()
            );
            return Err(2);
        }
    };

    let rewrite_count = source.matches("localdb.").count();
    let rewritten_source = source.replace("localdb.", "db.");
    if rewritten_source != source {
        if let Err(err) = fs::write(&composition_root, rewritten_source) {
            eprintln!(
                "promote failed: could not rewrite composition root `{}`: {err}",
                composition_root.display()
            );
            return Err(2);
        }
    }

    let mut scaffold_written = Vec::new();
    for generated in generated_files {
        if generated == "server/reports/promote-plan.json" {
            continue;
        }
        let generated_path = project_root.join(generated);
        let Some(parent) = generated_path.parent() else {
            continue;
        };
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!(
                "promote failed: could not create scaffold directory `{}`: {err}",
                parent.display()
            );
            return Err(2);
        }
        let content = promote_generated_file_content(generated);
        if let Err(err) = fs::write(&generated_path, content) {
            eprintln!(
                "promote failed: could not write scaffold file `{}`: {err}",
                generated_path.display()
            );
            return Err(2);
        }
        scaffold_written.push(generated.clone());
    }

    let guarded_references = localdb_references
        .iter()
        .filter(|reference| reference.file != "src/main.ut")
        .map(|reference| {
            serde_json::json!({
                "file": reference.file,
                "line": reference.line
            })
        })
        .collect::<Vec<_>>();

    let report_rel_path = "server/reports/promote-plan.json";
    let report_path = project_root.join(report_rel_path);
    if let Some(parent) = report_path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!(
                "promote failed: could not create report directory `{}`: {err}",
                parent.display()
            );
            return Err(2);
        }
    }

    let report = serde_json::json!({
        "version": "0.1",
        "mode": "apply",
        "from": "browser",
        "to": "server",
        "compositionRewrite": {
            "file": "src/main.ut",
            "rewrites": rewrite_count
        },
        "guardedSkippedReferences": guarded_references,
        "generatedFiles": scaffold_written,
        "reportPath": report_rel_path,
        "precheckPlan": precheck_plan
    });
    if let Err(err) = fs::write(
        &report_path,
        serde_json::to_string_pretty(&report).expect("promote apply report should serialize"),
    ) {
        eprintln!(
            "promote failed: could not write apply report `{}`: {err}",
            report_path.display()
        );
        return Err(2);
    }

    Ok(report)
}

fn promote_generated_file_content(relative_path: &str) -> &'static str {
    match relative_path {
        "server/sec4.toml" => {
            "[package]\nname = \"promoted-server\"\nversion = \"0.1.0\"\n\n[build]\nentry = \"src/main.ut\"\nprofile = \"server\"\n"
        }
        "server/sec4.policy" => "",
        "server/src/main.ut" => {
            "use repo.db_repo;\n\nfn main() -> Int {\n  // Generated server composition root.\n  0\n}\n"
        }
        "server/src/repo/db_repo.ut" => {
            "fn fetch_by_id(id: Int) -> Int {\n  // Generated baseline server repository adapter placeholder.\n  id\n}\n"
        }
        "server/db/schema.sql" => {
            "-- Generated schema baseline for promoted server target.\nCREATE TABLE IF NOT EXISTS app_items (\n  id INTEGER PRIMARY KEY,\n  created_at_ms BIGINT NOT NULL\n);\n"
        }
        "server/deploy/sec4.server.toml" => {
            "[server]\nentry = \"server/src/main.ut\"\nprofile = \"server\"\n"
        }
        _ => "",
    }
}

fn write_json_artifact(path: &Path, payload: &serde_json::Value, label: &str) -> Result<(), i32> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(err) = fs::create_dir_all(parent) {
                eprintln!(
                    "promote failed: could not create artifact directory `{}`: {err}",
                    parent.display()
                );
                return Err(2);
            }
        }
    }
    let encoded =
        serde_json::to_string_pretty(payload).expect("promotion artifact payload should serialize");
    if let Err(err) = fs::write(path, encoded) {
        eprintln!(
            "promote failed: could not write {label} `{}`: {err}",
            path.display()
        );
        return Err(2);
    }
    Ok(())
}

fn precondition_from_diagnostic(
    project_root: &Path,
    diagnostic: &Diagnostic,
    force_blocking: bool,
) -> PromotePrecondition {
    let severity = if force_blocking || promote_diagnostic_blocks_plan(diagnostic) {
        "error".to_string()
    } else {
        "warning".to_string()
    };
    PromotePrecondition {
        code: format!("DIAG.{}", diagnostic.code),
        severity,
        message: diagnostic.message.clone(),
        file: Some(project_relative_path(project_root, &diagnostic.span.file)),
        line: Some(diagnostic.span.start_line),
    }
}

fn promote_diagnostic_blocks_plan(diagnostic: &Diagnostic) -> bool {
    let code = diagnostic.code.as_str();
    if code.starts_with('P') || code.starts_with('C') {
        return true;
    }
    false
}

fn project_relative_path(project_root: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(project_root).unwrap_or(file);
    relative.to_string_lossy().replace('\\', "/")
}

fn collect_promote_binding_references(
    project_root: &Path,
    source_files: &[PathBuf],
    needle: &str,
) -> Result<Vec<PromoteBindingReference>, i32> {
    let mut references = Vec::new();
    for source_file in source_files {
        let source = match fs::read_to_string(source_file) {
            Ok(source) => source,
            Err(err) => {
                eprintln!(
                    "promote failed: could not read source file `{}`: {err}",
                    source_file.display()
                );
                return Err(2);
            }
        };

        for (index, line) in source.lines().enumerate() {
            if line.contains(needle) {
                references.push(PromoteBindingReference {
                    file: project_relative_path(project_root, source_file),
                    line: index + 1,
                });
            }
        }
    }
    Ok(references)
}

fn collect_ut_files(path: &Path) -> Result<Vec<PathBuf>, i32> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    if path.is_file() {
        if is_ut_file(path) {
            return Ok(vec![path.to_path_buf()]);
        }
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    collect_ut_files_recursive(path, &mut files)?;
    Ok(files)
}

fn collect_ut_files_recursive(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), i32> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("could not read directory `{}`: {err}", path.display());
            return Err(2);
        }
    };

    let mut children = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => children.push(entry.path()),
            Err(err) => {
                eprintln!(
                    "could not read directory entry under `{}`: {err}",
                    path.display()
                );
                return Err(2);
            }
        }
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

fn is_ut_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ut"))
}

fn analyze_test_entry(
    source_root: &Path,
    entry_path: &Path,
    policy: &Policy,
) -> Result<sec4_core::ast::Program, Vec<Diagnostic>> {
    let resolved = sec4_core::resolve_modules_from_entry(source_root, entry_path)?;
    let mut combined_items = Vec::new();

    for module in resolved.modules {
        let source_for_parser = strip_allow_annotations(&module.source_without_uses);
        let parsed = parse_source(&module.file_path, &source_for_parser)?;
        combined_items.extend(parsed.items);
    }

    let program = sec4_core::ast::Program {
        items: combined_items,
        span: sec4_core::Span::point(entry_path.to_path_buf(), 1, 1),
    };
    analyze_program_with_policy(&program, policy)?;
    Ok(program)
}

fn compile_and_execute_test_entry(
    project_root: &Path,
    test_entry: &Path,
    test_index: usize,
    program: &sec4_core::ast::Program,
) -> Result<(), i32> {
    let mir = sec4_core::lower_program_to_mir(program);
    let backend_emit = emit_program_with_backend(BackendKind::C, &mir);
    let binary_name = format!("sec4-test-{:04}", test_index + 1);
    let (_, binary_path) = match compile_c_binary(
        project_root,
        &binary_name,
        &backend_emit,
        BuildTlsBackend::None,
    ) {
        Ok(paths) => paths,
        Err(code) => {
            eprintln!("test runtime compile failed for `{}`", test_entry.display());
            return Err(code);
        }
    };

    let status = match Command::new(&binary_path).status() {
        Ok(status) => status,
        Err(err) => {
            eprintln!(
                "could not execute test binary `{}` for `{}`: {err}",
                binary_path.display(),
                test_entry.display()
            );
            return Err(2);
        }
    };

    if status.success() {
        Ok(())
    } else {
        if let Some(code) = status.code() {
            eprintln!(
                "test runtime failed for `{}`: exited with status {code}",
                test_entry.display()
            );
        } else {
            eprintln!(
                "test runtime failed for `{}`: terminated by signal",
                test_entry.display()
            );
        }
        Err(1)
    }
}

fn program_declares_main(program: &sec4_core::ast::Program) -> bool {
    program.items.iter().any(|item| {
        matches!(
            &item.kind,
            sec4_core::ast::ItemKind::Function(function) if function.name == "main"
        )
    })
}

fn format_ut_source(source: &str) -> String {
    let mut formatted = source
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    formatted.push('\n');
    formatted
}

fn print_diagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        eprintln!("{}", diagnostic.render_color());
    }
}
