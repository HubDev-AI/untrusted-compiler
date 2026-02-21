use arc_swap::ArcSwap;
use base64::Engine;
use clap::{Args, Parser, Subcommand, ValueEnum};
use crossbeam_channel::{bounded, Sender, TrySendError};
use sec4_core::{
    analyze_entry, analyze_entry_with_allows, analyze_program_with_policy,
    build_security_map_with_allows, emit_program_with_backend, parse_source,
    render_security_audit_text, run_security_audit_with_baseline, should_fail,
    strip_allow_annotations, summarize_history_window, validate_lockfile_stub,
    write_build_metadata, write_lockfile_stub, write_sbom, write_security_map,
    AuditHistoryWindowSummary, AuditReport, AuditSeverity, BackendEmitOutput, BackendKind,
    Diagnostic, Policy,
};
use socket2::{Domain, Protocol, Socket, Type};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod lasm_db_adapter_state;
mod lasm_db_config;
mod lasm_db_headers;
mod lasm_db_plan;
mod lasm_db_records_log;
mod lasm_db_runtime_common;
mod lasm_db_runtime_dispatch;
mod lasm_db_runtime_postgres;
mod lasm_db_runtime_sqlite;
mod lasm_dynamic_state;

use lasm_db_config::{lasm_db_records_adapter_label, load_lasm_db_postgres_dsn_from_file};
pub(crate) use lasm_db_headers::{
    clear_lasm_internal_db_response_markers, LASM_INTERNAL_DB_HANDLE_HEADER,
    LASM_INTERNAL_DB_OP_HEADER, LASM_INTERNAL_DB_PARAMS_HEADER, LASM_INTERNAL_DB_ROW_SCHEMA_HEADER,
    LASM_INTERNAL_DB_TEMPLATE_HEADER, LASM_INTERNAL_DB_TX_DB_HEADER, LASM_INTERNAL_DB_TX_HEADER,
};
pub(crate) use lasm_db_records_log::lasm_db_record_to_json;
pub(crate) use lasm_dynamic_state::{
    build_lasm_dynamic_response_state, persist_lasm_dynamic_users_to_disk, LasmDbRecord,
    LasmDbRecordsAdapter, LasmDynamicResponseState, LASM_DYNAMIC_DB_POSTGRES_RECORDS_TABLE,
};

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
        max_header_bytes: Option<u64>,
        #[arg(long)]
        max_body_bytes: Option<u64>,
        #[arg(long)]
        max_concurrency: Option<u64>,
        #[arg(long)]
        max_pending: Option<u64>,
        #[arg(long)]
        serve_timeout_ms: Option<u64>,
        #[arg(long)]
        overflow_probe_timeout_ms: Option<u64>,
        #[arg(long)]
        max_runtime_steps: Option<u64>,
        #[arg(long)]
        max_keep_alive_requests: Option<u64>,
        #[arg(long)]
        db_base: Option<PathBuf>,
        #[arg(long, value_enum)]
        db_adapter: Option<RunDbAdapter>,
        #[arg(long)]
        db_postgres_dsn: Option<String>,
        #[arg(long)]
        db_postgres_dsn_file: Option<PathBuf>,
        #[arg(long)]
        db_max_tx_handles: Option<u64>,
        #[arg(long, default_value_t = 1)]
        instances: usize,
        #[arg(long)]
        autoscale_max_instances: Option<usize>,
        #[arg(long)]
        autoscale_target_connections: Option<usize>,
        #[arg(long, default_value_t = 1000)]
        autoscale_check_ms: u64,
        #[arg(long, default_value_t = 250)]
        autoscale_scale_up_cooldown_ms: u64,
        #[arg(long, default_value_t = 2000)]
        autoscale_scale_down_cooldown_ms: u64,
        #[arg(long, default_value_t = 2)]
        autoscale_scale_up_step: usize,
        #[arg(long, default_value_t = 1)]
        autoscale_scale_down_step: usize,
        #[arg(long, default_value_t = 4)]
        autoscale_saturation_boost_step: usize,
        #[arg(long)]
        cluster_relay_workers: Option<usize>,
        #[arg(long)]
        cluster_relay_queue: Option<usize>,
        #[arg(long)]
        cluster_accept_workers: Option<usize>,
        #[arg(long)]
        cluster_relay_accept_batch_max: Option<usize>,
        #[arg(long)]
        cluster_relay_pump_batch_max: Option<usize>,
        #[arg(long)]
        cluster_backend_connect_timeout_ms: Option<u64>,
        #[arg(long)]
        cluster_backend_connect_cooldown_ms: Option<u64>,
        #[arg(long)]
        cluster_status_json: Option<PathBuf>,
        #[arg(long, hide = true, default_value_t = false)]
        reuse_port: bool,
        #[arg(long, value_enum, default_value_t = RunBackend::Lasm)]
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
        #[arg(long = "request-header")]
        request_header: Vec<String>,
        #[arg(long)]
        request_body: Option<String>,
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
enum RunDbAdapter {
    #[value(alias = "records", alias = "records.log")]
    RecordsLog,
    Sqlite,
    #[value(alias = "pg")]
    Postgres,
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
            max_header_bytes,
            max_body_bytes,
            max_concurrency,
            max_pending,
            serve_timeout_ms,
            overflow_probe_timeout_ms,
            max_runtime_steps,
            max_keep_alive_requests,
            db_base,
            db_adapter,
            db_postgres_dsn,
            db_postgres_dsn_file,
            db_max_tx_handles,
            instances,
            autoscale_max_instances,
            autoscale_target_connections,
            autoscale_check_ms,
            autoscale_scale_up_cooldown_ms,
            autoscale_scale_down_cooldown_ms,
            autoscale_scale_up_step,
            autoscale_scale_down_step,
            autoscale_saturation_boost_step,
            cluster_relay_workers,
            cluster_relay_queue,
            cluster_accept_workers,
            cluster_relay_accept_batch_max,
            cluster_relay_pump_batch_max,
            cluster_backend_connect_timeout_ms,
            cluster_backend_connect_cooldown_ms,
            cluster_status_json,
            reuse_port,
            backend,
            tls_backend,
        } => cmd_run(
            &path,
            port,
            oneshot,
            max_header_bytes,
            max_body_bytes,
            max_concurrency,
            max_pending,
            serve_timeout_ms,
            overflow_probe_timeout_ms,
            max_runtime_steps,
            max_keep_alive_requests,
            db_base.as_deref(),
            db_adapter,
            db_postgres_dsn.as_deref(),
            db_postgres_dsn_file.as_deref(),
            db_max_tx_handles,
            instances,
            autoscale_max_instances,
            autoscale_target_connections,
            autoscale_check_ms,
            autoscale_scale_up_cooldown_ms,
            autoscale_scale_down_cooldown_ms,
            autoscale_scale_up_step,
            autoscale_scale_down_step,
            autoscale_saturation_boost_step,
            cluster_relay_workers,
            cluster_relay_queue,
            cluster_accept_workers,
            cluster_relay_accept_batch_max,
            cluster_relay_pump_batch_max,
            cluster_backend_connect_timeout_ms,
            cluster_backend_connect_cooldown_ms,
            cluster_status_json.as_deref(),
            reuse_port,
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
            request_header,
            request_body,
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
            request_header.as_slice(),
            request_body.as_deref(),
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
    request_headers: &[String],
    request_body: Option<&str>,
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
    let request_body = request_body.unwrap_or_default();
    let request_headers =
        match parse_lasm_smoke_request_headers(request_headers, request_body.len()) {
            Ok(headers) => headers,
            Err(message) => {
                eprintln!("lasm-smoke failed: {message}");
                return Err(2);
            }
        };
    let request_headers = finalize_lasm_smoke_request_headers(request_headers, request_body.len());

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
    let derived_max_pending = max_pending.or(max_in_flight);
    if let Some(limit) = derived_max_pending {
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
    let mut runtime_route_method = method.trim().to_ascii_uppercase();
    let mut runtime_route_path = route.to_string();
    let mut route_registrations = Vec::<(String, String, sec4_core::HttpResponse)>::new();
    let response_origin =
        match resolve_lasm_smoke_route_plan(&program, entry.name.as_str(), method, route) {
            Some(route_plan) => {
                runtime_route_method = route_plan.route_method.clone();
                runtime_route_path = route_plan.route_path.clone();
                let mut response =
                    sec4_core::HttpResponse::text(route_plan.status, route_plan.body);
                response.headers = route_plan.headers;
                route_registrations.push((
                    runtime_route_method.clone(),
                    runtime_route_path.clone(),
                    response,
                ));
                format!("handler:{}", route_plan.handler_name)
            }
            None => {
                let allowed_methods =
                    resolve_lasm_smoke_allowed_methods(&program, entry.name.as_str(), route);
                if let Some(allowed_methods) = allowed_methods {
                    let selected_has_match = allowed_methods
                        .iter()
                        .any(|candidate| candidate == &runtime_route_method);
                    if !selected_has_match {
                        for allowed_method in allowed_methods {
                            route_registrations.push((
                                allowed_method,
                                route.to_string(),
                                sec4_core::HttpResponse::text(200, ""),
                            ));
                        }
                        "method-mismatch".to_string()
                    } else {
                        "route-miss".to_string()
                    }
                } else {
                    "route-miss".to_string()
                }
            }
        };
    for (registration_method, registration_path, registration_response) in route_registrations {
        if let Err(message) = runtime.register_route(
            registration_method.as_str(),
            registration_path.as_str(),
            runtime_actions.clone(),
            registration_response,
        ) {
            eprintln!("lasm-smoke failed: {message}");
            return Err(1);
        }
    }

    let (smoke_request_path, smoke_query_params) = split_lasm_path_and_query(request_path);
    let smoke_request = LasmRunRequest {
        method: method.to_string(),
        http_version: "HTTP/1.1".to_string(),
        path: smoke_request_path,
        query_params: smoke_query_params,
        headers: request_headers.clone(),
        body: request_body.as_bytes().to_vec(),
    };

    for _ in 0..requests {
        let mut request = sec4_core::HttpRequest::new(method, request_path);
        request.headers = request_headers.clone();
        request.body = request_body.as_bytes().to_vec();
        runtime.submit(request);
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
    let mut first_error_code = None;
    let mut first_error_kind = None;
    while let Some(mut exchange) = runtime.pop_response() {
        apply_lasm_text_placeholder_materialization(
            &mut exchange.response,
            &smoke_request,
            &exchange.path_params,
        );
        apply_lasm_header_placeholder_materialization(
            &mut exchange.response,
            &smoke_request,
            &exchange.path_params,
        );
        let runtime_error_code = find_lasm_header_key_case_insensitive(
            &exchange.response.headers,
            LASM_INTERNAL_RUNTIME_ERROR_CODE_HEADER,
        )
        .and_then(|key| exchange.response.headers.get(&key).cloned())
        .filter(|code| !code.trim().is_empty());
        let runtime_error_kind = runtime_error_code.as_deref().map(|code| {
            lasm_internal_error_kind_for_code(code, exchange.response.status).to_string()
        });
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
            first_error_code = runtime_error_code;
            first_error_kind = runtime_error_kind;
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
    let first_error_code_text = first_error_code.as_deref().unwrap_or("-");
    let first_error_kind_text = first_error_kind.as_deref().unwrap_or("-");
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
                "lasm smoke succeeded: requestId={} responseRequestId={} entry={} origin={} resolvedRouteMethod={} resolvedRoutePath={} requests={} requestHeaderCount={} maxInFlight={} maxPending={} maxRequestMs={} ok={} errors={} statusCounts={} durationMinMs={} durationMaxMs={} durationAvgMs={} steps={} nowMs={} status={} errorCode={} errorKind={} firstDurationMs={} pathParams={} headerCount={} body={}",
                first_request_id.unwrap_or(0),
                first_response_id.unwrap_or(0),
                entry.name,
                response_origin,
                runtime_route_method,
                runtime_route_path,
                requests,
                request_headers.len(),
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
                first_error_code_text,
                first_error_kind_text,
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
                "resolvedRouteMethod": runtime_route_method,
                "resolvedRoutePath": runtime_route_path,
                "requests": requests,
                "maxInFlight": effective_max_in_flight,
                "maxPending": effective_max_pending,
                "maxRequestMs": effective_max_request_ms,
                "requestPath": request_path,
                "requestHeaderCount": request_headers.len(),
                "requestHeaders": request_headers,
                "requestBodyBytes": request_body.len(),
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
                "errorCode": first_error_code,
                "errorKind": first_error_kind,
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

fn parse_lasm_smoke_request_headers(
    request_headers: &[String],
    request_body_len: usize,
) -> Result<BTreeMap<String, String>, String> {
    let mut parsed = BTreeMap::new();
    for raw_header in request_headers {
        let header = raw_header.trim();
        if header.is_empty() {
            return Err("invalid --request-header value: expected NAME:VALUE".to_string());
        }
        let Some((name_raw, value_raw)) = header.split_once(':') else {
            return Err(format!(
                "invalid --request-header value `{header}`: missing ':' separator"
            ));
        };
        if name_raw != name_raw.trim() {
            return Err(format!(
                "invalid --request-header value `{header}`: whitespace around header name"
            ));
        }
        let name = name_raw.trim();
        if name.is_empty() {
            return Err(format!(
                "invalid --request-header value `{header}`: empty header name"
            ));
        }
        if !is_lasm_http_token(name) {
            return Err(format!(
                "invalid --request-header value `{header}`: invalid header name token"
            ));
        }
        let value = value_raw.trim();
        if !is_lasm_http_header_value(value) {
            return Err(format!(
                "invalid --request-header value `{header}`: invalid header value character"
            ));
        }
        if name.eq_ignore_ascii_case("host") {
            let host = parse_lasm_authority(value).ok_or_else(|| {
                format!("invalid --request-header value `{header}`: invalid host header")
            })?;
            if let Some(existing) = find_lasm_header_value(&parsed, name) {
                let existing_host = parse_lasm_authority(existing).ok_or_else(|| {
                    "invalid --request-header value: existing host header is invalid".to_string()
                })?;
                if existing_host != host {
                    return Err(
                        "invalid --request-header value: conflicting host headers".to_string()
                    );
                }
            }
        }
        if name.eq_ignore_ascii_case("content-length") {
            let content_length = value.parse::<usize>().map_err(|_| {
                format!(
                    "invalid --request-header value `{header}`: content-length must be a positive integer or zero"
                )
            })?;
            if content_length != request_body_len {
                return Err(format!(
                    "invalid --request-header value `{header}`: content-length does not match request body bytes ({request_body_len})"
                ));
            }
            if let Some(existing) = find_lasm_header_value(&parsed, name) {
                let existing_length = existing.parse::<usize>().map_err(|_| {
                    "invalid --request-header value: existing content-length is invalid".to_string()
                })?;
                if existing_length != content_length {
                    return Err(
                        "invalid --request-header value: conflicting content-length headers"
                            .to_string(),
                    );
                }
            }
        }
        if name.eq_ignore_ascii_case("transfer-encoding") && !value.is_empty() {
            return Err(
                "invalid --request-header value: transfer-encoding is not supported".to_string(),
            );
        }
        if name.eq_ignore_ascii_case("expect") && !value.is_empty() {
            return Err("invalid --request-header value: expect is not supported".to_string());
        }
        insert_lasm_request_header_case_insensitive(&mut parsed, name, value);
    }
    Ok(parsed)
}

fn finalize_lasm_smoke_request_headers(
    mut request_headers: BTreeMap<String, String>,
    request_body_len: usize,
) -> BTreeMap<String, String> {
    if find_lasm_header_key_case_insensitive(&request_headers, "Host").is_none() {
        request_headers.insert("Host".to_string(), "127.0.0.1".to_string());
    }
    if find_lasm_header_key_case_insensitive(&request_headers, "Content-Length").is_none() {
        let content_length = request_body_len.to_string();
        insert_lasm_request_header_case_insensitive(
            &mut request_headers,
            "Content-Length",
            content_length.as_str(),
        );
    }
    request_headers
}

#[derive(Debug, Clone)]
struct LasmSmokeRoutePlan {
    handler_name: String,
    route_method: String,
    route_path: String,
    status: u16,
    body: String,
    headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct LasmRouteRegistration {
    method: String,
    path: String,
    handler_name: String,
    router_binding: Option<String>,
    require_auth_middleware: bool,
    require_csrf_middleware: bool,
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

#[derive(Debug, Clone, Default)]
struct LasmAuthRequirement {
    require_auth: bool,
    required_role: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct LasmRouteMiddlewareRequirements {
    require_auth: bool,
    require_csrf: bool,
}

const LASM_INTERNAL_AUTH_REQUIRE_HEADER: &str = "X-Sec4-Internal-Auth-Require";
const LASM_INTERNAL_AUTH_REQUIRE_ROLE_HEADER: &str = "X-Sec4-Internal-Auth-Require-Role";
const LASM_INTERNAL_AUTH_MIDDLEWARE_REQUIRE_HEADER: &str =
    "X-Sec4-Internal-Auth-Middleware-Require";
const LASM_INTERNAL_CSRF_REQUIRE_HEADER: &str = "X-Sec4-Internal-Csrf-Require";
const LASM_INTERNAL_RUNTIME_ERROR_CODE_HEADER: &str = "X-Sec4-Internal-Error-Code";

pub(crate) fn has_lasm_sql_non_trailing_statement_separator(query_template: &str) -> bool {
    let bytes = query_template.as_bytes();
    let mut index = 0usize;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_line_comment = false;
    let mut block_comment_depth = 0usize;
    while index < bytes.len() {
        if in_line_comment {
            if bytes[index] == b'\n' {
                in_line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment_depth > 0 {
            if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
                block_comment_depth += 1;
                index += 2;
                continue;
            }
            if index + 1 < bytes.len() && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                block_comment_depth = block_comment_depth.saturating_sub(1);
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }
        if in_single_quote {
            if bytes[index] == b'\'' {
                if index + 1 < bytes.len() && bytes[index + 1] == b'\'' {
                    index += 2;
                    continue;
                }
                in_single_quote = false;
            }
            index += 1;
            continue;
        }
        if in_double_quote {
            if bytes[index] == b'"' {
                if index + 1 < bytes.len() && bytes[index + 1] == b'"' {
                    index += 2;
                    continue;
                }
                in_double_quote = false;
            }
            index += 1;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'-' && bytes[index + 1] == b'-' {
            in_line_comment = true;
            index += 2;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            block_comment_depth = 1;
            index += 2;
            continue;
        }
        if bytes[index] == b'\'' {
            in_single_quote = true;
            index += 1;
            continue;
        }
        if bytes[index] == b'"' {
            in_double_quote = true;
            index += 1;
            continue;
        }
        if bytes[index] == b';' {
            if !query_template[index + 1..].trim().is_empty() {
                return true;
            }
        }
        index += 1;
    }
    false
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
        None,
    );
    let mut plans = Vec::new();
    let mut latest_route_registrations = HashMap::new();
    for (index, registration) in registrations.into_iter().enumerate() {
        let route_key = format!("{} {}", registration.method, registration.path);
        latest_route_registrations.insert(route_key, (index, registration));
    }
    let mut registration_entries = latest_route_registrations
        .into_values()
        .collect::<Vec<(usize, LasmRouteRegistration)>>();
    registration_entries.sort_by_key(|(index, _)| *index);
    for (_, registration) in registration_entries {
        let Some(response_plan) =
            extract_response_plan(&functions, registration.handler_name.as_str())
        else {
            continue;
        };
        let mut headers = extract_response_headers(&functions, registration.handler_name.as_str());
        let auth_requirement =
            extract_auth_requirement(&functions, registration.handler_name.as_str());
        if auth_requirement.require_auth {
            headers.insert(
                LASM_INTERNAL_AUTH_REQUIRE_HEADER.to_string(),
                "1".to_string(),
            );
        }
        if registration.require_auth_middleware {
            headers.insert(
                LASM_INTERNAL_AUTH_MIDDLEWARE_REQUIRE_HEADER.to_string(),
                "1".to_string(),
            );
        }
        if let Some(required_role) = auth_requirement.required_role {
            headers.insert(
                LASM_INTERNAL_AUTH_REQUIRE_ROLE_HEADER.to_string(),
                required_role,
            );
        }
        if registration.require_csrf_middleware {
            headers.insert(
                LASM_INTERNAL_CSRF_REQUIRE_HEADER.to_string(),
                "1".to_string(),
            );
        }
        if let Some(db_operation) =
            lasm_db_plan::extract_lasm_db_operation(&functions, registration.handler_name.as_str())
        {
            lasm_db_plan::apply_lasm_db_operation_plan_headers(&mut headers, &db_operation);
        }
        if let Some(content_type) = response_plan.default_content_type {
            headers
                .entry("Content-Type".to_string())
                .or_insert(content_type);
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
    let registration = find_latest_route_registration(&functions, entry_name, method, route)?;
    let response_plan = extract_response_plan(&functions, registration.handler_name.as_str())?;
    let mut headers = extract_response_headers(&functions, registration.handler_name.as_str());
    if let Some(content_type) = response_plan.default_content_type {
        headers
            .entry("Content-Type".to_string())
            .or_insert(content_type);
    }

    Some(LasmSmokeRoutePlan {
        handler_name: registration.handler_name,
        route_method: registration.method,
        route_path: registration.path,
        status: response_plan.status,
        body: response_plan.body,
        headers,
    })
}

fn resolve_lasm_smoke_allowed_methods(
    program: &sec4_core::ast::Program,
    entry_name: &str,
    route: &str,
) -> Option<Vec<String>> {
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
        None,
    );

    let mut methods = BTreeSet::new();
    for registration in registrations {
        if route_registration_matches_selected_route(registration.path.as_str(), route) {
            methods.insert(registration.method);
        }
    }
    if methods.is_empty() {
        return None;
    }
    if methods.contains("GET") {
        methods.insert("HEAD".to_string());
    }
    Some(methods.into_iter().collect())
}

fn collect_route_registrations_in_function(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    visited: &mut HashSet<String>,
    registrations: &mut Vec<LasmRouteRegistration>,
    seed_bindings: Option<HashMap<String, sec4_core::ast::Expr>>,
) {
    if visited.contains(function_name) {
        return;
    }
    visited.insert(function_name.to_string());
    let Some(function) = functions.get(function_name) else {
        visited.remove(function_name);
        return;
    };
    let mut local_bindings = seed_bindings.unwrap_or_default();
    collect_route_registrations_in_block(
        functions,
        &function.body,
        visited,
        registrations,
        &mut local_bindings,
    );
    visited.remove(function_name);
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
            collect_route_registrations_in_expr(functions, expr, visited, registrations, bindings);
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
            maybe_apply_router_middleware_call_binding(functions, expr, bindings, registrations);
            let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
                .unwrap_or_else(|| callee.as_ref().clone());
            if let Some(registration) =
                match_route_registration_details(functions, &resolved_callee, args, bindings)
            {
                registrations.push(registration);
            }
            if let sec4_core::ast::ExprKind::Identifier(function_name) = &resolved_callee.kind {
                let callee_bindings = collect_route_registration_call_bindings(
                    functions,
                    function_name,
                    args,
                    bindings,
                );
                collect_route_registrations_in_function(
                    functions,
                    function_name,
                    visited,
                    registrations,
                    Some(callee_bindings),
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

fn find_latest_route_registration(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    entry_name: &str,
    method: &str,
    route: &str,
) -> Option<LasmRouteRegistration> {
    let normalized_method = method.trim().to_ascii_uppercase();
    let mut visited_functions = HashSet::new();
    let mut registrations = Vec::new();
    collect_route_registrations_in_function(
        functions,
        entry_name,
        &mut visited_functions,
        &mut registrations,
        None,
    );
    let find_latest_by_method = |target_method: &str| {
        registrations
            .iter()
            .rev()
            .find(|registration| {
                registration.method == target_method
                    && route_registration_matches_selected_route(registration.path.as_str(), route)
            })
            .cloned()
    };
    if let Some(registration) = find_latest_by_method(normalized_method.as_str()) {
        return Some(registration);
    }
    if normalized_method == "HEAD" {
        return find_latest_by_method("GET");
    }
    None
}

fn route_registration_matches_selected_route(
    registration_path: &str,
    selected_route: &str,
) -> bool {
    let registration_match_path = normalized_lasm_route_match_path(registration_path);
    let selected_match_path = normalized_lasm_route_match_path(selected_route);
    if registration_match_path == selected_match_path {
        return true;
    }

    let registration_segments = split_lasm_route_match_segments(registration_match_path);
    let selected_segments = split_lasm_route_match_segments(selected_match_path);
    if registration_segments.len() != selected_segments.len() {
        return false;
    }

    for (pattern_segment, selected_segment) in
        registration_segments.iter().zip(selected_segments.iter())
    {
        if let Some(param_name) = pattern_segment.strip_prefix(':') {
            if is_valid_lasm_route_param_name(param_name) {
                continue;
            }
        }
        if pattern_segment != selected_segment {
            return false;
        }
    }
    true
}

fn normalized_lasm_route_match_path(path: &str) -> &str {
    let mut end = path.len();
    if let Some(index) = path.find('?') {
        end = end.min(index);
    }
    if let Some(index) = path.find('#') {
        end = end.min(index);
    }
    &path[..end]
}

fn split_lasm_route_match_segments(path: &str) -> Vec<&str> {
    path.trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn is_valid_lasm_route_param_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn match_route_registration_details(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
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
    let middleware_requirements = if args.len() >= 3 {
        extract_router_middleware_requirements(functions, &args[0], bindings, 0)
    } else {
        LasmRouteMiddlewareRequirements::default()
    };

    let route_path = extract_route_path_literal(&args[route_arg_index], bindings)?;
    let handler_name = extract_handler_identifier(&args[handler_arg_index], bindings)?;
    let router_binding = if args.len() >= 3 {
        extract_router_binding_target(&args[0], bindings, 0)
    } else {
        None
    };
    Some(LasmRouteRegistration {
        method: route_method.to_string(),
        path: route_path,
        handler_name,
        router_binding,
        require_auth_middleware: middleware_requirements.require_auth,
        require_csrf_middleware: middleware_requirements.require_csrf,
    })
}

fn extract_router_middleware_requirements(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> LasmRouteMiddlewareRequirements {
    if depth > 32 {
        return LasmRouteMiddlewareRequirements::default();
    }
    match &expr.kind {
        sec4_core::ast::ExprKind::Identifier(name) => {
            if let Some(bound) = bindings.get(name) {
                return extract_router_middleware_requirements(
                    functions,
                    bound,
                    bindings,
                    depth + 1,
                );
            }
            LasmRouteMiddlewareRequirements::default()
        }
        sec4_core::ast::ExprKind::Call { callee, args } => {
            let mut requirements = LasmRouteMiddlewareRequirements::default();
            let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
                .unwrap_or_else(|| callee.as_ref().clone());
            if match_auth_middleware_call(&resolved_callee, bindings) {
                requirements.require_auth = true;
            }
            if match_csrf_middleware_call(&resolved_callee, bindings) {
                requirements.require_csrf = true;
            }
            if let sec4_core::ast::ExprKind::Identifier(function_name) = &resolved_callee.kind {
                let wrapper_requirements =
                    extract_router_middleware_requirements_from_function_call(
                        functions,
                        function_name,
                        args,
                        bindings,
                        depth + 1,
                    );
                requirements.require_auth |= wrapper_requirements.require_auth;
                requirements.require_csrf |= wrapper_requirements.require_csrf;
            }
            if let Some(base_router_expr) = args.first() {
                let base_requirements = extract_router_middleware_requirements(
                    functions,
                    base_router_expr,
                    bindings,
                    depth + 1,
                );
                requirements.require_auth |= base_requirements.require_auth;
                requirements.require_csrf |= base_requirements.require_csrf;
            }
            requirements
        }
        _ => LasmRouteMiddlewareRequirements::default(),
    }
}

fn collect_route_registration_call_bindings(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> HashMap<String, sec4_core::ast::Expr> {
    let mut call_bindings = HashMap::new();
    let Some(function) = functions.get(function_name) else {
        return call_bindings;
    };
    for (param, arg) in function.params.iter().zip(args.iter()) {
        let resolved =
            resolve_route_registration_expr(arg, bindings, 0).unwrap_or_else(|| arg.clone());
        call_bindings.insert(param.name.clone(), resolved);
    }
    call_bindings
}

fn extract_router_middleware_requirements_from_function_call(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    call_args: &[sec4_core::ast::Expr],
    caller_bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> LasmRouteMiddlewareRequirements {
    if depth > 32 {
        return LasmRouteMiddlewareRequirements::default();
    }
    let Some(function) = functions.get(function_name) else {
        return LasmRouteMiddlewareRequirements::default();
    };
    let mut bindings = HashMap::new();
    for (param, arg) in function.params.iter().zip(call_args.iter()) {
        let resolved =
            resolve_route_registration_expr(arg, caller_bindings, 0).unwrap_or_else(|| arg.clone());
        bindings.insert(param.name.clone(), resolved);
    }
    extract_router_middleware_requirements_from_block(
        functions,
        &function.body,
        &mut bindings,
        depth + 1,
    )
}

fn extract_router_middleware_requirements_from_block(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    block: &sec4_core::ast::Block,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> LasmRouteMiddlewareRequirements {
    let mut accumulated = LasmRouteMiddlewareRequirements::default();
    for statement in &block.statements {
        match &statement.kind {
            sec4_core::ast::StmtKind::Let { name, value, .. } => {
                let value_requirements =
                    extract_router_middleware_requirements(functions, value, bindings, depth + 1);
                accumulated.require_auth |= value_requirements.require_auth;
                accumulated.require_csrf |= value_requirements.require_csrf;
                bindings.insert(name.clone(), value.clone());
            }
            sec4_core::ast::StmtKind::Return { value } => {
                if let Some(value) = value {
                    let mut result = extract_router_middleware_requirements(
                        functions,
                        value,
                        bindings,
                        depth + 1,
                    );
                    result.require_auth |= accumulated.require_auth;
                    result.require_csrf |= accumulated.require_csrf;
                    return result;
                }
                return accumulated;
            }
            sec4_core::ast::StmtKind::Expr { expr } => {
                if let sec4_core::ast::ExprKind::Call { callee, args } = &expr.kind {
                    let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
                        .unwrap_or_else(|| callee.as_ref().clone());
                    let direct_auth = match_auth_middleware_call(&resolved_callee, bindings);
                    let direct_csrf = match_csrf_middleware_call(&resolved_callee, bindings);
                    let helper_requirements =
                        if let sec4_core::ast::ExprKind::Identifier(function_name) =
                            &resolved_callee.kind
                        {
                            extract_router_middleware_requirements_from_function_call(
                                functions,
                                function_name.as_str(),
                                args,
                                bindings,
                                depth + 1,
                            )
                        } else {
                            LasmRouteMiddlewareRequirements::default()
                        };
                    let applies_auth = direct_auth || helper_requirements.require_auth;
                    let applies_csrf = direct_csrf || helper_requirements.require_csrf;
                    if applies_auth || applies_csrf {
                        accumulated.require_auth |= applies_auth;
                        accumulated.require_csrf |= applies_csrf;
                        if let Some(name) = extract_router_binding_target_from_call(
                            functions,
                            &resolved_callee,
                            args,
                            bindings,
                        ) {
                            bindings.insert(name, expr.clone());
                        }
                    }
                }
            }
        }
    }
    if let Some(tail) = &block.tail {
        let mut result =
            extract_router_middleware_requirements(functions, tail, bindings, depth + 1);
        result.require_auth |= accumulated.require_auth;
        result.require_csrf |= accumulated.require_csrf;
        return result;
    }
    accumulated
}

fn maybe_apply_router_middleware_call_binding(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    expr: &sec4_core::ast::Expr,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
    registrations: &mut [LasmRouteRegistration],
) {
    let sec4_core::ast::ExprKind::Call { callee, args } = &expr.kind else {
        return;
    };
    let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
        .unwrap_or_else(|| callee.as_ref().clone());
    let mut requirements = LasmRouteMiddlewareRequirements::default();
    if match_auth_middleware_call(&resolved_callee, bindings) {
        requirements.require_auth = true;
    }
    if match_csrf_middleware_call(&resolved_callee, bindings) {
        requirements.require_csrf = true;
    }
    if requirements.require_auth || requirements.require_csrf {
        // no-op; direct middleware calls already captured above
    } else if let sec4_core::ast::ExprKind::Identifier(function_name) = &resolved_callee.kind {
        let helper_requirements = extract_router_middleware_requirements_from_function_call(
            functions,
            function_name.as_str(),
            args,
            bindings,
            1,
        );
        requirements.require_auth = helper_requirements.require_auth;
        requirements.require_csrf = helper_requirements.require_csrf;
    }
    if !requirements.require_auth && !requirements.require_csrf {
        return;
    }

    if let Some(name) =
        extract_router_binding_target_from_call(functions, &resolved_callee, args, bindings)
    {
        for registration in registrations.iter_mut() {
            if registration.router_binding.as_deref() == Some(name.as_str()) {
                registration.require_auth_middleware |= requirements.require_auth;
                registration.require_csrf_middleware |= requirements.require_csrf;
            }
        }
        bindings.insert(name, expr.clone());
    }
}

fn extract_router_binding_target_from_call(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    if args.is_empty() {
        return None;
    }
    if match_auth_middleware_call(callee, bindings) || match_csrf_middleware_call(callee, bindings)
    {
        return args
            .first()
            .and_then(|entry| extract_router_binding_target(entry, bindings, 0));
    }
    let resolved_callee =
        resolve_route_registration_expr(callee, bindings, 0).unwrap_or_else(|| callee.clone());
    if let sec4_core::ast::ExprKind::Identifier(function_name) = &resolved_callee.kind {
        if let Some(function) = functions.get(function_name.as_str()) {
            if let Some(index) = function.params.iter().position(|param| {
                matches!(
                    &param.ty.kind,
                    sec4_core::ast::TypeExprKind::Named { name, args }
                    if name == "Router" && args.is_empty()
                )
            }) {
                if let Some(argument) = args.get(index) {
                    if let Some(target) = extract_router_binding_target(argument, bindings, 0) {
                        return Some(target);
                    }
                }
            }
        }
    }
    args.first()
        .and_then(|entry| extract_router_binding_target(entry, bindings, 0))
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

fn extract_router_binding_target(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<String> {
    if depth > 32 {
        return None;
    }
    let sec4_core::ast::ExprKind::Identifier(name) = &expr.kind else {
        return None;
    };
    if let Some(bound) = bindings.get(name) {
        if let Some(inner) = extract_router_binding_target(bound, bindings, depth + 1) {
            return Some(inner);
        }
    }
    Some(name.clone())
}

fn extract_response_plan(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
) -> Option<LasmResponsePlan> {
    let mut visited = HashSet::new();
    extract_response_plan_in_function(functions, function_name, &mut visited, None)
}

fn extract_response_plan_in_function(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    visited: &mut HashSet<String>,
    seed_bindings: Option<HashMap<String, sec4_core::ast::Expr>>,
) -> Option<LasmResponsePlan> {
    if visited.contains(function_name) {
        return None;
    }
    visited.insert(function_name.to_string());
    let Some(function) = functions.get(function_name) else {
        visited.remove(function_name);
        return None;
    };
    let mut bindings = seed_bindings.unwrap_or_default();
    let response =
        extract_response_plan_in_block(functions, &function.body, visited, &mut bindings);
    visited.remove(function_name);
    response
}

fn extract_response_plan_in_block(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    block: &sec4_core::ast::Block,
    visited: &mut HashSet<String>,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    let mut response = None;
    for statement in &block.statements {
        if let Some(next) = extract_response_plan_in_stmt(functions, statement, visited, bindings) {
            response = Some(next);
        }
    }
    if let Some(tail) = &block.tail {
        if let Some(next) = extract_response_plan_in_expr(functions, tail, visited, bindings) {
            response = Some(next);
        }
    }
    response
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
            let mut response = None;
            let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
                .unwrap_or_else(|| callee.as_ref().clone());
            if let Some(next) = extract_response_plan_in_expr(functions, callee, visited, bindings)
            {
                response = Some(next);
            }
            for argument in args {
                if let Some(next) =
                    extract_response_plan_in_expr(functions, argument, visited, bindings)
                {
                    response = Some(next);
                }
            }
            if let Some(response) = match_response_helper_call(&resolved_callee, args, bindings) {
                return Some(response);
            }
            if let sec4_core::ast::ExprKind::Identifier(function_name) = &resolved_callee.kind {
                let call_bindings =
                    collect_response_plan_call_bindings(functions, function_name, args, bindings);
                if let Some(response) = extract_response_plan_in_function(
                    functions,
                    function_name,
                    visited,
                    Some(call_bindings),
                ) {
                    return Some(response);
                }
            }
            response
        }
        sec4_core::ast::ExprKind::Unary { expr, .. } => {
            extract_response_plan_in_expr(functions, expr, visited, bindings)
        }
        sec4_core::ast::ExprKind::Binary { left, right, .. } => {
            let mut response = extract_response_plan_in_expr(functions, left, visited, bindings);
            if let Some(next) = extract_response_plan_in_expr(functions, right, visited, bindings) {
                response = Some(next);
            }
            response
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

fn collect_response_plan_call_bindings(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> HashMap<String, sec4_core::ast::Expr> {
    let mut call_bindings = HashMap::new();
    let Some(function) = functions.get(function_name) else {
        return call_bindings;
    };
    for (param, arg) in function.params.iter().zip(args.iter()) {
        let resolved = resolve_response_expr(arg, bindings, 0).unwrap_or_else(|| arg.clone());
        call_bindings.insert(param.name.clone(), resolved);
    }
    call_bindings
}

fn extract_auth_requirement(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
) -> LasmAuthRequirement {
    let mut requirement = LasmAuthRequirement::default();
    let mut visited = HashSet::new();
    extract_auth_requirement_in_function(
        functions,
        function_name,
        &mut visited,
        &mut requirement,
        None,
    );
    requirement
}

fn match_auth_middleware_call(
    callee: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> bool {
    let resolved = resolve_route_registration_expr(callee, bindings, 0);
    match resolved.as_ref().map(|expr| &expr.kind) {
        Some(sec4_core::ast::ExprKind::Identifier(name)) => name == "withAuth",
        Some(sec4_core::ast::ExprKind::Member { object, field }) => {
            if let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind {
                namespace == "auth" && field == "withAuth"
            } else {
                false
            }
        }
        _ => false,
    }
}

fn match_csrf_middleware_call(
    callee: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> bool {
    let resolved = resolve_route_registration_expr(callee, bindings, 0);
    match resolved.as_ref().map(|expr| &expr.kind) {
        Some(sec4_core::ast::ExprKind::Identifier(name)) => name == "withCsrf",
        Some(sec4_core::ast::ExprKind::Member { object, field }) => {
            if let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind {
                namespace == "csrf" && field == "withCsrf"
            } else {
                false
            }
        }
        _ => false,
    }
}

fn extract_auth_requirement_in_function(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    visited: &mut HashSet<String>,
    requirement: &mut LasmAuthRequirement,
    seed_bindings: Option<HashMap<String, sec4_core::ast::Expr>>,
) {
    if visited.contains(function_name) {
        return;
    }
    visited.insert(function_name.to_string());
    let Some(function) = functions.get(function_name) else {
        visited.remove(function_name);
        return;
    };
    let mut bindings = seed_bindings.unwrap_or_default();
    extract_auth_requirement_in_block(
        functions,
        &function.body,
        visited,
        requirement,
        &mut bindings,
    );
    visited.remove(function_name);
}

fn extract_auth_requirement_in_block(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    block: &sec4_core::ast::Block,
    visited: &mut HashSet<String>,
    requirement: &mut LasmAuthRequirement,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) {
    for statement in &block.statements {
        extract_auth_requirement_in_stmt(functions, statement, visited, requirement, bindings);
    }
    if let Some(tail) = &block.tail {
        extract_auth_requirement_in_expr(functions, tail, visited, requirement, bindings);
    }
}

fn extract_auth_requirement_in_stmt(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    statement: &sec4_core::ast::Stmt,
    visited: &mut HashSet<String>,
    requirement: &mut LasmAuthRequirement,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) {
    match &statement.kind {
        sec4_core::ast::StmtKind::Let { name, value, .. } => {
            extract_auth_requirement_in_expr(functions, value, visited, requirement, bindings);
            bindings.insert(name.clone(), value.clone());
        }
        sec4_core::ast::StmtKind::Return { value } => {
            if let Some(value) = value {
                extract_auth_requirement_in_expr(functions, value, visited, requirement, bindings);
            }
        }
        sec4_core::ast::StmtKind::Expr { expr } => {
            extract_auth_requirement_in_expr(functions, expr, visited, requirement, bindings);
        }
    }
}

fn extract_auth_requirement_in_expr(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    expr: &sec4_core::ast::Expr,
    visited: &mut HashSet<String>,
    requirement: &mut LasmAuthRequirement,
    bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) {
    match &expr.kind {
        sec4_core::ast::ExprKind::Call { callee, args } => {
            let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
                .unwrap_or_else(|| callee.as_ref().clone());
            if let Some(call_requirement) =
                match_auth_requirement_call(&resolved_callee, args, bindings)
            {
                requirement.require_auth |= call_requirement.require_auth;
                if let Some(required_role) = call_requirement.required_role {
                    requirement.required_role = Some(required_role);
                }
            }
            if let sec4_core::ast::ExprKind::Identifier(function_name) = &resolved_callee.kind {
                let call_bindings = collect_auth_requirement_call_bindings(
                    functions,
                    function_name,
                    args,
                    bindings,
                );
                extract_auth_requirement_in_function(
                    functions,
                    function_name,
                    visited,
                    requirement,
                    Some(call_bindings),
                );
            }
            extract_auth_requirement_in_expr(functions, callee, visited, requirement, bindings);
            for argument in args {
                extract_auth_requirement_in_expr(
                    functions,
                    argument,
                    visited,
                    requirement,
                    bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Unary { expr, .. } => {
            extract_auth_requirement_in_expr(functions, expr, visited, requirement, bindings)
        }
        sec4_core::ast::ExprKind::Binary { left, right, .. } => {
            extract_auth_requirement_in_expr(functions, left, visited, requirement, bindings);
            extract_auth_requirement_in_expr(functions, right, visited, requirement, bindings);
        }
        sec4_core::ast::ExprKind::Member { object, .. } => {
            extract_auth_requirement_in_expr(functions, object, visited, requirement, bindings)
        }
        sec4_core::ast::ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            extract_auth_requirement_in_expr(functions, condition, visited, requirement, bindings);
            let mut then_bindings = bindings.clone();
            extract_auth_requirement_in_block(
                functions,
                then_branch,
                visited,
                requirement,
                &mut then_bindings,
            );
            if let Some(else_branch) = else_branch {
                let mut else_bindings = bindings.clone();
                extract_auth_requirement_in_expr(
                    functions,
                    else_branch,
                    visited,
                    requirement,
                    &mut else_bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Match { scrutinee, arms } => {
            extract_auth_requirement_in_expr(functions, scrutinee, visited, requirement, bindings);
            for arm in arms {
                let mut arm_bindings = bindings.clone();
                extract_auth_requirement_in_expr(
                    functions,
                    &arm.value,
                    visited,
                    requirement,
                    &mut arm_bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Block(block) => {
            let mut block_bindings = bindings.clone();
            extract_auth_requirement_in_block(
                functions,
                block,
                visited,
                requirement,
                &mut block_bindings,
            );
        }
        sec4_core::ast::ExprKind::Identifier(_)
        | sec4_core::ast::ExprKind::Number(_)
        | sec4_core::ast::ExprKind::String(_)
        | sec4_core::ast::ExprKind::Bool(_) => {}
    }
}

fn collect_auth_requirement_call_bindings(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> HashMap<String, sec4_core::ast::Expr> {
    let mut call_bindings = HashMap::new();
    let Some(function) = functions.get(function_name) else {
        return call_bindings;
    };
    for (param, arg) in function.params.iter().zip(args.iter()) {
        let resolved =
            resolve_route_registration_expr(arg, bindings, 0).unwrap_or_else(|| arg.clone());
        call_bindings.insert(param.name.clone(), resolved);
    }
    call_bindings
}

fn match_auth_requirement_call(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmAuthRequirement> {
    let mut is_require = false;
    let mut is_require_role = false;
    let resolved_callee =
        resolve_route_registration_expr(callee, bindings, 0).unwrap_or_else(|| callee.clone());
    match &resolved_callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => {
            is_require = name == "auth_require";
            is_require_role = name == "auth_require_role";
        }
        sec4_core::ast::ExprKind::Member { object, field } => {
            if let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind {
                if namespace == "auth" {
                    is_require = field == "require";
                    is_require_role = field == "requireRole";
                }
            }
        }
        _ => {}
    }

    if is_require {
        return Some(LasmAuthRequirement {
            require_auth: true,
            required_role: None,
        });
    }

    if is_require_role {
        let required_role = args
            .get(1)
            .and_then(|expr| parse_lasm_res_text_template(expr, bindings, 0))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "role".to_string());
        return Some(LasmAuthRequirement {
            require_auth: true,
            required_role: Some(required_role),
        });
    }

    None
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
    let body = parse_lasm_res_text_body(&args[1], bindings)?;
    Some(LasmResponsePlan {
        status,
        body,
        default_content_type: Some("text/plain; charset=utf-8".to_string()),
    })
}

fn parse_lasm_res_text_body(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    parse_lasm_res_text_template(expr, bindings, 0)
}

fn parse_lasm_res_text_template(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<String> {
    if depth > 32 {
        return None;
    }
    if let Some(literal) = parse_string_literal(expr, bindings) {
        return Some(literal);
    }
    let resolved = resolve_response_expr(expr, bindings, depth)?;
    match &resolved.kind {
        sec4_core::ast::ExprKind::Call { callee, args } => {
            let resolved_callee = resolve_route_registration_expr(callee, bindings, 0)
                .unwrap_or_else(|| callee.as_ref().clone());
            if let Some(placeholder) =
                parse_lasm_request_text_placeholder(&resolved_callee, args, bindings)
            {
                return Some(placeholder);
            }
            if is_lasm_sanitize_html_call(&resolved_callee) && !args.is_empty() {
                let template = parse_lasm_res_text_template(&args[0], bindings, depth + 1)?;
                return Some(build_lasm_sanitize_html_template(template.as_str()));
            }
            if is_lasm_validate_non_empty_call(&resolved_callee) && !args.is_empty() {
                return parse_lasm_res_text_template(&args[0], bindings, depth + 1);
            }
            None
        }
        sec4_core::ast::ExprKind::Binary { op, left, right }
            if *op == sec4_core::ast::BinaryOp::Add =>
        {
            let lhs = parse_lasm_res_text_template(left, bindings, depth + 1)?;
            let rhs = parse_lasm_res_text_template(right, bindings, depth + 1)?;
            Some(format!("{lhs}{rhs}"))
        }
        _ => None,
    }
}

fn parse_lasm_request_text_placeholder(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<String> {
    let sec4_core::ast::ExprKind::Member { object, field } = &callee.kind else {
        return None;
    };
    let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind else {
        return None;
    };
    if namespace != "req" {
        return None;
    }
    match field.as_str() {
        "method" if args.is_empty() => Some("{{req.method}}".to_string()),
        "path" if args.is_empty() => Some("{{req.path}}".to_string()),
        "httpVersion" if args.is_empty() => Some("{{req.httpVersion}}".to_string()),
        "body" => Some("{{req.body}}".to_string()),
        "pathParam" | "header" | "query" | "cookie" => {
            if args.is_empty() {
                return None;
            }
            let key = parse_string_literal(&args[0], bindings)?;
            if key.trim().is_empty() {
                return None;
            }
            match field.as_str() {
                "pathParam" => Some(format!("{{{{req.pathParam:{key}}}}}")),
                "header" => Some(format!("{{{{req.header:{key}}}}}")),
                "query" => Some(format!("{{{{req.query:{key}}}}}")),
                "cookie" => Some(format!("{{{{req.cookie:{key}}}}}")),
                _ => None,
            }
        }
        _ => None,
    }
}

fn is_lasm_validate_non_empty_call(callee: &sec4_core::ast::Expr) -> bool {
    match &callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => name == "validate_non_empty",
        sec4_core::ast::ExprKind::Member { object, field } => {
            field == "nonEmpty"
                && matches!(
                    object.kind,
                    sec4_core::ast::ExprKind::Identifier(ref name) if name == "validate"
                )
        }
        _ => false,
    }
}

fn is_lasm_validate_header_value_call(callee: &sec4_core::ast::Expr) -> bool {
    match &callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => name == "validate_header_value",
        sec4_core::ast::ExprKind::Member { object, field } => {
            field == "headerValue"
                && matches!(
                    object.kind,
                    sec4_core::ast::ExprKind::Identifier(ref name) if name == "validate"
                )
        }
        _ => false,
    }
}

fn is_lasm_validate_int64_call(callee: &sec4_core::ast::Expr) -> bool {
    match &callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => name == "validate_int64",
        sec4_core::ast::ExprKind::Member { object, field } => {
            field == "int64"
                && matches!(
                    object.kind,
                    sec4_core::ast::ExprKind::Identifier(ref name) if name == "validate"
                )
        }
        _ => false,
    }
}

fn is_lasm_schema_row_call(callee: &sec4_core::ast::Expr) -> bool {
    match &callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => name == "schema_row",
        sec4_core::ast::ExprKind::Member { object, field } => {
            field == "row"
                && matches!(
                    object.kind,
                    sec4_core::ast::ExprKind::Identifier(ref name) if name == "schema"
                )
        }
        _ => false,
    }
}

fn is_lasm_sanitize_html_call(callee: &sec4_core::ast::Expr) -> bool {
    match &callee.kind {
        sec4_core::ast::ExprKind::Identifier(name) => name == "sanitize_html",
        sec4_core::ast::ExprKind::Member { object, field } => {
            field == "html"
                && matches!(
                    object.kind,
                    sec4_core::ast::ExprKind::Identifier(ref name) if name == "sanitize"
                )
        }
        _ => false,
    }
}

fn build_lasm_sanitize_html_template(template: &str) -> String {
    let escaped = escape_lasm_html(template);
    let with_method = escaped.replace("{{req.method}}", "{{sanitizeHtml:req.method}}");
    let with_path = with_method.replace("{{req.path}}", "{{sanitizeHtml:req.path}}");
    let with_http_version =
        with_path.replace("{{req.httpVersion}}", "{{sanitizeHtml:req.httpVersion}}");
    let with_body = with_http_version.replace("{{req.body}}", "{{sanitizeHtml:req.body}}");
    let with_path_params = rewrite_lasm_placeholder_prefix(
        &with_body,
        "{{req.pathParam:",
        "{{sanitizeHtml:req.pathParam:",
    );
    let with_headers = rewrite_lasm_placeholder_prefix(
        &with_path_params,
        "{{req.header:",
        "{{sanitizeHtml:req.header:",
    );
    let with_cookies = rewrite_lasm_placeholder_prefix(
        &with_headers,
        "{{req.cookie:",
        "{{sanitizeHtml:req.cookie:",
    );
    rewrite_lasm_placeholder_prefix(&with_cookies, "{{req.query:", "{{sanitizeHtml:req.query:")
}

fn rewrite_lasm_placeholder_prefix(value: &str, prefix: &str, rewritten_prefix: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut rest = value;
    loop {
        let Some(start) = rest.find(prefix) else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..start]);
        let value_start = start + prefix.len();
        let after_value_start = &rest[value_start..];
        let Some(value_end) = after_value_start.find("}}") else {
            output.push_str(&rest[start..]);
            break;
        };
        let token_value = &after_value_start[..value_end];
        output.push_str(rewritten_prefix);
        output.push_str(token_value);
        output.push_str("}}");
        rest = &after_value_start[value_end + 2..];
    }
    output
}

fn match_res_html_call(
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> Option<LasmResponsePlan> {
    if args.len() != 1 {
        return None;
    }
    let body = parse_lasm_res_text_template(&args[0], bindings, 0)
        .unwrap_or_else(|| "<html></html>".to_string());
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
    let schema = match args.len() {
        2 => parse_string_literal(&args[0], bindings),
        _ => parse_string_literal(&args[1], bindings),
    };
    Some(LasmResponsePlan {
        status,
        body: build_lasm_json_response_body(status, schema.as_deref()),
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
    let schema = parse_string_literal(&args[1], bindings);
    Some(LasmResponsePlan {
        status,
        body: build_lasm_ok_response_body(status, schema.as_deref()),
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
    let schema = parse_string_literal(&args[1], bindings);
    Some(LasmResponsePlan {
        status,
        body: build_lasm_ok_response_body(status, schema.as_deref()),
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

fn build_lasm_json_response_body(status: u16, schema: Option<&str>) -> String {
    match schema {
        Some(schema_name) => format!(
            "{{\"status\":{status},\"schema\":\"{}\"}}",
            escape_json_string(schema_name)
        ),
        None => format!("{{\"status\":{status}}}"),
    }
}

fn build_lasm_ok_response_body(status: u16, schema: Option<&str>) -> String {
    match schema {
        Some(schema_name) => format!(
            "{{\"ok\":true,\"status\":{status},\"schema\":\"{}\"}}",
            escape_json_string(schema_name)
        ),
        None => format!("{{\"ok\":true,\"status\":{status}}}"),
    }
}

fn escape_json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
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
    extract_response_headers_in_function(
        functions,
        function_name,
        &mut visited,
        &mut headers,
        None,
        None,
    );
    headers
}

fn extract_response_headers_in_function(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    visited: &mut HashSet<String>,
    headers: &mut BTreeMap<String, String>,
    seed_value_bindings: Option<HashMap<String, String>>,
    seed_expr_bindings: Option<HashMap<String, sec4_core::ast::Expr>>,
) {
    if visited.contains(function_name) {
        return;
    }
    visited.insert(function_name.to_string());
    let Some(function) = functions.get(function_name) else {
        visited.remove(function_name);
        return;
    };
    let mut value_bindings = seed_value_bindings.unwrap_or_default();
    let mut expr_bindings = seed_expr_bindings.unwrap_or_default();
    extract_response_headers_in_block(
        functions,
        &function.body,
        visited,
        headers,
        &mut value_bindings,
        &mut expr_bindings,
    );
    visited.remove(function_name);
}

fn extract_response_headers_in_block(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    block: &sec4_core::ast::Block,
    visited: &mut HashSet<String>,
    headers: &mut BTreeMap<String, String>,
    value_bindings: &mut HashMap<String, String>,
    expr_bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) {
    for statement in &block.statements {
        extract_response_headers_in_stmt(
            functions,
            statement,
            visited,
            headers,
            value_bindings,
            expr_bindings,
        );
    }
    if let Some(tail) = &block.tail {
        extract_response_headers_in_expr(
            functions,
            tail,
            visited,
            headers,
            value_bindings,
            expr_bindings,
        );
    }
}

fn extract_response_headers_in_stmt(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    statement: &sec4_core::ast::Stmt,
    visited: &mut HashSet<String>,
    headers: &mut BTreeMap<String, String>,
    value_bindings: &mut HashMap<String, String>,
    expr_bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) {
    match &statement.kind {
        sec4_core::ast::StmtKind::Let { name, value, .. } => {
            expr_bindings.insert(name.clone(), value.clone());
            if let Some(binding_value) =
                extract_header_binding_literal(value, value_bindings, expr_bindings, 0)
            {
                value_bindings.insert(name.clone(), binding_value);
            }
            extract_response_headers_in_expr(
                functions,
                value,
                visited,
                headers,
                value_bindings,
                expr_bindings,
            )
        }
        sec4_core::ast::StmtKind::Return { value } => {
            if let Some(value) = value {
                extract_response_headers_in_expr(
                    functions,
                    value,
                    visited,
                    headers,
                    value_bindings,
                    expr_bindings,
                );
            }
        }
        sec4_core::ast::StmtKind::Expr { expr } => extract_response_headers_in_expr(
            functions,
            expr,
            visited,
            headers,
            value_bindings,
            expr_bindings,
        ),
    }
}

fn extract_response_headers_in_expr(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    expr: &sec4_core::ast::Expr,
    visited: &mut HashSet<String>,
    headers: &mut BTreeMap<String, String>,
    value_bindings: &mut HashMap<String, String>,
    expr_bindings: &mut HashMap<String, sec4_core::ast::Expr>,
) {
    match &expr.kind {
        sec4_core::ast::ExprKind::Call { callee, args } => {
            let resolved_callee = resolve_route_registration_expr(callee, expr_bindings, 0)
                .unwrap_or_else(|| callee.as_ref().clone());
            if let Some((name, value)) =
                match_res_set_header_call(&resolved_callee, args, value_bindings, expr_bindings)
            {
                headers.insert(name, value);
            }
            if let Some(cookie) =
                match_res_add_cookie_call(&resolved_callee, args, value_bindings, expr_bindings)
            {
                append_lasm_set_cookie_header(headers, cookie.as_str());
            }
            if let sec4_core::ast::ExprKind::Identifier(function_name) = &resolved_callee.kind {
                let (call_value_bindings, call_expr_bindings) =
                    collect_response_header_call_bindings(
                        functions,
                        function_name,
                        args,
                        value_bindings,
                        expr_bindings,
                    );
                extract_response_headers_in_function(
                    functions,
                    function_name,
                    visited,
                    headers,
                    Some(call_value_bindings),
                    Some(call_expr_bindings),
                );
            }
            extract_response_headers_in_expr(
                functions,
                callee,
                visited,
                headers,
                value_bindings,
                expr_bindings,
            );
            for argument in args {
                extract_response_headers_in_expr(
                    functions,
                    argument,
                    visited,
                    headers,
                    value_bindings,
                    expr_bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Unary { expr, .. } => extract_response_headers_in_expr(
            functions,
            expr,
            visited,
            headers,
            value_bindings,
            expr_bindings,
        ),
        sec4_core::ast::ExprKind::Binary { left, right, .. } => {
            extract_response_headers_in_expr(
                functions,
                left,
                visited,
                headers,
                value_bindings,
                expr_bindings,
            );
            extract_response_headers_in_expr(
                functions,
                right,
                visited,
                headers,
                value_bindings,
                expr_bindings,
            );
        }
        sec4_core::ast::ExprKind::Member { object, .. } => extract_response_headers_in_expr(
            functions,
            object,
            visited,
            headers,
            value_bindings,
            expr_bindings,
        ),
        sec4_core::ast::ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            extract_response_headers_in_expr(
                functions,
                condition,
                visited,
                headers,
                value_bindings,
                expr_bindings,
            );
            let mut then_value_bindings = value_bindings.clone();
            let mut then_expr_bindings = expr_bindings.clone();
            extract_response_headers_in_block(
                functions,
                then_branch,
                visited,
                headers,
                &mut then_value_bindings,
                &mut then_expr_bindings,
            );
            if let Some(else_branch) = else_branch {
                let mut else_value_bindings = value_bindings.clone();
                let mut else_expr_bindings = expr_bindings.clone();
                extract_response_headers_in_expr(
                    functions,
                    else_branch,
                    visited,
                    headers,
                    &mut else_value_bindings,
                    &mut else_expr_bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Match { scrutinee, arms } => {
            extract_response_headers_in_expr(
                functions,
                scrutinee,
                visited,
                headers,
                value_bindings,
                expr_bindings,
            );
            for arm in arms {
                let mut arm_value_bindings = value_bindings.clone();
                let mut arm_expr_bindings = expr_bindings.clone();
                extract_response_headers_in_expr(
                    functions,
                    &arm.value,
                    visited,
                    headers,
                    &mut arm_value_bindings,
                    &mut arm_expr_bindings,
                );
            }
        }
        sec4_core::ast::ExprKind::Block(block) => {
            let mut block_value_bindings = value_bindings.clone();
            let mut block_expr_bindings = expr_bindings.clone();
            extract_response_headers_in_block(
                functions,
                block,
                visited,
                headers,
                &mut block_value_bindings,
                &mut block_expr_bindings,
            )
        }
        sec4_core::ast::ExprKind::Identifier(_)
        | sec4_core::ast::ExprKind::Number(_)
        | sec4_core::ast::ExprKind::String(_)
        | sec4_core::ast::ExprKind::Bool(_) => {}
    }
}

fn collect_response_header_call_bindings(
    functions: &HashMap<&str, &sec4_core::ast::FunctionDecl>,
    function_name: &str,
    args: &[sec4_core::ast::Expr],
    value_bindings: &HashMap<String, String>,
    expr_bindings: &HashMap<String, sec4_core::ast::Expr>,
) -> (
    HashMap<String, String>,
    HashMap<String, sec4_core::ast::Expr>,
) {
    let mut call_value_bindings = HashMap::new();
    let mut call_expr_bindings = HashMap::new();
    let Some(function) = functions.get(function_name) else {
        return (call_value_bindings, call_expr_bindings);
    };
    for (param, arg) in function.params.iter().zip(args.iter()) {
        let resolved =
            resolve_route_registration_expr(arg, expr_bindings, 0).unwrap_or_else(|| arg.clone());
        if let Some(value) =
            extract_header_binding_literal(&resolved, value_bindings, expr_bindings, 0)
        {
            call_value_bindings.insert(param.name.clone(), value);
        }
        call_expr_bindings.insert(param.name.clone(), resolved);
    }
    (call_value_bindings, call_expr_bindings)
}

fn append_lasm_set_cookie_header(headers: &mut BTreeMap<String, String>, cookie: &str) {
    let key = "Set-Cookie".to_string();
    match headers.get_mut(&key) {
        Some(existing) => {
            if !existing.is_empty() {
                existing.push('\n');
            }
            existing.push_str(cookie);
        }
        None => {
            headers.insert(key, cookie.to_string());
        }
    }
}

fn match_res_set_header_call(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    value_bindings: &HashMap<String, String>,
    expr_bindings: &HashMap<String, sec4_core::ast::Expr>,
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
    let name = extract_header_gate_literal(&args[0], "name", value_bindings, expr_bindings, 0)?;
    let value = extract_header_gate_literal(&args[1], "value", value_bindings, expr_bindings, 0)?;
    Some((name, value))
}

fn match_res_add_cookie_call(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    value_bindings: &HashMap<String, String>,
    expr_bindings: &HashMap<String, sec4_core::ast::Expr>,
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
    extract_cookie_literal(&args[0], value_bindings, expr_bindings, 0)
}

fn extract_header_binding_literal(
    expr: &sec4_core::ast::Expr,
    value_bindings: &HashMap<String, String>,
    expr_bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<String> {
    if depth > 32 {
        return None;
    }
    extract_header_gate_literal(expr, "name", value_bindings, expr_bindings, depth)
        .or_else(|| {
            extract_header_gate_literal(expr, "value", value_bindings, expr_bindings, depth)
        })
        .or_else(|| extract_lasm_request_header_placeholder(expr, value_bindings, 0))
        .or_else(|| extract_cookie_literal(expr, value_bindings, expr_bindings, depth))
}

fn extract_cookie_literal(
    expr: &sec4_core::ast::Expr,
    value_bindings: &HashMap<String, String>,
    expr_bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<String> {
    if depth > 32 {
        return None;
    }
    match &expr.kind {
        sec4_core::ast::ExprKind::String(value) => Some(value.clone()),
        sec4_core::ast::ExprKind::Identifier(name) => {
            if let Some(value) = value_bindings.get(name) {
                return Some(value.clone());
            }
            let bound = expr_bindings.get(name)?;
            extract_cookie_literal(bound, value_bindings, expr_bindings, depth + 1)
        }
        sec4_core::ast::ExprKind::Call { callee, args } => {
            let resolved_callee = resolve_route_registration_expr(callee, expr_bindings, 0)
                .unwrap_or_else(|| callee.as_ref().clone());
            let sec4_core::ast::ExprKind::Member { object, field } = &resolved_callee.kind else {
                return None;
            };
            let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind else {
                return None;
            };
            if namespace != "cookie" || field != "build" || args.len() < 2 {
                return None;
            }
            let name = extract_string_literal_or_binding_with_expr(
                &args[0],
                value_bindings,
                expr_bindings,
                depth + 1,
            )
            .or_else(|| extract_lasm_request_header_placeholder(&args[0], value_bindings, 0))?;
            let value = extract_string_literal_or_binding_with_expr(
                &args[1],
                value_bindings,
                expr_bindings,
                depth + 1,
            )
            .or_else(|| extract_lasm_request_header_placeholder(&args[1], value_bindings, 0))?;
            Some(format!("{name}={value}"))
        }
        _ => None,
    }
}

fn extract_string_literal_or_binding_with_expr(
    expr: &sec4_core::ast::Expr,
    value_bindings: &HashMap<String, String>,
    expr_bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<String> {
    if depth > 32 {
        return None;
    }
    match &expr.kind {
        sec4_core::ast::ExprKind::String(value) => Some(value.clone()),
        sec4_core::ast::ExprKind::Identifier(name) => {
            if let Some(value) = value_bindings.get(name) {
                return Some(value.clone());
            }
            let bound = expr_bindings.get(name)?;
            extract_string_literal_or_binding_with_expr(
                bound,
                value_bindings,
                expr_bindings,
                depth + 1,
            )
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
    value_bindings: &HashMap<String, String>,
    expr_bindings: &HashMap<String, sec4_core::ast::Expr>,
    depth: usize,
) -> Option<String> {
    if depth > 32 {
        return None;
    }
    match &expr.kind {
        sec4_core::ast::ExprKind::String(value) => Some(value.clone()),
        sec4_core::ast::ExprKind::Identifier(name) => {
            if let Some(value) = value_bindings.get(name) {
                return Some(value.clone());
            }
            let bound = expr_bindings.get(name)?;
            extract_header_gate_literal(
                bound,
                expected_gate,
                value_bindings,
                expr_bindings,
                depth + 1,
            )
        }
        sec4_core::ast::ExprKind::Call { callee, args } => {
            let resolved_callee = resolve_route_registration_expr(callee, expr_bindings, 0)
                .unwrap_or_else(|| callee.as_ref().clone());
            if expected_gate == "value" || expected_gate == "name" {
                if let Some(value) = parse_lasm_request_header_placeholder_call(
                    &resolved_callee,
                    args,
                    value_bindings,
                ) {
                    return Some(value);
                }
                if is_lasm_validate_non_empty_call(&resolved_callee) && !args.is_empty() {
                    return extract_header_gate_literal(
                        &args[0],
                        expected_gate,
                        value_bindings,
                        expr_bindings,
                        depth + 1,
                    );
                }
                if expected_gate == "value"
                    && is_lasm_validate_header_value_call(&resolved_callee)
                    && !args.is_empty()
                {
                    return extract_header_gate_literal(
                        &args[0],
                        expected_gate,
                        value_bindings,
                        expr_bindings,
                        depth + 1,
                    );
                }
            }
            let sec4_core::ast::ExprKind::Member { object, field } = &resolved_callee.kind else {
                return None;
            };
            let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind else {
                return None;
            };
            if namespace != "headers" || field != expected_gate || args.is_empty() {
                return None;
            }
            if expected_gate == "value" || expected_gate == "name" {
                return extract_header_gate_literal(
                    &args[0],
                    expected_gate,
                    value_bindings,
                    expr_bindings,
                    depth + 1,
                )
                .or_else(|| extract_lasm_request_header_placeholder(&args[0], value_bindings, 0));
            }
            let sec4_core::ast::ExprKind::String(value) = &args[0].kind else {
                return None;
            };
            Some(value.clone())
        }
        _ => None,
    }
}

fn extract_lasm_request_header_placeholder(
    expr: &sec4_core::ast::Expr,
    bindings: &HashMap<String, String>,
    depth: usize,
) -> Option<String> {
    if depth > 32 {
        return None;
    }
    match &expr.kind {
        sec4_core::ast::ExprKind::Identifier(name) => {
            let value = bindings.get(name)?;
            if contains_lasm_request_placeholder_tokens(value.as_str()) {
                Some(value.clone())
            } else {
                None
            }
        }
        sec4_core::ast::ExprKind::Call { callee, args } => {
            if let Some(value) = parse_lasm_request_header_placeholder_call(callee, args, bindings)
            {
                return Some(value);
            }
            if is_lasm_validate_non_empty_call(callee) && !args.is_empty() {
                return extract_lasm_request_header_placeholder(&args[0], bindings, depth + 1);
            }
            None
        }
        _ => None,
    }
}

fn parse_lasm_request_header_placeholder_call(
    callee: &sec4_core::ast::Expr,
    args: &[sec4_core::ast::Expr],
    bindings: &HashMap<String, String>,
) -> Option<String> {
    let sec4_core::ast::ExprKind::Member { object, field } = &callee.kind else {
        return None;
    };
    let sec4_core::ast::ExprKind::Identifier(namespace) = &object.kind else {
        return None;
    };
    if namespace != "req" {
        return None;
    }
    match field.as_str() {
        "method" if args.is_empty() => Some("{{req.method}}".to_string()),
        "path" if args.is_empty() => Some("{{req.path}}".to_string()),
        "httpVersion" if args.is_empty() => Some("{{req.httpVersion}}".to_string()),
        "body" => Some("{{req.body}}".to_string()),
        "pathParam" | "header" | "query" | "cookie" => {
            if args.is_empty() {
                return None;
            }
            let key = extract_string_literal_or_binding(&args[0], bindings)?;
            if key.trim().is_empty() {
                return None;
            }
            match field.as_str() {
                "pathParam" => Some(format!("{{{{req.pathParam:{key}}}}}")),
                "header" => Some(format!("{{{{req.header:{key}}}}}")),
                "query" => Some(format!("{{{{req.query:{key}}}}}")),
                "cookie" => Some(format!("{{{{req.cookie:{key}}}}}")),
                _ => None,
            }
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
    max_header_bytes: Option<u64>,
    max_body_bytes: Option<u64>,
    max_concurrency: Option<u64>,
    max_pending: Option<u64>,
    serve_timeout_ms: Option<u64>,
    overflow_probe_timeout_ms: Option<u64>,
    max_runtime_steps: Option<u64>,
    max_keep_alive_requests: Option<u64>,
    db_base: Option<&Path>,
    db_adapter: Option<RunDbAdapter>,
    db_postgres_dsn: Option<&str>,
    db_postgres_dsn_file: Option<&Path>,
    db_max_tx_handles: Option<u64>,
    instances: usize,
    autoscale_max_instances: Option<usize>,
    autoscale_target_connections: Option<usize>,
    autoscale_check_ms: u64,
    autoscale_scale_up_cooldown_ms: u64,
    autoscale_scale_down_cooldown_ms: u64,
    autoscale_scale_up_step: usize,
    autoscale_scale_down_step: usize,
    autoscale_saturation_boost_step: usize,
    cluster_relay_workers: Option<usize>,
    cluster_relay_queue: Option<usize>,
    cluster_accept_workers: Option<usize>,
    cluster_relay_accept_batch_max: Option<usize>,
    cluster_relay_pump_batch_max: Option<usize>,
    cluster_backend_connect_timeout_ms: Option<u64>,
    cluster_backend_connect_cooldown_ms: Option<u64>,
    cluster_status_json: Option<&Path>,
    reuse_port: bool,
    backend: RunBackend,
    tls_backend: BuildTlsBackend,
) -> Result<(), i32> {
    if instances == 0 {
        eprintln!("run failed: --instances must be >= 1");
        return Err(2);
    }
    if autoscale_max_instances == Some(0) {
        eprintln!("run failed: --autoscale-max-instances must be >= 1");
        return Err(2);
    }
    if autoscale_target_connections == Some(0) {
        eprintln!("run failed: --autoscale-target-connections must be >= 1");
        return Err(2);
    }
    if autoscale_check_ms == 0 {
        eprintln!("run failed: --autoscale-check-ms must be >= 1");
        return Err(2);
    }
    if autoscale_scale_up_cooldown_ms == 0 {
        eprintln!("run failed: --autoscale-scale-up-cooldown-ms must be >= 1");
        return Err(2);
    }
    if autoscale_scale_down_cooldown_ms == 0 {
        eprintln!("run failed: --autoscale-scale-down-cooldown-ms must be >= 1");
        return Err(2);
    }
    if autoscale_scale_up_step == 0 {
        eprintln!("run failed: --autoscale-scale-up-step must be >= 1");
        return Err(2);
    }
    if autoscale_scale_down_step == 0 {
        eprintln!("run failed: --autoscale-scale-down-step must be >= 1");
        return Err(2);
    }
    if autoscale_saturation_boost_step == 0 {
        eprintln!("run failed: --autoscale-saturation-boost-step must be >= 1");
        return Err(2);
    }
    if cluster_relay_workers == Some(0) {
        eprintln!("run failed: --cluster-relay-workers must be >= 1");
        return Err(2);
    }
    if cluster_relay_queue == Some(0) {
        eprintln!("run failed: --cluster-relay-queue must be >= 1");
        return Err(2);
    }
    if cluster_accept_workers == Some(0) {
        eprintln!("run failed: --cluster-accept-workers must be >= 1");
        return Err(2);
    }
    if cluster_relay_accept_batch_max == Some(0) {
        eprintln!("run failed: --cluster-relay-accept-batch-max must be >= 1");
        return Err(2);
    }
    if cluster_relay_pump_batch_max == Some(0) {
        eprintln!("run failed: --cluster-relay-pump-batch-max must be >= 1");
        return Err(2);
    }
    if cluster_backend_connect_timeout_ms == Some(0) {
        eprintln!("run failed: --cluster-backend-connect-timeout-ms must be >= 1");
        return Err(2);
    }
    if cluster_backend_connect_cooldown_ms == Some(0) {
        eprintln!("run failed: --cluster-backend-connect-cooldown-ms must be >= 1");
        return Err(2);
    }
    if max_header_bytes == Some(0) {
        eprintln!("run failed: --max-header-bytes must be >= 1");
        return Err(2);
    }
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
    if overflow_probe_timeout_ms == Some(0) {
        eprintln!("run failed: --overflow-probe-timeout-ms must be >= 1");
        return Err(2);
    }
    if max_runtime_steps == Some(0) {
        eprintln!("run failed: --max-runtime-steps must be >= 1");
        return Err(2);
    }
    if max_keep_alive_requests == Some(0) {
        eprintln!("run failed: --max-keep-alive-requests must be >= 1");
        return Err(2);
    }
    if backend != RunBackend::Lasm && max_runtime_steps.is_some() {
        eprintln!("run failed: --max-runtime-steps is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && max_pending.is_some() {
        eprintln!("run failed: --max-pending is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && max_keep_alive_requests.is_some() {
        eprintln!("run failed: --max-keep-alive-requests is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && overflow_probe_timeout_ms.is_some() {
        eprintln!("run failed: --overflow-probe-timeout-ms is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && db_base.is_some() {
        eprintln!("run failed: --db-base is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && db_adapter.is_some() {
        eprintln!("run failed: --db-adapter is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && db_postgres_dsn.is_some() {
        eprintln!("run failed: --db-postgres-dsn is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && db_postgres_dsn_file.is_some() {
        eprintln!("run failed: --db-postgres-dsn-file is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && db_max_tx_handles.is_some() {
        eprintln!("run failed: --db-max-tx-handles is only supported with --backend lasm");
        return Err(2);
    }
    if db_max_tx_handles == Some(0) {
        eprintln!("run failed: --db-max-tx-handles must be >= 1");
        return Err(2);
    }
    if db_postgres_dsn.is_some() && db_postgres_dsn_file.is_some() {
        eprintln!("run failed: use only one of --db-postgres-dsn or --db-postgres-dsn-file");
        return Err(2);
    }
    let has_explicit_postgres_dsn = db_postgres_dsn.is_some() || db_postgres_dsn_file.is_some();
    let effective_db_adapter = if backend == RunBackend::Lasm && has_explicit_postgres_dsn {
        match db_adapter {
            Some(RunDbAdapter::Postgres) => Some(RunDbAdapter::Postgres),
            Some(_) => {
                eprintln!(
                    "run failed: --db-postgres-dsn and --db-postgres-dsn-file require --db-adapter postgres when adapter is set explicitly"
                );
                return Err(2);
            }
            None => Some(RunDbAdapter::Postgres),
        }
    } else {
        db_adapter
    };
    let explicit_db_postgres_dsn = if let Some(dsn) = db_postgres_dsn {
        let dsn = dsn.trim();
        if dsn.is_empty() {
            eprintln!("run failed: --db-postgres-dsn must not be empty");
            return Err(2);
        }
        Some(dsn.to_string())
    } else if let Some(path) = db_postgres_dsn_file {
        match load_lasm_db_postgres_dsn_from_file(path) {
            Ok(dsn) => Some(dsn),
            Err(message) => {
                eprintln!("run failed: {message}");
                return Err(2);
            }
        }
    } else {
        None
    };
    if backend != RunBackend::Lasm && instances != 1 {
        eprintln!("run failed: --instances is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && autoscale_max_instances.is_some() {
        eprintln!("run failed: --autoscale-max-instances is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && autoscale_target_connections.is_some() {
        eprintln!(
            "run failed: --autoscale-target-connections is only supported with --backend lasm"
        );
        return Err(2);
    }
    if backend != RunBackend::Lasm && autoscale_check_ms != 1000 {
        eprintln!("run failed: --autoscale-check-ms is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && autoscale_scale_up_cooldown_ms != 250 {
        eprintln!(
            "run failed: --autoscale-scale-up-cooldown-ms is only supported with --backend lasm"
        );
        return Err(2);
    }
    if backend != RunBackend::Lasm && autoscale_scale_down_cooldown_ms != 2000 {
        eprintln!(
            "run failed: --autoscale-scale-down-cooldown-ms is only supported with --backend lasm"
        );
        return Err(2);
    }
    if backend != RunBackend::Lasm && autoscale_scale_up_step != 2 {
        eprintln!("run failed: --autoscale-scale-up-step is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && autoscale_scale_down_step != 1 {
        eprintln!("run failed: --autoscale-scale-down-step is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && autoscale_saturation_boost_step != 4 {
        eprintln!(
            "run failed: --autoscale-saturation-boost-step is only supported with --backend lasm"
        );
        return Err(2);
    }
    if backend != RunBackend::Lasm && cluster_relay_workers.is_some() {
        eprintln!("run failed: --cluster-relay-workers is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && cluster_relay_queue.is_some() {
        eprintln!("run failed: --cluster-relay-queue is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && cluster_accept_workers.is_some() {
        eprintln!("run failed: --cluster-accept-workers is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && cluster_relay_accept_batch_max.is_some() {
        eprintln!(
            "run failed: --cluster-relay-accept-batch-max is only supported with --backend lasm"
        );
        return Err(2);
    }
    if backend != RunBackend::Lasm && cluster_relay_pump_batch_max.is_some() {
        eprintln!(
            "run failed: --cluster-relay-pump-batch-max is only supported with --backend lasm"
        );
        return Err(2);
    }
    if backend != RunBackend::Lasm && cluster_backend_connect_timeout_ms.is_some() {
        eprintln!(
            "run failed: --cluster-backend-connect-timeout-ms is only supported with --backend lasm"
        );
        return Err(2);
    }
    if backend != RunBackend::Lasm && cluster_backend_connect_cooldown_ms.is_some() {
        eprintln!(
            "run failed: --cluster-backend-connect-cooldown-ms is only supported with --backend lasm"
        );
        return Err(2);
    }
    if backend != RunBackend::Lasm && cluster_status_json.is_some() {
        eprintln!("run failed: --cluster-status-json is only supported with --backend lasm");
        return Err(2);
    }
    if backend != RunBackend::Lasm && reuse_port {
        eprintln!("run failed: --reuse-port is only supported with --backend lasm");
        return Err(2);
    }
    let max_instances = autoscale_max_instances.unwrap_or(instances);
    let cluster_mode = max_instances > 1 || instances > 1;
    if backend == RunBackend::Lasm && !cluster_mode && cluster_relay_workers.is_some() {
        eprintln!("run failed: --cluster-relay-workers requires cluster mode (--instances > 1)");
        return Err(2);
    }
    if backend == RunBackend::Lasm && !cluster_mode && cluster_relay_queue.is_some() {
        eprintln!("run failed: --cluster-relay-queue requires cluster mode (--instances > 1)");
        return Err(2);
    }
    if backend == RunBackend::Lasm && !cluster_mode && cluster_accept_workers.is_some() {
        eprintln!("run failed: --cluster-accept-workers requires cluster mode (--instances > 1)");
        return Err(2);
    }
    if backend == RunBackend::Lasm && !cluster_mode && cluster_relay_accept_batch_max.is_some() {
        eprintln!(
            "run failed: --cluster-relay-accept-batch-max requires cluster mode (--instances > 1)"
        );
        return Err(2);
    }
    if backend == RunBackend::Lasm && !cluster_mode && cluster_relay_pump_batch_max.is_some() {
        eprintln!(
            "run failed: --cluster-relay-pump-batch-max requires cluster mode (--instances > 1)"
        );
        return Err(2);
    }
    if backend == RunBackend::Lasm && !cluster_mode && cluster_backend_connect_timeout_ms.is_some()
    {
        eprintln!(
            "run failed: --cluster-backend-connect-timeout-ms requires cluster mode (--instances > 1)"
        );
        return Err(2);
    }
    if backend == RunBackend::Lasm && !cluster_mode && cluster_backend_connect_cooldown_ms.is_some()
    {
        eprintln!(
            "run failed: --cluster-backend-connect-cooldown-ms requires cluster mode (--instances > 1)"
        );
        return Err(2);
    }
    if backend == RunBackend::Lasm && !cluster_mode && cluster_status_json.is_some() {
        eprintln!("run failed: --cluster-status-json requires cluster mode (--instances > 1)");
        return Err(2);
    }
    if max_instances < instances {
        eprintln!("run failed: --autoscale-max-instances must be >= --instances");
        return Err(2);
    }
    if backend == RunBackend::Lasm && oneshot && max_instances > 1 {
        eprintln!("run failed: cluster mode does not support --oneshot");
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
            max_header_bytes,
            max_body_bytes,
            max_concurrency,
            max_pending,
            serve_timeout_ms,
            overflow_probe_timeout_ms,
            max_runtime_steps,
            max_keep_alive_requests,
            db_base,
            effective_db_adapter,
            explicit_db_postgres_dsn.as_deref(),
            db_max_tx_handles,
            instances,
            autoscale_max_instances,
            autoscale_target_connections,
            autoscale_check_ms,
            autoscale_scale_up_cooldown_ms,
            autoscale_scale_down_cooldown_ms,
            autoscale_scale_up_step,
            autoscale_scale_down_step,
            autoscale_saturation_boost_step,
            cluster_relay_workers,
            cluster_relay_queue,
            cluster_accept_workers,
            cluster_relay_accept_batch_max,
            cluster_relay_pump_batch_max,
            cluster_backend_connect_timeout_ms,
            cluster_backend_connect_cooldown_ms,
            cluster_status_json,
            reuse_port,
        );
    }

    let policy_max_header_bytes = match u64::try_from(policy.http.max_header_bytes) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: policy http.max_header_bytes must be >= 0");
            return Err(2);
        }
    };
    let effective_max_header_bytes = max_header_bytes.unwrap_or(policy_max_header_bytes);

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
        effective_max_header_bytes.to_string(),
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

#[derive(Clone, Debug)]
struct LasmClusterConfig {
    path: PathBuf,
    listen_port: u16,
    max_header_bytes: Option<u64>,
    max_body_bytes: Option<u64>,
    max_concurrency: Option<u64>,
    max_pending: Option<u64>,
    serve_timeout_ms: Option<u64>,
    overflow_probe_timeout_ms: Option<u64>,
    max_runtime_steps: Option<u64>,
    max_keep_alive_requests: Option<u64>,
    db_base: Option<PathBuf>,
    db_adapter: Option<RunDbAdapter>,
    db_postgres_dsn: Option<String>,
    db_max_tx_handles: Option<u64>,
    min_instances: usize,
    max_instances: usize,
    target_connections_per_instance: usize,
    autoscale_check_ms: u64,
    autoscale_scale_up_cooldown_ms: u64,
    autoscale_scale_down_cooldown_ms: u64,
    autoscale_scale_up_step: usize,
    autoscale_scale_down_step: usize,
    autoscale_saturation_boost_step: usize,
    worker_ready_timeout_ms: u64,
    cluster_relay_workers: Option<usize>,
    cluster_relay_queue: Option<usize>,
    cluster_accept_workers: Option<usize>,
    cluster_relay_accept_batch_max: usize,
    cluster_relay_pump_batch_max: usize,
    cluster_backend_connect_timeout_ms: u64,
    cluster_backend_connect_cooldown_ms: u64,
    cluster_status_json: Option<PathBuf>,
    reuse_port_workers: bool,
}

#[derive(Debug)]
struct LasmClusterWorker {
    port: u16,
    child: Child,
}

#[derive(Debug)]
struct LasmClusterState {
    workers: Vec<LasmClusterWorker>,
    next_port: u16,
}

fn compute_lasm_cluster_base_port(listen_port: u16, max_instances: usize) -> Result<u16, String> {
    let base_port = u32::from(listen_port) + 100;
    let needed_span = u32::try_from(max_instances.saturating_sub(1))
        .map_err(|_| "cluster instance count exceeds supported port span".to_string())?;
    let last_port = base_port + needed_span;
    if last_port > u32::from(u16::MAX) {
        return Err(
            "cannot allocate worker ports: choose a lower --port for cluster mode".to_string(),
        );
    }
    Ok(u16::try_from(base_port).expect("base port range prevalidated"))
}

fn push_optional_u64_run_arg(cmd: &mut Command, flag: &str, value: Option<u64>) {
    if let Some(value) = value {
        cmd.arg(flag).arg(value.to_string());
    }
}

fn push_optional_path_run_arg(cmd: &mut Command, flag: &str, value: Option<&Path>) {
    if let Some(value) = value {
        cmd.arg(flag).arg(value);
    }
}

fn run_db_adapter_arg_value(adapter: RunDbAdapter) -> &'static str {
    match adapter {
        RunDbAdapter::RecordsLog => "records-log",
        RunDbAdapter::Sqlite => "sqlite",
        RunDbAdapter::Postgres => "postgres",
    }
}

fn run_db_adapter_to_lasm_db_records_adapter(adapter: RunDbAdapter) -> LasmDbRecordsAdapter {
    match adapter {
        RunDbAdapter::RecordsLog => LasmDbRecordsAdapter::RecordsLog,
        RunDbAdapter::Sqlite => LasmDbRecordsAdapter::Sqlite,
        RunDbAdapter::Postgres => LasmDbRecordsAdapter::Postgres,
    }
}

fn push_optional_db_adapter_run_arg(cmd: &mut Command, value: Option<RunDbAdapter>) {
    if let Some(value) = value {
        cmd.arg("--db-adapter").arg(run_db_adapter_arg_value(value));
    }
}

fn spawn_lasm_cluster_worker(
    config: &LasmClusterConfig,
    worker_port: u16,
) -> Result<Child, String> {
    let current_exe = std::env::current_exe().map_err(|err| {
        format!("could not resolve current sec4 executable for cluster mode: {err}")
    })?;
    let mut cmd = Command::new(current_exe);
    cmd.arg("run")
        .arg("--path")
        .arg(&config.path)
        .arg("--backend")
        .arg("lasm")
        .arg("--port")
        .arg(worker_port.to_string())
        .arg("--instances")
        .arg("1");
    if config.reuse_port_workers {
        cmd.arg("--reuse-port");
    }

    push_optional_u64_run_arg(&mut cmd, "--max-header-bytes", config.max_header_bytes);
    push_optional_u64_run_arg(&mut cmd, "--max-body-bytes", config.max_body_bytes);
    push_optional_u64_run_arg(&mut cmd, "--max-concurrency", config.max_concurrency);
    push_optional_u64_run_arg(&mut cmd, "--max-pending", config.max_pending);
    push_optional_u64_run_arg(&mut cmd, "--serve-timeout-ms", config.serve_timeout_ms);
    push_optional_u64_run_arg(
        &mut cmd,
        "--overflow-probe-timeout-ms",
        config.overflow_probe_timeout_ms,
    );
    push_optional_u64_run_arg(&mut cmd, "--max-runtime-steps", config.max_runtime_steps);
    push_optional_u64_run_arg(
        &mut cmd,
        "--max-keep-alive-requests",
        config.max_keep_alive_requests,
    );
    push_optional_path_run_arg(&mut cmd, "--db-base", config.db_base.as_deref());
    push_optional_db_adapter_run_arg(&mut cmd, config.db_adapter);
    push_optional_u64_run_arg(&mut cmd, "--db-max-tx-handles", config.db_max_tx_handles);
    if let Some(dsn) = config.db_postgres_dsn.as_deref() {
        cmd.env("SEC4_RT_LASM_DB_POSTGRES_DSN", dsn);
    }

    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    cmd.spawn()
        .map_err(|err| format!("could not spawn LASM cluster worker on port {worker_port}: {err}"))
}

fn wait_for_lasm_cluster_worker_ready(
    child: &mut Child,
    worker_port: u16,
    timeout_ms: u64,
) -> Result<(), String> {
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms.max(200));
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!(
                    "LASM cluster worker on port {worker_port} exited early with status {status}"
                ));
            }
            Ok(None) => {}
            Err(err) => {
                return Err(format!(
                    "could not check LASM cluster worker status on port {worker_port}: {err}"
                ));
            }
        }

        match TcpStream::connect(("127.0.0.1", worker_port)) {
            Ok(stream) => {
                let _ = stream.shutdown(Shutdown::Both);
                return Ok(());
            }
            Err(_) => {
                if start.elapsed() >= timeout {
                    return Err(format!(
                        "LASM cluster worker on port {worker_port} did not become ready within {} ms",
                        timeout.as_millis()
                    ));
                }
                std::thread::sleep(Duration::from_millis(40));
            }
        }
    }
}

fn wait_for_lasm_cluster_worker_alive(
    child: &mut Child,
    worker_port: u16,
    timeout_ms: u64,
) -> Result<(), String> {
    let grace_ms = timeout_ms.clamp(200, 2000);
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!(
                    "LASM cluster worker on port {worker_port} exited early with status {status}"
                ));
            }
            Ok(None) => {
                if start.elapsed() >= Duration::from_millis(grace_ms) {
                    return Ok(());
                }
            }
            Err(err) => {
                return Err(format!(
                    "could not check LASM cluster worker status on port {worker_port}: {err}"
                ));
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn spawn_and_wait_lasm_cluster_worker(
    config: &LasmClusterConfig,
    worker_port: u16,
) -> Result<LasmClusterWorker, String> {
    let mut child = spawn_lasm_cluster_worker(config, worker_port)?;
    let wait_result = if config.reuse_port_workers {
        wait_for_lasm_cluster_worker_alive(&mut child, worker_port, config.worker_ready_timeout_ms)
    } else {
        wait_for_lasm_cluster_worker_ready(&mut child, worker_port, config.worker_ready_timeout_ms)
    };
    if let Err(err) = wait_result {
        let _ = child.kill();
        let _ = child.wait();
        return Err(err);
    }
    Ok(LasmClusterWorker {
        port: worker_port,
        child,
    })
}

fn prune_dead_lasm_cluster_workers(state: &mut LasmClusterState) {
    let mut kept = Vec::with_capacity(state.workers.len());
    for mut worker in state.workers.drain(..) {
        match worker.child.try_wait() {
            Ok(Some(status)) => {
                eprintln!(
                    "warning: LASM cluster worker on port {} exited: {}",
                    worker.port, status
                );
            }
            Ok(None) => kept.push(worker),
            Err(err) => {
                eprintln!(
                    "warning: could not inspect LASM cluster worker on port {}: {}",
                    worker.port, err
                );
            }
        }
    }
    state.workers = kept;
}

fn recover_lasm_cluster_min_workers(
    state: &mut LasmClusterState,
    config: &LasmClusterConfig,
    warning_label: &str,
) {
    while state.workers.len() < config.min_instances {
        let worker_port = state.next_port;
        state.next_port = state.next_port.saturating_add(1);
        match spawn_and_wait_lasm_cluster_worker(config, worker_port) {
            Ok(worker) => state.workers.push(worker),
            Err(message) => {
                eprintln!("warning: LASM cluster {warning_label} failed: {message}");
                break;
            }
        }
    }
}

fn lasm_cluster_maintenance_interval_ms(config: &LasmClusterConfig) -> u64 {
    config.autoscale_check_ms.clamp(100, 500)
}

fn lasm_cluster_backend_connect_timeout(config: &LasmClusterConfig) -> Duration {
    Duration::from_millis(config.cluster_backend_connect_timeout_ms.max(25))
}

fn lasm_cluster_backend_connect_cooldown(config: &LasmClusterConfig) -> Duration {
    Duration::from_millis(config.cluster_backend_connect_cooldown_ms.max(25))
}

fn refresh_lasm_cluster_worker_ports_snapshot_if_changed(
    state: &LasmClusterState,
    snapshot: &Arc<ArcSwap<Vec<u16>>>,
    last_published_ports: &mut Vec<u16>,
) {
    if state.workers.len() == last_published_ports.len()
        && state
            .workers
            .iter()
            .zip(last_published_ports.iter())
            .all(|(worker, port)| worker.port == *port)
    {
        return;
    }

    last_published_ports.clear();
    last_published_ports.extend(state.workers.iter().map(|worker| worker.port));
    if last_published_ports
        .windows(2)
        .any(|window| window[0] > window[1])
    {
        last_published_ports.sort_unstable();
    }
    snapshot.store(Arc::new(last_published_ports.clone()));
}

fn lasm_cluster_remaining_cooldown_ms(
    now: Instant,
    last_at: Option<Instant>,
    cooldown_ms: u64,
) -> u64 {
    let Some(last_at) = last_at else {
        return 0;
    };
    let elapsed_ms = now
        .duration_since(last_at)
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;
    cooldown_ms.saturating_sub(elapsed_ms)
}

fn desired_lasm_cluster_instances(
    active_connections: usize,
    min_instances: usize,
    max_instances: usize,
    target_connections_per_instance: usize,
) -> usize {
    let needed = if active_connections == 0 {
        min_instances
    } else {
        active_connections.saturating_add(target_connections_per_instance.saturating_sub(1))
            / target_connections_per_instance
    };
    needed.clamp(min_instances, max_instances)
}

fn lasm_cluster_proxy_worker_count(config: &LasmClusterConfig) -> usize {
    if let Some(value) = config.cluster_relay_workers {
        return value;
    }
    let host_parallelism = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(4)
        .max(1);
    let instance_hint = config.max_instances.max(config.min_instances).max(1);
    let mut relay_hint = if instance_hint <= 1 {
        1
    } else {
        // Keep auto relay sizing conservative in multi-instance mode:
        // short capacity probes show lower relay-thread counts reduce contention
        // versus ceil(sqrt(...)) defaults under current cluster topology.
        ((instance_hint as f64).sqrt() as usize).max(2)
    };
    relay_hint = relay_hint.min(host_parallelism);
    relay_hint.clamp(1, 16)
}

fn lasm_cluster_proxy_queue_capacity(config: &LasmClusterConfig, worker_count: usize) -> usize {
    if let Some(value) = config.cluster_relay_queue {
        return value;
    }
    config
        .target_connections_per_instance
        .max(1)
        .saturating_mul(config.max_instances.max(1))
        .max(worker_count.saturating_mul(2))
        .min(65_536)
}

fn lasm_cluster_accept_worker_count(
    config: &LasmClusterConfig,
    relay_worker_count: usize,
) -> usize {
    if let Some(value) = config.cluster_accept_workers {
        return value;
    }
    let default_value = relay_worker_count.max(1).min(4);
    let Ok(raw) = std::env::var("SEC4_RT_LASM_CLUSTER_ACCEPT_WORKERS") else {
        return default_value;
    };
    let value = raw.trim();
    if value.is_empty() {
        return default_value;
    }
    value
        .parse::<usize>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .map(|parsed| parsed.clamp(1, relay_worker_count.max(1).min(16)))
        .unwrap_or(default_value)
}

fn lasm_cluster_relay_accept_batch_max(config: &LasmClusterConfig) -> usize {
    config.cluster_relay_accept_batch_max.max(1)
}

fn lasm_cluster_relay_pump_batch_max(config: &LasmClusterConfig) -> usize {
    config.cluster_relay_pump_batch_max.max(1)
}

enum LasmClusterUnavailableReason {
    NoHealthyWorkers,
    WorkerUnavailable,
    RelaySaturated,
    RelayUnavailable,
}

const LASM_CLUSTER_UNAVAILABLE_NO_HEALTHY_WORKERS_RESPONSE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: 54\r\nConnection: close\r\n\r\n{\"ok\":false,\"status\":503,\"error\":\"no healthy workers\"}";
const LASM_CLUSTER_UNAVAILABLE_WORKER_UNAVAILABLE_RESPONSE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: 54\r\nConnection: close\r\n\r\n{\"ok\":false,\"status\":503,\"error\":\"worker unavailable\"}";
const LASM_CLUSTER_UNAVAILABLE_RELAY_SATURATED_RESPONSE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: 59\r\nConnection: close\r\n\r\n{\"ok\":false,\"status\":503,\"error\":\"cluster relay saturated\"}";
const LASM_CLUSTER_UNAVAILABLE_RELAY_UNAVAILABLE_RESPONSE: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: 61\r\nConnection: close\r\n\r\n{\"ok\":false,\"status\":503,\"error\":\"cluster relay unavailable\"}";

fn lasm_cluster_unavailable_response(reason: LasmClusterUnavailableReason) -> &'static [u8] {
    match reason {
        LasmClusterUnavailableReason::NoHealthyWorkers => {
            LASM_CLUSTER_UNAVAILABLE_NO_HEALTHY_WORKERS_RESPONSE
        }
        LasmClusterUnavailableReason::WorkerUnavailable => {
            LASM_CLUSTER_UNAVAILABLE_WORKER_UNAVAILABLE_RESPONSE
        }
        LasmClusterUnavailableReason::RelaySaturated => {
            LASM_CLUSTER_UNAVAILABLE_RELAY_SATURATED_RESPONSE
        }
        LasmClusterUnavailableReason::RelayUnavailable => {
            LASM_CLUSTER_UNAVAILABLE_RELAY_UNAVAILABLE_RESPONSE
        }
    }
}

fn write_lasm_cluster_unavailable_response(
    client: &mut TcpStream,
    reason: LasmClusterUnavailableReason,
) -> Result<(), String> {
    let response = lasm_cluster_unavailable_response(reason);
    client
        .write_all(response)
        .map_err(|err| format!("could not write LASM cluster overload response: {err}"))
}

#[derive(Clone, Debug)]
struct LasmClusterStatusSnapshot {
    listen_port: u16,
    min_instances: usize,
    max_instances: usize,
    worker_count: usize,
    relay_worker_count: usize,
    relay_queue_capacity: usize,
    relay_queue_shard_capacity: usize,
    worker_ports: Arc<Vec<u16>>,
    active_connections: usize,
    active_connections_per_worker: f64,
    relay_saturation_events_pending: usize,
    relay_saturation_events_total: u64,
    relay_saturation_events_per_sec: f64,
    relay_accept_batch_max: usize,
    relay_pump_batch_max: usize,
    relay_accept_workers: usize,
    relay_backend_connect_timeout_ms: u64,
    relay_backend_connect_cooldown_ms: u64,
    relay_dispatch_fallback_total: u64,
    relay_dispatch_fallback_per_sec: f64,
    relay_dispatch_saturation_short_circuit_total: u64,
    relay_dispatch_saturation_short_circuit_per_sec: f64,
    relay_live_sender_count: usize,
    autoscale_desired_instances: usize,
    autoscale_last_saturation_events: usize,
    autoscale_last_dynamic_boost_step: usize,
    autoscale_scale_up_cooldown_remaining_ms: u64,
    autoscale_scale_down_cooldown_remaining_ms: u64,
}

impl PartialEq for LasmClusterStatusSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.listen_port == other.listen_port
            && self.min_instances == other.min_instances
            && self.max_instances == other.max_instances
            && self.worker_count == other.worker_count
            && self.relay_worker_count == other.relay_worker_count
            && self.relay_queue_capacity == other.relay_queue_capacity
            && self.relay_queue_shard_capacity == other.relay_queue_shard_capacity
            && (Arc::ptr_eq(&self.worker_ports, &other.worker_ports)
                || self.worker_ports.as_slice() == other.worker_ports.as_slice())
            && self.active_connections == other.active_connections
            && self.active_connections_per_worker == other.active_connections_per_worker
            && self.relay_saturation_events_pending == other.relay_saturation_events_pending
            && self.relay_saturation_events_total == other.relay_saturation_events_total
            && self.relay_saturation_events_per_sec == other.relay_saturation_events_per_sec
            && self.relay_accept_batch_max == other.relay_accept_batch_max
            && self.relay_pump_batch_max == other.relay_pump_batch_max
            && self.relay_accept_workers == other.relay_accept_workers
            && self.relay_backend_connect_timeout_ms == other.relay_backend_connect_timeout_ms
            && self.relay_backend_connect_cooldown_ms == other.relay_backend_connect_cooldown_ms
            && self.relay_dispatch_fallback_total == other.relay_dispatch_fallback_total
            && self.relay_dispatch_fallback_per_sec == other.relay_dispatch_fallback_per_sec
            && self.relay_dispatch_saturation_short_circuit_total
                == other.relay_dispatch_saturation_short_circuit_total
            && self.relay_dispatch_saturation_short_circuit_per_sec
                == other.relay_dispatch_saturation_short_circuit_per_sec
            && self.relay_live_sender_count == other.relay_live_sender_count
            && self.autoscale_desired_instances == other.autoscale_desired_instances
            && self.autoscale_last_saturation_events == other.autoscale_last_saturation_events
            && self.autoscale_last_dynamic_boost_step == other.autoscale_last_dynamic_boost_step
            && self.autoscale_scale_up_cooldown_remaining_ms
                == other.autoscale_scale_up_cooldown_remaining_ms
            && self.autoscale_scale_down_cooldown_remaining_ms
                == other.autoscale_scale_down_cooldown_remaining_ms
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LasmClusterStatusPayload<'a> {
    mode: &'static str,
    updated_at_ms: u64,
    listen_port: u16,
    min_instances: usize,
    max_instances: usize,
    worker_count: usize,
    relay_worker_count: usize,
    relay_queue_capacity: usize,
    relay_queue_shard_capacity: usize,
    worker_ports: &'a [u16],
    active_connections: usize,
    active_connections_per_worker: f64,
    relay_saturation_events_pending: usize,
    relay_saturation_events_total: u64,
    relay_saturation_events_per_sec: f64,
    relay_accept_batch_max: usize,
    relay_pump_batch_max: usize,
    relay_accept_workers: usize,
    relay_backend_connect_timeout_ms: u64,
    relay_backend_connect_cooldown_ms: u64,
    relay_dispatch_fallback_total: u64,
    relay_dispatch_fallback_per_sec: f64,
    relay_dispatch_saturation_short_circuit_total: u64,
    relay_dispatch_saturation_short_circuit_per_sec: f64,
    relay_live_sender_count: usize,
    autoscale_desired_instances: usize,
    autoscale_last_saturation_events: usize,
    autoscale_last_dynamic_boost_step: usize,
    autoscale_scale_up_cooldown_remaining_ms: u64,
    autoscale_scale_down_cooldown_remaining_ms: u64,
}

fn write_lasm_cluster_status_json(
    path: &Path,
    tmp_path: &Path,
    snapshot: LasmClusterStatusSnapshot,
    last_snapshot: &mut Option<LasmClusterStatusSnapshot>,
    status_parent_ready: &mut bool,
) -> Result<(), String> {
    if last_snapshot
        .as_ref()
        .map(|previous| previous == &snapshot)
        .unwrap_or(false)
    {
        return Ok(());
    }

    let ensure_status_parent_dir = |ready: &mut bool| -> Result<(), String> {
        if *ready {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|err| {
                    format!(
                        "could not create cluster status json parent directory {}: {err}",
                        parent.display()
                    )
                })?;
            }
        }
        *ready = true;
        Ok(())
    };
    ensure_status_parent_dir(status_parent_ready)?;

    let payload = LasmClusterStatusPayload {
        mode: "lasm-cluster",
        updated_at_ms: lasm_now_ms(),
        listen_port: snapshot.listen_port,
        min_instances: snapshot.min_instances,
        max_instances: snapshot.max_instances,
        worker_count: snapshot.worker_count,
        relay_worker_count: snapshot.relay_worker_count,
        relay_queue_capacity: snapshot.relay_queue_capacity,
        relay_queue_shard_capacity: snapshot.relay_queue_shard_capacity,
        worker_ports: snapshot.worker_ports.as_slice(),
        active_connections: snapshot.active_connections,
        active_connections_per_worker: snapshot.active_connections_per_worker,
        relay_saturation_events_pending: snapshot.relay_saturation_events_pending,
        relay_saturation_events_total: snapshot.relay_saturation_events_total,
        relay_saturation_events_per_sec: snapshot.relay_saturation_events_per_sec,
        relay_accept_batch_max: snapshot.relay_accept_batch_max,
        relay_pump_batch_max: snapshot.relay_pump_batch_max,
        relay_accept_workers: snapshot.relay_accept_workers,
        relay_backend_connect_timeout_ms: snapshot.relay_backend_connect_timeout_ms,
        relay_backend_connect_cooldown_ms: snapshot.relay_backend_connect_cooldown_ms,
        relay_dispatch_fallback_total: snapshot.relay_dispatch_fallback_total,
        relay_dispatch_fallback_per_sec: snapshot.relay_dispatch_fallback_per_sec,
        relay_dispatch_saturation_short_circuit_total: snapshot
            .relay_dispatch_saturation_short_circuit_total,
        relay_dispatch_saturation_short_circuit_per_sec: snapshot
            .relay_dispatch_saturation_short_circuit_per_sec,
        relay_live_sender_count: snapshot.relay_live_sender_count,
        autoscale_desired_instances: snapshot.autoscale_desired_instances,
        autoscale_last_saturation_events: snapshot.autoscale_last_saturation_events,
        autoscale_last_dynamic_boost_step: snapshot.autoscale_last_dynamic_boost_step,
        autoscale_scale_up_cooldown_remaining_ms: snapshot.autoscale_scale_up_cooldown_remaining_ms,
        autoscale_scale_down_cooldown_remaining_ms: snapshot
            .autoscale_scale_down_cooldown_remaining_ms,
    };
    let tmp_file = match fs::File::create(tmp_path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            *status_parent_ready = false;
            ensure_status_parent_dir(status_parent_ready)?;
            fs::File::create(tmp_path).map_err(|retry_err| {
                format!(
                    "could not create cluster status json temporary file {}: {retry_err}",
                    tmp_path.display()
                )
            })?
        }
        Err(err) => {
            return Err(format!(
                "could not create cluster status json temporary file {}: {err}",
                tmp_path.display()
            ));
        }
    };
    let mut tmp_writer = BufWriter::new(tmp_file);
    serde_json::to_writer(&mut tmp_writer, &payload)
        .map_err(|err| format!("could not encode cluster status json payload: {err}"))?;
    tmp_writer.flush().map_err(|err| {
        format!(
            "could not flush cluster status json temporary file {}: {err}",
            tmp_path.display()
        )
    })?;
    fs::rename(tmp_path, path).map_err(|err| {
        format!(
            "could not move cluster status json temporary file {} to {}: {err}",
            tmp_path.display(),
            path.display()
        )
    })?;
    *last_snapshot = Some(snapshot);
    Ok(())
}

const LASM_CLUSTER_RELAY_BUFFER_BYTES: usize = 16 * 1024;
const LASM_CLUSTER_RELAY_WARNING_THROTTLE_MS: u64 = 1000;
const LASM_CLUSTER_SATURATION_COUNTER_FLUSH_BATCH: usize = 8;
const LASM_CLUSTER_UNHEALTHY_PRUNE_INTERVAL_MS: u64 = 2;
const LASM_CLUSTER_IDLE_SPIN_THRESHOLD: u32 = 32;
const LASM_CLUSTER_IDLE_SLEEP_MICROS: u64 = 250;
const LASM_CLUSTER_SELECTION_RESERVATION_MIN_CHUNK: usize = 64;
const LASM_CLUSTER_RELAY_PUMP_BATCH_MULTIPLIER: usize = 4;
const LASM_CLUSTER_RELAY_PUMP_BATCH_MIN: usize = 64;
const LASM_CLUSTER_RELAY_PUMP_BATCH_MAX: usize = 4096;
const LASM_CLUSTER_RELAY_SENDER_LIVE: u8 = 1;
const LASM_CLUSTER_RELAY_SENDER_DEAD: u8 = 0;

#[inline(always)]
fn flush_lasm_cluster_saturation_counters(
    pending_counter: &AtomicUsize,
    total_counter: &AtomicU64,
    pending_local: &mut usize,
    total_local: &mut u64,
) {
    if *pending_local > 0 {
        pending_counter.fetch_add(*pending_local, Ordering::Relaxed);
        *pending_local = 0;
    }
    if *total_local > 0 {
        total_counter.fetch_add(*total_local, Ordering::Relaxed);
        *total_local = 0;
    }
}

#[inline(always)]
fn flush_lasm_cluster_dispatch_fallback_total(counter: &AtomicU64, total_local: &mut u64) {
    if *total_local > 0 {
        counter.fetch_add(*total_local, Ordering::Relaxed);
        *total_local = 0;
    }
}

#[inline(always)]
fn flush_lasm_cluster_dispatch_short_circuit_total(counter: &AtomicU64, total_local: &mut u64) {
    if *total_local > 0 {
        counter.fetch_add(*total_local, Ordering::Relaxed);
        *total_local = 0;
    }
}

#[inline(always)]
fn flush_lasm_cluster_active_connection_increments(
    active_counter: &AtomicUsize,
    increments_local: &mut usize,
) {
    if *increments_local > 0 {
        active_counter.fetch_add(*increments_local, Ordering::Relaxed);
        *increments_local = 0;
    }
}

#[inline(always)]
fn flush_lasm_cluster_active_connection_decrements(
    active_counter: &AtomicUsize,
    decrements_local: &mut usize,
) {
    if *decrements_local > 0 {
        active_counter.fetch_sub(*decrements_local, Ordering::Relaxed);
        *decrements_local = 0;
    }
}

enum LasmClusterRelayDispatchError {
    Saturated(TcpStream),
    Unavailable(TcpStream),
}

#[inline(always)]
fn lasm_cluster_next_index_wrapped(index: usize, count: usize) -> usize {
    debug_assert!(count > 0);
    if index + 1 == count {
        0
    } else {
        index + 1
    }
}

#[inline(always)]
fn lasm_cluster_next_live_sender_index(
    relay_sender_live: &[u8],
    start_index_wrapped: usize,
) -> Option<usize> {
    let sender_count = relay_sender_live.len();
    debug_assert!(sender_count > 0);
    debug_assert!(start_index_wrapped < sender_count);
    let mut scan_index = start_index_wrapped;
    for _ in 0..sender_count {
        if relay_sender_live[scan_index] == LASM_CLUSTER_RELAY_SENDER_LIVE {
            return Some(scan_index);
        }
        scan_index = lasm_cluster_next_index_wrapped(scan_index, sender_count);
    }
    None
}

#[inline(always)]
fn realign_lasm_cluster_dispatch_cursor_to_live(
    relay_sender_live: &[u8],
    relay_dispatch_cursor: &mut usize,
) -> bool {
    debug_assert!(!relay_sender_live.is_empty());
    if relay_sender_live[*relay_dispatch_cursor] == LASM_CLUSTER_RELAY_SENDER_LIVE {
        return true;
    }
    match lasm_cluster_next_live_sender_index(relay_sender_live, *relay_dispatch_cursor) {
        Some(index) => {
            *relay_dispatch_cursor = index;
            true
        }
        None => false,
    }
}

#[inline(always)]
fn refresh_lasm_cluster_single_live_sender_index(
    relay_sender_live: &[u8],
    relay_live_sender_count: usize,
    relay_single_live_sender_index: &mut Option<usize>,
) {
    if relay_live_sender_count != 1 {
        *relay_single_live_sender_index = None;
        return;
    }
    if let Some(index) = *relay_single_live_sender_index {
        if relay_sender_live[index] == LASM_CLUSTER_RELAY_SENDER_LIVE {
            return;
        }
    }
    *relay_single_live_sender_index = lasm_cluster_next_live_sender_index(relay_sender_live, 0);
}

#[inline(always)]
fn dispatch_lasm_cluster_relay_stream_fallback_multi(
    mut client_stream: TcpStream,
    relay_senders: &[Sender<TcpStream>],
    relay_sender_live: &mut [u8],
    relay_live_sender_count: &mut usize,
    relay_all_senders_live: &mut bool,
    start_index_wrapped: usize,
    mut saw_live_sender: bool,
) -> Result<(), LasmClusterRelayDispatchError> {
    let sender_count = relay_senders.len();
    debug_assert!(sender_count > 1);
    debug_assert_eq!(relay_sender_live.len(), sender_count);
    debug_assert!(start_index_wrapped < sender_count);
    if *relay_live_sender_count == 0 {
        return Err(LasmClusterRelayDispatchError::Unavailable(client_stream));
    }
    if sender_count == 2 {
        let alternate_index = start_index_wrapped;
        if relay_sender_live[alternate_index] == LASM_CLUSTER_RELAY_SENDER_LIVE {
            match relay_senders[alternate_index].try_send(client_stream) {
                Ok(()) => return Ok(()),
                Err(TrySendError::Full(next_stream)) => {
                    saw_live_sender = true;
                    client_stream = next_stream;
                }
                Err(TrySendError::Disconnected(next_stream)) => {
                    relay_sender_live[alternate_index] = LASM_CLUSTER_RELAY_SENDER_DEAD;
                    *relay_live_sender_count = relay_live_sender_count.saturating_sub(1);
                    *relay_all_senders_live = false;
                    client_stream = next_stream;
                }
            }
        }
        return if saw_live_sender && *relay_live_sender_count > 0 {
            Err(LasmClusterRelayDispatchError::Saturated(client_stream))
        } else {
            Err(LasmClusterRelayDispatchError::Unavailable(client_stream))
        };
    }
    let mut scan_index = start_index_wrapped;
    let scan_slot_limit = sender_count.saturating_sub(1);
    if *relay_all_senders_live {
        for _ in 0..scan_slot_limit {
            match relay_senders[scan_index].try_send(client_stream) {
                Ok(()) => return Ok(()),
                Err(TrySendError::Full(next_stream)) => {
                    saw_live_sender = true;
                    client_stream = next_stream;
                }
                Err(TrySendError::Disconnected(next_stream)) => {
                    relay_sender_live[scan_index] = LASM_CLUSTER_RELAY_SENDER_DEAD;
                    *relay_live_sender_count = relay_live_sender_count.saturating_sub(1);
                    *relay_all_senders_live = false;
                    client_stream = next_stream;
                }
            }
            scan_index = lasm_cluster_next_index_wrapped(scan_index, sender_count);
        }
        return if saw_live_sender && *relay_live_sender_count > 0 {
            Err(LasmClusterRelayDispatchError::Saturated(client_stream))
        } else {
            Err(LasmClusterRelayDispatchError::Unavailable(client_stream))
        };
    }
    let scan_live_target = relay_live_sender_count.saturating_sub(1);
    if scan_live_target == 1 {
        if let Some(live_index) =
            lasm_cluster_next_live_sender_index(relay_sender_live, start_index_wrapped)
        {
            match relay_senders[live_index].try_send(client_stream) {
                Ok(()) => return Ok(()),
                Err(TrySendError::Full(next_stream)) => {
                    saw_live_sender = true;
                    client_stream = next_stream;
                }
                Err(TrySendError::Disconnected(next_stream)) => {
                    relay_sender_live[live_index] = LASM_CLUSTER_RELAY_SENDER_DEAD;
                    *relay_live_sender_count = relay_live_sender_count.saturating_sub(1);
                    *relay_all_senders_live = false;
                    client_stream = next_stream;
                }
            }
        }
        return if saw_live_sender && *relay_live_sender_count > 0 {
            Err(LasmClusterRelayDispatchError::Saturated(client_stream))
        } else {
            Err(LasmClusterRelayDispatchError::Unavailable(client_stream))
        };
    }
    let mut scanned_slots = 0usize;
    let mut scanned_live = 0usize;
    while scanned_slots < scan_slot_limit && scanned_live < scan_live_target {
        if relay_sender_live[scan_index] == LASM_CLUSTER_RELAY_SENDER_DEAD {
            scan_index = lasm_cluster_next_index_wrapped(scan_index, sender_count);
            scanned_slots += 1;
            continue;
        }
        scanned_live += 1;
        match relay_senders[scan_index].try_send(client_stream) {
            Ok(()) => return Ok(()),
            Err(TrySendError::Full(next_stream)) => {
                saw_live_sender = true;
                client_stream = next_stream;
            }
            Err(TrySendError::Disconnected(next_stream)) => {
                relay_sender_live[scan_index] = LASM_CLUSTER_RELAY_SENDER_DEAD;
                *relay_live_sender_count = relay_live_sender_count.saturating_sub(1);
                *relay_all_senders_live = false;
                client_stream = next_stream;
            }
        }
        scan_index = lasm_cluster_next_index_wrapped(scan_index, sender_count);
        scanned_slots += 1;
    }

    if saw_live_sender && *relay_live_sender_count > 0 {
        Err(LasmClusterRelayDispatchError::Saturated(client_stream))
    } else {
        Err(LasmClusterRelayDispatchError::Unavailable(client_stream))
    }
}

#[inline(always)]
fn handle_lasm_cluster_accept_dispatch_error(
    dispatch_error: LasmClusterRelayDispatchError,
    active_connections: &AtomicUsize,
    relay_saturation_events: &AtomicUsize,
    relay_saturation_events_total: &AtomicU64,
    relay_dispatch_fallback_total: &AtomicU64,
    relay_dispatch_short_circuit_total: &AtomicU64,
    listener_enqueued_local: &mut usize,
    listener_saturation_pending_local: &mut usize,
    listener_saturation_total_local: &mut u64,
    listener_dispatch_fallback_total_local: &mut u64,
    listener_dispatch_short_circuit_total_local: &mut u64,
) -> Result<(), String> {
    match dispatch_error {
        LasmClusterRelayDispatchError::Saturated(mut stream) => {
            *listener_saturation_pending_local += 1;
            *listener_saturation_total_local += 1;
            let _ = write_lasm_cluster_unavailable_response(
                &mut stream,
                LasmClusterUnavailableReason::RelaySaturated,
            );
            if *listener_saturation_pending_local >= LASM_CLUSTER_SATURATION_COUNTER_FLUSH_BATCH {
                flush_lasm_cluster_saturation_counters(
                    relay_saturation_events,
                    relay_saturation_events_total,
                    listener_saturation_pending_local,
                    listener_saturation_total_local,
                );
            }
            Ok(())
        }
        LasmClusterRelayDispatchError::Unavailable(mut stream) => {
            let _ = write_lasm_cluster_unavailable_response(
                &mut stream,
                LasmClusterUnavailableReason::RelayUnavailable,
            );
            flush_lasm_cluster_active_connection_increments(
                active_connections,
                listener_enqueued_local,
            );
            flush_lasm_cluster_saturation_counters(
                relay_saturation_events,
                relay_saturation_events_total,
                listener_saturation_pending_local,
                listener_saturation_total_local,
            );
            flush_lasm_cluster_dispatch_fallback_total(
                relay_dispatch_fallback_total,
                listener_dispatch_fallback_total_local,
            );
            flush_lasm_cluster_dispatch_short_circuit_total(
                relay_dispatch_short_circuit_total,
                listener_dispatch_short_circuit_total_local,
            );
            Err("LASM cluster relay worker pool disconnected unexpectedly".to_string())
        }
    }
}

fn run_lasm_cluster_accept_loop(
    listener: &TcpListener,
    relay_senders: &[Sender<TcpStream>],
    active_connections: &AtomicUsize,
    relay_saturation_events: &AtomicUsize,
    relay_saturation_events_total: &AtomicU64,
    relay_live_sender_count_observed: &AtomicUsize,
    initial_dispatch_cursor: usize,
    relay_dispatch_fallback_total: &AtomicU64,
    relay_dispatch_short_circuit_total: &AtomicU64,
    stop_flag: &AtomicBool,
    relay_accept_batch_max: usize,
) -> Result<(), String> {
    let mut listener_saturation_pending_local = 0_usize;
    let mut listener_saturation_total_local = 0_u64;
    let mut listener_dispatch_fallback_total_local = 0_u64;
    let mut listener_dispatch_short_circuit_total_local = 0_u64;
    let mut listener_idle_spins = 0_u32;
    let listener_idle_sleep_duration = Duration::from_micros(LASM_CLUSTER_IDLE_SLEEP_MICROS);
    let relay_sender_count = relay_senders.len();
    if relay_sender_count == 0 {
        return Err("LASM cluster relay sender pool unavailable".to_string());
    }
    let relay_single_sender = if relay_sender_count == 1 {
        relay_senders.first()
    } else {
        None
    };
    let mut relay_dispatch_cursor = if relay_sender_count > 1 {
        initial_dispatch_cursor % relay_sender_count
    } else {
        0
    };
    let mut relay_sender_live = if relay_sender_count > 1 {
        vec![LASM_CLUSTER_RELAY_SENDER_LIVE; relay_sender_count]
    } else {
        Vec::new()
    };
    let mut relay_live_sender_count = relay_sender_count;
    let mut relay_all_senders_live = relay_sender_count > 1;
    let mut relay_single_live_sender_index: Option<usize> = None;

    loop {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }
        let mut listener_enqueued_local = 0_usize;
        let mut listener_accepted_in_batch = 0_usize;
        let mut listener_all_senders_saturated_in_batch = false;
        if let Some(relay_single_sender) = relay_single_sender {
            while listener_accepted_in_batch < relay_accept_batch_max {
                match listener.accept() {
                    Ok((client_stream, _)) => {
                        listener_accepted_in_batch += 1;
                        match relay_single_sender.try_send(client_stream) {
                            Ok(()) => {
                                listener_enqueued_local += 1;
                            }
                            Err(send_error) => {
                                let dispatch_error = match send_error {
                                    TrySendError::Full(stream) => {
                                        LasmClusterRelayDispatchError::Saturated(stream)
                                    }
                                    TrySendError::Disconnected(stream) => {
                                        LasmClusterRelayDispatchError::Unavailable(stream)
                                    }
                                };
                                if let Err(message) = handle_lasm_cluster_accept_dispatch_error(
                                    dispatch_error,
                                    active_connections,
                                    relay_saturation_events,
                                    relay_saturation_events_total,
                                    relay_dispatch_fallback_total,
                                    relay_dispatch_short_circuit_total,
                                    &mut listener_enqueued_local,
                                    &mut listener_saturation_pending_local,
                                    &mut listener_saturation_total_local,
                                    &mut listener_dispatch_fallback_total_local,
                                    &mut listener_dispatch_short_circuit_total_local,
                                ) {
                                    return Err(message);
                                }
                            }
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(err) => {
                        flush_lasm_cluster_saturation_counters(
                            relay_saturation_events,
                            relay_saturation_events_total,
                            &mut listener_saturation_pending_local,
                            &mut listener_saturation_total_local,
                        );
                        flush_lasm_cluster_dispatch_fallback_total(
                            relay_dispatch_fallback_total,
                            &mut listener_dispatch_fallback_total_local,
                        );
                        flush_lasm_cluster_dispatch_short_circuit_total(
                            relay_dispatch_short_circuit_total,
                            &mut listener_dispatch_short_circuit_total_local,
                        );
                        return Err(format!("LASM cluster proxy accept error: {err}"));
                    }
                }
            }
        } else {
            while listener_accepted_in_batch < relay_accept_batch_max {
                match listener.accept() {
                    Ok((client_stream, _)) => {
                        listener_accepted_in_batch += 1;
                        if !relay_all_senders_live {
                            if relay_live_sender_count == 0 {
                                relay_live_sender_count_observed.fetch_min(0, Ordering::Relaxed);
                                if let Err(message) = handle_lasm_cluster_accept_dispatch_error(
                                    LasmClusterRelayDispatchError::Unavailable(client_stream),
                                    active_connections,
                                    relay_saturation_events,
                                    relay_saturation_events_total,
                                    relay_dispatch_fallback_total,
                                    relay_dispatch_short_circuit_total,
                                    &mut listener_enqueued_local,
                                    &mut listener_saturation_pending_local,
                                    &mut listener_saturation_total_local,
                                    &mut listener_dispatch_fallback_total_local,
                                    &mut listener_dispatch_short_circuit_total_local,
                                ) {
                                    return Err(message);
                                }
                                continue;
                            }
                            if relay_live_sender_count == 1 {
                                refresh_lasm_cluster_single_live_sender_index(
                                    relay_sender_live.as_slice(),
                                    relay_live_sender_count,
                                    &mut relay_single_live_sender_index,
                                );
                                let Some(single_live_index) = relay_single_live_sender_index else {
                                    if let Err(message) = handle_lasm_cluster_accept_dispatch_error(
                                        LasmClusterRelayDispatchError::Unavailable(client_stream),
                                        active_connections,
                                        relay_saturation_events,
                                        relay_saturation_events_total,
                                        relay_dispatch_fallback_total,
                                        relay_dispatch_short_circuit_total,
                                        &mut listener_enqueued_local,
                                        &mut listener_saturation_pending_local,
                                        &mut listener_saturation_total_local,
                                        &mut listener_dispatch_fallback_total_local,
                                        &mut listener_dispatch_short_circuit_total_local,
                                    ) {
                                        return Err(message);
                                    }
                                    continue;
                                };
                                relay_dispatch_cursor = single_live_index;
                            } else if !realign_lasm_cluster_dispatch_cursor_to_live(
                                relay_sender_live.as_slice(),
                                &mut relay_dispatch_cursor,
                            ) {
                                if let Err(message) = handle_lasm_cluster_accept_dispatch_error(
                                    LasmClusterRelayDispatchError::Unavailable(client_stream),
                                    active_connections,
                                    relay_saturation_events,
                                    relay_saturation_events_total,
                                    relay_dispatch_fallback_total,
                                    relay_dispatch_short_circuit_total,
                                    &mut listener_enqueued_local,
                                    &mut listener_saturation_pending_local,
                                    &mut listener_saturation_total_local,
                                    &mut listener_dispatch_fallback_total_local,
                                    &mut listener_dispatch_short_circuit_total_local,
                                ) {
                                    return Err(message);
                                }
                                continue;
                            }
                        }
                        let stream_dispatch_start = relay_dispatch_cursor;
                        let next_dispatch_index = lasm_cluster_next_index_wrapped(
                            stream_dispatch_start,
                            relay_sender_count,
                        );
                        relay_dispatch_cursor = next_dispatch_index;

                        match relay_senders[stream_dispatch_start].try_send(client_stream) {
                            Ok(()) => {
                                listener_enqueued_local += 1;
                                listener_all_senders_saturated_in_batch = false;
                            }
                            Err(TrySendError::Full(stream))
                                if listener_all_senders_saturated_in_batch =>
                            {
                                if let Err(message) = handle_lasm_cluster_accept_dispatch_error(
                                    LasmClusterRelayDispatchError::Saturated(stream),
                                    active_connections,
                                    relay_saturation_events,
                                    relay_saturation_events_total,
                                    relay_dispatch_fallback_total,
                                    relay_dispatch_short_circuit_total,
                                    &mut listener_enqueued_local,
                                    &mut listener_saturation_pending_local,
                                    &mut listener_saturation_total_local,
                                    &mut listener_dispatch_fallback_total_local,
                                    &mut listener_dispatch_short_circuit_total_local,
                                ) {
                                    return Err(message);
                                }
                                listener_dispatch_short_circuit_total_local += 1;
                            }
                            Err(send_error) => {
                                listener_dispatch_fallback_total_local += 1;
                                let (stream, saw_live_sender) = match send_error {
                                    TrySendError::Full(stream) => (stream, true),
                                    TrySendError::Disconnected(stream) => {
                                        relay_sender_live[stream_dispatch_start] =
                                            LASM_CLUSTER_RELAY_SENDER_DEAD;
                                        relay_live_sender_count =
                                            relay_live_sender_count.saturating_sub(1);
                                        relay_live_sender_count_observed
                                            .fetch_min(relay_live_sender_count, Ordering::Relaxed);
                                        relay_all_senders_live = false;
                                        refresh_lasm_cluster_single_live_sender_index(
                                            relay_sender_live.as_slice(),
                                            relay_live_sender_count,
                                            &mut relay_single_live_sender_index,
                                        );
                                        (stream, false)
                                    }
                                };
                                let dispatch_result =
                                    dispatch_lasm_cluster_relay_stream_fallback_multi(
                                        stream,
                                        relay_senders,
                                        relay_sender_live.as_mut_slice(),
                                        &mut relay_live_sender_count,
                                        &mut relay_all_senders_live,
                                        next_dispatch_index,
                                        saw_live_sender,
                                    );
                                if !relay_all_senders_live
                                    && relay_live_sender_count > 0
                                    && relay_live_sender_count > 1
                                    && relay_sender_live[relay_dispatch_cursor]
                                        == LASM_CLUSTER_RELAY_SENDER_DEAD
                                {
                                    let _ = realign_lasm_cluster_dispatch_cursor_to_live(
                                        relay_sender_live.as_slice(),
                                        &mut relay_dispatch_cursor,
                                    );
                                }
                                refresh_lasm_cluster_single_live_sender_index(
                                    relay_sender_live.as_slice(),
                                    relay_live_sender_count,
                                    &mut relay_single_live_sender_index,
                                );
                                relay_live_sender_count_observed
                                    .fetch_min(relay_live_sender_count, Ordering::Relaxed);
                                match dispatch_result {
                                    Ok(()) => {
                                        listener_enqueued_local += 1;
                                        listener_all_senders_saturated_in_batch = false;
                                    }
                                    Err(dispatch_error) => {
                                        if matches!(
                                            &dispatch_error,
                                            LasmClusterRelayDispatchError::Saturated(_)
                                        ) {
                                            listener_all_senders_saturated_in_batch = true;
                                        }
                                        if let Err(message) =
                                            handle_lasm_cluster_accept_dispatch_error(
                                                dispatch_error,
                                                active_connections,
                                                relay_saturation_events,
                                                relay_saturation_events_total,
                                                relay_dispatch_fallback_total,
                                                relay_dispatch_short_circuit_total,
                                                &mut listener_enqueued_local,
                                                &mut listener_saturation_pending_local,
                                                &mut listener_saturation_total_local,
                                                &mut listener_dispatch_fallback_total_local,
                                                &mut listener_dispatch_short_circuit_total_local,
                                            )
                                        {
                                            return Err(message);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(err) => {
                        flush_lasm_cluster_saturation_counters(
                            relay_saturation_events,
                            relay_saturation_events_total,
                            &mut listener_saturation_pending_local,
                            &mut listener_saturation_total_local,
                        );
                        flush_lasm_cluster_dispatch_fallback_total(
                            relay_dispatch_fallback_total,
                            &mut listener_dispatch_fallback_total_local,
                        );
                        flush_lasm_cluster_dispatch_short_circuit_total(
                            relay_dispatch_short_circuit_total,
                            &mut listener_dispatch_short_circuit_total_local,
                        );
                        return Err(format!("LASM cluster proxy accept error: {err}"));
                    }
                }
            }
        }

        if listener_accepted_in_batch == 0 {
            listener_idle_spins += 1;
            if listener_idle_spins < LASM_CLUSTER_IDLE_SPIN_THRESHOLD {
                std::thread::yield_now();
            } else {
                std::thread::sleep(listener_idle_sleep_duration);
                listener_idle_spins = 0;
            }
            continue;
        }
        listener_idle_spins = 0;

        flush_lasm_cluster_active_connection_increments(
            active_connections,
            &mut listener_enqueued_local,
        );
        flush_lasm_cluster_dispatch_fallback_total(
            relay_dispatch_fallback_total,
            &mut listener_dispatch_fallback_total_local,
        );
        flush_lasm_cluster_dispatch_short_circuit_total(
            relay_dispatch_short_circuit_total,
            &mut listener_dispatch_short_circuit_total_local,
        );
    }

    flush_lasm_cluster_saturation_counters(
        relay_saturation_events,
        relay_saturation_events_total,
        &mut listener_saturation_pending_local,
        &mut listener_saturation_total_local,
    );
    flush_lasm_cluster_dispatch_fallback_total(
        relay_dispatch_fallback_total,
        &mut listener_dispatch_fallback_total_local,
    );
    flush_lasm_cluster_dispatch_short_circuit_total(
        relay_dispatch_short_circuit_total,
        &mut listener_dispatch_short_circuit_total_local,
    );
    Ok(())
}

enum LasmClusterRelayPumpStep {
    Progressed,
    Idle,
    Complete,
}

struct LasmClusterRelayPump {
    client: TcpStream,
    upstream: TcpStream,
    client_to_upstream: Vec<u8>,
    upstream_to_client: Vec<u8>,
    c2u_start: usize,
    c2u_end: usize,
    u2c_start: usize,
    u2c_end: usize,
    client_read_closed: bool,
    upstream_read_closed: bool,
    client_write_closed: bool,
    upstream_write_closed: bool,
}

impl LasmClusterRelayPump {
    fn new(client: TcpStream, upstream: TcpStream) -> Result<Self, String> {
        Self::new_with_buffers(
            client,
            upstream,
            vec![0_u8; LASM_CLUSTER_RELAY_BUFFER_BYTES],
            vec![0_u8; LASM_CLUSTER_RELAY_BUFFER_BYTES],
        )
    }

    fn new_with_buffers(
        client: TcpStream,
        upstream: TcpStream,
        mut client_to_upstream: Vec<u8>,
        mut upstream_to_client: Vec<u8>,
    ) -> Result<Self, String> {
        client
            .set_nonblocking(true)
            .map_err(|err| format!("could not set client proxy stream nonblocking: {err}"))?;
        upstream
            .set_nonblocking(true)
            .map_err(|err| format!("could not set upstream proxy stream nonblocking: {err}"))?;
        if client_to_upstream.len() != LASM_CLUSTER_RELAY_BUFFER_BYTES {
            client_to_upstream.resize(LASM_CLUSTER_RELAY_BUFFER_BYTES, 0_u8);
        }
        if upstream_to_client.len() != LASM_CLUSTER_RELAY_BUFFER_BYTES {
            upstream_to_client.resize(LASM_CLUSTER_RELAY_BUFFER_BYTES, 0_u8);
        }
        Ok(Self {
            client,
            upstream,
            client_to_upstream,
            upstream_to_client,
            c2u_start: 0,
            c2u_end: 0,
            u2c_start: 0,
            u2c_end: 0,
            client_read_closed: false,
            upstream_read_closed: false,
            client_write_closed: false,
            upstream_write_closed: false,
        })
    }

    fn pump_once(&mut self) -> Result<LasmClusterRelayPumpStep, String> {
        let mut progressed = false;

        while !self.client_read_closed && self.c2u_end < self.client_to_upstream.len() {
            match self
                .client
                .read(&mut self.client_to_upstream[self.c2u_end..])
            {
                Ok(0) => {
                    self.client_read_closed = true;
                    progressed = true;
                    break;
                }
                Ok(bytes_read) => {
                    self.c2u_end += bytes_read;
                    progressed = true;
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => {
                    return Err(format!("could not read proxy client stream: {err}"));
                }
            }
        }

        while self.c2u_start < self.c2u_end {
            match self
                .upstream
                .write(&self.client_to_upstream[self.c2u_start..self.c2u_end])
            {
                Ok(0) => {
                    return Err(
                        "could not relay client payload to upstream: write returned 0 bytes"
                            .to_string(),
                    );
                }
                Ok(bytes_written) => {
                    self.c2u_start += bytes_written;
                    progressed = true;
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => {
                    return Err(format!("could not relay client payload to upstream: {err}"));
                }
            }
        }
        if self.c2u_start == self.c2u_end {
            self.c2u_start = 0;
            self.c2u_end = 0;
            if self.client_read_closed && !self.upstream_write_closed {
                let _ = self.upstream.shutdown(Shutdown::Write);
                self.upstream_write_closed = true;
                progressed = true;
            }
        }

        while !self.upstream_read_closed && self.u2c_end < self.upstream_to_client.len() {
            match self
                .upstream
                .read(&mut self.upstream_to_client[self.u2c_end..])
            {
                Ok(0) => {
                    self.upstream_read_closed = true;
                    progressed = true;
                    break;
                }
                Ok(bytes_read) => {
                    self.u2c_end += bytes_read;
                    progressed = true;
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => {
                    return Err(format!("could not read proxy upstream stream: {err}"));
                }
            }
        }

        while self.u2c_start < self.u2c_end {
            match self
                .client
                .write(&self.upstream_to_client[self.u2c_start..self.u2c_end])
            {
                Ok(0) => {
                    return Err(
                        "could not relay upstream response to client: write returned 0 bytes"
                            .to_string(),
                    );
                }
                Ok(bytes_written) => {
                    self.u2c_start += bytes_written;
                    progressed = true;
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => {
                    return Err(format!(
                        "could not relay upstream response to client: {err}"
                    ));
                }
            }
        }
        if self.u2c_start == self.u2c_end {
            self.u2c_start = 0;
            self.u2c_end = 0;
            if self.upstream_read_closed && !self.client_write_closed {
                let _ = self.client.shutdown(Shutdown::Write);
                self.client_write_closed = true;
                progressed = true;
            }
        }

        if self.client_write_closed
            && self.upstream_write_closed
            && self.c2u_end == 0
            && self.u2c_end == 0
        {
            return Ok(LasmClusterRelayPumpStep::Complete);
        }

        if progressed {
            Ok(LasmClusterRelayPumpStep::Progressed)
        } else {
            Ok(LasmClusterRelayPumpStep::Idle)
        }
    }

    fn into_buffers(self) -> (Vec<u8>, Vec<u8>) {
        (self.client_to_upstream, self.upstream_to_client)
    }
}

fn stop_lasm_cluster_workers(state: &mut LasmClusterState) {
    for worker in &mut state.workers {
        let _ = worker.child.kill();
        let _ = worker.child.wait();
    }
    state.workers.clear();
}

fn bind_lasm_listener(listen_port: u16, reuse_port: bool) -> Result<TcpListener, String> {
    if !reuse_port {
        return TcpListener::bind(("127.0.0.1", listen_port)).map_err(|err| {
            format!("could not bind LASM backend listener on 127.0.0.1:{listen_port}: {err}")
        });
    }

    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
        .map_err(|err| format!("could not create LASM reuse-port socket: {err}"))?;
    socket.set_reuse_address(true).map_err(|err| {
        format!("could not set LASM reuse-port socket option SO_REUSEADDR: {err}")
    })?;
    #[cfg(unix)]
    socket.set_reuse_port(true).map_err(|err| {
        format!("could not set LASM reuse-port socket option SO_REUSEPORT: {err}")
    })?;

    let addr = std::net::SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, listen_port);
    socket.bind(&addr.into()).map_err(|err| {
        format!("could not bind LASM reuse-port listener on 127.0.0.1:{listen_port}: {err}")
    })?;
    socket.listen(1024).map_err(|err| {
        format!("could not listen on LASM reuse-port listener 127.0.0.1:{listen_port}: {err}")
    })?;

    Ok(socket.into())
}

fn cmd_run_lasm_reuseport_cluster(config: LasmClusterConfig) -> Result<(), i32> {
    let mut state = LasmClusterState {
        workers: Vec::new(),
        next_port: config.listen_port,
    };
    for _ in 0..config.min_instances {
        match spawn_and_wait_lasm_cluster_worker(&config, config.listen_port) {
            Ok(worker) => state.workers.push(worker),
            Err(message) => {
                stop_lasm_cluster_workers(&mut state);
                eprintln!("run failed: {message}");
                return Err(2);
            }
        }
    }
    if state.workers.is_empty() {
        eprintln!("run failed: could not bootstrap LASM reuse-port workers");
        return Err(2);
    }

    loop {
        std::thread::sleep(Duration::from_millis(config.autoscale_check_ms.max(200)));
        prune_dead_lasm_cluster_workers(&mut state);
        while state.workers.len() < config.min_instances {
            match spawn_and_wait_lasm_cluster_worker(&config, config.listen_port) {
                Ok(worker) => state.workers.push(worker),
                Err(message) => {
                    eprintln!("warning: LASM reuse-port worker recovery failed: {message}");
                    break;
                }
            }
        }
    }
}

const LASM_CLUSTER_SELECTION_LOOKUP_NONE: usize = usize::MAX;

fn rebuild_lasm_cluster_backend_selection_lookup(
    worker_port_count: usize,
    unhealthy_ports_until_by_index: &[Option<Instant>],
    unhealthy_port_count: usize,
    lookup: &mut Vec<usize>,
) -> (bool, bool) {
    debug_assert!(unhealthy_ports_until_by_index.len() >= worker_port_count);
    lookup.clear();
    if worker_port_count == 0 {
        return (false, false);
    }
    if unhealthy_port_count >= worker_port_count {
        return (false, false);
    }
    if unhealthy_port_count == 0 {
        return (true, true);
    }
    lookup.resize(worker_port_count, LASM_CLUSTER_SELECTION_LOOKUP_NONE);

    let mut first_healthy_index: Option<usize> = None;
    let mut next_healthy_index = LASM_CLUSTER_SELECTION_LOOKUP_NONE;
    for index in (0..worker_port_count).rev() {
        if unhealthy_ports_until_by_index[index].is_none() {
            next_healthy_index = index;
            first_healthy_index = Some(index);
        }
        lookup[index] = next_healthy_index;
    }

    let Some(first_healthy_index) = first_healthy_index else {
        return (false, false);
    };

    for index in ((first_healthy_index.saturating_add(1))..worker_port_count).rev() {
        if lookup[index] != LASM_CLUSTER_SELECTION_LOOKUP_NONE {
            break;
        }
        lookup[index] = first_healthy_index;
    }
    (true, false)
}

fn rebuild_lasm_cluster_worker_backend_addrs(
    worker_ports: &[u16],
    addrs: &mut Vec<std::net::SocketAddr>,
) {
    addrs.clear();
    addrs.reserve(worker_ports.len());
    for port in worker_ports {
        addrs.push(std::net::SocketAddr::from((
            std::net::Ipv4Addr::LOCALHOST,
            *port,
        )));
    }
}

fn remap_lasm_cluster_relay_port_state_by_index(
    previous_ports: &[u16],
    next_ports: &[u16],
    previous_unhealthy_ports_until_by_index: &[Option<Instant>],
    previous_connect_warning_next_allowed_by_index: &[Option<Instant>],
    now: Instant,
    unhealthy_ports_until_by_index: &mut Vec<Option<Instant>>,
    connect_warning_next_allowed_by_index: &mut Vec<Option<Instant>>,
) -> usize {
    unhealthy_ports_until_by_index.clear();
    unhealthy_ports_until_by_index.resize(next_ports.len(), None);
    connect_warning_next_allowed_by_index.clear();
    connect_warning_next_allowed_by_index.resize(next_ports.len(), None);

    let mut previous_index = 0_usize;
    let mut next_index = 0_usize;
    let mut unhealthy_port_count = 0_usize;
    while previous_index < previous_ports.len() && next_index < next_ports.len() {
        match previous_ports[previous_index].cmp(&next_ports[next_index]) {
            std::cmp::Ordering::Less => {
                previous_index += 1;
            }
            std::cmp::Ordering::Greater => {
                next_index += 1;
            }
            std::cmp::Ordering::Equal => {
                if let Some(until) = previous_unhealthy_ports_until_by_index
                    .get(previous_index)
                    .and_then(|value| *value)
                {
                    if until > now {
                        unhealthy_ports_until_by_index[next_index] = Some(until);
                        unhealthy_port_count += 1;
                    }
                }
                if let Some(next_allowed) = previous_connect_warning_next_allowed_by_index
                    .get(previous_index)
                    .and_then(|value| *value)
                {
                    connect_warning_next_allowed_by_index[next_index] = Some(next_allowed);
                }
                previous_index += 1;
                next_index += 1;
            }
        }
    }
    unhealthy_port_count
}

fn cmd_run_lasm_cluster(config: LasmClusterConfig) -> Result<(), i32> {
    let listener = match TcpListener::bind(("127.0.0.1", config.listen_port)) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!(
                "run failed: could not bind LASM cluster proxy on 127.0.0.1:{}: {err}",
                config.listen_port
            );
            return Err(2);
        }
    };

    let base_port = match compute_lasm_cluster_base_port(config.listen_port, config.max_instances) {
        Ok(port) => port,
        Err(message) => {
            eprintln!("run failed: {message}");
            return Err(2);
        }
    };

    let mut state = LasmClusterState {
        workers: Vec::new(),
        next_port: base_port,
    };
    for _ in 0..config.min_instances {
        let worker_port = state.next_port;
        state.next_port = state.next_port.saturating_add(1);
        match spawn_and_wait_lasm_cluster_worker(&config, worker_port) {
            Ok(worker) => state.workers.push(worker),
            Err(message) => {
                stop_lasm_cluster_workers(&mut state);
                eprintln!("run failed: {message}");
                return Err(2);
            }
        }
    }
    if state.workers.is_empty() {
        eprintln!("run failed: could not bootstrap LASM cluster workers");
        return Err(2);
    }

    let worker_ports_snapshot = Arc::new(ArcSwap::from_pointee(
        state
            .workers
            .iter()
            .map(|worker| worker.port)
            .collect::<Vec<_>>(),
    ));
    let shared_state = Arc::new(RwLock::new(state));
    let shared_config = Arc::new(config);
    let active_connections = Arc::new(AtomicUsize::new(0));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let relay_saturation_events = Arc::new(AtomicUsize::new(0));
    let relay_saturation_events_total = Arc::new(AtomicU64::new(0));
    let relay_worker_count = lasm_cluster_proxy_worker_count(shared_config.as_ref());
    let relay_accept_worker_count =
        lasm_cluster_accept_worker_count(shared_config.as_ref(), relay_worker_count);
    let relay_queue_capacity =
        lasm_cluster_proxy_queue_capacity(shared_config.as_ref(), relay_worker_count);
    let relay_accept_batch_max = lasm_cluster_relay_accept_batch_max(shared_config.as_ref());
    let relay_pump_batch_max = lasm_cluster_relay_pump_batch_max(shared_config.as_ref());
    let relay_queue_shard_capacity = relay_queue_capacity
        .saturating_add(relay_worker_count.saturating_sub(1))
        / relay_worker_count.max(1);
    let relay_queue_shard_capacity = relay_queue_shard_capacity.max(1);
    let mut relay_senders = Vec::with_capacity(relay_worker_count);
    let mut relay_receivers = Vec::with_capacity(relay_worker_count);
    for _ in 0..relay_worker_count {
        let (relay_sender, relay_receiver) = bounded::<TcpStream>(relay_queue_shard_capacity);
        relay_senders.push(relay_sender);
        relay_receivers.push(relay_receiver);
    }
    let relay_senders = Arc::new(relay_senders);
    let relay_selection_counter = Arc::new(AtomicUsize::new(0));
    let relay_dispatch_fallback_total = Arc::new(AtomicU64::new(0));
    let relay_dispatch_saturation_short_circuit_total = Arc::new(AtomicU64::new(0));
    let relay_live_sender_count = Arc::new(AtomicUsize::new(relay_worker_count));
    let relay_backend_connect_timeout =
        lasm_cluster_backend_connect_timeout(shared_config.as_ref());
    let relay_backend_connect_cooldown =
        lasm_cluster_backend_connect_cooldown(shared_config.as_ref());
    let mut relay_handles = Vec::with_capacity(relay_worker_count);

    for relay_receiver in relay_receivers {
        let relay_active = Arc::clone(&active_connections);
        let relay_selection_counter = Arc::clone(&relay_selection_counter);
        let relay_worker_ports = Arc::clone(&worker_ports_snapshot);
        let relay_saturation_events = Arc::clone(&relay_saturation_events);
        let relay_saturation_events_total = Arc::clone(&relay_saturation_events_total);
        let relay_backend_connect_timeout = relay_backend_connect_timeout;
        let relay_backend_connect_cooldown = relay_backend_connect_cooldown;
        let relay_accept_batch_max = relay_accept_batch_max;
        let relay_pump_batch_max = relay_pump_batch_max;
        relay_handles.push(std::thread::spawn(move || {
            let relay_buffer_pool_max = relay_accept_batch_max.saturating_mul(4).max(64);
            let mut relay_connections: Vec<LasmClusterRelayPump> =
                Vec::with_capacity(relay_accept_batch_max.max(1));
            let mut relay_buffer_pool: Vec<(Vec<u8>, Vec<u8>)> =
                Vec::with_capacity(relay_buffer_pool_max);
            let mut unhealthy_ports_until_by_index: Vec<Option<Instant>> = Vec::new();
            let mut connect_warning_next_allowed_by_index: Vec<Option<Instant>> = Vec::new();
            let mut unhealthy_port_count = 0_usize;
            let mut pump_warning_next_allowed: Option<Instant> = None;
            let mut receiver_closed = false;
            let mut idle_spins = 0_u32;
            let relay_idle_sleep_duration = Duration::from_micros(LASM_CLUSTER_IDLE_SLEEP_MICROS);
            let mut saturation_events_pending_local = 0_usize;
            let mut saturation_events_total_local = 0_u64;
            let mut active_connection_decrements_local = 0_usize;
            let mut relay_selection_reservation_len = 0_usize;
            let mut relay_selection_reservation_offset = 0_usize;
            let mut relay_selection_reservation_next_index = 0_usize;
            let mut relay_selection_reservation_worker_port_count = 0_usize;
            let mut relay_pump_cursor = 0_usize;
            let mut unhealthy_prune_next_at: Option<Instant> = None;
            let relay_warning_throttle_duration =
                Duration::from_millis(LASM_CLUSTER_RELAY_WARNING_THROTTLE_MS);
            let unhealthy_prune_interval =
                Duration::from_millis(LASM_CLUSTER_UNHEALTHY_PRUNE_INTERVAL_MS);
            let mut selected_worker_ports_snapshot = relay_worker_ports.load_full();
            let mut selected_worker_port_count = selected_worker_ports_snapshot.len();
            let mut selected_worker_backend_addrs: Vec<std::net::SocketAddr> =
                Vec::with_capacity(selected_worker_port_count);
            rebuild_lasm_cluster_worker_backend_addrs(
                selected_worker_ports_snapshot.as_ref(),
                &mut selected_worker_backend_addrs,
            );
            unhealthy_ports_until_by_index.resize(selected_worker_port_count, None);
            connect_warning_next_allowed_by_index.resize(selected_worker_port_count, None);
            let mut selection_lookup: Vec<usize> = Vec::new();
            let mut selection_has_healthy_backends = false;
            let mut selection_lookup_is_identity = false;
            let mut selection_single_healthy_index = LASM_CLUSTER_SELECTION_LOOKUP_NONE;
            let mut selection_lookup_dirty = true;

            loop {
                let mut accepted = false;
                let mut accepted_in_batch = 0_usize;
                let mut worker_ports_snapshot: Option<Arc<Vec<u16>>> = None;
                if unhealthy_port_count > 0 {
                    let now = Instant::now();
                    let should_prune = match unhealthy_prune_next_at {
                        Some(next_at) => now >= next_at,
                        None => true,
                    };
                    if should_prune {
                        let snapshot = relay_worker_ports.load_full();
                        if !Arc::ptr_eq(&selected_worker_ports_snapshot, &snapshot) {
                            let previous_ports_snapshot =
                                std::mem::replace(&mut selected_worker_ports_snapshot, snapshot);
                            let previous_unhealthy_ports_until_by_index =
                                std::mem::take(&mut unhealthy_ports_until_by_index);
                            let previous_connect_warning_next_allowed_by_index =
                                std::mem::take(&mut connect_warning_next_allowed_by_index);
                            selected_worker_port_count = selected_worker_ports_snapshot.len();
                            rebuild_lasm_cluster_worker_backend_addrs(
                                selected_worker_ports_snapshot.as_ref(),
                                &mut selected_worker_backend_addrs,
                            );
                            unhealthy_port_count = remap_lasm_cluster_relay_port_state_by_index(
                                previous_ports_snapshot.as_ref(),
                                selected_worker_ports_snapshot.as_ref(),
                                previous_unhealthy_ports_until_by_index.as_slice(),
                                previous_connect_warning_next_allowed_by_index.as_slice(),
                                now,
                                &mut unhealthy_ports_until_by_index,
                                &mut connect_warning_next_allowed_by_index,
                            );
                            selection_lookup_dirty = true;
                        }
                        let mut unhealthy_port_count_after_prune = 0_usize;
                        for entry in &mut unhealthy_ports_until_by_index {
                            if let Some(until) = *entry {
                                if until <= now {
                                    *entry = None;
                                } else {
                                    unhealthy_port_count_after_prune += 1;
                                }
                            }
                        }
                        if unhealthy_port_count_after_prune != unhealthy_port_count {
                            selection_lookup_dirty = true;
                        }
                        unhealthy_port_count = unhealthy_port_count_after_prune;
                        unhealthy_prune_next_at = if unhealthy_port_count == 0 {
                            None
                        } else {
                            Some(now + unhealthy_prune_interval)
                        };
                        worker_ports_snapshot = Some(Arc::clone(&selected_worker_ports_snapshot));
                    }
                }
                loop {
                    if accepted_in_batch >= relay_accept_batch_max {
                        break;
                    }
                    let incoming = match relay_receiver.try_recv() {
                        Ok(stream) => Some(stream),
                        Err(crossbeam_channel::TryRecvError::Empty) => None,
                        Err(crossbeam_channel::TryRecvError::Disconnected) => {
                            receiver_closed = true;
                            None
                        }
                    };
                    let Some(mut client) = incoming else {
                        break;
                    };
                    accepted = true;
                    accepted_in_batch += 1;

                    if worker_ports_snapshot.is_none() || selection_lookup_dirty {
                        if worker_ports_snapshot.is_none() {
                            worker_ports_snapshot = Some(relay_worker_ports.load_full());
                        }
                        let worker_ports_snapshot_ref = worker_ports_snapshot
                            .as_ref()
                            .expect("worker port snapshot loaded before backend selection");
                        if !Arc::ptr_eq(&selected_worker_ports_snapshot, worker_ports_snapshot_ref)
                        {
                            let now = Instant::now();
                            let previous_ports_snapshot = std::mem::replace(
                                &mut selected_worker_ports_snapshot,
                                Arc::clone(worker_ports_snapshot_ref),
                            );
                            let previous_unhealthy_ports_until_by_index =
                                std::mem::take(&mut unhealthy_ports_until_by_index);
                            let previous_connect_warning_next_allowed_by_index =
                                std::mem::take(&mut connect_warning_next_allowed_by_index);
                            selected_worker_port_count = selected_worker_ports_snapshot.len();
                            rebuild_lasm_cluster_worker_backend_addrs(
                                selected_worker_ports_snapshot.as_ref(),
                                &mut selected_worker_backend_addrs,
                            );
                            unhealthy_port_count = remap_lasm_cluster_relay_port_state_by_index(
                                previous_ports_snapshot.as_ref(),
                                selected_worker_ports_snapshot.as_ref(),
                                previous_unhealthy_ports_until_by_index.as_slice(),
                                previous_connect_warning_next_allowed_by_index.as_slice(),
                                now,
                                &mut unhealthy_ports_until_by_index,
                                &mut connect_warning_next_allowed_by_index,
                            );
                            unhealthy_prune_next_at = if unhealthy_port_count == 0 {
                                None
                            } else {
                                Some(now + unhealthy_prune_interval)
                            };
                            selection_lookup_dirty = true;
                        }
                        let worker_port_count = selected_worker_port_count;
                        if selection_lookup_dirty
                            || (!selection_lookup_is_identity
                                && selection_lookup.len() != worker_port_count)
                        {
                            if worker_port_count == 0 {
                                selection_lookup.clear();
                                selection_has_healthy_backends = false;
                                selection_lookup_is_identity = false;
                                selection_single_healthy_index = LASM_CLUSTER_SELECTION_LOOKUP_NONE;
                            } else if unhealthy_port_count == 0 {
                                selection_lookup.clear();
                                selection_has_healthy_backends = true;
                                selection_lookup_is_identity = true;
                                selection_single_healthy_index = LASM_CLUSTER_SELECTION_LOOKUP_NONE;
                            } else if worker_port_count == 1 {
                                selection_lookup.clear();
                                selection_has_healthy_backends = false;
                                selection_lookup_is_identity = true;
                                selection_single_healthy_index = LASM_CLUSTER_SELECTION_LOOKUP_NONE;
                            } else if unhealthy_port_count + 1 == worker_port_count {
                                selection_lookup.clear();
                                if let Some(single_healthy_index) = unhealthy_ports_until_by_index
                                    .iter()
                                    .take(worker_port_count)
                                    .position(|entry| entry.is_none())
                                {
                                    selection_has_healthy_backends = true;
                                    selection_lookup_is_identity = false;
                                    selection_single_healthy_index = single_healthy_index;
                                } else {
                                    selection_has_healthy_backends = false;
                                    selection_lookup_is_identity = false;
                                    selection_single_healthy_index =
                                        LASM_CLUSTER_SELECTION_LOOKUP_NONE;
                                }
                            } else {
                                selection_single_healthy_index = LASM_CLUSTER_SELECTION_LOOKUP_NONE;
                                (selection_has_healthy_backends, selection_lookup_is_identity) =
                                    rebuild_lasm_cluster_backend_selection_lookup(
                                        worker_port_count,
                                        unhealthy_ports_until_by_index.as_slice(),
                                        unhealthy_port_count,
                                        &mut selection_lookup,
                                    );
                            }
                            selection_lookup_dirty = false;
                        }
                    }
                    let worker_port_count = selected_worker_port_count;
                    let selected_backend_index = if worker_port_count == 0
                        || !selection_has_healthy_backends
                    {
                        LASM_CLUSTER_SELECTION_LOOKUP_NONE
                    } else if worker_port_count == 1 {
                        0
                    } else if selection_single_healthy_index != LASM_CLUSTER_SELECTION_LOOKUP_NONE {
                        selection_single_healthy_index
                    } else {
                        if relay_selection_reservation_offset >= relay_selection_reservation_len
                            || relay_selection_reservation_worker_port_count != worker_port_count
                        {
                            let reservation_chunk = relay_accept_batch_max
                                .max(LASM_CLUSTER_SELECTION_RESERVATION_MIN_CHUNK);
                            let relay_selection_reservation_base = relay_selection_counter
                                .fetch_add(reservation_chunk, Ordering::Relaxed);
                            relay_selection_reservation_len = reservation_chunk;
                            relay_selection_reservation_offset = 0;
                            relay_selection_reservation_worker_port_count = worker_port_count;
                            relay_selection_reservation_next_index =
                                relay_selection_reservation_base % worker_port_count;
                        }
                        let start_index = relay_selection_reservation_next_index;
                        relay_selection_reservation_offset += 1;
                        relay_selection_reservation_next_index =
                            lasm_cluster_next_index_wrapped(start_index, worker_port_count);
                        if selection_lookup_is_identity {
                            start_index
                        } else {
                            debug_assert_eq!(selection_lookup.len(), worker_port_count);
                            selection_lookup[start_index]
                        }
                    };

                    if selected_backend_index == LASM_CLUSTER_SELECTION_LOOKUP_NONE {
                        saturation_events_pending_local += 1;
                        saturation_events_total_local += 1;
                        let _ = write_lasm_cluster_unavailable_response(
                            &mut client,
                            LasmClusterUnavailableReason::NoHealthyWorkers,
                        );
                        active_connection_decrements_local += 1;
                        continue;
                    }

                    let backend_addr = selected_worker_backend_addrs[selected_backend_index];
                    match TcpStream::connect_timeout(&backend_addr, relay_backend_connect_timeout) {
                        Ok(upstream) => {
                            let _ = client.set_nodelay(true);
                            let _ = upstream.set_nodelay(true);
                            let relay_result =
                                if let Some((client_to_upstream, upstream_to_client)) =
                                    relay_buffer_pool.pop()
                                {
                                    LasmClusterRelayPump::new_with_buffers(
                                        client,
                                        upstream,
                                        client_to_upstream,
                                        upstream_to_client,
                                    )
                                } else {
                                    LasmClusterRelayPump::new(client, upstream)
                                };
                            match relay_result {
                                Ok(relay) => relay_connections.push(relay),
                                Err(message) => {
                                    let now = Instant::now();
                                    let warning_allowed = match pump_warning_next_allowed {
                                        Some(next_allowed_at) => now >= next_allowed_at,
                                        None => true,
                                    };
                                    if warning_allowed {
                                        eprintln!(
                                            "warning: LASM cluster relay init failed: {message}"
                                        );
                                        pump_warning_next_allowed =
                                            Some(now + relay_warning_throttle_duration);
                                    }
                                    active_connection_decrements_local += 1;
                                }
                            }
                        }
                        Err(err) => {
                            saturation_events_pending_local += 1;
                            saturation_events_total_local += 1;
                            let now = Instant::now();
                            let unhealthy_until = now + relay_backend_connect_cooldown;
                            let unhealthy_entry =
                                &mut unhealthy_ports_until_by_index[selected_backend_index];
                            let should_mark_unhealthy = match *unhealthy_entry {
                                Some(existing_until) => existing_until <= now,
                                None => true,
                            };
                            if should_mark_unhealthy {
                                unhealthy_port_count += 1;
                                selection_lookup_dirty = true;
                            }
                            *unhealthy_entry = Some(unhealthy_until);
                            if unhealthy_prune_next_at.is_none() && unhealthy_port_count > 0 {
                                unhealthy_prune_next_at = Some(now + unhealthy_prune_interval);
                            }
                            let warning_next_allowed_entry =
                                &mut connect_warning_next_allowed_by_index[selected_backend_index];
                            let warning_allowed = match *warning_next_allowed_entry {
                                Some(next_allowed_at) => now >= next_allowed_at,
                                None => true,
                            };
                            if warning_allowed {
                                eprintln!(
                                    "warning: LASM cluster worker {} connect failed: {}",
                                    backend_addr.port(),
                                    err
                                );
                                *warning_next_allowed_entry =
                                    Some(now + relay_warning_throttle_duration);
                            }
                            let _ = write_lasm_cluster_unavailable_response(
                                &mut client,
                                LasmClusterUnavailableReason::WorkerUnavailable,
                            );
                            active_connection_decrements_local += 1;
                        }
                    }
                }

                let mut progressed = false;
                if relay_connections.len() <= relay_pump_batch_max {
                    let mut index = 0_usize;
                    while index < relay_connections.len() {
                        match relay_connections[index].pump_once() {
                            Ok(LasmClusterRelayPumpStep::Progressed) => {
                                progressed = true;
                                index += 1;
                            }
                            Ok(LasmClusterRelayPumpStep::Idle) => {
                                index += 1;
                            }
                            Ok(LasmClusterRelayPumpStep::Complete) => {
                                let relay = relay_connections.swap_remove(index);
                                if relay_buffer_pool.len() < relay_buffer_pool_max {
                                    relay_buffer_pool.push(relay.into_buffers());
                                }
                                active_connection_decrements_local += 1;
                                progressed = true;
                            }
                            Err(err) => {
                                let now = Instant::now();
                                let warning_allowed = match pump_warning_next_allowed {
                                    Some(next_allowed_at) => now >= next_allowed_at,
                                    None => true,
                                };
                                if warning_allowed {
                                    eprintln!("warning: LASM cluster relay pump failed: {err}");
                                    pump_warning_next_allowed =
                                        Some(now + relay_warning_throttle_duration);
                                }
                                let relay = relay_connections.swap_remove(index);
                                if relay_buffer_pool.len() < relay_buffer_pool_max {
                                    relay_buffer_pool.push(relay.into_buffers());
                                }
                                active_connection_decrements_local += 1;
                                progressed = true;
                            }
                        }
                    }
                    relay_pump_cursor = 0;
                } else {
                    if relay_pump_cursor >= relay_connections.len() {
                        relay_pump_cursor = 0;
                    }
                    let mut pump_budget = relay_pump_batch_max.min(relay_connections.len());
                    while pump_budget > 0 {
                        let relay_len_before_step = relay_connections.len();
                        match relay_connections[relay_pump_cursor].pump_once() {
                            Ok(LasmClusterRelayPumpStep::Progressed) => {
                                progressed = true;
                                relay_pump_cursor = lasm_cluster_next_index_wrapped(
                                    relay_pump_cursor,
                                    relay_len_before_step,
                                );
                                pump_budget -= 1;
                            }
                            Ok(LasmClusterRelayPumpStep::Idle) => {
                                relay_pump_cursor = lasm_cluster_next_index_wrapped(
                                    relay_pump_cursor,
                                    relay_len_before_step,
                                );
                                pump_budget -= 1;
                            }
                            Ok(LasmClusterRelayPumpStep::Complete) => {
                                let relay = relay_connections.swap_remove(relay_pump_cursor);
                                if relay_buffer_pool.len() < relay_buffer_pool_max {
                                    relay_buffer_pool.push(relay.into_buffers());
                                }
                                active_connection_decrements_local += 1;
                                progressed = true;
                                pump_budget -= 1;
                                if relay_connections.is_empty() {
                                    relay_pump_cursor = 0;
                                    break;
                                }
                                if relay_pump_cursor >= relay_connections.len() {
                                    relay_pump_cursor = 0;
                                }
                            }
                            Err(err) => {
                                let now = Instant::now();
                                let warning_allowed = match pump_warning_next_allowed {
                                    Some(next_allowed_at) => now >= next_allowed_at,
                                    None => true,
                                };
                                if warning_allowed {
                                    eprintln!("warning: LASM cluster relay pump failed: {err}");
                                    pump_warning_next_allowed =
                                        Some(now + relay_warning_throttle_duration);
                                }
                                let relay = relay_connections.swap_remove(relay_pump_cursor);
                                if relay_buffer_pool.len() < relay_buffer_pool_max {
                                    relay_buffer_pool.push(relay.into_buffers());
                                }
                                active_connection_decrements_local += 1;
                                progressed = true;
                                pump_budget -= 1;
                                if relay_connections.is_empty() {
                                    relay_pump_cursor = 0;
                                    break;
                                }
                                if relay_pump_cursor >= relay_connections.len() {
                                    relay_pump_cursor = 0;
                                }
                            }
                        }
                    }
                }

                flush_lasm_cluster_saturation_counters(
                    &relay_saturation_events,
                    &relay_saturation_events_total,
                    &mut saturation_events_pending_local,
                    &mut saturation_events_total_local,
                );
                flush_lasm_cluster_active_connection_decrements(
                    &relay_active,
                    &mut active_connection_decrements_local,
                );

                if receiver_closed && relay_connections.is_empty() {
                    break;
                }

                if accepted || progressed {
                    idle_spins = 0;
                    continue;
                }

                idle_spins += 1;
                if idle_spins < LASM_CLUSTER_IDLE_SPIN_THRESHOLD {
                    std::thread::yield_now();
                } else {
                    std::thread::sleep(relay_idle_sleep_duration);
                    idle_spins = 0;
                }
            }
        }));
    }

    let autoscale_last_desired_instances = Arc::new(AtomicUsize::new(shared_config.min_instances));
    let autoscale_last_saturation_events = Arc::new(AtomicUsize::new(0));
    let autoscale_last_dynamic_boost_step = Arc::new(AtomicUsize::new(
        shared_config.autoscale_scale_up_step.max(1),
    ));
    let autoscale_scale_up_cooldown_remaining_ms = Arc::new(AtomicU64::new(0));
    let autoscale_scale_down_cooldown_remaining_ms = Arc::new(AtomicU64::new(0));

    let status_writer_handle = if let Some(status_path) = shared_config.cluster_status_json.clone()
    {
        let status_stop_flag = Arc::clone(&stop_flag);
        let status_config = Arc::clone(&shared_config);
        let status_active_connections = Arc::clone(&active_connections);
        let status_saturation_events = Arc::clone(&relay_saturation_events);
        let status_saturation_events_total = Arc::clone(&relay_saturation_events_total);
        let status_worker_ports = Arc::clone(&worker_ports_snapshot);
        let status_relay_worker_count = relay_worker_count;
        let status_relay_queue_capacity = relay_queue_capacity;
        let status_relay_queue_shard_capacity = relay_queue_shard_capacity;
        let status_relay_accept_workers = relay_accept_worker_count;
        let status_relay_dispatch_fallback_total = Arc::clone(&relay_dispatch_fallback_total);
        let status_relay_dispatch_saturation_short_circuit_total =
            Arc::clone(&relay_dispatch_saturation_short_circuit_total);
        let status_relay_live_sender_count = Arc::clone(&relay_live_sender_count);
        let status_autoscale_last_desired_instances = Arc::clone(&autoscale_last_desired_instances);
        let status_autoscale_last_saturation_events = Arc::clone(&autoscale_last_saturation_events);
        let status_autoscale_last_dynamic_boost_step =
            Arc::clone(&autoscale_last_dynamic_boost_step);
        let status_autoscale_scale_up_cooldown_remaining_ms =
            Arc::clone(&autoscale_scale_up_cooldown_remaining_ms);
        let status_autoscale_scale_down_cooldown_remaining_ms =
            Arc::clone(&autoscale_scale_down_cooldown_remaining_ms);
        let status_interval_ms = shared_config.autoscale_check_ms.clamp(100, 1000);
        let status_interval_duration = Duration::from_millis(status_interval_ms);
        let status_tmp_path = status_path.with_extension(format!(
            "{}.tmp",
            status_path
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("json")
        ));
        Some(std::thread::spawn(move || {
            let mut last_saturation_total = status_saturation_events_total.load(Ordering::Relaxed);
            let mut last_dispatch_fallback_total =
                status_relay_dispatch_fallback_total.load(Ordering::Relaxed);
            let mut last_dispatch_saturation_short_circuit_total =
                status_relay_dispatch_saturation_short_circuit_total.load(Ordering::Relaxed);
            let mut last_saturation_sample_at = Instant::now();
            let mut last_status_snapshot: Option<LasmClusterStatusSnapshot> = None;
            let mut status_parent_ready = false;
            loop {
                let sample_now = Instant::now();
                let saturation_total = status_saturation_events_total.load(Ordering::Relaxed);
                let saturation_delta = saturation_total.saturating_sub(last_saturation_total);
                let dispatch_fallback_total =
                    status_relay_dispatch_fallback_total.load(Ordering::Relaxed);
                let dispatch_fallback_delta =
                    dispatch_fallback_total.saturating_sub(last_dispatch_fallback_total);
                let dispatch_saturation_short_circuit_total =
                    status_relay_dispatch_saturation_short_circuit_total.load(Ordering::Relaxed);
                let dispatch_saturation_short_circuit_delta =
                    dispatch_saturation_short_circuit_total
                        .saturating_sub(last_dispatch_saturation_short_circuit_total);
                let elapsed_secs = sample_now
                    .duration_since(last_saturation_sample_at)
                    .as_secs_f64()
                    .max(0.001);
                let saturation_per_sec = (saturation_delta as f64) / elapsed_secs;
                let dispatch_fallback_per_sec = (dispatch_fallback_delta as f64) / elapsed_secs;
                let dispatch_saturation_short_circuit_per_sec =
                    (dispatch_saturation_short_circuit_delta as f64) / elapsed_secs;
                let worker_ports = status_worker_ports.load_full();
                let worker_count = worker_ports.len();
                let active_connections = status_active_connections.load(Ordering::Relaxed);
                let active_connections_per_worker = if worker_count == 0 {
                    0.0
                } else {
                    (active_connections as f64) / (worker_count as f64)
                };
                let status_snapshot = LasmClusterStatusSnapshot {
                    listen_port: status_config.listen_port,
                    min_instances: status_config.min_instances,
                    max_instances: status_config.max_instances,
                    worker_count,
                    relay_worker_count: status_relay_worker_count,
                    relay_queue_capacity: status_relay_queue_capacity,
                    relay_queue_shard_capacity: status_relay_queue_shard_capacity,
                    worker_ports,
                    active_connections,
                    active_connections_per_worker,
                    relay_saturation_events_pending: status_saturation_events
                        .load(Ordering::Relaxed),
                    relay_saturation_events_total: saturation_total,
                    relay_saturation_events_per_sec: saturation_per_sec,
                    relay_accept_batch_max: status_config.cluster_relay_accept_batch_max,
                    relay_pump_batch_max: status_config.cluster_relay_pump_batch_max,
                    relay_accept_workers: status_relay_accept_workers,
                    relay_backend_connect_timeout_ms: status_config
                        .cluster_backend_connect_timeout_ms,
                    relay_backend_connect_cooldown_ms: status_config
                        .cluster_backend_connect_cooldown_ms,
                    relay_dispatch_fallback_total: dispatch_fallback_total,
                    relay_dispatch_fallback_per_sec: dispatch_fallback_per_sec,
                    relay_dispatch_saturation_short_circuit_total:
                        dispatch_saturation_short_circuit_total,
                    relay_dispatch_saturation_short_circuit_per_sec:
                        dispatch_saturation_short_circuit_per_sec,
                    relay_live_sender_count: status_relay_live_sender_count.load(Ordering::Relaxed),
                    autoscale_desired_instances: status_autoscale_last_desired_instances
                        .load(Ordering::Relaxed),
                    autoscale_last_saturation_events: status_autoscale_last_saturation_events
                        .load(Ordering::Relaxed),
                    autoscale_last_dynamic_boost_step: status_autoscale_last_dynamic_boost_step
                        .load(Ordering::Relaxed),
                    autoscale_scale_up_cooldown_remaining_ms:
                        status_autoscale_scale_up_cooldown_remaining_ms.load(Ordering::Relaxed),
                    autoscale_scale_down_cooldown_remaining_ms:
                        status_autoscale_scale_down_cooldown_remaining_ms.load(Ordering::Relaxed),
                };
                if let Err(err) = write_lasm_cluster_status_json(
                    status_path.as_path(),
                    status_tmp_path.as_path(),
                    status_snapshot,
                    &mut last_status_snapshot,
                    &mut status_parent_ready,
                ) {
                    eprintln!("warning: LASM cluster status json write failed: {err}");
                }
                last_saturation_total = saturation_total;
                last_dispatch_fallback_total = dispatch_fallback_total;
                last_dispatch_saturation_short_circuit_total =
                    dispatch_saturation_short_circuit_total;
                last_saturation_sample_at = sample_now;

                if status_stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(status_interval_duration);
            }
        }))
    } else {
        None
    };

    let autoscale_enabled = shared_config.max_instances > shared_config.min_instances;
    let maintenance_interval_ms = lasm_cluster_maintenance_interval_ms(shared_config.as_ref());
    let saturation_priority_interval_ms = maintenance_interval_ms.min(100);
    let autoscale_handle = {
        let autoscale_state = Arc::clone(&shared_state);
        let autoscale_config = Arc::clone(&shared_config);
        let autoscale_active_connections = Arc::clone(&active_connections);
        let autoscale_saturation_events = Arc::clone(&relay_saturation_events);
        let autoscale_stop_flag = Arc::clone(&stop_flag);
        let autoscale_worker_ports = Arc::clone(&worker_ports_snapshot);
        let autoscale_last_desired_instances = Arc::clone(&autoscale_last_desired_instances);
        let autoscale_last_saturation_events = Arc::clone(&autoscale_last_saturation_events);
        let autoscale_last_dynamic_boost_step = Arc::clone(&autoscale_last_dynamic_boost_step);
        let autoscale_scale_up_cooldown_remaining_ms =
            Arc::clone(&autoscale_scale_up_cooldown_remaining_ms);
        let autoscale_scale_down_cooldown_remaining_ms =
            Arc::clone(&autoscale_scale_down_cooldown_remaining_ms);
        std::thread::spawn(move || {
            let autoscale_check_interval =
                Duration::from_millis(autoscale_config.autoscale_check_ms);
            let autoscale_scale_up_cooldown =
                Duration::from_millis(autoscale_config.autoscale_scale_up_cooldown_ms);
            let autoscale_scale_down_cooldown =
                Duration::from_millis(autoscale_config.autoscale_scale_down_cooldown_ms);
            let mut last_scale_up_at: Option<Instant> = None;
            let mut last_scale_down_at: Option<Instant> = None;
            let mut last_published_worker_ports = autoscale_worker_ports.load().as_ref().clone();
            let mut last_scale_eval_at = Instant::now()
                .checked_sub(autoscale_check_interval)
                .unwrap_or_else(Instant::now);
            while !autoscale_stop_flag.load(Ordering::Relaxed) {
                let saturation_pending_before_sleep =
                    autoscale_saturation_events.load(Ordering::Relaxed);
                let sleep_ms = if autoscale_enabled && saturation_pending_before_sleep > 0 {
                    saturation_priority_interval_ms
                } else {
                    maintenance_interval_ms
                };
                std::thread::sleep(Duration::from_millis(sleep_ms));
                if autoscale_stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                let now = Instant::now();
                let mut state = match autoscale_state.write() {
                    Ok(state) => state,
                    Err(_) => break,
                };
                prune_dead_lasm_cluster_workers(&mut state);
                recover_lasm_cluster_min_workers(&mut state, &autoscale_config, "worker recovery");
                refresh_lasm_cluster_worker_ports_snapshot_if_changed(
                    &state,
                    &autoscale_worker_ports,
                    &mut last_published_worker_ports,
                );
                if !autoscale_enabled {
                    autoscale_last_desired_instances.store(state.workers.len(), Ordering::Relaxed);
                    autoscale_last_saturation_events.store(0, Ordering::Relaxed);
                    autoscale_last_dynamic_boost_step.store(
                        autoscale_config.autoscale_scale_up_step.max(1),
                        Ordering::Relaxed,
                    );
                    autoscale_scale_up_cooldown_remaining_ms.store(0, Ordering::Relaxed);
                    autoscale_scale_down_cooldown_remaining_ms.store(0, Ordering::Relaxed);
                    continue;
                }
                autoscale_scale_up_cooldown_remaining_ms.store(
                    lasm_cluster_remaining_cooldown_ms(
                        now,
                        last_scale_up_at,
                        autoscale_config.autoscale_scale_up_cooldown_ms,
                    ),
                    Ordering::Relaxed,
                );
                autoscale_scale_down_cooldown_remaining_ms.store(
                    lasm_cluster_remaining_cooldown_ms(
                        now,
                        last_scale_down_at,
                        autoscale_config.autoscale_scale_down_cooldown_ms,
                    ),
                    Ordering::Relaxed,
                );
                let saturation_events_pending = autoscale_saturation_events.load(Ordering::Relaxed);
                if now.duration_since(last_scale_eval_at) < autoscale_check_interval
                    && saturation_events_pending == 0
                {
                    continue;
                }
                last_scale_eval_at = now;

                let active = autoscale_active_connections.load(Ordering::Relaxed);
                let mut desired = desired_lasm_cluster_instances(
                    active,
                    autoscale_config.min_instances,
                    autoscale_config.max_instances,
                    autoscale_config.target_connections_per_instance,
                );
                let saturation_events = autoscale_saturation_events.swap(0, Ordering::Relaxed);
                let mut scale_up_step_budget = autoscale_config.autoscale_scale_up_step;
                let current_workers = state.workers.len();
                if saturation_events > 0 {
                    let saturation_batch_size = LASM_CLUSTER_SATURATION_COUNTER_FLUSH_BATCH.max(1);
                    let saturation_batches = saturation_events
                        .saturating_add(saturation_batch_size.saturating_sub(1))
                        / saturation_batch_size;
                    let dynamic_boost_step = autoscale_config
                        .autoscale_saturation_boost_step
                        .saturating_mul(saturation_batches.max(1))
                        .max(autoscale_config.autoscale_saturation_boost_step)
                        .max(1)
                        .min(autoscale_config.max_instances);
                    let boosted_target = current_workers
                        .saturating_add(dynamic_boost_step)
                        .min(autoscale_config.max_instances);
                    desired = desired.max(boosted_target);
                    scale_up_step_budget = scale_up_step_budget.max(dynamic_boost_step);
                }
                autoscale_last_desired_instances.store(desired, Ordering::Relaxed);
                autoscale_last_saturation_events.store(saturation_events, Ordering::Relaxed);
                autoscale_last_dynamic_boost_step.store(scale_up_step_budget, Ordering::Relaxed);
                let up_target = if desired > current_workers {
                    desired.min(current_workers.saturating_add(scale_up_step_budget))
                } else {
                    current_workers
                };
                let scale_up_cooldown_elapsed = match last_scale_up_at {
                    Some(at) => now.duration_since(at) >= autoscale_scale_up_cooldown,
                    None => true,
                };
                if desired > state.workers.len() && scale_up_cooldown_elapsed {
                    while state.workers.len() < up_target {
                        let worker_port = state.next_port;
                        state.next_port = state.next_port.saturating_add(1);
                        match spawn_and_wait_lasm_cluster_worker(&autoscale_config, worker_port) {
                            Ok(worker) => state.workers.push(worker),
                            Err(message) => {
                                eprintln!("warning: LASM cluster autoscale-up failed: {message}");
                                break;
                            }
                        }
                    }
                    last_scale_up_at = Some(now);
                }
                let current_workers = state.workers.len();
                let down_target = if desired < current_workers {
                    desired.max(
                        current_workers.saturating_sub(autoscale_config.autoscale_scale_down_step),
                    )
                } else {
                    current_workers
                };
                let scale_down_cooldown_elapsed = match last_scale_down_at {
                    Some(at) => now.duration_since(at) >= autoscale_scale_down_cooldown,
                    None => true,
                };
                if desired < state.workers.len() && scale_down_cooldown_elapsed {
                    while state.workers.len() > down_target {
                        if let Some(mut worker) = state.workers.pop() {
                            let _ = worker.child.kill();
                            let _ = worker.child.wait();
                        }
                    }
                    last_scale_down_at = Some(now);
                }
                refresh_lasm_cluster_worker_ports_snapshot_if_changed(
                    &state,
                    &autoscale_worker_ports,
                    &mut last_published_worker_ports,
                );
                autoscale_scale_up_cooldown_remaining_ms.store(
                    lasm_cluster_remaining_cooldown_ms(
                        now,
                        last_scale_up_at,
                        autoscale_config.autoscale_scale_up_cooldown_ms,
                    ),
                    Ordering::Relaxed,
                );
                autoscale_scale_down_cooldown_remaining_ms.store(
                    lasm_cluster_remaining_cooldown_ms(
                        now,
                        last_scale_down_at,
                        autoscale_config.autoscale_scale_down_cooldown_ms,
                    ),
                    Ordering::Relaxed,
                );
            }
        })
    };

    if let Err(err) = listener.set_nonblocking(true) {
        eprintln!("run failed: could not set LASM cluster proxy listener nonblocking: {err}");
        stop_flag.store(true, Ordering::Relaxed);
        drop(relay_senders);
        for handle in relay_handles {
            let _ = handle.join();
        }
        let _ = autoscale_handle.join();
        if let Some(handle) = status_writer_handle {
            let _ = handle.join();
        }
        if let Ok(mut state) = shared_state.write() {
            stop_lasm_cluster_workers(&mut state);
        }
        return Err(2);
    }

    let accept_error_reported = Arc::new(AtomicBool::new(false));
    let mut accept_handles = Vec::with_capacity(relay_accept_worker_count.saturating_sub(1));
    for accept_worker_index in 1..relay_accept_worker_count {
        let accept_listener = match listener.try_clone() {
            Ok(listener) => listener,
            Err(err) => {
                eprintln!("run failed: could not clone LASM cluster listener: {err}");
                stop_flag.store(true, Ordering::Relaxed);
                drop(relay_senders);
                for handle in relay_handles {
                    let _ = handle.join();
                }
                let _ = autoscale_handle.join();
                if let Some(handle) = status_writer_handle {
                    let _ = handle.join();
                }
                if let Ok(mut state) = shared_state.write() {
                    stop_lasm_cluster_workers(&mut state);
                }
                return Err(2);
            }
        };
        let accept_relay_senders = Arc::clone(&relay_senders);
        let accept_active_connections = Arc::clone(&active_connections);
        let accept_saturation_events = Arc::clone(&relay_saturation_events);
        let accept_saturation_events_total = Arc::clone(&relay_saturation_events_total);
        let accept_dispatch_fallback_total = Arc::clone(&relay_dispatch_fallback_total);
        let accept_dispatch_short_circuit_total =
            Arc::clone(&relay_dispatch_saturation_short_circuit_total);
        let accept_relay_live_sender_count = Arc::clone(&relay_live_sender_count);
        let accept_stop_flag = Arc::clone(&stop_flag);
        let accept_error_reported = Arc::clone(&accept_error_reported);
        accept_handles.push(std::thread::spawn(move || {
            if let Err(message) = run_lasm_cluster_accept_loop(
                &accept_listener,
                accept_relay_senders.as_slice(),
                accept_active_connections.as_ref(),
                accept_saturation_events.as_ref(),
                accept_saturation_events_total.as_ref(),
                accept_relay_live_sender_count.as_ref(),
                accept_worker_index,
                accept_dispatch_fallback_total.as_ref(),
                accept_dispatch_short_circuit_total.as_ref(),
                accept_stop_flag.as_ref(),
                relay_accept_batch_max,
            ) {
                if !accept_error_reported.swap(true, Ordering::Relaxed) {
                    eprintln!("run failed: {message}");
                }
                accept_stop_flag.store(true, Ordering::Relaxed);
            }
        }));
    }

    if let Err(message) = run_lasm_cluster_accept_loop(
        &listener,
        relay_senders.as_slice(),
        active_connections.as_ref(),
        relay_saturation_events.as_ref(),
        relay_saturation_events_total.as_ref(),
        relay_live_sender_count.as_ref(),
        0,
        relay_dispatch_fallback_total.as_ref(),
        relay_dispatch_saturation_short_circuit_total.as_ref(),
        stop_flag.as_ref(),
        relay_accept_batch_max,
    ) {
        if !accept_error_reported.swap(true, Ordering::Relaxed) {
            eprintln!("run failed: {message}");
        }
    }

    stop_flag.store(true, Ordering::Relaxed);
    for handle in accept_handles {
        let _ = handle.join();
    }
    drop(relay_senders);
    for handle in relay_handles {
        let _ = handle.join();
    }
    let _ = autoscale_handle.join();
    if let Some(handle) = status_writer_handle {
        let _ = handle.join();
    }
    if let Ok(mut state) = shared_state.write() {
        stop_lasm_cluster_workers(&mut state);
    }
    Ok(())
}

fn cmd_run_lasm_backend(
    path: &Path,
    manifest: &sec4_core::Manifest,
    policy: &Policy,
    port: Option<u16>,
    oneshot: bool,
    max_header_bytes: Option<u64>,
    max_body_bytes: Option<u64>,
    max_concurrency: Option<u64>,
    max_pending: Option<u64>,
    serve_timeout_ms: Option<u64>,
    overflow_probe_timeout_ms: Option<u64>,
    max_runtime_steps: Option<u64>,
    max_keep_alive_requests: Option<u64>,
    db_base: Option<&Path>,
    db_adapter: Option<RunDbAdapter>,
    db_postgres_dsn: Option<&str>,
    db_max_tx_handles: Option<u64>,
    instances: usize,
    autoscale_max_instances: Option<usize>,
    autoscale_target_connections: Option<usize>,
    autoscale_check_ms: u64,
    autoscale_scale_up_cooldown_ms: u64,
    autoscale_scale_down_cooldown_ms: u64,
    autoscale_scale_up_step: usize,
    autoscale_scale_down_step: usize,
    autoscale_saturation_boost_step: usize,
    cluster_relay_workers: Option<usize>,
    cluster_relay_queue: Option<usize>,
    cluster_accept_workers: Option<usize>,
    cluster_relay_accept_batch_max: Option<usize>,
    cluster_relay_pump_batch_max: Option<usize>,
    cluster_backend_connect_timeout_ms_override: Option<u64>,
    cluster_backend_connect_cooldown_ms_override: Option<u64>,
    cluster_status_json: Option<&Path>,
    reuse_port: bool,
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
    let policy_max_in_flight = match usize::try_from(policy_max_in_flight) {
        Ok(value) if value >= 1 => value,
        Ok(_) => {
            eprintln!("run failed: policy http.max_concurrency must be >= 1");
            return Err(2);
        }
        Err(_) => {
            eprintln!("run failed: policy http.max_concurrency exceeds platform limits");
            return Err(2);
        }
    };
    let effective_max_in_flight =
        match resolve_lasm_max_in_flight(max_concurrency, policy_max_in_flight) {
            Ok(value) => value,
            Err(message) => {
                eprintln!("run failed: {message}");
                return Err(2);
            }
        };
    let policy_max_pending = match u64::try_from(policy.http.max_pending) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: policy http.max_pending must be >= 0");
            return Err(2);
        }
    };
    let policy_max_pending = match usize::try_from(policy_max_pending) {
        Ok(value) if value >= 1 => value,
        Ok(_) => {
            eprintln!("run failed: policy http.max_pending must be >= 1");
            return Err(2);
        }
        Err(_) => {
            eprintln!("run failed: policy http.max_pending exceeds platform limits");
            return Err(2);
        }
    };
    let effective_max_pending = match resolve_lasm_max_pending(max_pending, policy_max_pending) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("run failed: {message}");
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
    let effective_timeout_ms =
        match resolve_lasm_serve_timeout_ms(serve_timeout_ms, policy_timeout_ms) {
            Ok(value) => value,
            Err(message) => {
                eprintln!("run failed: {message}");
                return Err(2);
            }
        };
    let policy_max_header_bytes = match u64::try_from(policy.http.max_header_bytes) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: policy http.max_header_bytes must be >= 0");
            return Err(2);
        }
    };
    let effective_max_header_bytes =
        match resolve_lasm_max_header_bytes(max_header_bytes, policy_max_header_bytes) {
            Ok(value) => value,
            Err(message) => {
                eprintln!("run failed: {message}");
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
    let effective_max_body_bytes =
        match resolve_lasm_max_body_bytes(max_body_bytes, policy_max_body_bytes) {
            Ok(value) => value,
            Err(message) => {
                eprintln!("run failed: {message}");
                return Err(2);
            }
        };
    let policy_max_runtime_steps = match u64::try_from(policy.http.max_runtime_steps) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: policy http.max_runtime_steps must be >= 0");
            return Err(2);
        }
    };
    let policy_max_runtime_steps = match usize::try_from(policy_max_runtime_steps) {
        Ok(value) if value >= 1 => value,
        Ok(_) => {
            eprintln!("run failed: policy http.max_runtime_steps must be >= 1");
            return Err(2);
        }
        Err(_) => {
            eprintln!("run failed: policy http.max_runtime_steps exceeds platform limits");
            return Err(2);
        }
    };
    let runtime_step_budget =
        match resolve_lasm_runtime_step_budget(max_runtime_steps, policy_max_runtime_steps) {
            Ok(value) => value,
            Err(message) => {
                eprintln!("run failed: {message}");
                return Err(2);
            }
        };
    let policy_max_keep_alive_requests = match u64::try_from(policy.http.max_keep_alive_requests) {
        Ok(value) => value,
        Err(_) => {
            eprintln!("run failed: policy http.max_keep_alive_requests must be >= 0");
            return Err(2);
        }
    };
    let policy_max_keep_alive_requests = match usize::try_from(policy_max_keep_alive_requests) {
        Ok(value) if value >= 1 => value,
        Ok(_) => {
            eprintln!("run failed: policy http.max_keep_alive_requests must be >= 1");
            return Err(2);
        }
        Err(_) => {
            eprintln!("run failed: policy http.max_keep_alive_requests exceeds platform limits");
            return Err(2);
        }
    };
    let max_requests_per_connection = match resolve_lasm_max_requests_per_connection(
        max_keep_alive_requests,
        policy_max_keep_alive_requests,
    ) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("run failed: {message}");
            return Err(2);
        }
    };
    let policy_overflow_probe_timeout_ms =
        match u64::try_from(policy.http.overflow_probe_timeout_ms) {
            Ok(value) => value,
            Err(_) => {
                eprintln!("run failed: policy http.overflow_probe_timeout_ms must be >= 0");
                return Err(2);
            }
        };
    if policy_overflow_probe_timeout_ms == 0 {
        eprintln!("run failed: policy http.overflow_probe_timeout_ms must be >= 1");
        return Err(2);
    }
    let overflow_probe_timeout_ms = match resolve_lasm_overflow_probe_timeout_ms(
        overflow_probe_timeout_ms,
        policy_overflow_probe_timeout_ms,
        effective_timeout_ms,
    ) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("run failed: {message}");
            return Err(2);
        }
    };
    let cluster_backend_connect_timeout_ms = resolve_lasm_cluster_backend_connect_timeout_ms(
        cluster_backend_connect_timeout_ms_override,
    );
    let cluster_backend_connect_cooldown_ms = resolve_lasm_cluster_backend_connect_cooldown_ms(
        cluster_backend_connect_cooldown_ms_override,
        cluster_backend_connect_timeout_ms,
    );
    let effective_cluster_relay_accept_batch_max =
        resolve_lasm_cluster_relay_accept_batch_max(cluster_relay_accept_batch_max);
    let effective_cluster_relay_pump_batch_max = resolve_lasm_cluster_relay_pump_batch_max(
        cluster_relay_pump_batch_max,
        effective_cluster_relay_accept_batch_max,
    );

    let max_instances = autoscale_max_instances.unwrap_or(instances);
    let explicit_db_postgres_dsn = db_postgres_dsn.map(ToOwned::to_owned);
    if max_instances < instances {
        eprintln!("run failed: --autoscale-max-instances must be >= --instances");
        return Err(2);
    }
    let cluster_mode = instances > 1 || max_instances > 1;
    if cluster_mode && oneshot {
        eprintln!("run failed: cluster mode does not support --oneshot");
        return Err(2);
    }
    let fixed_cluster_reuse_port_mode = cluster_mode && max_instances == instances;
    if fixed_cluster_reuse_port_mode && cluster_relay_workers.is_some() {
        eprintln!(
            "run failed: --cluster-relay-workers is not used in fixed reuse-port cluster mode"
        );
        return Err(2);
    }
    if fixed_cluster_reuse_port_mode && cluster_relay_queue.is_some() {
        eprintln!("run failed: --cluster-relay-queue is not used in fixed reuse-port cluster mode");
        return Err(2);
    }
    if fixed_cluster_reuse_port_mode && cluster_accept_workers.is_some() {
        eprintln!(
            "run failed: --cluster-accept-workers is not used in fixed reuse-port cluster mode"
        );
        return Err(2);
    }
    if fixed_cluster_reuse_port_mode && cluster_status_json.is_some() {
        eprintln!("run failed: --cluster-status-json is not used in fixed reuse-port cluster mode");
        return Err(2);
    }
    if fixed_cluster_reuse_port_mode && cluster_relay_accept_batch_max.is_some() {
        eprintln!(
            "run failed: --cluster-relay-accept-batch-max is not used in fixed reuse-port cluster mode"
        );
        return Err(2);
    }
    if fixed_cluster_reuse_port_mode && cluster_relay_pump_batch_max.is_some() {
        eprintln!(
            "run failed: --cluster-relay-pump-batch-max is not used in fixed reuse-port cluster mode"
        );
        return Err(2);
    }
    if fixed_cluster_reuse_port_mode && cluster_backend_connect_timeout_ms_override.is_some() {
        eprintln!(
            "run failed: --cluster-backend-connect-timeout-ms is not used in fixed reuse-port cluster mode"
        );
        return Err(2);
    }
    if fixed_cluster_reuse_port_mode && cluster_backend_connect_cooldown_ms_override.is_some() {
        eprintln!(
            "run failed: --cluster-backend-connect-cooldown-ms is not used in fixed reuse-port cluster mode"
        );
        return Err(2);
    }
    if fixed_cluster_reuse_port_mode {
        return cmd_run_lasm_reuseport_cluster(LasmClusterConfig {
            path: path.to_path_buf(),
            listen_port: port.unwrap_or(8080),
            max_header_bytes,
            max_body_bytes,
            max_concurrency,
            max_pending,
            serve_timeout_ms,
            overflow_probe_timeout_ms: Some(overflow_probe_timeout_ms),
            max_runtime_steps,
            max_keep_alive_requests,
            db_base: db_base.map(Path::to_path_buf),
            db_adapter,
            db_postgres_dsn: explicit_db_postgres_dsn.clone(),
            db_max_tx_handles,
            min_instances: instances,
            max_instances,
            target_connections_per_instance: autoscale_target_connections
                .unwrap_or(effective_max_in_flight)
                .max(1),
            autoscale_check_ms,
            autoscale_scale_up_cooldown_ms,
            autoscale_scale_down_cooldown_ms,
            autoscale_scale_up_step,
            autoscale_scale_down_step,
            autoscale_saturation_boost_step,
            worker_ready_timeout_ms: effective_timeout_ms.max(2000),
            cluster_relay_workers: None,
            cluster_relay_queue: None,
            cluster_accept_workers: None,
            cluster_relay_accept_batch_max: effective_cluster_relay_accept_batch_max,
            cluster_relay_pump_batch_max: effective_cluster_relay_pump_batch_max,
            cluster_backend_connect_timeout_ms,
            cluster_backend_connect_cooldown_ms,
            cluster_status_json: None,
            reuse_port_workers: true,
        });
    }
    if cluster_mode {
        let target_connections_per_instance = autoscale_target_connections
            .unwrap_or(effective_max_in_flight)
            .max(1);
        return cmd_run_lasm_cluster(LasmClusterConfig {
            path: path.to_path_buf(),
            listen_port: port.unwrap_or(8080),
            max_header_bytes,
            max_body_bytes,
            max_concurrency,
            max_pending,
            serve_timeout_ms,
            overflow_probe_timeout_ms: Some(overflow_probe_timeout_ms),
            max_runtime_steps,
            max_keep_alive_requests,
            db_base: db_base.map(Path::to_path_buf),
            db_adapter,
            db_postgres_dsn: explicit_db_postgres_dsn.clone(),
            db_max_tx_handles,
            min_instances: instances,
            max_instances,
            target_connections_per_instance,
            autoscale_check_ms,
            autoscale_scale_up_cooldown_ms,
            autoscale_scale_down_cooldown_ms,
            autoscale_scale_up_step,
            autoscale_scale_down_step,
            autoscale_saturation_boost_step,
            worker_ready_timeout_ms: effective_timeout_ms.max(2000),
            cluster_relay_workers,
            cluster_relay_queue,
            cluster_accept_workers,
            cluster_relay_accept_batch_max: effective_cluster_relay_accept_batch_max,
            cluster_relay_pump_batch_max: effective_cluster_relay_pump_batch_max,
            cluster_backend_connect_timeout_ms,
            cluster_backend_connect_cooldown_ms,
            cluster_status_json: cluster_status_json.map(Path::to_path_buf),
            reuse_port_workers: false,
        });
    }

    let routes = collect_lasm_route_plans(&program, entry.name.as_str());
    if routes.is_empty() {
        eprintln!(
            "run failed: no HTTP routes discovered from entry `{}` for LASM backend",
            entry.name
        );
        return Err(1);
    }
    let listen_port = port.unwrap_or(8080);
    let listener = match bind_lasm_listener(listen_port, reuse_port) {
        Ok(listener) => listener,
        Err(message) => {
            eprintln!("run failed: {message}");
            return Err(2);
        }
    };

    let mut worker_handles = Vec::new();
    let trace_counter = Arc::new(AtomicU64::new(0));
    let header_defaults = Arc::new(build_lasm_response_header_defaults(policy));
    let dynamic_state = Arc::new(Mutex::new(
        build_lasm_dynamic_response_state(
            db_base,
            db_adapter.map(run_db_adapter_to_lasm_db_records_adapter),
            explicit_db_postgres_dsn.as_deref(),
            db_max_tx_handles
                .map(|value| usize::try_from(value))
                .transpose()
                .map_err(|_| {
                    eprintln!("run failed: --db-max-tx-handles exceeds platform limits");
                    2
                })?,
        )
        .map_err(|message| {
            eprintln!("run failed: {message}");
            2
        })?,
    ));
    let mut oneshot_runtime = if oneshot {
        Some(
            build_lasm_http_runtime(&routes, effective_timeout_ms, effective_max_pending).map_err(
                |message| {
                    eprintln!("run failed: {message}");
                    2
                },
            )?,
        )
    } else {
        None
    };
    let worker_sender = if oneshot {
        None
    } else {
        let (sender, receiver) = bounded::<TcpStream>(effective_max_pending);
        for _ in 0..effective_max_in_flight {
            let worker_receiver = receiver.clone();
            let routes_for_worker = routes.clone();
            let trace_counter_for_worker = Arc::clone(&trace_counter);
            let header_defaults_for_worker = Arc::clone(&header_defaults);
            let dynamic_state_for_worker = Arc::clone(&dynamic_state);
            worker_handles.push(std::thread::spawn(move || {
                let mut runtime = match build_lasm_http_runtime(
                    &routes_for_worker,
                    effective_timeout_ms,
                    effective_max_pending,
                ) {
                    Ok(runtime) => runtime,
                    Err(message) => {
                        eprintln!("warning: LASM worker bootstrap failed: {message}");
                        return;
                    }
                };
                loop {
                    let mut stream = match worker_receiver.recv() {
                        Ok(stream) => stream,
                        Err(_) => break,
                    };
                    if let Err(message) = process_lasm_connection_with_runtime(
                        &mut stream,
                        &mut runtime,
                        effective_max_header_bytes,
                        effective_max_body_bytes,
                        max_requests_per_connection,
                        runtime_step_budget,
                        true,
                        trace_counter_for_worker.as_ref(),
                        header_defaults_for_worker.as_ref(),
                        dynamic_state_for_worker.as_ref(),
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
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {
                continue;
            }
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
        if let Err(err) =
            stream.set_write_timeout(Some(std::time::Duration::from_millis(effective_timeout_ms)))
        {
            eprintln!("run failed: LASM backend could not set write timeout: {err}");
            return Err(2);
        }
        if let Err(err) = stream.set_nodelay(true) {
            eprintln!("warning: LASM backend could not enable TCP_NODELAY: {err}");
        }

        if oneshot {
            if let Err(message) = process_lasm_connection_with_runtime(
                &mut stream,
                oneshot_runtime
                    .as_mut()
                    .expect("oneshot runtime should be initialized"),
                effective_max_header_bytes,
                effective_max_body_bytes,
                max_requests_per_connection,
                runtime_step_budget,
                false,
                trace_counter.as_ref(),
                header_defaults.as_ref(),
                dynamic_state.as_ref(),
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
                // Keep overflow probing bounded so saturated accept loops do not block
                // for the full request timeout waiting on slow clients.
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(
                    overflow_probe_timeout_ms,
                )));
                let _ = stream.set_write_timeout(Some(std::time::Duration::from_millis(
                    overflow_probe_timeout_ms,
                )));
                let trace_id = next_lasm_trace_id(trace_counter.as_ref());
                match read_lasm_request_head(&mut stream, effective_max_header_bytes) {
                    Ok(request_head) => {
                        let include_cors_defaults = should_include_lasm_cors_defaults(
                            Some(&request_head.headers),
                            header_defaults.as_ref(),
                        );
                        let omit_body = request_head.method.eq_ignore_ascii_case("HEAD");
                        let mut response = sec4_core::HttpResponse::text(503, "");
                        set_lasm_json_response(
                            &mut response,
                            503,
                            &lasm_error_envelope(
                                "HTTP.SERVICE_UNAVAILABLE",
                                "internal",
                                "server busy: max concurrency reached",
                                503,
                                trace_id.as_str(),
                            ),
                        );
                        apply_lasm_request_origin_header(
                            &mut response,
                            Some(&request_head.headers),
                            header_defaults.as_ref(),
                        );
                        set_lasm_trace_id(&mut response, trace_id.as_str());
                        let _ = write_lasm_http_response(
                            &mut stream,
                            &response,
                            header_defaults.as_ref(),
                            include_cors_defaults,
                            omit_body,
                            true,
                        );
                    }
                    Err(err) => {
                        let mut response = sec4_core::HttpResponse::text(503, "");
                        if lasm_overload_head_should_fallback_to_busy(&err) {
                            set_lasm_json_response(
                                &mut response,
                                503,
                                &lasm_error_envelope(
                                    "HTTP.SERVICE_UNAVAILABLE",
                                    "internal",
                                    "server busy: max concurrency reached",
                                    503,
                                    trace_id.as_str(),
                                ),
                            );
                        } else {
                            let (code, kind) = lasm_request_read_error_code_kind(err.status);
                            set_lasm_json_response(
                                &mut response,
                                err.status,
                                &lasm_error_envelope(
                                    code,
                                    kind,
                                    err.message.as_str(),
                                    err.status,
                                    trace_id.as_str(),
                                ),
                            );
                        }
                        set_lasm_trace_id(&mut response, trace_id.as_str());
                        let _ = write_lasm_http_response(
                            &mut stream,
                            &response,
                            header_defaults.as_ref(),
                            false,
                            false,
                            true,
                        );
                    }
                }
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
    max_pending: usize,
) -> Result<sec4_core::LasmHttpRuntime, String> {
    let mut runtime = sec4_core::LasmHttpRuntime::default();
    runtime
        .set_max_in_flight(1)
        .map_err(|message| format!("could not configure LASM runtime max in-flight: {message}"))?;
    runtime
        .set_max_pending(max_pending)
        .map_err(|message| format!("could not configure LASM runtime max pending: {message}"))?;
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
    let auth_mode = lasm_effective_auth_mode(policy.auth.mode.as_str());
    let auth_cookie_name = if let Ok(value) = std::env::var("SEC4_RT_AUTH_COOKIE_NAME") {
        if !value.trim().is_empty() && is_lasm_response_header_name_valid(value.trim()) {
            value.trim().to_string()
        } else if !policy.auth.cookie_name.trim().is_empty()
            && is_lasm_response_header_name_valid(policy.auth.cookie_name.trim())
        {
            policy.auth.cookie_name.trim().to_string()
        } else {
            "session".to_string()
        }
    } else if !policy.auth.cookie_name.trim().is_empty()
        && is_lasm_response_header_name_valid(policy.auth.cookie_name.trim())
    {
        policy.auth.cookie_name.trim().to_string()
    } else {
        "session".to_string()
    };
    let csrf_enabled = lasm_effective_csrf_enabled(policy.csrf.enabled, policy.csrf.mode.as_str());
    let csrf_cookie_name = lasm_effective_csrf_cookie_name(policy.csrf.cookie_name.as_str());
    let csrf_header_name = lasm_effective_csrf_header_name(policy.csrf.header_name.as_str());
    let csrf_protected_methods =
        lasm_effective_csrf_protected_methods(policy.csrf.protected_methods.as_slice());
    let mut cors_allow_any_origin = false;
    let mut cors_allowed_origins = Vec::new();
    let mut cors_allow_any_method = false;
    let mut cors_allowed_methods = Vec::new();
    let mut cors_allow_any_header = false;
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
            .map(|value| {
                let normalized = value.to_ascii_uppercase();
                if normalized == "*" {
                    cors_allow_any_method = true;
                }
                normalized
            })
            .filter(|value| value != "*")
            .collect();
        cors_allowed_headers = policy
            .cors
            .allowed_headers
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| {
                let normalized = value.to_ascii_lowercase();
                if normalized == "*" {
                    cors_allow_any_header = true;
                }
                normalized
            })
            .filter(|value| value != "*")
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
        if policy.security_headers.csp_enabled
            && !policy.security_headers.csp_policy.trim().is_empty()
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
        cors_allow_any_method,
        cors_allowed_methods,
        cors_allow_any_header,
        cors_allowed_headers,
        cors_allow_private_network,
        cors_default_origin,
        auth_mode,
        auth_cookie_name,
        csrf_enabled,
        csrf_cookie_name,
        csrf_header_name,
        csrf_protected_methods,
        headers,
    }
}

fn process_lasm_connection_with_runtime(
    stream: &mut TcpStream,
    runtime: &mut sec4_core::LasmHttpRuntime,
    max_header_bytes: usize,
    max_body_bytes: usize,
    max_requests_per_connection: usize,
    runtime_step_budget: usize,
    allow_keep_alive: bool,
    trace_counter: &AtomicU64,
    header_defaults: &LasmResponseHeaderDefaults,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
) -> Result<(), String> {
    let mut responses_written = 0usize;
    let mut request_reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|err| format!("could not clone stream for LASM request reader: {err}"))?,
    );
    loop {
        let trace_id = next_lasm_trace_id(trace_counter);
        let request =
            match read_lasm_http_request(&mut request_reader, max_header_bytes, max_body_bytes) {
                Ok(request) => request,
                Err(err) => {
                    if err.status == 400 && err.message == "empty request" {
                        return Ok(());
                    }
                    let mut response = sec4_core::HttpResponse::text(err.status, "");
                    let (code, kind) = lasm_request_read_error_code_kind(err.status);
                    set_lasm_json_response(
                        &mut response,
                        err.status,
                        &lasm_error_envelope(
                            code,
                            kind,
                            err.message.as_str(),
                            err.status,
                            trace_id.as_str(),
                        ),
                    );
                    apply_lasm_request_origin_header(&mut response, None, header_defaults);
                    set_lasm_trace_id(&mut response, trace_id.as_str());
                    write_lasm_http_response(
                        stream,
                        &response,
                        header_defaults,
                        false,
                        false,
                        true,
                    )?;
                    return Ok(());
                }
            };

        let close_connection = !allow_keep_alive
            || lasm_should_close_connection(&request)
            || responses_written.saturating_add(1) >= max_requests_per_connection;
        let omit_body = request.method.eq_ignore_ascii_case("HEAD");

        match evaluate_lasm_cors_preflight_request(&request, header_defaults) {
            LasmCorsPreflightDecision::NotPreflight => {}
            LasmCorsPreflightDecision::Accept => {
                let mut response = sec4_core::HttpResponse::text(204, "");
                response.body.clear();
                apply_lasm_request_origin_header(
                    &mut response,
                    Some(&request.headers),
                    header_defaults,
                );
                set_lasm_trace_id(&mut response, trace_id.as_str());
                write_lasm_http_response(
                    stream,
                    &response,
                    header_defaults,
                    true,
                    false,
                    close_connection,
                )?;
                if close_connection {
                    return Ok(());
                }
                responses_written = responses_written.saturating_add(1);
                continue;
            }
            LasmCorsPreflightDecision::Reject { status, message } => {
                let mut response = sec4_core::HttpResponse::text(status, message);
                set_lasm_trace_id(&mut response, trace_id.as_str());
                write_lasm_http_response(
                    stream,
                    &response,
                    header_defaults,
                    false,
                    false,
                    close_connection,
                )?;
                if close_connection {
                    return Ok(());
                }
                responses_written = responses_written.saturating_add(1);
                continue;
            }
        }
        let include_cors_defaults =
            should_include_lasm_cors_defaults(Some(&request.headers), header_defaults);

        let mut runtime_request =
            sec4_core::HttpRequest::new(request.method.clone(), request.path.clone());
        runtime_request.headers = request.headers.clone();
        runtime_request.body = request.body.clone();
        let request_id = runtime.submit(runtime_request);
        let report = runtime.run_until_idle(runtime_step_budget);
        if !report.idle {
            let message = format!(
                "run failed: LASM runtime remained active after step budget ({runtime_step_budget})"
            );
            let mut response = sec4_core::HttpResponse::text(500, "");
            set_lasm_json_response(
                &mut response,
                500,
                &lasm_error_envelope(
                    "LASM.STEP_BUDGET_EXCEEDED",
                    "internal",
                    message.as_str(),
                    500,
                    trace_id.as_str(),
                ),
            );
            apply_lasm_request_origin_header(
                &mut response,
                Some(&request.headers),
                header_defaults,
            );
            set_lasm_trace_id(&mut response, trace_id.as_str());
            write_lasm_http_response(
                stream,
                &response,
                header_defaults,
                include_cors_defaults,
                omit_body,
                close_connection,
            )?;
            runtime.reset_transient_state();
            responses_written = responses_written.saturating_add(1);
            if close_connection {
                return Ok(());
            }
            continue;
        }

        let mut matched = None;
        while let Some(exchange) = runtime.pop_response() {
            if exchange.request_id == request_id {
                matched = Some(exchange);
                break;
            }
        }
        let mut response = if let Some(exchange) = matched {
            let mut response = exchange.response;
            apply_lasm_dynamic_response_materialization(
                &mut response,
                &request,
                &exchange.path_params,
                header_defaults,
                dynamic_state,
                trace_id.as_str(),
            );
            materialize_lasm_internal_runtime_error_envelope(&mut response, trace_id.as_str());
            response
        } else {
            let mut response = sec4_core::HttpResponse::text(500, "");
            set_lasm_json_response(
                &mut response,
                500,
                &lasm_error_envelope(
                    "LASM.MISSING_RESPONSE",
                    "internal",
                    "missing LASM response for request",
                    500,
                    trace_id.as_str(),
                ),
            );
            response
        };
        apply_lasm_request_origin_header(&mut response, Some(&request.headers), header_defaults);
        set_lasm_trace_id(&mut response, trace_id.as_str());
        write_lasm_http_response(
            stream,
            &response,
            header_defaults,
            include_cors_defaults,
            omit_body,
            close_connection,
        )?;
        if close_connection {
            return Ok(());
        }
        responses_written = responses_written.saturating_add(1);
    }
}

fn resolve_lasm_max_in_flight(
    explicit_override: Option<u64>,
    policy_default: usize,
) -> Result<usize, String> {
    if let Some(value) = explicit_override {
        let parsed = usize::try_from(value)
            .map_err(|_| "invalid --max-concurrency: exceeds platform limits".to_string())?;
        if parsed == 0 {
            return Err("invalid --max-concurrency: expected usize >= 1".to_string());
        }
        return Ok(parsed);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_MAX_IN_FLIGHT") else {
        return Ok(policy_default);
    };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(policy_default);
    }
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "invalid SEC4_RT_LASM_MAX_IN_FLIGHT: expected usize >= 1".to_string())?;
    if parsed == 0 {
        return Err("invalid SEC4_RT_LASM_MAX_IN_FLIGHT: expected usize >= 1".to_string());
    }
    Ok(parsed)
}

fn resolve_lasm_max_pending(
    explicit_override: Option<u64>,
    policy_default: usize,
) -> Result<usize, String> {
    if let Some(value) = explicit_override {
        let parsed = usize::try_from(value)
            .map_err(|_| "invalid --max-pending: exceeds platform limits".to_string())?;
        if parsed == 0 {
            return Err("invalid --max-pending: expected usize >= 1".to_string());
        }
        return Ok(parsed);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_MAX_PENDING") else {
        return Ok(policy_default);
    };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(policy_default);
    }
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "invalid SEC4_RT_LASM_MAX_PENDING: expected usize >= 1".to_string())?;
    if parsed == 0 {
        return Err("invalid SEC4_RT_LASM_MAX_PENDING: expected usize >= 1".to_string());
    }
    Ok(parsed)
}

fn resolve_lasm_max_header_bytes(
    explicit_override: Option<u64>,
    policy_default: u64,
) -> Result<usize, String> {
    if let Some(value) = explicit_override {
        let parsed = usize::try_from(value)
            .map_err(|_| "invalid --max-header-bytes: exceeds platform limits".to_string())?;
        if parsed == 0 {
            return Err("invalid --max-header-bytes: expected usize >= 1".to_string());
        }
        return Ok(parsed);
    }
    let policy_default = usize::try_from(policy_default)
        .map_err(|_| "policy http.max_header_bytes exceeds platform limits".to_string())?;
    if policy_default == 0 {
        return Err("policy http.max_header_bytes must be >= 1".to_string());
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_MAX_HEADER_BYTES") else {
        return Ok(policy_default);
    };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(policy_default);
    }
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "invalid SEC4_RT_LASM_MAX_HEADER_BYTES: expected usize >= 1".to_string())?;
    if parsed == 0 {
        return Err("invalid SEC4_RT_LASM_MAX_HEADER_BYTES: expected usize >= 1".to_string());
    }
    Ok(parsed)
}

fn resolve_lasm_max_body_bytes(
    explicit_override: Option<u64>,
    policy_default: u64,
) -> Result<usize, String> {
    if let Some(value) = explicit_override {
        let parsed = usize::try_from(value)
            .map_err(|_| "invalid --max-body-bytes: exceeds platform limits".to_string())?;
        if parsed == 0 {
            return Err("invalid --max-body-bytes: expected usize >= 1".to_string());
        }
        return Ok(parsed);
    }
    let policy_default = usize::try_from(policy_default)
        .map_err(|_| "policy http.max_body_bytes exceeds platform limits".to_string())?;
    if policy_default == 0 {
        return Err("policy http.max_body_bytes must be >= 1".to_string());
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_MAX_BODY_BYTES") else {
        return Ok(policy_default);
    };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(policy_default);
    }
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "invalid SEC4_RT_LASM_MAX_BODY_BYTES: expected usize >= 1".to_string())?;
    if parsed == 0 {
        return Err("invalid SEC4_RT_LASM_MAX_BODY_BYTES: expected usize >= 1".to_string());
    }
    Ok(parsed)
}

fn resolve_lasm_serve_timeout_ms(
    explicit_override: Option<u64>,
    policy_default: u64,
) -> Result<u64, String> {
    if let Some(value) = explicit_override {
        if value == 0 {
            return Err("invalid --serve-timeout-ms: expected u64 >= 1".to_string());
        }
        return Ok(value);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_TIMEOUT_MS") else {
        return Ok(policy_default.max(1));
    };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(policy_default.max(1));
    }
    let parsed = value
        .parse::<u64>()
        .map_err(|_| "invalid SEC4_RT_LASM_TIMEOUT_MS: expected u64 >= 1".to_string())?;
    if parsed == 0 {
        return Err("invalid SEC4_RT_LASM_TIMEOUT_MS: expected u64 >= 1".to_string());
    }
    Ok(parsed)
}

fn resolve_lasm_runtime_step_budget(
    explicit_override: Option<u64>,
    policy_default: usize,
) -> Result<usize, String> {
    if let Some(value) = explicit_override {
        return usize::try_from(value)
            .map_err(|_| "invalid --max-runtime-steps: exceeds platform limits".to_string());
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_MAX_STEPS") else {
        return Ok(policy_default);
    };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(policy_default);
    }
    let parsed = value
        .parse::<usize>()
        .map_err(|_| "invalid SEC4_RT_LASM_MAX_STEPS: expected usize >= 1".to_string())?;
    if parsed == 0 {
        return Err("invalid SEC4_RT_LASM_MAX_STEPS: expected usize >= 1".to_string());
    }
    Ok(parsed)
}

fn resolve_lasm_overflow_probe_timeout_ms(
    explicit_override: Option<u64>,
    policy_default: u64,
    effective_timeout_ms: u64,
) -> Result<u64, String> {
    let fallback = policy_default.min(effective_timeout_ms).max(1);
    if let Some(value) = explicit_override {
        if value == 0 {
            return Err("invalid --overflow-probe-timeout-ms: expected u64 >= 1".to_string());
        }
        return Ok(value.min(effective_timeout_ms).max(1));
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_OVERFLOW_PROBE_TIMEOUT_MS") else {
        return Ok(fallback);
    };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(fallback);
    }
    let parsed = value.parse::<u64>().map_err(|_| {
        "invalid SEC4_RT_LASM_OVERFLOW_PROBE_TIMEOUT_MS: expected u64 >= 1".to_string()
    })?;
    if parsed == 0 {
        return Err(
            "invalid SEC4_RT_LASM_OVERFLOW_PROBE_TIMEOUT_MS: expected u64 >= 1".to_string(),
        );
    }
    Ok(parsed.min(effective_timeout_ms).max(1))
}

fn resolve_lasm_cluster_backend_connect_timeout_ms(explicit_override: Option<u64>) -> u64 {
    let default_value = 100_u64;
    if let Some(value) = explicit_override {
        return value.clamp(25, 5_000);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_CLUSTER_BACKEND_CONNECT_TIMEOUT_MS") else {
        return default_value;
    };
    let value = raw.trim();
    if value.is_empty() {
        return default_value;
    }
    value
        .parse::<u64>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .map(|parsed| parsed.clamp(25, 5_000))
        .unwrap_or(default_value)
}

fn resolve_lasm_cluster_backend_connect_cooldown_ms(
    explicit_override: Option<u64>,
    connect_timeout_ms: u64,
) -> u64 {
    let default_value = connect_timeout_ms.saturating_mul(2).clamp(150, 2_000);
    if let Some(value) = explicit_override {
        return value.clamp(25, 10_000);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_CLUSTER_BACKEND_CONNECT_COOLDOWN_MS") else {
        return default_value;
    };
    let value = raw.trim();
    if value.is_empty() {
        return default_value;
    }
    value
        .parse::<u64>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .map(|parsed| parsed.clamp(25, 10_000))
        .unwrap_or(default_value)
}

fn resolve_lasm_cluster_relay_accept_batch_max(explicit_override: Option<usize>) -> usize {
    let default_value = 64_usize;
    if let Some(value) = explicit_override {
        return value.clamp(1, 4_096);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_CLUSTER_RELAY_ACCEPT_BATCH_MAX") else {
        return default_value;
    };
    let value = raw.trim();
    if value.is_empty() {
        return default_value;
    }
    value
        .parse::<usize>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .map(|parsed| parsed.clamp(1, 4_096))
        .unwrap_or(default_value)
}

fn resolve_lasm_cluster_relay_pump_batch_max(
    explicit_override: Option<usize>,
    relay_accept_batch_max: usize,
) -> usize {
    let default_value = relay_accept_batch_max
        .saturating_mul(LASM_CLUSTER_RELAY_PUMP_BATCH_MULTIPLIER)
        .clamp(
            LASM_CLUSTER_RELAY_PUMP_BATCH_MIN,
            LASM_CLUSTER_RELAY_PUMP_BATCH_MAX,
        );
    if let Some(value) = explicit_override {
        return value.clamp(1, LASM_CLUSTER_RELAY_PUMP_BATCH_MAX);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_CLUSTER_RELAY_PUMP_BATCH_MAX") else {
        return default_value;
    };
    let value = raw.trim();
    if value.is_empty() {
        return default_value;
    }
    value
        .parse::<usize>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .map(|parsed| parsed.clamp(1, LASM_CLUSTER_RELAY_PUMP_BATCH_MAX))
        .unwrap_or(default_value)
}

fn resolve_lasm_max_requests_per_connection(
    explicit_override: Option<u64>,
    policy_default: usize,
) -> Result<usize, String> {
    if let Some(value) = explicit_override {
        let parsed = usize::try_from(value).map_err(|_| {
            "invalid --max-keep-alive-requests: exceeds platform limits".to_string()
        })?;
        if parsed == 0 {
            return Err("invalid --max-keep-alive-requests: expected usize >= 1".to_string());
        }
        return Ok(parsed);
    }
    let Ok(raw) = std::env::var("SEC4_RT_LASM_MAX_KEEP_ALIVE_REQUESTS") else {
        return Ok(policy_default);
    };
    let value = raw.trim();
    if value.is_empty() {
        return Ok(policy_default);
    }
    let parsed = value.parse::<usize>().map_err(|_| {
        "invalid SEC4_RT_LASM_MAX_KEEP_ALIVE_REQUESTS: expected usize >= 1".to_string()
    })?;
    if parsed == 0 {
        return Err(
            "invalid SEC4_RT_LASM_MAX_KEEP_ALIVE_REQUESTS: expected usize >= 1".to_string(),
        );
    }
    Ok(parsed)
}

fn apply_lasm_dynamic_response_materialization(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    header_defaults: &LasmResponseHeaderDefaults,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    trace_id: &str,
) {
    if apply_lasm_auth_requirement_enforcement(
        response,
        request,
        path_params,
        header_defaults,
        trace_id,
    ) {
        clear_lasm_internal_response_markers(response);
        return;
    }
    if apply_lasm_csrf_requirement_enforcement(response, request, header_defaults, trace_id) {
        clear_lasm_internal_response_markers(response);
        return;
    }

    apply_lasm_text_placeholder_materialization(response, request, path_params);
    apply_lasm_header_placeholder_materialization(response, request, path_params);
    if apply_lasm_internal_db_operation_materialization(
        response,
        request,
        path_params,
        dynamic_state,
        trace_id,
    ) {
        clear_lasm_internal_response_markers(response);
        return;
    }

    let Some(schema_hint) = extract_lasm_response_schema_hint(response) else {
        return;
    };

    match schema_hint.as_str() {
        "DecodeResponse" => {
            if request.body.is_empty() && !lasm_request_expects_json(request) {
                return;
            }
            if !request.body.is_empty() && !lasm_request_expects_json(request) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "HTTP.BAD_REQUEST",
                        "validation",
                        "content-type must be application/json",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
            let payload = match parse_lasm_json_payload(&request.body) {
                Some(payload) => payload,
                None => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "JSON.INVALID_SYNTAX",
                            "validation",
                            "invalid JSON payload",
                            400,
                            trace_id,
                        ),
                    );
                    return;
                }
            };
            if let Some((code, message)) = validate_lasm_benchmark_user_payload(&payload) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(code, "validation", message, 400, trace_id),
                );
                return;
            }
            let Some(id) = extract_lasm_payload_id(&payload) else {
                return;
            };
            set_lasm_json_response(
                response,
                200,
                &serde_json::json!({
                    "ok": true,
                    "id": id,
                }),
            );
        }
        "CreateUserResponse" => {
            if request.body.is_empty() && !lasm_request_expects_json(request) {
                return;
            }
            if !request.body.is_empty() && !lasm_request_expects_json(request) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "HTTP.BAD_REQUEST",
                        "validation",
                        "content-type must be application/json",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
            let payload = match parse_lasm_json_payload(&request.body) {
                Some(payload) => payload,
                None => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "JSON.INVALID_SYNTAX",
                            "validation",
                            "invalid JSON payload",
                            400,
                            trace_id,
                        ),
                    );
                    return;
                }
            };
            if let Some((code, message)) = validate_lasm_benchmark_user_payload(&payload) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(code, "validation", message, 400, trace_id),
                );
                return;
            }
            let Some(id) = extract_lasm_payload_id(&payload) else {
                return;
            };
            match dynamic_state.lock() {
                Ok(mut state) => {
                    if state.users_by_id.contains_key(&id) {
                        set_lasm_json_response(
                            response,
                            409,
                            &lasm_error_envelope(
                                "HTTP.CONFLICT",
                                "conflict",
                                "user already exists",
                                409,
                                trace_id,
                            ),
                        );
                        return;
                    }
                    state.users_by_id.insert(id.clone(), payload);
                    if let Err(message) = persist_lasm_dynamic_users_to_disk(&state) {
                        eprintln!(
                            "warning: LASM dynamic users store persistence failed: {message}"
                        );
                    }
                }
                Err(_) => {
                    set_lasm_json_response(
                        response,
                        500,
                        &lasm_error_envelope(
                            "HTTP.INTERNAL",
                            "internal",
                            "dynamic response state unavailable",
                            500,
                            trace_id,
                        ),
                    );
                    return;
                }
            }
            set_lasm_json_response(
                response,
                201,
                &serde_json::json!({
                    "ok": true,
                    "userId": id,
                }),
            );
        }
        "UpdateUserResponse" => {
            let Some(path_id) = resolve_lasm_user_lookup_id(request, path_params) else {
                return;
            };
            if !is_lasm_uuid_v4(&path_id) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "VALIDATION.UUID_INVALID",
                        "validation",
                        "id must be UUID v4",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
            if request.body.is_empty() && !lasm_request_expects_json(request) {
                return;
            }
            if !request.body.is_empty() && !lasm_request_expects_json(request) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "HTTP.BAD_REQUEST",
                        "validation",
                        "content-type must be application/json",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
            let payload = match parse_lasm_json_payload(&request.body) {
                Some(payload) => payload,
                None => {
                    set_lasm_json_response(
                        response,
                        400,
                        &lasm_error_envelope(
                            "JSON.INVALID_SYNTAX",
                            "validation",
                            "invalid JSON payload",
                            400,
                            trace_id,
                        ),
                    );
                    return;
                }
            };
            if let Some((code, message)) = validate_lasm_benchmark_user_payload(&payload) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(code, "validation", message, 400, trace_id),
                );
                return;
            }
            let Some(payload_id) = extract_lasm_payload_id(&payload) else {
                return;
            };
            if payload_id != path_id {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "VALIDATION.UUID_MISMATCH",
                        "validation",
                        "path id must match payload id",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
            let updated = match dynamic_state.lock() {
                Ok(mut state) => {
                    if !state.users_by_id.contains_key(&path_id) {
                        false
                    } else {
                        state.users_by_id.insert(path_id.clone(), payload);
                        if let Err(message) = persist_lasm_dynamic_users_to_disk(&state) {
                            eprintln!(
                                "warning: LASM dynamic users store persistence failed: {message}"
                            );
                        }
                        true
                    }
                }
                Err(_) => {
                    set_lasm_json_response(
                        response,
                        500,
                        &lasm_error_envelope(
                            "HTTP.INTERNAL",
                            "internal",
                            "dynamic response state unavailable",
                            500,
                            trace_id,
                        ),
                    );
                    return;
                }
            };
            if updated {
                set_lasm_json_response(
                    response,
                    200,
                    &serde_json::json!({
                        "ok": true,
                        "userId": path_id,
                        "updated": true,
                    }),
                );
            } else {
                set_lasm_json_response(
                    response,
                    404,
                    &lasm_error_envelope(
                        "HTTP.NOT_FOUND",
                        "not_found",
                        "user not found",
                        404,
                        trace_id,
                    ),
                );
            }
        }
        "UserResponse" => {
            let Some(id) = resolve_lasm_user_lookup_id(request, path_params) else {
                return;
            };
            if !is_lasm_uuid_v4(&id) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "VALIDATION.UUID_INVALID",
                        "validation",
                        "id must be UUID v4",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
            let user = match dynamic_state.lock() {
                Ok(state) => state.users_by_id.get(&id).cloned(),
                Err(_) => {
                    set_lasm_json_response(
                        response,
                        500,
                        &lasm_error_envelope(
                            "HTTP.INTERNAL",
                            "internal",
                            "dynamic response state unavailable",
                            500,
                            trace_id,
                        ),
                    );
                    return;
                }
            };
            match user {
                Some(user) => set_lasm_json_response(response, 200, &user),
                None => set_lasm_json_response(
                    response,
                    404,
                    &lasm_error_envelope(
                        "HTTP.NOT_FOUND",
                        "not_found",
                        "user not found",
                        404,
                        trace_id,
                    ),
                ),
            }
        }
        "UserByEmailResponse" => {
            let Some(email) = resolve_lasm_user_lookup_email(request) else {
                return;
            };
            if !is_lasm_email(&email) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "VALIDATION.INVALID",
                        "validation",
                        "email must be a valid email string",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
            let user = match dynamic_state.lock() {
                Ok(state) => state
                    .users_by_id
                    .iter()
                    .filter_map(|(id, user)| {
                        user.get("email")
                            .and_then(serde_json::Value::as_str)
                            .filter(|candidate| candidate.eq_ignore_ascii_case(email.as_str()))
                            .map(|_| (id.as_str(), user))
                    })
                    .min_by_key(|(id, _)| *id)
                    .map(|(_, user)| user.clone()),
                Err(_) => {
                    set_lasm_json_response(
                        response,
                        500,
                        &lasm_error_envelope(
                            "HTTP.INTERNAL",
                            "internal",
                            "dynamic response state unavailable",
                            500,
                            trace_id,
                        ),
                    );
                    return;
                }
            };
            match user {
                Some(user) => set_lasm_json_response(response, 200, &user),
                None => set_lasm_json_response(
                    response,
                    404,
                    &lasm_error_envelope(
                        "HTTP.NOT_FOUND",
                        "not_found",
                        "user not found",
                        404,
                        trace_id,
                    ),
                ),
            }
        }
        "DeleteUserResponse" => {
            let Some(id) = resolve_lasm_user_lookup_id(request, path_params) else {
                return;
            };
            if !is_lasm_uuid_v4(&id) {
                set_lasm_json_response(
                    response,
                    400,
                    &lasm_error_envelope(
                        "VALIDATION.UUID_INVALID",
                        "validation",
                        "id must be UUID v4",
                        400,
                        trace_id,
                    ),
                );
                return;
            }
            let deleted = match dynamic_state.lock() {
                Ok(mut state) => {
                    let deleted = state.users_by_id.remove(&id).is_some();
                    if deleted {
                        if let Err(message) = persist_lasm_dynamic_users_to_disk(&state) {
                            eprintln!(
                                "warning: LASM dynamic users store persistence failed: {message}"
                            );
                        }
                    }
                    deleted
                }
                Err(_) => {
                    set_lasm_json_response(
                        response,
                        500,
                        &lasm_error_envelope(
                            "HTTP.INTERNAL",
                            "internal",
                            "dynamic response state unavailable",
                            500,
                            trace_id,
                        ),
                    );
                    return;
                }
            };
            if deleted {
                set_lasm_json_response(
                    response,
                    200,
                    &serde_json::json!({
                        "ok": true,
                        "userId": id,
                        "deleted": true,
                    }),
                );
            } else {
                set_lasm_json_response(
                    response,
                    404,
                    &lasm_error_envelope(
                        "HTTP.NOT_FOUND",
                        "not_found",
                        "user not found",
                        404,
                        trace_id,
                    ),
                );
            }
        }
        "ListUsersResponse" => {
            let users = match dynamic_state.lock() {
                Ok(state) => {
                    let mut ordered = state
                        .users_by_id
                        .iter()
                        .map(|(id, user)| (id.clone(), user.clone()))
                        .collect::<Vec<_>>();
                    ordered.sort_by(|left, right| left.0.cmp(&right.0));
                    ordered
                        .into_iter()
                        .map(|(_, user)| user)
                        .collect::<Vec<_>>()
                }
                Err(_) => {
                    set_lasm_json_response(
                        response,
                        500,
                        &lasm_error_envelope(
                            "HTTP.INTERNAL",
                            "internal",
                            "dynamic response state unavailable",
                            500,
                            trace_id,
                        ),
                    );
                    return;
                }
            };
            set_lasm_json_response(
                response,
                200,
                &serde_json::json!({
                    "ok": true,
                    "count": users.len(),
                    "users": users,
                }),
            );
        }
        "DbListRecordsResponse" => {
            let (records, adapter, tx_handle_count, tx_handle_capacity) = match dynamic_state.lock()
            {
                Ok(state) => (
                    state.db_records.clone(),
                    lasm_db_records_adapter_label(state.db_records_adapter),
                    state.db_tx_handles.len(),
                    state.db_tx_max_handles,
                ),
                Err(_) => {
                    set_lasm_json_response(
                        response,
                        500,
                        &lasm_error_envelope(
                            "HTTP.INTERNAL",
                            "internal",
                            "dynamic response state unavailable",
                            500,
                            trace_id,
                        ),
                    );
                    return;
                }
            };
            let affected_rows_total = records
                .iter()
                .fold(0u64, |acc, record| acc.saturating_add(record.affected_rows));
            set_lasm_json_response(
                response,
                200,
                &serde_json::json!({
                    "ok": true,
                    "count": records.len(),
                    "affectedRowsTotal": affected_rows_total,
                    "adapter": adapter,
                    "txHandleCount": tx_handle_count,
                    "txHandleCapacity": tx_handle_capacity,
                    "records": records.iter().map(lasm_db_record_to_json).collect::<Vec<_>>(),
                }),
            );
        }
        _ => {}
    }
}

fn apply_lasm_auth_requirement_enforcement(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    header_defaults: &LasmResponseHeaderDefaults,
    trace_id: &str,
) -> bool {
    let requires_auth_helper = response
        .headers
        .remove(LASM_INTERNAL_AUTH_REQUIRE_HEADER)
        .is_some();
    let requires_auth_middleware = response
        .headers
        .remove(LASM_INTERNAL_AUTH_MIDDLEWARE_REQUIRE_HEADER)
        .is_some();
    let required_role = response
        .headers
        .remove(LASM_INTERNAL_AUTH_REQUIRE_ROLE_HEADER)
        .map(|template| {
            if contains_lasm_request_placeholder_tokens(template.as_str()) {
                materialize_lasm_request_placeholders(template.as_str(), request, path_params)
            } else {
                template
            }
        })
        .map(|value| value.trim().to_string())
        .map(|value| {
            if value.is_empty() {
                "role".to_string()
            } else {
                value
            }
        });
    if !requires_auth_helper && !requires_auth_middleware && required_role.is_none() {
        return false;
    }

    let auth_mode = normalize_lasm_auth_mode(header_defaults.auth_mode.as_str());
    if auth_mode.eq_ignore_ascii_case("off") && !requires_auth_helper && required_role.is_none() {
        return false;
    }
    let Some((subject, used_bearer)) = lasm_collect_auth_subject(
        request,
        auth_mode,
        header_defaults.auth_cookie_name.as_str(),
    ) else {
        set_lasm_json_response(
            response,
            401,
            &lasm_error_envelope(
                "AUTH.UNAUTHORIZED",
                "auth",
                lasm_auth_unauthorized_message(auth_mode),
                401,
                trace_id,
            ),
        );
        return true;
    };

    if let Some(required_role) = required_role {
        let has_required_role = if used_bearer {
            lasm_bearer_token_has_role(subject.as_str(), required_role.as_str())
        } else {
            lasm_auth_cookie_has_role(&request.headers, required_role.as_str())
        };
        if !has_required_role {
            let message = if used_bearer {
                "Authorization token missing required role"
            } else {
                "Authenticated cookie principal missing required role"
            };
            set_lasm_json_response(
                response,
                403,
                &lasm_error_envelope("AUTH.FORBIDDEN", "auth", message, 403, trace_id),
            );
            return true;
        }
    }

    false
}

fn apply_lasm_csrf_requirement_enforcement(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    header_defaults: &LasmResponseHeaderDefaults,
    trace_id: &str,
) -> bool {
    let requires_csrf = response
        .headers
        .remove(LASM_INTERNAL_CSRF_REQUIRE_HEADER)
        .is_some();
    if !requires_csrf || !header_defaults.csrf_enabled {
        return false;
    }
    if !lasm_is_csrf_protected_method(
        request.method.as_str(),
        header_defaults.csrf_protected_methods.as_slice(),
    ) {
        return false;
    }

    let csrf_header =
        find_lasm_header_value(&request.headers, header_defaults.csrf_header_name.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty());
    let csrf_cookie =
        find_lasm_cookie_value(&request.headers, header_defaults.csrf_cookie_name.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
    if csrf_header.is_none() || csrf_cookie.is_none() || csrf_header != csrf_cookie.as_deref() {
        set_lasm_json_response(
            response,
            403,
            &lasm_error_envelope(
                "AUTH.CSRF_TOKEN_INVALID",
                "auth",
                "CSRF token missing or invalid",
                403,
                trace_id,
            ),
        );
        return true;
    }

    false
}

fn normalize_lasm_auth_mode(mode: &str) -> &str {
    let mode = mode.trim();
    if mode.eq_ignore_ascii_case("off") {
        return "off";
    }
    if mode.eq_ignore_ascii_case("cookie") {
        return "cookie";
    }
    if mode.eq_ignore_ascii_case("mixed") {
        return "mixed";
    }
    "token"
}

fn is_lasm_supported_auth_mode(mode: &str) -> bool {
    let mode = mode.trim();
    mode.eq_ignore_ascii_case("off")
        || mode.eq_ignore_ascii_case("token")
        || mode.eq_ignore_ascii_case("cookie")
        || mode.eq_ignore_ascii_case("mixed")
}

fn lasm_effective_auth_mode(policy_mode: &str) -> String {
    if let Ok(env_mode) = std::env::var("SEC4_RT_AUTH_MODE") {
        if is_lasm_supported_auth_mode(env_mode.as_str()) {
            return normalize_lasm_auth_mode(env_mode.as_str()).to_string();
        }
    }
    normalize_lasm_auth_mode(policy_mode).to_string()
}

fn parse_lasm_env_bool(name: &str) -> Option<bool> {
    let value = std::env::var(name).ok()?;
    let normalized = value.trim();
    if normalized.eq_ignore_ascii_case("1")
        || normalized.eq_ignore_ascii_case("true")
        || normalized.eq_ignore_ascii_case("yes")
        || normalized.eq_ignore_ascii_case("on")
    {
        return Some(true);
    }
    if normalized.eq_ignore_ascii_case("0")
        || normalized.eq_ignore_ascii_case("false")
        || normalized.eq_ignore_ascii_case("no")
        || normalized.eq_ignore_ascii_case("off")
    {
        return Some(false);
    }
    None
}

fn lasm_effective_csrf_enabled(policy_enabled: bool, policy_mode: &str) -> bool {
    let mut enabled = policy_enabled && !policy_mode.trim().eq_ignore_ascii_case("off");
    if let Some(env_enabled) = parse_lasm_env_bool("SEC4_RT_CSRF_ENABLED") {
        enabled = env_enabled;
    }
    if let Ok(mode) = std::env::var("SEC4_RT_CSRF_MODE") {
        if mode.trim().eq_ignore_ascii_case("off") {
            enabled = false;
        }
    }
    enabled
}

fn lasm_effective_csrf_cookie_name(policy_cookie_name: &str) -> String {
    if let Ok(value) = std::env::var("SEC4_RT_CSRF_COOKIE_NAME") {
        if !value.trim().is_empty() && is_lasm_response_header_name_valid(value.trim()) {
            return value.trim().to_string();
        }
    }
    if !policy_cookie_name.trim().is_empty()
        && is_lasm_response_header_name_valid(policy_cookie_name.trim())
    {
        return policy_cookie_name.trim().to_string();
    }
    "csrf".to_string()
}

fn lasm_effective_csrf_header_name(policy_header_name: &str) -> String {
    if let Ok(value) = std::env::var("SEC4_RT_CSRF_HEADER_NAME") {
        if !value.trim().is_empty() && is_lasm_response_header_name_valid(value.trim()) {
            return value.trim().to_string();
        }
    }
    if !policy_header_name.trim().is_empty()
        && is_lasm_response_header_name_valid(policy_header_name.trim())
    {
        return policy_header_name.trim().to_string();
    }
    "X-CSRF-Token".to_string()
}

fn lasm_effective_csrf_protected_methods(policy_methods: &[String]) -> Vec<String> {
    if let Ok(raw) = std::env::var("SEC4_RT_CSRF_PROTECTED_METHODS") {
        let env_methods = raw
            .split(',')
            .map(|value| value.trim().to_string())
            .collect::<Vec<_>>();
        return normalize_lasm_csrf_protected_methods(env_methods.as_slice());
    }
    normalize_lasm_csrf_protected_methods(policy_methods)
}

fn lasm_auth_mode_allows_token(mode: &str) -> bool {
    mode.eq_ignore_ascii_case("token") || mode.eq_ignore_ascii_case("mixed")
}

fn lasm_auth_mode_allows_cookie(mode: &str) -> bool {
    mode.eq_ignore_ascii_case("cookie") || mode.eq_ignore_ascii_case("mixed")
}

fn normalize_lasm_csrf_protected_methods(methods: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for value in methods {
        let method = value.trim().to_ascii_uppercase();
        if method.is_empty() {
            continue;
        }
        if !normalized.contains(&method) {
            normalized.push(method);
        }
    }
    if normalized
        .iter()
        .any(|method| matches!(method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE"))
    {
        normalized
    } else {
        vec![
            "POST".to_string(),
            "PUT".to_string(),
            "PATCH".to_string(),
            "DELETE".to_string(),
        ]
    }
}

fn lasm_is_csrf_protected_method(method: &str, protected_methods: &[String]) -> bool {
    protected_methods
        .iter()
        .any(|entry| entry.eq_ignore_ascii_case(method))
}

fn lasm_auth_unauthorized_message(mode: &str) -> &'static str {
    if lasm_auth_mode_allows_token(mode) && lasm_auth_mode_allows_cookie(mode) {
        return "Authorization header or session cookie missing or invalid";
    }
    if lasm_auth_mode_allows_cookie(mode) {
        return "Session cookie missing or invalid";
    }
    "Authorization header missing or invalid"
}

fn lasm_collect_auth_subject(
    request: &LasmRunRequest,
    mode: &str,
    auth_cookie_name: &str,
) -> Option<(String, bool)> {
    if lasm_auth_mode_allows_token(mode) {
        if let Some(auth_header) = find_lasm_header_value(&request.headers, "Authorization") {
            if lasm_is_valid_bearer_auth(auth_header) {
                return Some((auth_header.to_string(), true));
            }
        }
    }

    if lasm_auth_mode_allows_cookie(mode) {
        if let Some(session_cookie) = find_lasm_cookie_value(&request.headers, auth_cookie_name) {
            if !session_cookie.trim().is_empty() {
                return Some((session_cookie, false));
            }
        }
    }

    None
}

fn lasm_is_valid_bearer_auth(auth_header: &str) -> bool {
    let bytes = auth_header.as_bytes();
    bytes.len() > 7 && bytes[..7].eq_ignore_ascii_case(b"Bearer ") && !bytes[7..].is_empty()
}

fn lasm_bearer_token_has_role(auth_header: &str, required_role: &str) -> bool {
    if !lasm_is_valid_bearer_auth(auth_header) || required_role.trim().is_empty() {
        return false;
    }
    let token = &auth_header[7..];
    let required_role = required_role.trim();
    let mut cursor = 0usize;
    while let Some(found_at) = token[cursor..].find(required_role) {
        let start = cursor + found_at;
        let end = start + required_role.len();
        let before = if start == 0 {
            ' '
        } else {
            token.as_bytes()[start - 1] as char
        };
        let after = if end >= token.len() {
            '\0'
        } else {
            token.as_bytes()[end] as char
        };
        let before_ok = matches!(before, ' ' | ',' | ':' | '=' | ';');
        let after_ok = matches!(after, '\0' | ' ' | ',' | ':' | '=' | ';');
        if before_ok && after_ok {
            return true;
        }
        cursor = start + 1;
    }
    false
}

fn lasm_auth_cookie_has_role(headers: &BTreeMap<String, String>, required_role: &str) -> bool {
    let required_role = required_role.trim();
    if required_role.is_empty() {
        return false;
    }
    if let Some(role_cookie) = find_lasm_cookie_value(headers, "role") {
        if role_cookie.trim() == required_role {
            return true;
        }
    }
    if let Some(role_header) = find_lasm_header_value(headers, "X-Role") {
        if role_header.trim() == required_role {
            return true;
        }
    }
    false
}

fn apply_lasm_text_placeholder_materialization(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
) {
    if response.body.is_empty() {
        return;
    }
    let original = String::from_utf8_lossy(&response.body);
    if !contains_lasm_request_placeholder_tokens(original.as_ref()) {
        return;
    }
    let materialized =
        materialize_lasm_request_placeholders(original.as_ref(), request, path_params);
    if materialized != original {
        response.body = materialized.into_bytes();
    }
}

fn apply_lasm_header_placeholder_materialization(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
) {
    let mut materialized_headers = BTreeMap::new();
    for (name, value) in std::mem::take(&mut response.headers) {
        let materialized_name = if contains_lasm_request_placeholder_tokens(name.as_str()) {
            materialize_lasm_request_placeholders(name.as_str(), request, path_params)
        } else {
            name
        };
        if materialized_name.trim().is_empty()
            || materialized_name != materialized_name.trim()
            || !is_lasm_response_header_name_valid(materialized_name.as_str())
        {
            continue;
        }
        let materialized_value = if contains_lasm_request_placeholder_tokens(value.as_str()) {
            materialize_lasm_request_placeholders(value.as_str(), request, path_params)
        } else {
            value
        };
        if materialized_name.eq_ignore_ascii_case("Set-Cookie") {
            for cookie in materialized_value.split('\n') {
                if !is_lasm_response_header_value_valid(cookie) {
                    continue;
                }
                append_lasm_set_cookie_header(&mut materialized_headers, cookie);
            }
            continue;
        }
        if !is_lasm_response_header_value_valid(materialized_value.as_str()) {
            continue;
        }
        materialized_headers.insert(materialized_name, materialized_value);
    }
    response.headers = materialized_headers;
}

fn clear_lasm_internal_response_markers(response: &mut sec4_core::HttpResponse) {
    response.headers.remove(LASM_INTERNAL_AUTH_REQUIRE_HEADER);
    response
        .headers
        .remove(LASM_INTERNAL_AUTH_REQUIRE_ROLE_HEADER);
    response
        .headers
        .remove(LASM_INTERNAL_AUTH_MIDDLEWARE_REQUIRE_HEADER);
    response.headers.remove(LASM_INTERNAL_CSRF_REQUIRE_HEADER);
    response
        .headers
        .remove(LASM_INTERNAL_RUNTIME_ERROR_CODE_HEADER);
    clear_lasm_internal_db_response_markers(&mut response.headers);
}

fn materialize_lasm_internal_runtime_error_envelope(
    response: &mut sec4_core::HttpResponse,
    trace_id: &str,
) {
    let Some(header_key) = find_lasm_header_key_case_insensitive(
        &response.headers,
        LASM_INTERNAL_RUNTIME_ERROR_CODE_HEADER,
    ) else {
        return;
    };
    let code = response.headers.remove(&header_key).unwrap_or_default();
    if code.trim().is_empty() {
        return;
    }
    let message = String::from_utf8_lossy(&response.body).to_string();
    let kind = lasm_internal_error_kind_for_code(code.as_str(), response.status);
    set_lasm_json_response(
        response,
        response.status,
        &lasm_error_envelope(
            code.as_str(),
            kind,
            message.as_str(),
            response.status,
            trace_id,
        ),
    );
}

fn lasm_internal_error_kind_for_code(code: &str, status: u16) -> &'static str {
    match code {
        "HTTP.NOT_FOUND" => "not_found",
        "HTTP.GATEWAY_TIMEOUT" => "timeout",
        "HTTP.SERVICE_UNAVAILABLE" => "resource_limit",
        "HTTP.METHOD_NOT_ALLOWED" => "validation",
        _ if status >= 500 => "internal",
        _ => "validation",
    }
}

fn contains_lasm_request_placeholder_tokens(value: &str) -> bool {
    value.contains("{{req.pathParam:")
        || value.contains("{{req.header:")
        || value.contains("{{req.query:")
        || value.contains("{{req.cookie:")
        || value.contains("{{req.method}}")
        || value.contains("{{req.path}}")
        || value.contains("{{req.httpVersion}}")
        || value.contains("{{req.body}}")
        || value.contains("{{sanitizeHtml:req.pathParam:")
        || value.contains("{{sanitizeHtml:req.header:")
        || value.contains("{{sanitizeHtml:req.query:")
        || value.contains("{{sanitizeHtml:req.cookie:")
        || value.contains("{{sanitizeHtml:req.method}}")
        || value.contains("{{sanitizeHtml:req.path}}")
        || value.contains("{{sanitizeHtml:req.httpVersion}}")
        || value.contains("{{sanitizeHtml:req.body}}")
}

fn escape_lasm_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn materialize_lasm_request_placeholders(
    value: &str,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
) -> String {
    let with_method = value.replace("{{req.method}}", request.method.as_str());
    let with_path = with_method.replace("{{req.path}}", request.path.as_str());
    let with_http_version = with_path.replace("{{req.httpVersion}}", request.http_version.as_str());
    let request_body = String::from_utf8_lossy(&request.body);
    let with_body = with_http_version.replace("{{req.body}}", request_body.as_ref());
    let with_path_params =
        replace_lasm_response_placeholder_tokens(&with_body, "{{req.pathParam:", |key| {
            path_params.get(key.trim()).cloned()
        });
    let with_headers =
        replace_lasm_response_placeholder_tokens(&with_path_params, "{{req.header:", |key| {
            find_lasm_header_value(&request.headers, key.trim()).map(ToOwned::to_owned)
        });
    let with_cookies =
        replace_lasm_response_placeholder_tokens(&with_headers, "{{req.cookie:", |key| {
            find_lasm_cookie_value(&request.headers, key.trim())
        });
    let with_queries =
        replace_lasm_response_placeholder_tokens(&with_cookies, "{{req.query:", |key| {
            request.query_params.get(key.trim()).cloned()
        });

    let escaped_method = escape_lasm_html(request.method.as_str());
    let escaped_path = escape_lasm_html(request.path.as_str());
    let escaped_http_version = escape_lasm_html(request.http_version.as_str());
    let escaped_body = escape_lasm_html(request_body.as_ref());

    let with_sanitized_method =
        with_queries.replace("{{sanitizeHtml:req.method}}", escaped_method.as_str());
    let with_sanitized_path =
        with_sanitized_method.replace("{{sanitizeHtml:req.path}}", escaped_path.as_str());
    let with_sanitized_http_version = with_sanitized_path.replace(
        "{{sanitizeHtml:req.httpVersion}}",
        escaped_http_version.as_str(),
    );
    let with_sanitized_body =
        with_sanitized_http_version.replace("{{sanitizeHtml:req.body}}", escaped_body.as_str());
    let with_sanitized_path_params = replace_lasm_response_placeholder_tokens(
        &with_sanitized_body,
        "{{sanitizeHtml:req.pathParam:",
        |key| {
            path_params
                .get(key.trim())
                .map(|value| escape_lasm_html(value.as_str()))
        },
    );
    let with_sanitized_headers = replace_lasm_response_placeholder_tokens(
        &with_sanitized_path_params,
        "{{sanitizeHtml:req.header:",
        |key| find_lasm_header_value(&request.headers, key.trim()).map(escape_lasm_html),
    );
    let with_sanitized_cookies = replace_lasm_response_placeholder_tokens(
        &with_sanitized_headers,
        "{{sanitizeHtml:req.cookie:",
        |key| {
            find_lasm_cookie_value(&request.headers, key.trim())
                .map(|value| escape_lasm_html(value.as_str()))
        },
    );
    replace_lasm_response_placeholder_tokens(
        &with_sanitized_cookies,
        "{{sanitizeHtml:req.query:",
        |key| {
            request
                .query_params
                .get(key.trim())
                .map(|value| escape_lasm_html(value.as_str()))
        },
    )
}

fn replace_lasm_response_placeholder_tokens(
    body: &str,
    prefix: &str,
    resolve: impl Fn(&str) -> Option<String>,
) -> String {
    let mut output = String::with_capacity(body.len());
    let mut rest = body;
    loop {
        let Some(start) = rest.find(prefix) else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..start]);
        let value_start = start + prefix.len();
        let after_value_start = &rest[value_start..];
        let Some(value_end) = after_value_start.find("}}") else {
            output.push_str(&rest[start..]);
            break;
        };
        let token_value = &after_value_start[..value_end];
        if let Some(value) = resolve(token_value) {
            output.push_str(value.as_str());
        }
        rest = &after_value_start[value_end + 2..];
    }
    output
}

fn lasm_request_expects_json(request: &LasmRunRequest) -> bool {
    find_lasm_header_value(&request.headers, "Content-Type")
        .map(|value| value.to_ascii_lowercase().contains("application/json"))
        .unwrap_or(false)
}

fn extract_lasm_response_schema_hint(response: &sec4_core::HttpResponse) -> Option<String> {
    let parsed = parse_lasm_json_payload(&response.body)?;
    parsed
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
}

fn parse_lasm_json_payload(bytes: &[u8]) -> Option<serde_json::Value> {
    if bytes.is_empty() {
        return None;
    }
    serde_json::from_slice(bytes).ok()
}

fn extract_lasm_payload_id(payload: &serde_json::Value) -> Option<String> {
    payload
        .get("id")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
}

fn validate_lasm_benchmark_user_payload(
    payload: &serde_json::Value,
) -> Option<(&'static str, &'static str)> {
    let Some(id) = payload.get("id").and_then(serde_json::Value::as_str) else {
        return Some(("VALIDATION.UUID_INVALID", "id must be UUID v4"));
    };
    if !is_lasm_uuid_v4(id) {
        return Some(("VALIDATION.UUID_INVALID", "id must be UUID v4"));
    }

    let Some(email) = payload.get("email").and_then(serde_json::Value::as_str) else {
        return Some(("VALIDATION.INVALID", "email must be a valid email string"));
    };
    if !is_lasm_email(email) {
        return Some(("VALIDATION.INVALID", "email must be a valid email string"));
    }

    let Some(age) = payload.get("age").and_then(serde_json::Value::as_i64) else {
        return Some((
            "VALIDATION.INVALID",
            "age must be an integer between 0 and 150",
        ));
    };
    if !(0..=150).contains(&age) {
        return Some((
            "VALIDATION.INVALID",
            "age must be an integer between 0 and 150",
        ));
    }

    let Some(tags) = payload.get("tags").and_then(serde_json::Value::as_array) else {
        return Some((
            "VALIDATION.INVALID",
            "tags must be an array of length <= 16",
        ));
    };
    if tags.len() > 16 {
        return Some((
            "VALIDATION.INVALID",
            "tags must be an array of length <= 16",
        ));
    }
    if tags.iter().any(|tag| {
        let Some(value) = tag.as_str() else {
            return true;
        };
        value.is_empty() || value.len() > 32
    }) {
        return Some((
            "VALIDATION.INVALID",
            "tags must contain strings of length 1..32",
        ));
    }

    let Some(zip) = payload
        .get("address")
        .and_then(|address| address.get("zip"))
        .and_then(serde_json::Value::as_str)
    else {
        return Some((
            "VALIDATION.INVALID",
            "address.zip must be a digit string of length 4..10",
        ));
    };
    if !is_lasm_zip(zip) {
        return Some((
            "VALIDATION.INVALID",
            "address.zip must be a digit string of length 4..10",
        ));
    }

    let Some(flags) = payload
        .get("meta")
        .and_then(|meta| meta.get("flags"))
        .and_then(serde_json::Value::as_object)
    else {
        return Some(("VALIDATION.INVALID", "meta.flags must be an object"));
    };
    for key in ["a", "b", "c"] {
        if !flags.get(key).is_some_and(serde_json::Value::is_boolean) {
            return Some(("VALIDATION.INVALID", "meta.flags must be an object"));
        }
    }

    None
}

fn is_lasm_email(value: &str) -> bool {
    let mut segments = value.split('@');
    let local = segments.next().unwrap_or_default();
    let domain = segments.next().unwrap_or_default();
    segments.next().is_none() && !local.is_empty() && domain.contains('.')
}

fn is_lasm_zip(value: &str) -> bool {
    (4..=10).contains(&value.len()) && value.chars().all(|ch| ch.is_ascii_digit())
}

fn resolve_lasm_user_lookup_id(
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
) -> Option<String> {
    if let Some(id) = path_params.get("id") {
        return Some(id.clone());
    }
    request
        .path
        .strip_prefix("/users/")
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn resolve_lasm_user_lookup_email(request: &LasmRunRequest) -> Option<String> {
    request
        .query_params
        .get("email")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn apply_lasm_internal_db_operation_materialization(
    response: &mut sec4_core::HttpResponse,
    request: &LasmRunRequest,
    path_params: &BTreeMap<String, String>,
    dynamic_state: &Mutex<LasmDynamicResponseState>,
    trace_id: &str,
) -> bool {
    lasm_db_runtime_dispatch::apply_lasm_internal_db_operation_materialization(
        response,
        request,
        path_params,
        dynamic_state,
        trace_id,
    )
}

fn set_lasm_json_response(
    response: &mut sec4_core::HttpResponse,
    status: u16,
    payload: &serde_json::Value,
) {
    response.status = status;
    response.headers.insert(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );
    response.body = serde_json::to_vec(payload).unwrap_or_else(|_| b"{}".to_vec());
}

fn lasm_error_envelope(
    code: &str,
    kind: &str,
    message: &str,
    status: u16,
    trace_id: &str,
) -> serde_json::Value {
    serde_json::json!({
        "error": {
            "code": code,
            "kind": kind,
            "message": message,
            "status": status,
            "traceId": trace_id,
            "timeMs": lasm_now_ms(),
        }
    })
}

fn lasm_request_read_error_code_kind(status: u16) -> (&'static str, &'static str) {
    match status {
        408 => ("HTTP.REQUEST_TIMEOUT", "timeout"),
        413 => ("HTTP.PAYLOAD_TOO_LARGE", "resource_limit"),
        417 => ("HTTP.EXPECTATION_FAILED", "validation"),
        431 => ("HTTP.REQUEST_HEADER_FIELDS_TOO_LARGE", "resource_limit"),
        501 => ("HTTP.NOT_IMPLEMENTED", "internal"),
        505 => ("HTTP.VERSION_NOT_SUPPORTED", "validation"),
        400 => ("HTTP.BAD_REQUEST", "validation"),
        _ if status >= 500 => ("HTTP.INTERNAL", "internal"),
        _ => ("HTTP.BAD_REQUEST", "validation"),
    }
}

fn lasm_overload_head_should_fallback_to_busy(err: &LasmRequestReadError) -> bool {
    err.status == 408
        || err.message == "empty request"
        || err
            .message
            .starts_with("incomplete request while reading request line")
}

fn is_lasm_uuid_v4(value: &str) -> bool {
    if value.len() != 36 {
        return false;
    }
    for (index, byte) in value.as_bytes().iter().enumerate() {
        let is_dash_position = matches!(index, 8 | 13 | 18 | 23);
        if is_dash_position {
            if *byte != b'-' {
                return false;
            }
            continue;
        }
        if !(*byte as char).is_ascii_hexdigit() {
            return false;
        }
    }
    if value.as_bytes()[14] != b'4' {
        return false;
    }
    matches!(
        value.as_bytes()[19].to_ascii_lowercase(),
        b'8' | b'9' | b'a' | b'b'
    )
}

fn evaluate_lasm_cors_preflight_request(
    request: &LasmRunRequest,
    header_defaults: &LasmResponseHeaderDefaults,
) -> LasmCorsPreflightDecision {
    if !header_defaults.cors_enabled || !request.method.eq_ignore_ascii_case("OPTIONS") {
        return LasmCorsPreflightDecision::NotPreflight;
    }
    let requested_method =
        find_lasm_header_value(&request.headers, "Access-Control-Request-Method")
            .map(str::trim)
            .unwrap_or("");
    let requested_headers =
        find_lasm_header_value(&request.headers, "Access-Control-Request-Headers")
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
    if header_defaults.cors_allow_any_method {
        return true;
    }
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
    if header_defaults.cors_allow_any_header {
        return true;
    }
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

fn find_lasm_header_value<'a>(
    headers: &'a BTreeMap<String, String>,
    name: &str,
) -> Option<&'a str> {
    headers
        .iter()
        .find_map(|(key, value)| key.eq_ignore_ascii_case(name).then_some(value.as_str()))
}

fn find_lasm_cookie_value(headers: &BTreeMap<String, String>, cookie_name: &str) -> Option<String> {
    if cookie_name.trim().is_empty() {
        return None;
    }
    headers.iter().find_map(|(header_name, header_value)| {
        if !header_name.eq_ignore_ascii_case("Cookie") {
            return None;
        }
        parse_lasm_cookie_header_value(header_value.as_str(), cookie_name).map(ToOwned::to_owned)
    })
}

fn parse_lasm_cookie_header_value<'a>(
    cookie_header: &'a str,
    cookie_name: &str,
) -> Option<&'a str> {
    let cookie_name = cookie_name.trim();
    if cookie_name.is_empty() {
        return None;
    }
    for segment in cookie_header.split(';') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let (name, value) = segment.split_once('=')?;
        if !name.trim().eq_ignore_ascii_case(cookie_name) {
            continue;
        }
        let value = value.trim();
        if value.is_empty() {
            return None;
        }
        return Some(value);
    }
    None
}

fn lasm_header_has_token(value: &str, token: &str) -> bool {
    value
        .split(',')
        .any(|part| part.trim().eq_ignore_ascii_case(token))
}

fn lasm_should_close_connection(request: &LasmRunRequest) -> bool {
    if let Some(connection) = find_lasm_header_value(&request.headers, "Connection") {
        if lasm_header_has_token(connection, "close") {
            return true;
        }
        if lasm_header_has_token(connection, "keep-alive") {
            return false;
        }
    }
    request.http_version.eq_ignore_ascii_case("HTTP/1.0")
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
) -> Result<LasmRequestHead, LasmRequestReadError> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|err| LasmRequestReadError {
        status: 400,
        message: format!("could not clone stream while reading request head: {err}"),
    })?);
    let parsed = read_lasm_http_request_head(&mut reader, max_header_bytes)?;
    Ok(LasmRequestHead {
        method: parsed.method,
        headers: parsed.headers,
    })
}

#[derive(Debug, Clone)]
struct LasmRunRequest {
    method: String,
    http_version: String,
    path: String,
    query_params: BTreeMap<String, String>,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

#[derive(Debug, Clone)]
struct LasmParsedRequestHead {
    method: String,
    http_version: String,
    path: String,
    query_params: BTreeMap<String, String>,
    headers: BTreeMap<String, String>,
    content_length: usize,
    transfer_encoding_chunked: bool,
}

#[derive(Debug, Clone)]
struct LasmRequestHead {
    method: String,
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
    cors_allow_any_method: bool,
    cors_allowed_methods: Vec<String>,
    cors_allow_any_header: bool,
    cors_allowed_headers: Vec<String>,
    cors_allow_private_network: bool,
    cors_default_origin: Option<String>,
    auth_mode: String,
    auth_cookie_name: String,
    csrf_enabled: bool,
    csrf_cookie_name: String,
    csrf_header_name: String,
    csrf_protected_methods: Vec<String>,
    headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LasmCorsPreflightDecision {
    NotPreflight,
    Accept,
    Reject { status: u16, message: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LasmAuthority {
    host: String,
    port: Option<u16>,
}

fn read_lasm_http_request(
    reader: &mut BufReader<TcpStream>,
    max_header_bytes: usize,
    max_body_bytes: usize,
) -> Result<LasmRunRequest, LasmRequestReadError> {
    let mut request_head = read_lasm_http_request_head(reader, max_header_bytes)?;
    if !request_head.transfer_encoding_chunked && request_head.content_length > max_body_bytes {
        return Err(LasmRequestReadError {
            status: 413,
            message: format!("request body exceeds configured limit ({max_body_bytes} bytes)"),
        });
    }

    let map_read_error = |stage: &str, err: std::io::Error| match err.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => LasmRequestReadError {
            status: 408,
            message: format!("request read timeout while {stage}"),
        },
        std::io::ErrorKind::InvalidData => LasmRequestReadError {
            status: 400,
            message: format!("invalid request encoding while {stage}"),
        },
        std::io::ErrorKind::UnexpectedEof => LasmRequestReadError {
            status: 400,
            message: format!("incomplete request while {stage}"),
        },
        _ => LasmRequestReadError {
            status: 400,
            message: format!("could not {stage}"),
        },
    };

    let body = if request_head.transfer_encoding_chunked {
        read_lasm_http_chunked_body(
            reader,
            &mut request_head.headers,
            max_body_bytes,
            max_header_bytes,
            &map_read_error,
        )?
    } else {
        let mut body = vec![0u8; request_head.content_length];
        if request_head.content_length > 0 {
            reader
                .read_exact(&mut body)
                .map_err(|err| map_read_error("reading request body", err))?;
        }
        body
    };

    Ok(LasmRunRequest {
        method: request_head.method,
        http_version: request_head.http_version,
        path: request_head.path,
        query_params: request_head.query_params,
        headers: request_head.headers,
        body,
    })
}

fn read_lasm_http_chunked_body(
    reader: &mut BufReader<TcpStream>,
    headers: &mut BTreeMap<String, String>,
    max_body_bytes: usize,
    max_header_bytes: usize,
    map_read_error: &dyn Fn(&str, std::io::Error) -> LasmRequestReadError,
) -> Result<Vec<u8>, LasmRequestReadError> {
    let mut body = Vec::new();
    let mut chunk_size_line = String::new();
    let mut trailer_bytes = 0usize;
    loop {
        chunk_size_line.clear();
        let read = reader
            .read_line(&mut chunk_size_line)
            .map_err(|err| map_read_error("reading chunk size", err))?;
        if read == 0 {
            return Err(LasmRequestReadError {
                status: 400,
                message: "incomplete request while reading chunk size".to_string(),
            });
        }
        if !chunk_size_line.ends_with('\n') {
            return Err(LasmRequestReadError {
                status: 400,
                message: "incomplete request while reading chunk size".to_string(),
            });
        }
        if read > max_header_bytes {
            return Err(LasmRequestReadError {
                status: 431,
                message: format!(
                    "request headers exceed configured limit ({max_header_bytes} bytes)"
                ),
            });
        }
        let raw_size = chunk_size_line.trim_end_matches(['\r', '\n']);
        let mut size_and_extensions = raw_size.split(';');
        let size_token = size_and_extensions.next().unwrap_or("").trim();
        if size_token.is_empty() {
            return Err(LasmRequestReadError {
                status: 400,
                message: "invalid transfer-encoding chunk size".to_string(),
            });
        }
        for extension in size_and_extensions {
            if !is_lasm_valid_chunk_extension(extension) {
                return Err(LasmRequestReadError {
                    status: 400,
                    message: "invalid transfer-encoding chunk extension".to_string(),
                });
            }
        }
        let chunk_size =
            usize::from_str_radix(size_token, 16).map_err(|_| LasmRequestReadError {
                status: 400,
                message: "invalid transfer-encoding chunk size".to_string(),
            })?;
        if chunk_size == 0 {
            let mut trailer_line = String::new();
            loop {
                trailer_line.clear();
                let trailer_read = reader
                    .read_line(&mut trailer_line)
                    .map_err(|err| map_read_error("reading chunk trailer", err))?;
                if trailer_read == 0 {
                    return Err(LasmRequestReadError {
                        status: 400,
                        message: "incomplete request while reading chunk trailer".to_string(),
                    });
                }
                if !trailer_line.ends_with('\n') {
                    return Err(LasmRequestReadError {
                        status: 400,
                        message: "incomplete request while reading chunk trailer".to_string(),
                    });
                }
                if trailer_read > max_header_bytes {
                    return Err(LasmRequestReadError {
                        status: 431,
                        message: format!(
                            "request headers exceed configured limit ({max_header_bytes} bytes)"
                        ),
                    });
                }
                trailer_bytes = trailer_bytes.saturating_add(trailer_read);
                if trailer_bytes > max_header_bytes {
                    return Err(LasmRequestReadError {
                        status: 431,
                        message: format!(
                            "request headers exceed configured limit ({max_header_bytes} bytes)"
                        ),
                    });
                }
                if trailer_line == "\r\n" || trailer_line == "\n" {
                    break;
                }
                let trailer = trailer_line.trim_end_matches(['\r', '\n']);
                let Some((name_raw, value_raw)) = trailer.split_once(':') else {
                    return Err(LasmRequestReadError {
                        status: 400,
                        message: "invalid chunk trailer: missing ':' separator".to_string(),
                    });
                };
                if name_raw != name_raw.trim() {
                    return Err(LasmRequestReadError {
                        status: 400,
                        message: "invalid chunk trailer: whitespace around header name".to_string(),
                    });
                }
                if !is_lasm_http_token(name_raw) {
                    return Err(LasmRequestReadError {
                        status: 400,
                        message: "invalid chunk trailer: invalid header name token".to_string(),
                    });
                }
                if is_lasm_forbidden_chunk_trailer(name_raw) {
                    return Err(LasmRequestReadError {
                        status: 400,
                        message: "invalid chunk trailer: forbidden trailer header".to_string(),
                    });
                }
                if !is_lasm_http_header_value(value_raw.trim()) {
                    return Err(LasmRequestReadError {
                        status: 400,
                        message: "invalid chunk trailer: invalid header value character"
                            .to_string(),
                    });
                }
                insert_lasm_request_header_case_insensitive(headers, name_raw, value_raw.trim());
            }
            return Ok(body);
        }
        if body.len().saturating_add(chunk_size) > max_body_bytes {
            return Err(LasmRequestReadError {
                status: 413,
                message: format!("request body exceeds configured limit ({max_body_bytes} bytes)"),
            });
        }
        let start = body.len();
        body.resize(start + chunk_size, 0);
        reader
            .read_exact(&mut body[start..])
            .map_err(|err| map_read_error("reading chunk body", err))?;
        let mut chunk_crlf = [0u8; 2];
        reader
            .read_exact(&mut chunk_crlf)
            .map_err(|err| map_read_error("reading chunk terminator", err))?;
        if chunk_crlf != [b'\r', b'\n'] {
            return Err(LasmRequestReadError {
                status: 400,
                message: "invalid transfer-encoding chunk framing".to_string(),
            });
        }
    }
}

fn is_lasm_valid_chunk_extension(extension: &str) -> bool {
    let trimmed = extension.trim();
    if trimmed.is_empty() {
        return false;
    }
    let (name_raw, value_raw) = match trimmed.split_once('=') {
        Some((name, value)) => (name.trim(), Some(value.trim())),
        None => (trimmed, None),
    };
    if name_raw.is_empty() || !is_lasm_http_token(name_raw) {
        return false;
    }
    let Some(value) = value_raw else {
        return true;
    };
    if value.is_empty() {
        return false;
    }
    is_lasm_http_token(value) || is_lasm_chunk_extension_quoted_string(value)
}

fn is_lasm_chunk_extension_quoted_string(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 2 || bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return false;
    }
    let mut index = 1usize;
    while index + 1 < bytes.len() {
        let byte = bytes[index];
        if byte == b'\\' {
            index += 1;
            if index + 1 >= bytes.len() {
                return false;
            }
            let escaped = bytes[index];
            if escaped < 0x20 || escaped == 0x7f {
                return false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' || byte < 0x20 || byte == 0x7f {
            return false;
        }
        index += 1;
    }
    true
}

fn is_lasm_forbidden_chunk_trailer(name: &str) -> bool {
    name.eq_ignore_ascii_case("content-length")
        || name.eq_ignore_ascii_case("transfer-encoding")
        || name.eq_ignore_ascii_case("host")
}

fn read_lasm_http_request_head(
    reader: &mut BufReader<TcpStream>,
    max_header_bytes: usize,
) -> Result<LasmParsedRequestHead, LasmRequestReadError> {
    let make_error = |status: u16, message: String| LasmRequestReadError { status, message };
    let map_read_error = |stage: &str, err: std::io::Error| match err.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
            make_error(408, format!("request read timeout while {stage}"))
        }
        std::io::ErrorKind::InvalidData => {
            make_error(400, format!("invalid request encoding while {stage}"))
        }
        std::io::ErrorKind::UnexpectedEof => {
            make_error(400, format!("incomplete request while {stage}"))
        }
        _ => make_error(400, format!("could not {stage}")),
    };
    let mut request_line = String::new();
    let bytes = reader
        .read_line(&mut request_line)
        .map_err(|err| map_read_error("reading request line", err))?;
    if bytes == 0 {
        return Err(make_error(400, "empty request".to_string()));
    }
    if !request_line.ends_with('\n') {
        return Err(make_error(
            400,
            "incomplete request while reading request line".to_string(),
        ));
    }
    let mut consumed = bytes;
    if consumed > max_header_bytes {
        return Err(make_error(
            431,
            format!("request headers exceed configured limit ({max_header_bytes} bytes)"),
        ));
    }
    let request_line = request_line.trim_end_matches(['\r', '\n']);
    if request_line.starts_with(char::is_whitespace) {
        return Err(make_error(
            400,
            "invalid request line: leading whitespace is not allowed".to_string(),
        ));
    }
    if request_line.ends_with(char::is_whitespace) {
        return Err(make_error(
            400,
            "invalid request line: trailing whitespace is not allowed".to_string(),
        ));
    }
    if request_line.contains('\t') {
        return Err(make_error(
            400,
            "invalid request line: tab separators are not allowed".to_string(),
        ));
    }
    let mut parts = request_line.split(' ');
    let method = parts
        .next()
        .ok_or_else(|| make_error(400, "invalid request line: missing method".to_string()))?;
    let request_target = parts
        .next()
        .ok_or_else(|| make_error(400, "invalid request line: missing path".to_string()))?;
    let http_version = parts.next().ok_or_else(|| {
        make_error(
            400,
            "invalid request line: missing http version".to_string(),
        )
    })?;
    if method.is_empty()
        || request_target.is_empty()
        || http_version.is_empty()
        || parts.next().is_some()
    {
        return Err(make_error(
            400,
            "invalid request line: expected single-space separators".to_string(),
        ));
    }
    if !is_lasm_http_token(method) {
        return Err(make_error(
            400,
            "invalid request line: invalid method token".to_string(),
        ));
    }
    if http_version != "HTTP/1.1" && http_version != "HTTP/1.0" {
        return Err(make_error(
            505,
            format!("unsupported http version: {http_version}"),
        ));
    }
    if request_target != "*" && request_target.contains('#') {
        return Err(make_error(
            400,
            "invalid request target: fragment is not allowed".to_string(),
        ));
    }

    let mut absolute_authority: Option<(&str, LasmAuthority)> = None;
    let normalized_target = if request_target == "*" {
        std::borrow::Cow::Borrowed(request_target)
    } else if request_target.starts_with('/') {
        std::borrow::Cow::Borrowed(request_target)
    } else if request_target.contains("://") {
        let (scheme_raw, authority_and_path) =
            request_target.split_once("://").ok_or_else(|| {
                make_error(
                    400,
                    "invalid request target: malformed absolute-form".to_string(),
                )
            })?;
        let scheme = if scheme_raw.eq_ignore_ascii_case("http") {
            "http"
        } else if scheme_raw.eq_ignore_ascii_case("https") {
            "https"
        } else {
            return Err(make_error(400, "invalid request target".to_string()));
        };
        let authority_end = authority_and_path
            .find(['/', '?'])
            .unwrap_or(authority_and_path.len());
        let authority = &authority_and_path[..authority_end];
        if authority.is_empty() {
            return Err(make_error(
                400,
                "invalid request target: missing authority".to_string(),
            ));
        }
        let parsed_authority = parse_lasm_authority(authority).ok_or_else(|| {
            make_error(
                400,
                "invalid request target: malformed absolute-form".to_string(),
            )
        })?;
        absolute_authority = Some((scheme, parsed_authority));
        if authority_end < authority_and_path.len() {
            let separator = authority_and_path.as_bytes()[authority_end];
            if separator == b'/' {
                std::borrow::Cow::Borrowed(&authority_and_path[authority_end..])
            } else if separator == b'?' {
                std::borrow::Cow::Owned(format!("/{}", &authority_and_path[authority_end..]))
            } else {
                std::borrow::Cow::Borrowed("/")
            }
        } else {
            std::borrow::Cow::Borrowed("/")
        }
    } else {
        return Err(make_error(400, "invalid request target".to_string()));
    };

    let mut headers = BTreeMap::new();
    let mut content_length = 0usize;
    let mut parsed_content_length: Option<usize> = None;
    let mut transfer_encoding_chunked = false;
    let mut parsed_host_header: Option<LasmAuthority> = None;

    let mut header_line = String::new();
    loop {
        header_line.clear();
        let read = reader
            .read_line(&mut header_line)
            .map_err(|err| map_read_error("reading header line", err))?;
        if read == 0 {
            return Err(make_error(
                400,
                "incomplete request while reading header line".to_string(),
            ));
        }
        if !header_line.ends_with('\n') {
            return Err(make_error(
                400,
                "incomplete request while reading header line".to_string(),
            ));
        }
        consumed = consumed.saturating_add(read);
        if consumed > max_header_bytes {
            return Err(make_error(
                431,
                format!("request headers exceed configured limit ({max_header_bytes} bytes)"),
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
        if name_raw != name_raw.trim() {
            return Err(make_error(
                400,
                "invalid header line: whitespace around header name".to_string(),
            ));
        }
        let name = name_raw;
        let value = value_raw.trim();
        if name.is_empty() {
            return Err(make_error(
                400,
                "invalid header line: empty header name".to_string(),
            ));
        }
        if !is_lasm_http_token(name) {
            return Err(make_error(
                400,
                "invalid header line: invalid header name token".to_string(),
            ));
        }
        if !is_lasm_http_header_value(value) {
            return Err(make_error(
                400,
                "invalid header line: invalid header value character".to_string(),
            ));
        }
        if name.eq_ignore_ascii_case("transfer-encoding") && !value.is_empty() {
            let mut encodings = value
                .split(',')
                .map(str::trim)
                .filter(|encoding| !encoding.is_empty());
            if !encodings.any(|encoding| encoding.eq_ignore_ascii_case("chunked"))
                || value
                    .split(',')
                    .map(str::trim)
                    .filter(|encoding| !encoding.is_empty())
                    .count()
                    != 1
            {
                return Err(make_error(
                    501,
                    "transfer-encoding is not supported".to_string(),
                ));
            }
            transfer_encoding_chunked = true;
        }
        if name.eq_ignore_ascii_case("expect") && !value.is_empty() {
            return Err(make_error(
                417,
                "expect header is not supported".to_string(),
            ));
        }
        if name.eq_ignore_ascii_case("content-length") {
            let parsed = value.parse::<usize>().map_err(|_| {
                make_error(
                    400,
                    "invalid content-length header: expected usize".to_string(),
                )
            })?;
            if let Some(existing) = parsed_content_length {
                if existing != parsed {
                    return Err(make_error(
                        400,
                        "conflicting content-length headers".to_string(),
                    ));
                }
            }
            parsed_content_length = Some(parsed);
            content_length = parsed;
        }
        if name.eq_ignore_ascii_case("host") {
            let parsed_host = parse_lasm_authority(value)
                .ok_or_else(|| make_error(400, "invalid host header".to_string()))?;
            if let Some(existing) = parsed_host_header.as_ref() {
                if existing != &parsed_host {
                    return Err(make_error(400, "conflicting host headers".to_string()));
                }
            } else {
                parsed_host_header = Some(parsed_host);
            }
        }
        insert_lasm_request_header_case_insensitive(&mut headers, name, value);
    }

    if transfer_encoding_chunked && parsed_content_length.is_some() {
        return Err(make_error(
            400,
            "conflicting content-length and transfer-encoding headers".to_string(),
        ));
    }

    if http_version.eq_ignore_ascii_case("HTTP/1.1") && parsed_host_header.is_none() {
        return Err(make_error(400, "missing host header".to_string()));
    }
    if let (Some((scheme, authority)), Some(host)) =
        (absolute_authority.as_ref(), parsed_host_header.as_ref())
    {
        if !lasm_authority_matches_absolute_form(host, authority, scheme) {
            return Err(make_error(
                400,
                "host header does not match request target authority".to_string(),
            ));
        }
    }

    let (path, query_params) = split_lasm_path_and_query(normalized_target.as_ref());

    Ok(LasmParsedRequestHead {
        method: method.to_ascii_uppercase(),
        http_version: http_version.to_string(),
        path,
        query_params,
        headers,
        content_length,
        transfer_encoding_chunked,
    })
}

fn parse_lasm_authority(value: &str) -> Option<LasmAuthority> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.contains(',')
        || trimmed.contains('@')
        || trimmed.chars().any(char::is_whitespace)
    {
        return None;
    }

    let (host, port) = if let Some(rest) = trimmed.strip_prefix('[') {
        let end = rest.find(']')?;
        let literal = &rest[..end];
        if literal.is_empty() {
            return None;
        }
        let remainder = &rest[end + 1..];
        let port = if remainder.is_empty() {
            None
        } else {
            let value = remainder.strip_prefix(':')?;
            Some(parse_lasm_authority_port(value)?)
        };
        (format!("[{}]", literal.to_ascii_lowercase()), port)
    } else {
        if trimmed.contains('/') || trimmed.contains('?') || trimmed.contains('#') {
            return None;
        }
        let colon_count = trimmed.as_bytes().iter().filter(|&&ch| ch == b':').count();
        let (host, port) = match colon_count {
            0 => (trimmed.to_ascii_lowercase(), None),
            1 => {
                let (host, port) = trimmed.rsplit_once(':')?;
                if host.is_empty() {
                    return None;
                }
                (
                    host.to_ascii_lowercase(),
                    Some(parse_lasm_authority_port(port)?),
                )
            }
            _ => return None,
        };
        (host, port)
    };

    if host.is_empty() {
        return None;
    }

    Some(LasmAuthority { host, port })
}

fn insert_lasm_request_header_case_insensitive(
    headers: &mut BTreeMap<String, String>,
    name: &str,
    value: &str,
) {
    if let Some(existing_key) = find_lasm_header_key_case_insensitive(headers, name) {
        let existing_key = existing_key.to_string();
        let existing_value = headers
            .remove(existing_key.as_str())
            .unwrap_or_else(|| "".to_string());
        headers.insert(
            existing_key.clone(),
            merge_lasm_request_header_values(existing_key.as_str(), existing_value.as_str(), value),
        );
        return;
    }
    headers.insert(name.to_string(), value.to_string());
}

fn merge_lasm_request_header_values(header_name: &str, existing: &str, next: &str) -> String {
    if header_name.eq_ignore_ascii_case("host")
        || header_name.eq_ignore_ascii_case("content-length")
    {
        return existing.to_string();
    }
    if existing.is_empty() {
        return next.to_string();
    }
    if next.is_empty() {
        return existing.to_string();
    }
    if header_name.eq_ignore_ascii_case("cookie") {
        return format!("{existing}; {next}");
    }
    format!("{existing}, {next}")
}

fn is_lasm_response_header_name_valid(value: &str) -> bool {
    !value.is_empty()
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
}

fn is_lasm_http_token(value: &str) -> bool {
    !value.is_empty()
        && value.as_bytes().iter().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    *byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

fn is_lasm_http_header_value(value: &str) -> bool {
    value.chars().all(|ch| ch == '\t' || !ch.is_control())
}

fn is_lasm_response_header_value_valid(value: &str) -> bool {
    !value.is_empty() && is_lasm_http_header_value(value)
}

fn parse_lasm_authority_port(value: &str) -> Option<u16> {
    if value.is_empty() || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let parsed = value.parse::<u16>().ok()?;
    if parsed == 0 {
        return None;
    }
    Some(parsed)
}

fn lasm_authority_matches_absolute_form(
    host: &LasmAuthority,
    request_target: &LasmAuthority,
    scheme: &str,
) -> bool {
    if host.host != request_target.host {
        return false;
    }
    let default_port = match scheme {
        "http" => Some(80),
        "https" => Some(443),
        _ => None,
    };
    host.port.or(default_port) == request_target.port.or(default_port)
}

fn split_lasm_path_and_query(target: &str) -> (String, BTreeMap<String, String>) {
    let mut query_params = BTreeMap::new();
    let Some((path, query)) = target.split_once('?') else {
        return (target.to_string(), query_params);
    };

    for segment in query.split('&') {
        if segment.is_empty() {
            continue;
        }
        let (key_raw, value_raw) = segment.split_once('=').unwrap_or((segment, ""));
        let decoded = decode_lasm_query_component(key_raw)
            .zip(decode_lasm_query_component(value_raw))
            .unwrap_or_else(|| (key_raw.to_string(), value_raw.to_string()));
        let (key, value) = decoded;
        if key.trim().is_empty() {
            continue;
        }
        if !query_params.contains_key(key.as_str()) {
            query_params.insert(key, value);
        }
    }

    (path.to_string(), query_params)
}

fn decode_lasm_query_component(component: &str) -> Option<String> {
    let mut decoded = Vec::with_capacity(component.len());
    let bytes = component.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' => {
                if index + 2 >= bytes.len() {
                    return None;
                }
                let hi = (bytes[index + 1] as char).to_digit(16)?;
                let lo = (bytes[index + 2] as char).to_digit(16)?;
                decoded.push(((hi << 4) | lo) as u8);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).ok()
}

fn write_lasm_http_response(
    stream: &mut TcpStream,
    response: &sec4_core::HttpResponse,
    header_defaults: &LasmResponseHeaderDefaults,
    include_cors_defaults: bool,
    omit_body: bool,
    close_connection: bool,
) -> Result<(), String> {
    let mut headers = normalize_lasm_response_headers_case_insensitive(response.headers.clone());
    if !include_cors_defaults {
        headers.retain(|name, _| !is_lasm_cors_default_header_name(name.as_str()));
    }
    for (name, value) in &header_defaults.headers {
        if !include_cors_defaults && is_lasm_cors_default_header_name(name.as_str()) {
            continue;
        }
        insert_lasm_header_if_missing_case_insensitive(&mut headers, name, value.clone());
    }
    upsert_lasm_header_case_insensitive(
        &mut headers,
        "Content-Length".to_string(),
        response.body.len().to_string(),
    );
    upsert_lasm_header_case_insensitive(
        &mut headers,
        "Connection".to_string(),
        if close_connection {
            "close".to_string()
        } else {
            "keep-alive".to_string()
        },
    );
    insert_lasm_header_if_missing_case_insensitive(
        &mut headers,
        "Content-Type",
        "text/plain; charset=utf-8".to_string(),
    );

    let status_text = http_status_text(response.status);
    let mut response_head = format!("HTTP/1.1 {} {}\r\n", response.status, status_text);
    for (name, value) in headers {
        if name.eq_ignore_ascii_case("Set-Cookie") {
            for cookie_value in value.split('\n') {
                if cookie_value.is_empty() {
                    continue;
                }
                response_head.push_str(name.as_str());
                response_head.push_str(": ");
                response_head.push_str(cookie_value);
                response_head.push_str("\r\n");
            }
            continue;
        }
        response_head.push_str(name.as_str());
        response_head.push_str(": ");
        response_head.push_str(value.as_str());
        response_head.push_str("\r\n");
    }
    response_head.push_str("\r\n");
    stream
        .write_all(response_head.as_bytes())
        .map_err(|err| format!("could not write response headers: {err}"))?;
    if !omit_body {
        stream
            .write_all(&response.body)
            .map_err(|err| format!("could not write response body: {err}"))?;
    }
    stream
        .flush()
        .map_err(|err| format!("could not flush response stream: {err}"))?;
    Ok(())
}

fn insert_lasm_header_if_missing_case_insensitive(
    headers: &mut BTreeMap<String, String>,
    canonical_name: &str,
    value: String,
) {
    if find_lasm_header_key_case_insensitive(headers, canonical_name).is_some() {
        return;
    }
    headers.insert(canonical_name.to_string(), value);
}

fn normalize_lasm_response_headers_case_insensitive(
    headers: BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut normalized = BTreeMap::new();
    for (name, value) in headers {
        if let Some(existing_key) =
            find_lasm_header_key_case_insensitive(&normalized, name.as_str())
        {
            normalized.insert(existing_key, value);
        } else {
            normalized.insert(name, value);
        }
    }
    normalized
}

fn upsert_lasm_header_case_insensitive(
    headers: &mut BTreeMap<String, String>,
    canonical_name: String,
    value: String,
) {
    if let Some(existing_key) =
        find_lasm_header_key_case_insensitive(headers, canonical_name.as_str())
    {
        headers.insert(existing_key, value);
        return;
    }
    headers.insert(canonical_name, value);
}

fn find_lasm_header_key_case_insensitive(
    headers: &BTreeMap<String, String>,
    target: &str,
) -> Option<String> {
    headers
        .keys()
        .find(|name| name.eq_ignore_ascii_case(target))
        .cloned()
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

fn set_lasm_trace_id(response: &mut sec4_core::HttpResponse, trace_id: &str) {
    if let Some(existing_key) =
        find_lasm_header_key_case_insensitive(&response.headers, "X-Trace-Id")
    {
        response.headers.insert(existing_key, trace_id.to_string());
        return;
    }
    response
        .headers
        .insert("X-Trace-Id".to_string(), trace_id.to_string());
}

fn lasm_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn http_status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        401 => "Unauthorized",
        403 => "Forbidden",
        400 => "Bad Request",
        417 => "Expectation Failed",
        408 => "Request Timeout",
        405 => "Method Not Allowed",
        409 => "Conflict",
        431 => "Request Header Fields Too Large",
        413 => "Payload Too Large",
        404 => "Not Found",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
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
