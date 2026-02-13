#ifndef AILANG_RUNTIME_H
#define AILANG_RUNTIME_H

#include <stdbool.h>
#include <stdint.h>

typedef int64_t (*ailang_handler_fn)(void);

int64_t ailang_rt_identity_i64(int64_t value);
bool ailang_rt_identity_bool(bool value);

int64_t ailang_rt_http_router(void);
int64_t ailang_rt_http_route_get(int64_t router, const char *path, ailang_handler_fn handler);
int64_t ailang_rt_http_route_post(int64_t router, const char *path, ailang_handler_fn handler);
int64_t ailang_rt_http_serve(int64_t port, int64_t router);

int64_t ailang_rt_req_json(const char *schema_name);
int64_t ailang_rt_req_path_param(const char *name);

int64_t ailang_rt_res_text(int64_t status, const char *text);
int64_t ailang_rt_res_ok(int64_t status, const char *schema_name, int64_t value);
int64_t ailang_rt_res_json(int64_t status, const char *schema_name, int64_t value);

#endif
