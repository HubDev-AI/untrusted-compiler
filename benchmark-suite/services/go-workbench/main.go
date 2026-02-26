package main

import (
	"bytes"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"time"
)

type httpError struct {
	Code    string
	Kind    string
	Status  int
	Message string
}

type apiServer struct {
	pgDSN     string
	authToken string
}

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "18090"
	}

	pgDSN := os.Getenv("BENCH_WORKBENCH_PG_DSN")
	if pgDSN == "" {
		pgDSN = os.Getenv("SEC4_RT_LASM_DB_POSTGRES_DSN")
	}
	if pgDSN == "" {
		pgDSN = "postgresql://127.0.0.1:5432/postgres?sslmode=disable"
	}

	authToken := os.Getenv("BENCH_WORKBENCH_AUTH_TOKEN")
	if authToken == "" {
		authToken = "token123"
	}

	s := &apiServer{
		pgDSN:     pgDSN,
		authToken: authToken,
	}

	addr := ":" + port
	log.Printf("go-workbench service listening on %s", addr)
	if err := http.ListenAndServe(addr, s); err != nil {
		log.Fatal(err)
	}
}

func (s *apiServer) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	traceID := newTraceID()

	if r.Method == http.MethodGet && r.URL.Path == "/health" {
		writeText(w, http.StatusOK, traceID, "ok")
		return
	}
	if r.Method == http.MethodPost && r.URL.Path == "/wb/setup" {
		if err := s.requireAuth(r); err != nil {
			writeAPIError(w, traceID, *err)
			return
		}
		if err := s.setupSchema(); err != nil {
			writeAPIError(w, traceID, *err)
			return
		}
		writeJSON(w, http.StatusOK, traceID, successEnvelope(http.StatusOK, traceID, map[string]any{"setup": true}))
		return
	}
	if r.Method == http.MethodPost && r.URL.Path == "/wb/tasks" {
		if err := s.requireAuth(r); err != nil {
			writeAPIError(w, traceID, *err)
			return
		}
		data, err := s.insertTask(r)
		if err != nil {
			writeAPIError(w, traceID, *err)
			return
		}
		writeJSON(w, http.StatusCreated, traceID, successEnvelope(http.StatusCreated, traceID, data))
		return
	}
	if r.Method == http.MethodPost && r.URL.Path == "/wb/tasks/with-comment" {
		if err := s.requireAuth(r); err != nil {
			writeAPIError(w, traceID, *err)
			return
		}
		data, err := s.createTaskWithComment(r)
		if err != nil {
			writeAPIError(w, traceID, *err)
			return
		}
		writeJSON(w, http.StatusCreated, traceID, successEnvelope(http.StatusCreated, traceID, data))
		return
	}
	if r.Method == http.MethodGet && r.URL.Path == "/wb/tasks" {
		data, err := s.listTasks(r)
		if err != nil {
			writeAPIError(w, traceID, *err)
			return
		}
		writeJSON(w, http.StatusOK, traceID, successEnvelope(http.StatusOK, traceID, data))
		return
	}
	if strings.HasPrefix(r.URL.Path, "/wb/tasks/") && strings.HasSuffix(r.URL.Path, "/comments") {
		if r.Method != http.MethodPost {
			writeAPIError(w, traceID, httpError{
				Code:    "HTTP.NOT_FOUND",
				Kind:    "missing_dependency",
				Status:  http.StatusNotFound,
				Message: "route not found",
			})
			return
		}
		if err := s.requireAuth(r); err != nil {
			writeAPIError(w, traceID, *err)
			return
		}
		taskID := strings.TrimSuffix(strings.TrimPrefix(r.URL.Path, "/wb/tasks/"), "/comments")
		data, err := s.insertComment(taskID, r)
		if err != nil {
			writeAPIError(w, traceID, *err)
			return
		}
		writeJSON(w, http.StatusCreated, traceID, successEnvelope(http.StatusCreated, traceID, data))
		return
	}
	if strings.HasPrefix(r.URL.Path, "/wb/tasks/") && r.Method == http.MethodGet {
		taskID := strings.TrimPrefix(r.URL.Path, "/wb/tasks/")
		data, err := s.getTask(taskID)
		if err != nil {
			writeAPIError(w, traceID, *err)
			return
		}
		writeJSON(w, http.StatusOK, traceID, successEnvelope(http.StatusOK, traceID, data))
		return
	}

	writeAPIError(w, traceID, httpError{
		Code:    "HTTP.NOT_FOUND",
		Kind:    "missing_dependency",
		Status:  http.StatusNotFound,
		Message: "route not found",
	})
}

func (s *apiServer) requireAuth(r *http.Request) *httpError {
	if r.Header.Get("Authorization") != "Bearer "+s.authToken {
		return &httpError{
			Code:    "AUTH.REQUIRED",
			Kind:    "auth",
			Status:  http.StatusUnauthorized,
			Message: "authorization token is required",
		}
	}
	return nil
}

func (s *apiServer) setupSchema() *httpError {
	statements := []string{
		"create table if not exists wb_tasks (id text primary key, title text not null, description text not null default '', status text not null, priority integer not null, created_at_ms bigint not null);",
		"create table if not exists wb_comments (id text primary key, task_id text not null, body text not null, created_at_ms bigint not null);",
		"create table if not exists wb_labels (task_id text not null, name text not null, primary key(task_id, name));",
	}
	for _, sql := range statements {
		if _, err := runPsql(s.pgDSN, sql); err != nil {
			return &httpError{Code: "DB.QUERY_FAILED", Kind: "internal", Status: 500, Message: err.Error()}
		}
	}
	return nil
}

func (s *apiServer) insertTask(r *http.Request) (map[string]any, *httpError) {
	id, err := requireTextParam(r, "id")
	if err != nil {
		return nil, err
	}
	title, err := requireTextParam(r, "title")
	if err != nil {
		return nil, err
	}
	description := optionalTextParam(r, "description", "")
	status, err := requireTextParam(r, "status")
	if err != nil {
		return nil, err
	}
	if !isValidStatus(status) {
		return nil, &httpError{Code: "VALIDATION.INVALID", Kind: "validation", Status: 400, Message: "status must be one of: open, in_progress, done"}
	}
	priority, err2 := requireIntParam(r, "priority")
	if err2 != nil {
		return nil, err2
	}
	if priority < 1 || priority > 5 {
		return nil, &httpError{Code: "VALIDATION.INVALID", Kind: "validation", Status: 400, Message: "priority must be in range 1..5"}
	}
	createdAtMs, err3 := requireIntParam(r, "created_at_ms")
	if err3 != nil {
		return nil, err3
	}

	sql := fmt.Sprintf(
		"insert into wb_tasks (id, title, description, status, priority, created_at_ms) values (%s, %s, %s, %s, %d, %d);",
		sqlLiteral(id),
		sqlLiteral(title),
		sqlLiteral(description),
		sqlLiteral(status),
		priority,
		createdAtMs,
	)
	if _, err := runPsql(s.pgDSN, sql); err != nil {
		return nil, &httpError{Code: "DB.QUERY_FAILED", Kind: "internal", Status: 500, Message: err.Error()}
	}
	return map[string]any{"id": id}, nil
}

func (s *apiServer) createTaskWithComment(r *http.Request) (map[string]any, *httpError) {
	taskID, err := requireTextParam(r, "id")
	if err != nil {
		return nil, err
	}
	title, err := requireTextParam(r, "title")
	if err != nil {
		return nil, err
	}
	description := optionalTextParam(r, "description", "")
	status, err := requireTextParam(r, "status")
	if err != nil {
		return nil, err
	}
	if !isValidStatus(status) {
		return nil, &httpError{Code: "VALIDATION.INVALID", Kind: "validation", Status: 400, Message: "status must be one of: open, in_progress, done"}
	}
	priority, err2 := requireIntParam(r, "priority")
	if err2 != nil {
		return nil, err2
	}
	if priority < 1 || priority > 5 {
		return nil, &httpError{Code: "VALIDATION.INVALID", Kind: "validation", Status: 400, Message: "priority must be in range 1..5"}
	}
	createdAtMs, err3 := requireIntParam(r, "created_at_ms")
	if err3 != nil {
		return nil, err3
	}
	commentID, err4 := requireTextParam(r, "comment_id")
	if err4 != nil {
		return nil, err4
	}
	commentBody, err5 := requireTextParam(r, "comment_body")
	if err5 != nil {
		return nil, err5
	}
	commentCreatedAtMs, err6 := requireIntParam(r, "comment_created_at_ms")
	if err6 != nil {
		return nil, err6
	}

	stmts := []string{
		"BEGIN;",
		fmt.Sprintf(
			"insert into wb_tasks (id, title, description, status, priority, created_at_ms) values (%s, %s, %s, %s, %d, %d);",
			sqlLiteral(taskID), sqlLiteral(title), sqlLiteral(description), sqlLiteral(status), priority, createdAtMs,
		),
		fmt.Sprintf(
			"insert into wb_comments (id, task_id, body, created_at_ms) values (%s, %s, %s, %d);",
			sqlLiteral(commentID), sqlLiteral(taskID), sqlLiteral(commentBody), commentCreatedAtMs,
		),
		"COMMIT;",
	}
	for _, stmt := range stmts {
		if _, err := runPsql(s.pgDSN, stmt); err != nil {
			_, _ = runPsql(s.pgDSN, "ROLLBACK;")
			return nil, &httpError{Code: "DB.QUERY_FAILED", Kind: "internal", Status: 500, Message: err.Error()}
		}
	}

	return map[string]any{"taskId": taskID, "commentId": commentID}, nil
}

func (s *apiServer) insertComment(taskID string, r *http.Request) (map[string]any, *httpError) {
	commentID, err := requireTextParam(r, "comment_id")
	if err != nil {
		return nil, err
	}
	commentBody, err2 := requireTextParam(r, "comment_body")
	if err2 != nil {
		return nil, err2
	}
	commentCreatedAtMs, err3 := requireIntParam(r, "comment_created_at_ms")
	if err3 != nil {
		return nil, err3
	}

	stmt := fmt.Sprintf(
		"insert into wb_comments (id, task_id, body, created_at_ms) values (%s, %s, %s, %d);",
		sqlLiteral(commentID), sqlLiteral(taskID), sqlLiteral(commentBody), commentCreatedAtMs,
	)
	if _, err := runPsql(s.pgDSN, stmt); err != nil {
		return nil, &httpError{Code: "DB.QUERY_FAILED", Kind: "internal", Status: 500, Message: err.Error()}
	}
	return map[string]any{"id": commentID}, nil
}

func (s *apiServer) getTask(taskID string) (map[string]any, *httpError) {
	stmt := fmt.Sprintf(
		"select row_to_json(t)::text from (select id, title, description, status, priority, created_at_ms, (select count(*)::int from wb_comments c where c.task_id = wb_tasks.id) as comments_count from wb_tasks where id = %s limit 1) t;",
		sqlLiteral(taskID),
	)
	raw, err := runPsql(s.pgDSN, stmt)
	if err != nil {
		return nil, &httpError{Code: "DB.QUERY_FAILED", Kind: "internal", Status: 500, Message: err.Error()}
	}
	if raw == "" {
		return nil, &httpError{Code: "TASK.NOT_FOUND", Kind: "missing_dependency", Status: 404, Message: "task not found"}
	}
	var row map[string]any
	if err := json.Unmarshal([]byte(raw), &row); err != nil {
		return nil, &httpError{Code: "DB.QUERY_FAILED", Kind: "internal", Status: 500, Message: "invalid JSON row payload"}
	}
	return row, nil
}

func (s *apiServer) listTasks(r *http.Request) (map[string]any, *httpError) {
	status := optionalTextParam(r, "status", "")
	limit := optionalIntParam(r, "limit", 20)
	offset := optionalIntParam(r, "offset", 0)
	if limit < 1 {
		limit = 1
	}
	if limit > 100 {
		limit = 100
	}
	if offset < 0 {
		offset = 0
	}

	whereClause := "true"
	if status != "" {
		whereClause = "status = " + sqlLiteral(status)
	}

	itemsSQL := fmt.Sprintf(
		"select coalesce(json_agg(t), '[]'::json)::text from (select id, title, description, status, priority, created_at_ms from wb_tasks where %s order by created_at_ms desc, id desc limit %d offset %d) t;",
		whereClause,
		limit,
		offset,
	)
	itemsRaw, err := runPsql(s.pgDSN, itemsSQL)
	if err != nil {
		return nil, &httpError{Code: "DB.QUERY_FAILED", Kind: "internal", Status: 500, Message: err.Error()}
	}
	var items []any
	if itemsRaw == "" {
		items = []any{}
	} else if err := json.Unmarshal([]byte(itemsRaw), &items); err != nil {
		return nil, &httpError{Code: "DB.QUERY_FAILED", Kind: "internal", Status: 500, Message: "invalid list JSON payload"}
	}

	countRaw, err := runPsql(s.pgDSN, fmt.Sprintf("select count(*)::text from wb_tasks where %s;", whereClause))
	if err != nil {
		return nil, &httpError{Code: "DB.QUERY_FAILED", Kind: "internal", Status: 500, Message: err.Error()}
	}
	count, _ := strconv.Atoi(strings.TrimSpace(countRaw))

	return map[string]any{
		"items":  items,
		"count":  count,
		"limit":  limit,
		"offset": offset,
	}, nil
}

func runPsql(dsn string, sql string) (string, error) {
	cmd := exec.Command("psql", dsn, "-t", "-A", "-v", "ON_ERROR_STOP=1", "-c", sql)
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	if err := cmd.Run(); err != nil {
		message := strings.TrimSpace(stderr.String())
		if message == "" {
			message = err.Error()
		}
		return "", fmt.Errorf("%s", message)
	}
	return strings.TrimSpace(stdout.String()), nil
}

func sqlLiteral(value string) string {
	return "'" + strings.ReplaceAll(value, "'", "''") + "'"
}

func requireTextParam(r *http.Request, key string) (string, *httpError) {
	value := r.URL.Query().Get(key)
	if strings.TrimSpace(value) == "" {
		return "", &httpError{
			Code:    "VALIDATION.INVALID",
			Kind:    "validation",
			Status:  400,
			Message: "missing required query param: " + key,
		}
	}
	return value, nil
}

func optionalTextParam(r *http.Request, key, fallback string) string {
	value := r.URL.Query().Get(key)
	if strings.TrimSpace(value) == "" {
		return fallback
	}
	return value
}

func requireIntParam(r *http.Request, key string) (int, *httpError) {
	raw := r.URL.Query().Get(key)
	if strings.TrimSpace(raw) == "" {
		return 0, &httpError{
			Code:    "VALIDATION.INVALID",
			Kind:    "validation",
			Status:  400,
			Message: "missing required query param: " + key,
		}
	}
	value, err := strconv.Atoi(raw)
	if err != nil {
		return 0, &httpError{
			Code:    "VALIDATION.INVALID",
			Kind:    "validation",
			Status:  400,
			Message: "invalid integer query param: " + key,
		}
	}
	return value, nil
}

func optionalIntParam(r *http.Request, key string, fallback int) int {
	raw := r.URL.Query().Get(key)
	if strings.TrimSpace(raw) == "" {
		return fallback
	}
	value, err := strconv.Atoi(raw)
	if err != nil {
		return fallback
	}
	return value
}

func isValidStatus(status string) bool {
	return status == "open" || status == "in_progress" || status == "done"
}

func successEnvelope(status int, traceID string, data any) map[string]any {
	return map[string]any{
		"ok":      true,
		"status":  status,
		"traceId": traceID,
		"timeMs":  time.Now().UnixMilli(),
		"data":    data,
	}
}

func errorEnvelope(err httpError, traceID string) map[string]any {
	return map[string]any{
		"ok":      false,
		"status":  err.Status,
		"traceId": traceID,
		"timeMs":  time.Now().UnixMilli(),
		"error": map[string]any{
			"code":    err.Code,
			"kind":    err.Kind,
			"message": err.Message,
		},
	}
}

func writeJSON(w http.ResponseWriter, status int, traceID string, payload any) {
	w.Header().Set("content-type", "application/json; charset=utf-8")
	w.Header().Set("x-trace-id", traceID)
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func writeText(w http.ResponseWriter, status int, traceID string, body string) {
	w.Header().Set("content-type", "text/plain; charset=utf-8")
	w.Header().Set("x-trace-id", traceID)
	w.WriteHeader(status)
	_, _ = w.Write([]byte(body))
}

func writeAPIError(w http.ResponseWriter, traceID string, err httpError) {
	writeJSON(w, err.Status, traceID, errorEnvelope(err, traceID))
}

func newTraceID() string {
	buf := make([]byte, 8)
	if _, err := rand.Read(buf); err != nil {
		return fmt.Sprintf("trace-%d", time.Now().UnixNano())
	}
	return "trace-" + hex.EncodeToString(buf)
}
