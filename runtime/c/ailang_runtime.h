#ifndef AILANG_RUNTIME_H
#define AILANG_RUNTIME_H

#include <stdbool.h>
#include <stdint.h>

int64_t ailang_rt_identity_i64(int64_t value);
bool ailang_rt_identity_bool(bool value);
int64_t ailang_rt_time_now(void);
void ailang_rt_log_any();
int64_t ailang_rt_req_json();
int64_t ailang_rt_res_json();
int64_t ailang_rt_res_html();
int64_t ailang_rt_set_header();
int64_t ailang_rt_set_cookie();
int64_t ailang_rt_db_exec();
int64_t ailang_rt_db_query_one();
int64_t ailang_rt_fs_read();
int64_t ailang_rt_fs_write();
int64_t ailang_rt_http_get();
int64_t ailang_rt_http_get_internal();
int64_t ailang_rt_secret_get();
int64_t ailang_rt_secret_reveal();
int64_t ailang_rt_validate_header_value();
int64_t ailang_rt_validate_email();
int64_t ailang_rt_validate_uuid();
int64_t ailang_rt_validate_int64();
int64_t ailang_rt_validate_non_empty();
int64_t ailang_rt_sanitize_html();
int64_t ailang_rt_url_public();
int64_t ailang_rt_url_internal();
int64_t ailang_rt_path_under();
int64_t ailang_rt_http_router();
int64_t ailang_rt_http_route_get();
int64_t ailang_rt_http_route_post();
int64_t ailang_rt_http_serve();

#endif
