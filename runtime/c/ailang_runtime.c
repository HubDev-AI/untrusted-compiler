#include "ailang_runtime.h"

int64_t ailang_rt_identity_i64(int64_t value) {
  return value;
}

bool ailang_rt_identity_bool(bool value) {
  return value;
}

int64_t ailang_rt_time_now(void) {
  return 0;
}

void ailang_rt_log_any() {
}

int64_t ailang_rt_req_json() {
  return 0;
}

int64_t ailang_rt_res_json() {
  return 0;
}

int64_t ailang_rt_res_html() {
  return 0;
}

int64_t ailang_rt_set_header() {
  return 0;
}

int64_t ailang_rt_set_cookie() {
  return 0;
}

int64_t ailang_rt_db_exec() {
  return 0;
}

int64_t ailang_rt_db_query_one() {
  return 0;
}

int64_t ailang_rt_fs_read() {
  return 0;
}

int64_t ailang_rt_fs_write() {
  return 0;
}

int64_t ailang_rt_http_get() {
  return 0;
}

int64_t ailang_rt_http_get_internal() {
  return 0;
}

int64_t ailang_rt_secret_get() {
  return 0;
}

int64_t ailang_rt_secret_reveal() {
  return 0;
}
