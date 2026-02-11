# 60 v0 Policy Keys Spec (Names, Types, Defaults)

This chapter defines the minimal policy-as-code contract enforced by compiler and runtime.

## 1) Policy identity

```toml
[policy]
version = "0.1"
name = "default-secure"
mode = "enforce"   # "enforce" | "warn"
```

Compiler/runtime must stamp build metadata with:
- `policyHash`
- `compilerHash`
- `runtimeHash`
- `policy.name`
- `policy.version`

## 2) Effects policy

```toml
[effects]
forbid = ["shell", "unsafe", "secrets.reveal"]
require_explicit_on_function_fields = true
```

Allowlist annotation contract should require `reason` and `ticket`, with optional expiry checks.

## 3) Secrets policy

```toml
[secrets]
forbid_reveal = true
allow_redact = true
forbid_formatting = true
```

## 4) JSON/schema policy

```toml
[json]
max_bytes = 1048576
max_depth = 32
require_schema_for_encode = true
forbid_secret_fields_in_schema = true
```

Optional bounds:
- `max_string_bytes`
- `max_container_entries`

## 5) HTTP and budget policy

```toml
[http]
max_body_bytes = 1048576
max_concurrency = 256
default_timeout_ms = 5000

[budget]
max_body_bytes = 1048576
max_json_bytes = 1048576
max_json_depth = 32
max_deadline_ms = 60000
```

Runtime should clamp request-level budgets to policy maxima.

## 6) Logging/privacy policy

```toml
[logging]
structured_only = true
include_remote_ip = false
include_user_agent = false
include_auth_user_id = true
max_attr_depth = 8
max_attr_bytes = 8192
sampling = 1.0
```

## 7) SQL policy

```toml
[sql]
forbid_raw = true
require_limit_on_select = "warn"   # "off" | "warn" | "enforce"
max_returned_rows = 10000
default_timeout_ms = 2000
```

## 8) Network/SSRF policy

```toml
[net.public]
allowed_schemes = ["https"]
allow_redirects = false
max_redirects = 0
allowed_domains = []
blocked_domains = []
allowed_ports = [443]

[net.internal]
enabled = false
allowed_cidrs = []
allowed_domains = []

[net.ssrf]
block_private_ranges = true
block_loopback = true
block_link_local = true
block_metadata_ips = true
resolve_dns = true
revalidate_redirects = true
```

## 9) Filesystem policy

```toml
[fs]
enabled = false
allowed_base_paths = []
forbid_absolute_paths = true
forbid_symlinks = "enforce"
```

## 10) Capture/replay policy

```toml
[capture]
mode = "errors"
sample_rate = 0.01
max_body_bytes = 262144
max_capture_bytes = 1048576
redact_headers = ["authorization", "cookie", "set-cookie", "x-api-key", "x-auth-token"]
redact_json_paths = ["$.password", "$.token", "$.secret", "$.apiKey"]
hash_only_non_json_bodies = true

[replay]
effects = "deny"   # "deny" | "mock" | "allow"
allow_policy_mismatch = false
```

## 11) CORS policy

```toml
[cors]
enabled = true
allowed_origins = ["https://app.example.com"]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allowed_headers = ["content-type", "authorization"]
exposed_headers = []
allow_credentials = true
max_age_seconds = 600
forbid_any_origin = true
forbid_reflect_origin = true
require_vary_origin = true
```

Rules:
- if credentials are allowed, wildcard origin must be forbidden.
- if wildcard origin is configured while `forbid_any_origin=true`, fail compile/policy check.

## 12) Schema validation rules for policy file

On load:
- unknown keys are errors
- numeric bounds validated
- scheme lists validated
- redirect/max_redirect coherence checked
- internal-network disabled state validated against internal allowlist fields
