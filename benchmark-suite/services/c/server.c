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
#include <time.h>
#include <unistd.h>

#define READ_BUF_SIZE 65536
#define BODY_MAX 16384
#define METHOD_MAX 16
#define PATH_MAX_LEN 256
#define TRACE_MAX 64
#define MAX_USERS 4096
#define USER_ID_MAX 64

typedef struct {
  bool used;
  char id[USER_ID_MAX];
  char body[BODY_MAX];
} UserEntry;

static UserEntry g_users[MAX_USERS];
static uint64_t g_trace_counter = 0;

static int64_t now_ms(void) {
  struct timeval tv;
  gettimeofday(&tv, NULL);
  return ((int64_t)tv.tv_sec * 1000) + (tv.tv_usec / 1000);
}

static void make_trace_id(char *out, size_t out_size) {
  g_trace_counter += 1;
  snprintf(out, out_size, "trace-%llu", (unsigned long long)g_trace_counter);
}

static void send_response(int fd, int status, const char *content_type, const char *trace_id, const char *body) {
  size_t body_len = strlen(body);
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
      status,
      (status == 200 || status == 201) ? "OK" : (status == 404 ? "Not Found" : "Bad Request"),
      content_type,
      body_len,
      trace_id);

  if (n > 0) {
    (void)write(fd, header, (size_t)n);
  }
  (void)write(fd, body, body_len);
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

static void send_error(int fd, int status, const char *trace_id, const char *code, const char *message) {
  char body[1024];
  snprintf(
      body,
      sizeof(body),
      "{\"error\":{\"code\":\"%s\",\"kind\":\"%s\",\"message\":\"%s\",\"status\":%d,\"traceId\":\"%s\",\"timeMs\":%lld}}",
      code,
      error_kind_for_status(status),
      message,
      status,
      trace_id,
      (long long)now_ms());
  send_response(fd, status, "application/json; charset=utf-8", trace_id, body);
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

static bool validate_user_payload(const char *body, char *id_out, size_t id_out_size, const char **error_msg) {
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

static bool read_request(
    int fd,
    char *method,
    size_t method_size,
    char *path,
    size_t path_size,
    char *body,
    size_t body_size) {
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

  if (sscanf(buffer, "%15s %255s", method, path) != 2) {
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

static void handle_connection(int fd) {
  char method[METHOD_MAX] = {0};
  char path[PATH_MAX_LEN] = {0};
  char body[BODY_MAX] = {0};
  char trace_id[TRACE_MAX] = {0};

  make_trace_id(trace_id, sizeof(trace_id));

  if (!read_request(fd, method, sizeof(method), path, sizeof(path), body, sizeof(body))) {
    send_error(fd, 400, trace_id, "HTTP.BAD_REQUEST", "invalid request");
    return;
  }

  if (strcmp(method, "GET") == 0 && strcmp(path, "/ping") == 0) {
    send_response(fd, 200, "text/plain; charset=utf-8", trace_id, "ok");
    return;
  }

  if (strcmp(method, "POST") == 0 && strcmp(path, "/decode") == 0) {
    const char *error_msg = NULL;
    char id[USER_ID_MAX] = {0};
    if (!validate_user_payload(body, id, sizeof(id), &error_msg)) {
      send_error(fd, 400, trace_id, "VALIDATION.INVALID", error_msg != NULL ? error_msg : "invalid payload");
      return;
    }

    char payload[512];
    snprintf(payload, sizeof(payload), "{\"ok\":true,\"id\":\"%s\"}", id);
    send_response(fd, 200, "application/json; charset=utf-8", trace_id, payload);
    return;
  }

  if (strcmp(method, "POST") == 0 && strcmp(path, "/users") == 0) {
    const char *error_msg = NULL;
    char id[USER_ID_MAX] = {0};
    if (!validate_user_payload(body, id, sizeof(id), &error_msg)) {
      send_error(fd, 400, trace_id, "VALIDATION.INVALID", error_msg != NULL ? error_msg : "invalid payload");
      return;
    }

    if (!upsert_user(id, body)) {
      send_error(fd, 500, trace_id, "HTTP.INTERNAL", "user store capacity exceeded");
      return;
    }

    char payload[512];
    snprintf(payload, sizeof(payload), "{\"ok\":true,\"userId\":\"%s\"}", id);
    send_response(fd, 201, "application/json; charset=utf-8", trace_id, payload);
    return;
  }

  if (strcmp(method, "GET") == 0 && strncmp(path, "/users/", 7) == 0) {
    const char *id = path + 7;
    if (!is_uuid_v4(id)) {
      send_error(fd, 400, trace_id, "VALIDATION.UUID_INVALID", "id must be UUID v4");
      return;
    }

    UserEntry *entry = find_user(id);
    if (entry == NULL) {
      send_error(fd, 404, trace_id, "HTTP.NOT_FOUND", "user not found");
      return;
    }

    send_response(fd, 200, "application/json; charset=utf-8", trace_id, entry->body);
    return;
  }

  send_error(fd, 404, trace_id, "HTTP.NOT_FOUND", "route not found");
}

int main(void) {
  signal(SIGPIPE, SIG_IGN);

  const char *port_env = getenv("PORT");
  int port = 8080;
  if (port_env != NULL && port_env[0] != '\0') {
    port = atoi(port_env);
  }

  int server_fd = socket(AF_INET, SOCK_STREAM, 0);
  if (server_fd < 0) {
    perror("socket");
    return 1;
  }

  int opt = 1;
  if (setsockopt(server_fd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt)) != 0) {
    perror("setsockopt");
    close(server_fd);
    return 1;
  }

  struct sockaddr_in addr;
  memset(&addr, 0, sizeof(addr));
  addr.sin_family = AF_INET;
  addr.sin_addr.s_addr = htonl(INADDR_ANY);
  addr.sin_port = htons((uint16_t)port);

  if (bind(server_fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
    perror("bind");
    close(server_fd);
    return 1;
  }

  if (listen(server_fd, 128) != 0) {
    perror("listen");
    close(server_fd);
    return 1;
  }

  fprintf(stderr, "c benchmark service listening on :%d\n", port);

  while (true) {
    int client_fd = accept(server_fd, NULL, NULL);
    if (client_fd < 0) {
      if (errno == EINTR) {
        continue;
      }
      perror("accept");
      break;
    }

    handle_connection(client_fd);
    close(client_fd);
  }

  close(server_fd);
  return 0;
}
