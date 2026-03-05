#include "sec4_runtime.h"

#include <arpa/inet.h>
#include <ctype.h>
#include <errno.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <sys/types.h>
#include <unistd.h>

#define READ_BUF_SIZE 65536
#define BODY_MAX 16384
#define METHOD_MAX 8
#define PATH_MAX_LEN 256
#define TRACE_MAX 64
#define MAX_ROUTES 64
#define ROUTE_PATH_MAX 128
#define MAX_USERS 4096
#define USER_ID_MAX 64
#define MAX_DB_QUERIES 4096
#define MAX_DB_TX 1024
#define MAX_DB_RECORDS 8192
#define DB_TEMPLATE_MAX 256
#define DB_PARAMS_MAX 256

typedef struct {
  char method[METHOD_MAX];
  char path[ROUTE_PATH_MAX];
  sec4_handler_fn handler;
} Route;

typedef struct {
  bool used;
  char id[USER_ID_MAX];
  char body[BODY_MAX];
} UserEntry;

typedef struct {
  bool used;
  int64_t handle;
  char query_template[DB_TEMPLATE_MAX];
  char params[DB_PARAMS_MAX];
} DbQueryEntry;

typedef struct {
  bool used;
  int64_t handle;
  int64_t db;
  int64_t tx;
  char op[16];
  int64_t query_handle;
  char query_template[DB_TEMPLATE_MAX];
  char params[DB_PARAMS_MAX];
} DbRecordEntry;

typedef struct {
  bool used;
  int64_t handle;
  int64_t db;
} DbTxEntry;

typedef struct {
  char method[METHOD_MAX];
  char path[PATH_MAX_LEN];
  char body[BODY_MAX];
  char trace_id[TRACE_MAX];

  bool failed;
  int status;
  char content_type[64];
  char response_body[BODY_MAX];

  bool has_path_id;
  char path_id[USER_ID_MAX];

  bool parsed_user_valid;
  char parsed_user_id[USER_ID_MAX];
  int64_t last_db_record_handle;
  int64_t last_db_query_one_source_handle;
  int64_t last_db_query_one_row_schema;
} RequestContext;

static Route g_routes[MAX_ROUTES];
static size_t g_route_count = 0;

static UserEntry g_users[MAX_USERS];
static DbQueryEntry g_db_queries[MAX_DB_QUERIES];
static DbRecordEntry g_db_records[MAX_DB_RECORDS];
static DbTxEntry g_db_txs[MAX_DB_TX];
static int64_t g_db_query_counter = 0;
static int64_t g_db_record_counter = 0;
static int64_t g_db_tx_counter = 0;
static uint64_t g_trace_counter = 0;
static RequestContext g_ctx;

static int64_t now_ms(void) {
  struct timeval tv;
  gettimeofday(&tv, NULL);
  return ((int64_t)tv.tv_sec * 1000) + (tv.tv_usec / 1000);
}

static void make_trace_id(char *out, size_t out_size) {
  g_trace_counter += 1;
  snprintf(out, out_size, "trace-%llu", (unsigned long long)g_trace_counter);
}

static const char *error_kind_for_status(int status) {
  if (status >= 500) {
    return "internal";
  }
  if (status == 404) {
    return "not_found";
  }
  return "validation";
}

static void set_error_response(int status, const char *code, const char *message) {
  g_ctx.failed = true;
  g_ctx.status = status;
  snprintf(g_ctx.content_type, sizeof(g_ctx.content_type), "%s", "application/json; charset=utf-8");
  snprintf(
      g_ctx.response_body,
      sizeof(g_ctx.response_body),
      "{\"error\":{\"code\":\"%s\",\"kind\":\"%s\",\"message\":\"%s\",\"status\":%d,\"traceId\":\"%s\",\"timeMs\":%lld}}",
      code,
      error_kind_for_status(status),
      message,
      status,
      g_ctx.trace_id,
      (long long)now_ms());
}

static void send_http_response(int fd) {
  if (g_ctx.status <= 0) {
    set_error_response(500, "HTTP.INTERNAL", "handler produced no response");
  }

  const char *reason = "OK";
  if (g_ctx.status == 400) {
    reason = "Bad Request";
  } else if (g_ctx.status == 404) {
    reason = "Not Found";
  } else if (g_ctx.status >= 500) {
    reason = "Internal Server Error";
  }

  size_t body_len = strlen(g_ctx.response_body);
  char header[1024];
  int n = snprintf(
      header,
      sizeof(header),
      "HTTP/1.1 %d %s\r\n"
      "Content-Type: %s\r\n"
      "Content-Length: %zu\r\n"
      "Connection: close\r\n"
      "x-trace-id: %s\r\n"
      "\r\n",
      g_ctx.status,
      reason,
      g_ctx.content_type,
      body_len,
      g_ctx.trace_id);

  if (n > 0) {
    (void)write(fd, header, (size_t)n);
  }
  (void)write(fd, g_ctx.response_body, body_len);
}

static bool is_uuid_v4(const char *value) {
  if (strlen(value) != 36) {
    return false;
  }
  for (int i = 0; i < 36; i++) {
    char c = value[i];
    if (i == 8 || i == 13 || i == 18 || i == 23) {
      if (c != '-') {
        return false;
      }
      continue;
    }
    if (!isxdigit((unsigned char)c)) {
      return false;
    }
  }
  if (value[14] != '4') {
    return false;
  }
  char variant = (char)tolower((unsigned char)value[19]);
  return variant == '8' || variant == '9' || variant == 'a' || variant == 'b';
}

static bool is_email(const char *value) {
  const char *at = strchr(value, '@');
  if (at == NULL || at == value || at[1] == '\0') {
    return false;
  }
  const char *dot = strchr(at + 1, '.');
  if (dot == NULL || dot == at + 1 || dot[1] == '\0') {
    return false;
  }
  return true;
}

static bool is_zip(const char *value) {
  size_t len = strlen(value);
  if (len < 4 || len > 10) {
    return false;
  }
  for (size_t i = 0; i < len; i++) {
    if (!isdigit((unsigned char)value[i])) {
      return false;
    }
  }
  return true;
}

static bool extract_json_string(const char *body, const char *key, char *out, size_t out_size) {
  char needle[64];
  snprintf(needle, sizeof(needle), "\"%s\"", key);

  const char *p = strstr(body, needle);
  if (p == NULL) {
    return false;
  }
  p += strlen(needle);

  while (*p != '\0' && isspace((unsigned char)*p)) {
    p++;
  }
  if (*p != ':') {
    return false;
  }
  p++;

  while (*p != '\0' && isspace((unsigned char)*p)) {
    p++;
  }
  if (*p != '"') {
    return false;
  }
  p++;

  size_t i = 0;
  while (*p != '\0' && *p != '"') {
    if (i + 1 >= out_size) {
      return false;
    }
    out[i++] = *p;
    p++;
  }
  if (*p != '"') {
    return false;
  }
  out[i] = '\0';
  return true;
}

static bool extract_json_int(const char *body, const char *key, int *out) {
  char needle[64];
  snprintf(needle, sizeof(needle), "\"%s\"", key);

  const char *p = strstr(body, needle);
  if (p == NULL) {
    return false;
  }
  p += strlen(needle);

  while (*p != '\0' && isspace((unsigned char)*p)) {
    p++;
  }
  if (*p != ':') {
    return false;
  }
  p++;

  while (*p != '\0' && isspace((unsigned char)*p)) {
    p++;
  }

  char *end_ptr = NULL;
  long parsed = strtol(p, &end_ptr, 10);
  if (end_ptr == p) {
    return false;
  }
  *out = (int)parsed;
  return true;
}

static bool has_bool_key(const char *body, const char *key) {
  char needle_true[64];
  char needle_false[64];
  snprintf(needle_true, sizeof(needle_true), "\"%s\": true", key);
  snprintf(needle_false, sizeof(needle_false), "\"%s\": false", key);
  return strstr(body, needle_true) != NULL || strstr(body, needle_false) != NULL;
}

static bool validate_tags(const char *body) {
  const char *tags = strstr(body, "\"tags\"");
  if (tags == NULL) {
    return false;
  }
  const char *lb = strchr(tags, '[');
  const char *rb = strchr(tags, ']');
  if (lb == NULL || rb == NULL || rb < lb) {
    return false;
  }

  int count = 0;
  const char *p = lb;
  while (p < rb) {
    if (*p == '"') {
      const char *start = p + 1;
      const char *end = strchr(start, '"');
      if (end == NULL || end > rb) {
        return false;
      }
      size_t len = (size_t)(end - start);
      if (len < 1 || len > 32) {
        return false;
      }
      count += 1;
      p = end;
    }
    p++;
  }

  return count <= 16;
}

static bool parse_and_validate_user(const char *body, char *id_out, size_t id_out_size, const char **error_msg) {
  char id[USER_ID_MAX];
  char email[256];
  char zip[32];
  int age = 0;

  if (!extract_json_string(body, "id", id, sizeof(id)) || !is_uuid_v4(id)) {
    *error_msg = "id must be a UUID v4 string";
    return false;
  }
  if (!extract_json_string(body, "email", email, sizeof(email)) || !is_email(email)) {
    *error_msg = "email must be a valid email string";
    return false;
  }
  if (!extract_json_int(body, "age", &age) || age < 0 || age > 150) {
    *error_msg = "age must be an integer between 0 and 150";
    return false;
  }
  if (!validate_tags(body)) {
    *error_msg = "tags must be an array of length <= 16 with values length 1..32";
    return false;
  }
  if (!extract_json_string(body, "zip", zip, sizeof(zip)) || !is_zip(zip)) {
    *error_msg = "address.zip must be a digit string of length 4..10";
    return false;
  }
  if (!has_bool_key(body, "a") || !has_bool_key(body, "b") || !has_bool_key(body, "c")) {
    *error_msg = "meta.flags.a,b,c must be boolean";
    return false;
  }

  snprintf(id_out, id_out_size, "%s", id);
  return true;
}

static UserEntry *find_user(const char *id) {
  for (size_t i = 0; i < MAX_USERS; i++) {
    if (g_users[i].used && strcmp(g_users[i].id, id) == 0) {
      return &g_users[i];
    }
  }
  return NULL;
}

static bool upsert_user(const char *id, const char *body) {
  UserEntry *existing = find_user(id);
  if (existing != NULL) {
    snprintf(existing->body, sizeof(existing->body), "%s", body);
    return true;
  }

  for (size_t i = 0; i < MAX_USERS; i++) {
    if (!g_users[i].used) {
      g_users[i].used = true;
      snprintf(g_users[i].id, sizeof(g_users[i].id), "%s", id);
      snprintf(g_users[i].body, sizeof(g_users[i].body), "%s", body);
      return true;
    }
  }
  return false;
}

static DbQueryEntry *find_db_query(int64_t handle) {
  for (size_t i = 0; i < MAX_DB_QUERIES; i++) {
    if (g_db_queries[i].used && g_db_queries[i].handle == handle) {
      return &g_db_queries[i];
    }
  }
  return NULL;
}

static DbTxEntry *find_db_tx(int64_t handle) {
  for (size_t i = 0; i < MAX_DB_TX; i++) {
    if (g_db_txs[i].used && g_db_txs[i].handle == handle) {
      return &g_db_txs[i];
    }
  }
  return NULL;
}

static DbRecordEntry *find_db_record(int64_t handle) {
  for (size_t i = 0; i < MAX_DB_RECORDS; i++) {
    if (g_db_records[i].used && g_db_records[i].handle == handle) {
      return &g_db_records[i];
    }
  }
  return NULL;
}

static DbRecordEntry *latest_db_record_for_query(int64_t db, int64_t query_handle) {
  for (size_t i = MAX_DB_RECORDS; i > 0; i--) {
    DbRecordEntry *entry = &g_db_records[i - 1];
    if (!entry->used) {
      continue;
    }
    if (entry->db == db && entry->query_handle == query_handle) {
      return entry;
    }
  }
  return NULL;
}

static DbRecordEntry *append_db_record(
    int64_t db,
    int64_t tx,
    const char *op,
    int64_t query_handle,
    const char *query_template,
    const char *params) {
  for (size_t i = 0; i < MAX_DB_RECORDS; i++) {
    if (g_db_records[i].used) {
      continue;
    }
    g_db_record_counter += 1;
    DbRecordEntry *entry = &g_db_records[i];
    entry->used = true;
    entry->handle = g_db_record_counter;
    entry->db = db;
    entry->tx = tx;
    entry->query_handle = query_handle;
    snprintf(entry->op, sizeof(entry->op), "%s", op != NULL ? op : "exec");
    snprintf(
        entry->query_template,
        sizeof(entry->query_template),
        "%s",
        query_template != NULL ? query_template : "");
    snprintf(entry->params, sizeof(entry->params), "%s", params != NULL ? params : "");
    return entry;
  }
  return NULL;
}

static bool read_request(
    int fd,
    char *method,
    size_t method_size,
    char *path,
    size_t path_size,
    char *body,
    size_t body_size) {
  (void)method_size;
  (void)path_size;

  char buffer[READ_BUF_SIZE];
  size_t total = 0;
  ssize_t n = 0;
  const char *headers_end = NULL;

  while (true) {
    if (total >= sizeof(buffer) - 1) {
      return false;
    }
    n = read(fd, buffer + total, sizeof(buffer) - 1 - total);
    if (n <= 0) {
      return false;
    }
    total += (size_t)n;
    buffer[total] = '\0';

    headers_end = strstr(buffer, "\r\n\r\n");
    if (headers_end != NULL) {
      break;
    }
  }

  char *line_end = strstr(buffer, "\r\n");
  if (line_end == NULL) {
    return false;
  }
  *line_end = '\0';
  if (sscanf(buffer, "%7s %255s", method, path) != 2) {
    return false;
  }
  *line_end = '\r';

  size_t header_len = (size_t)(headers_end - buffer) + 4;
  int content_length = 0;

  char headers_copy[READ_BUF_SIZE];
  if (header_len >= sizeof(headers_copy)) {
    return false;
  }
  memcpy(headers_copy, buffer, header_len);
  headers_copy[header_len] = '\0';

  char *line = strtok(headers_copy, "\r\n");
  while (line != NULL) {
    if (strncasecmp(line, "Content-Length:", 15) == 0) {
      content_length = atoi(line + 15);
      if (content_length < 0 || content_length > (int)(body_size - 1)) {
        return false;
      }
      break;
    }
    line = strtok(NULL, "\r\n");
  }

  size_t already_body = total - header_len;
  while ((int)already_body < content_length) {
    if (total >= sizeof(buffer) - 1) {
      return false;
    }
    n = read(fd, buffer + total, sizeof(buffer) - 1 - total);
    if (n <= 0) {
      return false;
    }
    total += (size_t)n;
    already_body += (size_t)n;
  }

  if (content_length > 0) {
    memcpy(body, buffer + header_len, (size_t)content_length);
  }
  body[content_length] = '\0';
  return true;
}

static bool match_route(const char *method, const char *path, Route **route_out) {
  for (size_t i = 0; i < g_route_count; i++) {
    Route *r = &g_routes[i];
    if (strcmp(r->method, method) != 0) {
      continue;
    }

    const char *param_pos = strstr(r->path, ":id");
    if (param_pos == NULL) {
      if (strcmp(r->path, path) == 0) {
        g_ctx.has_path_id = false;
        *route_out = r;
        return true;
      }
      continue;
    }

    size_t prefix_len = (size_t)(param_pos - r->path);
    if (strncmp(path, r->path, prefix_len) != 0) {
      continue;
    }

    const char *actual_id = path + prefix_len;
    if (*actual_id == '\0') {
      continue;
    }
    if (strlen(actual_id) >= sizeof(g_ctx.path_id)) {
      continue;
    }

    g_ctx.has_path_id = true;
    snprintf(g_ctx.path_id, sizeof(g_ctx.path_id), "%s", actual_id);
    *route_out = r;
    return true;
  }

  return false;
}

int64_t sec4_rt_identity_i64(int64_t value) {
  return value;
}

bool sec4_rt_identity_bool(bool value) {
  return value;
}

int64_t sec4_rt_http_router(void) {
  return 1;
}

static int64_t add_route(const char *method, const char *path, sec4_handler_fn handler) {
  if (g_route_count >= MAX_ROUTES) {
    return 0;
  }
  Route *r = &g_routes[g_route_count++];
  snprintf(r->method, sizeof(r->method), "%s", method);
  snprintf(r->path, sizeof(r->path), "%s", path);
  r->handler = handler;
  return 1;
}

int64_t sec4_rt_http_route_get(int64_t router, const char *path, sec4_handler_fn handler) {
  (void)router;
  return add_route("GET", path, handler);
}

int64_t sec4_rt_http_route_post(int64_t router, const char *path, sec4_handler_fn handler) {
  (void)router;
  return add_route("POST", path, handler);
}

int64_t sec4_rt_req_json(const char *schema_name) {
  (void)schema_name;
  if (g_ctx.failed) {
    return 0;
  }

  const char *error_msg = NULL;
  char id[USER_ID_MAX];
  if (!parse_and_validate_user(g_ctx.body, id, sizeof(id), &error_msg)) {
    set_error_response(400, "VALIDATION.INVALID", error_msg != NULL ? error_msg : "invalid payload");
    return 0;
  }

  g_ctx.parsed_user_valid = true;
  snprintf(g_ctx.parsed_user_id, sizeof(g_ctx.parsed_user_id), "%s", id);
  return 1;
}

int64_t sec4_rt_req_path_param(const char *name) {
  (void)name;
  if (g_ctx.failed) {
    return 0;
  }
  if (!g_ctx.has_path_id) {
    set_error_response(400, "VALIDATION.UUID_INVALID", "id must be UUID v4");
    return 0;
  }
  if (!is_uuid_v4(g_ctx.path_id)) {
    set_error_response(400, "VALIDATION.UUID_INVALID", "id must be UUID v4");
    return 0;
  }
  return 1;
}

int64_t sec4_rt_db_cap(void) {
  if (g_ctx.failed) {
    return 0;
  }
  return 1;
}

int64_t sec4_rt_sql_q(const char *query_template, const char *params) {
  if (g_ctx.failed) {
    return 0;
  }
  if (query_template == NULL || query_template[0] == '\0') {
    set_error_response(400, "DB.QUERY_INVALID", "query template must not be empty");
    return 0;
  }
  for (size_t i = 0; i < MAX_DB_QUERIES; i++) {
    if (g_db_queries[i].used) {
      continue;
    }
    g_db_query_counter += 1;
    g_db_queries[i].used = true;
    g_db_queries[i].handle = g_db_query_counter;
    snprintf(
        g_db_queries[i].query_template,
        sizeof(g_db_queries[i].query_template),
        "%s",
        query_template);
    snprintf(g_db_queries[i].params, sizeof(g_db_queries[i].params), "%s", params != NULL ? params : "");
    return g_db_queries[i].handle;
  }
  set_error_response(500, "DB.QUERY_CAPACITY", "query handle capacity exceeded");
  return 0;
}

int64_t sec4_rt_db_exec(int64_t db, int64_t query) {
  if (g_ctx.failed) {
    return 0;
  }
  DbQueryEntry *query_entry = find_db_query(query);
  if (query_entry == NULL) {
    set_error_response(400, "DB.QUERY_INVALID", "db.exec query handle not found");
    return 0;
  }
  DbRecordEntry *record = append_db_record(
      db, 0, "exec", query, query_entry->query_template, query_entry->params);
  if (record == NULL) {
    set_error_response(500, "DB.RECORD_CAPACITY", "db record capacity exceeded");
    return 0;
  }
  g_ctx.last_db_record_handle = record->handle;
  g_ctx.last_db_query_one_source_handle = 0;
  g_ctx.last_db_query_one_row_schema = 0;
  return record->handle;
}

int64_t sec4_rt_db_tx(int64_t db) {
  if (g_ctx.failed) {
    return 0;
  }
  for (size_t i = 0; i < MAX_DB_TX; i++) {
    if (g_db_txs[i].used) {
      continue;
    }
    g_db_tx_counter += 1;
    g_db_txs[i].used = true;
    g_db_txs[i].handle = g_db_tx_counter;
    g_db_txs[i].db = db;
    return g_db_txs[i].handle;
  }
  set_error_response(500, "DB.TX_CAPACITY", "db tx capacity exceeded");
  return 0;
}

int64_t sec4_rt_db_exec_tx(int64_t tx, int64_t query) {
  if (g_ctx.failed) {
    return 0;
  }
  DbTxEntry *tx_entry = find_db_tx(tx);
  if (tx_entry == NULL) {
    set_error_response(400, "DB.TX_INVALID", "db.execTx tx handle not found");
    return 0;
  }
  DbQueryEntry *query_entry = find_db_query(query);
  if (query_entry == NULL) {
    set_error_response(400, "DB.QUERY_INVALID", "db.execTx query handle not found");
    return 0;
  }
  DbRecordEntry *record = append_db_record(
      tx_entry->db, tx, "execTx", query, query_entry->query_template, query_entry->params);
  if (record == NULL) {
    set_error_response(500, "DB.RECORD_CAPACITY", "db record capacity exceeded");
    return 0;
  }
  g_ctx.last_db_record_handle = record->handle;
  g_ctx.last_db_query_one_source_handle = 0;
  g_ctx.last_db_query_one_row_schema = 0;
  return record->handle;
}

int64_t sec4_rt_db_query_one(int64_t db, int64_t query, int64_t row_schema) {
  if (g_ctx.failed) {
    return 0;
  }
  DbQueryEntry *query_entry = find_db_query(query);
  if (query_entry == NULL) {
    set_error_response(400, "DB.QUERY_INVALID", "db.queryOne query handle not found");
    return 0;
  }
  DbRecordEntry *source = latest_db_record_for_query(db, query);
  if (source == NULL) {
    set_error_response(404, "DB.QUERY_ONE_NOT_FOUND", "db.queryOne record not found");
    return 0;
  }

  DbRecordEntry *query_record = append_db_record(
      db, 0, "queryOne", query, query_entry->query_template, query_entry->params);
  if (query_record == NULL) {
    set_error_response(500, "DB.RECORD_CAPACITY", "db record capacity exceeded");
    return 0;
  }
  g_ctx.last_db_record_handle = query_record->handle;
  g_ctx.last_db_query_one_source_handle = source->handle;
  g_ctx.last_db_query_one_row_schema = row_schema;
  return query_record->handle;
}

int64_t sec4_rt_schema_row(int64_t value) {
  return value;
}

int64_t sec4_rt_res_text(int64_t status, const char *text) {
  if (g_ctx.failed) {
    return 0;
  }
  g_ctx.status = (int)status;
  snprintf(g_ctx.content_type, sizeof(g_ctx.content_type), "%s", "text/plain; charset=utf-8");
  snprintf(g_ctx.response_body, sizeof(g_ctx.response_body), "%s", text);
  return 1;
}

int64_t sec4_rt_res_ok(int64_t status, const char *schema_name, int64_t value) {
  (void)schema_name;
  (void)value;

  if (g_ctx.failed) {
    return 0;
  }
  if (!g_ctx.parsed_user_valid) {
    set_error_response(500, "HTTP.INTERNAL", "req.json must succeed before res.ok");
    return 0;
  }

  g_ctx.status = (int)status;
  snprintf(g_ctx.content_type, sizeof(g_ctx.content_type), "%s", "application/json; charset=utf-8");

  if (status == 201) {
    if (!upsert_user(g_ctx.parsed_user_id, g_ctx.body)) {
      set_error_response(500, "HTTP.INTERNAL", "user store capacity exceeded");
      return 0;
    }
    snprintf(g_ctx.response_body, sizeof(g_ctx.response_body), "{\"ok\":true,\"userId\":\"%s\"}", g_ctx.parsed_user_id);
  } else {
    snprintf(g_ctx.response_body, sizeof(g_ctx.response_body), "{\"ok\":true,\"id\":\"%s\"}", g_ctx.parsed_user_id);
  }

  return 1;
}

int64_t sec4_rt_res_json(int64_t status, const char *schema_name, int64_t value) {
  (void)value;
  if (g_ctx.failed) {
    return 0;
  }

  g_ctx.status = (int)status;
  snprintf(g_ctx.content_type, sizeof(g_ctx.content_type), "%s", "application/json; charset=utf-8");

  if (schema_name != NULL && strcmp(schema_name, "DbExecResponse") == 0) {
    DbRecordEntry *record = find_db_record(g_ctx.last_db_record_handle);
    if (record == NULL) {
      set_error_response(500, "DB.EXEC_MISSING", "db.exec did not produce a record");
      return 0;
    }
    snprintf(
        g_ctx.response_body,
        sizeof(g_ctx.response_body),
        "{\"ok\":true,\"recordId\":%lld,\"db\":%lld,\"tx\":%lld,\"op\":\"%s\",\"template\":\"%s\",\"params\":\"%s\",\"affectedRows\":0}",
        (long long)record->handle,
        (long long)record->db,
        (long long)record->tx,
        record->op,
        record->query_template,
        record->params);
    return 1;
  }
  if (schema_name != NULL && strcmp(schema_name, "DbExecTxResponse") == 0) {
    DbRecordEntry *record = find_db_record(g_ctx.last_db_record_handle);
    if (record == NULL) {
      set_error_response(500, "DB.EXEC_TX_MISSING", "db.execTx did not produce a record");
      return 0;
    }
    snprintf(
        g_ctx.response_body,
        sizeof(g_ctx.response_body),
        "{\"ok\":true,\"recordId\":%lld,\"db\":%lld,\"tx\":%lld,\"op\":\"%s\",\"template\":\"%s\",\"params\":\"%s\",\"affectedRows\":0}",
        (long long)record->handle,
        (long long)record->db,
        (long long)record->tx,
        record->op,
        record->query_template,
        record->params);
    return 1;
  }
  if (schema_name != NULL && strcmp(schema_name, "DbQueryOneResponse") == 0) {
    DbRecordEntry *record = find_db_record(g_ctx.last_db_record_handle);
    DbRecordEntry *source = find_db_record(g_ctx.last_db_query_one_source_handle);
    if (record == NULL || source == NULL) {
      set_error_response(500, "DB.QUERY_ONE_MISSING", "db.queryOne did not produce a record");
      return 0;
    }
    snprintf(
        g_ctx.response_body,
        sizeof(g_ctx.response_body),
        "{\"ok\":true,\"recordId\":%lld,\"rowSchema\":%lld,\"record\":{\"id\":%lld,\"db\":%lld,\"tx\":%lld,\"op\":\"%s\",\"template\":\"%s\",\"params\":\"%s\"}}",
        (long long)record->handle,
        (long long)g_ctx.last_db_query_one_row_schema,
        (long long)source->handle,
        (long long)source->db,
        (long long)source->tx,
        source->op,
        source->query_template,
        source->params);
    return 1;
  }
  if (schema_name != NULL && strcmp(schema_name, "DbListRecordsResponse") == 0) {
    int64_t count = 0;
    DbRecordEntry *latest = NULL;
    for (size_t i = 0; i < MAX_DB_RECORDS; i++) {
      if (!g_db_records[i].used) {
        continue;
      }
      count += 1;
      if (latest == NULL || g_db_records[i].handle > latest->handle) {
        latest = &g_db_records[i];
      }
    }
    if (latest == NULL) {
      snprintf(g_ctx.response_body, sizeof(g_ctx.response_body), "%s", "{\"ok\":true,\"count\":0}");
    } else {
      snprintf(
          g_ctx.response_body,
          sizeof(g_ctx.response_body),
          "{\"ok\":true,\"count\":%lld,\"latest\":{\"recordId\":%lld,\"db\":%lld,\"tx\":%lld,\"op\":\"%s\",\"template\":\"%s\",\"params\":\"%s\"}}",
          (long long)count,
          (long long)latest->handle,
          (long long)latest->db,
          (long long)latest->tx,
          latest->op,
          latest->query_template,
          latest->params);
    }
    return 1;
  }

  if (!g_ctx.has_path_id) {
    set_error_response(400, "VALIDATION.UUID_INVALID", "id must be UUID v4");
    return 0;
  }
  UserEntry *entry = find_user(g_ctx.path_id);
  if (entry == NULL) {
    set_error_response(404, "HTTP.NOT_FOUND", "user not found");
    return 0;
  }

  snprintf(g_ctx.response_body, sizeof(g_ctx.response_body), "%s", entry->body);
  return 1;
}

int64_t sec4_rt_http_serve(int64_t port, int64_t router) {
  (void)router;
  signal(SIGPIPE, SIG_IGN);

  int listen_port = (int)port;
  const char *port_env = getenv("PORT");
  if (port_env != NULL && port_env[0] != '\0') {
    listen_port = atoi(port_env);
  }

  int server_fd = socket(AF_INET, SOCK_STREAM, 0);
  if (server_fd < 0) {
    return 0;
  }

  int opt = 1;
  if (setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt)) != 0) {
    close(server_fd);
    return 0;
  }

  struct sockaddr_in addr;
  memset(&addr, 0, sizeof(addr));
  addr.sin_family = AF_INET;
  addr.sin_addr.s_addr = htonl(INADDR_ANY);
  addr.sin_port = htons((uint16_t)listen_port);

  if (bind(server_fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
    close(server_fd);
    return 0;
  }

  if (listen(server_fd, 128) != 0) {
    close(server_fd);
    return 0;
  }

  fprintf(stderr, "sec4 benchmark service listening on :%d\n", listen_port);

  while (true) {
    int client_fd = accept(server_fd, NULL, NULL);
    if (client_fd < 0) {
      if (errno == EINTR) {
        continue;
      }
      break;
    }

    memset(&g_ctx, 0, sizeof(g_ctx));
    make_trace_id(g_ctx.trace_id, sizeof(g_ctx.trace_id));

    if (!read_request(client_fd, g_ctx.method, sizeof(g_ctx.method), g_ctx.path, sizeof(g_ctx.path), g_ctx.body, sizeof(g_ctx.body))) {
      set_error_response(400, "HTTP.BAD_REQUEST", "invalid request");
      send_http_response(client_fd);
      close(client_fd);
      continue;
    }

    Route *route = NULL;
    if (!match_route(g_ctx.method, g_ctx.path, &route)) {
      set_error_response(404, "HTTP.NOT_FOUND", "route not found");
      send_http_response(client_fd);
      close(client_fd);
      continue;
    }

    route->handler();
    send_http_response(client_fd);
    close(client_fd);
  }

  close(server_fd);
  return 1;
}
