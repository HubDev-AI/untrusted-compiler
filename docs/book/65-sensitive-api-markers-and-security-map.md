# 65 Sensitive API Markers and security_map Metadata (v0)

This chapter defines compiler-level semantic tags used by `sec.audit` and related tooling.

## 1) Tag model

### Tag record

```ailang
type Tag = {
  id: String
  kind: "source" | "sink" | "gate" | "effect" | "capability" | "policy" | "middleware"
  attrs?: Map<String, String | Bool | Int64>
}
```

### Attachment points
Tags can attach to:
- stdlib and intrinsic symbols
- call sites (including special forms)
- sensitive types
- modules

## 2) Metadata artifact
Compiler emits `build/security_map.json` with:
- tagged symbols
- tagged call sites
- middleware usage and effective attrs
- allowlist annotations and bypassed tag ids

## 3) Required tag namespaces

### Sources
- `source.http.body`
- `source.http.query`
- `source.http.header`
- `source.http.path`
- `source.env`

### Gates
- `gate.schema.json_decode`
- `gate.validate.*`
- `gate.sanitize.html`
- `gate.url.public`
- `gate.path.under`
- `gate.header.value`

### Sinks
- SQL: `sink.sql.exec`, `sink.sql.query`
- HTML: `sink.http.html`
- Headers/cookies: `sink.http.header_set`, `sink.http.cookie_set`
- Net: `sink.net.public_request`, `sink.net.internal_request`
- FS: `sink.fs.read`, `sink.fs.write`
- Logging: `sink.log.emit`
- JSON encode: `sink.json.encode`, `sink.json.encode_http_response`

### Effects
- `effect.net`
- `effect.db.read`
- `effect.db.write`
- `effect.fs.read`
- `effect.fs.write`
- `effect.secrets.read`
- `effect.secrets.reveal`
- `effect.log`

### Capabilities
- `capability.db`
- `capability.net`
- `capability.internal_net`
- `capability.fs`
- `capability.secrets`

### Middleware
- `middleware.cors`
- `middleware.security_headers`
- `middleware.csrf`
- `middleware.auth`
- `middleware.request_capture`

## 4) Tagging rules

### 4.1 Stdlib registry
All security-relevant stdlib symbols must be pre-tagged in compiler symbol registry.

### 4.2 Special forms
- SQL template call sites receive SQL-specific tags and parameter metadata.
- HTML template call sites receive safe composition tags.
- `req.json` receives both source and gate tags.
- `res.json` receives response encode sink tags.

## 5) Allowlist annotation integration

Annotation shape:

```ailang
@allow(policy="net.internal.enabled",
       bypass=["sink.net.internal_request"],
       reason="Calls internal inventory service",
       ticket="SEC-123",
       expires="2026-06-01")
```

Compiler requirements:
- `policy`, `bypass`, `reason`, `ticket`, `expires` required
- expired allow entries are compile errors
- each allow entry must be emitted with location and bypass tags

## 6) security_map minimal schema

```json
{
  "version": "0.1",
  "policyHash": "pol_...",
  "symbols": [ ],
  "calls": [ ],
  "middleware": [ ],
  "allows": [ ]
}
```

## 7) v0 required coverage
v0 is complete enough when tags cover:
- all HTTP input sources
- schema decode gate
- SQL/HTML/header/log sinks
- public/internal net sinks
- filesystem sinks
- secrets reveal effect
- cors/security_headers/csrf/auth middleware
