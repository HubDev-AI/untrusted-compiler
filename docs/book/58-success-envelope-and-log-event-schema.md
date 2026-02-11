# 58 Standard Success Envelope and Log Event Schema

This chapter adds two aligned API contracts:
1) Optional standard success response envelope
2) Recommended structured log event schema

## 1) Standard success envelope (optional)

### Goals
- consistent client parsing
- always include `traceId`
- avoid leaking internal metadata
- allow raw/stream/file responses to remain unwrapped

### Shape

```ailang
type StdSuccess<T> = {
  ok: Bool
  status: Int
  traceId: String
  timeMs: Int64
  data: T
  meta?: Map<String, String | Int64 | Bool>
}
```

Example JSON:

```json
{
  "ok": true,
  "status": 200,
  "traceId": "01HZY...ABC",
  "timeMs": 1760000000000,
  "data": { "id": "..." }
}
```

### Rules
- Use envelope for standard JSON endpoints.
- Streaming/file endpoints can return raw responses.
- In strict mode, success encoding still requires schema.

### Minimal API

```ailang
module res {
  fn ok<T>(status: Int, schema: Schema<T>, value: T) -> Result<Response, HttpError>
  fn okMeta<T>(status: Int, schema: Schema<T>, value: T, meta: Map<String, ErrorDetailValue>) -> Result<Response, HttpError>
}
```

Optional policy knob:
- `http.success_envelope = "none" | "standard"`

## 2) Log event schema (recommended)

### Goals
- structured/queryable logs
- no secret/PII leaks by default
- trace correlation
- budget/limit visibility
- direct incident-response alignment with `StdError`

### Types

```ailang
type LogLevel = "debug" | "info" | "warn" | "error"

type LogEvent = {
  timeMs: Int64
  level: LogLevel
  traceId: String

  span?: {
    spanId: String
    parentSpanId?: String
    name: String
  }

  event: String
  message?: String
  attrs?: Map<String, LogAttr>
  error?: StdError

  http?: {
    method: String
    path: String
    route?: String
    status?: Int
    remoteIp?: String
    userAgent?: String
    latencyMs?: Int64
  }

  auth?: {
    userId?: String
    sessionId?: String
    role?: String
  }

  budget?: {
    maxBodyBytes: Int64
    maxJsonBytes: Int64
    maxJsonDepth: Int
    deadlineMs: Int64
  }

  dependency?: {
    name: String
    op?: String
    latencyMs?: Int64
    retryable?: Bool
  }
}

type LogAttr =
  | String
  | Int64
  | Bool
  | List<LogAttr>
  | Map<String, LogAttr>
  | { redacted: String }
```

Hard rule:
- `Secret<_>` and `Untrusted<_>` cannot convert directly into `LogAttr`.

### Logging API

```ailang
type Log = {
  emit: fn(LogEvent) effects { log } -> Unit
  info: fn(event: String, attrs?: Map<String, LogAttr>) effects { log } -> Unit
  warn: fn(event: String, attrs?: Map<String, LogAttr>) effects { log } -> Unit
  error: fn(event: String, err: StdError, attrs?: Map<String, LogAttr>) effects { log } -> Unit
}

module log {
  fn attrRedacted(label: String) -> LogAttr
  fn event(name: String) -> LogEvent
  fn withAttr(e: LogEvent, k: String, v: LogAttr) -> LogEvent
  fn withHttp(e: LogEvent, method: String, path: String, status?: Int, latencyMs?: Int64) -> LogEvent
  fn withError(e: LogEvent, err: StdError) -> LogEvent
}
```

### Required behavior
- `traceId` injected automatically.
- For route logs, include normalized route when available.
- Structured JSON log output; escape control chars in messages.
- Emit limit/budget breach events (`limit.exceeded`) with resource-limit context.

### Starter canonical event names
- `http.request`
- `http.response`
- `auth.failed`
- `user.created`
- `db.query`
- `net.request`
- `limit.exceeded`
- `policy.blocked`
- `panic`
- `internal.error`
