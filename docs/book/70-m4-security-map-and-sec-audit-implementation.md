# 70 M4 Implementation: security_map and sec.audit (Current Slice)

This chapter documents the M4 implementation slice that introduced compiler-emitted security metadata and a first deterministic security audit command.

## Scope delivered
- Added security metadata generation:
  - compiler builds `SecurityMap` from AST call sites and policy context
  - output file: `build/security_map.json`
- Added initial sensitive marker/tag coverage:
  - source/gate/sink/effect/capability/middleware tags for v0 baseline intrinsics and middleware call names
- Added security audit engine:
  - deterministic finding rules (CORS, headers, CSRF/auth coupling, SSRF/internal-net, filesystem, capture/replay, secrets reveal)
  - risk scoring and severity ranking
  - text + JSON report output
- Added CLI command:
  - `ailang sec audit --path <project> [--format text|json] [--fail-on 'risk>=HIGH']`
- Added `@allow(...)` annotation pipeline:
  - parser-compatible source preprocessing strips `@allow(...)` before AST parsing
  - annotation parser validates required fields (`policy`, `bypass`, `reason`, `ticket`, `expires`)
  - expired annotations are rejected during analysis
  - parsed allowlist entries are emitted in `security_map.allows` and surfaced in `sec.audit` exceptions
- Added deterministic allowlist hygiene findings in `sec.audit`:
  - high-risk bypass findings (`SECRETS_REVEAL_ALLOWLISTED`, `INTERNAL_NET_CALL_ALLOWLISTED`)
  - expiry hygiene findings (`ALLOW_EXPIRED`, `ALLOW_EXPIRING_SOON`)
  - exception-volume hygiene finding (`ALLOW_COUNT_HIGH`)
- Added dotted stdlib call support for security/effect metadata paths:
  - parser now supports member-call expressions (`module.fn(...)`)
  - semantic intrinsic/effect/capability checks accept dotted aliases (e.g. `db.exec`, `req.json`, `secrets.reveal`)
  - `security_map` call and middleware tag extraction supports dotted call names
  - intrinsic symbol registry now includes dotted stdlib symbols alongside underscore aliases
- Added semantic sink flow enforcement for secrets/untrusted data:
  - generic wrappers `Secret<T>` and `Untrusted<T>` are recognized in the type catalog
  - log sinks reject `Secret<_>` and `Untrusted<_>` values with origin-aware diagnostics
  - JSON response sinks reject `Secret<_>` and `Untrusted<_>` values with origin-aware diagnostics
  - additional typed sinks now enforce `Secret`/`Untrusted` boundaries:
    - SQL sinks (`db.exec`, `db.queryOne`)
    - URL/network sinks (`httpClient.get`, `httpClient.getInternal`)
    - filesystem sinks (`fs.read`, `fs.write`)
    - header/cookie sinks (`res.setHeader`, `res.addCookie`)
  - error codes actively exercised in semantic fixtures: `E1002`, `E1003`, `E1004`, `E1005`
- Added first-class trust-gate semantics for request boundary APIs:
  - request sources now return explicit untrusted wrappers:
    - `req.body` -> `Untrusted<Bytes>`
    - `req.query`, `req.pathParam`, `req.header` -> `Untrusted<String>`
  - `req.json(schema)` is enforced as an explicit schema gate:
    - missing schema argument is rejected with `E4001`
    - diagnostic explains canonical gate usage and trust-boundary intent
  - semantic fixtures added for:
    - missing schema gate argument rejection
    - untrusted `req.query` flow into SQL sink rejection
- Added typed validator/sanitizer trust-gate contracts:
  - gate outputs are now typed for core safe wrappers:
    - `validate.headerValue` -> `HeaderValue`
    - `sanitize.html` -> `HtmlSafe`
    - `path.under` / `validate.pathUnder` -> `PathSafe`
    - `url.public` -> `PublicUrl`
    - `url.internal` -> `InternalUrl`
  - gates enforce canonical argument contracts:
    - string validation/sanitization gates require first argument `Untrusted<String>`
    - path gate requires `(PathSafe, Untrusted<String>)`
  - semantic fixtures cover valid gate usage and contract violations
- Expanded `security_map` marker coverage for gate/source symbols:
  - callsite tags now include:
    - `gate.header.value`
    - `gate.sanitize.html`
    - `gate.path.under`
    - `gate.url.public`
    - `gate.url.internal`
    - source tags for `req.query`/`req.pathParam`/`req.header`/`req.body`
  - intrinsic symbol registry now includes these gate and source entries for both underscore and dotted stdlib aliases
  - `security_map` tests now assert gate/source call tagging and registry presence
- Added logging/SQL policy ingestion and deterministic audit findings:
  - `Policy` now carries parsed `[logging]` and `[sql]` posture fields
  - parser validation now enforces valid `sql.require_limit_on_select` values (`off|warn|enforce`)
  - `sec.audit` now emits policy-aware findings:
    - `LOG_STRUCTURED_ONLY_DISABLED`
    - `LOG_REMOTE_IP_ENABLED`
    - `LOG_USER_AGENT_ENABLED`
    - `SQL_RAW_ALLOWED_BY_POLICY`
    - `SQL_LIMIT_RULE_DISABLED`
  - policy and audit tests were extended for this coverage
- Added callsite-driven SQL limit hygiene detection:
  - `security_map` now tags SQL sink callsites with `sql.select_without_limit` when a query literal/template call contains `SELECT` and no `LIMIT`
  - detection currently covers direct string SQL and `sql.q("...", ...)` call forms at `db.exec` / `db.queryOne` sinks
  - `sec.audit` now emits `SQL_SELECT_WITHOUT_LIMIT` with deterministic severity mapping:
    - `MEDIUM` when `sql.require_limit_on_select = "warn"`
    - `HIGH` when `sql.require_limit_on_select = "enforce"`
  - tests cover callsite tagging and severity mapping behavior
  - keyword scanning now ignores quoted string segments and SQL comments (`-- ...`, `/* ... */`) to reduce false positives/negatives from naive text matching
- Expanded validator gate coverage beyond single-function mappings:
  - semantic trust-gate enforcement now covers core `validate.*`/`sanitize.*` string gates with required `Untrusted<String>` input contracts
  - intrinsic return typing now includes:
    - `validate.email` -> `Email`
    - `validate.uuid` -> `Uuid`
    - `validate.int64` -> `Int64`
    - `validate.nonEmpty` -> `String`
  - new semantic fixtures cover:
    - invalid `validate.email(String)` rejection
    - valid `validate.email(req.query(...))` flow
  - `security_map` marker coverage now includes validator gate tags:
    - `gate.validate.email`
    - `gate.validate.uuid`
    - `gate.validate.int64`
    - `gate.validate.non_empty`
- Added strict JSON encode schema policy coupling:
  - policy model now carries `json.require_schema_for_encode` (default `true`)
  - semantic trust-gate checks now enforce minimum schema-argument contract for `res.json` in strict mode
  - encoding without schema argument emits `E4004` with explicit strict-mode guidance
  - semantic fixtures now cover:
    - strict-mode rejection for single-argument `res.json(...)` calls
    - valid `res.json(schema, value)` path
  - policy tests now cover toggling `json.require_schema_for_encode`
- Strengthened strict JSON signature enforcement:
  - strict mode now validates full `res.json` signatures, not only minimum arity:
    - `res.json(schema, value)`
    - `res.json(status, schema, value)` where `status` is numeric
  - schema argument is now validated as non-numeric/non-boolean and not `Secret<_>`/`Untrusted<_>`
  - invalid arity, non-numeric status, and invalid schema argument each produce dedicated `E4004` diagnostics
  - JSON sink flow checks now evaluate only the value argument (last payload arg), avoiding schema/status false positives
  - semantic fixtures now cover invalid status type, invalid schema type, invalid arity, and valid status+schema+value form
- Added typed schema-value pairing checks for JSON responses:
  - introduced generic `Schema<T>` type recognition in semantic type catalog
  - when `res.json` receives a typed `Schema<T>` argument, compiler now checks payload compatibility with `T`
  - mismatches produce `E4004` (`json response value does not match schema type`) with expected/actual type notes
  - semantic fixtures now cover:
    - valid typed schema encoding (`Schema<Int>` + `Int` payload)
    - invalid typed schema mismatch (`Schema<Int>` + `String` payload)
- Added callsite argument-role metadata for audit explainability:
  - `security_map.calls[]` now optionally includes `arg_roles`
  - role labels are emitted for core sensitive APIs (for example capability/query/url/schema/value/path)
  - JSON roles adapt to signature form (`schema,value` vs `status,schema,value`)
  - tests now assert role metadata for representative callsites (`res.json`, `db.exec`)
- Added source-origin edge metadata for call arguments:
  - `security_map.calls[]` now optionally includes `origin_edges`
  - each edge captures `arg_index`, canonical origin label (for example `call:req.query`), source/gate tag context, and deterministic provenance `trace` chain
  - origin tracking currently follows local `let` bindings and propagates through:
    - member/unary wrappers
    - binary wrappers with single-origin or same-origin operands
    - `if`/`match` expressions when branch origins are consistent
    - block-tail expressions with local shadow bindings
    - interprocedural forwarding via iterative function origin summaries (`from-param` and source/gate-tagged return summaries)
  - tests now assert sink-argument origin tracing for:
    - direct `db.exec(DbCap(), raw)` flow from `req.query`
    - composite expression flows (`binary`, `if`, `match`, `block`)
    - forwarding function flows (`queryParam -> passThrough -> db.exec`)
    - deeper forwarding chains (`queryParam -> passThrough -> passthroughTwice -> wrap -> db.exec`)
- Added richer deterministic callsite evidence in `sec.audit` findings:
  - finding evidence now includes bounded `sampleCalls` arrays for representative callsites
  - each sample includes callee, location, argument roles, and available origin-edge metadata
  - enabled finding coverage currently includes:
    - `SQL_SELECT_WITHOUT_LIMIT`
    - `SECRETS_REVEAL_USED`
    - `LOG_STRUCTURED_ONLY_DISABLED`
    - `LOG_REMOTE_IP_ENABLED`
    - `REFERRER_POLICY_WEAK`
    - `SQL_RAW_ALLOWED_BY_POLICY`
    - `SQL_LIMIT_RULE_DISABLED`
    - `INTERNAL_NET_ENABLED_NO_ALLOWLIST`
    - `FS_ENABLED_NO_BASE_ALLOWLIST`
    - `SYMLINK_POLICY_WEAK`
    - `CSRF_PROTECTED_METHODS_INCOMPLETE`
    - `PUBLIC_REDIRECTS_ENABLED_WITHOUT_REVALIDATION`
    - `PUBLIC_EGRESS_NO_DOMAIN_POLICY`
    - `CAPTURE_REDACTION_INCOMPLETE`
    - `CAPTURE_ALL_IN_PROD`
    - `REPLAY_EFFECTS_ALLOW`
    - CORS/security-headers/auth/CSRF posture findings via middleware-tagged sample callsites
    - allowlisted bypass findings (`SECRETS_REVEAL_ALLOWLISTED`, `INTERNAL_NET_CALL_ALLOWLISTED`) via bypass-tag sampling
    - non-call exception posture findings (`ALLOW_COUNT_HIGH`) via bounded `sampleExceptions` snapshots
    - non-call expiry-window posture rollups (`ALLOW_EXPIRY_WINDOW_ROLLUP`) with sampled exceptions
  - call sampling now supports both single-tag and multi-tag families for deterministic SQL sink aggregation
  - audit tests now assert presence and shape of `sampleCalls` evidence
  - text-format `sec.audit` output now shows bounded sample-call previews and trace snippets for fast triage
- Added context-first stdlib capability signature support:
  - semantic capability checks now resolve capability argument positions for both compact and context-first forms:
    - compact examples: `db.exec(cap, query)`, `httpClient.get(cap, url)`
    - context-first examples: `db.exec(ctx, cap, query)`, `httpClient.get(ctx, cap, url)`
  - sink-flow restriction offsets now adapt to call shape, so context/capability arguments are not misclassified as user payloads
  - `security_map` SQL query extraction now supports both `db.exec(cap, query)` and `db.exec(ctx, cap, query)` patterns
  - `security_map` argument-role metadata now emits context-aware roles (`context`, `capability`, `query`, etc.) for extended call forms
  - semantic and security-map fixtures now cover valid and invalid context-first capability paths
- Added typed stdlib symbol resolution for local alias/value-call paths:
  - semantic analyzer now resolves callable aliases (for example `let exec = db.exec; exec(...)`) to canonical stdlib symbols
  - capability/effect/sink checks run against resolved symbols so alias calls preserve security enforcement
  - semantic member-expression handling now avoids false `unknown identifier` diagnostics for intrinsic symbol references in alias bindings
  - `security_map` now resolves alias-invoked callsites to canonical callees and preserves tags/arg-roles/origin-edges
  - semantic and security-map tests now cover alias-invoked intrinsic calls
  - member alias paths are now canonicalized as well (for example `let repo = db; repo.exec(...)` -> `db.exec(...)`)
  - interprocedural callable-forwarding summaries now canonicalize helper-function aliases (for example `let exec = getExec(); exec(...)` when `getExec` forwards `db.exec`)
- sink-flow diagnostics now use callable-aware origin notes for call expressions, so security errors report canonical forwarded origins (for example `req.query(...)` through helper calls)

### Slice Explanation: Strict JSON Encode Signature Checks

#### 1) What it is
This slice adds strict signature validation for `res.json` in semantic analysis when `json.require_schema_for_encode = true`.

#### 2) Why it exists
Arity-only validation still allowed malformed response calls (wrong status type, schema position misuse). This change makes encode paths explicit and auditable at compile time.

#### 3) How it works internally
- `enforce_trust_gate_requirements` now delegates JSON encoding checks to `enforce_json_encode_signature`.
- `enforce_json_encode_signature` enforces accepted signatures and validates:
  - argument count,
  - numeric status for 3-arg form,
  - schema argument safety/type constraints.
- JSON sink flow checks now inspect only the value argument via `json_sink_value_arg_index`.

#### 4) Inputs/outputs and constraints
- Input: semantic call expression for `res.json(...)`.
- Output: either accepted call shape or `E4004` diagnostics.
- Constraints:
  - strict mode controlled by `json.require_schema_for_encode`,
  - accepted signatures are limited to 2-arg and 3-arg forms.

#### 5) Failure modes and diagnostics
- Missing schema argument: `E4004 json response encoding requires explicit schema argument`.
- Invalid argument count: `E4004 json response encoding has invalid argument count`.
- Non-numeric status: `E4004 json response status must be numeric`.
- Invalid schema argument type/taint: `E4004 json response schema argument is invalid`.

#### 6) Example usage
- Valid:
  - `res.json("UserSchema", value)`
  - `res.json(201, "UserSchema", value)`
- Invalid:
  - `res.json(value)` (strict mode)
  - `res.json("200", "UserSchema", value)`
  - `res.json(200, 123, value)`

#### 7) Tradeoffs and next steps
- Current schema argument validation is structural/heuristic, not true schema-type pairing.
- Typed schema pairing now works for `Schema<T>` arguments, but untyped schema descriptors (for example string placeholders) still rely on structural checks.
- Next step is to move from descriptor-style schema arguments to richer typed schema symbols across stdlib APIs.

### Slice Explanation: `security_map` Source-Origin Edge Metadata

#### 1) What it is
This slice adds argument-level origin traces to `security_map` call records via `origin_edges`.

#### 2) Why it exists
Call-level tags alone do not show which argument carried untrusted or gate-derived data into a sink. Origin edges provide deterministic evidence for audit findings and future diagnostics.

#### 3) How it works internally
- `collect_block` now tracks local binding origins in a scoped map.
- `collect_expr` emits `origin_edges` for call arguments by inspecting inferred origins.
- `infer_expr_origin` resolves origins from:
  - tracked identifiers,
  - source/gate-tagged calls (for example `req.query`, `validate.*`, `sanitize.*`),
  - wrapper/control-flow forms (`member`, `unary`, `binary`, `if`, `match`, block tails) when origin can be resolved deterministically.

#### 4) Inputs/outputs and constraints
- Input: function body expressions and local `let` bindings.
- Output: optional `origin_edges` array on each `security_map.calls[]` entry.
- Constraints:
  - flow tracking is structural with summary-based interprocedural forwarding,
  - only source/gate-tagged origins are attached.

#### 5) Failure modes and diagnostics
- This metadata path does not emit compile errors directly.
- If no origin can be inferred for an argument, no edge is emitted for that argument.

#### 6) Example usage
- Source:
  - `let raw = req.query("q");`
  - `db.exec(DbCap(), raw);`
- Emitted edge (conceptual):
  - `{ arg_index: 1, origin: "call:req.query", tags: ["source.http.query"] }`

#### 7) Tradeoffs and next steps
- Current origin inference is intentionally conservative and tag-oriented (origin chain details are flattened in summaries).
- Next step is to extend provenance detail beyond summary tags (for example branch-sensitive call-chain traces).

### Slice Explanation: `sec.audit` Callsite Evidence

#### 1) What it is
This slice enriches audit findings with deterministic `sampleCalls` evidence extracted from `security_map.calls`.

#### 2) Why it exists
Count-only findings hide which callsites triggered risk. Sample call evidence makes findings actionable in CI and review without requiring a separate metadata inspection step.

#### 3) How it works internally
- `run_security_audit` now attaches `sampleCalls` for selected findings.
- `call_samples_for_tag` and `call_samples_for_tags` filter tagged call records and emit bounded deterministic lists.
- each sample includes:
  - callee,
  - source location,
  - optional `argRoles`,
  - optional `originEdges` (when available from security-map origin tracking).

#### 4) Inputs/outputs and constraints
- Input: `SecurityMap.calls[]` plus finding tag id.
- Output: JSON evidence field `sampleCalls`.
- Constraints:
  - sample set is bounded (currently top 5 callsites per finding),
  - ordering follows deterministic AST traversal order.

#### 5) Failure modes and diagnostics
- If no tagged callsites exist, `sampleCalls` is empty or absent based on finding trigger conditions.
- No new compile diagnostics are introduced by this slice.

#### 6) Example usage
- `SQL_SELECT_WITHOUT_LIMIT` evidence now includes callsite objects like:
  - `{ callee: "db.exec", location: { file, line, column }, argRoles: [...] }`
- `SECRETS_REVEAL_USED` evidence now includes reveal callsite samples for triage.
- `CORS_ANY_ORIGIN` evidence now includes router middleware callsites (for example `withCors`).
- allowlisted bypass findings now include sampled matching callsites when bypass tags map to observed calls.

#### 7) Tradeoffs and next steps
- Current sampling is per-tag and static; it does not yet group by module or severity hot spots.
- Next step is to expand evidence coverage for remaining policy/exception hygiene findings with similarly deterministic samples.

### Slice Explanation: Exception Snapshot Evidence for `ALLOW_COUNT_HIGH`

#### 1) What it is
This slice adds bounded `sampleExceptions` evidence to the `ALLOW_COUNT_HIGH` finding so exception posture alerts include concrete representative `@allow` entries.

#### 2) Why it exists
Count-only posture findings are hard to act on. Sample snapshots let reviewers immediately see policy keys, tickets, bypass tags, and locations behind high exception volume.

#### 3) How it works internally
- audit now builds deterministic exception samples from `security_map.allows`.
- each sample includes:
  - policy key,
  - ticket,
  - expiry,
  - bypass tags,
  - source location.
- sample list is bounded (top 5 in parse order).

#### 4) Inputs/outputs and constraints
- Input: `security_map.allows`.
- Output: `ALLOW_COUNT_HIGH.evidence.sampleExceptions`.
- Constraints:
  - samples are representative, not exhaustive,
  - ordering is deterministic by existing allow ordering.

#### 5) Failure modes and diagnostics
- no compile-time diagnostics are introduced.
- if allows exist but malformed metadata is already filtered by prior parsing/validation stages.

#### 6) Example usage
- when allow count exceeds threshold, finding evidence now includes entries like:
  - `{ policyKey, ticket, expires, bypass, location }`.

#### 7) Tradeoffs and next steps
- this adds visibility for high exception volume but not trend/aging aggregates.
- next step is rollup evidence for expiring/soon-expiring exception posture.

### Slice Explanation: Trace-Aware `sec.audit` Text Previews

#### 1) What it is
This slice enhances text-mode `sec.audit` rendering to include concise sample-call previews, including provenance trace snippets when present.

#### 2) Why it exists
JSON output already carried detailed `sampleCalls`, but text output only listed finding IDs and suggestions. Operators needed quick actionable context directly in terminal output.

#### 3) How it works internally
- renderer now inspects each finding’s `evidence.sampleCalls`.
- for up to two samples per finding, it prints:
  - callee + file:line,
  - optional `trace=` preview derived from first origin-edge trace chain.
- if more samples exist, renderer prints a bounded `+N more` line.

#### 4) Inputs/outputs and constraints
- Input: audit finding evidence JSON.
- Output: richer text report lines under each finding.
- Constraints:
  - preview is intentionally bounded and concise,
  - trace preview is truncated to the first few steps.

#### 5) Failure modes and diagnostics
- no compile-time diagnostics are introduced.
- findings without `sampleCalls` remain unchanged in text output.

#### 6) Example usage
- finding line:
  - `HIGH SQL_RAW_ALLOWED_BY_POLICY ...`
- preview line:
  - `sample: db.exec@main.ai:12 trace=call:req.query -> call:wrap -> ...`

#### 7) Tradeoffs and next steps
- previews improve triage speed but are not full provenance dumps.
- next step is attaching selected provenance context directly into compiler diagnostics.

### Slice Explanation: Exception Expiry-Window Rollup Findings

#### 1) What it is
This slice adds an aggregate policy finding (`ALLOW_EXPIRY_WINDOW_ROLLUP`) that summarizes expired and soon-expiring `@allow` exceptions.

#### 2) Why it exists
Per-annotation findings (`ALLOW_EXPIRED`, `ALLOW_EXPIRING_SOON`) are precise but noisy at scale. A rollup gives a deterministic posture signal for CI gating and triage.

#### 3) How it works internally
- audit now collects:
  - expired exceptions,
  - soon-expiring exceptions (within configured window).
- after per-annotation checks, audit emits one rollup finding containing:
  - `expiredCount`,
  - `expiringSoonCount`,
  - `windowDays`,
  - bounded `sampleExceptions`.
- severity is `HIGH` when any expired exception exists, else `MEDIUM`.

#### 4) Inputs/outputs and constraints
- Input: parsed allow annotations + current date.
- Output: `ALLOW_EXPIRY_WINDOW_ROLLUP` finding evidence.
- Constraints:
  - rollup is deterministic and bounded,
  - it complements (does not replace) per-exception findings.

#### 5) Failure modes and diagnostics
- no compile-time diagnostics are introduced.
- malformed dates are already handled by prior validation; invalid entries are excluded from window math.

#### 6) Example usage
- if one exception is expired and two are expiring soon, rollup evidence includes:
  - `expiredCount=1`,
  - `expiringSoonCount=2`,
  - representative sampled entries for triage.

#### 7) Tradeoffs and next steps
- rollups improve posture visibility but do not yet include temporal trend analysis across builds.
- next step is adding trend/aging signals and policy-tunable severity weighting.

### Slice Explanation: Allowlist Bypass Sample Evidence

#### 1) What it is
This slice adds deterministic `sampleCalls` evidence to allowlist bypass findings by mapping `@allow(... bypass=[...])` tags to observed security-map callsites.

#### 2) Why it exists
Allowlist findings previously reported only annotation metadata. Reviewers still needed separate traces to see what sensitive calls were actually exercised.

#### 3) How it works internally
- for each `SecurityAllow`, audit evidence now includes:
  - policy key/ticket/location/expiry metadata,
  - bounded sampled callsites matching bypass tags.
- tag matching reuses `call_samples_for_tags`, so behavior stays deterministic and bounded.

#### 4) Inputs/outputs and constraints
- Input: `security_map.allows[]`, each allow’s `bypass` tags, and `security_map.calls[]`.
- Output: finding evidence for allowlist families includes `sampleCalls`.
- Constraints:
  - if bypass tags have no matching callsite tags, `sampleCalls` is empty.
  - coverage is limited to call-tag-based bypass families.

#### 5) Failure modes and diagnostics
- no compile-time diagnostics are introduced.
- stale or overbroad bypass tags can produce low-signal samples; this is surfaced as evidence quality, not a compiler error.

#### 6) Example usage
- `SECRETS_REVEAL_ALLOWLISTED` now includes reveal callsite samples when `effect.secrets.reveal` is observed.
- `INTERNAL_NET_CALL_ALLOWLISTED` now includes internal-net request callsite samples when `sink.net.internal_request` is observed.

#### 7) Tradeoffs and next steps
- evidence currently focuses on call-tag matches and does not include richer exception aggregation views.
- next step is adding deterministic evidence snapshots for non-call posture exceptions (for example high exception-count families).

### Slice Explanation: Middleware-Tagged Posture Evidence in `sec.audit`

#### 1) What it is
This slice extends `sec.audit` evidence to include middleware-tagged `sampleCalls` for CORS, security-headers, and auth/CSRF posture findings.

#### 2) Why it exists
Posture findings previously showed only policy state, without concrete code locations that configured router security. Middleware-tagged samples make these findings actionable.

#### 3) How it works internally
- `security_map.call_tags_for` now tags middleware bootstrap calls:
  - `middleware.cors`
  - `middleware.security_headers`
  - `middleware.csrf`
  - `middleware.auth`
- `run_security_audit` now attaches `sampleCalls` for posture findings by selecting these tags.
- coupling findings (for example cross-site cookie auth) sample across multiple middleware tags to preserve context.

#### 4) Inputs/outputs and constraints
- Input: middleware call tags in `SecurityMap.calls` plus policy posture.
- Output: finding evidence includes bounded middleware call samples.
- Constraints:
  - if middleware is configured externally and not present in source, samples may be empty,
  - sampling remains bounded and deterministic.

#### 5) Failure modes and diagnostics
- this slice does not change compile-time diagnostics.
- missing middleware call tags only affects evidence richness; finding detection still runs from policy posture.

#### 6) Example usage
- `CORS_ANY_ORIGIN` now carries `sampleCalls` with `withCors` / `cors.withCors` callsites.
- `CSP_DISABLED` now carries `sampleCalls` with `withSecurityHeaders` callsites.
- `CSRF_REQUIRED_BUT_DISABLED` now carries `sampleCalls` from `withAuth`/`withCsrf` context.

#### 7) Tradeoffs and next steps
- evidence is still callsite-centric; it does not yet include middleware attribute diffs per call in findings.
- next step is extending deterministic evidence to policy/exception hygiene families where location context is available.

### Slice Explanation: Context-First Stdlib Capability Signatures

#### 1) What it is
This slice extends capability/effect enforcement and sink indexing to handle both compact and context-first stdlib call signatures.

#### 2) Why it exists
The security stdlib contract models handlers around `Ctx` plus capabilities. Previous capability checks assumed the first argument was always the capability, which misaligned with `ctx, cap, ...` forms.

#### 3) How it works internally
- semantic analyzer now computes capability argument index per intrinsic family and argument count.
- sink-flow start offsets are computed with the same call-shape logic.
- security-map SQL query argument extraction now branches by call form for `db.exec`/`db.queryOne`.
- security-map role mapping now emits context-aware role labels for these forms.

#### 4) Inputs/outputs and constraints
- Input: intrinsic call name + argument list length.
- Output:
  - capability diagnostics pinned to the correct argument index,
  - sink checks applied to payload arguments only,
  - security-map roles/query extraction aligned to call shape.
- Constraints:
  - shape inference is currently arity-based,
  - only known intrinsic families are covered.

#### 5) Failure modes and diagnostics
- missing capability argument still emits `E2003`.
- wrong capability type emits `E2004` and now reports the exact argument position.

#### 6) Example usage
- valid:
  - `db.exec(dbCap, "SELECT ...")`
  - `db.exec(ctx, dbCap, "SELECT ...")`
- invalid:
  - `db.exec(ctx, netCap, "SELECT ...")` -> `E2004` at argument 2.

#### 7) Tradeoffs and next steps
- current mapping is intrinsic-name based and arity-driven.
- next step is to lift this into richer typed stdlib symbol metadata so alias/value-call paths can share the same enforcement.

### Slice Explanation: Typed Symbol Resolution for Alias/Value Calls

#### 1) What it is
This slice adds local callable alias resolution so stdlib intrinsic semantics apply even when calls go through bound values.

#### 2) Why it exists
Without alias resolution, code like `let exec = db.exec; exec(...)` bypasses canonical callee matching, weakening capability/effect/sink checks and audit metadata.

#### 3) How it works internally
- semantic scope now tracks `callable_aliases` alongside type bindings.
- when a `let` binds a callable intrinsic/function symbol, an alias entry is recorded.
- call analysis resolves callee name through alias chains before:
  - intrinsic capability/effect enforcement,
  - sink-flow restriction checks.
- `security_map` mirrors this with alias-aware call collection and origin inference.

#### 4) Inputs/outputs and constraints
- Input: local `let` bindings that reference callable symbols.
- Output: canonical callee resolution for analysis and emitted metadata.
- Constraints:
  - alias resolution is currently local-scope only,
  - resolution is limited to symbols known as intrinsic/function call targets.

#### 5) Failure modes and diagnostics
- capability mismatches still emit `E2004`, now anchored to resolved canonical intrinsic signatures.
- unresolved alias chains fall back to existing unknown-call diagnostics behavior.

#### 6) Example usage
- source:
  - `let exec = db.exec;`
  - `exec(dbCap, "SELECT ...");`
- result:
  - semantic checks treat call as `db.exec(...)`,
  - `security_map` records callee `db.exec` with sink/capability tags.

#### 7) Tradeoffs and next steps
- current alias support is lexical and local, without cross-function symbol-table propagation.
- next step is to extend typed symbol metadata through deeper interprocedural value-call forwarding.

### Slice Explanation: Typed Symbol Resolution for Member Alias Value Calls

#### 1) What it is
This slice extends alias-aware call resolution from direct callable aliases to member value-call aliases such as `let repo = db; repo.exec(...)`.

#### 2) Why it exists
Without member alias canonicalization, capability/effect/sink checks could miss real stdlib sink calls when the callee is reached through a value alias plus member access.

#### 3) How it works internally
- alias inference now accepts typed stdlib namespace stems as alias targets (for example `db`, `fs`, `secrets`).
- callee resolution now supports dotted prefix alias rewrites:
  - if a direct alias exists, it is resolved first.
  - otherwise, for dotted names, the head segment is alias-resolved and the full callee is rebuilt (for example `repo.exec` -> `db.exec`).
- the same resolution path is implemented in both semantic analysis and `security_map` collection/origin inference.

#### 4) Inputs/outputs and constraints
- Input: local value aliases whose callsites are member expressions.
- Output: canonical callee names for enforcement and metadata (`db.exec` instead of `repo.exec`).
- Constraints:
  - resolution remains lexical and local-scope,
  - namespace matching is currently stdlib-family based.

#### 5) Failure modes and diagnostics
- capability mismatches continue to emit `E2004`, now also for member alias forms.
- unresolved member alias paths fall back to existing unknown-call behavior (no false positive sink tagging).

#### 6) Example usage
- valid:
  - `let repo = db;`
  - `repo.exec(dbCap, "SELECT id FROM users");`
- invalid:
  - `repo.exec(netCap, "SELECT id FROM users");` -> `E2004` (expects `DbCap`).

#### 7) Tradeoffs and next steps
- current namespace stem detection is explicit and conservative.
- next step is deeper value-call forwarding across function boundaries and richer callable-value shapes.

### Slice Explanation: Interprocedural Callable-Forwarding Alias Resolution

#### 1) What it is
This slice adds callable-forwarding summaries so alias resolution can traverse helper-function returns, not only direct local/member alias bindings.

#### 2) Why it exists
Patterns like `let exec = getExec(); exec(...)` previously lost canonical sink identity when `getExec` returned a stdlib callable symbol. That weakened sink enforcement and metadata consistency.

#### 3) How it works internally
- semantic analyzer now computes iterative callable-forwarding summaries per function body.
- security-map builder computes the same summary class for canonical call tagging.
- alias inference can now resolve:
  - direct/local aliases,
  - member aliases,
  - helper-call aliases via forwarding summaries.
- canonicalization still resolves to stable stdlib/middleware symbols (for example `db.exec`).

#### 4) Inputs/outputs and constraints
- Input: function bodies with tail expressions that forward callable symbols.
- Output: canonical alias targets used by semantic checks and security-map call records.
- Constraints:
  - summaries are conservative and expression-structure based,
  - callable-forwarding currently targets stdlib/middleware families and known callable symbols.

#### 5) Failure modes and diagnostics
- no new diagnostic codes were introduced.
- unresolved/non-forwarding helper returns fall back to existing call resolution behavior.
- capability/effect diagnostics (`E2003`, `E2004`) remain unchanged and now apply to forwarded alias paths.

#### 6) Example usage
- helper:
  - `fn getExec() { db.exec }`
- caller:
  - `let exec = getExec();`
  - `exec(dbCap, "SELECT id FROM users");`
- result:
  - semantic checks and `security_map` both treat the call as canonical `db.exec`.

#### 7) Tradeoffs and next steps
- forwarding summaries are intentionally lightweight and avoid full higher-order callable typing.
- next step is extending this to richer callable-value shapes (for example capability-object function fields).

### Slice Explanation: Callable-Aware Origin Notes in Sink Diagnostics

#### 1) What it is
This slice upgrades sink-flow diagnostic origin notes to resolve alias and callable-forwarding summaries for call expressions.

#### 2) Why it exists
Security diagnostics already included origin notes, but helper-forwarded calls could report intermediate helper names instead of canonical trust-boundary origins.

#### 3) How it works internally
- sink-flow enforcement now passes active callable alias state into origin-note formatting.
- origin-note rendering resolves:
  - local/member aliases,
  - callable-forward summary chains.
- call-expression notes now prefer canonical resolved names (for example `req.query(...)`).

#### 4) Inputs/outputs and constraints
- Input: sink argument expression + inferred type + callable alias/summary context.
- Output: diagnostic note line: `origin: value comes from call ...`.
- Constraints:
  - this slice targets call-expression origin notes,
  - identifier/member-only notes remain type-based and scope-based.

#### 5) Failure modes and diagnostics
- no new diagnostic codes were added.
- unresolved call targets fall back to existing generic call-expression origin notes.

#### 6) Example usage
- source:
  - `fn getRaw() -> Untrusted<String> { req.query("q") }`
  - `db.exec(cap, getRaw())`
- diagnostic note now points to canonical call origin:
  - `origin: value comes from call req.query(...)`.

#### 7) Tradeoffs and next steps
- this improves call-origin clarity but does not yet inject full provenance trace chains into compiler diagnostics.
- next step is exposing trace-chain context in editor tooling and diagnostic enrichments.

### Slice Explanation: Iterative Interprocedural Origin Summaries

#### 1) What it is
This slice upgrades function-origin summarization from a single-pass snapshot to an iterative fixed-point computation so deeper forwarding chains preserve source tags.

#### 2) Why it exists
Single-pass summaries only captured direct/one-hop forwarding. Multi-hop helper chains could drop source-origin tags before reaching sinks, weakening `security_map` evidence.

#### 3) How it works internally
- `build_function_origin_summaries` now:
  - gathers all functions,
  - recomputes summaries iteratively using currently known summaries,
  - stops when no summary changes or when bounded rounds are exhausted.
- summary inference still uses deterministic forms:
  - `FromParam { index, forwarded_calls }` for forwarding functions,
  - `Tagged { tags, trace }` for source/gate-returning functions.
- call argument origin inference consumes these converged summaries, so sink edges retain source tags across deeper helper stacks.

#### 4) Inputs/outputs and constraints
- Input: full program function set.
- Output: converged summary map used by origin inference.
- Constraints:
  - round count is bounded by function count (deterministic runtime),
  - summaries intentionally carry compact forwarding/source trace fragments, not unbounded control-flow path graphs.

#### 5) Failure modes and diagnostics
- this path does not add compile diagnostics directly.
- if no stable informative summary is inferable, function summary is omitted and downstream edges may be absent.

#### 6) Example usage
- flow:
  - `queryParam()` -> `passThrough()` -> `passthroughTwice()` -> `wrap()` -> `db.exec(...)`
- result:
  - sink argument edge keeps `source.http.query` tag instead of losing origin through helper depth.

#### 7) Tradeoffs and next steps
- summaries remain intentionally compact and avoid path explosion.
- next step is richer call-chain provenance output while keeping deterministic bounded analysis.

### Slice Explanation: Origin Edge Provenance Trace Chains

#### 1) What it is
This slice extends `origin_edges` with a deterministic `trace` list that records source-to-sink forwarding steps across helper calls.

#### 2) Why it exists
Tag-only origin summaries answered "what source family reached the sink" but not "how it got there". Trace chains make multi-hop provenance inspectable without introducing dynamic analysis.

#### 3) How it works internally
- `TrackedOrigin` now carries:
  - canonical `origin` label,
  - source/gate `tags`,
  - ordered `trace` call markers.
- direct source/gate calls initialize trace with their call marker.
- param-forwarding summaries append forwarded call markers plus current call marker.
- tagged summaries preserve source traces and append the current callee marker.
- merge points keep deterministic behavior by requiring the same canonical origin and using common trace-prefix merging.

#### 4) Inputs/outputs and constraints
- Input: expression-level origin inference + function origin summaries.
- Output: `security_map.calls[].origin_edges[].trace`.
- Constraints:
  - trace depth remains bounded by static call expression depth and summary rounds,
  - traces are deterministic and deduplicated for repeated adjacent markers.

#### 5) Failure modes and diagnostics
- no new compile-time diagnostics are emitted by this slice.
- if branches diverge to incompatible origins, merge yields no edge (same as prior conservative behavior).

#### 6) Example usage
- flow:
  - `queryParam -> passThrough -> wrap -> db.exec`
- emitted edge includes trace markers such as:
  - `call:req.query`, `call:queryParam`, `call:passThrough`, `call:wrap`.

#### 7) Tradeoffs and next steps
- traces expose forwarding history but do not yet encode branch-sensitive alternatives as separate paths.
- next step is to surface these traces directly in compiler diagnostics and editor/tooling explainability.

### Slice Explanation: Capability-Typed Namespace Seeding for Member Calls

#### 1) What it is
This slice seeds callable alias resolution from capability-typed parameters and typed `let` bindings so member callsites can canonicalize without requiring explicit local alias assignment.

#### 2) Why it exists
Before this slice, canonicalization for member calls depended on value aliases like `let repo = db; repo.exec(...)`. That missed realistic shapes where the capability is already named differently in the signature (for example `repo: DbCap`) or forwarded through helper functions.

#### 3) How it works internally
- semantic analyzer now seeds callable alias state from capability-typed parameters using a deterministic mapping:
  - `DbCap`/`TxCap` -> `db`
  - `NetCap`/`InternalNetCap` -> `httpClient`
  - `FsCap` -> `fs`
  - `SecretsCap` -> `secrets`
- the same seed mapping is applied in callable-forwarding summary inference, so helper returns like `fn getExec(repo: DbCap) { repo.exec }` canonicalize to `db.exec`.
- typed `let` bindings now also fallback to namespace seeding when value-based alias inference is unavailable.
- `security_map` uses the same seed rules in call collection, origin inference, and forwarding summaries, keeping metadata and semantic diagnostics aligned.

#### 4) Inputs/outputs and constraints
- Input:
  - function parameter type annotations,
  - optional `let` type annotations,
  - member call expressions and callable forwarding helpers.
- Output:
  - canonical call names for enforcement/metadata (`db.exec` instead of `repo.exec`),
  - preserved sink tags, capability/effect checks, and origin edge tracking on those calls.
- Constraints:
  - mapping is intentionally explicit to known capability types in v0.1-lite,
  - this does not yet infer arbitrary user-defined callable object shapes.

#### 5) Failure modes and diagnostics
- no new diagnostic codes were introduced.
- capability errors remain existing stable codes:
  - `E2003` for missing capability argument,
  - `E2004` for capability type mismatch.
- unresolved non-capability member call patterns still fall back to existing unknown callable behavior.

#### 6) Example usage
- direct typed member call (no local alias required):
  - `fn ok(repo: DbCap) effects { db.write } -> Int { repo.exec(repo, "..."); 1 }`
- forwarded typed member callable:
  - `fn getExec(repo: DbCap) { repo.exec }`
  - `let exec = getExec(repo); exec(repo, raw);`
- both semantic checks and `security_map` treat these as canonical `db.exec`.

#### 7) Tradeoffs and next steps
- capability namespace seeding removes a practical blind spot while preserving deterministic, local analysis.
- next step is extending this approach beyond built-in capability names to richer user-defined callable object field shapes.

### Slice Explanation: Forwarding-Chain Context in Sink Diagnostics

#### 1) What it is
This slice enriches sink-flow origin notes for call expressions with a compact callable-forwarding chain when helper forwarding is involved.

#### 2) Why it exists
Canonicalized origin notes already reported the final source call (for example `req.query(...)`), but they did not show the helper path that carried the value to the sink. That reduced debuggability when many wrapper helpers were present.

#### 3) How it works internally
- semantic origin-note rendering now computes a deterministic callable summary chain by following callable-forward summaries from the callsite callee to the canonical endpoint.
- when the chain has multiple steps, the diagnostic note includes both:
  - canonical source call,
  - forwarding chain text (for example `getRaw -> req.query`).
- single-step/direct calls keep the existing concise origin note.

#### 4) Inputs/outputs and constraints
- Input:
  - call expression callee,
  - active alias map,
  - callable-forward summary map.
- Output:
  - sink diagnostic origin note with optional forwarding-chain suffix.
- Constraints:
  - chain generation is bounded and cycle-safe (seen-set guarded),
  - output remains deterministic for identical source input.

#### 5) Failure modes and diagnostics
- no new diagnostic codes were added.
- if summary chaining is unavailable, diagnostics fall back to the existing call-origin note shape.

#### 6) Example usage
- source:
  - `fn getRaw() -> Untrusted<String> { req.query("q") }`
  - `db.exec(cap, getRaw())`
- diagnostic note:
  - `origin: value comes from call req.query(...) via forwarding chain getRaw -> req.query`

#### 7) Tradeoffs and next steps
- this improves explainability while keeping diagnostics single-line and compact.
- next step is exposing richer multi-step provenance (for example trace arrays) in editor tooling surfaces (hover/code actions/LSP diagnostics data).

### Slice Explanation: Let-Bound Provenance Notes for Sink Diagnostics

#### 1) What it is
This slice adds scoped value-origin tracking in semantic analysis so sink diagnostics can explain provenance for identifier arguments, not only inline call expressions.

#### 2) Why it exists
Before this slice, diagnostics for `db.exec(cap, raw)` where `raw` was produced by a helper (`let raw = getRaw()`) only showed generic identifier type origin. That hid the trust-boundary path.

#### 3) How it works internally
- analyzer now keeps a scoped `value_origins` map during function-body analysis.
- on each `let` binding, it infers/stores origin messages when the value comes from:
  - call expressions (including callable-summary forwarding chains),
  - identifier forwarding,
  - selected member forwarding.
- block analysis snapshots/restores origin state so shadowing and lexical scope stay deterministic.
- sink-flow checks consume this map and upgrade identifier origin notes when available.

#### 4) Inputs/outputs and constraints
- Input:
  - function body statements/tail expressions,
  - callable alias/forward summary context.
- Output:
  - improved diagnostic note text:
    - from `origin: identifier raw carries type ...`
    - to `origin: value comes from call req.query(...) via forwarding chain ...` when inferable.
- Constraints:
  - origin tracking is intentionally lightweight and text-oriented in v0.1-lite,
  - it does not yet emit structured trace arrays in semantic diagnostics.

#### 5) Failure modes and diagnostics
- no new diagnostic codes were added.
- if provenance cannot be inferred for a binding, diagnostics fall back to existing identifier/type-based origin notes.

#### 6) Example usage
- source:
  - `let raw = getRaw();`
  - `db.exec(cap, raw);`
- diagnostic note now includes canonical source and helper chain:
  - `origin: value comes from call req.query(...) via forwarding chain getRaw -> req.query`.

#### 7) Tradeoffs and next steps
- this raises explainability for common helper-and-variable flows with minimal analysis overhead.
- next step is to expose the same provenance in structured diagnostic metadata for editor/LSP quick-fix tooling.

### Slice Explanation: Expiry-Window Aging Metrics and Severity Inputs in `sec.audit`

#### 1) What it is
This slice extends `ALLOW_EXPIRY_WINDOW_ROLLUP` evidence with deterministic aging metrics and explicit severity-input metadata.

#### 2) Why it exists
The rollup previously provided counts and samples, but not enough structured age context to explain risk pressure or tune policy thresholds transparently.

#### 3) How it works internally
- audit now computes per-window expiry deltas (`expiry_day - today_day`) for expired/soon-expiring exceptions.
- rollup evidence now includes:
  - `minDaysUntilExpiry`
  - `medianDaysUntilExpiry`
  - `maxDaysUntilExpiry`
  - `expiringIn7DaysCount`
  - `expiringIn30DaysCount`
  - `severityInputs.expiringSoonHighThreshold`
- rollup severity remains deterministic and now uses explicit inputs:
  - `HIGH` if any expired exception exists,
  - `HIGH` if expiring-soon count crosses configured threshold,
  - otherwise `MEDIUM`.

#### 4) Inputs/outputs and constraints
- Input:
  - parsed `@allow` expiry dates,
  - current UTC day index used elsewhere in audit.
- Output:
  - richer `ALLOW_EXPIRY_WINDOW_ROLLUP` evidence payload for text/json consumers.
- Constraints:
  - date parsing remains strict ISO-like (`YYYY-MM-DD`),
  - metrics are static, deterministic, and bounded to current report data only (no historical storage yet).

#### 5) Failure modes and diagnostics
- no new compiler diagnostics were added.
- if no valid expiry dates are available in the window, rollup falls back to existing count/sample evidence.

#### 6) Example usage
- a report with one expired allow now includes both:
  - `expiredCount: 1`
  - age fields such as `minDaysUntilExpiry` and `medianDaysUntilExpiry`,
  - plus `severityInputs.expiringSoonHighThreshold` for explainable severity mapping.

#### 7) Tradeoffs and next steps
- this gives deterministic aging context in a single report but does not yet compute longitudinal trends across multiple runs.
- next step is adding persisted trend deltas (for example day-over-day expiry pressure change) while keeping audits reproducible.

### Slice Explanation: Baseline Trend Deltas for `sec.audit`

#### 1) What it is
This slice adds optional baseline comparison to `sec.audit` and emits deterministic trend deltas in the audit report.

#### 2) Why it exists
Single-report posture is useful, but teams also need quick answers to "did risk get better or worse since last stable run?" without introducing non-deterministic storage dependencies.

#### 3) How it works internally
- core audit now supports `run_security_audit_with_baseline(...)` alongside the existing default runner.
- when a baseline report is provided, output includes `trend` with:
  - `baselinePolicyHash`
  - `baselineRiskScore`
  - `riskScoreDelta`
  - `findingCountDelta`
  - `severityDeltas` (`LOW|MEDIUM|HIGH|CRITICAL`)
  - bounded `addedFindingIds` / `resolvedFindingIds`
- CLI `sec audit` accepts `--baseline <path>` and parses prior JSON report to populate trend output.
- text rendering includes a compact trend summary line plus added/resolved IDs when present.

#### 4) Inputs/outputs and constraints
- Input:
  - current policy/security_map,
  - optional prior `AuditReport` JSON.
- Output:
  - regular audit report plus optional `trend` object.
- Constraints:
  - comparison is purely ID/count/score based and deterministic,
  - no hidden state; caller controls baseline file selection,
  - added/resolved finding ID lists are bounded.

#### 5) Failure modes and diagnostics
- baseline loading/parsing errors are explicit CLI failures with non-zero exit (`Err(2)` path).
- missing baseline keeps existing behavior (no trend field emitted).

#### 6) Example usage
- run once:
  - `ailang sec audit --path . --format json > build/audit-baseline.json`
- compare later:
  - `ailang sec audit --path . --format text --baseline build/audit-baseline.json`
- output shows whether risk score and finding counts increased or decreased and which finding IDs were added/resolved.

#### 7) Tradeoffs and next steps
- this enables deterministic two-point comparison but does not yet persist/aggregate multi-run history.
- next step is opt-in history storage and windowed trend aggregation (for example 7-day/30-day delta summaries).

### Slice Explanation: Structured Diagnostic Tags for Security Tooling

#### 1) What it is
This slice adds structured tag metadata on compiler diagnostics and applies it to core M4 security/effect/policy errors.

#### 2) Why it exists
Roadmapped editor/LSP support needs machine-consumable diagnostics (not only text). Stable tags let clients group/filter diagnostics and drive targeted quick-fix UX without parsing note strings.

#### 3) How it works internally
- `Diagnostic` now includes `tags: Vec<String>` plus deduplicating builder method `with_tag(...)`.
- semantic analyzer now tags representative diagnostics:
  - taint-to-sink (`E1002`) -> `security`, `taint`, `sink`
  - secret-to-sink (`E1003/E1004/E1005`) -> `security`, `secret`, `sink`
  - capability issues (`E2003/E2004`) -> `security`, `capability`
  - forbidden effects (`E2002`) -> `security`, `policy`, `effects`
  - undeclared effects (`E4002`) -> `effects`
  - strict JSON schema errors (`E4004`) -> `security`, `schema`
- `@allow` annotation diagnostics in `security_map` now carry `security` + `policy` tags (`A7001/A7002`).
- plain/color rendering remains unchanged (tags are metadata for structured consumers).

#### 4) Inputs/outputs and constraints
- Input: existing diagnostic construction paths in semantic/policy parsing flows.
- Output: same diagnostics plus stable tag vectors.
- Constraints:
  - tag assignment remains deterministic,
  - duplicate tag insertion is prevented by `with_tag`.

#### 5) Failure modes and diagnostics
- no diagnostic codes or message text changed for existing failures.
- if tag coverage is missing for future diagnostics, behavior remains backward-compatible (empty tag list).

#### 6) Example usage
- an `E1002` SQL-taint diagnostic now includes tags:
  - `["security", "taint", "sink"]`
- a policy-expired allow diagnostic `A7002` includes:
  - `["security", "policy"]`

#### 7) Tradeoffs and next steps
- this provides a minimal structured layer without changing user-facing output shape.
- next step is exposing these tags through the future LSP diagnostic payload and code-action routing.

### Slice Explanation: CLI Diagnostics JSON Emission for Tooling

#### 1) What it is
This slice adds `ailang check --emit diagnostics-json`, a machine-readable diagnostics output mode for automation/editor bootstrap workflows.

#### 2) Why it exists
Structured tags and spans are most useful when directly consumable by tools. This mode removes the need to parse ANSI/plain text diagnostics.

#### 3) How it works internally
- CLI `EmitTarget` now includes `DiagnosticsJson`.
- on check success with diagnostics-json mode, CLI emits `[]` with no extra stdout lines.
- on validation/analysis failure, CLI prints serialized diagnostics (`severity`, `code`, `message`, `span`, `notes`, `tags`) and returns non-zero status.
- existing default behavior remains unchanged for plain/color output modes.

#### 4) Inputs/outputs and constraints
- Input:
  - project path and optional emit mode.
- Output:
  - JSON array of diagnostics when `--emit diagnostics-json` is selected.
- Constraints:
  - this is currently wired to `check` command only,
  - output is deterministic and directly based on compiler diagnostic structs.

#### 5) Failure modes and diagnostics
- invalid project/manifest/source still produce diagnostics; only rendering format changes in diagnostics-json mode.
- non-zero exit status is preserved for failing checks.

#### 6) Example usage
- success:
  - `ailang check --path examples/hello --emit diagnostics-json`
  - output: `[]`
- failure:
  - returns array with diagnostic tags (for example `security`, `capability`) suitable for tool pipelines.

#### 7) Tradeoffs and next steps
- this is a thin bridge to tooling and does not yet provide an LSP server.
- next step is exposing the same structured diagnostics through an official language-server process.

### Slice Explanation: Persisted Audit Report Output (`--write-report`)

#### 1) What it is
This slice adds `ailang sec audit --write-report <path>` so audit reports can be persisted as JSON artifacts for later baseline/trend comparison.

#### 2) Why it exists
Baseline comparison is most useful when teams can capture and store reports in CI artifacts or repository-local build outputs without manual redirection pipelines.

#### 3) How it works internally
- `sec audit` accepts optional `--write-report`.
- after report generation, CLI serializes the exact `AuditReport` JSON and writes it to the requested path.
- parent directories are created automatically when needed.
- write/serialization failures return explicit non-zero exit (`Err(2)`).

#### 4) Inputs/outputs and constraints
- Input:
  - current audit run result,
  - output path from `--write-report`.
- Output:
  - persisted JSON report identical to the in-memory audit structure.
- Constraints:
  - report writing is opt-in only,
  - no implicit rotation/history policy is applied yet.

#### 5) Failure modes and diagnostics
- invalid/unwritable path results in CLI error and non-zero exit.
- report emission to stdout still follows selected `--format` and is unaffected.

#### 6) Example usage
- capture baseline:
  - `ailang sec audit --path . --format json --write-report build/audit-baseline.json`
- compare against baseline later:
  - `ailang sec audit --path . --baseline build/audit-baseline.json --write-report build/audit-current.json`

#### 7) Tradeoffs and next steps
- this provides deterministic artifact persistence but not managed time-series history.
- next step is an optional rolling history mode (for example timestamped report snapshots plus window summaries).

### Slice Explanation: Audit History Directory (`--history-dir`) with Auto-Baseline

#### 1) What it is
This slice adds `ailang sec audit --history-dir <path>` for opt-in report history capture and automatic baseline loading from prior history entries.

#### 2) Why it exists
Single-file baseline comparisons are useful but manual. Teams need a low-friction way to persist each run and get deterministic trend deltas across iterative runs.

#### 3) How it works internally
- CLI `sec audit` now accepts optional `--history-dir`.
- when `--baseline` is not provided and `--history-dir` is set:
  - CLI loads the latest `.json` report in that directory as baseline (if present),
  - audit engine computes trend deltas against that loaded baseline.
- after each successful audit run with `--history-dir`, CLI writes the current report to a timestamped history file:
  - `audit-<timeMs>-<nanos>.json`.
- auxiliary artifact hints are emitted:
  - text mode: stdout,
  - json mode: stderr (stdout remains JSON-only).

#### 4) Inputs/outputs and constraints
- Input:
  - `sec audit --history-dir <dir>` (optionally with `--baseline` and/or `--write-report`).
- Output:
  - persisted timestamped history report under the target directory,
  - optional `trend` section when a prior baseline is available.
- Constraints:
  - explicit `--baseline` takes precedence over auto-loaded history baseline,
  - history baseline selection is lexicographic over `.json` filenames.

#### 5) Failure modes and diagnostics
- unreadable history directory or malformed latest JSON baseline returns CLI error (`Err(2)`).
- unwritable history directory/path returns CLI error (`Err(2)`).
- when no prior history exists, run succeeds without trend output.

#### 6) Example usage
- first run (no trend yet):
  - `ailang sec audit --path examples/hello --format json --history-dir build/audit-history`
- second run (auto-baseline from first report):
  - `ailang sec audit --path examples/hello --format json --history-dir build/audit-history`
  - output includes `trend` with deterministic deltas.

#### 7) Tradeoffs and next steps
- history selection is currently single-baseline (latest only), not windowed aggregation.
- next step is optional rolling summaries (for example 7-day/30-day trend windows) built on top of history artifacts.

### Slice Explanation: Strict JSON-Only Stdout Contracts for CLI Modes

#### 1) What it is
This slice hardens machine-readable CLI outputs so JSON modes are strictly JSON on stdout.

#### 2) Why it exists
Tooling and editor integrations require parseable stdout payloads without human text prefixes/suffixes.

#### 3) How it works internally
- `check --emit diagnostics-json` now suppresses success banner lines and emits only JSON (`[]` or diagnostics array) on stdout.
- `sec audit --format json` now emits only the serialized `AuditReport` on stdout.
- human-oriented context lines (for example `security map: ...`) are redirected to stderr in JSON mode.
- text modes keep existing human-readable output behavior.
- integration tests in the CLI crate assert JSON parsing directly from stdout for both command families.

#### 4) Inputs/outputs and constraints
- Input:
  - `ailang check --emit diagnostics-json`
  - `ailang sec audit --format json`
- Output:
  - stdout is always valid JSON payload for the selected command in JSON mode.
  - stderr may contain auxiliary human hints.
- Constraints:
  - contract applies only to explicit JSON modes, not text/default modes.

#### 5) Failure modes and diagnostics
- failing checks still return non-zero exit codes, but diagnostics remain machine-readable JSON on stdout.
- failing audit baseline/report path operations still report CLI errors on stderr with non-zero exit.

#### 6) Example usage
- `ailang check --path examples/hello --emit diagnostics-json | jq .`
- `ailang sec audit --path examples/hello --format json | jq '.summary'`

#### 7) Tradeoffs and next steps
- this keeps compatibility for human workflows while making JSON modes deterministic for automation.
- next step is wiring the same contract into future LSP transport/output utilities and CI adapters.

### Slice Explanation: Uniform Logging Sample-Call Evidence (`LOG_USER_AGENT_ENABLED`)

#### 1) What it is
This slice extends `LOG_USER_AGENT_ENABLED` findings to include deterministic `sampleCalls` evidence, matching the existing logging finding format.

#### 2) Why it exists
`LOG_STRUCTURED_ONLY_DISABLED` and `LOG_REMOTE_IP_ENABLED` already carried callsite evidence. `LOG_USER_AGENT_ENABLED` was still count-only, which made triage less actionable.

#### 3) How it works internally
- audit rule payload for `LOG_USER_AGENT_ENABLED` now adds:
  - `sampleCalls: call_samples_for_tag(security_map, "sink.log.emit", 5)`.
- call sampling reuses the existing bounded/deterministic call extraction path used by other logging findings.
- `sec_audit` tests now assert sample evidence presence and expected callee content for this finding.

#### 4) Inputs/outputs and constraints
- Input:
  - policy with `logging.include_user_agent = true`,
  - `security_map` call records containing log sink callsites.
- Output:
  - finding evidence now includes `includeUserAgent` + `sampleCalls`.
- Constraints:
  - evidence is bounded (max 5 samples) and deterministic by existing traversal order.

#### 5) Failure modes and diagnostics
- if no log sink callsites are tagged, the finding still emits with empty sample set.
- no compiler diagnostic codes changed; this is audit evidence enrichment only.

#### 6) Example usage
- finding payload now resembles:
  - `{ "includeUserAgent": true, "sampleCalls": [{ "callee": "log.info", ... }] }`

#### 7) Tradeoffs and next steps
- this improves evidence parity for logging posture but does not yet add module-level grouping for logging hotspots.
- next step is optional grouping/aggregation of sample evidence by module/service area.

### Slice Explanation: CORS Origin Reflection Policy + Audit Finding

#### 1) What it is
This slice adds explicit CORS origin-reflection posture support to policy parsing, security metadata, and audit findings.

#### 2) Why it exists
Origin reflection is a high-risk CORS pattern. The policy model already had `forbid_reflect_origin`, but there was no explicit runtime posture flag or dedicated audit signal when reflection was enabled.

#### 3) How it works internally
- policy parser now accepts:
  - `cors.reflect_origin = <bool>`.
- policy validation now rejects forbidden reflection combinations:
  - `cors.reflect_origin=true` with `cors.forbid_reflect_origin=true` -> `P6003`.
- `security_map` middleware tagging now emits:
  - `reflectOrigin` attribute for `middleware.cors`.
- `sec.audit` adds deterministic finding:
  - `CORS_REFLECT_ORIGIN_ENABLED` (HIGH) with `sampleCalls` evidence from `middleware.cors` callsites.

#### 4) Inputs/outputs and constraints
- Input:
  - policy CORS reflection keys,
  - router middleware callsites (`withCors` / `cors.withCors`).
- Output:
  - validated policy state,
  - middleware attribute evidence in `security_map`,
  - audit finding with bounded callsite samples.
- Constraints:
  - finding requires reflection posture enabled in middleware attrs,
  - policy validation blocks explicitly forbidden reflection configs at parse/build time.

#### 5) Failure modes and diagnostics
- invalid policy combination emits `P6003` with guidance to disable reflection or relax `forbid_reflect_origin`.
- when reflection is enabled but not forbidden, audit emits `CORS_REFLECT_ORIGIN_ENABLED` for posture visibility.

#### 6) Example usage
- policy:
  - `[cors] reflect_origin = true; forbid_reflect_origin = false`
- audit:
  - includes `CORS_REFLECT_ORIGIN_ENABLED` with `sampleCalls` pointing to CORS middleware callsites.

#### 7) Tradeoffs and next steps
- current finding is posture-level and does not yet distinguish narrow allowlisted reflection strategies.
- next step is finer-grained middleware evidence (for example exact matching strategy tags) for richer risk scoring.

### Slice Explanation: DNS Resolution Posture Finding (`DNS_RESOLUTION_DISABLED`)

#### 1) What it is
This slice adds a deterministic SSRF posture finding when DNS resolution checks are disabled in production policy.

#### 2) Why it exists
The security baseline requires DNS/final-IP validation for outbound public requests. Without an explicit audit signal, teams can disable `net.ssrf.resolve_dns` and miss a meaningful regression in SSRF defenses.

#### 3) How it works internally
- `sec.audit` now emits `DNS_RESOLUTION_DISABLED` when:
  - `policy.env == "prod"` and
  - `policy.net_ssrf.resolve_dns == false`.
- evidence payload includes:
  - `resolveDns`, `env`,
  - bounded `sampleCalls` from `sink.net.public_request` callsites.
- existing sample-call extraction is reused, preserving deterministic ordering and bounds.

#### 4) Inputs/outputs and constraints
- Input:
  - effective policy (`env`, `net.ssrf.resolve_dns`),
  - `security_map` public-net sink callsite tags.
- Output:
  - `DNS_RESOLUTION_DISABLED` finding with severity `HIGH` and callsite evidence.
- Constraints:
  - finding is production-focused (dev/staging are not flagged by this rule).

#### 5) Failure modes and diagnostics
- if no public-net callsites are present, finding still emits with empty sample set.
- no compiler parse/type diagnostics changed; this is an audit-layer posture signal.

#### 6) Example usage
- policy:
  - `[policy] env = "prod"`
  - `[net.ssrf] resolve_dns = false`
- audit:
  - includes `DNS_RESOLUTION_DISABLED` with `httpClient.get(...)` sample callsites when present.

#### 7) Tradeoffs and next steps
- this flags DNS-resolution posture but does not yet differentiate environment-specific allowlist exceptions.
- next step is adding policy-governed exception metadata for controlled prod waivers.

### Slice Explanation: Public Egress Domain Policy Finding (`PUBLIC_EGRESS_NO_DOMAIN_POLICY`)

#### 1) What it is
This slice adds a deterministic SSRF posture finding when outbound public network calls are used but no domain allowlist/blocklist policy is configured.

#### 2) Why it exists
The security baseline treats outbound egress posture as first-class. Without explicit domain controls, public egress remains overly broad even when URL typing and redirect checks are present.

#### 3) How it works internally
- policy model now persists `[net.public]` domain lists:
  - `allowed_domains`
  - `blocked_domains`
- `sec.audit` now emits `PUBLIC_EGRESS_NO_DOMAIN_POLICY` when:
  - at least one `sink.net.public_request` callsite exists in `security_map`,
  - `policy.net_public.allowed_domains` is empty, and
  - `policy.net_public.blocked_domains` is empty.
- evidence payload includes:
  - `allowedDomains`, `blockedDomains`, `env`,
  - bounded `sampleCalls` from `sink.net.public_request`.

#### 4) Inputs/outputs and constraints
- Input:
  - effective net-public policy domain lists,
  - `security_map` public-net sink callsites.
- Output:
  - `PUBLIC_EGRESS_NO_DOMAIN_POLICY` finding with severity `MEDIUM`.
- Constraints:
  - finding is usage-gated; it does not emit when no public-net sink callsites are present.

#### 5) Failure modes and diagnostics
- no compiler parse/type diagnostics are introduced by this slice.
- if public egress is used and domain policy is omitted, audit emits posture finding with deterministic evidence.
- if domain policy is configured (allowlist or blocklist), this finding is suppressed.

#### 6) Example usage
- policy:
  - `[net.public] allowed_domains = []`
  - `[net.public] blocked_domains = []`
- code:
  - `httpClient.get(net, "https://example.com")`
- audit:
  - includes `PUBLIC_EGRESS_NO_DOMAIN_POLICY` with `httpClient.get(...)` sample callsites.

#### 7) Tradeoffs and next steps
- v0 rule checks only list presence, not domain quality/coverage.
- next step is optional policy quality checks (for example minimum allowlist specificity and explicit deny rules for high-risk domains).

### Slice Explanation: CSRF Protected-Method Coverage Finding (`CSRF_PROTECTED_METHODS_INCOMPLETE`)

#### 1) What it is
This slice adds a deterministic posture finding when CSRF protection is enabled but does not cover the full default unsafe-method set.

#### 2) Why it exists
Partial method protection (`POST` only, for example) creates easy bypass paths through other state-mutating methods (`PUT`, `PATCH`, `DELETE`). The audit now calls that out explicitly.

#### 3) How it works internally
- `sec.audit` computes normalized uppercase method coverage from `policy.csrf.protected_methods`.
- required baseline set is fixed in v0:
  - `POST`
  - `PUT`
  - `PATCH`
  - `DELETE`
- when any required methods are missing and `csrf.enabled=true`, audit emits:
  - `CSRF_PROTECTED_METHODS_INCOMPLETE` (LOW).
- evidence includes:
  - `missingMethods`,
  - configured `protectedMethods`,
  - bounded middleware `sampleCalls` from `middleware.csrf`/`middleware.auth`.

#### 4) Inputs/outputs and constraints
- Input:
  - effective CSRF policy (`enabled`, `protected_methods`),
  - middleware tags from `security_map`.
- Output:
  - low-severity posture finding for incomplete protected-method coverage.
- Constraints:
  - rule is inactive when `csrf.enabled=false` (that path is already covered by `CSRF_REQUIRED_BUT_DISABLED` when auth mode requires CSRF).

#### 5) Failure modes and diagnostics
- no parser/typechecker diagnostics were added by this slice.
- finding quality depends on policy key quality; unknown/invalid method names are treated as non-matching and increase missing coverage.

#### 6) Example usage
- policy:
  - `[csrf] enabled = true`
  - `protected_methods = ["POST"]`
- audit:
  - emits `CSRF_PROTECTED_METHODS_INCOMPLETE` with `missingMethods = ["PUT", "PATCH", "DELETE"]`.

#### 7) Tradeoffs and next steps
- v0 rule uses a fixed baseline method set and does not yet model per-route custom method posture.
- next step is route-aware CSRF posture checks tied to normalized router method metadata.

### Slice Explanation: Filesystem Symlink Posture Finding (`SYMLINK_POLICY_WEAK`)

#### 1) What it is
This slice adds typed-policy support for filesystem symlink posture and a deterministic audit finding when symlink restrictions are not enforced.

#### 2) Why it exists
`fs.enabled=true` without strong symlink policy can allow traversal bypasses even with base-path controls. The audit now highlights weaker modes explicitly.

#### 3) How it works internally
- policy model now persists:
  - `fs.forbid_symlinks` with accepted values `off|warn|enforce`.
- parser validation now rejects invalid values with `P6003`.
- `sec.audit` emits `SYMLINK_POLICY_WEAK` when:
  - `fs.enabled=true`, and
  - `fs.forbid_symlinks != "enforce"`.
- evidence includes:
  - effective `forbidSymlinks` value,
  - bounded `sampleCalls` from `sink.fs.read`/`sink.fs.write`.

#### 4) Inputs/outputs and constraints
- Input:
  - effective filesystem policy (`enabled`, `forbid_symlinks`),
  - filesystem sink tags from `security_map`.
- Output:
  - `SYMLINK_POLICY_WEAK` finding with severity `MEDIUM`.
- Constraints:
  - finding is posture-level and independent of specific path normalization implementation details.

#### 5) Failure modes and diagnostics
- invalid policy value emits parser diagnostic:
  - `P6003 invalid fs.forbid_symlinks`.
- with valid policy, this is audit-only and produces deterministic posture evidence.

#### 6) Example usage
- policy:
  - `[fs] enabled = true`
  - `forbid_symlinks = "warn"`
- audit:
  - emits `SYMLINK_POLICY_WEAK` with filesystem call samples when present.

#### 7) Tradeoffs and next steps
- v0 checks policy posture, not runtime symlink resolution behavior.
- next step is runtime-level fs path canonicalization tests that tie directly to this policy signal.

### Slice Explanation: Weak Referrer Policy Finding (`REFERRER_POLICY_WEAK`)

#### 1) What it is
This slice adds a low-severity security-headers posture finding when an explicitly weak referrer policy is configured.

#### 2) Why it exists
Referrer policy is part of baseline response-hardening posture. Weak values can leak more request-origin information than needed.

#### 3) How it works internally
- `sec.audit` now evaluates `policy.security_headers.referrer_policy` when security headers are enabled.
- weak policy set in v0:
  - `no-referrer-when-downgrade`
  - `unsafe-url`
  - `origin`
  - `origin-when-cross-origin`
- if matched, audit emits:
  - `REFERRER_POLICY_WEAK` (LOW).
- evidence includes:
  - effective `referrerPolicy`,
  - bounded middleware `sampleCalls` from `middleware.security_headers`.

#### 4) Inputs/outputs and constraints
- Input:
  - effective referrer policy value,
  - security-headers middleware call tags.
- Output:
  - low-severity posture finding with deterministic evidence.
- Constraints:
  - rule only applies when security headers middleware posture is enabled.

#### 5) Failure modes and diagnostics
- this is an audit-only posture signal; no parser/typechecker diagnostics are changed.
- unknown policies are not automatically classified as weak in v0 unless they match the explicit weak set.

#### 6) Example usage
- policy:
  - `[security_headers] referrer_policy = "unsafe-url"`
- audit:
  - emits `REFERRER_POLICY_WEAK` with security-headers middleware sample calls.

#### 7) Tradeoffs and next steps
- v0 uses a fixed weak-policy set and does not yet differentiate environment-specific acceptable values.
- next step is environment-aware posture mapping for referrer policy strictness.

## Core architecture
- `policy` remains source of truth for effective security posture and validation.
- `security_map` is generated statically from parsed program calls + policy-derived middleware attrs.
- `sec.audit` consumes policy + `security_map` and computes findings via a deterministic rule table.

## Files added/updated
- `compiler/ailang-core/src/security_map.rs`
- `compiler/ailang-core/src/audit.rs`
- `compiler/ailang-core/src/policy.rs` (expanded policy fields + validation for security_headers/csrf/auth coupling)
- `compiler/ailang-core/src/lib.rs` (module exports + `write_security_map`)
- `compiler/ailang-cli/src/main.rs` (new `sec audit` command)

## Tests added
- `compiler/ailang-core/tests/security_map.rs`
- `compiler/ailang-core/tests/sec_audit.rs`
- `compiler/ailang-core/tests/diagnostic_tags.rs`
- `compiler/ailang-cli/tests/json_output.rs`
- extended `compiler/ailang-core/tests/policy.rs` for csrf/auth/cors validations

## Current limitations
- middleware detection currently relies on known callable names (dynamic dispatch and indirect call targets are not yet mapped).
- finding set is intentionally baseline-focused and will expand in M4/M8.

## Next implementation steps
1. Extend callable/member canonicalization beyond built-in capability namespace seeding into richer user-defined callable object field shapes.
2. Add richer SQL hygiene parsing (full query normalization/AST) for robust handling beyond keyword heuristics.
3. Extend typed schema enforcement beyond `res.json` into broader encode/decode stdlib paths.
4. Surface richer provenance trace chains from audit/security-map metadata into editor tooling outputs (hover/code actions/LSP) beyond compact compiler notes.
5. Add opt-in rolling audit-history capture (timestamped reports) and windowed trend summarization on top of persisted report artifacts.
