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

#endif
