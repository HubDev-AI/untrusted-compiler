#ifndef SEC4_RUNTIME_H
#define SEC4_RUNTIME_H

#include <stdbool.h>
#include <stdint.h>

int64_t sec4_rt_identity_i64(int64_t value);
bool sec4_rt_identity_bool(bool value);
int64_t sec4_rt_ctx(void);
int64_t sec4_rt_db_cap(void);
int64_t sec4_rt_fs_cap(void);
int64_t sec4_rt_net_cap(void);
int64_t sec4_rt_internal_net_cap(void);
int64_t sec4_rt_secrets_cap(void);
int64_t sec4_rt_time_now(void);
void sec4_rt_log_any(int64_t event);
void sec4_rt_log_info(int64_t event);
void sec4_rt_log_warn(int64_t event);
void sec4_rt_log_error(int64_t event);
int64_t sec4_rt_log_event(const char *event_name);
int64_t sec4_rt_log_field(const char *key, int64_t value);
int64_t sec4_rt_log_obj(int64_t field);
int64_t sec4_rt_log_str(const char *value);
int64_t sec4_rt_log_i64(int64_t value);
int64_t sec4_rt_log_bool(int64_t value);
int64_t sec4_rt_log_redacted(const char *value);
int64_t sec4_rt_log_attr_redacted(const char *value);
int64_t sec4_rt_log_with_attr(int64_t event, const char *key, int64_t value);
int64_t sec4_rt_log_with_http(
    int64_t event,
    const char *method,
    const char *path,
    int64_t status,
    int64_t duration_ms
);
int64_t sec4_rt_log_with_error(int64_t event, int64_t error);
int64_t sec4_rt_req_json(int64_t schema);
int64_t sec4_rt_json_decode(int64_t ctx, int64_t schema, int64_t raw);
int64_t sec4_rt_json_encode(int64_t schema, int64_t value);
int64_t sec4_rt_req_body(int64_t ctx, int64_t req);
int64_t sec4_rt_req_query(const char *name);
int64_t sec4_rt_req_path_param(const char *name);
int64_t sec4_rt_req_header(const char *name);
int64_t sec4_rt_req_cookie(const char *name);
int64_t sec4_rt_req_method(void);
int64_t sec4_rt_req_path(void);
int64_t sec4_rt_req_http_version(void);
int64_t sec4_rt_res_json(int64_t schema, int64_t value);
int64_t sec4_rt_res_ok(int64_t status, int64_t schema, int64_t value);
int64_t sec4_rt_res_ok_meta(int64_t status, int64_t schema, int64_t value, int64_t meta);
int64_t sec4_rt_res_html(int64_t html);
int64_t sec4_rt_res_text(int64_t status, int64_t body);
int64_t sec4_rt_set_header(int64_t name, int64_t value);
int64_t sec4_rt_cookie_build(int64_t name, int64_t value);
int64_t sec4_rt_set_cookie(int64_t cookie);
int64_t sec4_rt_sql_q(const char *query_template, int64_t params);
int64_t sec4_rt_db_exec(int64_t db, int64_t query);
int64_t sec4_rt_db_tx(int64_t db);
int64_t sec4_rt_db_exec_tx(int64_t tx, int64_t query);
int64_t sec4_rt_db_query_one(int64_t db, int64_t query, int64_t row_schema);
int64_t sec4_rt_fs_read(int64_t fs, int64_t path);
int64_t sec4_rt_fs_write(int64_t fs, int64_t path, int64_t value);
int64_t sec4_rt_http_get(int64_t net, int64_t url);
int64_t sec4_rt_http_get_internal(int64_t net, int64_t url);
int64_t sec4_rt_secret_get(int64_t secrets_cap, const char *name);
int64_t sec4_rt_secret_redact(int64_t secret_value);
int64_t sec4_rt_secret_reveal(int64_t secrets_cap, int64_t secret_value);
bool sec4_rt_crypto_ct_eq(int64_t left_secret, int64_t right_secret);
int64_t sec4_rt_validate_header_value(int64_t input);
int64_t sec4_rt_validate_email(int64_t input);
int64_t sec4_rt_validate_uuid(int64_t input);
int64_t sec4_rt_validate_int64(int64_t input);
int64_t sec4_rt_validate_non_empty(int64_t input);
int64_t sec4_rt_sanitize_html(int64_t input);
int64_t sec4_rt_url_public(int64_t input);
int64_t sec4_rt_url_internal(int64_t input);
int64_t sec4_rt_path_under(int64_t base, int64_t input);
int64_t sec4_rt_path_base_literal(const char *input);
int64_t sec4_rt_path_base_handle(int64_t input);
#define sec4_rt_path_base(input)                                              \
  _Generic(                                                                   \
      (input),                                                                \
      const char *: sec4_rt_path_base_literal,                                \
      char *: sec4_rt_path_base_literal,                                      \
      default: sec4_rt_path_base_handle                                       \
  )((input))
int64_t sec4_rt_headers_name(int64_t input);
int64_t sec4_rt_headers_value(int64_t input);
int64_t sec4_rt_http_router(void);
int64_t sec4_rt_http_route_get(
    int64_t router,
    const char *path,
    int64_t (*handler)(void)
);
int64_t sec4_rt_http_route_post(
    int64_t router,
    const char *path,
    int64_t (*handler)(void)
);
int64_t sec4_rt_http_serve(int64_t port, int64_t router);
int64_t sec4_rt_with_cors(int64_t router, int64_t cfg);
int64_t sec4_rt_with_security_headers(int64_t router, int64_t cfg);
int64_t sec4_rt_with_csrf(int64_t router, int64_t cfg);
int64_t sec4_rt_with_auth(int64_t router, int64_t cfg);
int64_t sec4_rt_sec_default_headers(void);
int64_t sec4_rt_sec_csp(void);
int64_t sec4_rt_sec_csp_add(int64_t csp, const char *directive, const char *value);
int64_t sec4_rt_cors_from_policy(void);
int64_t sec4_rt_cors_origin(int64_t origin);
int64_t sec4_rt_csrf_from_policy(void);
int64_t sec4_rt_csrf_issue_token(int64_t ctx);
int64_t sec4_rt_auth_from_policy(void);
int64_t sec4_rt_auth_require(int64_t ctx);
int64_t sec4_rt_auth_require_role(int64_t ctx, const char *required_role);
int64_t sec4_rt_err_validation(int64_t code, int64_t message);
int64_t sec4_rt_err_auth(int64_t code, int64_t message, int64_t status);
int64_t sec4_rt_err_not_found(int64_t code, int64_t message);
int64_t sec4_rt_err_conflict(int64_t code, int64_t message);
int64_t sec4_rt_err_rate_limit(int64_t code, int64_t message, int64_t limit);
int64_t sec4_rt_err_internal(int64_t message);
int64_t sec4_rt_err_with_path(int64_t error, const char *path);
int64_t sec4_rt_err_with_detail(int64_t error, const char *key, int64_t value);
int64_t sec4_rt_err_with_limit(int64_t error, const char *name, int64_t value, int64_t max);
int64_t sec4_rt_err_with_dependency(
    int64_t error,
    const char *service,
    const char *operation,
    int64_t retryable
);
int64_t sec4_rt_err_with_cause(int64_t error, int64_t cause);

#endif
