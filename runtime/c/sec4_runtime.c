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
#define SEC4_RT_MAX_TRACKED_VALUES 256
#define SEC4_RT_MAX_TRACKED_VALUE_BYTES 1024
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
  bool cors_enabled;
  bool security_headers_enabled;
  bool csrf_enabled;
  bool auth_enabled;
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
} sec4_rt_request_state;

typedef struct {
  bool active;
  int64_t handle;
  char value[SEC4_RT_MAX_TRACKED_VALUE_BYTES];
} sec4_rt_tracked_value;

static sec4_rt_router_state g_sec4_rt_routers[SEC4_RT_MAX_ROUTERS];
static int64_t g_sec4_rt_next_router_handle = 1;
static sec4_rt_response_state g_sec4_rt_response;
static sec4_rt_request_state g_sec4_rt_request;
static sec4_rt_tracked_value g_sec4_rt_tracked_values[SEC4_RT_MAX_TRACKED_VALUES];
static uint64_t g_sec4_rt_next_trace_id = 1;

static const char *sec4_rt_current_trace_id(void);
static void sec4_rt_assign_trace_id(void);

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
      "{\"error\":{\"code\":\"%s\",\"kind\":\"%s\",\"message\":\"%s\",\"status\":%lld,\"traceId\":\"%s\",\"timeMs\":0}}",
      code != NULL ? code : "INTERNAL.ERROR",
      kind != NULL ? kind : "internal",
      message != NULL ? message : "internal error",
      (long long) status,
      sec4_rt_current_trace_id()
  );

  if (written <= 0 || (size_t) written >= sizeof(payload)) {
    sec4_rt_store_response(
        status,
        "application/json; charset=utf-8",
        "{\"error\":{\"code\":\"INTERNAL.ERROR\",\"kind\":\"internal\",\"message\":\"error\",\"status\":500,\"traceId\":\"rt-0\",\"timeMs\":0}}"
    );
    return;
  }

  sec4_rt_store_response(status, "application/json; charset=utf-8", payload);
}

static void sec4_rt_store_std_success_response(int64_t status, bool include_meta) {
  char payload[768];
  int written = 0;
  if (include_meta) {
    written = snprintf(
        payload,
        sizeof(payload),
        "{\"ok\":true,\"status\":%lld,\"traceId\":\"%s\",\"timeMs\":0,\"data\":{},\"meta\":{}}",
        (long long) status,
        sec4_rt_current_trace_id()
    );
  } else {
    written = snprintf(
        payload,
        sizeof(payload),
        "{\"ok\":true,\"status\":%lld,\"traceId\":\"%s\",\"timeMs\":0,\"data\":{}}",
        (long long) status,
        sec4_rt_current_trace_id()
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

static int64_t sec4_rt_track_string_value(const char *value, uint64_t salt) {
  if (value == NULL || value[0] == '\0') {
    return 0;
  }

  int64_t handle = sec4_rt_gate_handle_from_string(value, salt);
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

    if (slot->handle == handle && strcmp(slot->value, value) == 0) {
      return handle;
    }
  }

  size_t target = first_free == SEC4_RT_MAX_TRACKED_VALUES
      ? (size_t) (((uint64_t) handle) % SEC4_RT_MAX_TRACKED_VALUES)
      : first_free;
  sec4_rt_tracked_value *slot = &g_sec4_rt_tracked_values[target];
  slot->active = true;
  slot->handle = handle;
  strncpy(slot->value, value, sizeof(slot->value) - 1);
  slot->value[sizeof(slot->value) - 1] = '\0';
  return handle;
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

static bool sec4_rt_parse_ipv4_private(const char *host, size_t host_len) {
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

  int a = octets[0];
  int b = octets[1];
  if (a == 10 || a == 127) {
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
  if (!is_https) {
    return false;
  }
  if (sec4_rt_host_is_internal(host, host_len)) {
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
  return sec4_rt_host_is_internal(host, host_len);
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

static bool sec4_rt_is_path_base_valid(const char *value) {
  if (value == NULL || value[0] == '\0') {
    return false;
  }
  if (value[0] != '/') {
    return false;
  }
  if (strstr(value, "..") != NULL) {
    return false;
  }
  if (strchr(value, '\r') != NULL || strchr(value, '\n') != NULL) {
    return false;
  }
  return true;
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

static bool sec4_rt_is_csrf_protected_method(const char *method) {
  if (method == NULL) {
    return false;
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

static const char *sec4_rt_security_headers_block(sec4_rt_router_state *router) {
  if (router == NULL || !router->security_headers_enabled) {
    return NULL;
  }
  return
      "X-Content-Type-Options: nosniff\r\n"
      "X-Frame-Options: DENY\r\n"
      "Referrer-Policy: strict-origin-when-cross-origin\r\n";
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
  const char *security_headers = sec4_rt_security_headers_block(router);
  const char *cors_headers = router != NULL && router->cors_enabled
      ? "Access-Control-Allow-Origin: *\r\n"
      : NULL;
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
    const char *extra_headers =
        "Access-Control-Allow-Origin: *\r\n"
        "Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS\r\n"
        "Access-Control-Allow-Headers: content-type, authorization\r\n"
        "Access-Control-Max-Age: 600\r\n";
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
    char auth_header[256];
    bool has_auth = sec4_rt_parse_header_value(
        request,
        headers_len,
        "Authorization",
        auth_header,
        sizeof(auth_header)
    );
    bool valid_auth = has_auth
        && strncasecmp(auth_header, "Bearer ", 7) == 0
        && auth_header[7] != '\0';
    if (!valid_auth) {
      sec4_rt_store_std_error_response(
          401,
          "AUTH.UNAUTHORIZED",
          "auth",
          "Authorization header missing or invalid"
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

  if (router->csrf_enabled && sec4_rt_is_csrf_protected_method(method)) {
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
        NULL,
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
  (void) schema;
  (void) value;
  if (g_sec4_rt_request.json_checked && !g_sec4_rt_request.json_valid) {
    return 1;
  }
  sec4_rt_store_std_success_response(200, false);
  return 0;
}

int64_t sec4_rt_res_ok(int64_t status, int64_t schema, int64_t value) {
  (void) schema;
  (void) value;
  if (g_sec4_rt_request.json_checked && !g_sec4_rt_request.json_valid) {
    return 1;
  }
  sec4_rt_store_std_success_response(status > 0 ? status : 201, false);
  return 0;
}

int64_t sec4_rt_res_ok_meta(int64_t status, int64_t schema, int64_t value, int64_t meta) {
  (void) schema;
  (void) value;
  (void) meta;
  if (g_sec4_rt_request.json_checked && !g_sec4_rt_request.json_valid) {
    return 1;
  }
  sec4_rt_store_std_success_response(status > 0 ? status : 201, true);
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

int64_t sec4_rt_validate_header_value(int64_t input) {
  return sec4_rt_gate_handle_from_input(input, UINT64_C(0x40404));
}

int64_t sec4_rt_validate_email(int64_t input) {
  return sec4_rt_gate_handle_from_input(input, UINT64_C(0x50505));
}

int64_t sec4_rt_validate_uuid(int64_t input) {
  return sec4_rt_gate_handle_from_input(input, UINT64_C(0x60606));
}

int64_t sec4_rt_validate_int64(int64_t input) {
  return sec4_rt_gate_handle_from_input(input, UINT64_C(0x70707));
}

int64_t sec4_rt_validate_non_empty(int64_t input) {
  return sec4_rt_gate_handle_from_input(input, UINT64_C(0x80808));
}

int64_t sec4_rt_sanitize_html(int64_t input) {
  return sec4_rt_gate_handle_from_input(input, UINT64_C(0x90909));
}

int64_t sec4_rt_url_public(int64_t input) {
  const char *url = sec4_rt_lookup_tracked_value(input);
  if (url == NULL) {
    return sec4_rt_gate_handle_from_input(input, UINT64_C(0xA0A0A));
  }
  if (!sec4_rt_is_public_url_valid(url)) {
    return 0;
  }
  return sec4_rt_track_string_value(url, UINT64_C(0xA0A0A));
}

int64_t sec4_rt_url_internal(int64_t input) {
  const char *url = sec4_rt_lookup_tracked_value(input);
  if (url == NULL) {
    return sec4_rt_gate_handle_from_input(input, UINT64_C(0xB0B0B));
  }
  if (!sec4_rt_is_internal_url_valid(url)) {
    return 0;
  }
  return sec4_rt_track_string_value(url, UINT64_C(0xB0B0B));
}

int64_t sec4_rt_path_under(int64_t base, int64_t input) {
  if (base == 0 || input == 0) {
    return 0;
  }
  return sec4_rt_hash_token(((uint64_t) base) ^ ((uint64_t) input), UINT64_C(0xC0C0C));
}

int64_t sec4_rt_path_base(const char *input) {
  if (!sec4_rt_is_path_base_valid(input)) {
    return 0;
  }
  return sec4_rt_gate_handle_from_string(input, UINT64_C(0xD0D0D));
}

int64_t sec4_rt_headers_name(const char *input) {
  if (!sec4_rt_is_header_name_valid(input)) {
    return 0;
  }
  return sec4_rt_gate_handle_from_string(input, UINT64_C(0xE0E0E));
}

int64_t sec4_rt_headers_value(const char *input) {
  if (!sec4_rt_is_header_value_valid(input)) {
    return 0;
  }
  return sec4_rt_gate_handle_from_string(input, UINT64_C(0xF0F0F));
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

int64_t sec4_rt_with_cors(int64_t router, int64_t cfg) {
  (void) cfg;
  sec4_rt_router_state *slot = sec4_rt_router_slot(router);
  if (slot != NULL) {
    slot->cors_enabled = true;
  }
  return router;
}

int64_t sec4_rt_with_security_headers(int64_t router, int64_t cfg) {
  (void) cfg;
  sec4_rt_router_state *slot = sec4_rt_router_slot(router);
  if (slot != NULL) {
    slot->security_headers_enabled = true;
  }
  return router;
}

int64_t sec4_rt_with_csrf(int64_t router, int64_t cfg) {
  (void) cfg;
  sec4_rt_router_state *slot = sec4_rt_router_slot(router);
  if (slot != NULL) {
    slot->csrf_enabled = true;
  }
  return router;
}

int64_t sec4_rt_with_auth(int64_t router, int64_t cfg) {
  (void) cfg;
  sec4_rt_router_state *slot = sec4_rt_router_slot(router);
  if (slot != NULL) {
    slot->auth_enabled = true;
  }
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
