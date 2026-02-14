#include "sec4_runtime.h"

#include <arpa/inet.h>
#include <ctype.h>
#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <stdio.h>
#include <sys/select.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

#define SEC4_RT_MAX_ROUTERS 16
#define SEC4_RT_MAX_ROUTES 64
#define SEC4_RT_MAX_PATH_BYTES 256
#define SEC4_RT_MAX_RESPONSE_BYTES 4096
#define SEC4_RT_MAX_REQUEST_BODY_BYTES 4096
#define SEC4_RT_REQUEST_BUFFER_BYTES 8192
#define SEC4_RT_DEFAULT_ONESHOT_TIMEOUT_MS 200

typedef int64_t (*sec4_rt_handler_fn)(void);

typedef struct {
  char method[8];
  char path[SEC4_RT_MAX_PATH_BYTES];
  sec4_rt_handler_fn handler;
} sec4_rt_route;

typedef struct {
  bool active;
  int64_t handle;
  size_t route_count;
  sec4_rt_route routes[SEC4_RT_MAX_ROUTES];
} sec4_rt_router_state;

typedef struct {
  bool active;
  int64_t status;
  char content_type[64];
  char body[SEC4_RT_MAX_RESPONSE_BYTES];
  size_t body_len;
} sec4_rt_response_state;

typedef struct {
  bool has_request;
  char method[8];
  char path[SEC4_RT_MAX_PATH_BYTES];
  char body[SEC4_RT_MAX_REQUEST_BODY_BYTES];
  size_t body_len;
  bool body_limit_exceeded;
  bool has_content_type;
  bool content_type_is_json;
  bool json_checked;
  bool json_valid;
} sec4_rt_request_state;

static sec4_rt_router_state g_sec4_rt_routers[SEC4_RT_MAX_ROUTERS];
static int64_t g_sec4_rt_next_router_handle = 1;
static sec4_rt_response_state g_sec4_rt_response;
static sec4_rt_request_state g_sec4_rt_request;

static void sec4_rt_reset_response(void) {
  g_sec4_rt_response.active = false;
  g_sec4_rt_response.status = 204;
  g_sec4_rt_response.content_type[0] = '\0';
  g_sec4_rt_response.body[0] = '\0';
  g_sec4_rt_response.body_len = 0;
}

static void sec4_rt_store_response(
    int64_t status,
    const char *content_type,
    const char *body
) {
  sec4_rt_reset_response();
  g_sec4_rt_response.active = true;
  g_sec4_rt_response.status = status > 0 ? status : 200;

  if (content_type == NULL || content_type[0] == '\0') {
    content_type = "text/plain; charset=utf-8";
  }
  strncpy(
      g_sec4_rt_response.content_type,
      content_type,
      sizeof(g_sec4_rt_response.content_type) - 1
  );
  g_sec4_rt_response.content_type[sizeof(g_sec4_rt_response.content_type) - 1] = '\0';

  if (body == NULL) {
    g_sec4_rt_response.body[0] = '\0';
    g_sec4_rt_response.body_len = 0;
    return;
  }

  size_t body_len = strlen(body);
  if (body_len >= sizeof(g_sec4_rt_response.body)) {
    body_len = sizeof(g_sec4_rt_response.body) - 1;
  }
  memcpy(g_sec4_rt_response.body, body, body_len);
  g_sec4_rt_response.body[body_len] = '\0';
  g_sec4_rt_response.body_len = body_len;
}

static void sec4_rt_store_std_error_response(
    int64_t status,
    const char *code,
    const char *kind,
    const char *message
) {
  char payload[768];
  int written = snprintf(
      payload,
      sizeof(payload),
      "{\"error\":{\"code\":\"%s\",\"kind\":\"%s\",\"message\":\"%s\",\"status\":%lld,\"traceId\":\"rt_trace\",\"timeMs\":0}}",
      code != NULL ? code : "INTERNAL.ERROR",
      kind != NULL ? kind : "internal",
      message != NULL ? message : "internal error",
      (long long) status
  );

  if (written <= 0 || (size_t) written >= sizeof(payload)) {
    sec4_rt_store_response(
        status,
        "application/json; charset=utf-8",
        "{\"error\":{\"code\":\"INTERNAL.ERROR\",\"kind\":\"internal\",\"message\":\"error\",\"status\":500,\"traceId\":\"rt_trace\",\"timeMs\":0}}"
    );
    return;
  }

  sec4_rt_store_response(status, "application/json; charset=utf-8", payload);
}

static void sec4_rt_reset_request(void) {
  memset(&g_sec4_rt_request, 0, sizeof(g_sec4_rt_request));
}

static bool sec4_rt_is_likely_json(const char *body, size_t body_len) {
  if (body == NULL || body_len == 0) {
    return false;
  }

  size_t start = 0;
  while (start < body_len && isspace((unsigned char) body[start])) {
    start += 1;
  }
  if (start >= body_len) {
    return false;
  }

  size_t end = body_len;
  while (end > start && isspace((unsigned char) body[end - 1])) {
    end -= 1;
  }
  if (end <= start) {
    return false;
  }

  char first = body[start];
  char last = body[end - 1];
  return (first == '{' && last == '}') || (first == '[' && last == ']');
}

static size_t sec4_rt_parse_content_length(const char *request, size_t request_len) {
  const char *cursor = request;
  const char *request_end = request + request_len;

  while (cursor < request_end) {
    const char *line_end = strstr(cursor, "\r\n");
    if (line_end == NULL || line_end > request_end) {
      break;
    }
    if (line_end == cursor) {
      break;
    }

    if (strncmp(cursor, "Content-Length:", 15) == 0) {
      const char *value = cursor + 15;
      while (value < line_end && isspace((unsigned char) *value)) {
        value += 1;
      }

      char number[32];
      size_t number_len = (size_t) (line_end - value);
      if (number_len == 0 || number_len >= sizeof(number)) {
        return 0;
      }
      memcpy(number, value, number_len);
      number[number_len] = '\0';
      return (size_t) strtoull(number, NULL, 10);
    }

    cursor = line_end + 2;
  }

  return 0;
}

static bool sec4_rt_has_json_media_type(const char *value, size_t value_len) {
  size_t start = 0;
  while (start < value_len && isspace((unsigned char) value[start])) {
    start += 1;
  }
  if (start >= value_len) {
    return false;
  }

  size_t end = start;
  while (end < value_len && value[end] != ';' && !isspace((unsigned char) value[end])) {
    end += 1;
  }
  if (end <= start) {
    return false;
  }

  size_t token_len = end - start;
  const char *token = value + start;
  if (token_len == 16 && strncasecmp(token, "application/json", 16) == 0) {
    return true;
  }

  if (token_len > 5 && strncasecmp(token + token_len - 5, "+json", 5) == 0) {
    for (size_t i = 0; i < token_len; i++) {
      if (token[i] == '/') {
        return true;
      }
    }
  }

  return false;
}

static bool sec4_rt_parse_content_type_is_json(
    const char *request,
    size_t request_len,
    bool *has_content_type
) {
  const char *cursor = request;
  const char *request_end = request + request_len;
  *has_content_type = false;

  while (cursor < request_end) {
    const char *line_end = strstr(cursor, "\r\n");
    if (line_end == NULL || line_end > request_end) {
      break;
    }
    if (line_end == cursor) {
      break;
    }

    if ((size_t) (line_end - cursor) >= 13 && strncasecmp(cursor, "Content-Type:", 13) == 0) {
      const char *value = cursor + 13;
      *has_content_type = true;
      return sec4_rt_has_json_media_type(value, (size_t) (line_end - value));
    }

    cursor = line_end + 2;
  }

  return false;
}

static sec4_rt_router_state *sec4_rt_router_slot(int64_t router) {
  for (size_t i = 0; i < SEC4_RT_MAX_ROUTERS; i++) {
    if (g_sec4_rt_routers[i].active && g_sec4_rt_routers[i].handle == router) {
      return &g_sec4_rt_routers[i];
    }
  }
  return NULL;
}

static int64_t sec4_rt_register_route(
    int64_t router,
    const char *method,
    const char *path,
    sec4_rt_handler_fn handler
) {
  sec4_rt_router_state *slot = sec4_rt_router_slot(router);
  if (slot == NULL || method == NULL || path == NULL || handler == NULL) {
    return 1;
  }

  if (slot->route_count >= SEC4_RT_MAX_ROUTES) {
    return 1;
  }

  sec4_rt_route *route = &slot->routes[slot->route_count];
  memset(route, 0, sizeof(*route));

  strncpy(route->method, method, sizeof(route->method) - 1);
  route->method[sizeof(route->method) - 1] = '\0';
  strncpy(route->path, path, sizeof(route->path) - 1);
  route->path[sizeof(route->path) - 1] = '\0';
  route->handler = handler;

  slot->route_count += 1;
  return 0;
}

static const char *sec4_rt_status_text(int64_t status) {
  switch (status) {
    case 200:
      return "OK";
    case 201:
      return "Created";
    case 204:
      return "No Content";
    case 400:
      return "Bad Request";
    case 413:
      return "Payload Too Large";
    case 415:
      return "Unsupported Media Type";
    case 404:
      return "Not Found";
    case 500:
      return "Internal Server Error";
    default:
      return "OK";
  }
}

static int sec4_rt_write_all(int socket_fd, const char *buffer, size_t size) {
  size_t written = 0;
  while (written < size) {
    ssize_t rc = send(socket_fd, buffer + written, size - written, 0);
    if (rc <= 0) {
      return -1;
    }
    written += (size_t) rc;
  }
  return 0;
}

static int sec4_rt_send_response(
    int socket_fd,
    int64_t status,
    const char *content_type,
    const char *body,
    size_t body_len
) {
  if (content_type == NULL || content_type[0] == '\0') {
    content_type = "text/plain; charset=utf-8";
  }

  char header[512];
  int header_len = snprintf(
      header,
      sizeof(header),
      "HTTP/1.1 %lld %s\r\n"
      "Content-Type: %s\r\n"
      "Content-Length: %zu\r\n"
      "Connection: close\r\n"
      "\r\n",
      (long long) status,
      sec4_rt_status_text(status),
      content_type,
      body_len
  );
  if (header_len < 0 || (size_t) header_len >= sizeof(header)) {
    return -1;
  }

  if (sec4_rt_write_all(socket_fd, header, (size_t) header_len) != 0) {
    return -1;
  }
  if (body_len > 0 && sec4_rt_write_all(socket_fd, body, body_len) != 0) {
    return -1;
  }
  return 0;
}

static int64_t sec4_rt_parse_env_i64(const char *name, int64_t fallback) {
  const char *raw = getenv(name);
  if (raw == NULL || raw[0] == '\0') {
    return fallback;
  }

  char *end = NULL;
  long long value = strtoll(raw, &end, 10);
  if (end == raw || (end != NULL && *end != '\0')) {
    return fallback;
  }
  if (value < 0) {
    return fallback;
  }
  return (int64_t) value;
}

static bool sec4_rt_oneshot_mode_enabled(void) {
  const char *mode = getenv("SEC4_RT_HTTP_SERVE_MODE");
  if (mode != NULL && strcmp(mode, "oneshot") == 0) {
    return true;
  }
  const char *flag = getenv("SEC4_RT_HTTP_SERVE_ONCE");
  return flag != NULL && strcmp(flag, "0") != 0;
}

static void sec4_rt_handle_client(int socket_fd, sec4_rt_router_state *router) {
  sec4_rt_reset_request();

  char request[SEC4_RT_REQUEST_BUFFER_BYTES];
  size_t total_bytes = 0;
  ssize_t bytes_read = recv(socket_fd, request, sizeof(request) - 1, 0);
  if (bytes_read <= 0) {
    return;
  }
  total_bytes = (size_t) bytes_read;
  request[total_bytes] = '\0';

  char *headers_end = strstr(request, "\r\n\r\n");
  if (headers_end != NULL) {
    size_t headers_len = (size_t) (headers_end - request) + 4;
    size_t content_length = sec4_rt_parse_content_length(request, headers_len);
    size_t available_body = total_bytes > headers_len ? total_bytes - headers_len : 0;

    while (content_length > available_body && total_bytes < sizeof(request) - 1) {
      size_t remaining = (sizeof(request) - 1) - total_bytes;
      ssize_t next = recv(socket_fd, request + total_bytes, remaining, 0);
      if (next <= 0) {
        break;
      }
      total_bytes += (size_t) next;
      request[total_bytes] = '\0';
      available_body = total_bytes > headers_len ? total_bytes - headers_len : 0;
    }
  }

  char method[8] = {0};
  char path[SEC4_RT_MAX_PATH_BYTES] = {0};
  if (sscanf(request, "%7s %255s", method, path) != 2) {
    const char *body = "bad request";
    (void) sec4_rt_send_response(
        socket_fd,
        400,
        "text/plain; charset=utf-8",
        body,
        strlen(body)
    );
    return;
  }

  char *query_start = strchr(path, '?');
  if (query_start != NULL) {
    *query_start = '\0';
  }

  headers_end = strstr(request, "\r\n\r\n");
  if (headers_end != NULL) {
    size_t headers_len = (size_t) (headers_end - request) + 4;
    size_t content_length = sec4_rt_parse_content_length(request, headers_len);
    bool has_content_type = false;
    bool content_type_is_json = sec4_rt_parse_content_type_is_json(
        request,
        headers_len,
        &has_content_type
    );
    size_t available_body = total_bytes > headers_len ? total_bytes - headers_len : 0;
    size_t body_cap = sizeof(g_sec4_rt_request.body) - 1;
    if (content_length > body_cap || available_body > body_cap) {
      g_sec4_rt_request.body_limit_exceeded = true;
    }
    size_t body_len = content_length > 0 ? content_length : available_body;
    if (body_len > available_body) {
      body_len = available_body;
    }
    if (body_len > body_cap) {
      body_len = body_cap;
    }
    if (body_len > 0) {
      memcpy(g_sec4_rt_request.body, request + headers_len, body_len);
      g_sec4_rt_request.body[body_len] = '\0';
      g_sec4_rt_request.body_len = body_len;
    }
    g_sec4_rt_request.has_content_type = has_content_type;
    g_sec4_rt_request.content_type_is_json = content_type_is_json;
  }

  g_sec4_rt_request.has_request = true;
  strncpy(g_sec4_rt_request.method, method, sizeof(g_sec4_rt_request.method) - 1);
  g_sec4_rt_request.method[sizeof(g_sec4_rt_request.method) - 1] = '\0';
  strncpy(g_sec4_rt_request.path, path, sizeof(g_sec4_rt_request.path) - 1);
  g_sec4_rt_request.path[sizeof(g_sec4_rt_request.path) - 1] = '\0';

  sec4_rt_route *match = NULL;
  for (size_t i = 0; i < router->route_count; i++) {
    sec4_rt_route *candidate = &router->routes[i];
    if (strcmp(candidate->method, method) == 0 && strcmp(candidate->path, path) == 0) {
      match = candidate;
      break;
    }
  }

  if (match == NULL) {
    const char *body = "not found";
    (void) sec4_rt_send_response(
        socket_fd,
        404,
        "text/plain; charset=utf-8",
        body,
        strlen(body)
    );
    return;
  }

  sec4_rt_reset_response();
  (void) match->handler();

  if (!g_sec4_rt_response.active) {
    const char *body = "";
    (void) sec4_rt_send_response(socket_fd, 204, "text/plain; charset=utf-8", body, 0);
    return;
  }

  (void) sec4_rt_send_response(
      socket_fd,
      g_sec4_rt_response.status,
      g_sec4_rt_response.content_type,
      g_sec4_rt_response.body,
      g_sec4_rt_response.body_len
  );
}

int64_t sec4_rt_identity_i64(int64_t value) {
  return value;
}

bool sec4_rt_identity_bool(bool value) {
  return value;
}

int64_t sec4_rt_time_now(void) {
  return 0;
}

void sec4_rt_log_any() {
}

int64_t sec4_rt_log_event() {
  return 0;
}

int64_t sec4_rt_log_field() {
  return 0;
}

int64_t sec4_rt_log_obj() {
  return 0;
}

int64_t sec4_rt_log_str() {
  return 0;
}

int64_t sec4_rt_log_i64() {
  return 0;
}

int64_t sec4_rt_log_bool() {
  return 0;
}

int64_t sec4_rt_log_redacted() {
  return 0;
}

int64_t sec4_rt_log_attr_redacted() {
  return 0;
}

int64_t sec4_rt_log_with_attr() {
  return 0;
}

int64_t sec4_rt_log_with_http() {
  return 0;
}

int64_t sec4_rt_log_with_error() {
  return 0;
}

int64_t sec4_rt_req_json(int64_t schema) {
  (void) schema;
  g_sec4_rt_request.json_checked = true;

  if (!g_sec4_rt_request.has_request || g_sec4_rt_request.body_len == 0) {
    g_sec4_rt_request.json_valid = false;
    sec4_rt_store_std_error_response(
        400,
        "JSON.BODY_REQUIRED",
        "validation",
        "JSON body required"
    );
    return 1;
  }

  if (g_sec4_rt_request.body_limit_exceeded) {
    g_sec4_rt_request.json_valid = false;
    sec4_rt_store_std_error_response(
        413,
        "LIMIT.BODY_BYTES",
        "resource_limit",
        "request body exceeds runtime limit"
    );
    return 1;
  }

  if (!g_sec4_rt_request.has_content_type || !g_sec4_rt_request.content_type_is_json) {
    g_sec4_rt_request.json_valid = false;
    sec4_rt_store_std_error_response(
        415,
        "HTTP.CONTENT_TYPE_INVALID",
        "validation",
        "content-type must be application/json"
    );
    return 1;
  }

  if (!sec4_rt_is_likely_json(g_sec4_rt_request.body, g_sec4_rt_request.body_len)) {
    g_sec4_rt_request.json_valid = false;
    sec4_rt_store_std_error_response(
        400,
        "JSON.INVALID_BODY",
        "validation",
        "invalid json body"
    );
    return 1;
  }

  g_sec4_rt_request.json_valid = true;
  return 0;
}

int64_t sec4_rt_json_decode(int64_t ctx, int64_t schema, int64_t raw) {
  (void) ctx;
  (void) schema;
  (void) raw;
  return 0;
}

int64_t sec4_rt_json_encode(int64_t schema, int64_t value) {
  (void) schema;
  return value;
}

int64_t sec4_rt_req_body() {
  return 0;
}

int64_t sec4_rt_req_query() {
  return 0;
}

int64_t sec4_rt_req_path_param() {
  return 0;
}

int64_t sec4_rt_req_header() {
  return 0;
}

int64_t sec4_rt_res_json(int64_t schema, int64_t value) {
  (void) schema;
  (void) value;
  if (g_sec4_rt_request.json_checked && !g_sec4_rt_request.json_valid) {
    return 1;
  }
  sec4_rt_store_response(
      200,
      "application/json; charset=utf-8",
      "{\"ok\":true}"
  );
  return 0;
}

int64_t sec4_rt_res_ok(int64_t status, int64_t schema, int64_t value) {
  (void) schema;
  (void) value;
  if (g_sec4_rt_request.json_checked && !g_sec4_rt_request.json_valid) {
    return 1;
  }
  sec4_rt_store_response(
      status > 0 ? status : 201,
      "application/json; charset=utf-8",
      "{\"ok\":true}"
  );
  return 0;
}

int64_t sec4_rt_res_ok_meta(int64_t status, int64_t schema, int64_t value, int64_t meta) {
  (void) schema;
  (void) value;
  (void) meta;
  if (g_sec4_rt_request.json_checked && !g_sec4_rt_request.json_valid) {
    return 1;
  }
  sec4_rt_store_response(
      status > 0 ? status : 201,
      "application/json; charset=utf-8",
      "{\"ok\":true,\"meta\":{}}"
  );
  return 0;
}

int64_t sec4_rt_res_html() {
  sec4_rt_store_response(200, "text/html; charset=utf-8", "<html></html>");
  return 0;
}

int64_t sec4_rt_res_text(int64_t status, const char *body) {
  sec4_rt_store_response(status, "text/plain; charset=utf-8", body);
  return 0;
}

int64_t sec4_rt_set_header() {
  return 0;
}

int64_t sec4_rt_cookie_build() {
  return 0;
}

int64_t sec4_rt_set_cookie() {
  return 0;
}

int64_t sec4_rt_sql_q() {
  return 0;
}

int64_t sec4_rt_db_exec() {
  return 0;
}

int64_t sec4_rt_db_tx() {
  return 0;
}

int64_t sec4_rt_db_exec_tx() {
  return 0;
}

int64_t sec4_rt_db_query_one() {
  return 0;
}

int64_t sec4_rt_fs_read() {
  return 0;
}

int64_t sec4_rt_fs_write() {
  return 0;
}

int64_t sec4_rt_http_get() {
  return 0;
}

int64_t sec4_rt_http_get_internal() {
  return 0;
}

int64_t sec4_rt_secret_get() {
  return 0;
}

int64_t sec4_rt_secret_redact() {
  return 0;
}

int64_t sec4_rt_secret_reveal() {
  return 0;
}

bool sec4_rt_crypto_ct_eq() {
  return false;
}

int64_t sec4_rt_validate_header_value() {
  return 0;
}

int64_t sec4_rt_validate_email() {
  return 0;
}

int64_t sec4_rt_validate_uuid() {
  return 0;
}

int64_t sec4_rt_validate_int64() {
  return 0;
}

int64_t sec4_rt_validate_non_empty() {
  return 0;
}

int64_t sec4_rt_sanitize_html() {
  return 0;
}

int64_t sec4_rt_url_public() {
  return 0;
}

int64_t sec4_rt_url_internal() {
  return 0;
}

int64_t sec4_rt_path_under() {
  return 0;
}

int64_t sec4_rt_path_base() {
  return 0;
}

int64_t sec4_rt_headers_name() {
  return 0;
}

int64_t sec4_rt_headers_value() {
  return 0;
}

int64_t sec4_rt_http_router(void) {
  for (size_t i = 0; i < SEC4_RT_MAX_ROUTERS; i++) {
    if (!g_sec4_rt_routers[i].active) {
      memset(&g_sec4_rt_routers[i], 0, sizeof(g_sec4_rt_routers[i]));
      g_sec4_rt_routers[i].active = true;
      g_sec4_rt_routers[i].handle = g_sec4_rt_next_router_handle++;
      return g_sec4_rt_routers[i].handle;
    }
  }
  return 0;
}

int64_t sec4_rt_http_route_get(
    int64_t router,
    const char *path,
    int64_t (*handler)(void)
) {
  return sec4_rt_register_route(router, "GET", path, handler);
}

int64_t sec4_rt_http_route_post(
    int64_t router,
    const char *path,
    int64_t (*handler)(void)
) {
  return sec4_rt_register_route(router, "POST", path, handler);
}

int64_t sec4_rt_http_serve(int64_t port, int64_t router) {
  sec4_rt_router_state *slot = sec4_rt_router_slot(router);
  if (slot == NULL || port <= 0 || port > 65535) {
    return 1;
  }

  int server_fd = socket(AF_INET, SOCK_STREAM, 0);
  if (server_fd < 0) {
    return 1;
  }

  int opt = 1;
  (void) setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

  struct sockaddr_in addr;
  memset(&addr, 0, sizeof(addr));
  addr.sin_family = AF_INET;
  addr.sin_addr.s_addr = htonl(INADDR_ANY);
  addr.sin_port = htons((uint16_t) port);

  if (bind(server_fd, (struct sockaddr *) &addr, sizeof(addr)) != 0) {
    close(server_fd);
    return 1;
  }

  if (listen(server_fd, 16) != 0) {
    close(server_fd);
    return 1;
  }

  bool oneshot = sec4_rt_oneshot_mode_enabled();
  int64_t timeout_ms = sec4_rt_parse_env_i64(
      "SEC4_RT_HTTP_SERVE_TIMEOUT_MS",
      SEC4_RT_DEFAULT_ONESHOT_TIMEOUT_MS
  );
  if (timeout_ms < 0) {
    timeout_ms = SEC4_RT_DEFAULT_ONESHOT_TIMEOUT_MS;
  }

  for (;;) {
    if (oneshot) {
      fd_set fds;
      FD_ZERO(&fds);
      FD_SET(server_fd, &fds);

      struct timeval timeout;
      timeout.tv_sec = (time_t) (timeout_ms / 1000);
      timeout.tv_usec = (suseconds_t) ((timeout_ms % 1000) * 1000);

      int select_rc = select(server_fd + 1, &fds, NULL, NULL, &timeout);
      if (select_rc <= 0) {
        break;
      }
    }

    int client_fd = accept(server_fd, NULL, NULL);
    if (client_fd < 0) {
      if (errno == EINTR) {
        continue;
      }
      break;
    }

    sec4_rt_handle_client(client_fd, slot);
    close(client_fd);

    if (oneshot) {
      break;
    }
  }

  close(server_fd);
  return 0;
}

int64_t sec4_rt_with_cors(int64_t router, int64_t cfg) {
  (void) cfg;
  return router;
}

int64_t sec4_rt_with_security_headers(int64_t router, int64_t cfg) {
  (void) cfg;
  return router;
}

int64_t sec4_rt_with_csrf(int64_t router, int64_t cfg) {
  (void) cfg;
  return router;
}

int64_t sec4_rt_with_auth(int64_t router, int64_t cfg) {
  (void) cfg;
  return router;
}

int64_t sec4_rt_sec_default_headers() {
  return 0;
}

int64_t sec4_rt_sec_csp() {
  return 0;
}

int64_t sec4_rt_sec_csp_add() {
  return 0;
}

int64_t sec4_rt_cors_from_policy() {
  return 0;
}

int64_t sec4_rt_cors_origin() {
  return 0;
}

int64_t sec4_rt_csrf_from_policy() {
  return 0;
}

int64_t sec4_rt_csrf_issue_token() {
  return 0;
}

int64_t sec4_rt_auth_from_policy() {
  return 0;
}

int64_t sec4_rt_auth_require() {
  return 0;
}

int64_t sec4_rt_auth_require_role() {
  return 0;
}

int64_t sec4_rt_err_validation() {
  return 0;
}

int64_t sec4_rt_err_auth() {
  return 0;
}

int64_t sec4_rt_err_not_found() {
  return 0;
}

int64_t sec4_rt_err_conflict() {
  return 0;
}

int64_t sec4_rt_err_rate_limit() {
  return 0;
}

int64_t sec4_rt_err_internal() {
  return 0;
}

int64_t sec4_rt_err_with_path() {
  return 0;
}

int64_t sec4_rt_err_with_detail() {
  return 0;
}

int64_t sec4_rt_err_with_limit() {
  return 0;
}

int64_t sec4_rt_err_with_dependency() {
  return 0;
}

int64_t sec4_rt_err_with_cause() {
  return 0;
}
