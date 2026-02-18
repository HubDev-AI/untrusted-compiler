use crate::mir::{MirFunction, MirInstructionKind, MirProgram, MirTerminator};
use std::collections::BTreeSet;
use std::fmt::Write;

pub fn emit_c_program(program: &MirProgram) -> String {
    let mut out = String::new();
    out.push_str("#include <stdbool.h>\n#include <stdint.h>\n#include \"sec4_runtime.h\"\n\n");

    for function in &program.functions {
        writeln!(&mut out, "{};", c_function_signature(function))
            .expect("write to string must succeed");
    }

    if !program.functions.is_empty() {
        out.push('\n');
    }

    for (index, function) in program.functions.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        render_function(&mut out, function);
    }

    out
}

pub fn emit_runtime_header() -> &'static str {
    include_str!("../../../runtime/c/sec4_runtime.h")
}

pub fn emit_runtime_source() -> &'static str {
    include_str!("../../../runtime/c/sec4_runtime.c")
}

fn render_function(out: &mut String, function: &MirFunction) {
    writeln!(out, "{} {{", c_function_signature(function)).expect("write to string must succeed");
    let runtime_return_identity = runtime_identity_for_return_type(function.return_type.as_deref());

    let locals = collect_locals(function);
    for local in &locals {
        writeln!(out, "  int64_t {} = 0;", local).expect("write to string must succeed");
    }

    if !function.blocks.is_empty() {
        writeln!(out, "  goto bb0;").expect("write to string must succeed");
    }

    for block in &function.blocks {
        writeln!(out, "bb{}:", block.id).expect("write to string must succeed");

        for instruction in &block.instructions {
            match &instruction.kind {
                MirInstructionKind::Let { name, value } => {
                    writeln!(out, "  {} = {};", name, lower_c_expr(value))
                        .expect("write to string must succeed");
                }
                MirInstructionKind::Eval { value } => {
                    writeln!(out, "  (void)({});", lower_c_expr(value))
                        .expect("write to string must succeed");
                }
            }
        }

        match &block.terminator {
            MirTerminator::Return { value, .. } => {
                if let Some(value) = value {
                    let value = lower_c_expr(value);
                    if let Some(identity_fn) = runtime_return_identity {
                        writeln!(out, "  return {}({});", identity_fn, value)
                            .expect("write to string must succeed");
                    } else {
                        writeln!(out, "  return {};", value).expect("write to string must succeed");
                    }
                } else {
                    writeln!(out, "  return;").expect("write to string must succeed");
                }
            }
            MirTerminator::Goto { target, .. } => {
                writeln!(out, "  goto bb{};", target).expect("write to string must succeed");
            }
            MirTerminator::Branch {
                condition,
                then_target,
                else_target,
                ..
            } => {
                let condition = lower_c_expr(condition);
                writeln!(
                    out,
                    "  if ({}) goto bb{}; else goto bb{};",
                    condition, then_target, else_target
                )
                .expect("write to string must succeed");
            }
            MirTerminator::Switch {
                scrutinee, targets, ..
            } => {
                let scrutinee = lower_c_expr(scrutinee);
                let mut default_target = None;
                for target in targets {
                    if target.pattern == "_" {
                        default_target = Some(target.target);
                    } else {
                        writeln!(
                            out,
                            "  if ({} == {}) goto bb{};",
                            scrutinee, target.pattern, target.target
                        )
                        .expect("write to string must succeed");
                    }
                }

                let fallback_target =
                    default_target.or_else(|| targets.first().map(|target| target.target));
                if let Some(target) = fallback_target {
                    writeln!(out, "  goto bb{};", target).expect("write to string must succeed");
                } else {
                    writeln!(out, "  return;").expect("write to string must succeed");
                }
            }
        }
    }

    writeln!(out, "}}").expect("write to string must succeed");
}

fn collect_locals(function: &MirFunction) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for block in &function.blocks {
        for instruction in &block.instructions {
            if let MirInstructionKind::Let { name, .. } = &instruction.kind {
                names.insert(name.clone());
            }
        }
    }
    names
}

fn c_function_signature(function: &MirFunction) -> String {
    let return_type = if function.name == "main" {
        "int"
    } else {
        c_type(function.return_type.as_deref())
    };
    let params = function
        .params
        .iter()
        .map(|param| format!("{} {}", c_type(Some(param.ty.as_str())), param.name))
        .collect::<Vec<_>>()
        .join(", ");

    if params.is_empty() {
        format!("{} {}(void)", return_type, function.name)
    } else {
        format!("{} {}({})", return_type, function.name, params)
    }
}

fn c_type(type_name: Option<&str>) -> &'static str {
    match type_name {
        Some("Bool") => "bool",
        Some("Unit") => "void",
        Some("Int") | Some("Int64") => "int64_t",
        None => "void",
        Some(_) => "int64_t",
    }
}

fn runtime_identity_for_return_type(type_name: Option<&str>) -> Option<&'static str> {
    match type_name {
        Some("Bool") => Some("sec4_rt_identity_bool"),
        Some("Int") | Some("Int64") => Some("sec4_rt_identity_i64"),
        _ => None,
    }
}

fn lower_c_expr(expr: &str) -> String {
    let mut lowered = expr.to_string();
    lowered = lowered.replace("InternalNetCap(", "__SEC4_CONSTRUCTOR_INTERNAL_NET_CAP__(");
    lowered = lowered.replace("SecretsCap(", "__SEC4_CONSTRUCTOR_SECRETS_CAP__(");
    lowered = lowered.replace("NetCap(", "__SEC4_CONSTRUCTOR_NET_CAP__(");
    lowered = lowered.replace("DbCap(", "__SEC4_CONSTRUCTOR_DB_CAP__(");
    lowered = lowered.replace("FsCap(", "__SEC4_CONSTRUCTOR_FS_CAP__(");
    lowered = lowered.replace("Ctx(", "__SEC4_CONSTRUCTOR_CTX__(");
    lowered = lowered.replace("time.now(", "__SEC4_INTRINSIC_TIME_NOW__(");
    lowered = lowered.replace("time_now(", "__SEC4_INTRINSIC_TIME_NOW__(");
    lowered = lowered.replace("log.info(", "__SEC4_INTRINSIC_LOG_INFO__(");
    lowered = lowered.replace("log.warn(", "__SEC4_INTRINSIC_LOG_WARN__(");
    lowered = lowered.replace("log.error(", "__SEC4_INTRINSIC_LOG_ERROR__(");
    lowered = lowered.replace("log.emit(", "__SEC4_INTRINSIC_LOG_ANY__(");
    lowered = lowered.replace("log_info(", "__SEC4_INTRINSIC_LOG_INFO__(");
    lowered = lowered.replace("log_warn(", "__SEC4_INTRINSIC_LOG_WARN__(");
    lowered = lowered.replace("log_error(", "__SEC4_INTRINSIC_LOG_ERROR__(");
    lowered = lowered.replace("log_emit(", "__SEC4_INTRINSIC_LOG_ANY__(");
    lowered = lowered.replace("log.event(", "__SEC4_INTRINSIC_LOG_EVENT__(");
    lowered = lowered.replace("log_event(", "__SEC4_INTRINSIC_LOG_EVENT__(");
    lowered = lowered.replace("log.field(", "__SEC4_INTRINSIC_LOG_FIELD__(");
    lowered = lowered.replace("log_field(", "__SEC4_INTRINSIC_LOG_FIELD__(");
    lowered = lowered.replace("log.obj(", "__SEC4_INTRINSIC_LOG_OBJ__(");
    lowered = lowered.replace("log_obj(", "__SEC4_INTRINSIC_LOG_OBJ__(");
    lowered = lowered.replace("log.str(", "__SEC4_INTRINSIC_LOG_STR__(");
    lowered = lowered.replace("log_str(", "__SEC4_INTRINSIC_LOG_STR__(");
    lowered = lowered.replace("log.i64(", "__SEC4_INTRINSIC_LOG_I64__(");
    lowered = lowered.replace("log_i64(", "__SEC4_INTRINSIC_LOG_I64__(");
    lowered = lowered.replace("log.bool(", "__SEC4_INTRINSIC_LOG_BOOL__(");
    lowered = lowered.replace("log_bool(", "__SEC4_INTRINSIC_LOG_BOOL__(");
    lowered = lowered.replace("log.redacted(", "__SEC4_INTRINSIC_LOG_REDACTED__(");
    lowered = lowered.replace("log_redacted(", "__SEC4_INTRINSIC_LOG_REDACTED__(");
    lowered = lowered.replace("log.attrRedacted(", "__SEC4_INTRINSIC_LOG_ATTR_REDACTED__(");
    lowered = lowered.replace(
        "log_attr_redacted(",
        "__SEC4_INTRINSIC_LOG_ATTR_REDACTED__(",
    );
    lowered = lowered.replace("log.withAttr(", "__SEC4_INTRINSIC_LOG_WITH_ATTR__(");
    lowered = lowered.replace("log_with_attr(", "__SEC4_INTRINSIC_LOG_WITH_ATTR__(");
    lowered = lowered.replace("log.withHttp(", "__SEC4_INTRINSIC_LOG_WITH_HTTP__(");
    lowered = lowered.replace("log_with_http(", "__SEC4_INTRINSIC_LOG_WITH_HTTP__(");
    lowered = lowered.replace("log.withError(", "__SEC4_INTRINSIC_LOG_WITH_ERROR__(");
    lowered = lowered.replace("log_with_error(", "__SEC4_INTRINSIC_LOG_WITH_ERROR__(");
    lowered = lowered.replace("req.json(", "__SEC4_INTRINSIC_REQ_JSON__(");
    lowered = lowered.replace("req_json(", "__SEC4_INTRINSIC_REQ_JSON__(");
    lowered = lowered.replace("json.decode(", "__SEC4_INTRINSIC_JSON_DECODE__(");
    lowered = lowered.replace("json_decode(", "__SEC4_INTRINSIC_JSON_DECODE__(");
    lowered = lowered.replace("json.encode(", "__SEC4_INTRINSIC_JSON_ENCODE__(");
    lowered = lowered.replace("json_encode(", "__SEC4_INTRINSIC_JSON_ENCODE__(");
    lowered = lowered.replace("req.body(", "__SEC4_INTRINSIC_REQ_BODY__(");
    lowered = lowered.replace("req_body(", "__SEC4_INTRINSIC_REQ_BODY__(");
    lowered = lowered.replace("req.query(", "__SEC4_INTRINSIC_REQ_QUERY__(");
    lowered = lowered.replace("req_query(", "__SEC4_INTRINSIC_REQ_QUERY__(");
    lowered = lowered.replace("req.pathParam(", "__SEC4_INTRINSIC_REQ_PATH_PARAM__(");
    lowered = lowered.replace("req_path_param(", "__SEC4_INTRINSIC_REQ_PATH_PARAM__(");
    lowered = lowered.replace("req.header(", "__SEC4_INTRINSIC_REQ_HEADER__(");
    lowered = lowered.replace("req_header(", "__SEC4_INTRINSIC_REQ_HEADER__(");
    lowered = lowered.replace("res.json(", "__SEC4_INTRINSIC_RES_JSON__(");
    lowered = lowered.replace("res_json(", "__SEC4_INTRINSIC_RES_JSON__(");
    lowered = lowered.replace("res.okMeta(", "__SEC4_INTRINSIC_RES_OK_META__(");
    lowered = lowered.replace("res_ok_meta(", "__SEC4_INTRINSIC_RES_OK_META__(");
    lowered = lowered.replace("res.ok(", "__SEC4_INTRINSIC_RES_OK__(");
    lowered = lowered.replace("res_ok(", "__SEC4_INTRINSIC_RES_OK__(");
    lowered = lowered.replace("res.html(", "__SEC4_INTRINSIC_RES_HTML__(");
    lowered = lowered.replace("res_html(", "__SEC4_INTRINSIC_RES_HTML__(");
    lowered = lowered.replace("res.text(", "__SEC4_INTRINSIC_RES_TEXT__(");
    lowered = lowered.replace("res_text(", "__SEC4_INTRINSIC_RES_TEXT__(");
    lowered = lowered.replace("res.setHeader(", "__SEC4_INTRINSIC_SET_HEADER__(");
    lowered = lowered.replace("set_header(", "__SEC4_INTRINSIC_SET_HEADER__(");
    lowered = lowered.replace("cookie.build(", "__SEC4_INTRINSIC_COOKIE_BUILD__(");
    lowered = lowered.replace("cookie_build(", "__SEC4_INTRINSIC_COOKIE_BUILD__(");
    lowered = lowered.replace("res.addCookie(", "__SEC4_INTRINSIC_SET_COOKIE__(");
    lowered = lowered.replace("set_cookie(", "__SEC4_INTRINSIC_SET_COOKIE__(");
    lowered = lowered.replace("sql.q(", "__SEC4_INTRINSIC_SQL_Q__(");
    lowered = lowered.replace("sql_q(", "__SEC4_INTRINSIC_SQL_Q__(");
    lowered = lowered.replace("db.execTx(", "__SEC4_INTRINSIC_DB_EXEC_TX__(");
    lowered = lowered.replace("db_exec_tx(", "__SEC4_INTRINSIC_DB_EXEC_TX__(");
    lowered = lowered.replace("db.tx(", "__SEC4_INTRINSIC_DB_TX__(");
    lowered = lowered.replace("db_tx(", "__SEC4_INTRINSIC_DB_TX__(");
    lowered = lowered.replace("db.exec(", "__SEC4_INTRINSIC_DB_EXEC__(");
    lowered = lowered.replace("db_write(", "__SEC4_INTRINSIC_DB_EXEC__(");
    lowered = lowered.replace("db.queryOne(", "__SEC4_INTRINSIC_DB_QUERY_ONE__(");
    lowered = lowered.replace("db_read(", "__SEC4_INTRINSIC_DB_QUERY_ONE__(");
    lowered = lowered.replace("fs.read(", "__SEC4_INTRINSIC_FS_READ__(");
    lowered = lowered.replace("fs_read(", "__SEC4_INTRINSIC_FS_READ__(");
    lowered = lowered.replace("fs.write(", "__SEC4_INTRINSIC_FS_WRITE__(");
    lowered = lowered.replace("fs_write(", "__SEC4_INTRINSIC_FS_WRITE__(");
    lowered = lowered.replace("httpClient.get(", "__SEC4_INTRINSIC_HTTP_GET__(");
    lowered = lowered.replace("net_call(", "__SEC4_INTRINSIC_HTTP_GET__(");
    lowered = lowered.replace(
        "httpClient.getInternal(",
        "__SEC4_INTRINSIC_HTTP_GET_INTERNAL__(",
    );
    lowered = lowered.replace(
        "net_internal_call(",
        "__SEC4_INTRINSIC_HTTP_GET_INTERNAL__(",
    );
    lowered = lowered.replace("secrets.get(", "__SEC4_INTRINSIC_SECRET_GET__(");
    lowered = lowered.replace("secret_read(", "__SEC4_INTRINSIC_SECRET_GET__(");
    lowered = lowered.replace("secrets.redact(", "__SEC4_INTRINSIC_SECRET_REDACT__(");
    lowered = lowered.replace("secret_redact(", "__SEC4_INTRINSIC_SECRET_REDACT__(");
    lowered = lowered.replace("secrets.reveal(", "__SEC4_INTRINSIC_SECRET_REVEAL__(");
    lowered = lowered.replace("secret_reveal(", "__SEC4_INTRINSIC_SECRET_REVEAL__(");
    lowered = lowered.replace("crypto.ctEq(", "__SEC4_INTRINSIC_CRYPTO_CT_EQ__(");
    lowered = lowered.replace("crypto_ct_eq(", "__SEC4_INTRINSIC_CRYPTO_CT_EQ__(");
    lowered = lowered.replace(
        "validate.headerValue(",
        "__SEC4_INTRINSIC_VALIDATE_HEADER_VALUE__(",
    );
    lowered = lowered.replace(
        "validate_header_value(",
        "__SEC4_INTRINSIC_VALIDATE_HEADER_VALUE__(",
    );
    lowered = lowered.replace("validate.email(", "__SEC4_INTRINSIC_VALIDATE_EMAIL__(");
    lowered = lowered.replace("validate_email(", "__SEC4_INTRINSIC_VALIDATE_EMAIL__(");
    lowered = lowered.replace("validate.uuid(", "__SEC4_INTRINSIC_VALIDATE_UUID__(");
    lowered = lowered.replace("validate_uuid(", "__SEC4_INTRINSIC_VALIDATE_UUID__(");
    lowered = lowered.replace("validate.int64(", "__SEC4_INTRINSIC_VALIDATE_INT64__(");
    lowered = lowered.replace("validate_int64(", "__SEC4_INTRINSIC_VALIDATE_INT64__(");
    lowered = lowered.replace(
        "validate.nonEmpty(",
        "__SEC4_INTRINSIC_VALIDATE_NON_EMPTY__(",
    );
    lowered = lowered.replace(
        "validate_non_empty(",
        "__SEC4_INTRINSIC_VALIDATE_NON_EMPTY__(",
    );
    lowered = lowered.replace("sanitize.html(", "__SEC4_INTRINSIC_SANITIZE_HTML__(");
    lowered = lowered.replace("sanitize_html(", "__SEC4_INTRINSIC_SANITIZE_HTML__(");
    lowered = lowered.replace("url.public(", "__SEC4_INTRINSIC_URL_PUBLIC__(");
    lowered = lowered.replace("url_public(", "__SEC4_INTRINSIC_URL_PUBLIC__(");
    lowered = lowered.replace("url.internal(", "__SEC4_INTRINSIC_URL_INTERNAL__(");
    lowered = lowered.replace("url_internal(", "__SEC4_INTRINSIC_URL_INTERNAL__(");
    lowered = lowered.replace("path.under(", "__SEC4_INTRINSIC_PATH_UNDER__(");
    lowered = lowered.replace("path_under(", "__SEC4_INTRINSIC_PATH_UNDER__(");
    lowered = lowered.replace("validate.pathUnder(", "__SEC4_INTRINSIC_PATH_UNDER__(");
    lowered = lowered.replace("validate_path_under(", "__SEC4_INTRINSIC_PATH_UNDER__(");
    lowered = lowered.replace("path.base(", "__SEC4_INTRINSIC_PATH_BASE__(");
    lowered = lowered.replace("path_base(", "__SEC4_INTRINSIC_PATH_BASE__(");
    lowered = lowered.replace("headers.name(", "__SEC4_INTRINSIC_HEADERS_NAME__(");
    lowered = lowered.replace("headers_name(", "__SEC4_INTRINSIC_HEADERS_NAME__(");
    lowered = lowered.replace("headers.value(", "__SEC4_INTRINSIC_HEADERS_VALUE__(");
    lowered = lowered.replace("headers_value(", "__SEC4_INTRINSIC_HEADERS_VALUE__(");
    lowered = lowered.replace("http.router(", "__SEC4_INTRINSIC_HTTP_ROUTER__(");
    lowered = lowered.replace("http_router(", "__SEC4_INTRINSIC_HTTP_ROUTER__(");
    lowered = lowered.replace("http.get(", "__SEC4_INTRINSIC_HTTP_ROUTE_GET__(");
    lowered = lowered.replace("http_get_route(", "__SEC4_INTRINSIC_HTTP_ROUTE_GET__(");
    lowered = lowered.replace("http.post(", "__SEC4_INTRINSIC_HTTP_ROUTE_POST__(");
    lowered = lowered.replace("http_post_route(", "__SEC4_INTRINSIC_HTTP_ROUTE_POST__(");
    lowered = lowered.replace("http.serve(", "__SEC4_INTRINSIC_HTTP_SERVE__(");
    lowered = lowered.replace("http_serve(", "__SEC4_INTRINSIC_HTTP_SERVE__(");
    lowered = lowered.replace("cors.withCors(", "__SEC4_INTRINSIC_WITH_CORS__(");
    lowered = lowered.replace("withCors(", "__SEC4_INTRINSIC_WITH_CORS__(");
    lowered = lowered.replace("cors_with(", "__SEC4_INTRINSIC_WITH_CORS__(");
    lowered = lowered.replace(
        "sec.withSecurityHeaders(",
        "__SEC4_INTRINSIC_WITH_SECURITY_HEADERS__(",
    );
    lowered = lowered.replace(
        "withSecurityHeaders(",
        "__SEC4_INTRINSIC_WITH_SECURITY_HEADERS__(",
    );
    lowered = lowered.replace(
        "sec_with_security_headers(",
        "__SEC4_INTRINSIC_WITH_SECURITY_HEADERS__(",
    );
    lowered = lowered.replace("csrf.withCsrf(", "__SEC4_INTRINSIC_WITH_CSRF__(");
    lowered = lowered.replace("withCsrf(", "__SEC4_INTRINSIC_WITH_CSRF__(");
    lowered = lowered.replace("csrf_with(", "__SEC4_INTRINSIC_WITH_CSRF__(");
    lowered = lowered.replace("auth.withAuth(", "__SEC4_INTRINSIC_WITH_AUTH__(");
    lowered = lowered.replace("withAuth(", "__SEC4_INTRINSIC_WITH_AUTH__(");
    lowered = lowered.replace("auth_with(", "__SEC4_INTRINSIC_WITH_AUTH__(");
    lowered = lowered.replace(
        "sec.defaultHeaders(",
        "__SEC4_INTRINSIC_SEC_DEFAULT_HEADERS__(",
    );
    lowered = lowered.replace(
        "sec_default_headers(",
        "__SEC4_INTRINSIC_SEC_DEFAULT_HEADERS__(",
    );
    lowered = lowered.replace("sec.cspAdd(", "__SEC4_INTRINSIC_SEC_CSP_ADD__(");
    lowered = lowered.replace("sec_csp_add(", "__SEC4_INTRINSIC_SEC_CSP_ADD__(");
    lowered = lowered.replace("sec.csp(", "__SEC4_INTRINSIC_SEC_CSP__(");
    lowered = lowered.replace("sec_csp(", "__SEC4_INTRINSIC_SEC_CSP__(");
    lowered = lowered.replace("cors.fromPolicy(", "__SEC4_INTRINSIC_CORS_FROM_POLICY__(");
    lowered = lowered.replace("cors_from_policy(", "__SEC4_INTRINSIC_CORS_FROM_POLICY__(");
    lowered = lowered.replace("cors.origin(", "__SEC4_INTRINSIC_CORS_ORIGIN__(");
    lowered = lowered.replace("cors_origin(", "__SEC4_INTRINSIC_CORS_ORIGIN__(");
    lowered = lowered.replace("csrf.issueToken(", "__SEC4_INTRINSIC_CSRF_ISSUE_TOKEN__(");
    lowered = lowered.replace("csrf_issue_token(", "__SEC4_INTRINSIC_CSRF_ISSUE_TOKEN__(");
    lowered = lowered.replace("csrf.fromPolicy(", "__SEC4_INTRINSIC_CSRF_FROM_POLICY__(");
    lowered = lowered.replace("csrf_from_policy(", "__SEC4_INTRINSIC_CSRF_FROM_POLICY__(");
    lowered = lowered.replace("auth.fromPolicy(", "__SEC4_INTRINSIC_AUTH_FROM_POLICY__(");
    lowered = lowered.replace("auth_from_policy(", "__SEC4_INTRINSIC_AUTH_FROM_POLICY__(");
    lowered = lowered.replace("auth.requireRole(", "__SEC4_INTRINSIC_AUTH_REQUIRE_ROLE__(");
    lowered = lowered.replace(
        "auth_require_role(",
        "__SEC4_INTRINSIC_AUTH_REQUIRE_ROLE__(",
    );
    lowered = lowered.replace("auth.require(", "__SEC4_INTRINSIC_AUTH_REQUIRE__(");
    lowered = lowered.replace("auth_require(", "__SEC4_INTRINSIC_AUTH_REQUIRE__(");
    lowered = lowered.replace(
        "err.withDependency(",
        "__SEC4_INTRINSIC_ERR_WITH_DEPENDENCY__(",
    );
    lowered = lowered.replace(
        "err_with_dependency(",
        "__SEC4_INTRINSIC_ERR_WITH_DEPENDENCY__(",
    );
    lowered = lowered.replace("err.withDetail(", "__SEC4_INTRINSIC_ERR_WITH_DETAIL__(");
    lowered = lowered.replace("err_with_detail(", "__SEC4_INTRINSIC_ERR_WITH_DETAIL__(");
    lowered = lowered.replace("err.withCause(", "__SEC4_INTRINSIC_ERR_WITH_CAUSE__(");
    lowered = lowered.replace("err_with_cause(", "__SEC4_INTRINSIC_ERR_WITH_CAUSE__(");
    lowered = lowered.replace("err.withLimit(", "__SEC4_INTRINSIC_ERR_WITH_LIMIT__(");
    lowered = lowered.replace("err_with_limit(", "__SEC4_INTRINSIC_ERR_WITH_LIMIT__(");
    lowered = lowered.replace("err.withPath(", "__SEC4_INTRINSIC_ERR_WITH_PATH__(");
    lowered = lowered.replace("err_with_path(", "__SEC4_INTRINSIC_ERR_WITH_PATH__(");
    lowered = lowered.replace("err.validation(", "__SEC4_INTRINSIC_ERR_VALIDATION__(");
    lowered = lowered.replace("err_validation(", "__SEC4_INTRINSIC_ERR_VALIDATION__(");
    lowered = lowered.replace("err.notFound(", "__SEC4_INTRINSIC_ERR_NOT_FOUND__(");
    lowered = lowered.replace("err_not_found(", "__SEC4_INTRINSIC_ERR_NOT_FOUND__(");
    lowered = lowered.replace("err.rateLimit(", "__SEC4_INTRINSIC_ERR_RATE_LIMIT__(");
    lowered = lowered.replace("err_rate_limit(", "__SEC4_INTRINSIC_ERR_RATE_LIMIT__(");
    lowered = lowered.replace("err.conflict(", "__SEC4_INTRINSIC_ERR_CONFLICT__(");
    lowered = lowered.replace("err_conflict(", "__SEC4_INTRINSIC_ERR_CONFLICT__(");
    lowered = lowered.replace("err.internal(", "__SEC4_INTRINSIC_ERR_INTERNAL__(");
    lowered = lowered.replace("err_internal(", "__SEC4_INTRINSIC_ERR_INTERNAL__(");
    lowered = lowered.replace("err.auth(", "__SEC4_INTRINSIC_ERR_AUTH__(");
    lowered = lowered.replace("err_auth(", "__SEC4_INTRINSIC_ERR_AUTH__(");
    lowered = lowered.replace("__SEC4_CONSTRUCTOR_CTX__(", "sec4_rt_ctx(");
    lowered = lowered.replace("__SEC4_CONSTRUCTOR_DB_CAP__(", "sec4_rt_db_cap(");
    lowered = lowered.replace("__SEC4_CONSTRUCTOR_FS_CAP__(", "sec4_rt_fs_cap(");
    lowered = lowered.replace("__SEC4_CONSTRUCTOR_NET_CAP__(", "sec4_rt_net_cap(");
    lowered = lowered.replace(
        "__SEC4_CONSTRUCTOR_INTERNAL_NET_CAP__(",
        "sec4_rt_internal_net_cap(",
    );
    lowered = lowered.replace(
        "__SEC4_CONSTRUCTOR_SECRETS_CAP__(",
        "sec4_rt_secrets_cap(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_TIME_NOW__(", "sec4_rt_time_now(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_INFO__(", "sec4_rt_log_info(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_WARN__(", "sec4_rt_log_warn(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_ERROR__(", "sec4_rt_log_error(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_ANY__(", "sec4_rt_log_any(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_EVENT__(", "sec4_rt_log_event(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_FIELD__(", "sec4_rt_log_field(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_OBJ__(", "sec4_rt_log_obj(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_STR__(", "sec4_rt_log_str(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_I64__(", "sec4_rt_log_i64(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_BOOL__(", "sec4_rt_log_bool(");
    lowered = lowered.replace("__SEC4_INTRINSIC_LOG_REDACTED__(", "sec4_rt_log_redacted(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_LOG_ATTR_REDACTED__(",
        "sec4_rt_log_attr_redacted(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_LOG_WITH_ATTR__(",
        "sec4_rt_log_with_attr(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_LOG_WITH_HTTP__(",
        "sec4_rt_log_with_http(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_LOG_WITH_ERROR__(",
        "sec4_rt_log_with_error(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_REQ_JSON__(", "sec4_rt_req_json(");
    lowered = lowered.replace("__SEC4_INTRINSIC_JSON_DECODE__(", "sec4_rt_json_decode(");
    lowered = lowered.replace("__SEC4_INTRINSIC_JSON_ENCODE__(", "sec4_rt_json_encode(");
    lowered = lowered.replace("__SEC4_INTRINSIC_REQ_BODY__(", "sec4_rt_req_body(");
    lowered = lowered.replace("__SEC4_INTRINSIC_REQ_QUERY__(", "sec4_rt_req_query(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_REQ_PATH_PARAM__(",
        "sec4_rt_req_path_param(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_REQ_HEADER__(", "sec4_rt_req_header(");
    lowered = lowered.replace("__SEC4_INTRINSIC_RES_JSON__(", "sec4_rt_res_json(");
    lowered = lowered.replace("__SEC4_INTRINSIC_RES_OK__(", "sec4_rt_res_ok(");
    lowered = lowered.replace("__SEC4_INTRINSIC_RES_OK_META__(", "sec4_rt_res_ok_meta(");
    lowered = lowered.replace("__SEC4_INTRINSIC_RES_HTML__(", "sec4_rt_res_html(");
    lowered = lowered.replace("__SEC4_INTRINSIC_RES_TEXT__(", "sec4_rt_res_text(");
    lowered = lowered.replace("__SEC4_INTRINSIC_SET_HEADER__(", "sec4_rt_set_header(");
    lowered = lowered.replace("__SEC4_INTRINSIC_COOKIE_BUILD__(", "sec4_rt_cookie_build(");
    lowered = lowered.replace("__SEC4_INTRINSIC_SET_COOKIE__(", "sec4_rt_set_cookie(");
    lowered = lowered.replace("__SEC4_INTRINSIC_SQL_Q__(", "sec4_rt_sql_q(");
    lowered = lowered.replace("__SEC4_INTRINSIC_DB_EXEC__(", "sec4_rt_db_exec(");
    lowered = lowered.replace("__SEC4_INTRINSIC_DB_TX__(", "sec4_rt_db_tx(");
    lowered = lowered.replace("__SEC4_INTRINSIC_DB_EXEC_TX__(", "sec4_rt_db_exec_tx(");
    lowered = lowered.replace("__SEC4_INTRINSIC_DB_QUERY_ONE__(", "sec4_rt_db_query_one(");
    lowered = lowered.replace("__SEC4_INTRINSIC_FS_READ__(", "sec4_rt_fs_read(");
    lowered = lowered.replace("__SEC4_INTRINSIC_FS_WRITE__(", "sec4_rt_fs_write(");
    lowered = lowered.replace("__SEC4_INTRINSIC_HTTP_GET__(", "sec4_rt_http_get(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_HTTP_GET_INTERNAL__(",
        "sec4_rt_http_get_internal(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_SECRET_GET__(", "sec4_rt_secret_get(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_SECRET_REDACT__(",
        "sec4_rt_secret_redact(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_SECRET_REVEAL__(",
        "sec4_rt_secret_reveal(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_CRYPTO_CT_EQ__(", "sec4_rt_crypto_ct_eq(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_VALIDATE_HEADER_VALUE__(",
        "sec4_rt_validate_header_value(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_VALIDATE_EMAIL__(",
        "sec4_rt_validate_email(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_VALIDATE_UUID__(",
        "sec4_rt_validate_uuid(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_VALIDATE_INT64__(",
        "sec4_rt_validate_int64(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_VALIDATE_NON_EMPTY__(",
        "sec4_rt_validate_non_empty(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_SANITIZE_HTML__(",
        "sec4_rt_sanitize_html(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_URL_PUBLIC__(", "sec4_rt_url_public(");
    lowered = lowered.replace("__SEC4_INTRINSIC_URL_INTERNAL__(", "sec4_rt_url_internal(");
    lowered = lowered.replace("__SEC4_INTRINSIC_PATH_UNDER__(", "sec4_rt_path_under(");
    lowered = lowered.replace("__SEC4_INTRINSIC_PATH_BASE__(", "sec4_rt_path_base(");
    lowered = lowered.replace("__SEC4_INTRINSIC_HEADERS_NAME__(", "sec4_rt_headers_name(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_HEADERS_VALUE__(",
        "sec4_rt_headers_value(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_HTTP_ROUTER__(", "sec4_rt_http_router(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_HTTP_ROUTE_GET__(",
        "sec4_rt_http_route_get(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_HTTP_ROUTE_POST__(",
        "sec4_rt_http_route_post(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_HTTP_SERVE__(", "sec4_rt_http_serve(");
    lowered = lowered.replace("__SEC4_INTRINSIC_WITH_CORS__(", "sec4_rt_with_cors(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_WITH_SECURITY_HEADERS__(",
        "sec4_rt_with_security_headers(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_WITH_CSRF__(", "sec4_rt_with_csrf(");
    lowered = lowered.replace("__SEC4_INTRINSIC_WITH_AUTH__(", "sec4_rt_with_auth(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_SEC_DEFAULT_HEADERS__(",
        "sec4_rt_sec_default_headers(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_SEC_CSP__(", "sec4_rt_sec_csp(");
    lowered = lowered.replace("__SEC4_INTRINSIC_SEC_CSP_ADD__(", "sec4_rt_sec_csp_add(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_CORS_FROM_POLICY__(",
        "sec4_rt_cors_from_policy(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_CORS_ORIGIN__(", "sec4_rt_cors_origin(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_CSRF_FROM_POLICY__(",
        "sec4_rt_csrf_from_policy(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_CSRF_ISSUE_TOKEN__(",
        "sec4_rt_csrf_issue_token(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_AUTH_FROM_POLICY__(",
        "sec4_rt_auth_from_policy(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_AUTH_REQUIRE__(", "sec4_rt_auth_require(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_AUTH_REQUIRE_ROLE__(",
        "sec4_rt_auth_require_role(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_ERR_VALIDATION__(",
        "sec4_rt_err_validation(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_ERR_AUTH__(", "sec4_rt_err_auth(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_ERR_NOT_FOUND__(",
        "sec4_rt_err_not_found(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_ERR_CONFLICT__(", "sec4_rt_err_conflict(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_ERR_RATE_LIMIT__(",
        "sec4_rt_err_rate_limit(",
    );
    lowered = lowered.replace("__SEC4_INTRINSIC_ERR_INTERNAL__(", "sec4_rt_err_internal(");
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_ERR_WITH_PATH__(",
        "sec4_rt_err_with_path(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_ERR_WITH_DETAIL__(",
        "sec4_rt_err_with_detail(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_ERR_WITH_LIMIT__(",
        "sec4_rt_err_with_limit(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_ERR_WITH_DEPENDENCY__(",
        "sec4_rt_err_with_dependency(",
    );
    lowered = lowered.replace(
        "__SEC4_INTRINSIC_ERR_WITH_CAUSE__(",
        "sec4_rt_err_with_cause(",
    );
    lowered
}
