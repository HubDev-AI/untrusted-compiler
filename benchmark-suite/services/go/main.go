package main

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"regexp"
	"strconv"
	"sync"
	"time"
)

var (
	uuidV4Re = regexp.MustCompile(`^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$`)
	emailRe  = regexp.MustCompile(`^[^\s@]+@[^\s@]+\.[^\s@]+$`)
	zipRe    = regexp.MustCompile(`^\d{4,10}$`)
)

type userPayload struct {
	ID      string   `json:"id"`
	Email   string   `json:"email"`
	Age     int      `json:"age"`
	Name    string   `json:"name"`
	Tags    []string `json:"tags"`
	Address struct {
		Zip string `json:"zip"`
	} `json:"address"`
	Meta struct {
		Flags struct {
			A bool `json:"a"`
			B bool `json:"b"`
			C bool `json:"c"`
		} `json:"flags"`
	} `json:"meta"`
}

type server struct {
	mu    sync.RWMutex
	users map[string]userPayload
}

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	s := &server{users: map[string]userPayload{}}
	mux := http.NewServeMux()
	mux.HandleFunc("/ping", s.handlePing)
	mux.HandleFunc("/decode", s.handleDecode)
	mux.HandleFunc("/users", s.handleUsersPost)
	mux.HandleFunc("/users/", s.handleUsersGet)

	addr := ":" + port
	log.Printf("go benchmark service listening on %s", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatal(err)
	}
}

func (s *server) handlePing(w http.ResponseWriter, r *http.Request) {
	traceID := newTraceID()
	if r.Method != http.MethodGet {
		writeError(w, traceID, http.StatusNotFound, "HTTP.NOT_FOUND", "route not found")
		return
	}
	w.Header().Set("content-type", "text/plain; charset=utf-8")
	w.Header().Set("x-trace-id", traceID)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte("ok"))
}

func (s *server) handleDecode(w http.ResponseWriter, r *http.Request) {
	traceID := newTraceID()
	if r.Method != http.MethodPost {
		writeError(w, traceID, http.StatusNotFound, "HTTP.NOT_FOUND", "route not found")
		return
	}

	payload, ok := decodeAndValidateUser(w, r, traceID)
	if !ok {
		return
	}

	writeJSON(w, traceID, http.StatusOK, map[string]any{"ok": true, "id": payload.ID})
}

func (s *server) handleUsersPost(w http.ResponseWriter, r *http.Request) {
	traceID := newTraceID()
	if r.Method != http.MethodPost {
		writeError(w, traceID, http.StatusNotFound, "HTTP.NOT_FOUND", "route not found")
		return
	}

	payload, ok := decodeAndValidateUser(w, r, traceID)
	if !ok {
		return
	}

	s.mu.Lock()
	s.users[payload.ID] = payload
	s.mu.Unlock()

	writeJSON(w, traceID, http.StatusCreated, map[string]any{"ok": true, "userId": payload.ID})
}

func (s *server) handleUsersGet(w http.ResponseWriter, r *http.Request) {
	traceID := newTraceID()
	if r.Method != http.MethodGet {
		writeError(w, traceID, http.StatusNotFound, "HTTP.NOT_FOUND", "route not found")
		return
	}

	id := r.URL.Path[len("/users/"):]
	if !uuidV4Re.MatchString(id) {
		writeError(w, traceID, http.StatusBadRequest, "VALIDATION.UUID_INVALID", "id must be UUID v4")
		return
	}

	s.mu.RLock()
	user, ok := s.users[id]
	s.mu.RUnlock()
	if !ok {
		writeError(w, traceID, http.StatusNotFound, "HTTP.NOT_FOUND", "user not found")
		return
	}

	writeJSON(w, traceID, http.StatusOK, user)
}

func decodeAndValidateUser(w http.ResponseWriter, r *http.Request, traceID string) (userPayload, bool) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		writeError(w, traceID, http.StatusBadRequest, "HTTP.BAD_REQUEST", "could not read body")
		return userPayload{}, false
	}

	var payload userPayload
	if err := json.Unmarshal(body, &payload); err != nil {
		writeError(w, traceID, http.StatusBadRequest, "JSON.INVALID_SYNTAX", "invalid JSON payload")
		return userPayload{}, false
	}

	if msg := validateUserPayload(payload); msg != "" {
		writeError(w, traceID, http.StatusBadRequest, "VALIDATION.INVALID", msg)
		return userPayload{}, false
	}

	return payload, true
}

func validateUserPayload(p userPayload) string {
	if !uuidV4Re.MatchString(p.ID) {
		return "id must be a UUID v4 string"
	}
	if !emailRe.MatchString(p.Email) {
		return "email must be a valid email string"
	}
	if p.Age < 0 || p.Age > 150 {
		return "age must be an integer between 0 and 150"
	}
	if len(p.Tags) > 16 {
		return "tags must be an array of length <= 16"
	}
	for _, tag := range p.Tags {
		if len(tag) < 1 || len(tag) > 32 {
			return "tags must contain strings of length 1..32"
		}
	}
	if !zipRe.MatchString(p.Address.Zip) {
		return "address.zip must be a digit string of length 4..10"
	}
	return ""
}

func writeJSON(w http.ResponseWriter, traceID string, status int, payload any) {
	w.Header().Set("content-type", "application/json; charset=utf-8")
	w.Header().Set("x-trace-id", traceID)
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func writeError(w http.ResponseWriter, traceID string, status int, code string, message string) {
	kind := "validation"
	if status == http.StatusNotFound {
		kind = "not_found"
	} else if status >= 500 {
		kind = "internal"
	}

	writeJSON(w, traceID, status, map[string]any{
		"error": map[string]any{
			"code":    code,
			"kind":    kind,
			"message": message,
			"status":  status,
			"traceId": traceID,
			"timeMs":  time.Now().UnixMilli(),
		},
	})
}

func newTraceID() string {
	buf := make([]byte, 8)
	if _, err := rand.Read(buf); err != nil {
		return "trace-" + strconv.FormatInt(time.Now().UnixNano(), 10)
	}
	return fmt.Sprintf("trace-%s", hex.EncodeToString(buf))
}
