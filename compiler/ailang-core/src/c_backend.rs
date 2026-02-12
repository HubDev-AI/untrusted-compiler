use crate::mir::{MirFunction, MirInstructionKind, MirProgram, MirTerminator};
use std::collections::BTreeSet;
use std::fmt::Write;

pub fn emit_c_program(program: &MirProgram) -> String {
    let mut out = String::new();
    out.push_str("#include <stdbool.h>\n#include <stdint.h>\n#include \"ailang_runtime.h\"\n\n");

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
    include_str!("../../../runtime/c/ailang_runtime.h")
}

pub fn emit_runtime_source() -> &'static str {
    include_str!("../../../runtime/c/ailang_runtime.c")
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
        Some("Bool") => Some("ailang_rt_identity_bool"),
        Some("Int") | Some("Int64") => Some("ailang_rt_identity_i64"),
        _ => None,
    }
}

fn lower_c_expr(expr: &str) -> String {
    let mut lowered = expr.to_string();
    lowered = lowered.replace("time.now(", "__AILANG_INTRINSIC_TIME_NOW__(");
    lowered = lowered.replace("time_now(", "__AILANG_INTRINSIC_TIME_NOW__(");
    lowered = lowered.replace("log.info(", "__AILANG_INTRINSIC_LOG_ANY__(");
    lowered = lowered.replace("log.warn(", "__AILANG_INTRINSIC_LOG_ANY__(");
    lowered = lowered.replace("log.error(", "__AILANG_INTRINSIC_LOG_ANY__(");
    lowered = lowered.replace("log.emit(", "__AILANG_INTRINSIC_LOG_ANY__(");
    lowered = lowered.replace("log_info(", "__AILANG_INTRINSIC_LOG_ANY__(");
    lowered = lowered.replace("log_warn(", "__AILANG_INTRINSIC_LOG_ANY__(");
    lowered = lowered.replace("log_error(", "__AILANG_INTRINSIC_LOG_ANY__(");
    lowered = lowered.replace("log_emit(", "__AILANG_INTRINSIC_LOG_ANY__(");
    lowered = lowered.replace("req.json(", "__AILANG_INTRINSIC_REQ_JSON__(");
    lowered = lowered.replace("req_json(", "__AILANG_INTRINSIC_REQ_JSON__(");
    lowered = lowered.replace("res.json(", "__AILANG_INTRINSIC_RES_JSON__(");
    lowered = lowered.replace("res_json(", "__AILANG_INTRINSIC_RES_JSON__(");
    lowered = lowered.replace("res.html(", "__AILANG_INTRINSIC_RES_HTML__(");
    lowered = lowered.replace("res_html(", "__AILANG_INTRINSIC_RES_HTML__(");
    lowered = lowered.replace("res.setHeader(", "__AILANG_INTRINSIC_SET_HEADER__(");
    lowered = lowered.replace("set_header(", "__AILANG_INTRINSIC_SET_HEADER__(");
    lowered = lowered.replace("res.addCookie(", "__AILANG_INTRINSIC_SET_COOKIE__(");
    lowered = lowered.replace("set_cookie(", "__AILANG_INTRINSIC_SET_COOKIE__(");
    lowered = lowered.replace("db.exec(", "__AILANG_INTRINSIC_DB_EXEC__(");
    lowered = lowered.replace("db_write(", "__AILANG_INTRINSIC_DB_EXEC__(");
    lowered = lowered.replace("db.queryOne(", "__AILANG_INTRINSIC_DB_QUERY_ONE__(");
    lowered = lowered.replace("db_read(", "__AILANG_INTRINSIC_DB_QUERY_ONE__(");
    lowered = lowered.replace("fs.read(", "__AILANG_INTRINSIC_FS_READ__(");
    lowered = lowered.replace("fs_read(", "__AILANG_INTRINSIC_FS_READ__(");
    lowered = lowered.replace("fs.write(", "__AILANG_INTRINSIC_FS_WRITE__(");
    lowered = lowered.replace("fs_write(", "__AILANG_INTRINSIC_FS_WRITE__(");
    lowered = lowered.replace("httpClient.get(", "__AILANG_INTRINSIC_HTTP_GET__(");
    lowered = lowered.replace("net_call(", "__AILANG_INTRINSIC_HTTP_GET__(");
    lowered = lowered.replace(
        "httpClient.getInternal(",
        "__AILANG_INTRINSIC_HTTP_GET_INTERNAL__(",
    );
    lowered = lowered.replace(
        "net_internal_call(",
        "__AILANG_INTRINSIC_HTTP_GET_INTERNAL__(",
    );
    lowered = lowered.replace("secrets.get(", "__AILANG_INTRINSIC_SECRET_GET__(");
    lowered = lowered.replace("secret_read(", "__AILANG_INTRINSIC_SECRET_GET__(");
    lowered = lowered.replace("secrets.reveal(", "__AILANG_INTRINSIC_SECRET_REVEAL__(");
    lowered = lowered.replace("secret_reveal(", "__AILANG_INTRINSIC_SECRET_REVEAL__(");
    lowered = lowered.replace(
        "validate.headerValue(",
        "__AILANG_INTRINSIC_VALIDATE_HEADER_VALUE__(",
    );
    lowered = lowered.replace(
        "validate_header_value(",
        "__AILANG_INTRINSIC_VALIDATE_HEADER_VALUE__(",
    );
    lowered = lowered.replace("validate.email(", "__AILANG_INTRINSIC_VALIDATE_EMAIL__(");
    lowered = lowered.replace("validate_email(", "__AILANG_INTRINSIC_VALIDATE_EMAIL__(");
    lowered = lowered.replace("validate.uuid(", "__AILANG_INTRINSIC_VALIDATE_UUID__(");
    lowered = lowered.replace("validate_uuid(", "__AILANG_INTRINSIC_VALIDATE_UUID__(");
    lowered = lowered.replace("validate.int64(", "__AILANG_INTRINSIC_VALIDATE_INT64__(");
    lowered = lowered.replace("validate_int64(", "__AILANG_INTRINSIC_VALIDATE_INT64__(");
    lowered = lowered.replace("validate.nonEmpty(", "__AILANG_INTRINSIC_VALIDATE_NON_EMPTY__(");
    lowered = lowered.replace("validate_non_empty(", "__AILANG_INTRINSIC_VALIDATE_NON_EMPTY__(");
    lowered = lowered.replace("sanitize.html(", "__AILANG_INTRINSIC_SANITIZE_HTML__(");
    lowered = lowered.replace("sanitize_html(", "__AILANG_INTRINSIC_SANITIZE_HTML__(");
    lowered = lowered.replace("url.public(", "__AILANG_INTRINSIC_URL_PUBLIC__(");
    lowered = lowered.replace("url_public(", "__AILANG_INTRINSIC_URL_PUBLIC__(");
    lowered = lowered.replace("url.internal(", "__AILANG_INTRINSIC_URL_INTERNAL__(");
    lowered = lowered.replace("url_internal(", "__AILANG_INTRINSIC_URL_INTERNAL__(");
    lowered = lowered.replace("path.under(", "__AILANG_INTRINSIC_PATH_UNDER__(");
    lowered = lowered.replace("path_under(", "__AILANG_INTRINSIC_PATH_UNDER__(");
    lowered = lowered.replace("validate.pathUnder(", "__AILANG_INTRINSIC_PATH_UNDER__(");
    lowered = lowered.replace("validate_path_under(", "__AILANG_INTRINSIC_PATH_UNDER__(");
    lowered = lowered.replace("http.router(", "__AILANG_INTRINSIC_HTTP_ROUTER__(");
    lowered = lowered.replace("http_router(", "__AILANG_INTRINSIC_HTTP_ROUTER__(");
    lowered = lowered.replace("http.get(", "__AILANG_INTRINSIC_HTTP_ROUTE_GET__(");
    lowered = lowered.replace("http_get_route(", "__AILANG_INTRINSIC_HTTP_ROUTE_GET__(");
    lowered = lowered.replace("http.post(", "__AILANG_INTRINSIC_HTTP_ROUTE_POST__(");
    lowered = lowered.replace("http_post_route(", "__AILANG_INTRINSIC_HTTP_ROUTE_POST__(");
    lowered = lowered.replace("http.serve(", "__AILANG_INTRINSIC_HTTP_SERVE__(");
    lowered = lowered.replace("http_serve(", "__AILANG_INTRINSIC_HTTP_SERVE__(");
    lowered = lowered.replace("cors.withCors(", "__AILANG_INTRINSIC_WITH_CORS__(");
    lowered = lowered.replace("withCors(", "__AILANG_INTRINSIC_WITH_CORS__(");
    lowered = lowered.replace("cors_with(", "__AILANG_INTRINSIC_WITH_CORS__(");
    lowered = lowered.replace(
        "sec.withSecurityHeaders(",
        "__AILANG_INTRINSIC_WITH_SECURITY_HEADERS__(",
    );
    lowered = lowered.replace(
        "withSecurityHeaders(",
        "__AILANG_INTRINSIC_WITH_SECURITY_HEADERS__(",
    );
    lowered = lowered.replace(
        "sec_with_security_headers(",
        "__AILANG_INTRINSIC_WITH_SECURITY_HEADERS__(",
    );
    lowered = lowered.replace("csrf.withCsrf(", "__AILANG_INTRINSIC_WITH_CSRF__(");
    lowered = lowered.replace("withCsrf(", "__AILANG_INTRINSIC_WITH_CSRF__(");
    lowered = lowered.replace("csrf_with(", "__AILANG_INTRINSIC_WITH_CSRF__(");
    lowered = lowered.replace("auth.withAuth(", "__AILANG_INTRINSIC_WITH_AUTH__(");
    lowered = lowered.replace("withAuth(", "__AILANG_INTRINSIC_WITH_AUTH__(");
    lowered = lowered.replace("auth_with(", "__AILANG_INTRINSIC_WITH_AUTH__(");
    lowered = lowered.replace("__AILANG_INTRINSIC_TIME_NOW__(", "ailang_rt_time_now(");
    lowered = lowered.replace("__AILANG_INTRINSIC_LOG_ANY__(", "ailang_rt_log_any(");
    lowered = lowered.replace("__AILANG_INTRINSIC_REQ_JSON__(", "ailang_rt_req_json(");
    lowered = lowered.replace("__AILANG_INTRINSIC_RES_JSON__(", "ailang_rt_res_json(");
    lowered = lowered.replace("__AILANG_INTRINSIC_RES_HTML__(", "ailang_rt_res_html(");
    lowered = lowered.replace("__AILANG_INTRINSIC_SET_HEADER__(", "ailang_rt_set_header(");
    lowered = lowered.replace("__AILANG_INTRINSIC_SET_COOKIE__(", "ailang_rt_set_cookie(");
    lowered = lowered.replace("__AILANG_INTRINSIC_DB_EXEC__(", "ailang_rt_db_exec(");
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_DB_QUERY_ONE__(",
        "ailang_rt_db_query_one(",
    );
    lowered = lowered.replace("__AILANG_INTRINSIC_FS_READ__(", "ailang_rt_fs_read(");
    lowered = lowered.replace("__AILANG_INTRINSIC_FS_WRITE__(", "ailang_rt_fs_write(");
    lowered = lowered.replace("__AILANG_INTRINSIC_HTTP_GET__(", "ailang_rt_http_get(");
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_HTTP_GET_INTERNAL__(",
        "ailang_rt_http_get_internal(",
    );
    lowered = lowered.replace("__AILANG_INTRINSIC_SECRET_GET__(", "ailang_rt_secret_get(");
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_SECRET_REVEAL__(",
        "ailang_rt_secret_reveal(",
    );
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_VALIDATE_HEADER_VALUE__(",
        "ailang_rt_validate_header_value(",
    );
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_VALIDATE_EMAIL__(",
        "ailang_rt_validate_email(",
    );
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_VALIDATE_UUID__(",
        "ailang_rt_validate_uuid(",
    );
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_VALIDATE_INT64__(",
        "ailang_rt_validate_int64(",
    );
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_VALIDATE_NON_EMPTY__(",
        "ailang_rt_validate_non_empty(",
    );
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_SANITIZE_HTML__(",
        "ailang_rt_sanitize_html(",
    );
    lowered = lowered.replace("__AILANG_INTRINSIC_URL_PUBLIC__(", "ailang_rt_url_public(");
    lowered = lowered.replace("__AILANG_INTRINSIC_URL_INTERNAL__(", "ailang_rt_url_internal(");
    lowered = lowered.replace("__AILANG_INTRINSIC_PATH_UNDER__(", "ailang_rt_path_under(");
    lowered = lowered.replace("__AILANG_INTRINSIC_HTTP_ROUTER__(", "ailang_rt_http_router(");
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_HTTP_ROUTE_GET__(",
        "ailang_rt_http_route_get(",
    );
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_HTTP_ROUTE_POST__(",
        "ailang_rt_http_route_post(",
    );
    lowered = lowered.replace("__AILANG_INTRINSIC_HTTP_SERVE__(", "ailang_rt_http_serve(");
    lowered = lowered.replace("__AILANG_INTRINSIC_WITH_CORS__(", "ailang_rt_with_cors(");
    lowered = lowered.replace(
        "__AILANG_INTRINSIC_WITH_SECURITY_HEADERS__(",
        "ailang_rt_with_security_headers(",
    );
    lowered = lowered.replace("__AILANG_INTRINSIC_WITH_CSRF__(", "ailang_rt_with_csrf(");
    lowered = lowered.replace("__AILANG_INTRINSIC_WITH_AUTH__(", "ailang_rt_with_auth(");
    lowered
}
