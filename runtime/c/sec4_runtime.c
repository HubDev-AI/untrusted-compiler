#include "sec4_runtime.h"

#include <arpa/inet.h>
#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <netdb.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <stdio.h>
#include <sys/select.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

#ifdef SEC4_RT_ENABLE_OPENSSL_TLS
#include <openssl/ssl.h>
#endif

#define SEC4_RT_MAX_ROUTERS 16
#define SEC4_RT_MAX_ROUTES 64
#define SEC4_RT_MAX_PATH_BYTES 256
#define SEC4_RT_MAX_TRACKED_VALUES 256
#define SEC4_RT_MAX_TRACKED_VALUE_BYTES 1024
#define SEC4_RT_MAX_FS_PATH_BYTES 4096
#define SEC4_RT_MAX_RESPONSE_BYTES 4096
#define SEC4_RT_MAX_RESPONSE_EXTRA_HEADERS_BYTES 2048
#define SEC4_RT_MAX_REQUEST_BODY_BYTES 4096
#define SEC4_RT_DEFAULT_JSON_MAX_BYTES 2048
#define SEC4_RT_DEFAULT_JSON_MAX_DEPTH 16
#define SEC4_RT_REQUEST_BUFFER_BYTES 8192
#define SEC4_RT_MAX_OUTBOUND_HTTP_URL_BYTES 2048
#define SEC4_RT_MAX_OUTBOUND_HTTP_REQUEST_BYTES 2048
#define SEC4_RT_MAX_OUTBOUND_HTTP_HOST_BYTES 256
#define SEC4_RT_MAX_OUTBOUND_HTTP_HEADER_BYTES 8192
#define SEC4_RT_DEFAULT_NET_TIMEOUT_MS 2000
#define SEC4_RT_DEFAULT_NET_MAX_BODY_BYTES 1048576
#define SEC4_RT_DEFAULT_ONESHOT_TIMEOUT_MS 200
#define SEC4_RT_MAX_DB_QUERIES 256
#define SEC4_RT_MAX_DB_TXS 256
#define SEC4_RT_MAX_LOG_VALUES 256
#define SEC4_RT_MAX_LOG_EVENTS 128
#define SEC4_RT_MAX_ERROR_STATES 128
#define SEC4_RT_MAX_LOG_EVENT_NAME_BYTES 128
#define SEC4_RT_MAX_LOG_JSON_VALUE_BYTES 256
#define SEC4_RT_MAX_LOG_ATTRS_BYTES 768
#define SEC4_RT_MAX_LOG_LINE_BYTES 2048
#define SEC4_RT_MAX_DB_RECORD_HEX_BYTES (SEC4_RT_MAX_TRACKED_VALUE_BYTES * 2 + 1)
#define SEC4_RT_MAX_DB_RECORD_LINE_BYTES (SEC4_RT_MAX_DB_RECORD_HEX_BYTES + 256)
#define SEC4_RT_POLICY_CORS_HANDLE INT64_C(0x6EC4001)
#define SEC4_RT_POLICY_SECURITY_HEADERS_HANDLE INT64_C(0x6EC4002)
#define SEC4_RT_POLICY_CSRF_HANDLE INT64_C(0x6EC4003)
#define SEC4_RT_POLICY_AUTH_HANDLE INT64_C(0x6EC4004)

typedef int64_t (*sec4_rt_handler_fn)(void);

typedef struct {
  char method[8];
  char path[SEC4_RT_MAX_PATH_BYTES];
  sec4_rt_handler_fn handler;
} sec4_rt_route;

typedef struct {
  bool active;
  int64_t handle;
  bool cors_enabled;
  char cors_allow_origin[256];
  bool cors_allow_credentials;
  bool cors_require_vary_origin;
  char cors_allow_methods[128];
  char cors_allow_headers[128];
  int64_t cors_max_age_seconds;
  bool security_headers_enabled;
  bool security_x_content_type_options;
  char security_x_frame_options[16];
  char security_referrer_policy[128];
  bool csrf_enabled;
  char csrf_mode[32];
  char csrf_protected_methods[64];
  bool auth_enabled;
  char auth_mode[16];
  size_t route_count;
  sec4_rt_route routes[SEC4_RT_MAX_ROUTES];
} sec4_rt_router_state;

typedef struct {
  bool active;
  int64_t status;
  char content_type[64];
  char body[SEC4_RT_MAX_RESPONSE_BYTES];
  size_t body_len;
  char extra_headers[SEC4_RT_MAX_RESPONSE_EXTRA_HEADERS_BYTES];
  size_t extra_headers_len;
} sec4_rt_response_state;

typedef struct {
  bool has_request;
  char trace_id[32];
  char method[8];
  char path[SEC4_RT_MAX_PATH_BYTES];
  char route_path[SEC4_RT_MAX_PATH_BYTES];
  char matched_route_pattern[SEC4_RT_MAX_PATH_BYTES];
  char raw_headers[SEC4_RT_REQUEST_BUFFER_BYTES];
  size_t raw_headers_len;
  char body[SEC4_RT_MAX_REQUEST_BODY_BYTES];
  size_t body_len;
  bool body_limit_exceeded;
  bool has_content_type;
  bool content_type_is_json;
  bool json_checked;
  bool json_valid;
  int64_t json_schema_handle;
} sec4_rt_request_state;

typedef struct {
  bool active;
  int64_t handle;
  char value[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
} sec4_rt_tracked_value;

typedef struct {
  bool active;
  int64_t query_handle;
  int64_t template_handle;
  int64_t params;
} sec4_rt_db_query_state;

typedef struct {
  bool active;
  int64_t tx_handle;
  int64_t db_handle;
} sec4_rt_db_tx_state;

typedef enum {
  SEC4_RT_FS_RESULT_OK = 0,
  SEC4_RT_FS_RESULT_INVALID = 1,
  SEC4_RT_FS_RESULT_OUT_OF_BASE = 2,
  SEC4_RT_FS_RESULT_IO = 3
} sec4_rt_fs_result;

typedef enum {
  SEC4_RT_DB_RESULT_OK = 0,
  SEC4_RT_DB_RESULT_INVALID = 1,
  SEC4_RT_DB_RESULT_NOT_FOUND = 2,
  SEC4_RT_DB_RESULT_IO = 3
} sec4_rt_db_result;

typedef enum {
  SEC4_RT_NET_SCOPE_PUBLIC = 0,
  SEC4_RT_NET_SCOPE_INTERNAL = 1
} sec4_rt_net_scope;

typedef struct {
  bool loaded;
  bool enabled;
  char allow_origin[256];
  bool allow_credentials;
  bool require_vary_origin;
} sec4_rt_cors_policy_state;

typedef struct {
  bool loaded;
  bool enabled;
  bool x_content_type_options;
  char x_frame_options[16];
  char referrer_policy[128];
} sec4_rt_security_headers_policy_state;

typedef struct {
  bool loaded;
  bool enabled;
  char mode[32];
  char protected_methods[64];
} sec4_rt_csrf_policy_state;

typedef struct {
  bool loaded;
  bool enabled;
  char mode[16];
} sec4_rt_auth_policy_state;

typedef struct {
  bool active;
  int64_t handle;
  char json[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES];
} sec4_rt_log_value_state;

typedef struct {
  bool active;
  int64_t handle;
  char event_name[SEC4_RT_MAX_LOG_EVENT_NAME_BYTES];
  char level[8];
  char attrs_json[SEC4_RT_MAX_LOG_ATTRS_BYTES];
  bool has_http;
  char http_method[8];
  char http_path[SEC4_RT_MAX_PATH_BYTES];
  int64_t http_status;
  int64_t http_latency_ms;
  bool has_error;
  int64_t error_handle;
} sec4_rt_log_event_state;

typedef struct {
  bool active;
  int64_t handle;
  int64_t status;
  char code[64];
  char kind[32];
  char message[256];
  char path[SEC4_RT_MAX_PATH_BYTES];
  bool has_details;
  char details_json[SEC4_RT_MAX_LOG_ATTRS_BYTES];
  bool has_limit;
  char limit_name[96];
  int64_t limit_value;
  int64_t limit_max;
  bool has_dependency;
  char dependency_service[96];
  char dependency_operation[96];
  bool dependency_retryable;
  bool has_cause;
  int64_t cause_handle;
} sec4_rt_error_state;

static sec4_rt_router_state g_sec4_rt_routers[SEC4_RT_MAX_ROUTERS];
static int64_t g_sec4_rt_next_router_handle = 1;
static sec4_rt_response_state g_sec4_rt_response;
static sec4_rt_request_state g_sec4_rt_request;
static sec4_rt_tracked_value g_sec4_rt_tracked_values[SEC4_RT_MAX_TRACKED_VALUES];
static sec4_rt_db_query_state g_sec4_rt_db_queries[SEC4_RT_MAX_DB_QUERIES];
static sec4_rt_db_tx_state g_sec4_rt_db_txs[SEC4_RT_MAX_DB_TXS];
static uint64_t g_sec4_rt_next_trace_id = 1;
static int64_t g_sec4_rt_last_log_handle = 0;
static sec4_rt_cors_policy_state g_sec4_rt_cors_policy;
static sec4_rt_security_headers_policy_state g_sec4_rt_security_headers_policy;
static sec4_rt_csrf_policy_state g_sec4_rt_csrf_policy;
static sec4_rt_auth_policy_state g_sec4_rt_auth_policy;
static sec4_rt_log_value_state g_sec4_rt_log_values[SEC4_RT_MAX_LOG_VALUES];
static sec4_rt_log_event_state g_sec4_rt_log_events[SEC4_RT_MAX_LOG_EVENTS];
static sec4_rt_error_state g_sec4_rt_error_states[SEC4_RT_MAX_ERROR_STATES];

static const char *sec4_rt_current_trace_id(void);
static void sec4_rt_assign_trace_id(void);
static bool sec4_rt_parse_header_value(
    const char *request,
    size_t request_len,
    const char *name,
    char *value,
    size_t value_size
);
static int64_t sec4_rt_parse_env_i64(const char *name, int64_t fallback);
static bool sec4_rt_is_public_url_valid(const char *url);
static bool sec4_rt_is_internal_url_valid(const char *url);
static bool sec4_rt_ipv4_octets_are_private(const uint8_t octets[4]);
static bool sec4_rt_env_flag_enabled_default(const char *name, bool fallback);
static bool sec4_rt_env_flag_enabled(const char *name);
static void sec4_rt_read_env_string(
    const char *name,
    const char *fallback,
    char *out,
    size_t out_size
);
static bool sec4_rt_csv_copy_first_token(
    const char *csv,
    char *out,
    size_t out_size
);
static bool sec4_rt_is_csrf_protected_method(sec4_rt_router_state *router, const char *method);
static size_t sec4_rt_json_escape(
    const char *input,
    char *output,
    size_t output_size
);
static bool sec4_rt_log_json_for_handle(
    int64_t handle,
    char *buffer,
    size_t buffer_size
);
static sec4_rt_log_event_state *sec4_rt_log_event_state_for_handle(int64_t handle);
static int64_t sec4_rt_log_register_event(const char *event_name);
static int64_t sec4_rt_log_register_value(const char *json_value, uint64_t salt);
static void sec4_rt_log_emit_json(int64_t event_handle);
static sec4_rt_error_state *sec4_rt_error_state_for_handle(int64_t handle);
static void sec4_rt_error_store_state(const sec4_rt_error_state *state);
static void sec4_rt_error_load_state(int64_t handle, sec4_rt_error_state *out);
static void sec4_rt_error_render_json(
    const sec4_rt_error_state *state,
    char *buffer,
    size_t buffer_size
);
static void sec4_rt_error_store_response(const sec4_rt_error_state *state);
static void sec4_rt_error_append_detail_entry(
    sec4_rt_error_state *state,
    const char *key,
    int64_t value
);
static int sec4_rt_write_all(int socket_fd, const char *buffer, size_t size);
static bool sec4_rt_fs_mkdirs(const char *path);

static void sec4_rt_reset_response(void) {
  g_sec4_rt_response.active = false;
  g_sec4_rt_response.status = 204;
  g_sec4_rt_response.content_type[0] = '\0';
  g_sec4_rt_response.body[0] = '\0';
  g_sec4_rt_response.body_len = 0;
  g_sec4_rt_response.extra_headers[0] = '\0';
  g_sec4_rt_response.extra_headers_len = 0;
}

static void sec4_rt_store_response(
    int64_t status,
    const char *content_type,
    const char *body
) {
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
  int64_t now_ms = sec4_rt_time_now();
  char payload[768];
  int written = snprintf(
      payload,
      sizeof(payload),
      "{\"error\":{\"code\":\"%s\",\"kind\":\"%s\",\"message\":\"%s\",\"status\":%lld,\"traceId\":\"%s\",\"timeMs\":%lld}}",
      code != NULL ? code : "INTERNAL.ERROR",
      kind != NULL ? kind : "internal",
      message != NULL ? message : "internal error",
      (long long) status,
      sec4_rt_current_trace_id(),
      (long long) now_ms
  );

  if (written <= 0 || (size_t) written >= sizeof(payload)) {
    sec4_rt_store_response(
        status,
        "application/json; charset=utf-8",
        "{\"error\":{\"code\":\"INTERNAL.ERROR\",\"kind\":\"internal\",\"message\":\"error\",\"status\":500,\"traceId\":\"rt-0\",\"timeMs\":1}}"
    );
    return;
  }

  sec4_rt_store_response(status, "application/json; charset=utf-8", payload);
}

static void sec4_rt_store_std_success_response(
    int64_t status,
    const char *data_json,
    const char *meta_json
) {
  int64_t now_ms = sec4_rt_time_now();
  char payload[768];
  if (data_json == NULL || data_json[0] == '\0') {
    data_json = "{}";
  }

  int written = 0;
  if (meta_json != NULL) {
    if (meta_json[0] == '\0') {
      meta_json = "{}";
    }
    written = snprintf(
        payload,
        sizeof(payload),
        "{\"ok\":true,\"status\":%lld,\"traceId\":\"%s\",\"timeMs\":%lld,\"data\":%s,\"meta\":%s}",
        (long long) status,
        sec4_rt_current_trace_id(),
        (long long) now_ms,
        data_json,
        meta_json
    );
  } else {
    written = snprintf(
        payload,
        sizeof(payload),
        "{\"ok\":true,\"status\":%lld,\"traceId\":\"%s\",\"timeMs\":%lld,\"data\":%s}",
        (long long) status,
        sec4_rt_current_trace_id(),
        (long long) now_ms,
        data_json
    );
  }

  if (written <= 0 || (size_t) written >= sizeof(payload)) {
    sec4_rt_store_response(status, "application/json; charset=utf-8", "{\"ok\":true}");
    return;
  }

  sec4_rt_store_response(status, "application/json; charset=utf-8", payload);
}

static void sec4_rt_reset_request(void) {
  memset(&g_sec4_rt_request, 0, sizeof(g_sec4_rt_request));
  memset(g_sec4_rt_tracked_values, 0, sizeof(g_sec4_rt_tracked_values));
}

static const char *sec4_rt_current_trace_id(void) {
  if (g_sec4_rt_request.trace_id[0] == '\0') {
    return "rt-0";
  }
  return g_sec4_rt_request.trace_id;
}

static void sec4_rt_assign_trace_id(void) {
  unsigned long long trace = (unsigned long long) g_sec4_rt_next_trace_id;
  g_sec4_rt_next_trace_id += 1;
  if (g_sec4_rt_next_trace_id == 0) {
    g_sec4_rt_next_trace_id = 1;
  }
  (void) snprintf(g_sec4_rt_request.trace_id, sizeof(g_sec4_rt_request.trace_id), "rt-%llu", trace);
}

static int64_t sec4_rt_hash_token(uint64_t token, uint64_t salt) {
  uint64_t value = token ^ salt;
  value ^= value >> 33;
  value *= UINT64_C(0xff51afd7ed558ccd);
  value ^= value >> 33;
  value *= UINT64_C(0xc4ceb9fe1a85ec53);
  value ^= value >> 33;
  value &= UINT64_C(0x7fffffffffffffff);
  if (value == 0) {
    value = salt & UINT64_C(0x7fffffffffffffff);
    if (value == 0) {
      value = UINT64_C(1);
    }
  }
  return (int64_t) value;
}

static int64_t sec4_rt_gate_handle_from_input(int64_t input, uint64_t salt) {
  if (input == 0) {
    return 0;
  }
  return sec4_rt_hash_token((uint64_t) input, salt);
}

static int64_t sec4_rt_gate_handle_from_string(const char *input, uint64_t salt) {
  if (input == NULL || input[0] == '\0') {
    return 0;
  }

  uint64_t token = UINT64_C(1469598103934665603);
  while (*input != '\0') {
    token ^= (uint64_t) (unsigned char) *input;
    token *= UINT64_C(1099511628211);
    input += 1;
  }
  return sec4_rt_hash_token(token, salt);
}

static int64_t sec4_rt_track_sized_value(
    const char *value,
    size_t value_len,
    uint64_t salt
) {
  if (value == NULL || value_len >= SEC4_RT_MAX_TRACKED_VALUE_BYTES) {
    return 0;
  }

  uint64_t token = UINT64_C(1469598103934665603);
  for (size_t i = 0; i < value_len; i++) {
    token ^= (uint64_t) (unsigned char) value[i];
    token *= UINT64_C(1099511628211);
  }
  token ^= (uint64_t) value_len;

  int64_t handle = sec4_rt_hash_token(token, salt);
  if (handle == 0) {
    return 0;
  }

  size_t first_free = SEC4_RT_MAX_TRACKED_VALUES;
  for (size_t i = 0; i < SEC4_RT_MAX_TRACKED_VALUES; i++) {
    sec4_rt_tracked_value *slot = &g_sec4_rt_tracked_values[i];
    if (!slot->active) {
      if (first_free == SEC4_RT_MAX_TRACKED_VALUES) {
        first_free = i;
      }
      continue;
    }

    if (slot->handle == handle
        && strlen(slot->value) == value_len
        && memcmp(slot->value, value, value_len) == 0) {
      return handle;
    }
  }

  size_t target = first_free == SEC4_RT_MAX_TRACKED_VALUES
      ? (size_t) (((uint64_t) handle) % SEC4_RT_MAX_TRACKED_VALUES)
      : first_free;
  sec4_rt_tracked_value *slot = &g_sec4_rt_tracked_values[target];
  slot->active = true;
  slot->handle = handle;
  if (value_len > 0) {
    memcpy(slot->value, value, value_len);
  }
  slot->value[value_len] = '\0';
  return handle;
}

static int64_t sec4_rt_track_string_value(const char *value, uint64_t salt) {
  if (value == NULL || value[0] == '\0') {
    return 0;
  }

  return sec4_rt_track_sized_value(value, strlen(value), salt);
}

static const char *sec4_rt_lookup_tracked_value(int64_t handle) {
  if (handle == 0) {
    return NULL;
  }
  for (size_t i = 0; i < SEC4_RT_MAX_TRACKED_VALUES; i++) {
    const sec4_rt_tracked_value *slot = &g_sec4_rt_tracked_values[i];
    if (slot->active && slot->handle == handle) {
      return slot->value;
    }
  }
  return NULL;
}

static bool sec4_rt_constant_time_bytes_eq(
    const unsigned char *left,
    size_t left_len,
    const unsigned char *right,
    size_t right_len
) {
  size_t max_len = left_len > right_len ? left_len : right_len;
  unsigned char diff = (unsigned char) (left_len ^ right_len);
  for (size_t i = 0; i < max_len; i++) {
    unsigned char left_byte = i < left_len ? left[i] : 0;
    unsigned char right_byte = i < right_len ? right[i] : 0;
    diff |= (unsigned char) (left_byte ^ right_byte);
  }
  return diff == 0;
}

static int64_t sec4_rt_nonzero_constant_handle(uint64_t salt) {
  return sec4_rt_hash_token(UINT64_C(0x9e3779b97f4a7c15) ^ salt, salt);
}

static int64_t sec4_rt_nonzero_handle_from_string(const char *input, uint64_t salt) {
  int64_t handle = sec4_rt_track_string_value(input, salt);
  if (handle != 0) {
    return handle;
  }
  return sec4_rt_nonzero_constant_handle(salt);
}

static int64_t sec4_rt_handle_from_two(int64_t a, int64_t b, uint64_t salt) {
  uint64_t token = ((uint64_t) a) ^ ((((uint64_t) b) << 1) | (((uint64_t) b) >> 63));
  token ^= UINT64_C(0x94d049bb133111eb);
  return sec4_rt_hash_token(token, salt);
}

static int64_t sec4_rt_handle_from_three(int64_t a, int64_t b, int64_t c, uint64_t salt) {
  int64_t left = sec4_rt_handle_from_two(a, b, salt ^ UINT64_C(0x51a7d8a4));
  return sec4_rt_handle_from_two(left, c, salt);
}

static bool sec4_rt_extract_request_header(
    const char *name,
    char *value,
    size_t value_size
) {
  if (!g_sec4_rt_request.has_request || g_sec4_rt_request.raw_headers_len == 0) {
    return false;
  }
  return sec4_rt_parse_header_value(
      g_sec4_rt_request.raw_headers,
      g_sec4_rt_request.raw_headers_len,
      name,
      value,
      value_size
  );
}

static bool sec4_rt_is_valid_bearer_auth(const char *auth_header) {
  if (auth_header == NULL) {
    return false;
  }
  return strncasecmp(auth_header, "Bearer ", 7) == 0 && auth_header[7] != '\0';
}

static bool sec4_rt_bearer_token_has_role(const char *auth_header, const char *required_role) {
  if (!sec4_rt_is_valid_bearer_auth(auth_header)
      || required_role == NULL
      || required_role[0] == '\0') {
    return false;
  }

  const char *token = auth_header + 7;
  size_t role_len = strlen(required_role);
  const char *cursor = token;
  while ((cursor = strstr(cursor, required_role)) != NULL) {
    char before = cursor == token ? ' ' : cursor[-1];
    char after = cursor[role_len];
    bool before_ok =
        before == ' ' || before == ',' || before == ':' || before == '=' || before == ';';
    bool after_ok = after == '\0'
        || after == ' '
        || after == ','
        || after == ':'
        || after == '='
        || after == ';';
    if (before_ok && after_ok) {
      return true;
    }
    cursor += 1;
  }
  return false;
}

static void sec4_rt_ensure_error_response_in_request(void) {
  if (!g_sec4_rt_request.has_request || g_sec4_rt_response.active) {
    return;
  }
  sec4_rt_store_std_error_response(
      500,
      "INTERNAL.ERROR",
      "internal",
      "internal error"
  );
}

static int64_t sec4_rt_emit_error_handle(
    int64_t status,
    const char *code,
    const char *kind,
    const char *message,
    uint64_t salt
) {
  sec4_rt_error_state state;
  memset(&state, 0, sizeof(state));
  state.active = true;
  state.status = status;
  strncpy(state.code, code != NULL ? code : "INTERNAL.ERROR", sizeof(state.code) - 1);
  state.code[sizeof(state.code) - 1] = '\0';
  strncpy(state.kind, kind != NULL ? kind : "internal", sizeof(state.kind) - 1);
  state.kind[sizeof(state.kind) - 1] = '\0';
  strncpy(
      state.message,
      message != NULL ? message : "internal error",
      sizeof(state.message) - 1
  );
  state.message[sizeof(state.message) - 1] = '\0';
  int64_t code_handle = sec4_rt_nonzero_handle_from_string(code, salt ^ UINT64_C(0x1111));
  int64_t message_handle =
      sec4_rt_nonzero_handle_from_string(message, salt ^ UINT64_C(0x2222));
  state.handle = sec4_rt_handle_from_three(status, code_handle, message_handle, salt);
  sec4_rt_error_store_state(&state);
  if (g_sec4_rt_request.has_request) {
    sec4_rt_error_store_response(&state);
  }
  return state.handle;
}

static sec4_rt_error_state *sec4_rt_error_state_for_handle(int64_t handle) {
  if (handle == 0) {
    return NULL;
  }
  for (size_t i = 0; i < SEC4_RT_MAX_ERROR_STATES; i++) {
    if (g_sec4_rt_error_states[i].active && g_sec4_rt_error_states[i].handle == handle) {
      return &g_sec4_rt_error_states[i];
    }
  }
  return NULL;
}

static void sec4_rt_error_store_state(const sec4_rt_error_state *state) {
  if (state == NULL || state->handle == 0) {
    return;
  }

  sec4_rt_error_state *slot = sec4_rt_error_state_for_handle(state->handle);
  if (slot == NULL) {
    for (size_t i = 0; i < SEC4_RT_MAX_ERROR_STATES; i++) {
      if (!g_sec4_rt_error_states[i].active) {
        slot = &g_sec4_rt_error_states[i];
        break;
      }
    }
  }
  if (slot == NULL) {
    slot = &g_sec4_rt_error_states[(size_t) ((uint64_t) state->handle % SEC4_RT_MAX_ERROR_STATES)];
  }
  memcpy(slot, state, sizeof(*slot));
  slot->active = true;
}

static void sec4_rt_error_load_state(int64_t handle, sec4_rt_error_state *out) {
  if (out == NULL) {
    return;
  }
  memset(out, 0, sizeof(*out));
  out->active = true;
  out->handle = handle;
  out->status = 500;
  strncpy(out->code, "INTERNAL.ERROR", sizeof(out->code) - 1);
  out->code[sizeof(out->code) - 1] = '\0';
  strncpy(out->kind, "internal", sizeof(out->kind) - 1);
  out->kind[sizeof(out->kind) - 1] = '\0';
  strncpy(out->message, "internal error", sizeof(out->message) - 1);
  out->message[sizeof(out->message) - 1] = '\0';

  sec4_rt_error_state *existing = sec4_rt_error_state_for_handle(handle);
  if (existing != NULL) {
    memcpy(out, existing, sizeof(*out));
  }
}

static void sec4_rt_error_render_json(
    const sec4_rt_error_state *state,
    char *buffer,
    size_t buffer_size
) {
  if (buffer == NULL || buffer_size == 0) {
    return;
  }
  if (state == NULL) {
    strncpy(
        buffer,
        "{\"code\":\"INTERNAL.ERROR\",\"kind\":\"internal\",\"message\":\"internal error\",\"status\":500,\"traceId\":\"rt-0\",\"timeMs\":0}",
        buffer_size - 1
    );
    buffer[buffer_size - 1] = '\0';
    return;
  }

  char code[128];
  char kind[64];
  char message[512];
  char trace_id[64];
  sec4_rt_json_escape(state->code, code, sizeof(code));
  sec4_rt_json_escape(state->kind, kind, sizeof(kind));
  sec4_rt_json_escape(state->message, message, sizeof(message));
  sec4_rt_json_escape(sec4_rt_current_trace_id(), trace_id, sizeof(trace_id));

  char path_segment[SEC4_RT_MAX_PATH_BYTES * 2 + 24];
  path_segment[0] = '\0';
  if (state->path[0] != '\0') {
    char path[SEC4_RT_MAX_PATH_BYTES * 2];
    sec4_rt_json_escape(state->path, path, sizeof(path));
    (void) snprintf(path_segment, sizeof(path_segment), ",\"path\":\"%s\"", path);
  }

  char details_segment[SEC4_RT_MAX_LOG_ATTRS_BYTES + 16];
  details_segment[0] = '\0';
  if (state->has_details && state->details_json[0] != '\0') {
    (void) snprintf(details_segment, sizeof(details_segment), ",\"details\":%s", state->details_json);
  }

  char limit_segment[256];
  limit_segment[0] = '\0';
  if (state->has_limit) {
    char name[128];
    sec4_rt_json_escape(state->limit_name, name, sizeof(name));
    (void) snprintf(
        limit_segment,
        sizeof(limit_segment),
        ",\"limit\":{\"name\":\"%s\",\"value\":%lld,\"max\":%lld}",
        name,
        (long long) state->limit_value,
        (long long) state->limit_max
    );
  }

  char dependency_segment[320];
  dependency_segment[0] = '\0';
  if (state->has_dependency) {
    char service[128];
    char operation[128];
    sec4_rt_json_escape(state->dependency_service, service, sizeof(service));
    sec4_rt_json_escape(state->dependency_operation, operation, sizeof(operation));
    (void) snprintf(
        dependency_segment,
        sizeof(dependency_segment),
        ",\"dependency\":{\"service\":\"%s\",\"operation\":\"%s\",\"retryable\":%s}",
        service,
        operation,
        state->dependency_retryable ? "true" : "false"
    );
  }

  char cause_segment[96];
  cause_segment[0] = '\0';
  if (state->has_cause) {
    (void) snprintf(
        cause_segment,
        sizeof(cause_segment),
        ",\"cause\":{\"handle\":%lld}",
        (long long) state->cause_handle
    );
  }

  int written = snprintf(
      buffer,
      buffer_size,
      "{\"code\":\"%s\",\"kind\":\"%s\",\"message\":\"%s\",\"status\":%lld,\"traceId\":\"%s\",\"timeMs\":%lld%s%s%s%s%s}",
      code,
      kind,
      message,
      (long long) state->status,
      trace_id,
      (long long) sec4_rt_time_now(),
      path_segment,
      details_segment,
      limit_segment,
      dependency_segment,
      cause_segment
  );
  if (written <= 0 || (size_t) written >= buffer_size) {
    strncpy(
        buffer,
        "{\"code\":\"INTERNAL.ERROR\",\"kind\":\"internal\",\"message\":\"internal error\",\"status\":500,\"traceId\":\"rt-0\",\"timeMs\":1}",
        buffer_size - 1
    );
    buffer[buffer_size - 1] = '\0';
  }
}

static void sec4_rt_error_store_response(const sec4_rt_error_state *state) {
  if (state == NULL) {
    return;
  }
  char error_json[SEC4_RT_MAX_RESPONSE_BYTES];
  sec4_rt_error_render_json(state, error_json, sizeof(error_json));
  char payload[SEC4_RT_MAX_RESPONSE_BYTES];
  int written = snprintf(payload, sizeof(payload), "{\"error\":%s}", error_json);
  if (written <= 0 || (size_t) written >= sizeof(payload)) {
    sec4_rt_store_std_error_response(500, "INTERNAL.ERROR", "internal", "internal error");
    return;
  }
  sec4_rt_store_response(state->status, "application/json; charset=utf-8", payload);
}

static void sec4_rt_error_append_detail_entry(
    sec4_rt_error_state *state,
    const char *key,
    int64_t value
) {
  if (state == NULL) {
    return;
  }
  char escaped_key[96];
  sec4_rt_json_escape(
      key != NULL && key[0] != '\0' ? key : "detail",
      escaped_key,
      sizeof(escaped_key)
  );
  char value_json[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES];
  sec4_rt_log_json_for_handle(value, value_json, sizeof(value_json));
  char entry[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES + 140];
  (void) snprintf(
      entry,
      sizeof(entry),
      "{\"key\":\"%s\",\"value\":%s}",
      escaped_key,
      value_json
  );

  if (!state->has_details || state->details_json[0] == '\0') {
    int written = snprintf(
        state->details_json,
        sizeof(state->details_json),
        "[%s]",
        entry
    );
    if (written > 0 && (size_t) written < sizeof(state->details_json)) {
      state->has_details = true;
    }
    return;
  }

  size_t len = strlen(state->details_json);
  if (len < 1 || state->details_json[len - 1] != ']') {
    state->details_json[0] = '\0';
    state->has_details = false;
    return;
  }
  state->details_json[len - 1] = '\0';
  int written = snprintf(
      state->details_json + (len - 1),
      sizeof(state->details_json) - (len - 1),
      ",%s]",
      entry
  );
  if (written <= 0 || (size_t) written >= sizeof(state->details_json) - (len - 1)) {
    state->details_json[len - 1] = ']';
    state->details_json[len] = '\0';
  }
}

static bool sec4_rt_extract_query_value(
    const char *path,
    const char *name,
    char *out,
    size_t out_size
) {
  if (path == NULL || name == NULL || out == NULL || out_size == 0 || name[0] == '\0') {
    return false;
  }

  const char *query = strchr(path, '?');
  if (query == NULL) {
    return false;
  }
  query += 1;

  size_t key_len = strlen(name);
  while (*query != '\0') {
    const char *segment_start = query;
    const char *segment_end = strchr(segment_start, '&');
    if (segment_end == NULL) {
      segment_end = segment_start + strlen(segment_start);
    }

    const char *equals = memchr(segment_start, '=', (size_t) (segment_end - segment_start));
    const char *key_end = equals != NULL ? equals : segment_end;

    size_t this_key_len = (size_t) (key_end - segment_start);
    if (this_key_len == key_len && strncmp(segment_start, name, key_len) == 0) {
      const char *value_start = equals != NULL ? equals + 1 : key_end;
      size_t value_len = (size_t) (segment_end - value_start);
      if (value_len >= out_size) {
        value_len = out_size - 1;
      }
      memcpy(out, value_start, value_len);
      out[value_len] = '\0';
      return true;
    }

    if (*segment_end == '\0') {
      break;
    }
    query = segment_end + 1;
  }

  return false;
}

static bool sec4_rt_path_pattern_matches(const char *pattern, const char *path) {
  if (pattern == NULL || path == NULL) {
    return false;
  }

  const char *p = pattern;
  const char *q = path;
  while (true) {
    while (*p == '/') {
      p += 1;
    }
    while (*q == '/') {
      q += 1;
    }

    if (*p == '\0' || *q == '\0') {
      break;
    }

    const char *p_start = p;
    while (*p != '\0' && *p != '/') {
      p += 1;
    }
    const char *q_start = q;
    while (*q != '\0' && *q != '/') {
      q += 1;
    }

    size_t p_len = (size_t) (p - p_start);
    size_t q_len = (size_t) (q - q_start);
    if (p_len == 0 || q_len == 0) {
      return false;
    }

    if (p_start[0] == ':') {
      continue;
    }
    if (p_len != q_len || strncmp(p_start, q_start, p_len) != 0) {
      return false;
    }
  }

  while (*p == '/') {
    p += 1;
  }
  while (*q == '/') {
    q += 1;
  }
  return *p == '\0' && *q == '\0';
}

static bool sec4_rt_extract_path_param(
    const char *pattern,
    const char *path,
    const char *name,
    char *out,
    size_t out_size
) {
  if (out == NULL || out_size == 0) {
    return false;
  }
  out[0] = '\0';
  if (pattern == NULL || path == NULL || name == NULL || name[0] == '\0') {
    return false;
  }
  if (!sec4_rt_path_pattern_matches(pattern, path)) {
    return false;
  }

  size_t target_len = strlen(name);
  const char *p = pattern;
  const char *q = path;
  while (true) {
    while (*p == '/') {
      p += 1;
    }
    while (*q == '/') {
      q += 1;
    }
    if (*p == '\0' || *q == '\0') {
      break;
    }

    const char *p_start = p;
    while (*p != '\0' && *p != '/') {
      p += 1;
    }
    const char *q_start = q;
    while (*q != '\0' && *q != '/') {
      q += 1;
    }

    size_t p_len = (size_t) (p - p_start);
    size_t q_len = (size_t) (q - q_start);
    if (p_len > 1 && p_start[0] == ':') {
      if (p_len - 1 == target_len && strncmp(p_start + 1, name, target_len) == 0) {
        size_t copy_len = q_len;
        if (copy_len >= out_size) {
          copy_len = out_size - 1;
        }
        memcpy(out, q_start, copy_len);
        out[copy_len] = '\0';
        return copy_len > 0;
      }
    }
  }

  return false;
}

static bool sec4_rt_host_equals(const char *host, size_t host_len, const char *target) {
  size_t target_len = strlen(target);
  return host_len == target_len && strncasecmp(host, target, host_len) == 0;
}

static bool sec4_rt_host_ends_with(const char *host, size_t host_len, const char *suffix) {
  size_t suffix_len = strlen(suffix);
  if (host_len < suffix_len) {
    return false;
  }
  return strncasecmp(host + (host_len - suffix_len), suffix, suffix_len) == 0;
}

static void sec4_rt_trim_csv_token(const char **token_start, const char **token_end) {
  while (*token_start < *token_end && isspace((unsigned char) **token_start)) {
    *token_start += 1;
  }
  while (*token_end > *token_start && isspace((unsigned char) *(*token_end - 1))) {
    *token_end -= 1;
  }
}

static bool sec4_rt_csv_has_any_token(const char *csv) {
  if (csv == NULL || csv[0] == '\0') {
    return false;
  }

  const char *cursor = csv;
  while (*cursor != '\0') {
    const char *token_start = cursor;
    while (*cursor != '\0' && *cursor != ',') {
      cursor += 1;
    }
    const char *token_end = cursor;
    sec4_rt_trim_csv_token(&token_start, &token_end);
    if (token_end > token_start) {
      return true;
    }
    if (*cursor == ',') {
      cursor += 1;
    }
  }

  return false;
}

static bool sec4_rt_csv_copy_first_token(
    const char *csv,
    char *out,
    size_t out_size
) {
  if (out == NULL || out_size == 0) {
    return false;
  }
  out[0] = '\0';
  if (csv == NULL || csv[0] == '\0') {
    return false;
  }

  const char *cursor = csv;
  while (*cursor != '\0') {
    const char *token_start = cursor;
    while (*cursor != '\0' && *cursor != ',') {
      cursor += 1;
    }
    const char *token_end = cursor;
    sec4_rt_trim_csv_token(&token_start, &token_end);
    if (token_end > token_start) {
      size_t token_len = (size_t) (token_end - token_start);
      if (token_len >= out_size) {
        token_len = out_size - 1;
      }
      memcpy(out, token_start, token_len);
      out[token_len] = '\0';
      return true;
    }
    if (*cursor == ',') {
      cursor += 1;
    }
  }

  return false;
}

static bool sec4_rt_csv_contains_token_ci(
    const char *csv,
    const char *candidate,
    size_t candidate_len
) {
  if (csv == NULL || candidate == NULL || candidate_len == 0) {
    return false;
  }

  const char *cursor = csv;
  while (*cursor != '\0') {
    const char *token_start = cursor;
    while (*cursor != '\0' && *cursor != ',') {
      cursor += 1;
    }
    const char *token_end = cursor;
    sec4_rt_trim_csv_token(&token_start, &token_end);

    size_t token_len = (size_t) (token_end - token_start);
    if (token_len == candidate_len
        && strncasecmp(token_start, candidate, candidate_len) == 0) {
      return true;
    }

    if (*cursor == ',') {
      cursor += 1;
    }
  }

  return false;
}

static bool sec4_rt_csv_contains_port(const char *csv, uint16_t candidate_port) {
  if (csv == NULL || candidate_port == 0) {
    return false;
  }

  const char *cursor = csv;
  while (*cursor != '\0') {
    const char *token_start = cursor;
    while (*cursor != '\0' && *cursor != ',') {
      cursor += 1;
    }
    const char *token_end = cursor;
    sec4_rt_trim_csv_token(&token_start, &token_end);

    if (token_end > token_start) {
      unsigned long parsed_port = 0;
      bool token_valid = true;
      const char *digit = token_start;
      while (digit < token_end) {
        if (!isdigit((unsigned char) *digit)) {
          token_valid = false;
          break;
        }
        parsed_port = (parsed_port * 10UL) + (unsigned long) (*digit - '0');
        if (parsed_port > 65535UL) {
          token_valid = false;
          break;
        }
        digit += 1;
      }
      if (token_valid
          && parsed_port != 0UL
          && parsed_port == (unsigned long) candidate_port) {
        return true;
      }
    }

    if (*cursor == ',') {
      cursor += 1;
    }
  }

  return false;
}

static bool sec4_rt_parse_ipv4_octets(const char *host, size_t host_len, uint8_t out[4]) {
  if (host == NULL || out == NULL || host_len == 0) {
    return false;
  }

  int octets[4] = {0, 0, 0, 0};
  int idx = 0;
  int value = 0;
  int digits = 0;

  for (size_t i = 0; i < host_len; i++) {
    char ch = host[i];
    if (isdigit((unsigned char) ch)) {
      value = (value * 10) + (ch - '0');
      if (value > 255) {
        return false;
      }
      digits += 1;
      continue;
    }

    if (ch != '.' || digits == 0 || idx >= 3) {
      return false;
    }
    octets[idx++] = value;
    value = 0;
    digits = 0;
  }

  if (idx != 3 || digits == 0) {
    return false;
  }
  octets[3] = value;

  for (size_t i = 0; i < 4; i++) {
    out[i] = (uint8_t) octets[i];
  }
  return true;
}

static uint32_t sec4_rt_ipv4_octets_to_u32(const uint8_t octets[4]) {
  return ((uint32_t) octets[0] << 24)
      | ((uint32_t) octets[1] << 16)
      | ((uint32_t) octets[2] << 8)
      | ((uint32_t) octets[3]);
}

static bool sec4_rt_parse_ipv4_cidr_token(
    const char *token_start,
    const char *token_end,
    uint32_t *network_out,
    uint32_t *mask_out
) {
  if (token_start == NULL || token_end == NULL || token_end <= token_start
      || network_out == NULL || mask_out == NULL) {
    return false;
  }

  const char *slash = NULL;
  for (const char *cursor = token_start; cursor < token_end; cursor++) {
    if (*cursor == '/') {
      if (slash != NULL) {
        return false;
      }
      slash = cursor;
    }
  }
  if (slash == NULL || slash == token_start || slash + 1 >= token_end) {
    return false;
  }

  uint8_t cidr_octets[4];
  if (!sec4_rt_parse_ipv4_octets(token_start, (size_t) (slash - token_start), cidr_octets)) {
    return false;
  }

  uint32_t prefix = 0;
  for (const char *cursor = slash + 1; cursor < token_end; cursor++) {
    if (!isdigit((unsigned char) *cursor)) {
      return false;
    }
    prefix = (prefix * 10U) + (uint32_t) (*cursor - '0');
    if (prefix > 32U) {
      return false;
    }
  }

  uint32_t mask = 0;
  if (prefix == 0U) {
    mask = 0U;
  } else if (prefix == 32U) {
    mask = UINT32_MAX;
  } else {
    mask = UINT32_MAX << (32U - prefix);
  }

  *mask_out = mask;
  *network_out = sec4_rt_ipv4_octets_to_u32(cidr_octets) & mask;
  return true;
}

static bool sec4_rt_csv_contains_ipv4_cidr_match(
    const char *csv,
    const char *host,
    size_t host_len
) {
  if (csv == NULL || host == NULL || host_len == 0) {
    return false;
  }

  uint8_t host_octets[4];
  if (!sec4_rt_parse_ipv4_octets(host, host_len, host_octets)) {
    return false;
  }
  uint32_t host_ip = sec4_rt_ipv4_octets_to_u32(host_octets);

  const char *cursor = csv;
  while (*cursor != '\0') {
    const char *token_start = cursor;
    while (*cursor != '\0' && *cursor != ',') {
      cursor += 1;
    }
    const char *token_end = cursor;
    sec4_rt_trim_csv_token(&token_start, &token_end);

    if (token_end > token_start) {
      uint32_t network = 0;
      uint32_t mask = 0;
      if (sec4_rt_parse_ipv4_cidr_token(token_start, token_end, &network, &mask)
          && ((host_ip & mask) == network)) {
        return true;
      }
    }

    if (*cursor == ',') {
      cursor += 1;
    }
  }

  return false;
}

static bool sec4_rt_parse_ipv4_private(const char *host, size_t host_len) {
  uint8_t octets[4];
  if (!sec4_rt_parse_ipv4_octets(host, host_len, octets)) {
    return false;
  }
  return sec4_rt_ipv4_octets_are_private(octets);
}

static bool sec4_rt_ipv4_octets_are_private(const uint8_t octets[4]) {
  if (octets == NULL) {
    return false;
  }
  int a = (int) octets[0];
  int b = (int) octets[1];
  if (a == 0 || a == 10 || a == 127) {
    return true;
  }
  if (a == 169 && b == 254) {
    return true;
  }
  if (a == 192 && b == 168) {
    return true;
  }
  if (a == 172 && b >= 16 && b <= 31) {
    return true;
  }
  return false;
}

static bool sec4_rt_ipv6_addr_is_internal(const struct in6_addr *addr) {
  if (addr == NULL) {
    return false;
  }

  if (IN6_IS_ADDR_LOOPBACK(addr) || IN6_IS_ADDR_LINKLOCAL(addr)) {
    return true;
  }

  const uint8_t *bytes = addr->s6_addr;
  if ((bytes[0] & 0xFEu) == 0xFCu) {
    return true;
  }

#ifdef IN6_IS_ADDR_V4MAPPED
  if (IN6_IS_ADDR_V4MAPPED(addr)) {
    return sec4_rt_ipv4_octets_are_private(&bytes[12]);
  }
#endif

  return false;
}

static bool sec4_rt_host_resolves_to_internal(const char *host, size_t host_len) {
  if (host == NULL || host_len == 0 || host_len >= SEC4_RT_MAX_OUTBOUND_HTTP_HOST_BYTES) {
    return false;
  }

  char host_text[SEC4_RT_MAX_OUTBOUND_HTTP_HOST_BYTES];
  memcpy(host_text, host, host_len);
  host_text[host_len] = '\0';

  struct addrinfo hints;
  memset(&hints, 0, sizeof(hints));
  hints.ai_family = AF_UNSPEC;
  hints.ai_socktype = SOCK_STREAM;

  struct addrinfo *addresses = NULL;
  if (getaddrinfo(host_text, "80", &hints, &addresses) != 0 || addresses == NULL) {
    return false;
  }

  bool internal = false;
  for (struct addrinfo *current = addresses; current != NULL; current = current->ai_next) {
    if (current->ai_family == AF_INET && current->ai_addrlen >= sizeof(struct sockaddr_in)) {
      const struct sockaddr_in *addr = (const struct sockaddr_in *) current->ai_addr;
      uint32_t host_addr = ntohl(addr->sin_addr.s_addr);
      uint8_t octets[4];
      octets[0] = (uint8_t) ((host_addr >> 24) & 0xFFu);
      octets[1] = (uint8_t) ((host_addr >> 16) & 0xFFu);
      octets[2] = (uint8_t) ((host_addr >> 8) & 0xFFu);
      octets[3] = (uint8_t) (host_addr & 0xFFu);
      if (sec4_rt_ipv4_octets_are_private(octets)) {
        internal = true;
        break;
      }
    } else if (
        current->ai_family == AF_INET6 && current->ai_addrlen >= sizeof(struct sockaddr_in6)
    ) {
      const struct sockaddr_in6 *addr6 = (const struct sockaddr_in6 *) current->ai_addr;
      if (sec4_rt_ipv6_addr_is_internal(&addr6->sin6_addr)) {
        internal = true;
        break;
      }
    }
  }

  freeaddrinfo(addresses);
  return internal;
}

static bool sec4_rt_parse_url_host(
    const char *url,
    const char **host_start,
    size_t *host_len,
    bool *is_http,
    bool *is_https
) {
  if (url == NULL || host_start == NULL || host_len == NULL || is_http == NULL
      || is_https == NULL) {
    return false;
  }

  const char *scheme_sep = strstr(url, "://");
  if (scheme_sep == NULL) {
    return false;
  }

  size_t scheme_len = (size_t) (scheme_sep - url);
  *is_http = scheme_len == 4 && strncasecmp(url, "http", 4) == 0;
  *is_https = scheme_len == 5 && strncasecmp(url, "https", 5) == 0;
  if (!(*is_http || *is_https)) {
    return false;
  }

  const char *host = scheme_sep + 3;
  if (*host == '\0') {
    return false;
  }

  const char *host_end = host;
  while (*host_end != '\0' && *host_end != '/' && *host_end != '?' && *host_end != ':') {
    host_end += 1;
  }

  size_t len = (size_t) (host_end - host);
  if (len == 0) {
    return false;
  }
  if (memchr(host, '@', len) != NULL) {
    return false;
  }

  *host_start = host;
  *host_len = len;
  return true;
}

static bool sec4_rt_resolve_url_port(
    const char *host_start,
    size_t host_len,
    bool is_https,
    uint16_t *port_out
) {
  if (host_start == NULL || host_len == 0 || port_out == NULL) {
    return false;
  }

  const char *cursor = host_start + host_len;
  uint16_t resolved_port = is_https ? 443 : 80;
  if (*cursor == ':') {
    cursor += 1;
    if (!isdigit((unsigned char) *cursor)) {
      return false;
    }
    unsigned long port_value = 0;
    while (isdigit((unsigned char) *cursor)) {
      port_value = (port_value * 10UL) + (unsigned long) (*cursor - '0');
      if (port_value > 65535UL) {
        return false;
      }
      cursor += 1;
    }
    if (port_value == 0UL) {
      return false;
    }
    resolved_port = (uint16_t) port_value;
  }

  if (*cursor != '\0' && *cursor != '/' && *cursor != '?') {
    return false;
  }

  *port_out = resolved_port;
  return true;
}

static bool sec4_rt_parse_outbound_http_url(
    const char *url,
    char *host,
    size_t host_size,
    uint16_t *port,
    const char **target_start,
    size_t *target_len,
    bool *is_http,
    bool *is_https
) {
  if (url == NULL || host == NULL || host_size < 2 || port == NULL || target_start == NULL
      || target_len == NULL || is_http == NULL || is_https == NULL) {
    return false;
  }

  const char *scheme_sep = strstr(url, "://");
  if (scheme_sep == NULL) {
    return false;
  }
  size_t scheme_len = (size_t) (scheme_sep - url);
  *is_http = scheme_len == 4 && strncasecmp(url, "http", 4) == 0;
  *is_https = scheme_len == 5 && strncasecmp(url, "https", 5) == 0;
  if (!(*is_http || *is_https)) {
    return false;
  }

  const char *cursor = scheme_sep + 3;
  const char *host_begin = cursor;
  while (*cursor != '\0' && *cursor != ':' && *cursor != '/' && *cursor != '?') {
    if (*cursor == '@' || *cursor == '#' || isspace((unsigned char) *cursor)) {
      return false;
    }
    cursor += 1;
  }

  size_t host_len = (size_t) (cursor - host_begin);
  if (host_len == 0 || host_len >= host_size) {
    return false;
  }
  memcpy(host, host_begin, host_len);
  host[host_len] = '\0';

  uint16_t resolved_port = *is_https ? 443 : 80;
  if (*cursor == ':') {
    cursor += 1;
    if (!isdigit((unsigned char) *cursor)) {
      return false;
    }
    unsigned long port_value = 0;
    const char *port_start = cursor;
    while (isdigit((unsigned char) *cursor)) {
      port_value = (port_value * 10UL) + (unsigned long) (*cursor - '0');
      if (port_value > 65535UL) {
        return false;
      }
      cursor += 1;
    }
    if (cursor == port_start || port_value == 0UL) {
      return false;
    }
    resolved_port = (uint16_t) port_value;
  }

  const char *target = "/";
  size_t parsed_target_len = 1;
  if (*cursor != '\0') {
    if (*cursor != '/' && *cursor != '?') {
      return false;
    }
    const char *target_end = cursor;
    while (*target_end != '\0') {
      if (*target_end == '#' || *target_end == '\r' || *target_end == '\n') {
        return false;
      }
      target_end += 1;
    }
    parsed_target_len = (size_t) (target_end - cursor);
    if (parsed_target_len == 0) {
      parsed_target_len = 1;
    } else {
      target = cursor;
    }
  }

  *port = resolved_port;
  *target_start = target;
  *target_len = parsed_target_len;
  return true;
}

static int sec4_rt_build_outbound_http_request(
    const char *host,
    uint16_t port,
    bool is_https,
    const char *target,
    size_t target_len,
    char *request,
    size_t request_size
) {
  if (host == NULL || host[0] == '\0' || target == NULL || request == NULL || request_size < 2) {
    return -1;
  }

  char host_header[SEC4_RT_MAX_OUTBOUND_HTTP_HOST_BYTES + 8];
  int host_header_len = 0;
  bool use_default_port = (is_https && port == 443) || (!is_https && port == 80);
  if (use_default_port) {
    host_header_len = snprintf(host_header, sizeof(host_header), "%s", host);
  } else {
    host_header_len = snprintf(host_header, sizeof(host_header), "%s:%u", host, (unsigned int) port);
  }
  if (host_header_len <= 0 || (size_t) host_header_len >= sizeof(host_header)) {
    return -1;
  }

  int request_len = snprintf(
      request,
      request_size,
      "GET %.*s HTTP/1.1\r\n"
      "Host: %s\r\n"
      "Connection: close\r\n"
      "Accept: */*\r\n"
      "\r\n",
      (int) target_len,
      target,
      host_header
  );
  if (request_len <= 0 || (size_t) request_len >= request_size) {
    return -1;
  }

  return request_len;
}

static int64_t sec4_rt_outbound_http_timeout_ms(void) {
  int64_t timeout_ms = sec4_rt_parse_env_i64(
      "SEC4_RT_NET_TIMEOUT_MS",
      SEC4_RT_DEFAULT_NET_TIMEOUT_MS
  );
  if (timeout_ms <= 0) {
    timeout_ms = SEC4_RT_DEFAULT_NET_TIMEOUT_MS;
  }
  return timeout_ms;
}

static size_t sec4_rt_outbound_http_max_body_bytes(void) {
  int64_t max_body_bytes = sec4_rt_parse_env_i64(
      "SEC4_RT_NET_MAX_BODY_BYTES",
      SEC4_RT_DEFAULT_NET_MAX_BODY_BYTES
  );
  if (max_body_bytes <= 0) {
    max_body_bytes = SEC4_RT_DEFAULT_NET_MAX_BODY_BYTES;
  }
  return (size_t) max_body_bytes;
}

static bool sec4_rt_set_socket_nonblocking(int socket_fd, bool nonblocking) {
  int flags = fcntl(socket_fd, F_GETFL, 0);
  if (flags < 0) {
    return false;
  }

  int updated = nonblocking ? (flags | O_NONBLOCK) : (flags & ~O_NONBLOCK);
  return fcntl(socket_fd, F_SETFL, updated) == 0;
}

static int sec4_rt_open_outbound_tcp_socket(
    const char *host,
    uint16_t port,
    int64_t timeout_ms,
    bool *timed_out
) {
  if (timed_out != NULL) {
    *timed_out = false;
  }
  if (host == NULL || host[0] == '\0' || port == 0 || timeout_ms <= 0) {
    return -1;
  }

  char port_text[8];
  int rendered = snprintf(port_text, sizeof(port_text), "%u", (unsigned int) port);
  if (rendered <= 0 || (size_t) rendered >= sizeof(port_text)) {
    return -1;
  }

  struct addrinfo hints;
  memset(&hints, 0, sizeof(hints));
  hints.ai_socktype = SOCK_STREAM;
  hints.ai_family = AF_UNSPEC;
  hints.ai_protocol = IPPROTO_TCP;

  struct addrinfo *addresses = NULL;
  if (getaddrinfo(host, port_text, &hints, &addresses) != 0 || addresses == NULL) {
    return -1;
  }

  int socket_fd = -1;
  for (struct addrinfo *current = addresses; current != NULL; current = current->ai_next) {
    socket_fd = socket(current->ai_family, current->ai_socktype, current->ai_protocol);
    if (socket_fd < 0) {
      continue;
    }

    if (!sec4_rt_set_socket_nonblocking(socket_fd, true)) {
      close(socket_fd);
      socket_fd = -1;
      continue;
    }

    int connect_rc = connect(socket_fd, current->ai_addr, current->ai_addrlen);
    if (connect_rc != 0) {
      if (errno != EINPROGRESS && errno != EWOULDBLOCK) {
        if (timed_out != NULL && errno == ETIMEDOUT) {
          *timed_out = true;
        }
        close(socket_fd);
        socket_fd = -1;
        continue;
      }

      fd_set wfds;
      FD_ZERO(&wfds);
      FD_SET(socket_fd, &wfds);

      struct timeval connect_timeout;
      connect_timeout.tv_sec = (time_t) (timeout_ms / 1000);
      connect_timeout.tv_usec = (suseconds_t) ((timeout_ms % 1000) * 1000);

      int select_rc = select(socket_fd + 1, NULL, &wfds, NULL, &connect_timeout);
      if (select_rc == 0) {
        if (timed_out != NULL) {
          *timed_out = true;
        }
        close(socket_fd);
        socket_fd = -1;
        continue;
      }
      if (select_rc < 0) {
        close(socket_fd);
        socket_fd = -1;
        continue;
      }

      int connect_error = 0;
      socklen_t connect_error_len = sizeof(connect_error);
      if (getsockopt(socket_fd, SOL_SOCKET, SO_ERROR, &connect_error, &connect_error_len) != 0) {
        close(socket_fd);
        socket_fd = -1;
        continue;
      }
      if (connect_error != 0) {
        if (timed_out != NULL && connect_error == ETIMEDOUT) {
          *timed_out = true;
        }
        close(socket_fd);
        socket_fd = -1;
        continue;
      }
    }

    if (!sec4_rt_set_socket_nonblocking(socket_fd, false)) {
      close(socket_fd);
      socket_fd = -1;
      continue;
    }

    struct timeval timeout;
    timeout.tv_sec = (time_t) (timeout_ms / 1000);
    timeout.tv_usec = (suseconds_t) ((timeout_ms % 1000) * 1000);
    (void) setsockopt(socket_fd, SOL_SOCKET, SO_RCVTIMEO, &timeout, sizeof(timeout));
    (void) setsockopt(socket_fd, SOL_SOCKET, SO_SNDTIMEO, &timeout, sizeof(timeout));

    break;
  }

  freeaddrinfo(addresses);
  return socket_fd;
}

static int sec4_rt_extract_outbound_http_body(
    const char *response,
    size_t total,
    char *body,
    size_t body_size,
    size_t max_body_bytes,
    size_t *body_len,
    int *status_code,
    char *redirect_location,
    size_t redirect_location_size
) {
  if (response == NULL || body == NULL || body_size == 0 || body_len == NULL || max_body_bytes == 0) {
    return -1;
  }
  if (status_code != NULL) {
    *status_code = 0;
  }
  if (redirect_location != NULL && redirect_location_size > 0) {
    redirect_location[0] = '\0';
  }

  if (total < 12 || strncmp(response, "HTTP/", 5) != 0) {
    return -3;
  }

  const char *status_line_end = strstr(response, "\r\n");
  if (status_line_end == NULL) {
    return -3;
  }
  const char *status_cursor = response;
  while (status_cursor < status_line_end && *status_cursor != ' ') {
    status_cursor += 1;
  }
  while (status_cursor < status_line_end && *status_cursor == ' ') {
    status_cursor += 1;
  }
  int parsed_status = 0;
  int status_digits = 0;
  while (status_cursor < status_line_end && isdigit((unsigned char) *status_cursor)
         && status_digits < 3) {
    parsed_status = (parsed_status * 10) + (int) (*status_cursor - '0');
    status_cursor += 1;
    status_digits += 1;
  }
  if (status_digits != 3) {
    return -3;
  }
  if (status_code != NULL) {
    *status_code = parsed_status;
  }

  const char *headers_end = strstr(response, "\r\n\r\n");
  if (headers_end == NULL) {
    return -3;
  }
  if (status_line_end > headers_end) {
    return -3;
  }

  size_t header_bytes = (size_t) (headers_end - response) + 4;
  if (header_bytes > total) {
    return -3;
  }
  if (header_bytes > SEC4_RT_MAX_OUTBOUND_HTTP_HEADER_BYTES) {
    return -2;
  }

  const char *header_cursor = status_line_end + 2;
  bool transfer_chunked = false;
  bool has_content_length = false;
  size_t content_length = 0;
  while (header_cursor < headers_end) {
    const char *line_end = strstr(header_cursor, "\r\n");
    if (line_end == NULL || line_end > headers_end) {
      break;
    }
    if (line_end == header_cursor) {
      break;
    }

    if ((size_t) (line_end - header_cursor) >= 9
        && strncasecmp(header_cursor, "Location:", 9) == 0) {
      const char *value_start = header_cursor + 9;
      while (value_start < line_end && isspace((unsigned char) *value_start)) {
        value_start += 1;
      }
      const char *value_end = line_end;
      while (value_end > value_start && isspace((unsigned char) *(value_end - 1))) {
        value_end -= 1;
      }
      size_t value_len = (size_t) (value_end - value_start);
      if (redirect_location != NULL && redirect_location_size > 0 && value_len > 0
          && value_len < redirect_location_size) {
        memcpy(redirect_location, value_start, value_len);
        redirect_location[value_len] = '\0';
      }
    } else if ((size_t) (line_end - header_cursor) >= 18
               && strncasecmp(header_cursor, "Transfer-Encoding:", 18) == 0) {
      const char *value_start = header_cursor + 18;
      while (value_start < line_end) {
        while (value_start < line_end
               && (isspace((unsigned char) *value_start) || *value_start == ',')) {
          value_start += 1;
        }
        const char *token_start = value_start;
        while (value_start < line_end && *value_start != ',' && *value_start != ';') {
          value_start += 1;
        }
        const char *token_end = value_start;
        while (token_end > token_start && isspace((unsigned char) *(token_end - 1))) {
          token_end -= 1;
        }
        size_t token_len = (size_t) (token_end - token_start);
        if (token_len == 7 && strncasecmp(token_start, "chunked", 7) == 0) {
          transfer_chunked = true;
          break;
        }
        while (value_start < line_end && *value_start != ',') {
          value_start += 1;
        }
      }
    } else if ((size_t) (line_end - header_cursor) >= 15
               && strncasecmp(header_cursor, "Content-Length:", 15) == 0) {
      const char *value_start = header_cursor + 15;
      while (value_start < line_end && isspace((unsigned char) *value_start)) {
        value_start += 1;
      }
      if (value_start >= line_end) {
        return -3;
      }
      size_t parsed = 0;
      bool has_digits = false;
      while (value_start < line_end && isdigit((unsigned char) *value_start)) {
        has_digits = true;
        size_t digit = (size_t) (*value_start - '0');
        if (parsed > (SIZE_MAX - digit) / 10) {
          return -2;
        }
        parsed = (parsed * 10) + digit;
        value_start += 1;
      }
      while (value_start < line_end && isspace((unsigned char) *value_start)) {
        value_start += 1;
      }
      if (!has_digits || value_start != line_end) {
        return -3;
      }
      content_length = parsed;
      has_content_length = true;
    }

    header_cursor = line_end + 2;
  }

  size_t payload_bytes = total - header_bytes;
  const char *payload = response + header_bytes;

  if (transfer_chunked) {
    size_t cursor = 0;
    size_t decoded_len = 0;
    while (true) {
      size_t line_start = cursor;
      while (cursor + 1 < payload_bytes
             && !(payload[cursor] == '\r' && payload[cursor + 1] == '\n')) {
        cursor += 1;
      }
      if (cursor + 1 >= payload_bytes) {
        return -3;
      }
      size_t line_end = cursor;
      cursor += 2;

      size_t token_start = line_start;
      size_t token_end = line_end;
      while (token_start < token_end && isspace((unsigned char) payload[token_start])) {
        token_start += 1;
      }
      while (token_end > token_start && isspace((unsigned char) payload[token_end - 1])) {
        token_end -= 1;
      }
      for (size_t i = token_start; i < token_end; i++) {
        if (payload[i] == ';') {
          token_end = i;
          break;
        }
      }
      if (token_start >= token_end) {
        return -3;
      }

      size_t chunk_size = 0;
      for (size_t i = token_start; i < token_end; i++) {
        unsigned char ch = (unsigned char) payload[i];
        int value = -1;
        if (ch >= '0' && ch <= '9') {
          value = (int) (ch - '0');
        } else if (ch >= 'a' && ch <= 'f') {
          value = 10 + (int) (ch - 'a');
        } else if (ch >= 'A' && ch <= 'F') {
          value = 10 + (int) (ch - 'A');
        } else {
          return -3;
        }
        if (chunk_size > (SIZE_MAX - (size_t) value) / 16) {
          return -2;
        }
        chunk_size = (chunk_size * 16) + (size_t) value;
      }

      if (chunk_size == 0) {
        if (cursor + 1 < payload_bytes && payload[cursor] == '\r' && payload[cursor + 1] == '\n') {
          cursor += 2;
        } else {
          while (cursor + 1 < payload_bytes) {
            if (payload[cursor] == '\r' && payload[cursor + 1] == '\n') {
              cursor += 2;
              if (cursor + 1 < payload_bytes && payload[cursor] == '\r'
                  && payload[cursor + 1] == '\n') {
                cursor += 2;
                break;
              }
              continue;
            }
            cursor += 1;
          }
        }
        if (decoded_len >= body_size) {
          return -5;
        }
        body[decoded_len] = '\0';
        *body_len = decoded_len;
        return 0;
      }

      if (chunk_size > max_body_bytes || decoded_len > max_body_bytes - chunk_size) {
        return -2;
      }
      if (decoded_len + chunk_size >= body_size) {
        return -5;
      }
      if (cursor + chunk_size + 2 > payload_bytes) {
        return -3;
      }

      memcpy(body + decoded_len, payload + cursor, chunk_size);
      decoded_len += chunk_size;
      cursor += chunk_size;
      if (!(payload[cursor] == '\r' && payload[cursor + 1] == '\n')) {
        return -3;
      }
      cursor += 2;
    }
  }

  if (has_content_length) {
    if (content_length > payload_bytes) {
      return -3;
    }
    payload_bytes = content_length;
  }

  if (payload_bytes > max_body_bytes) {
    return -2;
  }
  if (payload_bytes >= body_size) {
    return -5;
  }

  if (payload_bytes > 0) {
    memcpy(body, payload, payload_bytes);
  }
  body[payload_bytes] = '\0';
  *body_len = payload_bytes;
  return 0;
}

static int sec4_rt_read_outbound_http_body(
    int socket_fd,
    char *body,
    size_t body_size,
    size_t max_body_bytes,
    size_t *body_len,
    int *status_code,
    char *redirect_location,
    size_t redirect_location_size
) {
  if (socket_fd < 0 || body == NULL || body_size == 0 || body_len == NULL || max_body_bytes == 0) {
    return -1;
  }
  body[0] = '\0';
  *body_len = 0;
  if (status_code != NULL) {
    *status_code = 0;
  }
  if (redirect_location != NULL && redirect_location_size > 0) {
    redirect_location[0] = '\0';
  }

  size_t response_capacity = max_body_bytes + SEC4_RT_MAX_OUTBOUND_HTTP_HEADER_BYTES + 1;
  if (response_capacity <= max_body_bytes || response_capacity <= SEC4_RT_MAX_OUTBOUND_HTTP_HEADER_BYTES) {
    return -1;
  }

  char *response = (char *) malloc(response_capacity);
  if (response == NULL) {
    return -1;
  }

  size_t total = 0;
  while (total < response_capacity - 1) {
    ssize_t bytes_read = recv(socket_fd, response + total, response_capacity - 1 - total, 0);
    if (bytes_read < 0) {
      if (errno == EINTR) {
        continue;
      }
      if (errno == EAGAIN || errno == EWOULDBLOCK || errno == ETIMEDOUT) {
        free(response);
        return -4;
      }
      free(response);
      return -1;
    }
    if (bytes_read == 0) {
      break;
    }
    total += (size_t) bytes_read;
  }

  if (total == response_capacity - 1) {
    free(response);
    return -2;
  }

  response[total] = '\0';
  int extract_status = sec4_rt_extract_outbound_http_body(
      response,
      total,
      body,
      body_size,
      max_body_bytes,
      body_len,
      status_code,
      redirect_location,
      redirect_location_size
  );
  free(response);
  return extract_status;
}

#ifdef SEC4_RT_ENABLE_OPENSSL_TLS
static int sec4_rt_ssl_write_all(SSL *ssl, const char *buffer, size_t size) {
  if (ssl == NULL || buffer == NULL) {
    return -1;
  }

  size_t written = 0;
  while (written < size) {
    int chunk_size = (int) (size - written);
    int rc = SSL_write(ssl, buffer + written, chunk_size);
    if (rc > 0) {
      written += (size_t) rc;
      continue;
    }

    int ssl_error = SSL_get_error(ssl, rc);
    if (ssl_error == SSL_ERROR_WANT_READ || ssl_error == SSL_ERROR_WANT_WRITE) {
      continue;
    }
    return -1;
  }
  return 0;
}

static int sec4_rt_read_outbound_https_body(
    SSL *ssl,
    char *body,
    size_t body_size,
    size_t max_body_bytes,
    size_t *body_len,
    int *status_code,
    char *redirect_location,
    size_t redirect_location_size
) {
  if (ssl == NULL || body == NULL || body_size == 0 || body_len == NULL || max_body_bytes == 0) {
    return -1;
  }
  body[0] = '\0';
  *body_len = 0;
  if (status_code != NULL) {
    *status_code = 0;
  }
  if (redirect_location != NULL && redirect_location_size > 0) {
    redirect_location[0] = '\0';
  }

  size_t response_capacity = max_body_bytes + SEC4_RT_MAX_OUTBOUND_HTTP_HEADER_BYTES + 1;
  if (response_capacity <= max_body_bytes || response_capacity <= SEC4_RT_MAX_OUTBOUND_HTTP_HEADER_BYTES) {
    return -1;
  }

  char *response = (char *) malloc(response_capacity);
  if (response == NULL) {
    return -1;
  }

  size_t total = 0;
  while (total < response_capacity - 1) {
    int chunk_size = (int) (response_capacity - 1 - total);
    int bytes_read = SSL_read(ssl, response + total, chunk_size);
    if (bytes_read > 0) {
      total += (size_t) bytes_read;
      continue;
    }
    if (bytes_read == 0) {
      break;
    }

    int ssl_error = SSL_get_error(ssl, bytes_read);
    if (ssl_error == SSL_ERROR_WANT_READ || ssl_error == SSL_ERROR_WANT_WRITE) {
      continue;
    }
    if (ssl_error == SSL_ERROR_SYSCALL
        && (errno == EAGAIN || errno == EWOULDBLOCK || errno == ETIMEDOUT)) {
      free(response);
      return -4;
    }
    free(response);
    return -1;
  }

  if (total == response_capacity - 1) {
    free(response);
    return -2;
  }

  response[total] = '\0';
  int extract_status = sec4_rt_extract_outbound_http_body(
      response,
      total,
      body,
      body_size,
      max_body_bytes,
      body_len,
      status_code,
      redirect_location,
      redirect_location_size
  );
  free(response);
  return extract_status;
}

static int sec4_rt_outbound_https_read_response(
    int socket_fd,
    const char *host,
    const char *request,
    size_t request_len,
    size_t max_body_bytes,
    char *body,
    size_t body_size,
    size_t *body_len,
    int *status_code,
    char *redirect_location,
    size_t redirect_location_size
) {
  if (socket_fd < 0 || host == NULL || request == NULL || request_len == 0 || body == NULL
      || body_size == 0 || body_len == NULL) {
    return -1;
  }
  body[0] = '\0';
  *body_len = 0;
  if (status_code != NULL) {
    *status_code = 0;
  }
  if (redirect_location != NULL && redirect_location_size > 0) {
    redirect_location[0] = '\0';
  }

  SSL_CTX *ssl_ctx = SSL_CTX_new(TLS_client_method());
  if (ssl_ctx == NULL) {
    sec4_rt_store_std_error_response(
        500,
        "NET.TLS_INIT_FAILED",
        "internal",
        "failed to initialize tls backend for outbound request"
    );
    return -6;
  }
  bool allow_insecure_tls = sec4_rt_env_flag_enabled("SEC4_RT_TLS_ALLOW_INSECURE");
  if (allow_insecure_tls) {
    SSL_CTX_set_verify(ssl_ctx, SSL_VERIFY_NONE, NULL);
  } else {
    SSL_CTX_set_verify(ssl_ctx, SSL_VERIFY_PEER, NULL);
    if (SSL_CTX_set_default_verify_paths(ssl_ctx) != 1) {
      SSL_CTX_free(ssl_ctx);
      sec4_rt_store_std_error_response(
          500,
          "NET.TLS_INIT_FAILED",
          "internal",
          "failed to configure tls trust store for outbound request"
      );
      return -6;
    }
  }

  SSL *ssl = SSL_new(ssl_ctx);
  if (ssl == NULL) {
    SSL_CTX_free(ssl_ctx);
    sec4_rt_store_std_error_response(
        500,
        "NET.TLS_INIT_FAILED",
        "internal",
        "failed to initialize tls backend for outbound request"
    );
    return -6;
  }

  bool handshake_complete = false;
  if (SSL_set_fd(ssl, socket_fd) != 1 || SSL_set_tlsext_host_name(ssl, host) != 1) {
    SSL_free(ssl);
    SSL_CTX_free(ssl_ctx);
    sec4_rt_store_std_error_response(
        500,
        "NET.TLS_HANDSHAKE_FAILED",
        "internal",
        "failed to complete outbound tls handshake"
    );
    return -6;
  }
  if (!allow_insecure_tls && SSL_set1_host(ssl, host) != 1) {
    SSL_free(ssl);
    SSL_CTX_free(ssl_ctx);
    sec4_rt_store_std_error_response(
        500,
        "NET.TLS_INIT_FAILED",
        "internal",
        "failed to configure tls hostname verification for outbound request"
    );
    return -6;
  }

  if (SSL_connect(ssl) != 1) {
    long verify_result = SSL_get_verify_result(ssl);
    X509 *peer_cert = !allow_insecure_tls ? SSL_get_peer_certificate(ssl) : NULL;
    bool verify_failed =
        !allow_insecure_tls && (verify_result != X509_V_OK || peer_cert == NULL);
    if (peer_cert != NULL) {
      X509_free(peer_cert);
    }
    SSL_free(ssl);
    SSL_CTX_free(ssl_ctx);
    if (verify_failed) {
      sec4_rt_store_std_error_response(
          500,
          "NET.TLS_VERIFY_FAILED",
          "validation",
          "outbound tls certificate verification failed"
      );
      return -6;
    }
    sec4_rt_store_std_error_response(
        500,
        "NET.TLS_HANDSHAKE_FAILED",
        "internal",
        "failed to complete outbound tls handshake"
    );
    return -6;
  }
  handshake_complete = true;
  if (!allow_insecure_tls) {
    X509 *peer_cert = SSL_get_peer_certificate(ssl);
    if (peer_cert == NULL || SSL_get_verify_result(ssl) != X509_V_OK) {
      if (peer_cert != NULL) {
        X509_free(peer_cert);
      }
      (void) SSL_shutdown(ssl);
      SSL_free(ssl);
      SSL_CTX_free(ssl_ctx);
      sec4_rt_store_std_error_response(
          500,
          "NET.TLS_VERIFY_FAILED",
          "validation",
          "outbound tls certificate verification failed"
      );
      return -6;
    }
    X509_free(peer_cert);
  }

  if (sec4_rt_ssl_write_all(ssl, request, request_len) != 0) {
    if (handshake_complete) {
      (void) SSL_shutdown(ssl);
    }
    SSL_free(ssl);
    SSL_CTX_free(ssl_ctx);
    sec4_rt_store_std_error_response(
        500,
        "NET.REQUEST_IO_FAILED",
        "internal",
        "failed to send outbound request"
    );
    return -6;
  }

  int read_status = sec4_rt_read_outbound_https_body(
      ssl,
      body,
      body_size,
      max_body_bytes,
      body_len,
      status_code,
      redirect_location,
      redirect_location_size
  );
  if (handshake_complete) {
    (void) SSL_shutdown(ssl);
  }
  SSL_free(ssl);
  SSL_CTX_free(ssl_ctx);
  return read_status;
}
#endif

static bool sec4_rt_store_outbound_http_read_error(int read_status) {
  if (read_status == 0) {
    return false;
  }
  if (read_status == -2) {
    sec4_rt_store_std_error_response(
        500,
        "NET.RESPONSE_TOO_LARGE",
        "validation",
        "outbound http response body exceeds runtime max body limit"
    );
    return true;
  }
  if (read_status == -4) {
    sec4_rt_store_std_error_response(
        500,
        "NET.READ_TIMEOUT",
        "timeout",
        "outbound http response read timed out"
    );
    return true;
  }
  if (read_status == -5) {
    sec4_rt_store_std_error_response(
        500,
        "NET.RESPONSE_TRACK_FAILED",
        "internal",
        "outbound http response exceeds runtime tracked value limits"
    );
    return true;
  }
  sec4_rt_store_std_error_response(
      500,
      "NET.RESPONSE_INVALID",
      "internal",
      "failed to read outbound http response"
  );
  return true;
}

static int64_t sec4_rt_track_outbound_http_body_handle(
    const char *body,
    size_t body_len,
    uint64_t salt
) {
  int64_t handle = sec4_rt_track_sized_value(body, body_len, salt);
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "NET.RESPONSE_TRACK_FAILED",
        "internal",
        "failed to track outbound http response body"
    );
    return 0;
  }
  return handle;
}

static bool sec4_rt_http_status_is_redirect(int status_code) {
  return status_code == 301 || status_code == 302 || status_code == 303 || status_code == 307
      || status_code == 308;
}

static bool sec4_rt_redirect_url_passes_scope(const char *url, sec4_rt_net_scope scope) {
  if (scope == SEC4_RT_NET_SCOPE_INTERNAL) {
    return sec4_rt_is_internal_url_valid(url);
  }
  return sec4_rt_is_public_url_valid(url);
}

static bool sec4_rt_resolve_redirect_url(
    const char *current_url,
    const char *location,
    char *resolved_url,
    size_t resolved_url_size
) {
  if (current_url == NULL || current_url[0] == '\0' || location == NULL || location[0] == '\0'
      || resolved_url == NULL || resolved_url_size < 2) {
    return false;
  }
  resolved_url[0] = '\0';

  for (const char *cursor = location; *cursor != '\0'; cursor++) {
    if (*cursor == '\r' || *cursor == '\n') {
      return false;
    }
  }

  if (strncasecmp(location, "http://", 7) == 0 || strncasecmp(location, "https://", 8) == 0) {
    int copied = snprintf(resolved_url, resolved_url_size, "%s", location);
    if (copied <= 0 || (size_t) copied >= resolved_url_size) {
      return false;
    }
  } else {
    char host[SEC4_RT_MAX_OUTBOUND_HTTP_HOST_BYTES];
    uint16_t port = 0;
    const char *target = NULL;
    size_t target_len = 0;
    bool is_http = false;
    bool is_https = false;
    if (!sec4_rt_parse_outbound_http_url(
            current_url,
            host,
            sizeof(host),
            &port,
            &target,
            &target_len,
            &is_http,
            &is_https
        )) {
      return false;
    }

    const char *scheme = is_https ? "https" : "http";
    char authority[SEC4_RT_MAX_OUTBOUND_HTTP_HOST_BYTES + 8];
    bool default_port = (is_https && port == 443) || (is_http && port == 80);
    int authority_len = default_port
        ? snprintf(authority, sizeof(authority), "%s", host)
        : snprintf(authority, sizeof(authority), "%s:%u", host, (unsigned int) port);
    if (authority_len <= 0 || (size_t) authority_len >= sizeof(authority)) {
      return false;
    }

    if (location[0] == '/' && location[1] == '/') {
      int written = snprintf(resolved_url, resolved_url_size, "%s:%s", scheme, location);
      if (written <= 0 || (size_t) written >= resolved_url_size) {
        return false;
      }
    } else {
      const char *resolved_target = location;
      char target_buffer[SEC4_RT_MAX_OUTBOUND_HTTP_URL_BYTES];
      target_buffer[0] = '\0';

      const char *target_path = target;
      size_t target_path_len = target_len;
      if (target_path_len == 0 || target_path == NULL) {
        target_path = "/";
        target_path_len = 1;
      }
      if (target_path[0] == '?') {
        target_path = "/";
        target_path_len = 1;
      } else {
        const char *query_sep = memchr(target_path, '?', target_path_len);
        if (query_sep != NULL) {
          target_path_len = (size_t) (query_sep - target_path);
        }
        if (target_path_len == 0) {
          target_path = "/";
          target_path_len = 1;
        }
      }

      if (location[0] == '?') {
        int merged_len = snprintf(
            target_buffer,
            sizeof(target_buffer),
            "%.*s%s",
            (int) target_path_len,
            target_path,
            location
        );
        if (merged_len <= 0 || (size_t) merged_len >= sizeof(target_buffer)) {
          return false;
        }
        resolved_target = target_buffer;
      } else if (location[0] != '/') {
        size_t dir_len = 1;
        for (size_t i = 0; i < target_path_len; i++) {
          if (target_path[i] == '/') {
            dir_len = i + 1;
          }
        }
        int merged_len = snprintf(
            target_buffer,
            sizeof(target_buffer),
            "%.*s%s",
            (int) dir_len,
            target_path,
            location
        );
        if (merged_len <= 0 || (size_t) merged_len >= sizeof(target_buffer)) {
          return false;
        }
        resolved_target = target_buffer;
      }

      int written = snprintf(
          resolved_url,
          resolved_url_size,
          "%s://%s%s",
          scheme,
          authority,
          resolved_target
      );
      if (written <= 0 || (size_t) written >= resolved_url_size) {
        return false;
      }
    }
  }

  char parsed_host[SEC4_RT_MAX_OUTBOUND_HTTP_HOST_BYTES];
  uint16_t parsed_port = 0;
  const char *parsed_target = NULL;
  size_t parsed_target_len = 0;
  bool parsed_http = false;
  bool parsed_https = false;
  if (!sec4_rt_parse_outbound_http_url(
          resolved_url,
          parsed_host,
          sizeof(parsed_host),
          &parsed_port,
          &parsed_target,
          &parsed_target_len,
          &parsed_http,
          &parsed_https
      )) {
    return false;
  }
  return parsed_http || parsed_https;
}

static int64_t sec4_rt_outbound_http_get_handle(
    const char *url_value,
    uint64_t salt,
    sec4_rt_net_scope scope
) {
  if (url_value == NULL || url_value[0] == '\0') {
    return 0;
  }

  char current_url[SEC4_RT_MAX_OUTBOUND_HTTP_URL_BYTES];
  int current_url_len = snprintf(current_url, sizeof(current_url), "%s", url_value);
  if (current_url_len <= 0 || (size_t) current_url_len >= sizeof(current_url)) {
    sec4_rt_store_std_error_response(
        400,
        "NET.URL_INVALID",
        "validation",
        "invalid outbound http url"
    );
    return 0;
  }

  int64_t timeout_ms = sec4_rt_outbound_http_timeout_ms();
  size_t max_body_bytes = sec4_rt_outbound_http_max_body_bytes();
  bool redirects_allowed = sec4_rt_env_flag_enabled("SEC4_RT_NET_PUBLIC_ALLOW_REDIRECTS");
  int64_t max_redirects = sec4_rt_parse_env_i64("SEC4_RT_NET_PUBLIC_MAX_REDIRECTS", 0);
  bool revalidate_redirects =
      sec4_rt_env_flag_enabled_default("SEC4_RT_NET_SSRF_REVALIDATE_REDIRECTS", true);
  int64_t redirects_followed = 0;

  while (true) {
    char host[SEC4_RT_MAX_OUTBOUND_HTTP_HOST_BYTES];
    uint16_t port = 0;
    const char *target = NULL;
    size_t target_len = 0;
    bool is_http = false;
    bool is_https = false;
    if (!sec4_rt_parse_outbound_http_url(
            current_url,
            host,
            sizeof(host),
            &port,
            &target,
            &target_len,
            &is_http,
            &is_https
        )) {
      sec4_rt_store_std_error_response(
          400,
          "NET.URL_INVALID",
          "validation",
          "invalid outbound http url"
      );
      return 0;
    }

#if !defined(SEC4_RT_ENABLE_OPENSSL_TLS)
    if (is_https) {
      sec4_rt_store_std_error_response(
          501,
          "NET.TLS_UNSUPPORTED",
          "runtime",
          "https outbound transport is not supported by this runtime"
      );
      return 0;
    }
#endif
    if (!(is_http || is_https)) {
      sec4_rt_store_std_error_response(
          400,
          "NET.URL_SCHEME_INVALID",
          "validation",
          "http outbound transport requires http scheme"
      );
      return 0;
    }

    char request[SEC4_RT_MAX_OUTBOUND_HTTP_REQUEST_BYTES];
    int request_len = sec4_rt_build_outbound_http_request(
        host,
        port,
        is_https,
        target,
        target_len,
        request,
        sizeof(request)
    );
    if (request_len <= 0) {
      sec4_rt_store_std_error_response(
          500,
          "NET.REQUEST_BUILD_FAILED",
          "internal",
          "failed to construct outbound request payload"
      );
      return 0;
    }

    bool connect_timed_out = false;
    int socket_fd = sec4_rt_open_outbound_tcp_socket(host, port, timeout_ms, &connect_timed_out);
    if (socket_fd < 0) {
      if (connect_timed_out) {
        sec4_rt_store_std_error_response(
            500,
            "NET.CONNECT_TIMEOUT",
            "timeout",
            "outbound http connect timed out"
        );
        return 0;
      }
      sec4_rt_store_std_error_response(
          500,
          "NET.CONNECT_FAILED",
          "internal",
          "failed to connect outbound http socket"
      );
      return 0;
    }

    char body[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
    size_t body_len = 0;
    int status_code = 0;
    char redirect_location[SEC4_RT_MAX_OUTBOUND_HTTP_URL_BYTES];
    int read_status = 0;

#ifdef SEC4_RT_ENABLE_OPENSSL_TLS
    if (is_https) {
      read_status = sec4_rt_outbound_https_read_response(
          socket_fd,
          host,
          request,
          (size_t) request_len,
          max_body_bytes,
          body,
          sizeof(body),
          &body_len,
          &status_code,
          redirect_location,
          sizeof(redirect_location)
      );
      close(socket_fd);
      if (read_status == -6) {
        return 0;
      }
    } else
#endif
    {
      if (sec4_rt_write_all(socket_fd, request, (size_t) request_len) != 0) {
        close(socket_fd);
        sec4_rt_store_std_error_response(
            500,
            "NET.REQUEST_IO_FAILED",
            "internal",
            "failed to send outbound request"
        );
        return 0;
      }

      read_status = sec4_rt_read_outbound_http_body(
          socket_fd,
          body,
          sizeof(body),
          max_body_bytes,
          &body_len,
          &status_code,
          redirect_location,
          sizeof(redirect_location)
      );
      close(socket_fd);
    }

    if (sec4_rt_store_outbound_http_read_error(read_status)) {
      return 0;
    }

    if (sec4_rt_http_status_is_redirect(status_code)) {
      if (!redirects_allowed) {
        sec4_rt_store_std_error_response(
            403,
            "NET.REDIRECT_FORBIDDEN",
            "authorization",
            "outbound redirect denied by runtime policy"
        );
        return 0;
      }
      if (redirects_followed >= max_redirects) {
        sec4_rt_store_std_error_response(
            400,
            "NET.REDIRECT_LIMIT",
            "validation",
            "outbound redirect hop limit exceeded"
        );
        return 0;
      }

      char next_url[SEC4_RT_MAX_OUTBOUND_HTTP_URL_BYTES];
      bool redirect_valid = sec4_rt_resolve_redirect_url(
          current_url,
          redirect_location,
          next_url,
          sizeof(next_url)
      );
      if (redirect_valid && revalidate_redirects) {
        redirect_valid = sec4_rt_redirect_url_passes_scope(next_url, scope);
      }
      if (!redirect_valid) {
        sec4_rt_store_std_error_response(
            400,
            "NET.REDIRECT_INVALID",
            "validation",
            "outbound redirect location is invalid"
        );
        return 0;
      }

      int copied = snprintf(current_url, sizeof(current_url), "%s", next_url);
      if (copied <= 0 || (size_t) copied >= sizeof(current_url)) {
        sec4_rt_store_std_error_response(
            400,
            "NET.REDIRECT_INVALID",
            "validation",
            "outbound redirect location is invalid"
        );
        return 0;
      }
      redirects_followed += 1;
      continue;
    }

    return sec4_rt_track_outbound_http_body_handle(body, body_len, salt);
  }
}

static bool sec4_rt_host_is_internal(const char *host, size_t host_len) {
  if (sec4_rt_host_equals(host, host_len, "localhost")) {
    return true;
  }
  if (sec4_rt_host_ends_with(host, host_len, ".local")
      || sec4_rt_host_ends_with(host, host_len, ".internal")) {
    return true;
  }
  if (sec4_rt_parse_ipv4_private(host, host_len)) {
    return true;
  }
  return false;
}

static bool sec4_rt_is_public_url_valid(const char *url) {
  const char *host = NULL;
  size_t host_len = 0;
  bool is_http = false;
  bool is_https = false;
  if (!sec4_rt_parse_url_host(url, &host, &host_len, &is_http, &is_https)) {
    return false;
  }
  if (!(is_http || is_https)) {
    return false;
  }
  if (sec4_rt_host_is_internal(host, host_len)) {
    return false;
  }
  if (sec4_rt_env_flag_enabled_default("SEC4_RT_NET_SSRF_RESOLVE_DNS", true)
      && sec4_rt_host_resolves_to_internal(host, host_len)) {
    return false;
  }

  const char *scheme = is_https ? "https" : "http";
  const char *allowed_schemes = getenv("SEC4_RT_NET_PUBLIC_ALLOWED_SCHEMES");
  if (sec4_rt_csv_has_any_token(allowed_schemes)
      && !sec4_rt_csv_contains_token_ci(allowed_schemes, scheme, strlen(scheme))) {
    return false;
  }

  const char *allowed_ports = getenv("SEC4_RT_NET_PUBLIC_ALLOWED_PORTS");
  if (sec4_rt_csv_has_any_token(allowed_ports)) {
    uint16_t resolved_port = 0;
    if (!sec4_rt_resolve_url_port(host, host_len, is_https, &resolved_port)
        || !sec4_rt_csv_contains_port(allowed_ports, resolved_port)) {
      return false;
    }
  }

  const char *blocked_domains = getenv("SEC4_RT_NET_PUBLIC_BLOCKED_DOMAINS");
  if (sec4_rt_csv_contains_token_ci(blocked_domains, host, host_len)) {
    return false;
  }

  const char *allowed_domains = getenv("SEC4_RT_NET_PUBLIC_ALLOWED_DOMAINS");
  if (sec4_rt_csv_has_any_token(allowed_domains)
      && !sec4_rt_csv_contains_token_ci(allowed_domains, host, host_len)) {
    return false;
  }

  return true;
}

static bool sec4_rt_is_internal_url_valid(const char *url) {
  const char *host = NULL;
  size_t host_len = 0;
  bool is_http = false;
  bool is_https = false;
  if (!sec4_rt_parse_url_host(url, &host, &host_len, &is_http, &is_https)) {
    return false;
  }
  if (!(is_http || is_https)) {
    return false;
  }
  if (!sec4_rt_host_is_internal(host, host_len)) {
    return false;
  }

  const char *allowed_domains = getenv("SEC4_RT_NET_INTERNAL_ALLOWED_DOMAINS");
  const char *allowed_cidrs = getenv("SEC4_RT_NET_INTERNAL_ALLOWED_CIDRS");
  bool has_allowed_domains = sec4_rt_csv_has_any_token(allowed_domains);
  bool has_allowed_cidrs = sec4_rt_csv_has_any_token(allowed_cidrs);
  bool domains_match =
      has_allowed_domains && sec4_rt_csv_contains_token_ci(allowed_domains, host, host_len);
  bool cidrs_match = has_allowed_cidrs
      && sec4_rt_csv_contains_ipv4_cidr_match(allowed_cidrs, host, host_len);
  if ((has_allowed_domains || has_allowed_cidrs) && !(domains_match || cidrs_match)) {
    return false;
  }

  return true;
}

static bool sec4_rt_is_header_name_valid(const char *value) {
  if (value == NULL || value[0] == '\0') {
    return false;
  }
  while (*value != '\0') {
    unsigned char ch = (unsigned char) *value;
    if (!(isalnum(ch) || ch == '-')) {
      return false;
    }
    value += 1;
  }
  return true;
}

static bool sec4_rt_is_header_value_valid(const char *value) {
  if (value == NULL || value[0] == '\0') {
    return false;
  }
  while (*value != '\0') {
    if (*value == '\r' || *value == '\n') {
      return false;
    }
    value += 1;
  }
  return true;
}

static bool sec4_rt_is_email_local_char(unsigned char ch) {
  return isalnum(ch)
      || ch == '.'
      || ch == '!'
      || ch == '#'
      || ch == '$'
      || ch == '%'
      || ch == '&'
      || ch == '\''
      || ch == '*'
      || ch == '+'
      || ch == '/'
      || ch == '='
      || ch == '?'
      || ch == '^'
      || ch == '_'
      || ch == '`'
      || ch == '{'
      || ch == '|'
      || ch == '}'
      || ch == '~'
      || ch == '-';
}

static bool sec4_rt_is_email_valid(const char *value) {
  if (value == NULL) {
    return false;
  }

  size_t total_len = strlen(value);
  if (total_len < 3 || total_len > 320) {
    return false;
  }

  const char *at = strchr(value, '@');
  if (at == NULL || strchr(at + 1, '@') != NULL) {
    return false;
  }

  size_t local_len = (size_t) (at - value);
  const char *domain = at + 1;
  size_t domain_len = strlen(domain);
  if (local_len == 0 || local_len > 64 || domain_len < 3 || domain_len > 255) {
    return false;
  }

  if (value[0] == '.' || value[local_len - 1] == '.') {
    return false;
  }
  for (size_t i = 0; i < local_len; i++) {
    unsigned char ch = (unsigned char) value[i];
    if (!sec4_rt_is_email_local_char(ch)) {
      return false;
    }
    if (ch == '.' && i > 0 && value[i - 1] == '.') {
      return false;
    }
  }

  if (domain[0] == '.' || domain[domain_len - 1] == '.') {
    return false;
  }

  bool has_dot = false;
  const char *label_start = domain;
  size_t label_len = 0;
  for (size_t i = 0; i <= domain_len; i++) {
    char ch = domain[i];
    if (ch == '.' || ch == '\0') {
      if (label_len == 0 || label_len > 63) {
        return false;
      }
      if (label_start[0] == '-' || label_start[label_len - 1] == '-') {
        return false;
      }
      if (ch == '.') {
        has_dot = true;
        label_start = domain + i + 1;
        label_len = 0;
      } else {
        break;
      }
      continue;
    }

    if (!(isalnum((unsigned char) ch) || ch == '-')) {
      return false;
    }
    label_len += 1;
  }

  return has_dot;
}

static bool sec4_rt_is_uuid_valid(const char *value) {
  if (value == NULL || strlen(value) != 36) {
    return false;
  }

  for (size_t i = 0; i < 36; i++) {
    if (i == 8 || i == 13 || i == 18 || i == 23) {
      if (value[i] != '-') {
        return false;
      }
      continue;
    }

    if (!isxdigit((unsigned char) value[i])) {
      return false;
    }
  }
  return true;
}

static bool sec4_rt_is_int64_text_valid(const char *value) {
  if (value == NULL || value[0] == '\0') {
    return false;
  }

  size_t index = 0;
  if (value[index] == '+' || value[index] == '-') {
    index += 1;
  }
  if (value[index] == '\0') {
    return false;
  }
  for (; value[index] != '\0'; index++) {
    if (!isdigit((unsigned char) value[index])) {
      return false;
    }
  }

  errno = 0;
  char *end = NULL;
  (void) strtoll(value, &end, 10);
  if (errno == ERANGE || end == NULL || *end != '\0') {
    return false;
  }
  return true;
}

static bool sec4_rt_escape_html(
    const char *input,
    char *escaped,
    size_t escaped_size,
    size_t *escaped_len
) {
  if (input == NULL || escaped == NULL || escaped_size == 0 || escaped_len == NULL) {
    return false;
  }

  size_t write = 0;
  for (size_t i = 0; input[i] != '\0'; i++) {
    const char *replacement = NULL;
    switch (input[i]) {
      case '&':
        replacement = "&amp;";
        break;
      case '<':
        replacement = "&lt;";
        break;
      case '>':
        replacement = "&gt;";
        break;
      case '"':
        replacement = "&quot;";
        break;
      case '\'':
        replacement = "&#39;";
        break;
      default:
        replacement = NULL;
        break;
    }

    if (replacement != NULL) {
      size_t replacement_len = strlen(replacement);
      if (write + replacement_len >= escaped_size) {
        return false;
      }
      memcpy(escaped + write, replacement, replacement_len);
      write += replacement_len;
      continue;
    }

    if (write + 1 >= escaped_size) {
      return false;
    }
    escaped[write++] = input[i];
  }

  escaped[write] = '\0';
  *escaped_len = write;
  return true;
}

static bool sec4_rt_normalize_absolute_path(
    const char *value,
    char *normalized,
    size_t normalized_size
) {
  if (value == NULL || normalized == NULL || normalized_size < 2 || value[0] != '/') {
    return false;
  }

  size_t write = 0;
  normalized[write++] = '/';

  const char *cursor = value;
  while (*cursor == '/') {
    cursor += 1;
  }

  while (*cursor != '\0') {
    const char *segment_start = cursor;
    while (*cursor != '\0' && *cursor != '/') {
      unsigned char ch = (unsigned char) *cursor;
      if (ch < 0x20 || ch == 0x7f || ch == '\\') {
        return false;
      }
      cursor += 1;
    }

    size_t segment_len = (size_t) (cursor - segment_start);
    if (segment_len == 0) {
      while (*cursor == '/') {
        cursor += 1;
      }
      continue;
    }

    if ((segment_len == 1 && segment_start[0] == '.')
        || (segment_len == 2 && segment_start[0] == '.' && segment_start[1] == '.')) {
      return false;
    }

    if (write > 1) {
      if (write + 1 >= normalized_size) {
        return false;
      }
      normalized[write++] = '/';
    }

    if (write + segment_len >= normalized_size) {
      return false;
    }
    memcpy(normalized + write, segment_start, segment_len);
    write += segment_len;

    while (*cursor == '/') {
      cursor += 1;
    }
  }

  normalized[write] = '\0';
  return true;
}

static bool sec4_rt_normalize_under_base_path(
    const char *normalized_base,
    const char *input,
    char *normalized,
    size_t normalized_size
) {
  if (normalized_base == NULL || input == NULL || input[0] == '\0') {
    return false;
  }

  if (input[0] == '/') {
    return sec4_rt_normalize_absolute_path(input, normalized, normalized_size);
  }

  char joined[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  int written = 0;
  if (strcmp(normalized_base, "/") == 0) {
    written = snprintf(joined, sizeof(joined), "/%s", input);
  } else {
    written = snprintf(joined, sizeof(joined), "%s/%s", normalized_base, input);
  }

  if (written <= 0 || (size_t) written >= sizeof(joined)) {
    return false;
  }
  return sec4_rt_normalize_absolute_path(joined, normalized, normalized_size);
}

static bool sec4_rt_path_within_base(const char *normalized_base, const char *normalized_path) {
  if (normalized_base == NULL || normalized_path == NULL) {
    return false;
  }
  if (strcmp(normalized_base, "/") == 0) {
    return normalized_path[0] == '/';
  }

  size_t base_len = strlen(normalized_base);
  if (strncmp(normalized_base, normalized_path, base_len) != 0) {
    return false;
  }

  return normalized_path[base_len] == '\0' || normalized_path[base_len] == '/';
}

static int64_t sec4_rt_append_response_header(const char *name, const char *value) {
  if (!sec4_rt_is_header_name_valid(name) || !sec4_rt_is_header_value_valid(value)) {
    return 1;
  }

  size_t used = g_sec4_rt_response.extra_headers_len;
  if (used >= sizeof(g_sec4_rt_response.extra_headers)) {
    return 1;
  }

  size_t remaining = sizeof(g_sec4_rt_response.extra_headers) - used;
  int written = snprintf(
      g_sec4_rt_response.extra_headers + used,
      remaining,
      "%s: %s\r\n",
      name,
      value
  );
  if (written <= 0 || (size_t) written >= remaining) {
    return 1;
  }
  g_sec4_rt_response.extra_headers_len += (size_t) written;
  return 0;
}

static const char *sec4_rt_response_extra_headers(void) {
  if (g_sec4_rt_response.extra_headers_len == 0) {
    return NULL;
  }
  return g_sec4_rt_response.extra_headers;
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

static bool sec4_rt_trim_json_token_bounds(
    const char *input,
    size_t input_len,
    const char **start_out,
    size_t *len_out
) {
  if (input == NULL || start_out == NULL || len_out == NULL || input_len == 0) {
    return false;
  }

  size_t start = 0;
  while (start < input_len && isspace((unsigned char) input[start])) {
    start += 1;
  }
  if (start >= input_len) {
    return false;
  }

  size_t end = input_len;
  while (end > start && isspace((unsigned char) input[end - 1])) {
    end -= 1;
  }
  if (end <= start) {
    return false;
  }

  *start_out = input + start;
  *len_out = end - start;
  return true;
}

static bool sec4_rt_is_json_number_token(const char *token, size_t token_len) {
  if (token == NULL || token_len == 0) {
    return false;
  }

  size_t index = 0;
  if (token[index] == '-') {
    index += 1;
  }
  if (index >= token_len) {
    return false;
  }

  if (token[index] == '0') {
    index += 1;
  } else if (token[index] >= '1' && token[index] <= '9') {
    index += 1;
    while (index < token_len && isdigit((unsigned char) token[index])) {
      index += 1;
    }
  } else {
    return false;
  }

  if (index < token_len && token[index] == '.') {
    index += 1;
    if (index >= token_len || !isdigit((unsigned char) token[index])) {
      return false;
    }
    while (index < token_len && isdigit((unsigned char) token[index])) {
      index += 1;
    }
  }

  if (index < token_len && (token[index] == 'e' || token[index] == 'E')) {
    index += 1;
    if (index < token_len && (token[index] == '+' || token[index] == '-')) {
      index += 1;
    }
    if (index >= token_len || !isdigit((unsigned char) token[index])) {
      return false;
    }
    while (index < token_len && isdigit((unsigned char) token[index])) {
      index += 1;
    }
  }

  return index == token_len;
}

static bool sec4_rt_is_json_scalar_literal(const char *value, size_t value_len) {
  const char *token = NULL;
  size_t token_len = 0;
  if (!sec4_rt_trim_json_token_bounds(value, value_len, &token, &token_len)) {
    return false;
  }

  if (token_len >= 2 && token[0] == '"' && token[token_len - 1] == '"') {
    return true;
  }
  if (token_len == 4 && strncmp(token, "true", 4) == 0) {
    return true;
  }
  if (token_len == 5 && strncmp(token, "false", 5) == 0) {
    return true;
  }
  if (token_len == 4 && strncmp(token, "null", 4) == 0) {
    return true;
  }
  return sec4_rt_is_json_number_token(token, token_len);
}

static bool sec4_rt_escape_json_string(
    const char *input,
    char *escaped,
    size_t escaped_size,
    size_t *escaped_len
) {
  if (input == NULL || escaped == NULL || escaped_size == 0 || escaped_len == NULL) {
    return false;
  }

  static const char *hex = "0123456789abcdef";
  size_t write = 0;
  if (write + 2 > escaped_size) {
    return false;
  }
  escaped[write++] = '"';

  for (size_t i = 0; input[i] != '\0'; i++) {
    unsigned char ch = (unsigned char) input[i];
    const char *replacement = NULL;
    size_t replacement_len = 0;
    char unicode_escape[7];

    switch (ch) {
      case '"':
        replacement = "\\\"";
        replacement_len = 2;
        break;
      case '\\':
        replacement = "\\\\";
        replacement_len = 2;
        break;
      case '\b':
        replacement = "\\b";
        replacement_len = 2;
        break;
      case '\f':
        replacement = "\\f";
        replacement_len = 2;
        break;
      case '\n':
        replacement = "\\n";
        replacement_len = 2;
        break;
      case '\r':
        replacement = "\\r";
        replacement_len = 2;
        break;
      case '\t':
        replacement = "\\t";
        replacement_len = 2;
        break;
      default:
        if (ch < 0x20) {
          unicode_escape[0] = '\\';
          unicode_escape[1] = 'u';
          unicode_escape[2] = '0';
          unicode_escape[3] = '0';
          unicode_escape[4] = hex[(ch >> 4) & 0x0f];
          unicode_escape[5] = hex[ch & 0x0f];
          unicode_escape[6] = '\0';
          replacement = unicode_escape;
          replacement_len = 6;
        }
        break;
    }

    if (replacement != NULL) {
      if (write + replacement_len + 1 > escaped_size) {
        return false;
      }
      memcpy(escaped + write, replacement, replacement_len);
      write += replacement_len;
      continue;
    }

    if (write + 2 > escaped_size) {
      return false;
    }
    escaped[write++] = (char) ch;
  }

  if (write + 2 > escaped_size) {
    return false;
  }
  escaped[write++] = '"';
  escaped[write] = '\0';
  *escaped_len = write;
  return true;
}

static bool sec4_rt_render_json_fragment_from_value(
    int64_t value,
    char *fragment,
    size_t fragment_size,
    size_t *fragment_len
) {
  if (fragment == NULL || fragment_size == 0 || fragment_len == NULL) {
    return false;
  }

  const char *tracked = sec4_rt_lookup_tracked_value(value);
  if (tracked != NULL) {
    size_t tracked_len = strlen(tracked);
    if (sec4_rt_is_likely_json(tracked, tracked_len)
        || sec4_rt_is_json_scalar_literal(tracked, tracked_len)) {
      if (tracked_len + 1 > fragment_size) {
        return false;
      }
      memcpy(fragment, tracked, tracked_len + 1);
      *fragment_len = tracked_len;
      return true;
    }
    return sec4_rt_escape_json_string(tracked, fragment, fragment_size, fragment_len);
  }

  int written = snprintf(fragment, fragment_size, "%lld", (long long) value);
  if (written <= 0 || (size_t) written >= fragment_size) {
    return false;
  }
  *fragment_len = (size_t) written;
  return true;
}

static size_t sec4_rt_json_max_bytes_limit(void) {
  int64_t parsed = sec4_rt_parse_env_i64("SEC4_RT_JSON_MAX_BYTES", SEC4_RT_DEFAULT_JSON_MAX_BYTES);
  if (parsed <= 0) {
    parsed = SEC4_RT_DEFAULT_JSON_MAX_BYTES;
  }
  return (size_t) parsed;
}

static size_t sec4_rt_json_max_depth_limit(void) {
  int64_t parsed = sec4_rt_parse_env_i64("SEC4_RT_JSON_MAX_DEPTH", SEC4_RT_DEFAULT_JSON_MAX_DEPTH);
  if (parsed <= 0) {
    parsed = SEC4_RT_DEFAULT_JSON_MAX_DEPTH;
  }
  return (size_t) parsed;
}

static bool sec4_rt_json_exceeds_depth(const char *body, size_t body_len, size_t max_depth) {
  if (body == NULL || body_len == 0) {
    return false;
  }
  if (max_depth == 0) {
    return true;
  }

  bool in_string = false;
  bool escaped = false;
  size_t depth = 0;

  for (size_t i = 0; i < body_len; i++) {
    unsigned char ch = (unsigned char) body[i];
    if (in_string) {
      if (escaped) {
        escaped = false;
        continue;
      }
      if (ch == '\\') {
        escaped = true;
        continue;
      }
      if (ch == '"') {
        in_string = false;
      }
      continue;
    }

    if (ch == '"') {
      in_string = true;
      continue;
    }

    if (ch == '{' || ch == '[') {
      depth += 1;
      if (depth > max_depth) {
        return true;
      }
      continue;
    }

    if ((ch == '}' || ch == ']') && depth > 0) {
      depth -= 1;
    }
  }

  return false;
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

static bool sec4_rt_parse_header_value(
    const char *request,
    size_t request_len,
    const char *name,
    char *value,
    size_t value_size
) {
  if (value == NULL || value_size == 0) {
    return false;
  }
  value[0] = '\0';
  if (request == NULL || request_len == 0 || name == NULL || name[0] == '\0') {
    return false;
  }

  size_t name_len = strlen(name);
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

    if ((size_t) (line_end - cursor) > name_len && strncasecmp(cursor, name, name_len) == 0
        && cursor[name_len] == ':') {
      const char *start = cursor + name_len + 1;
      while (start < line_end && isspace((unsigned char) *start)) {
        start += 1;
      }
      const char *end = line_end;
      while (end > start && isspace((unsigned char) *(end - 1))) {
        end -= 1;
      }
      size_t len = (size_t) (end - start);
      if (len >= value_size) {
        len = value_size - 1;
      }
      memcpy(value, start, len);
      value[len] = '\0';
      return true;
    }

    cursor = line_end + 2;
  }

  return false;
}

static bool sec4_rt_parse_cookie_value(
    const char *cookie_header,
    const char *cookie_name,
    char *value,
    size_t value_size
) {
  if (value == NULL || value_size == 0) {
    return false;
  }
  value[0] = '\0';
  if (cookie_header == NULL || cookie_name == NULL || cookie_name[0] == '\0') {
    return false;
  }

  size_t name_len = strlen(cookie_name);
  const char *cursor = cookie_header;
  while (*cursor != '\0') {
    while (*cursor == ' ' || *cursor == ';') {
      cursor += 1;
    }
    if (*cursor == '\0') {
      break;
    }

    if (strncasecmp(cursor, cookie_name, name_len) == 0 && cursor[name_len] == '=') {
      const char *start = cursor + name_len + 1;
      const char *end = start;
      while (*end != '\0' && *end != ';') {
        end += 1;
      }
      size_t len = (size_t) (end - start);
      if (len >= value_size) {
        len = value_size - 1;
      }
      memcpy(value, start, len);
      value[len] = '\0';
      return len > 0;
    }

    while (*cursor != '\0' && *cursor != ';') {
      cursor += 1;
    }
  }

  return false;
}

static bool sec4_rt_auth_mode_allows_token(const char *mode) {
  if (mode == NULL || mode[0] == '\0') {
    return true;
  }
  if (strcasecmp(mode, "off") == 0) {
    return false;
  }
  if (strcasecmp(mode, "token") == 0 || strcasecmp(mode, "mixed") == 0) {
    return true;
  }
  return false;
}

static bool sec4_rt_auth_mode_allows_cookie(const char *mode) {
  if (mode == NULL || mode[0] == '\0') {
    return false;
  }
  if (strcasecmp(mode, "cookie") == 0 || strcasecmp(mode, "mixed") == 0) {
    return true;
  }
  return false;
}

static const char *sec4_rt_auth_cookie_name(void) {
  const char *configured = getenv("SEC4_RT_AUTH_COOKIE_NAME");
  if (configured != NULL && configured[0] != '\0') {
    return configured;
  }
  return "session";
}

static const char *sec4_rt_effective_auth_mode(const sec4_rt_router_state *router) {
  if (router != NULL && router->auth_mode[0] != '\0') {
    return router->auth_mode;
  }
  if (g_sec4_rt_auth_policy.loaded && g_sec4_rt_auth_policy.mode[0] != '\0') {
    return g_sec4_rt_auth_policy.mode;
  }
  const char *configured = getenv("SEC4_RT_AUTH_MODE");
  if (configured != NULL && configured[0] != '\0') {
    return configured;
  }
  return "token";
}

static const char *sec4_rt_auth_unauthorized_message(const char *mode) {
  if (sec4_rt_auth_mode_allows_token(mode) && sec4_rt_auth_mode_allows_cookie(mode)) {
    return "Authorization header or session cookie missing or invalid";
  }
  if (sec4_rt_auth_mode_allows_cookie(mode)) {
    return "Session cookie missing or invalid";
  }
  return "Authorization header missing or invalid";
}

static bool sec4_rt_extract_auth_cookie(char *cookie_value, size_t cookie_value_size) {
  if (cookie_value == NULL || cookie_value_size == 0) {
    return false;
  }
  cookie_value[0] = '\0';
  char cookie_header[512];
  if (!sec4_rt_extract_request_header("Cookie", cookie_header, sizeof(cookie_header))) {
    return false;
  }
  return sec4_rt_parse_cookie_value(
      cookie_header,
      sec4_rt_auth_cookie_name(),
      cookie_value,
      cookie_value_size
  );
}

static bool sec4_rt_auth_collect_subject(
    const char *mode,
    char *subject,
    size_t subject_size,
    bool *used_bearer_out
) {
  if (subject == NULL || subject_size == 0) {
    return false;
  }
  subject[0] = '\0';
  if (used_bearer_out != NULL) {
    *used_bearer_out = false;
  }

  if (sec4_rt_auth_mode_allows_token(mode)) {
    char auth_header[256];
    bool has_auth = sec4_rt_extract_request_header(
        "Authorization",
        auth_header,
        sizeof(auth_header)
    );
    if (has_auth && sec4_rt_is_valid_bearer_auth(auth_header)) {
      int copied = snprintf(subject, subject_size, "%s", auth_header);
      if (copied > 0 && (size_t) copied < subject_size) {
        if (used_bearer_out != NULL) {
          *used_bearer_out = true;
        }
        return true;
      }
    }
  }

  if (sec4_rt_auth_mode_allows_cookie(mode)) {
    char session_cookie[256];
    if (sec4_rt_extract_auth_cookie(session_cookie, sizeof(session_cookie))) {
      int copied = snprintf(subject, subject_size, "%s", session_cookie);
      if (copied > 0 && (size_t) copied < subject_size) {
        return true;
      }
    }
  }

  return false;
}

static bool sec4_rt_auth_cookie_has_role(const char *required_role) {
  if (required_role == NULL || required_role[0] == '\0') {
    return false;
  }

  char cookie_header[512];
  if (sec4_rt_extract_request_header("Cookie", cookie_header, sizeof(cookie_header))) {
    char role_cookie[128];
    if (sec4_rt_parse_cookie_value(cookie_header, "role", role_cookie, sizeof(role_cookie))) {
      return sec4_rt_constant_time_bytes_eq(
          (const unsigned char *) role_cookie,
          strlen(role_cookie),
          (const unsigned char *) required_role,
          strlen(required_role)
      );
    }
  }

  char role_header[128];
  if (sec4_rt_extract_request_header("X-Role", role_header, sizeof(role_header))) {
    return sec4_rt_constant_time_bytes_eq(
        (const unsigned char *) role_header,
        strlen(role_header),
        (const unsigned char *) required_role,
        strlen(required_role)
    );
  }

  return false;
}

static bool sec4_rt_is_csrf_protected_method(sec4_rt_router_state *router, const char *method) {
  if (method == NULL) {
    return false;
  }
  if (router != NULL && sec4_rt_csv_has_any_token(router->csrf_protected_methods)) {
    return sec4_rt_csv_contains_token_ci(
        router->csrf_protected_methods,
        method,
        strlen(method)
    );
  }
  return strcmp(method, "POST") == 0
      || strcmp(method, "PUT") == 0
      || strcmp(method, "PATCH") == 0
      || strcmp(method, "DELETE") == 0;
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
    case 401:
      return "Unauthorized";
    case 200:
      return "OK";
    case 201:
      return "Created";
    case 204:
      return "No Content";
    case 400:
      return "Bad Request";
    case 403:
      return "Forbidden";
    case 413:
      return "Payload Too Large";
    case 415:
      return "Unsupported Media Type";
    case 404:
      return "Not Found";
    case 405:
      return "Method Not Allowed";
    case 409:
      return "Conflict";
    case 429:
      return "Too Many Requests";
    case 501:
      return "Not Implemented";
    case 502:
      return "Bad Gateway";
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

static int sec4_rt_send_response_with_extra_headers(
    int socket_fd,
    int64_t status,
    const char *content_type,
    const char *body,
    size_t body_len,
    const char *extra_headers
) {
  if (content_type == NULL || content_type[0] == '\0') {
    content_type = "text/plain; charset=utf-8";
  }

  if (extra_headers == NULL) {
    extra_headers = "";
  }

  char header[640];
  int header_len = snprintf(
      header,
      sizeof(header),
      "HTTP/1.1 %lld %s\r\n"
      "Content-Type: %s\r\n"
      "Content-Length: %zu\r\n"
      "X-Trace-Id: %s\r\n"
      "%s"
      "Connection: close\r\n"
      "\r\n",
      (long long) status,
      sec4_rt_status_text(status),
      content_type,
      body_len,
      sec4_rt_current_trace_id(),
      extra_headers
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

static int sec4_rt_send_response(
    int socket_fd,
    int64_t status,
    const char *content_type,
    const char *body,
    size_t body_len
) {
  return sec4_rt_send_response_with_extra_headers(
      socket_fd,
      status,
      content_type,
      body,
      body_len,
      NULL
  );
}

static void sec4_rt_collect_allow_methods(
    sec4_rt_router_state *router,
    const char *path,
    char *allow,
    size_t allow_size
) {
  if (allow_size == 0) {
    return;
  }
  allow[0] = '\0';

  size_t used = 0;
  for (size_t i = 0; i < router->route_count; i++) {
    sec4_rt_route *candidate = &router->routes[i];
    if (!sec4_rt_path_pattern_matches(candidate->path, path)) {
      continue;
    }
    if (strstr(allow, candidate->method) != NULL) {
      continue;
    }

    const char *prefix = used == 0 ? "" : ", ";
    int written = snprintf(
        allow + used,
        allow_size - used,
        "%s%s",
        prefix,
        candidate->method
    );
    if (written <= 0 || (size_t) written >= allow_size - used) {
      break;
    }
    used += (size_t) written;
  }
}

static const char *sec4_rt_cors_headers_block(
    sec4_rt_router_state *router,
    char *buffer,
    size_t buffer_size
) {
  if (router == NULL || !router->cors_enabled || buffer == NULL || buffer_size == 0) {
    return NULL;
  }

  const char *allow_origin = router->cors_allow_origin[0] != '\0'
      ? router->cors_allow_origin
      : "*";
  int written = snprintf(
      buffer,
      buffer_size,
      "Access-Control-Allow-Origin: %s\r\n",
      allow_origin
  );
  if (written <= 0 || (size_t) written >= buffer_size) {
    return NULL;
  }

  size_t used = (size_t) written;
  if (router->cors_allow_credentials) {
    written = snprintf(
        buffer + used,
        buffer_size - used,
        "Access-Control-Allow-Credentials: true\r\n"
    );
    if (written > 0 && (size_t) written < (buffer_size - used)) {
      used += (size_t) written;
    }
  }
  if (router->cors_require_vary_origin && strcmp(allow_origin, "*") != 0) {
    written = snprintf(
        buffer + used,
        buffer_size - used,
        "Vary: Origin\r\n"
    );
    if (written > 0 && (size_t) written < (buffer_size - used)) {
      used += (size_t) written;
    }
  }
  (void) used;
  return buffer;
}

static const char *sec4_rt_preflight_headers_block(
    sec4_rt_router_state *router,
    char *buffer,
    size_t buffer_size
) {
  if (router == NULL || !router->cors_enabled || buffer == NULL || buffer_size == 0) {
    return NULL;
  }

  const char *allow_origin = router->cors_allow_origin[0] != '\0'
      ? router->cors_allow_origin
      : "*";
  const char *allow_methods = router->cors_allow_methods[0] != '\0'
      ? router->cors_allow_methods
      : "GET, POST, PUT, PATCH, DELETE, OPTIONS";
  const char *allow_headers = router->cors_allow_headers[0] != '\0'
      ? router->cors_allow_headers
      : "content-type, authorization";
  int64_t max_age = router->cors_max_age_seconds > 0
      ? router->cors_max_age_seconds
      : 600;

  int written = snprintf(
      buffer,
      buffer_size,
      "Access-Control-Allow-Origin: %s\r\n"
      "Access-Control-Allow-Methods: %s\r\n"
      "Access-Control-Allow-Headers: %s\r\n"
      "Access-Control-Max-Age: %lld\r\n",
      allow_origin,
      allow_methods,
      allow_headers,
      (long long) max_age
  );
  if (written <= 0 || (size_t) written >= buffer_size) {
    return NULL;
  }

  size_t used = (size_t) written;
  if (router->cors_allow_credentials) {
    written = snprintf(
        buffer + used,
        buffer_size - used,
        "Access-Control-Allow-Credentials: true\r\n"
    );
    if (written > 0 && (size_t) written < (buffer_size - used)) {
      used += (size_t) written;
    }
  }
  if (router->cors_require_vary_origin && strcmp(allow_origin, "*") != 0) {
    written = snprintf(
        buffer + used,
        buffer_size - used,
        "Vary: Origin\r\n"
    );
    if (written > 0 && (size_t) written < (buffer_size - used)) {
      used += (size_t) written;
    }
  }
  (void) used;
  return buffer;
}

static const char *sec4_rt_security_headers_block(
    sec4_rt_router_state *router,
    char *buffer,
    size_t buffer_size
) {
  if (router == NULL || !router->security_headers_enabled || buffer == NULL || buffer_size == 0) {
    return NULL;
  }

  size_t used = 0;
  int written = 0;
  if (router->security_x_content_type_options) {
    written = snprintf(
        buffer + used,
        buffer_size - used,
        "X-Content-Type-Options: nosniff\r\n"
    );
    if (written <= 0 || (size_t) written >= buffer_size - used) {
      return used > 0 ? buffer : NULL;
    }
    used += (size_t) written;
  }

  if (router->security_x_frame_options[0] != '\0') {
    written = snprintf(
        buffer + used,
        buffer_size - used,
        "X-Frame-Options: %s\r\n",
        router->security_x_frame_options
    );
    if (written <= 0 || (size_t) written >= buffer_size - used) {
      return used > 0 ? buffer : NULL;
    }
    used += (size_t) written;
  }

  if (router->security_referrer_policy[0] != '\0') {
    written = snprintf(
        buffer + used,
        buffer_size - used,
        "Referrer-Policy: %s\r\n",
        router->security_referrer_policy
    );
    if (written <= 0 || (size_t) written >= buffer_size - used) {
      return used > 0 ? buffer : NULL;
    }
    used += (size_t) written;
  }

  return used > 0 ? buffer : NULL;
}

static const char *sec4_rt_merge_three_headers(
    const char *first,
    const char *second,
    const char *third,
    char *buffer,
    size_t buffer_size
) {
  const char *a = first != NULL ? first : "";
  const char *b = second != NULL ? second : "";
  const char *c = third != NULL ? third : "";
  if (a[0] == '\0' && b[0] == '\0' && c[0] == '\0') {
    return NULL;
  }
  if (buffer == NULL || buffer_size == 0) {
    if (a[0] != '\0') {
      return a;
    }
    if (b[0] != '\0') {
      return b;
    }
    return c;
  }

  int written = snprintf(buffer, buffer_size, "%s%s%s", a, b, c);
  if (written <= 0 || (size_t) written >= buffer_size) {
    if (a[0] != '\0') {
      return a;
    }
    if (b[0] != '\0') {
      return b;
    }
    return c;
  }
  return buffer;
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

static bool sec4_rt_env_flag_enabled_default(const char *name, bool fallback) {
  const char *raw = getenv(name);
  if (raw == NULL || raw[0] == '\0') {
    return fallback;
  }
  if (strcmp(raw, "1") == 0
      || strcasecmp(raw, "true") == 0
      || strcasecmp(raw, "yes") == 0
      || strcasecmp(raw, "on") == 0
      || strcasecmp(raw, "allow") == 0) {
    return true;
  }
  if (strcmp(raw, "0") == 0
      || strcasecmp(raw, "false") == 0
      || strcasecmp(raw, "no") == 0
      || strcasecmp(raw, "off") == 0
      || strcasecmp(raw, "deny") == 0) {
    return false;
  }
  return fallback;
}

static bool sec4_rt_env_flag_enabled(const char *name) {
  return sec4_rt_env_flag_enabled_default(name, false);
}

static void sec4_rt_read_env_string(
    const char *name,
    const char *fallback,
    char *out,
    size_t out_size
) {
  if (out == NULL || out_size == 0) {
    return;
  }
  out[0] = '\0';

  const char *value = getenv(name);
  if (value == NULL || value[0] == '\0') {
    value = fallback;
  }
  if (value == NULL || value[0] == '\0') {
    return;
  }

  strncpy(out, value, out_size - 1);
  out[out_size - 1] = '\0';
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
  sec4_rt_assign_trace_id();
  sec4_rt_reset_response();
  char security_headers_buffer[384];
  char cors_headers_buffer[384];
  char preflight_headers_buffer[512];
  const char *security_headers = sec4_rt_security_headers_block(
      router,
      security_headers_buffer,
      sizeof(security_headers_buffer)
  );
  const char *cors_headers = sec4_rt_cors_headers_block(
      router,
      cors_headers_buffer,
      sizeof(cors_headers_buffer)
  );
  char merged_headers[320];

  char request[SEC4_RT_REQUEST_BUFFER_BYTES];
  size_t total_bytes = 0;
  ssize_t bytes_read = recv(socket_fd, request, sizeof(request) - 1, 0);
  if (bytes_read <= 0) {
    return;
  }
  total_bytes = (size_t) bytes_read;
  request[total_bytes] = '\0';

  char *headers_end = strstr(request, "\r\n\r\n");
  size_t headers_len = 0;
  if (headers_end != NULL) {
    headers_len = (size_t) (headers_end - request) + 4;
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
    const char *final_headers = sec4_rt_merge_three_headers(
        NULL,
        cors_headers,
        security_headers,
        merged_headers,
        sizeof(merged_headers)
    );
    (void) sec4_rt_send_response_with_extra_headers(
        socket_fd,
        400,
        "text/plain; charset=utf-8",
        body,
        strlen(body),
        final_headers
    );
    return;
  }

  char request_target[SEC4_RT_MAX_PATH_BYTES];
  strncpy(request_target, path, sizeof(request_target) - 1);
  request_target[sizeof(request_target) - 1] = '\0';

  char *query_start = strchr(path, '?');
  if (query_start != NULL) {
    *query_start = '\0';
  }

  headers_end = strstr(request, "\r\n\r\n");
  if (headers_end != NULL) {
    headers_len = (size_t) (headers_end - request) + 4;
    size_t content_length = sec4_rt_parse_content_length(request, headers_len);
    bool has_content_type = false;
    bool content_type_is_json = sec4_rt_parse_content_type_is_json(
        request,
        headers_len,
        &has_content_type
    );
    size_t available_body = total_bytes > headers_len ? total_bytes - headers_len : 0;
    size_t body_cap = sizeof(g_sec4_rt_request.body) - 1;
    int64_t configured_body_cap = sec4_rt_parse_env_i64(
        "SEC4_RT_HTTP_MAX_BODY_BYTES",
        (int64_t) body_cap
    );
    if (configured_body_cap > 0 && (size_t) configured_body_cap < body_cap) {
      body_cap = (size_t) configured_body_cap;
    }
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
    size_t headers_copy_len = headers_len;
    if (headers_copy_len >= sizeof(g_sec4_rt_request.raw_headers)) {
      headers_copy_len = sizeof(g_sec4_rt_request.raw_headers) - 1;
    }
    memcpy(g_sec4_rt_request.raw_headers, request, headers_copy_len);
    g_sec4_rt_request.raw_headers[headers_copy_len] = '\0';
    g_sec4_rt_request.raw_headers_len = headers_copy_len;
  }

  g_sec4_rt_request.has_request = true;
  strncpy(g_sec4_rt_request.method, method, sizeof(g_sec4_rt_request.method) - 1);
  g_sec4_rt_request.method[sizeof(g_sec4_rt_request.method) - 1] = '\0';
  strncpy(g_sec4_rt_request.path, request_target, sizeof(g_sec4_rt_request.path) - 1);
  g_sec4_rt_request.path[sizeof(g_sec4_rt_request.path) - 1] = '\0';
  strncpy(g_sec4_rt_request.route_path, path, sizeof(g_sec4_rt_request.route_path) - 1);
  g_sec4_rt_request.route_path[sizeof(g_sec4_rt_request.route_path) - 1] = '\0';

  if (router->cors_enabled && strcmp(method, "OPTIONS") == 0) {
    const char *extra_headers = sec4_rt_preflight_headers_block(
        router,
        preflight_headers_buffer,
        sizeof(preflight_headers_buffer)
    );
    const char *final_headers = sec4_rt_merge_three_headers(
        extra_headers,
        NULL,
        security_headers,
        merged_headers,
        sizeof(merged_headers)
    );
    (void) sec4_rt_send_response_with_extra_headers(
        socket_fd,
        204,
        "text/plain; charset=utf-8",
        "",
        0,
        final_headers
    );
    return;
  }

  if (router->auth_enabled && strcmp(method, "OPTIONS") != 0) {
    const char *auth_mode = sec4_rt_effective_auth_mode(router);
    char auth_subject[256];
    if (!sec4_rt_auth_collect_subject(
            auth_mode,
            auth_subject,
            sizeof(auth_subject),
            NULL
        )) {
      sec4_rt_store_std_error_response(
          401,
          "AUTH.UNAUTHORIZED",
          "auth",
          sec4_rt_auth_unauthorized_message(auth_mode)
      );
      const char *final_headers = sec4_rt_merge_three_headers(
          NULL,
          cors_headers,
          security_headers,
          merged_headers,
          sizeof(merged_headers)
      );
      (void) sec4_rt_send_response_with_extra_headers(
          socket_fd,
          g_sec4_rt_response.status,
          g_sec4_rt_response.content_type,
          g_sec4_rt_response.body,
          g_sec4_rt_response.body_len,
          final_headers
      );
      return;
    }
  }

  if (router->csrf_enabled && sec4_rt_is_csrf_protected_method(router, method)) {
    char csrf_header[128];
    char cookie_header[512];
    char csrf_cookie[128];
    bool has_csrf_header = sec4_rt_parse_header_value(
        request,
        headers_len,
        "X-CSRF-Token",
        csrf_header,
        sizeof(csrf_header)
    );
    bool has_cookie_header = sec4_rt_parse_header_value(
        request,
        headers_len,
        "Cookie",
        cookie_header,
        sizeof(cookie_header)
    );
    bool has_csrf_cookie = has_cookie_header
        && sec4_rt_parse_cookie_value(cookie_header, "csrf", csrf_cookie, sizeof(csrf_cookie));

    if (!has_csrf_header || !has_csrf_cookie || strcmp(csrf_header, csrf_cookie) != 0) {
      sec4_rt_store_std_error_response(
          403,
          "AUTH.CSRF_TOKEN_INVALID",
          "auth",
          "CSRF token missing or invalid"
      );
      const char *final_headers = sec4_rt_merge_three_headers(
          NULL,
          cors_headers,
          security_headers,
          merged_headers,
          sizeof(merged_headers)
      );
      (void) sec4_rt_send_response_with_extra_headers(
          socket_fd,
          g_sec4_rt_response.status,
          g_sec4_rt_response.content_type,
          g_sec4_rt_response.body,
          g_sec4_rt_response.body_len,
          final_headers
      );
      return;
    }
  }

  sec4_rt_route *match = NULL;
  sec4_rt_route *method_mismatch = NULL;
  for (size_t i = 0; i < router->route_count; i++) {
    sec4_rt_route *candidate = &router->routes[i];
    if (!sec4_rt_path_pattern_matches(candidate->path, path)) {
      continue;
    }
    if (strcmp(candidate->method, method) == 0) {
      match = candidate;
      break;
    }
    if (method_mismatch == NULL) {
      method_mismatch = candidate;
    }
  }

  if (match == NULL) {
    if (method_mismatch != NULL) {
      const char *body = "method not allowed";
      char allow_methods[64];
      char extra_headers[96];
      const char *allow_headers = NULL;
      sec4_rt_collect_allow_methods(router, path, allow_methods, sizeof(allow_methods));
      if (allow_methods[0] != '\0') {
        int extra_len = snprintf(extra_headers, sizeof(extra_headers), "Allow: %s\r\n", allow_methods);
        if (extra_len > 0 && (size_t) extra_len < sizeof(extra_headers)) {
          allow_headers = extra_headers;
        }
      }
      const char *final_headers = sec4_rt_merge_three_headers(
          allow_headers,
          cors_headers,
          security_headers,
          merged_headers,
          sizeof(merged_headers)
      );

      (void) sec4_rt_send_response_with_extra_headers(
          socket_fd,
          405,
          "text/plain; charset=utf-8",
          body,
          strlen(body),
          final_headers
      );
      return;
    }

    const char *body = "not found";
    const char *final_headers = sec4_rt_merge_three_headers(
        NULL,
        cors_headers,
        security_headers,
        merged_headers,
        sizeof(merged_headers)
    );
    (void) sec4_rt_send_response_with_extra_headers(
        socket_fd,
        404,
        "text/plain; charset=utf-8",
        body,
        strlen(body),
        final_headers
    );
    return;
  }

  sec4_rt_reset_response();
  strncpy(
      g_sec4_rt_request.matched_route_pattern,
      match->path,
      sizeof(g_sec4_rt_request.matched_route_pattern) - 1
  );
  g_sec4_rt_request.matched_route_pattern[sizeof(g_sec4_rt_request.matched_route_pattern) - 1] = '\0';
  (void) match->handler();

  if (!g_sec4_rt_response.active) {
    const char *body = "";
    const char *final_headers = sec4_rt_merge_three_headers(
        sec4_rt_response_extra_headers(),
        cors_headers,
        security_headers,
        merged_headers,
        sizeof(merged_headers)
    );
    (void) sec4_rt_send_response_with_extra_headers(
        socket_fd,
        204,
        "text/plain; charset=utf-8",
        body,
        0,
        final_headers
    );
    return;
  }

  const char *final_headers = sec4_rt_merge_three_headers(
      sec4_rt_response_extra_headers(),
      cors_headers,
      security_headers,
      merged_headers,
      sizeof(merged_headers)
  );
  (void) sec4_rt_send_response_with_extra_headers(
      socket_fd,
      g_sec4_rt_response.status,
      g_sec4_rt_response.content_type,
      g_sec4_rt_response.body,
      g_sec4_rt_response.body_len,
      final_headers
  );
}

int64_t sec4_rt_identity_i64(int64_t value) {
  return value;
}

bool sec4_rt_identity_bool(bool value) {
  return value;
}

int64_t sec4_rt_time_now(void) {
  struct timeval tv;
  if (gettimeofday(&tv, NULL) == 0) {
    int64_t ms = ((int64_t) tv.tv_sec * 1000) + ((int64_t) tv.tv_usec / 1000);
    if (ms > 0) {
      return ms;
    }
  }

  time_t now = time(NULL);
  if (now > 0) {
    return (int64_t) now * 1000;
  }
  return 1;
}

static size_t sec4_rt_json_escape(
    const char *input,
    char *output,
    size_t output_size
) {
  if (output == NULL || output_size == 0) {
    return 0;
  }
  output[0] = '\0';
  if (input == NULL) {
    return 0;
  }

  size_t used = 0;
  for (size_t i = 0; input[i] != '\0'; i++) {
    unsigned char ch = (unsigned char) input[i];
    const char *replacement = NULL;
    char replacement_buf[7];
    switch (ch) {
      case '\\':
        replacement = "\\\\";
        break;
      case '"':
        replacement = "\\\"";
        break;
      case '\n':
        replacement = "\\n";
        break;
      case '\r':
        replacement = "\\r";
        break;
      case '\t':
        replacement = "\\t";
        break;
      default:
        if (ch < 0x20) {
          (void) snprintf(replacement_buf, sizeof(replacement_buf), "\\u%04x", (unsigned int) ch);
          replacement = replacement_buf;
        }
        break;
    }

    if (replacement != NULL) {
      size_t replacement_len = strlen(replacement);
      if (used + replacement_len >= output_size) {
        break;
      }
      memcpy(output + used, replacement, replacement_len);
      used += replacement_len;
    } else {
      if (used + 1 >= output_size) {
        break;
      }
      output[used++] = (char) ch;
    }
  }
  output[used] = '\0';
  return used;
}

static sec4_rt_log_value_state *sec4_rt_log_value_state_for_handle(int64_t handle) {
  if (handle == 0) {
    return NULL;
  }
  for (size_t i = 0; i < SEC4_RT_MAX_LOG_VALUES; i++) {
    if (g_sec4_rt_log_values[i].active && g_sec4_rt_log_values[i].handle == handle) {
      return &g_sec4_rt_log_values[i];
    }
  }
  return NULL;
}

static int64_t sec4_rt_log_register_value(const char *json_value, uint64_t salt) {
  const char *resolved = json_value != NULL ? json_value : "null";
  int64_t handle = sec4_rt_nonzero_handle_from_string(resolved, salt);

  sec4_rt_log_value_state *slot = sec4_rt_log_value_state_for_handle(handle);
  if (slot == NULL) {
    for (size_t i = 0; i < SEC4_RT_MAX_LOG_VALUES; i++) {
      if (!g_sec4_rt_log_values[i].active) {
        slot = &g_sec4_rt_log_values[i];
        break;
      }
    }
  }
  if (slot == NULL) {
    slot = &g_sec4_rt_log_values[(size_t) ((uint64_t) handle % SEC4_RT_MAX_LOG_VALUES)];
  }

  memset(slot, 0, sizeof(*slot));
  slot->active = true;
  slot->handle = handle;
  strncpy(slot->json, resolved, sizeof(slot->json) - 1);
  slot->json[sizeof(slot->json) - 1] = '\0';
  return handle;
}

static bool sec4_rt_log_json_for_handle(
    int64_t handle,
    char *buffer,
    size_t buffer_size
) {
  if (buffer == NULL || buffer_size == 0) {
    return false;
  }
  buffer[0] = '\0';
  if (handle == 0) {
    strncpy(buffer, "null", buffer_size - 1);
    buffer[buffer_size - 1] = '\0';
    return true;
  }

  sec4_rt_log_value_state *slot = sec4_rt_log_value_state_for_handle(handle);
  if (slot != NULL) {
    strncpy(buffer, slot->json, buffer_size - 1);
    buffer[buffer_size - 1] = '\0';
    return true;
  }

  const char *tracked = sec4_rt_lookup_tracked_value(handle);
  if (tracked != NULL) {
    char escaped[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
    sec4_rt_json_escape(tracked, escaped, sizeof(escaped));
    (void) snprintf(buffer, buffer_size, "\"%s\"", escaped);
    return true;
  }

  (void) snprintf(buffer, buffer_size, "%lld", (long long) handle);
  return true;
}

static sec4_rt_log_event_state *sec4_rt_log_event_state_for_handle(int64_t handle) {
  if (handle == 0) {
    return NULL;
  }
  for (size_t i = 0; i < SEC4_RT_MAX_LOG_EVENTS; i++) {
    if (g_sec4_rt_log_events[i].active && g_sec4_rt_log_events[i].handle == handle) {
      return &g_sec4_rt_log_events[i];
    }
  }
  return NULL;
}

static void sec4_rt_log_store_event_state(const sec4_rt_log_event_state *state) {
  if (state == NULL || state->handle == 0) {
    return;
  }

  sec4_rt_log_event_state *slot = sec4_rt_log_event_state_for_handle(state->handle);
  if (slot == NULL) {
    for (size_t i = 0; i < SEC4_RT_MAX_LOG_EVENTS; i++) {
      if (!g_sec4_rt_log_events[i].active) {
        slot = &g_sec4_rt_log_events[i];
        break;
      }
    }
  }
  if (slot == NULL) {
    slot = &g_sec4_rt_log_events[(size_t) ((uint64_t) state->handle % SEC4_RT_MAX_LOG_EVENTS)];
  }

  memcpy(slot, state, sizeof(*slot));
  slot->active = true;
}

static void sec4_rt_log_load_event_state(
    int64_t handle,
    sec4_rt_log_event_state *out
) {
  if (out == NULL) {
    return;
  }
  memset(out, 0, sizeof(*out));
  out->active = true;
  out->handle = handle;

  sec4_rt_log_event_state *existing = sec4_rt_log_event_state_for_handle(handle);
  if (existing != NULL) {
    memcpy(out, existing, sizeof(*out));
    if (out->level[0] == '\0') {
      strncpy(out->level, "info", sizeof(out->level) - 1);
      out->level[sizeof(out->level) - 1] = '\0';
    }
    return;
  }

  const char *tracked = sec4_rt_lookup_tracked_value(handle);
  if (tracked != NULL && tracked[0] != '\0') {
    strncpy(out->event_name, tracked, sizeof(out->event_name) - 1);
  } else {
    strncpy(out->event_name, "runtime.event", sizeof(out->event_name) - 1);
  }
  out->event_name[sizeof(out->event_name) - 1] = '\0';
  strncpy(out->level, "info", sizeof(out->level) - 1);
  out->level[sizeof(out->level) - 1] = '\0';
}

static int64_t sec4_rt_log_register_event(const char *event_name) {
  const char *resolved = (event_name != NULL && event_name[0] != '\0')
      ? event_name
      : "runtime.event";
  int64_t handle = sec4_rt_nonzero_handle_from_string(resolved, UINT64_C(0xA1002));
  sec4_rt_log_event_state state;
  memset(&state, 0, sizeof(state));
  state.active = true;
  state.handle = handle;
  strncpy(state.event_name, resolved, sizeof(state.event_name) - 1);
  state.event_name[sizeof(state.event_name) - 1] = '\0';
  strncpy(state.level, "info", sizeof(state.level) - 1);
  state.level[sizeof(state.level) - 1] = '\0';
  state.attrs_json[0] = '\0';
  sec4_rt_log_store_event_state(&state);
  return handle;
}

static void sec4_rt_log_append_attr(
    sec4_rt_log_event_state *state,
    const char *key,
    int64_t value
) {
  if (state == NULL) {
    return;
  }
  char escaped_key[96];
  sec4_rt_json_escape(
      key != NULL && key[0] != '\0' ? key : "value",
      escaped_key,
      sizeof(escaped_key)
  );
  char value_json[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES];
  sec4_rt_log_json_for_handle(value, value_json, sizeof(value_json));

  if (state->attrs_json[0] == '\0') {
    (void) snprintf(
        state->attrs_json,
        sizeof(state->attrs_json),
        "{\"%s\":%s}",
        escaped_key,
        value_json
    );
    return;
  }

  size_t len = strlen(state->attrs_json);
  if (len < 2 || state->attrs_json[len - 1] != '}') {
    state->attrs_json[0] = '\0';
    (void) snprintf(
        state->attrs_json,
        sizeof(state->attrs_json),
        "{\"%s\":%s}",
        escaped_key,
        value_json
    );
    return;
  }

  state->attrs_json[len - 1] = '\0';
  int written = snprintf(
      state->attrs_json + (len - 1),
      sizeof(state->attrs_json) - (len - 1),
      ",\"%s\":%s}",
      escaped_key,
      value_json
  );
  if (written <= 0 || (size_t) written >= sizeof(state->attrs_json) - (len - 1)) {
    state->attrs_json[len - 1] = '}';
    state->attrs_json[len] = '\0';
  }
}

static void sec4_rt_log_emit_json(int64_t event_handle) {
  const char *log_output = getenv("SEC4_RT_LOG_OUTPUT");
  if (log_output != NULL && strcasecmp(log_output, "off") == 0) {
    return;
  }

  FILE *stream = stderr;
  if (log_output != NULL && strcasecmp(log_output, "stdout") == 0) {
    stream = stdout;
  }

  sec4_rt_log_event_state state;
  sec4_rt_log_load_event_state(event_handle, &state);
  const char *level = state.level[0] != '\0' ? state.level : "info";

  char event_name[SEC4_RT_MAX_LOG_EVENT_NAME_BYTES * 2];
  sec4_rt_json_escape(state.event_name, event_name, sizeof(event_name));
  char level_json[24];
  sec4_rt_json_escape(level, level_json, sizeof(level_json));
  char trace_id[64];
  sec4_rt_json_escape(sec4_rt_current_trace_id(), trace_id, sizeof(trace_id));

  char attrs_segment[SEC4_RT_MAX_LOG_ATTRS_BYTES + 16];
  attrs_segment[0] = '\0';
  if (state.attrs_json[0] != '\0') {
    (void) snprintf(attrs_segment, sizeof(attrs_segment), ",\"attrs\":%s", state.attrs_json);
  }

  char http_segment[512];
  http_segment[0] = '\0';
  if (state.has_http) {
    char method[24];
    char path[SEC4_RT_MAX_PATH_BYTES * 2];
    sec4_rt_json_escape(state.http_method, method, sizeof(method));
    sec4_rt_json_escape(state.http_path, path, sizeof(path));
    (void) snprintf(
        http_segment,
        sizeof(http_segment),
        ",\"http\":{\"method\":\"%s\",\"path\":\"%s\",\"status\":%lld,\"latencyMs\":%lld}",
        method,
        path,
        (long long) state.http_status,
        (long long) state.http_latency_ms
    );
  }

  char error_segment[SEC4_RT_MAX_RESPONSE_BYTES + 16];
  error_segment[0] = '\0';
  if (state.has_error) {
    sec4_rt_error_state error_state;
    sec4_rt_error_load_state(state.error_handle, &error_state);
    if (error_state.handle != 0 && sec4_rt_error_state_for_handle(state.error_handle) != NULL) {
      char error_json[SEC4_RT_MAX_RESPONSE_BYTES];
      sec4_rt_error_render_json(&error_state, error_json, sizeof(error_json));
      (void) snprintf(
          error_segment,
          sizeof(error_segment),
          ",\"error\":%s",
          error_json
      );
    } else {
      (void) snprintf(
          error_segment,
          sizeof(error_segment),
          ",\"error\":{\"handle\":%lld}",
          (long long) state.error_handle
      );
    }
  }

  char line[SEC4_RT_MAX_LOG_LINE_BYTES];
  int written = snprintf(
      line,
      sizeof(line),
      "{\"timeMs\":%lld,\"level\":\"%s\",\"traceId\":\"%s\",\"event\":\"%s\"%s%s%s}",
      (long long) sec4_rt_time_now(),
      level_json,
      trace_id,
      event_name,
      attrs_segment,
      http_segment,
      error_segment
  );
  if (written <= 0 || (size_t) written >= sizeof(line)) {
    (void) fprintf(
        stream,
        "{\"timeMs\":%lld,\"level\":\"%s\",\"traceId\":\"%s\",\"event\":\"runtime.log_overflow\"}\n",
        (long long) sec4_rt_time_now(),
        level_json,
        trace_id
    );
    (void) fflush(stream);
    return;
  }

  (void) fprintf(stream, "%s\n", line);
  (void) fflush(stream);
}

static const char *sec4_rt_log_resolve_level(const char *level) {
  if (level == NULL || level[0] == '\0') {
    return "info";
  }
  if (strcasecmp(level, "warn") == 0) {
    return "warn";
  }
  if (strcasecmp(level, "error") == 0) {
    return "error";
  }
  return "info";
}

static void sec4_rt_log_emit_with_level(int64_t event, const char *level) {
  int64_t resolved_event = event != 0
      ? event
      : sec4_rt_log_register_event("runtime.event");
  sec4_rt_log_event_state state;
  sec4_rt_log_load_event_state(resolved_event, &state);
  const char *resolved_level = sec4_rt_log_resolve_level(level);
  strncpy(state.level, resolved_level, sizeof(state.level) - 1);
  state.level[sizeof(state.level) - 1] = '\0';
  sec4_rt_log_store_event_state(&state);
  g_sec4_rt_last_log_handle = resolved_event;
  sec4_rt_log_emit_json(resolved_event);
}

void sec4_rt_log_any(int64_t event) {
  sec4_rt_log_emit_with_level(event, "info");
}

void sec4_rt_log_info(int64_t event) {
  sec4_rt_log_emit_with_level(event, "info");
}

void sec4_rt_log_warn(int64_t event) {
  sec4_rt_log_emit_with_level(event, "warn");
}

void sec4_rt_log_error(int64_t event) {
  sec4_rt_log_emit_with_level(event, "error");
}

int64_t sec4_rt_log_event(const char *event_name) {
  return sec4_rt_log_register_event(event_name);
}

int64_t sec4_rt_log_field(const char *key, int64_t value) {
  char escaped_key[96];
  sec4_rt_json_escape(
      key != NULL && key[0] != '\0' ? key : "field",
      escaped_key,
      sizeof(escaped_key)
  );
  char value_json[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES];
  sec4_rt_log_json_for_handle(value, value_json, sizeof(value_json));
  char field_json[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES];
  (void) snprintf(
      field_json,
      sizeof(field_json),
      "{\"%s\":%s}",
      escaped_key,
      value_json
  );
  return sec4_rt_log_register_value(field_json, UINT64_C(0xA1004));
}

int64_t sec4_rt_log_obj(int64_t field) {
  char field_json[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES];
  sec4_rt_log_json_for_handle(field, field_json, sizeof(field_json));
  if (field_json[0] != '{') {
    char wrapped_json[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES];
    (void) snprintf(wrapped_json, sizeof(wrapped_json), "{\"value\":%s}", field_json);
    return sec4_rt_log_register_value(wrapped_json, UINT64_C(0xA1005));
  }
  return sec4_rt_log_register_value(field_json, UINT64_C(0xA1005));
}

int64_t sec4_rt_log_str(const char *value) {
  char escaped[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  sec4_rt_json_escape(value != NULL ? value : "", escaped, sizeof(escaped));
  char json_value[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES];
  (void) snprintf(json_value, sizeof(json_value), "\"%s\"", escaped);
  return sec4_rt_log_register_value(json_value, UINT64_C(0xA1006));
}

int64_t sec4_rt_log_i64(int64_t value) {
  char json_value[32];
  (void) snprintf(json_value, sizeof(json_value), "%lld", (long long) value);
  return sec4_rt_log_register_value(json_value, UINT64_C(0xA1007));
}

int64_t sec4_rt_log_bool(int64_t value) {
  return sec4_rt_log_register_value(
      value != 0 ? "true" : "false",
      UINT64_C(0xA1008)
  );
}

int64_t sec4_rt_log_redacted(const char *value) {
  char escaped[96];
  sec4_rt_json_escape(value != NULL ? value : "secret", escaped, sizeof(escaped));
  char json_value[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES];
  (void) snprintf(
      json_value,
      sizeof(json_value),
      "{\"redacted\":\"%s\"}",
      escaped
  );
  return sec4_rt_log_register_value(json_value, UINT64_C(0xA1009));
}

int64_t sec4_rt_log_attr_redacted(const char *value) {
  char escaped[96];
  sec4_rt_json_escape(value != NULL ? value : "secret", escaped, sizeof(escaped));
  char json_value[SEC4_RT_MAX_LOG_JSON_VALUE_BYTES];
  (void) snprintf(
      json_value,
      sizeof(json_value),
      "{\"redacted\":\"%s\"}",
      escaped
  );
  return sec4_rt_log_register_value(json_value, UINT64_C(0xA100A));
}

int64_t sec4_rt_log_with_attr(int64_t event, const char *key, int64_t value) {
  sec4_rt_log_event_state state;
  sec4_rt_log_load_event_state(event, &state);
  sec4_rt_log_append_attr(&state, key, value);

  int64_t key_handle = sec4_rt_nonzero_handle_from_string(key, UINT64_C(0xA100B));
  int64_t attr_handle = sec4_rt_handle_from_two(key_handle, value, UINT64_C(0xA100C));
  int64_t next_event = sec4_rt_handle_from_two(event, attr_handle, UINT64_C(0xA100D));
  state.handle = next_event;
  sec4_rt_log_store_event_state(&state);
  return next_event;
}

int64_t sec4_rt_log_with_http(
    int64_t event,
    const char *method,
    const char *path,
    int64_t status,
    int64_t duration_ms
) {
  sec4_rt_log_event_state state;
  sec4_rt_log_load_event_state(event, &state);
  state.has_http = true;
  strncpy(state.http_method, method != NULL ? method : "", sizeof(state.http_method) - 1);
  state.http_method[sizeof(state.http_method) - 1] = '\0';
  strncpy(state.http_path, path != NULL ? path : "", sizeof(state.http_path) - 1);
  state.http_path[sizeof(state.http_path) - 1] = '\0';
  state.http_status = status;
  state.http_latency_ms = duration_ms;

  int64_t method_handle = sec4_rt_nonzero_handle_from_string(method, UINT64_C(0xA100E));
  int64_t path_handle = sec4_rt_nonzero_handle_from_string(path, UINT64_C(0xA100F));
  int64_t route_handle = sec4_rt_handle_from_two(method_handle, path_handle, UINT64_C(0xA1010));
  int64_t http_meta = sec4_rt_handle_from_two(status, duration_ms, UINT64_C(0xA1011));
  int64_t http_handle = sec4_rt_handle_from_two(route_handle, http_meta, UINT64_C(0xA1012));
  int64_t next_event = sec4_rt_handle_from_two(event, http_handle, UINT64_C(0xA1013));
  state.handle = next_event;
  sec4_rt_log_store_event_state(&state);
  return next_event;
}

int64_t sec4_rt_log_with_error(int64_t event, int64_t error) {
  sec4_rt_log_event_state state;
  sec4_rt_log_load_event_state(event, &state);
  state.has_error = true;
  state.error_handle = error;

  int64_t next_event = sec4_rt_handle_from_two(event, error, UINT64_C(0xA1014));
  state.handle = next_event;
  sec4_rt_log_store_event_state(&state);
  return next_event;
}

int64_t sec4_rt_req_json(int64_t schema) {
  g_sec4_rt_request.json_checked = true;
  g_sec4_rt_request.json_schema_handle = schema;

  if (schema == 0) {
    g_sec4_rt_request.json_valid = false;
    sec4_rt_store_std_error_response(
        400,
        "JSON.SCHEMA_INVALID",
        "validation",
        "req.json requires non-zero schema descriptor"
    );
    return 1;
  }

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

  if (g_sec4_rt_request.body_len > sec4_rt_json_max_bytes_limit()) {
    g_sec4_rt_request.json_valid = false;
    sec4_rt_store_std_error_response(
        413,
        "JSON.SIZE_LIMIT",
        "resource_limit",
        "json payload exceeds runtime size limit"
    );
    return 1;
  }

  if (sec4_rt_json_exceeds_depth(
          g_sec4_rt_request.body,
          g_sec4_rt_request.body_len,
          sec4_rt_json_max_depth_limit()
      )) {
    g_sec4_rt_request.json_valid = false;
    sec4_rt_store_std_error_response(
        400,
        "JSON.DEPTH_LIMIT",
        "validation",
        "json payload exceeds runtime depth limit"
    );
    return 1;
  }

  g_sec4_rt_request.json_valid = true;
  return 0;
}

int64_t sec4_rt_json_decode(int64_t ctx, int64_t schema, int64_t raw) {
  if (ctx == 0 || raw == 0) {
    sec4_rt_store_std_error_response(
        400,
        "JSON.DECODE_INVALID",
        "validation",
        "invalid json.decode input"
    );
    return 0;
  }

  if (schema == 0) {
    sec4_rt_store_std_error_response(
        400,
        "JSON.SCHEMA_INVALID",
        "validation",
        "json.decode requires non-zero schema descriptor"
    );
    return 0;
  }

  const char *raw_value = sec4_rt_lookup_tracked_value(raw);
  size_t raw_len = raw_value != NULL ? strlen(raw_value) : 0;
  if (raw_value == NULL || raw_len == 0 || !sec4_rt_is_likely_json(raw_value, raw_len)) {
    sec4_rt_store_std_error_response(
        400,
        "JSON.DECODE_INVALID",
        "validation",
        "invalid json.decode input"
    );
    return 0;
  }

  if (g_sec4_rt_request.has_request) {
    bool raw_matches_request_body = g_sec4_rt_request.body_len > 0
        && raw_len == g_sec4_rt_request.body_len
        && memcmp(raw_value, g_sec4_rt_request.body, raw_len) == 0;

    if (raw_matches_request_body && !g_sec4_rt_request.json_checked) {
      sec4_rt_store_std_error_response(
          400,
          "JSON.GATE_REQUIRED",
          "validation",
          "req.json(schema) must succeed before json.decode on request body"
      );
      return 0;
    }

    if (g_sec4_rt_request.json_checked && !g_sec4_rt_request.json_valid) {
      sec4_rt_store_std_error_response(
          400,
          "JSON.GATE_FAILED",
          "validation",
          "json.decode blocked because req.json gate is invalid"
      );
      return 0;
    }

    if (g_sec4_rt_request.json_checked
        && g_sec4_rt_request.json_valid
        && g_sec4_rt_request.json_schema_handle != 0
        && g_sec4_rt_request.json_schema_handle != schema) {
      const char *request_schema =
          sec4_rt_lookup_tracked_value(g_sec4_rt_request.json_schema_handle);
      const char *decode_schema = sec4_rt_lookup_tracked_value(schema);
      if (request_schema != NULL && decode_schema != NULL && strcmp(request_schema, decode_schema) != 0) {
        sec4_rt_store_std_error_response(
            400,
            "JSON.SCHEMA_MISMATCH",
            "validation",
            "json.decode schema does not match req.json schema for active request"
        );
        return 0;
      }
    }
  }

  if (raw_len > sec4_rt_json_max_bytes_limit()) {
    sec4_rt_store_std_error_response(
        413,
        "JSON.SIZE_LIMIT",
        "resource_limit",
        "json payload exceeds runtime size limit"
    );
    return 0;
  }

  if (sec4_rt_json_exceeds_depth(raw_value, raw_len, sec4_rt_json_max_depth_limit())) {
    sec4_rt_store_std_error_response(
        400,
        "JSON.DEPTH_LIMIT",
        "validation",
        "json payload exceeds runtime depth limit"
    );
    return 0;
  }

  int64_t decoded = sec4_rt_track_string_value(raw_value, UINT64_C(0x13131));
  if (decoded == 0) {
    sec4_rt_store_std_error_response(
        500,
        "JSON.DECODE_INTERNAL",
        "internal",
        "json.decode runtime failure"
    );
    return 0;
  }
  return decoded;
}

int64_t sec4_rt_json_encode(int64_t schema, int64_t value) {
  if (schema == 0) {
    sec4_rt_store_std_error_response(
        400,
        "JSON.SCHEMA_INVALID",
        "validation",
        "json.encode requires non-zero schema descriptor"
    );
    return 0;
  }

  char encoded[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  size_t encoded_len = 0;
  if (!sec4_rt_render_json_fragment_from_value(value, encoded, sizeof(encoded), &encoded_len)) {
    sec4_rt_store_std_error_response(
        500,
        "JSON.ENCODE_INTERNAL",
        "internal",
        "json.encode runtime failure"
    );
    return 0;
  }

  int64_t encoded_handle = sec4_rt_track_sized_value(encoded, encoded_len, UINT64_C(0x14141));
  if (encoded_handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "JSON.ENCODE_INTERNAL",
        "internal",
        "json.encode runtime failure"
    );
    return 0;
  }
  return encoded_handle;
}

int64_t sec4_rt_req_body(int64_t ctx, int64_t req) {
  (void) ctx;
  (void) req;
  if (!g_sec4_rt_request.has_request || g_sec4_rt_request.body_len == 0) {
    return 0;
  }
  return sec4_rt_track_string_value(g_sec4_rt_request.body, UINT64_C(0x11111));
}

int64_t sec4_rt_req_query(const char *name) {
  char extracted[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  const char *value = name;
  if (g_sec4_rt_request.has_request
      && sec4_rt_extract_query_value(
          g_sec4_rt_request.path,
          name,
          extracted,
          sizeof(extracted)
      )) {
    value = extracted;
  }
  return sec4_rt_track_string_value(value, UINT64_C(0x10101));
}

int64_t sec4_rt_req_path_param(const char *name) {
  char extracted[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  const char *value = name;
  if (g_sec4_rt_request.has_request
      && sec4_rt_extract_path_param(
          g_sec4_rt_request.matched_route_pattern,
          g_sec4_rt_request.route_path,
          name,
          extracted,
          sizeof(extracted)
      )) {
    value = extracted;
  }
  return sec4_rt_track_string_value(value, UINT64_C(0x20202));
}

int64_t sec4_rt_req_header(const char *name) {
  char extracted[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  const char *value = name;
  if (g_sec4_rt_request.has_request
      && g_sec4_rt_request.raw_headers_len > 0
      && sec4_rt_parse_header_value(
          g_sec4_rt_request.raw_headers,
          g_sec4_rt_request.raw_headers_len,
          name,
          extracted,
          sizeof(extracted)
      )) {
    value = extracted;
  }
  return sec4_rt_track_string_value(value, UINT64_C(0x30303));
}

int64_t sec4_rt_res_json(int64_t schema, int64_t value) {
  if (g_sec4_rt_request.json_checked && !g_sec4_rt_request.json_valid) {
    return 1;
  }

  int64_t encoded = sec4_rt_json_encode(schema, value);
  if (encoded == 0) {
    return 1;
  }
  const char *data_json = sec4_rt_lookup_tracked_value(encoded);
  if (data_json == NULL) {
    sec4_rt_store_std_error_response(
        500,
        "JSON.ENCODE_INTERNAL",
        "internal",
        "res.json failed to materialize encoded payload"
    );
    return 1;
  }
  sec4_rt_store_std_success_response(200, data_json, NULL);
  return 0;
}

int64_t sec4_rt_res_ok(int64_t status, int64_t schema, int64_t value) {
  if (g_sec4_rt_request.json_checked && !g_sec4_rt_request.json_valid) {
    return 1;
  }

  int64_t encoded = sec4_rt_json_encode(schema, value);
  if (encoded == 0) {
    return 1;
  }
  const char *data_json = sec4_rt_lookup_tracked_value(encoded);
  if (data_json == NULL) {
    sec4_rt_store_std_error_response(
        500,
        "JSON.ENCODE_INTERNAL",
        "internal",
        "res.ok failed to materialize encoded payload"
    );
    return 1;
  }
  sec4_rt_store_std_success_response(status > 0 ? status : 201, data_json, NULL);
  return 0;
}

int64_t sec4_rt_res_ok_meta(int64_t status, int64_t schema, int64_t value, int64_t meta) {
  if (g_sec4_rt_request.json_checked && !g_sec4_rt_request.json_valid) {
    return 1;
  }

  int64_t encoded = sec4_rt_json_encode(schema, value);
  if (encoded == 0) {
    return 1;
  }
  const char *data_json = sec4_rt_lookup_tracked_value(encoded);
  if (data_json == NULL) {
    sec4_rt_store_std_error_response(
        500,
        "JSON.ENCODE_INTERNAL",
        "internal",
        "res.okMeta failed to materialize encoded payload"
    );
    return 1;
  }

  char meta_json[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  size_t meta_json_len = 0;
  if (!sec4_rt_render_json_fragment_from_value(meta, meta_json, sizeof(meta_json), &meta_json_len)) {
    sec4_rt_store_std_error_response(
        500,
        "JSON.ENCODE_INTERNAL",
        "internal",
        "res.okMeta failed to materialize meta payload"
    );
    return 1;
  }
  (void) meta_json_len;
  sec4_rt_store_std_success_response(status > 0 ? status : 201, data_json, meta_json);
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

int64_t sec4_rt_set_header(int64_t name, int64_t value) {
  const char *header_name = sec4_rt_lookup_tracked_value(name);
  const char *header_value = sec4_rt_lookup_tracked_value(value);
  if (header_name == NULL || header_value == NULL) {
    return 1;
  }
  return sec4_rt_append_response_header(header_name, header_value);
}

int64_t sec4_rt_cookie_build(const char *name, const char *value) {
  if (!sec4_rt_is_header_name_valid(name) || !sec4_rt_is_header_value_valid(value)) {
    return 0;
  }

  char cookie[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  int written = snprintf(cookie, sizeof(cookie), "%s=%s", name, value);
  if (written <= 0 || (size_t) written >= sizeof(cookie)) {
    return 0;
  }
  return sec4_rt_track_string_value(cookie, UINT64_C(0x12121));
}

int64_t sec4_rt_set_cookie(int64_t cookie) {
  const char *cookie_value = sec4_rt_lookup_tracked_value(cookie);
  if (cookie_value == NULL) {
    return 1;
  }
  return sec4_rt_append_response_header("Set-Cookie", cookie_value);
}

static bool sec4_rt_parse_i64_strict(const char *raw, int64_t *value_out) {
  if (raw == NULL || raw[0] == '\0' || value_out == NULL) {
    return false;
  }

  errno = 0;
  char *end = NULL;
  long long parsed = strtoll(raw, &end, 10);
  if (errno != 0 || end == raw || (end != NULL && *end != '\0')) {
    return false;
  }

  *value_out = (int64_t) parsed;
  return true;
}

static const sec4_rt_db_query_state *sec4_rt_db_lookup_query_state(int64_t query_handle) {
  if (query_handle == 0) {
    return NULL;
  }

  for (size_t i = 0; i < SEC4_RT_MAX_DB_QUERIES; i++) {
    const sec4_rt_db_query_state *slot = &g_sec4_rt_db_queries[i];
    if (slot->active && slot->query_handle == query_handle) {
      return slot;
    }
  }
  return NULL;
}

static bool sec4_rt_db_register_query_state(
    int64_t query_handle,
    int64_t template_handle,
    int64_t params
) {
  if (query_handle == 0 || template_handle == 0) {
    return false;
  }

  size_t first_free = SEC4_RT_MAX_DB_QUERIES;
  size_t target = SEC4_RT_MAX_DB_QUERIES;
  for (size_t i = 0; i < SEC4_RT_MAX_DB_QUERIES; i++) {
    sec4_rt_db_query_state *slot = &g_sec4_rt_db_queries[i];
    if (!slot->active) {
      if (first_free == SEC4_RT_MAX_DB_QUERIES) {
        first_free = i;
      }
      continue;
    }
    if (slot->query_handle == query_handle) {
      target = i;
      break;
    }
  }

  if (target == SEC4_RT_MAX_DB_QUERIES) {
    if (first_free != SEC4_RT_MAX_DB_QUERIES) {
      target = first_free;
    } else {
      target = (size_t) (((uint64_t) query_handle) % SEC4_RT_MAX_DB_QUERIES);
    }
  }

  sec4_rt_db_query_state *slot = &g_sec4_rt_db_queries[target];
  slot->active = true;
  slot->query_handle = query_handle;
  slot->template_handle = template_handle;
  slot->params = params;
  return true;
}

static bool sec4_rt_db_register_tx_state(int64_t tx_handle, int64_t db_handle) {
  if (tx_handle == 0 || db_handle == 0) {
    return false;
  }

  size_t first_free = SEC4_RT_MAX_DB_TXS;
  size_t target = SEC4_RT_MAX_DB_TXS;
  for (size_t i = 0; i < SEC4_RT_MAX_DB_TXS; i++) {
    sec4_rt_db_tx_state *slot = &g_sec4_rt_db_txs[i];
    if (!slot->active) {
      if (first_free == SEC4_RT_MAX_DB_TXS) {
        first_free = i;
      }
      continue;
    }
    if (slot->tx_handle == tx_handle) {
      target = i;
      break;
    }
  }

  if (target == SEC4_RT_MAX_DB_TXS) {
    if (first_free != SEC4_RT_MAX_DB_TXS) {
      target = first_free;
    } else {
      target = (size_t) (((uint64_t) tx_handle) % SEC4_RT_MAX_DB_TXS);
    }
  }

  sec4_rt_db_tx_state *slot = &g_sec4_rt_db_txs[target];
  slot->active = true;
  slot->tx_handle = tx_handle;
  slot->db_handle = db_handle;
  return true;
}

static bool sec4_rt_db_lookup_tx_db(int64_t tx_handle, int64_t *db_handle_out) {
  if (tx_handle == 0 || db_handle_out == NULL) {
    return false;
  }

  for (size_t i = 0; i < SEC4_RT_MAX_DB_TXS; i++) {
    const sec4_rt_db_tx_state *slot = &g_sec4_rt_db_txs[i];
    if (slot->active && slot->tx_handle == tx_handle) {
      *db_handle_out = slot->db_handle;
      return true;
    }
  }
  return false;
}

static sec4_rt_db_result sec4_rt_db_resolve_base_root(
    char *base_root,
    size_t base_root_size,
    bool allow_create
) {
  if (base_root == NULL || base_root_size == 0) {
    return SEC4_RT_DB_RESULT_INVALID;
  }

  char configured[SEC4_RT_MAX_FS_PATH_BYTES];
  const char *raw_base = getenv("SEC4_RT_DB_BASE");
  if (raw_base == NULL || raw_base[0] == '\0') {
    char cwd[SEC4_RT_MAX_FS_PATH_BYTES];
    if (getcwd(cwd, sizeof(cwd)) == NULL) {
      return SEC4_RT_DB_RESULT_IO;
    }
    int written = snprintf(configured, sizeof(configured), "%s/.sec4-db", cwd);
    if (written <= 0 || (size_t) written >= sizeof(configured)) {
      return SEC4_RT_DB_RESULT_INVALID;
    }
  } else if (raw_base[0] == '/') {
    if (strlen(raw_base) >= sizeof(configured)) {
      return SEC4_RT_DB_RESULT_INVALID;
    }
    strncpy(configured, raw_base, sizeof(configured) - 1);
    configured[sizeof(configured) - 1] = '\0';
  } else {
    char cwd[SEC4_RT_MAX_FS_PATH_BYTES];
    if (getcwd(cwd, sizeof(cwd)) == NULL) {
      return SEC4_RT_DB_RESULT_IO;
    }
    int written = snprintf(configured, sizeof(configured), "%s/%s", cwd, raw_base);
    if (written <= 0 || (size_t) written >= sizeof(configured)) {
      return SEC4_RT_DB_RESULT_INVALID;
    }
  }

  if (strchr(configured, '\r') != NULL || strchr(configured, '\n') != NULL) {
    return SEC4_RT_DB_RESULT_INVALID;
  }

  char resolved[SEC4_RT_MAX_FS_PATH_BYTES];
  if (realpath(configured, resolved) == NULL) {
    if (errno == ENOENT) {
      if (!allow_create) {
        return SEC4_RT_DB_RESULT_NOT_FOUND;
      }
      if (!sec4_rt_fs_mkdirs(configured)) {
        return SEC4_RT_DB_RESULT_IO;
      }
      if (realpath(configured, resolved) == NULL) {
        return SEC4_RT_DB_RESULT_IO;
      }
    } else {
      return SEC4_RT_DB_RESULT_IO;
    }
  }

  struct stat statbuf;
  if (stat(resolved, &statbuf) != 0) {
    return SEC4_RT_DB_RESULT_IO;
  }
  if (!S_ISDIR(statbuf.st_mode)) {
    return SEC4_RT_DB_RESULT_INVALID;
  }
  if (resolved[0] != '/') {
    return SEC4_RT_DB_RESULT_INVALID;
  }

  strncpy(base_root, resolved, base_root_size - 1);
  base_root[base_root_size - 1] = '\0';
  return SEC4_RT_DB_RESULT_OK;
}

static sec4_rt_db_result sec4_rt_db_resolve_record_path(
    const char *base_root,
    char *record_path,
    size_t record_path_size
) {
  if (base_root == NULL || base_root[0] == '\0' || record_path == NULL || record_path_size == 0) {
    return SEC4_RT_DB_RESULT_INVALID;
  }

  int written = snprintf(record_path, record_path_size, "%s/records.log", base_root);
  if (written <= 0 || (size_t) written >= record_path_size) {
    return SEC4_RT_DB_RESULT_INVALID;
  }
  return SEC4_RT_DB_RESULT_OK;
}

static bool sec4_rt_hex_encode_bytes(
    const char *input,
    size_t input_len,
    char *hex_out,
    size_t hex_out_size
) {
  if (input == NULL || hex_out == NULL) {
    return false;
  }

  size_t required = input_len * 2 + 1;
  if (required > hex_out_size) {
    return false;
  }

  static const char k_hex[16] = {
      '0', '1', '2', '3', '4', '5', '6', '7',
      '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'
  };
  for (size_t i = 0; i < input_len; i++) {
    unsigned char byte = (unsigned char) input[i];
    hex_out[i * 2] = k_hex[(byte >> 4) & 0x0f];
    hex_out[i * 2 + 1] = k_hex[byte & 0x0f];
  }
  hex_out[input_len * 2] = '\0';
  return true;
}

static int sec4_rt_hex_nibble(char value) {
  if (value >= '0' && value <= '9') {
    return value - '0';
  }
  if (value >= 'a' && value <= 'f') {
    return 10 + (value - 'a');
  }
  if (value >= 'A' && value <= 'F') {
    return 10 + (value - 'A');
  }
  return -1;
}

static bool sec4_rt_hex_decode_bytes(
    const char *hex_input,
    char *output,
    size_t output_size,
    size_t *output_len
) {
  if (hex_input == NULL || output == NULL || output_len == NULL) {
    return false;
  }

  size_t hex_len = strlen(hex_input);
  if ((hex_len % 2) != 0) {
    return false;
  }

  size_t decoded_len = hex_len / 2;
  if (decoded_len >= output_size) {
    return false;
  }

  for (size_t i = 0; i < decoded_len; i++) {
    int high = sec4_rt_hex_nibble(hex_input[i * 2]);
    int low = sec4_rt_hex_nibble(hex_input[i * 2 + 1]);
    if (high < 0 || low < 0) {
      return false;
    }
    output[i] = (char) ((high << 4) | low);
  }

  output[decoded_len] = '\0';
  *output_len = decoded_len;
  return true;
}

static bool sec4_rt_db_parse_record_line(
    char *line,
    int64_t *db_handle,
    int64_t *query_handle,
    const char **body_hex
) {
  if (line == NULL || db_handle == NULL || query_handle == NULL || body_hex == NULL) {
    return false;
  }

  char *first_sep = strchr(line, '|');
  if (first_sep == NULL) {
    return false;
  }
  *first_sep = '\0';
  if (strcmp(line, "v1") != 0) {
    return false;
  }

  char *db_raw = first_sep + 1;
  char *second_sep = strchr(db_raw, '|');
  if (second_sep == NULL) {
    return false;
  }
  *second_sep = '\0';

  char *query_raw = second_sep + 1;
  char *third_sep = strchr(query_raw, '|');
  if (third_sep == NULL) {
    return false;
  }
  *third_sep = '\0';

  char *hex_raw = third_sep + 1;
  if (hex_raw[0] == '\0' || strchr(hex_raw, '|') != NULL) {
    return false;
  }

  if (!sec4_rt_parse_i64_strict(db_raw, db_handle)
      || !sec4_rt_parse_i64_strict(query_raw, query_handle)) {
    return false;
  }

  *body_hex = hex_raw;
  return true;
}

static sec4_rt_db_result sec4_rt_db_append_record(
    int64_t db_handle,
    int64_t query_handle,
    const char *body
) {
  if (db_handle == 0 || query_handle == 0 || body == NULL || body[0] == '\0') {
    return SEC4_RT_DB_RESULT_INVALID;
  }

  char base_root[SEC4_RT_MAX_FS_PATH_BYTES];
  sec4_rt_db_result base_result =
      sec4_rt_db_resolve_base_root(base_root, sizeof(base_root), true);
  if (base_result != SEC4_RT_DB_RESULT_OK) {
    return base_result;
  }

  char record_path[SEC4_RT_MAX_FS_PATH_BYTES];
  sec4_rt_db_result record_path_result =
      sec4_rt_db_resolve_record_path(base_root, record_path, sizeof(record_path));
  if (record_path_result != SEC4_RT_DB_RESULT_OK) {
    return record_path_result;
  }

  size_t body_len = strlen(body);
  if (body_len == 0 || body_len >= SEC4_RT_MAX_TRACKED_VALUE_BYTES) {
    return SEC4_RT_DB_RESULT_INVALID;
  }

  char encoded_body[SEC4_RT_MAX_DB_RECORD_HEX_BYTES];
  if (!sec4_rt_hex_encode_bytes(body, body_len, encoded_body, sizeof(encoded_body))) {
    return SEC4_RT_DB_RESULT_INVALID;
  }

  char line[SEC4_RT_MAX_DB_RECORD_LINE_BYTES];
  int written = snprintf(
      line,
      sizeof(line),
      "v1|%lld|%lld|%s\n",
      (long long) db_handle,
      (long long) query_handle,
      encoded_body
  );
  if (written <= 0 || (size_t) written >= sizeof(line)) {
    return SEC4_RT_DB_RESULT_INVALID;
  }

  FILE *file = fopen(record_path, "ab");
  if (file == NULL) {
    return SEC4_RT_DB_RESULT_IO;
  }

  bool io_failed = false;
  size_t line_len = (size_t) written;
  if (fwrite(line, 1, line_len, file) != line_len) {
    io_failed = true;
  }
  if (fclose(file) != 0) {
    io_failed = true;
  }
  if (io_failed) {
    return SEC4_RT_DB_RESULT_IO;
  }

  return SEC4_RT_DB_RESULT_OK;
}

static sec4_rt_db_result sec4_rt_db_read_latest_record_body(
    int64_t db_handle,
    int64_t query_handle,
    char *body_out,
    size_t body_out_size
) {
  if (db_handle == 0 || query_handle == 0 || body_out == NULL || body_out_size == 0) {
    return SEC4_RT_DB_RESULT_INVALID;
  }

  char base_root[SEC4_RT_MAX_FS_PATH_BYTES];
  sec4_rt_db_result base_result =
      sec4_rt_db_resolve_base_root(base_root, sizeof(base_root), false);
  if (base_result != SEC4_RT_DB_RESULT_OK) {
    return base_result;
  }

  char record_path[SEC4_RT_MAX_FS_PATH_BYTES];
  sec4_rt_db_result record_path_result =
      sec4_rt_db_resolve_record_path(base_root, record_path, sizeof(record_path));
  if (record_path_result != SEC4_RT_DB_RESULT_OK) {
    return record_path_result;
  }

  FILE *file = fopen(record_path, "rb");
  if (file == NULL) {
    if (errno == ENOENT) {
      return SEC4_RT_DB_RESULT_NOT_FOUND;
    }
    return SEC4_RT_DB_RESULT_IO;
  }

  bool io_failed = false;
  bool found = false;
  char line[SEC4_RT_MAX_DB_RECORD_LINE_BYTES];
  while (fgets(line, sizeof(line), file) != NULL) {
    size_t line_len = strlen(line);
    if (line_len == 0) {
      continue;
    }
    if (line[line_len - 1] == '\n') {
      line[line_len - 1] = '\0';
    } else if (!feof(file)) {
      io_failed = true;
      break;
    }

    if (line[0] == '\0') {
      continue;
    }

    int64_t record_db = 0;
    int64_t record_query = 0;
    const char *record_body_hex = NULL;
    if (!sec4_rt_db_parse_record_line(
            line,
            &record_db,
            &record_query,
            &record_body_hex
        )) {
      io_failed = true;
      break;
    }

    if (record_db != db_handle || record_query != query_handle) {
      continue;
    }

    size_t decoded_len = 0;
    if (!sec4_rt_hex_decode_bytes(record_body_hex, body_out, body_out_size, &decoded_len)) {
      io_failed = true;
      break;
    }
    found = true;
  }

  if (ferror(file) != 0) {
    io_failed = true;
  }
  if (fclose(file) != 0) {
    io_failed = true;
  }

  if (io_failed) {
    return SEC4_RT_DB_RESULT_IO;
  }
  if (!found) {
    return SEC4_RT_DB_RESULT_NOT_FOUND;
  }
  return SEC4_RT_DB_RESULT_OK;
}

int64_t sec4_rt_sql_q(const char *query_template, int64_t params) {
  if (query_template == NULL || query_template[0] == '\0') {
    sec4_rt_store_std_error_response(
        400,
        "DB.SQL_TEMPLATE_INVALID",
        "validation",
        "sql.q query template is required"
    );
    return 0;
  }

  int64_t template_handle = sec4_rt_track_string_value(query_template, UINT64_C(0x18181));
  if (template_handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "DB.SQL_TEMPLATE_INTERNAL",
        "internal",
        "sql.q runtime failure"
    );
    return 0;
  }

  int64_t query_handle = sec4_rt_handle_from_two(template_handle, params, UINT64_C(0x18182));
  if (query_handle == 0 || !sec4_rt_db_register_query_state(query_handle, template_handle, params)) {
    sec4_rt_store_std_error_response(
        500,
        "DB.SQL_TEMPLATE_INTERNAL",
        "internal",
        "sql.q runtime failure"
    );
    return 0;
  }
  return query_handle;
}

int64_t sec4_rt_db_exec(int64_t db, int64_t query) {
  if (db == 0 || query == 0) {
    sec4_rt_store_std_error_response(
        400,
        "DB.EXEC_INVALID",
        "validation",
        "db.exec requires db capability and query handle"
    );
    return 0;
  }

  const sec4_rt_db_query_state *query_state = sec4_rt_db_lookup_query_state(query);
  if (query_state == NULL) {
    sec4_rt_store_std_error_response(
        400,
        "DB.EXEC_QUERY_INVALID",
        "validation",
        "db.exec query handle must come from sql.q"
    );
    return 0;
  }

  char record_body[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  int written = snprintf(
      record_body,
      sizeof(record_body),
      "op=exec;db=%lld;query=%lld;template=%lld;params=%lld;tx=0",
      (long long) db,
      (long long) query,
      (long long) query_state->template_handle,
      (long long) query_state->params
  );
  if (written <= 0 || (size_t) written >= sizeof(record_body)) {
    sec4_rt_store_std_error_response(
        500,
        "DB.EXEC_IO",
        "internal",
        "db.exec runtime io failure"
    );
    return 0;
  }

  sec4_rt_db_result append_result = sec4_rt_db_append_record(db, query, record_body);
  if (append_result == SEC4_RT_DB_RESULT_INVALID) {
    sec4_rt_store_std_error_response(
        400,
        "DB.EXEC_BASE_INVALID",
        "validation",
        "db.exec runtime base path is invalid"
    );
    return 0;
  }
  if (append_result != SEC4_RT_DB_RESULT_OK) {
    sec4_rt_store_std_error_response(
        500,
        "DB.EXEC_IO",
        "internal",
        "db.exec runtime io failure"
    );
    return 0;
  }

  return sec4_rt_handle_from_two(db, query, UINT64_C(0x18183));
}

int64_t sec4_rt_db_tx(int64_t db) {
  if (db == 0) {
    sec4_rt_store_std_error_response(
        400,
        "DB.TX_INVALID",
        "validation",
        "db.tx requires db capability handle"
    );
    return 0;
  }

  int64_t tx_handle = sec4_rt_handle_from_two(db, 1, UINT64_C(0x18184));
  if (tx_handle == 0 || !sec4_rt_db_register_tx_state(tx_handle, db)) {
    sec4_rt_store_std_error_response(
        500,
        "DB.TX_INTERNAL",
        "internal",
        "db.tx runtime failure"
    );
    return 0;
  }
  return tx_handle;
}

int64_t sec4_rt_db_exec_tx(int64_t tx, int64_t query) {
  if (tx == 0 || query == 0) {
    sec4_rt_store_std_error_response(
        400,
        "DB.EXEC_TX_INVALID",
        "validation",
        "db.execTx requires transaction and query handles"
    );
    return 0;
  }

  int64_t db = 0;
  if (!sec4_rt_db_lookup_tx_db(tx, &db)) {
    sec4_rt_store_std_error_response(
        400,
        "DB.EXEC_TX_HANDLE_INVALID",
        "validation",
        "db.execTx transaction handle must come from db.tx"
    );
    return 0;
  }

  const sec4_rt_db_query_state *query_state = sec4_rt_db_lookup_query_state(query);
  if (query_state == NULL) {
    sec4_rt_store_std_error_response(
        400,
        "DB.EXEC_TX_QUERY_INVALID",
        "validation",
        "db.execTx query handle must come from sql.q"
    );
    return 0;
  }

  char record_body[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  int written = snprintf(
      record_body,
      sizeof(record_body),
      "op=execTx;db=%lld;query=%lld;template=%lld;params=%lld;tx=%lld",
      (long long) db,
      (long long) query,
      (long long) query_state->template_handle,
      (long long) query_state->params,
      (long long) tx
  );
  if (written <= 0 || (size_t) written >= sizeof(record_body)) {
    sec4_rt_store_std_error_response(
        500,
        "DB.EXEC_TX_IO",
        "internal",
        "db.execTx runtime io failure"
    );
    return 0;
  }

  sec4_rt_db_result append_result = sec4_rt_db_append_record(db, query, record_body);
  if (append_result == SEC4_RT_DB_RESULT_INVALID) {
    sec4_rt_store_std_error_response(
        400,
        "DB.EXEC_TX_BASE_INVALID",
        "validation",
        "db.execTx runtime base path is invalid"
    );
    return 0;
  }
  if (append_result != SEC4_RT_DB_RESULT_OK) {
    sec4_rt_store_std_error_response(
        500,
        "DB.EXEC_TX_IO",
        "internal",
        "db.execTx runtime io failure"
    );
    return 0;
  }

  return sec4_rt_handle_from_two(tx, query, UINT64_C(0x18185));
}

int64_t sec4_rt_db_query_one(int64_t db, int64_t query, int64_t row_schema) {
  if (db == 0 || query == 0 || row_schema == 0) {
    sec4_rt_store_std_error_response(
        400,
        "DB.QUERY_ONE_INVALID",
        "validation",
        "db.queryOne requires db capability, query, and row schema handles"
    );
    return 0;
  }

  const sec4_rt_db_query_state *query_state = sec4_rt_db_lookup_query_state(query);
  if (query_state == NULL) {
    sec4_rt_store_std_error_response(
        400,
        "DB.QUERY_ONE_QUERY_INVALID",
        "validation",
        "db.queryOne query handle must come from sql.q"
    );
    return 0;
  }
  (void) query_state;

  char record_body[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  sec4_rt_db_result read_result = sec4_rt_db_read_latest_record_body(
      db,
      query,
      record_body,
      sizeof(record_body)
  );
  if (read_result == SEC4_RT_DB_RESULT_INVALID) {
    sec4_rt_store_std_error_response(
        400,
        "DB.QUERY_ONE_BASE_INVALID",
        "validation",
        "db.queryOne runtime base path is invalid"
    );
    return 0;
  }
  if (read_result == SEC4_RT_DB_RESULT_NOT_FOUND) {
    sec4_rt_store_std_error_response(
        404,
        "DB.QUERY_ONE_NOT_FOUND",
        "missing_dependency",
        "db.queryOne record not found"
    );
    return 0;
  }
  if (read_result != SEC4_RT_DB_RESULT_OK) {
    sec4_rt_store_std_error_response(
        500,
        "DB.QUERY_ONE_IO",
        "internal",
        "db.queryOne runtime io failure"
    );
    return 0;
  }

  char row_body[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  int written = snprintf(
      row_body,
      sizeof(row_body),
      "%s;rowSchema=%lld",
      record_body,
      (long long) row_schema
  );
  if (written <= 0 || (size_t) written >= sizeof(row_body)) {
    sec4_rt_store_std_error_response(
        500,
        "DB.QUERY_ONE_IO",
        "internal",
        "db.queryOne runtime io failure"
    );
    return 0;
  }

  int64_t row_handle = sec4_rt_track_string_value(row_body, UINT64_C(0x18186));
  if (row_handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "DB.QUERY_ONE_IO",
        "internal",
        "db.queryOne runtime io failure"
    );
    return 0;
  }

  return row_handle;
}

static bool sec4_rt_fs_path_has_traversal(const char *path) {
  if (path == NULL) {
    return false;
  }
  const char *cursor = path;
  while (*cursor != '\0') {
    if (cursor[0] == '.'
        && cursor[1] == '.'
        && (cursor == path || cursor[-1] == '/')
        && (cursor[2] == '\0' || cursor[2] == '/')) {
      return true;
    }
    cursor += 1;
  }
  return false;
}

static bool sec4_rt_fs_path_within_base(const char *base_root, const char *path) {
  if (base_root == NULL || path == NULL || base_root[0] == '\0' || path[0] == '\0') {
    return false;
  }
  if (strcmp(base_root, "/") == 0) {
    return path[0] == '/';
  }

  size_t base_len = strlen(base_root);
  if (strncmp(base_root, path, base_len) != 0) {
    return false;
  }
  return path[base_len] == '\0' || path[base_len] == '/';
}

static bool sec4_rt_fs_mkdirs(const char *path) {
  if (path == NULL || path[0] == '\0') {
    return false;
  }

  if (strcmp(path, "/") == 0) {
    return true;
  }

  char mutable_path[SEC4_RT_MAX_FS_PATH_BYTES];
  strncpy(mutable_path, path, sizeof(mutable_path) - 1);
  mutable_path[sizeof(mutable_path) - 1] = '\0';
  if (mutable_path[0] == '\0') {
    return false;
  }

  for (char *cursor = mutable_path + 1; *cursor != '\0'; cursor++) {
    if (*cursor != '/') {
      continue;
    }
    *cursor = '\0';
    if (mutable_path[0] != '\0' && mkdir(mutable_path, 0755) != 0 && errno != EEXIST) {
      return false;
    }
    *cursor = '/';
  }

  if (mkdir(mutable_path, 0755) != 0 && errno != EEXIST) {
    return false;
  }
  return true;
}

static sec4_rt_fs_result sec4_rt_fs_resolve_base_root(
    char *base_root,
    size_t base_root_size,
    bool allow_create
) {
  if (base_root == NULL || base_root_size == 0) {
    return SEC4_RT_FS_RESULT_INVALID;
  }

  char configured[SEC4_RT_MAX_FS_PATH_BYTES];
  const char *raw_base = getenv("SEC4_RT_FS_BASE");
  if (raw_base == NULL || raw_base[0] == '\0') {
    if (getcwd(configured, sizeof(configured)) == NULL) {
      return SEC4_RT_FS_RESULT_IO;
    }
  } else if (raw_base[0] == '/') {
    if (strlen(raw_base) >= sizeof(configured)) {
      return SEC4_RT_FS_RESULT_INVALID;
    }
    strncpy(configured, raw_base, sizeof(configured) - 1);
    configured[sizeof(configured) - 1] = '\0';
  } else {
    char cwd[SEC4_RT_MAX_FS_PATH_BYTES];
    if (getcwd(cwd, sizeof(cwd)) == NULL) {
      return SEC4_RT_FS_RESULT_IO;
    }
    int written = snprintf(configured, sizeof(configured), "%s/%s", cwd, raw_base);
    if (written <= 0 || (size_t) written >= sizeof(configured)) {
      return SEC4_RT_FS_RESULT_INVALID;
    }
  }

  if (strchr(configured, '\r') != NULL || strchr(configured, '\n') != NULL) {
    return SEC4_RT_FS_RESULT_INVALID;
  }

  char resolved[SEC4_RT_MAX_FS_PATH_BYTES];
  if (realpath(configured, resolved) == NULL) {
    if (allow_create && errno == ENOENT) {
      if (!sec4_rt_fs_mkdirs(configured)) {
        return SEC4_RT_FS_RESULT_IO;
      }
      if (realpath(configured, resolved) == NULL) {
        return SEC4_RT_FS_RESULT_IO;
      }
    } else {
      return SEC4_RT_FS_RESULT_IO;
    }
  }

  if (resolved[0] != '/') {
    return SEC4_RT_FS_RESULT_INVALID;
  }

  strncpy(base_root, resolved, base_root_size - 1);
  base_root[base_root_size - 1] = '\0';
  return SEC4_RT_FS_RESULT_OK;
}

static sec4_rt_fs_result sec4_rt_fs_resolve_target_path(
    const char *base_root,
    const char *raw_path,
    bool for_write,
    char *resolved_path,
    size_t resolved_path_size
) {
  if (base_root == NULL || raw_path == NULL || resolved_path == NULL || resolved_path_size == 0) {
    return SEC4_RT_FS_RESULT_INVALID;
  }
  if (raw_path[0] == '\0' || strchr(raw_path, '\r') != NULL || strchr(raw_path, '\n') != NULL) {
    return SEC4_RT_FS_RESULT_INVALID;
  }
  if (sec4_rt_fs_path_has_traversal(raw_path)) {
    return SEC4_RT_FS_RESULT_OUT_OF_BASE;
  }

  char candidate[SEC4_RT_MAX_FS_PATH_BYTES];
  if (raw_path[0] == '/') {
    if (!sec4_rt_fs_path_within_base(base_root, raw_path)) {
      return SEC4_RT_FS_RESULT_OUT_OF_BASE;
    }
    if (strlen(raw_path) >= sizeof(candidate)) {
      return SEC4_RT_FS_RESULT_INVALID;
    }
    strncpy(candidate, raw_path, sizeof(candidate) - 1);
    candidate[sizeof(candidate) - 1] = '\0';
  } else {
    int written = snprintf(candidate, sizeof(candidate), "%s/%s", base_root, raw_path);
    if (written <= 0 || (size_t) written >= sizeof(candidate)) {
      return SEC4_RT_FS_RESULT_INVALID;
    }
  }

  if (!for_write) {
    if (realpath(candidate, resolved_path) == NULL) {
      return SEC4_RT_FS_RESULT_IO;
    }
    if (!sec4_rt_fs_path_within_base(base_root, resolved_path)) {
      return SEC4_RT_FS_RESULT_OUT_OF_BASE;
    }
    return SEC4_RT_FS_RESULT_OK;
  }

  char parent_candidate[SEC4_RT_MAX_FS_PATH_BYTES];
  strncpy(parent_candidate, candidate, sizeof(parent_candidate) - 1);
  parent_candidate[sizeof(parent_candidate) - 1] = '\0';
  char *slash = strrchr(parent_candidate, '/');
  if (slash == NULL) {
    return SEC4_RT_FS_RESULT_INVALID;
  }

  const char *leaf = slash + 1;
  if (leaf[0] == '\0' || strcmp(leaf, ".") == 0 || strcmp(leaf, "..") == 0) {
    return SEC4_RT_FS_RESULT_INVALID;
  }

  if (slash == parent_candidate) {
    parent_candidate[1] = '\0';
  } else {
    *slash = '\0';
  }

  if (!sec4_rt_fs_path_within_base(base_root, parent_candidate)) {
    return SEC4_RT_FS_RESULT_OUT_OF_BASE;
  }
  if (!sec4_rt_fs_mkdirs(parent_candidate)) {
    return SEC4_RT_FS_RESULT_IO;
  }

  char resolved_parent[SEC4_RT_MAX_FS_PATH_BYTES];
  if (realpath(parent_candidate, resolved_parent) == NULL) {
    return SEC4_RT_FS_RESULT_IO;
  }
  if (!sec4_rt_fs_path_within_base(base_root, resolved_parent)) {
    return SEC4_RT_FS_RESULT_OUT_OF_BASE;
  }

  int written = 0;
  if (strcmp(resolved_parent, "/") == 0) {
    written = snprintf(resolved_path, resolved_path_size, "/%s", leaf);
  } else {
    written = snprintf(resolved_path, resolved_path_size, "%s/%s", resolved_parent, leaf);
  }
  if (written <= 0 || (size_t) written >= resolved_path_size) {
    return SEC4_RT_FS_RESULT_INVALID;
  }
  return SEC4_RT_FS_RESULT_OK;
}

int64_t sec4_rt_fs_read(int64_t fs, int64_t path) {
  if (fs == 0 || path == 0) {
    sec4_rt_store_std_error_response(
        400,
        "FS.READ_INVALID",
        "validation",
        "fs.read requires fs capability and path handles"
    );
    return 0;
  }

  const char *path_value = sec4_rt_lookup_tracked_value(path);
  if (path_value == NULL) {
    sec4_rt_store_std_error_response(
        400,
        "FS.READ_PATH_INVALID",
        "validation",
        "fs.read path handle must be tracked"
    );
    return 0;
  }

  char base_root[SEC4_RT_MAX_FS_PATH_BYTES];
  sec4_rt_fs_result base_result =
      sec4_rt_fs_resolve_base_root(base_root, sizeof(base_root), false);
  if (base_result == SEC4_RT_FS_RESULT_INVALID) {
    sec4_rt_store_std_error_response(
        400,
        "FS.READ_BASE_INVALID",
        "validation",
        "fs.read runtime base path is invalid"
    );
    return 0;
  }
  if (base_result == SEC4_RT_FS_RESULT_IO) {
    sec4_rt_store_std_error_response(
        500,
        "FS.READ_IO",
        "internal",
        "fs.read runtime io failure"
    );
    return 0;
  }

  char resolved_path[SEC4_RT_MAX_FS_PATH_BYTES];
  sec4_rt_fs_result path_result = sec4_rt_fs_resolve_target_path(
      base_root,
      path_value,
      false,
      resolved_path,
      sizeof(resolved_path)
  );
  if (path_result == SEC4_RT_FS_RESULT_INVALID) {
    sec4_rt_store_std_error_response(
        400,
        "FS.READ_PATH_INVALID",
        "validation",
        "fs.read path is invalid"
    );
    return 0;
  }
  if (path_result == SEC4_RT_FS_RESULT_OUT_OF_BASE) {
    sec4_rt_store_std_error_response(
        400,
        "FS.READ_PATH_DENIED",
        "validation",
        "fs.read path is outside runtime base"
    );
    return 0;
  }
  if (path_result == SEC4_RT_FS_RESULT_IO) {
    sec4_rt_store_std_error_response(
        500,
        "FS.READ_IO",
        "internal",
        "fs.read runtime io failure"
    );
    return 0;
  }

  FILE *file = fopen(resolved_path, "rb");
  if (file == NULL) {
    sec4_rt_store_std_error_response(
        500,
        "FS.READ_IO",
        "internal",
        "fs.read runtime io failure"
    );
    return 0;
  }

  char buffer[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  size_t total_read = 0;
  while (total_read < sizeof(buffer) - 1) {
    size_t chunk_size = (sizeof(buffer) - 1) - total_read;
    size_t bytes = fread(buffer + total_read, 1, chunk_size, file);
    total_read += bytes;
    if (bytes == 0) {
      break;
    }
  }

  bool io_failed = ferror(file) != 0;
  bool too_large = false;
  if (!io_failed && !feof(file)) {
    int extra = fgetc(file);
    if (extra != EOF) {
      too_large = true;
    }
  }

  if (fclose(file) != 0) {
    io_failed = true;
  }

  if (io_failed || too_large) {
    sec4_rt_store_std_error_response(
        500,
        "FS.READ_IO",
        "internal",
        "fs.read runtime io failure"
    );
    return 0;
  }

  buffer[total_read] = '\0';
  int64_t value_handle = sec4_rt_track_sized_value(buffer, total_read, UINT64_C(0x18187));
  if (value_handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "FS.READ_IO",
        "internal",
        "fs.read runtime io failure"
    );
    return 0;
  }

  return value_handle;
}

int64_t sec4_rt_fs_write(int64_t fs, int64_t path, int64_t value) {
  if (fs == 0 || path == 0 || value == 0) {
    sec4_rt_store_std_error_response(
        400,
        "FS.WRITE_INVALID",
        "validation",
        "fs.write requires fs capability, path handle, and value handle"
    );
    return 0;
  }

  const char *path_value = sec4_rt_lookup_tracked_value(path);
  if (path_value == NULL) {
    sec4_rt_store_std_error_response(
        400,
        "FS.WRITE_PATH_INVALID",
        "validation",
        "fs.write path handle must be tracked"
    );
    return 0;
  }

  const char *value_bytes = sec4_rt_lookup_tracked_value(value);
  if (value_bytes == NULL) {
    sec4_rt_store_std_error_response(
        400,
        "FS.WRITE_VALUE_INVALID",
        "validation",
        "fs.write value handle must be tracked"
    );
    return 0;
  }

  char base_root[SEC4_RT_MAX_FS_PATH_BYTES];
  sec4_rt_fs_result base_result =
      sec4_rt_fs_resolve_base_root(base_root, sizeof(base_root), true);
  if (base_result == SEC4_RT_FS_RESULT_INVALID) {
    sec4_rt_store_std_error_response(
        400,
        "FS.WRITE_BASE_INVALID",
        "validation",
        "fs.write runtime base path is invalid"
    );
    return 0;
  }
  if (base_result == SEC4_RT_FS_RESULT_IO) {
    sec4_rt_store_std_error_response(
        500,
        "FS.WRITE_IO",
        "internal",
        "fs.write runtime io failure"
    );
    return 0;
  }

  char resolved_path[SEC4_RT_MAX_FS_PATH_BYTES];
  sec4_rt_fs_result path_result = sec4_rt_fs_resolve_target_path(
      base_root,
      path_value,
      true,
      resolved_path,
      sizeof(resolved_path)
  );
  if (path_result == SEC4_RT_FS_RESULT_INVALID) {
    sec4_rt_store_std_error_response(
        400,
        "FS.WRITE_PATH_INVALID",
        "validation",
        "fs.write path is invalid"
    );
    return 0;
  }
  if (path_result == SEC4_RT_FS_RESULT_OUT_OF_BASE) {
    sec4_rt_store_std_error_response(
        400,
        "FS.WRITE_PATH_DENIED",
        "validation",
        "fs.write path is outside runtime base"
    );
    return 0;
  }
  if (path_result == SEC4_RT_FS_RESULT_IO) {
    sec4_rt_store_std_error_response(
        500,
        "FS.WRITE_IO",
        "internal",
        "fs.write runtime io failure"
    );
    return 0;
  }

  FILE *file = fopen(resolved_path, "wb");
  if (file == NULL) {
    sec4_rt_store_std_error_response(
        500,
        "FS.WRITE_IO",
        "internal",
        "fs.write runtime io failure"
    );
    return 0;
  }

  size_t value_len = strlen(value_bytes);
  bool io_failed = false;
  if (value_len > 0) {
    size_t bytes_written = fwrite(value_bytes, 1, value_len, file);
    if (bytes_written != value_len) {
      io_failed = true;
    }
  }
  if (fclose(file) != 0) {
    io_failed = true;
  }
  if (io_failed) {
    sec4_rt_store_std_error_response(
        500,
        "FS.WRITE_IO",
        "internal",
        "fs.write runtime io failure"
    );
    return 0;
  }

  return sec4_rt_handle_from_three(fs, path, value, UINT64_C(0x18188));
}

int64_t sec4_rt_http_get(int64_t net, int64_t url) {
  if (net == 0 || url == 0) {
    sec4_rt_store_std_error_response(
        400,
        "NET.GET_INVALID",
        "validation",
        "httpClient.get requires net capability and url handles"
    );
    return 0;
  }

  const char *url_value = sec4_rt_lookup_tracked_value(url);
  if (url_value == NULL || url_value[0] == '\0' || !sec4_rt_is_public_url_valid(url_value)) {
    sec4_rt_store_std_error_response(
        400,
        "NET.URL_PUBLIC_INVALID",
        "validation",
        "httpClient.get requires PublicUrl input"
    );
    return 0;
  }

  (void) net;
  return sec4_rt_outbound_http_get_handle(
      url_value,
      UINT64_C(0x18189),
      SEC4_RT_NET_SCOPE_PUBLIC
  );
}

int64_t sec4_rt_http_get_internal(int64_t net, int64_t url) {
  if (net == 0 || url == 0) {
    sec4_rt_store_std_error_response(
        400,
        "NET.GET_INTERNAL_INVALID",
        "validation",
        "httpClient.getInternal requires net capability and url handles"
    );
    return 0;
  }

  if (!sec4_rt_env_flag_enabled("SEC4_RT_ALLOW_INTERNAL_NET")) {
    sec4_rt_store_std_error_response(
        403,
        "NET.INTERNAL_DENIED",
        "authorization",
        "internal network access denied by runtime policy"
    );
    return 0;
  }

  const char *url_value = sec4_rt_lookup_tracked_value(url);
  if (url_value == NULL || url_value[0] == '\0' || !sec4_rt_is_internal_url_valid(url_value)) {
    sec4_rt_store_std_error_response(
        400,
        "NET.URL_INTERNAL_INVALID",
        "validation",
        "httpClient.getInternal requires InternalUrl input"
    );
    return 0;
  }

  (void) net;
  return sec4_rt_outbound_http_get_handle(
      url_value,
      UINT64_C(0x1818A),
      SEC4_RT_NET_SCOPE_INTERNAL
  );
}

int64_t sec4_rt_secret_get(int64_t secrets_cap, const char *name) {
  if (secrets_cap == 0 || name == NULL || name[0] == '\0') {
    sec4_rt_store_std_error_response(
        400,
        "SECRET.GET_INVALID",
        "validation",
        "secret key is required"
    );
    return 0;
  }

  const char *value = getenv(name);
  if (value == NULL || value[0] == '\0') {
    sec4_rt_store_std_error_response(
        404,
        "SECRET.NOT_FOUND",
        "missing_secret",
        "secret not found"
    );
    return 0;
  }

  int64_t handle = sec4_rt_track_string_value(value, UINT64_C(0x41414));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "SECRET.GET_INTERNAL",
        "internal",
        "secret runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_secret_redact(int64_t secret_value) {
  const char *raw = sec4_rt_lookup_tracked_value(secret_value);
  if (raw == NULL || raw[0] == '\0') {
    return 0;
  }

  int64_t digest = sec4_rt_gate_handle_from_string(raw, UINT64_C(0x51515));
  if (digest == 0) {
    return 0;
  }

  char redacted[64];
  int written = snprintf(
      redacted,
      sizeof(redacted),
      "[redacted:%016llx]",
      (unsigned long long) ((uint64_t) digest)
  );
  if (written <= 0 || (size_t) written >= sizeof(redacted)) {
    return 0;
  }

  return sec4_rt_track_string_value(redacted, UINT64_C(0x61616));
}

int64_t sec4_rt_secret_reveal(int64_t secrets_cap, int64_t secret_value) {
  if (secrets_cap == 0 || secret_value == 0) {
    sec4_rt_store_std_error_response(
        400,
        "SECRET.REVEAL_INVALID",
        "validation",
        "secret value is required"
    );
    return 0;
  }

  if (!sec4_rt_env_flag_enabled("SEC4_RT_ALLOW_SECRET_REVEAL")) {
    sec4_rt_store_std_error_response(
        403,
        "SECRET.REVEAL_DENIED",
        "authorization",
        "secret reveal disabled by runtime policy"
    );
    return 0;
  }

  const char *value = sec4_rt_lookup_tracked_value(secret_value);
  if (value == NULL || value[0] == '\0') {
    sec4_rt_store_std_error_response(
        404,
        "SECRET.NOT_FOUND",
        "missing_secret",
        "secret not found"
    );
    return 0;
  }

  return sec4_rt_track_string_value(value, UINT64_C(0x71717));
}

bool sec4_rt_crypto_ct_eq(int64_t left_secret, int64_t right_secret) {
  const char *left_value = sec4_rt_lookup_tracked_value(left_secret);
  const char *right_value = sec4_rt_lookup_tracked_value(right_secret);

  if (left_value != NULL && right_value != NULL) {
    return sec4_rt_constant_time_bytes_eq(
        (const unsigned char *) left_value,
        strlen(left_value),
        (const unsigned char *) right_value,
        strlen(right_value)
    );
  }

  if (left_secret == 0 || right_secret == 0) {
    return false;
  }

  unsigned char left_bytes[sizeof(uint64_t)];
  unsigned char right_bytes[sizeof(uint64_t)];
  uint64_t left_bits = (uint64_t) left_secret;
  uint64_t right_bits = (uint64_t) right_secret;
  for (size_t i = 0; i < sizeof(uint64_t); i++) {
    left_bytes[i] = (unsigned char) ((left_bits >> (i * 8)) & UINT64_C(0xff));
    right_bytes[i] = (unsigned char) ((right_bits >> (i * 8)) & UINT64_C(0xff));
  }

  return sec4_rt_constant_time_bytes_eq(
      left_bytes,
      sizeof(left_bytes),
      right_bytes,
      sizeof(right_bytes)
  );
}

int64_t sec4_rt_validate_header_value(int64_t input) {
  const char *value = sec4_rt_lookup_tracked_value(input);
  if (value == NULL || !sec4_rt_is_header_value_valid(value)) {
    sec4_rt_store_std_error_response(
        400,
        "VALIDATE.HEADER_VALUE_INVALID",
        "validation",
        "validate.headerValue requires tracked valid header value input"
    );
    return 0;
  }

  int64_t handle = sec4_rt_track_string_value(value, UINT64_C(0x40404));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "VALIDATE.HEADER_VALUE_INTERNAL",
        "internal",
        "validate.headerValue runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_validate_email(int64_t input) {
  const char *value = sec4_rt_lookup_tracked_value(input);
  if (value == NULL || !sec4_rt_is_email_valid(value)) {
    sec4_rt_store_std_error_response(
        400,
        "VALIDATE.EMAIL_INVALID",
        "validation",
        "validate.email requires tracked valid email input"
    );
    return 0;
  }

  int64_t handle = sec4_rt_track_string_value(value, UINT64_C(0x50505));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "VALIDATE.EMAIL_INTERNAL",
        "internal",
        "validate.email runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_validate_uuid(int64_t input) {
  const char *value = sec4_rt_lookup_tracked_value(input);
  if (value == NULL || !sec4_rt_is_uuid_valid(value)) {
    sec4_rt_store_std_error_response(
        400,
        "VALIDATE.UUID_INVALID",
        "validation",
        "validate.uuid requires tracked valid uuid input"
    );
    return 0;
  }

  int64_t handle = sec4_rt_track_string_value(value, UINT64_C(0x60606));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "VALIDATE.UUID_INTERNAL",
        "internal",
        "validate.uuid runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_validate_int64(int64_t input) {
  const char *value = sec4_rt_lookup_tracked_value(input);
  if (value == NULL || !sec4_rt_is_int64_text_valid(value)) {
    sec4_rt_store_std_error_response(
        400,
        "VALIDATE.INT64_INVALID",
        "validation",
        "validate.int64 requires tracked valid int64 input"
    );
    return 0;
  }

  int64_t handle = sec4_rt_track_string_value(value, UINT64_C(0x70707));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "VALIDATE.INT64_INTERNAL",
        "internal",
        "validate.int64 runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_validate_non_empty(int64_t input) {
  const char *value = sec4_rt_lookup_tracked_value(input);
  if (value == NULL || value[0] == '\0') {
    sec4_rt_store_std_error_response(
        400,
        "VALIDATE.NON_EMPTY_INVALID",
        "validation",
        "validate.nonEmpty requires tracked non-empty input"
    );
    return 0;
  }

  int64_t handle = sec4_rt_track_string_value(value, UINT64_C(0x80808));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "VALIDATE.NON_EMPTY_INTERNAL",
        "internal",
        "validate.nonEmpty runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_sanitize_html(int64_t input) {
  const char *value = sec4_rt_lookup_tracked_value(input);
  if (value == NULL) {
    sec4_rt_store_std_error_response(
        400,
        "SANITIZE.HTML_INVALID",
        "validation",
        "sanitize.html requires tracked input"
    );
    return 0;
  }

  char escaped[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  size_t escaped_len = 0;
  if (!sec4_rt_escape_html(value, escaped, sizeof(escaped), &escaped_len)) {
    sec4_rt_store_std_error_response(
        400,
        "SANITIZE.HTML_TOO_LARGE",
        "validation",
        "sanitize.html output exceeds runtime limits"
    );
    return 0;
  }

  int64_t handle = sec4_rt_track_sized_value(escaped, escaped_len, UINT64_C(0x90909));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "SANITIZE.HTML_INTERNAL",
        "internal",
        "sanitize.html runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_url_public(int64_t input) {
  const char *url = sec4_rt_lookup_tracked_value(input);
  if (url == NULL || url[0] == '\0') {
    sec4_rt_store_std_error_response(
        400,
        "NET.URL_PUBLIC_INVALID",
        "validation",
        "url.public requires tracked URL input"
    );
    return 0;
  }
  if (!sec4_rt_is_public_url_valid(url)) {
    sec4_rt_store_std_error_response(
        400,
        "NET.URL_PUBLIC_INVALID",
        "validation",
        "url.public value failed runtime public-url policy checks"
    );
    return 0;
  }
  int64_t handle = sec4_rt_track_string_value(url, UINT64_C(0xA0A0A));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "NET.URL_PUBLIC_INTERNAL",
        "internal",
        "url.public runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_url_internal(int64_t input) {
  const char *url = sec4_rt_lookup_tracked_value(input);
  if (url == NULL || url[0] == '\0') {
    sec4_rt_store_std_error_response(
        400,
        "NET.URL_INTERNAL_INVALID",
        "validation",
        "url.internal requires tracked URL input"
    );
    return 0;
  }
  if (!sec4_rt_is_internal_url_valid(url)) {
    sec4_rt_store_std_error_response(
        400,
        "NET.URL_INTERNAL_INVALID",
        "validation",
        "url.internal value failed runtime internal-url policy checks"
    );
    return 0;
  }
  int64_t handle = sec4_rt_track_string_value(url, UINT64_C(0xB0B0B));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "NET.URL_INTERNAL_INTERNAL",
        "internal",
        "url.internal runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_path_under(int64_t base, int64_t input) {
  if (base == 0 || input == 0) {
    sec4_rt_store_std_error_response(
        400,
        "PATH.UNDER_INVALID",
        "validation",
        "path.under requires tracked base and input handles"
    );
    return 0;
  }

  const char *base_value = sec4_rt_lookup_tracked_value(base);
  const char *path_value = sec4_rt_lookup_tracked_value(input);
  if (base_value == NULL || path_value == NULL) {
    sec4_rt_store_std_error_response(
        400,
        "PATH.UNDER_INVALID",
        "validation",
        "path.under requires tracked base and input handles"
    );
    return 0;
  }

  char normalized_base[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  if (!sec4_rt_normalize_absolute_path(
          base_value,
          normalized_base,
          sizeof(normalized_base)
      )) {
    sec4_rt_store_std_error_response(
        400,
        "PATH.UNDER_BASE_INVALID",
        "validation",
        "path.under base handle must contain an absolute normalized path"
    );
    return 0;
  }

  char normalized_path[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  if (!sec4_rt_normalize_under_base_path(
          normalized_base,
          path_value,
          normalized_path,
          sizeof(normalized_path)
      )) {
    sec4_rt_store_std_error_response(
        400,
        "PATH.UNDER_PATH_INVALID",
        "validation",
        "path.under input path is invalid"
    );
    return 0;
  }

  if (!sec4_rt_path_within_base(normalized_base, normalized_path)) {
    sec4_rt_store_std_error_response(
        400,
        "PATH.UNDER_OUT_OF_BASE",
        "validation",
        "path.under input path is outside base"
    );
    return 0;
  }

  int64_t handle = sec4_rt_track_string_value(normalized_path, UINT64_C(0xC0C0C));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "PATH.UNDER_INTERNAL",
        "internal",
        "path.under runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_path_base(const char *input) {
  char normalized[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
  if (!sec4_rt_normalize_absolute_path(input, normalized, sizeof(normalized))) {
    sec4_rt_store_std_error_response(
        400,
        "PATH.BASE_INVALID",
        "validation",
        "path.base requires an absolute normalized path"
    );
    return 0;
  }

  int64_t handle = sec4_rt_track_string_value(normalized, UINT64_C(0xD0D0D));
  if (handle == 0) {
    sec4_rt_store_std_error_response(
        500,
        "PATH.BASE_INTERNAL",
        "internal",
        "path.base runtime failure"
    );
    return 0;
  }
  return handle;
}

int64_t sec4_rt_headers_name(const char *input) {
  if (!sec4_rt_is_header_name_valid(input)) {
    return 0;
  }
  return sec4_rt_track_string_value(input, UINT64_C(0xE0E0E));
}

int64_t sec4_rt_headers_value(const char *input) {
  if (!sec4_rt_is_header_value_valid(input)) {
    return 0;
  }
  return sec4_rt_track_string_value(input, UINT64_C(0xF0F0F));
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
  if (slot == NULL) {
    return 1;
  }

  port = sec4_rt_parse_env_i64("SEC4_RT_HTTP_PORT", port);
  if (port <= 0 || port > 65535) {
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

static void sec4_rt_router_apply_default_cors(sec4_rt_router_state *slot) {
  if (slot == NULL) {
    return;
  }
  slot->cors_enabled = true;
  strncpy(slot->cors_allow_origin, "*", sizeof(slot->cors_allow_origin) - 1);
  slot->cors_allow_origin[sizeof(slot->cors_allow_origin) - 1] = '\0';
  slot->cors_allow_credentials = false;
  slot->cors_require_vary_origin = false;
  strncpy(
      slot->cors_allow_methods,
      "GET, POST, PUT, PATCH, DELETE, OPTIONS",
      sizeof(slot->cors_allow_methods) - 1
  );
  slot->cors_allow_methods[sizeof(slot->cors_allow_methods) - 1] = '\0';
  strncpy(
      slot->cors_allow_headers,
      "content-type, authorization",
      sizeof(slot->cors_allow_headers) - 1
  );
  slot->cors_allow_headers[sizeof(slot->cors_allow_headers) - 1] = '\0';
  slot->cors_max_age_seconds = 600;
}

static void sec4_rt_router_apply_default_security_headers(sec4_rt_router_state *slot) {
  if (slot == NULL) {
    return;
  }
  slot->security_headers_enabled = true;
  slot->security_x_content_type_options = true;
  strncpy(
      slot->security_x_frame_options,
      "DENY",
      sizeof(slot->security_x_frame_options) - 1
  );
  slot->security_x_frame_options[sizeof(slot->security_x_frame_options) - 1] = '\0';
  strncpy(
      slot->security_referrer_policy,
      "strict-origin-when-cross-origin",
      sizeof(slot->security_referrer_policy) - 1
  );
  slot->security_referrer_policy[sizeof(slot->security_referrer_policy) - 1] = '\0';
}

static void sec4_rt_router_apply_default_csrf(sec4_rt_router_state *slot) {
  if (slot == NULL) {
    return;
  }
  slot->csrf_enabled = true;
  strncpy(slot->csrf_mode, "double_submit", sizeof(slot->csrf_mode) - 1);
  slot->csrf_mode[sizeof(slot->csrf_mode) - 1] = '\0';
  strncpy(
      slot->csrf_protected_methods,
      "POST,PUT,PATCH,DELETE",
      sizeof(slot->csrf_protected_methods) - 1
  );
  slot->csrf_protected_methods[sizeof(slot->csrf_protected_methods) - 1] = '\0';
}

static void sec4_rt_router_apply_default_auth(sec4_rt_router_state *slot) {
  if (slot == NULL) {
    return;
  }
  slot->auth_enabled = true;
  strncpy(slot->auth_mode, "token", sizeof(slot->auth_mode) - 1);
  slot->auth_mode[sizeof(slot->auth_mode) - 1] = '\0';
}

static void sec4_rt_load_cors_policy_from_env(void) {
  memset(&g_sec4_rt_cors_policy, 0, sizeof(g_sec4_rt_cors_policy));
  g_sec4_rt_cors_policy.loaded = true;
  g_sec4_rt_cors_policy.enabled = sec4_rt_env_flag_enabled_default("SEC4_RT_CORS_ENABLED", true);
  g_sec4_rt_cors_policy.allow_credentials = sec4_rt_env_flag_enabled_default(
      "SEC4_RT_CORS_ALLOW_CREDENTIALS",
      false
  );
  g_sec4_rt_cors_policy.require_vary_origin = sec4_rt_env_flag_enabled_default(
      "SEC4_RT_CORS_REQUIRE_VARY_ORIGIN",
      false
  );
  const char *allowed_origins = getenv("SEC4_RT_CORS_ALLOWED_ORIGINS");
  if (!sec4_rt_csv_copy_first_token(
          allowed_origins,
          g_sec4_rt_cors_policy.allow_origin,
          sizeof(g_sec4_rt_cors_policy.allow_origin)
      )) {
    strncpy(
        g_sec4_rt_cors_policy.allow_origin,
        "*",
        sizeof(g_sec4_rt_cors_policy.allow_origin) - 1
    );
    g_sec4_rt_cors_policy.allow_origin[sizeof(g_sec4_rt_cors_policy.allow_origin) - 1] = '\0';
  }
}

static void sec4_rt_load_security_headers_policy_from_env(void) {
  memset(
      &g_sec4_rt_security_headers_policy,
      0,
      sizeof(g_sec4_rt_security_headers_policy)
  );
  g_sec4_rt_security_headers_policy.loaded = true;
  g_sec4_rt_security_headers_policy.enabled = sec4_rt_env_flag_enabled_default(
      "SEC4_RT_SECURITY_HEADERS_ENABLED",
      true
  );
  g_sec4_rt_security_headers_policy.x_content_type_options = sec4_rt_env_flag_enabled_default(
      "SEC4_RT_SECURITY_HEADERS_X_CONTENT_TYPE_OPTIONS",
      true
  );
  sec4_rt_read_env_string(
      "SEC4_RT_SECURITY_HEADERS_X_FRAME_OPTIONS",
      "DENY",
      g_sec4_rt_security_headers_policy.x_frame_options,
      sizeof(g_sec4_rt_security_headers_policy.x_frame_options)
  );
  if (strcasecmp(g_sec4_rt_security_headers_policy.x_frame_options, "DENY") != 0
      && strcasecmp(g_sec4_rt_security_headers_policy.x_frame_options, "SAMEORIGIN") != 0) {
    strncpy(
        g_sec4_rt_security_headers_policy.x_frame_options,
        "DENY",
        sizeof(g_sec4_rt_security_headers_policy.x_frame_options) - 1
    );
    g_sec4_rt_security_headers_policy.x_frame_options
        [sizeof(g_sec4_rt_security_headers_policy.x_frame_options) - 1] = '\0';
  }
  sec4_rt_read_env_string(
      "SEC4_RT_SECURITY_HEADERS_REFERRER_POLICY",
      "strict-origin-when-cross-origin",
      g_sec4_rt_security_headers_policy.referrer_policy,
      sizeof(g_sec4_rt_security_headers_policy.referrer_policy)
  );
}

static void sec4_rt_load_csrf_policy_from_env(void) {
  memset(&g_sec4_rt_csrf_policy, 0, sizeof(g_sec4_rt_csrf_policy));
  g_sec4_rt_csrf_policy.loaded = true;
  g_sec4_rt_csrf_policy.enabled = sec4_rt_env_flag_enabled_default("SEC4_RT_CSRF_ENABLED", true);
  sec4_rt_read_env_string(
      "SEC4_RT_CSRF_MODE",
      "double_submit",
      g_sec4_rt_csrf_policy.mode,
      sizeof(g_sec4_rt_csrf_policy.mode)
  );
  sec4_rt_read_env_string(
      "SEC4_RT_CSRF_PROTECTED_METHODS",
      "POST,PUT,PATCH,DELETE",
      g_sec4_rt_csrf_policy.protected_methods,
      sizeof(g_sec4_rt_csrf_policy.protected_methods)
  );
  if (strcasecmp(g_sec4_rt_csrf_policy.mode, "off") == 0) {
    g_sec4_rt_csrf_policy.enabled = false;
  }
}

static void sec4_rt_load_auth_policy_from_env(void) {
  memset(&g_sec4_rt_auth_policy, 0, sizeof(g_sec4_rt_auth_policy));
  g_sec4_rt_auth_policy.loaded = true;
  sec4_rt_read_env_string(
      "SEC4_RT_AUTH_MODE",
      "token",
      g_sec4_rt_auth_policy.mode,
      sizeof(g_sec4_rt_auth_policy.mode)
  );
  g_sec4_rt_auth_policy.enabled = strcasecmp(g_sec4_rt_auth_policy.mode, "off") != 0;
}

int64_t sec4_rt_with_cors(int64_t router, int64_t cfg) {
  sec4_rt_router_state *slot = sec4_rt_router_slot(router);
  if (slot == NULL) {
    return 1;
  }
  if (cfg == 0) {
    slot->cors_enabled = false;
    slot->cors_allow_origin[0] = '\0';
    return router;
  }
  if (cfg == SEC4_RT_POLICY_CORS_HANDLE) {
    if (!g_sec4_rt_cors_policy.loaded) {
      sec4_rt_load_cors_policy_from_env();
    }
    slot->cors_enabled = g_sec4_rt_cors_policy.enabled;
    if (!slot->cors_enabled) {
      slot->cors_allow_origin[0] = '\0';
      return router;
    }
    strncpy(
        slot->cors_allow_origin,
        g_sec4_rt_cors_policy.allow_origin,
        sizeof(slot->cors_allow_origin) - 1
    );
    slot->cors_allow_origin[sizeof(slot->cors_allow_origin) - 1] = '\0';
    slot->cors_allow_credentials = g_sec4_rt_cors_policy.allow_credentials;
    slot->cors_require_vary_origin = g_sec4_rt_cors_policy.require_vary_origin;
    strncpy(
        slot->cors_allow_methods,
        "GET, POST, PUT, PATCH, DELETE, OPTIONS",
        sizeof(slot->cors_allow_methods) - 1
    );
    slot->cors_allow_methods[sizeof(slot->cors_allow_methods) - 1] = '\0';
    strncpy(
        slot->cors_allow_headers,
        "content-type, authorization",
        sizeof(slot->cors_allow_headers) - 1
    );
    slot->cors_allow_headers[sizeof(slot->cors_allow_headers) - 1] = '\0';
    slot->cors_max_age_seconds = 600;
    return router;
  }
  sec4_rt_router_apply_default_cors(slot);
  return router;
}

int64_t sec4_rt_with_security_headers(int64_t router, int64_t cfg) {
  sec4_rt_router_state *slot = sec4_rt_router_slot(router);
  if (slot == NULL) {
    return 1;
  }
  if (cfg == 0) {
    slot->security_headers_enabled = false;
    return router;
  }
  if (cfg == SEC4_RT_POLICY_SECURITY_HEADERS_HANDLE) {
    if (!g_sec4_rt_security_headers_policy.loaded) {
      sec4_rt_load_security_headers_policy_from_env();
    }
    slot->security_headers_enabled = g_sec4_rt_security_headers_policy.enabled;
    if (!slot->security_headers_enabled) {
      return router;
    }
    slot->security_x_content_type_options = g_sec4_rt_security_headers_policy.x_content_type_options;
    strncpy(
        slot->security_x_frame_options,
        g_sec4_rt_security_headers_policy.x_frame_options,
        sizeof(slot->security_x_frame_options) - 1
    );
    slot->security_x_frame_options[sizeof(slot->security_x_frame_options) - 1] = '\0';
    strncpy(
        slot->security_referrer_policy,
        g_sec4_rt_security_headers_policy.referrer_policy,
        sizeof(slot->security_referrer_policy) - 1
    );
    slot->security_referrer_policy[sizeof(slot->security_referrer_policy) - 1] = '\0';
    return router;
  }
  sec4_rt_router_apply_default_security_headers(slot);
  return router;
}

int64_t sec4_rt_with_csrf(int64_t router, int64_t cfg) {
  sec4_rt_router_state *slot = sec4_rt_router_slot(router);
  if (slot == NULL) {
    return 1;
  }
  if (cfg == 0) {
    slot->csrf_enabled = false;
    return router;
  }
  if (cfg == SEC4_RT_POLICY_CSRF_HANDLE) {
    if (!g_sec4_rt_csrf_policy.loaded) {
      sec4_rt_load_csrf_policy_from_env();
    }
    slot->csrf_enabled = g_sec4_rt_csrf_policy.enabled;
    if (!slot->csrf_enabled) {
      return router;
    }
    strncpy(
        slot->csrf_mode,
        g_sec4_rt_csrf_policy.mode,
        sizeof(slot->csrf_mode) - 1
    );
    slot->csrf_mode[sizeof(slot->csrf_mode) - 1] = '\0';
    strncpy(
        slot->csrf_protected_methods,
        g_sec4_rt_csrf_policy.protected_methods,
        sizeof(slot->csrf_protected_methods) - 1
    );
    slot->csrf_protected_methods[sizeof(slot->csrf_protected_methods) - 1] = '\0';
    return router;
  }
  sec4_rt_router_apply_default_csrf(slot);
  return router;
}

int64_t sec4_rt_with_auth(int64_t router, int64_t cfg) {
  sec4_rt_router_state *slot = sec4_rt_router_slot(router);
  if (slot == NULL) {
    return 1;
  }
  if (cfg == 0) {
    slot->auth_enabled = false;
    slot->auth_mode[0] = '\0';
    return router;
  }
  if (cfg == SEC4_RT_POLICY_AUTH_HANDLE) {
    if (!g_sec4_rt_auth_policy.loaded) {
      sec4_rt_load_auth_policy_from_env();
    }
    slot->auth_enabled = g_sec4_rt_auth_policy.enabled;
    strncpy(
        slot->auth_mode,
        g_sec4_rt_auth_policy.mode,
        sizeof(slot->auth_mode) - 1
    );
    slot->auth_mode[sizeof(slot->auth_mode) - 1] = '\0';
    return router;
  }
  sec4_rt_router_apply_default_auth(slot);
  return router;
}

int64_t sec4_rt_sec_default_headers(void) {
  sec4_rt_load_security_headers_policy_from_env();
  return SEC4_RT_POLICY_SECURITY_HEADERS_HANDLE;
}

int64_t sec4_rt_sec_csp(void) {
  return sec4_rt_nonzero_constant_handle(UINT64_C(0xB2002));
}

int64_t sec4_rt_sec_csp_add(int64_t csp, const char *directive, const char *value) {
  int64_t directive_handle = sec4_rt_nonzero_handle_from_string(directive, UINT64_C(0xB2003));
  int64_t value_handle = sec4_rt_nonzero_handle_from_string(value, UINT64_C(0xB2004));
  int64_t addition = sec4_rt_handle_from_two(directive_handle, value_handle, UINT64_C(0xB2005));
  return sec4_rt_handle_from_two(csp, addition, UINT64_C(0xB2006));
}

int64_t sec4_rt_cors_from_policy(void) {
  sec4_rt_load_cors_policy_from_env();
  return SEC4_RT_POLICY_CORS_HANDLE;
}

int64_t sec4_rt_cors_origin(int64_t origin) {
  return sec4_rt_handle_from_two(origin, 1, UINT64_C(0xB2008));
}

int64_t sec4_rt_csrf_from_policy(void) {
  sec4_rt_load_csrf_policy_from_env();
  return SEC4_RT_POLICY_CSRF_HANDLE;
}

int64_t sec4_rt_csrf_issue_token(int64_t ctx) {
  (void) ctx;
  char token[128];
  int written = snprintf(
      token,
      sizeof(token),
      "csrf-%s",
      sec4_rt_current_trace_id()
  );
  if (written <= 0 || (size_t) written >= sizeof(token)) {
    return sec4_rt_nonzero_constant_handle(UINT64_C(0xB2010));
  }

  int64_t handle = sec4_rt_nonzero_handle_from_string(token, UINT64_C(0xB2011));
  if (g_sec4_rt_request.has_request) {
    char cookie[192];
    int cookie_written = snprintf(
        cookie,
        sizeof(cookie),
        "csrf=%s; Path=/; SameSite=Lax",
        token
    );
    if (cookie_written > 0 && (size_t) cookie_written < sizeof(cookie)) {
      (void) sec4_rt_append_response_header("Set-Cookie", cookie);
    }
    (void) sec4_rt_append_response_header("X-CSRF-Token", token);
  }
  return handle;
}

int64_t sec4_rt_auth_from_policy(void) {
  sec4_rt_load_auth_policy_from_env();
  return SEC4_RT_POLICY_AUTH_HANDLE;
}

int64_t sec4_rt_auth_require(int64_t ctx) {
  (void) ctx;
  const char *auth_mode = sec4_rt_effective_auth_mode(NULL);
  char auth_subject[256];
  if (!sec4_rt_auth_collect_subject(
          auth_mode,
          auth_subject,
          sizeof(auth_subject),
          NULL
      )) {
    if (g_sec4_rt_request.has_request) {
      sec4_rt_store_std_error_response(
          401,
          "AUTH.UNAUTHORIZED",
          "auth",
          sec4_rt_auth_unauthorized_message(auth_mode)
      );
    }
    return sec4_rt_nonzero_constant_handle(UINT64_C(0xB2013));
  }
  return sec4_rt_nonzero_handle_from_string(auth_subject, UINT64_C(0xB2014));
}

int64_t sec4_rt_auth_require_role(int64_t ctx, const char *required_role) {
  (void) ctx;
  const char *auth_mode = sec4_rt_effective_auth_mode(NULL);
  char auth_subject[256];
  bool used_bearer = false;
  if (!sec4_rt_auth_collect_subject(
          auth_mode,
          auth_subject,
          sizeof(auth_subject),
          &used_bearer
      )) {
    if (g_sec4_rt_request.has_request) {
      sec4_rt_store_std_error_response(
          401,
          "AUTH.UNAUTHORIZED",
          "auth",
          sec4_rt_auth_unauthorized_message(auth_mode)
      );
    }
    return sec4_rt_nonzero_constant_handle(UINT64_C(0xB2015));
  }

  bool has_required_role = used_bearer
      ? sec4_rt_bearer_token_has_role(auth_subject, required_role)
      : sec4_rt_auth_cookie_has_role(required_role);
  if (!has_required_role) {
    const char *forbidden_message = used_bearer
        ? "Authorization token missing required role"
        : "Authenticated cookie principal missing required role";
    if (g_sec4_rt_request.has_request) {
      sec4_rt_store_std_error_response(
          403,
          "AUTH.FORBIDDEN",
          "auth",
          forbidden_message
      );
    }
    int64_t role_handle = sec4_rt_nonzero_handle_from_string(required_role, UINT64_C(0xB2016));
    return sec4_rt_handle_from_two(role_handle, 403, UINT64_C(0xB2017));
  }

  int64_t auth_handle = sec4_rt_nonzero_handle_from_string(auth_subject, UINT64_C(0xB2018));
  int64_t role_handle = sec4_rt_nonzero_handle_from_string(required_role, UINT64_C(0xB2019));
  return sec4_rt_handle_from_two(auth_handle, role_handle, UINT64_C(0xB2020));
}

int64_t sec4_rt_err_validation(const char *code, const char *message) {
  return sec4_rt_emit_error_handle(
      400,
      code != NULL ? code : "VALIDATION.BAD_REQUEST",
      "validation",
      message != NULL ? message : "validation failed",
      UINT64_C(0xC3001)
  );
}

int64_t sec4_rt_err_auth(const char *code, const char *message, int64_t status) {
  int64_t resolved_status = status > 0 ? status : 401;
  return sec4_rt_emit_error_handle(
      resolved_status,
      code != NULL ? code : "AUTH.UNAUTHORIZED",
      "auth",
      message != NULL ? message : "authorization failed",
      UINT64_C(0xC3002)
  );
}

int64_t sec4_rt_err_not_found(const char *code, const char *message) {
  return sec4_rt_emit_error_handle(
      404,
      code != NULL ? code : "RESOURCE.NOT_FOUND",
      "not_found",
      message != NULL ? message : "resource not found",
      UINT64_C(0xC3003)
  );
}

int64_t sec4_rt_err_conflict(const char *code, const char *message) {
  return sec4_rt_emit_error_handle(
      409,
      code != NULL ? code : "RESOURCE.CONFLICT",
      "conflict",
      message != NULL ? message : "conflict",
      UINT64_C(0xC3004)
  );
}

int64_t sec4_rt_err_rate_limit(const char *code, const char *message, int64_t limit) {
  const char *resolved_message = message != NULL ? message : "rate limit exceeded";
  int64_t error_handle = sec4_rt_emit_error_handle(
      429,
      code != NULL ? code : "LIMIT.RATE",
      "rate_limit",
      resolved_message,
      UINT64_C(0xC3005)
  );
  sec4_rt_error_state state;
  sec4_rt_error_load_state(error_handle, &state);
  state.has_limit = true;
  strncpy(state.limit_name, "limit", sizeof(state.limit_name) - 1);
  state.limit_name[sizeof(state.limit_name) - 1] = '\0';
  state.limit_value = limit;
  state.limit_max = limit;
  int64_t next = sec4_rt_handle_from_two(error_handle, limit, UINT64_C(0xC3006));
  state.handle = next;
  sec4_rt_error_store_state(&state);
  if (g_sec4_rt_request.has_request) {
    sec4_rt_error_store_response(&state);
  }
  return next;
}

int64_t sec4_rt_err_internal(const char *message) {
  return sec4_rt_emit_error_handle(
      500,
      "INTERNAL.ERROR",
      "internal",
      message != NULL ? message : "internal error",
      UINT64_C(0xC3007)
  );
}

int64_t sec4_rt_err_with_path(int64_t error, const char *path) {
  sec4_rt_ensure_error_response_in_request();
  sec4_rt_error_state state;
  sec4_rt_error_load_state(error, &state);
  if (path != NULL && path[0] != '\0') {
    strncpy(state.path, path, sizeof(state.path) - 1);
    state.path[sizeof(state.path) - 1] = '\0';
  }
  int64_t path_handle = sec4_rt_nonzero_handle_from_string(path, UINT64_C(0xC3008));
  int64_t next = sec4_rt_handle_from_two(error, path_handle, UINT64_C(0xC3009));
  state.handle = next;
  sec4_rt_error_store_state(&state);
  if (g_sec4_rt_request.has_request) {
    sec4_rt_error_store_response(&state);
  }
  return next;
}

int64_t sec4_rt_err_with_detail(int64_t error, const char *key, int64_t value) {
  sec4_rt_ensure_error_response_in_request();
  sec4_rt_error_state state;
  sec4_rt_error_load_state(error, &state);
  sec4_rt_error_append_detail_entry(&state, key, value);
  int64_t key_handle = sec4_rt_nonzero_handle_from_string(key, UINT64_C(0xC3010));
  int64_t detail_handle = sec4_rt_handle_from_two(key_handle, value, UINT64_C(0xC3011));
  int64_t next = sec4_rt_handle_from_two(error, detail_handle, UINT64_C(0xC3012));
  state.handle = next;
  sec4_rt_error_store_state(&state);
  if (g_sec4_rt_request.has_request) {
    sec4_rt_error_store_response(&state);
  }
  return next;
}

int64_t sec4_rt_err_with_limit(int64_t error, const char *name, int64_t value, int64_t max) {
  sec4_rt_ensure_error_response_in_request();
  sec4_rt_error_state state;
  sec4_rt_error_load_state(error, &state);
  state.has_limit = true;
  strncpy(
      state.limit_name,
      name != NULL && name[0] != '\0' ? name : "limit",
      sizeof(state.limit_name) - 1
  );
  state.limit_name[sizeof(state.limit_name) - 1] = '\0';
  state.limit_value = value;
  state.limit_max = max;
  int64_t limit_name = sec4_rt_nonzero_handle_from_string(name, UINT64_C(0xC3013));
  int64_t observed = sec4_rt_handle_from_two(value, max, UINT64_C(0xC3014));
  int64_t limit_handle = sec4_rt_handle_from_two(limit_name, observed, UINT64_C(0xC3015));
  int64_t next = sec4_rt_handle_from_two(error, limit_handle, UINT64_C(0xC3016));
  state.handle = next;
  sec4_rt_error_store_state(&state);
  if (g_sec4_rt_request.has_request) {
    sec4_rt_error_store_response(&state);
  }
  return next;
}

int64_t sec4_rt_err_with_dependency(
    int64_t error,
    const char *service,
    const char *operation,
    int64_t retryable
) {
  sec4_rt_ensure_error_response_in_request();
  sec4_rt_error_state state;
  sec4_rt_error_load_state(error, &state);
  state.has_dependency = true;
  strncpy(
      state.dependency_service,
      service != NULL && service[0] != '\0' ? service : "dependency",
      sizeof(state.dependency_service) - 1
  );
  state.dependency_service[sizeof(state.dependency_service) - 1] = '\0';
  strncpy(
      state.dependency_operation,
      operation != NULL && operation[0] != '\0' ? operation : "op",
      sizeof(state.dependency_operation) - 1
  );
  state.dependency_operation[sizeof(state.dependency_operation) - 1] = '\0';
  state.dependency_retryable = retryable != 0;
  int64_t service_handle = sec4_rt_nonzero_handle_from_string(service, UINT64_C(0xC3017));
  int64_t op_handle = sec4_rt_nonzero_handle_from_string(operation, UINT64_C(0xC3018));
  int64_t dep_handle = sec4_rt_handle_from_three(
      service_handle,
      op_handle,
      retryable != 0 ? 1 : 0,
      UINT64_C(0xC3019)
  );
  int64_t next = sec4_rt_handle_from_two(error, dep_handle, UINT64_C(0xC3020));
  state.handle = next;
  sec4_rt_error_store_state(&state);
  if (g_sec4_rt_request.has_request) {
    sec4_rt_error_store_response(&state);
  }
  return next;
}

int64_t sec4_rt_err_with_cause(int64_t error, int64_t cause) {
  sec4_rt_ensure_error_response_in_request();
  sec4_rt_error_state state;
  sec4_rt_error_load_state(error, &state);
  state.has_cause = true;
  state.cause_handle = cause;
  int64_t next = sec4_rt_handle_from_two(error, cause, UINT64_C(0xC3021));
  state.handle = next;
  sec4_rt_error_store_state(&state);
  if (g_sec4_rt_request.has_request) {
    sec4_rt_error_store_response(&state);
  }
  return next;
}
